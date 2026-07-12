#!/usr/bin/env python3
"""jira-tracking-pack — real parser: docs/jira/v26.7.11/tickets/index.md -> instances.ttl.

This is a REAL parse of the human-maintained markdown ticket table, not a
hand-transcribed snapshot: rerun this script any time the source table
changes and instances.ttl (and the ontology.ttl union built from it) tracks
the real file, never goes stale on its own. Reuses the same block-boundary
technique `scripts/verifier_report.py`'s `parse_ticket_statuses()` already
established for this exact file (row_start regex bounding each ticket's
markdown "row", which in this document spans multiple physical lines —
remediation paragraphs appended below the initial declaration, not standard
single-line GFM rows) and extends it to pull every structured column
(name/rail/dependencies/status), not just status.

Parsing strategy (see extract_ticket_rows() docstring for the exact
algorithm and its documented limitations).

Run: `python3 packs/jira-tracking-pack/make-ontology.py`, or via
`just jira-tracking-generate` (also runs `ggen sync run` afterward).
Writes:
  - packs/jira-tracking-pack/instances.ttl   (GENERATED ticket instance data)
  - packs/jira-tracking-pack/ontology.ttl    (GENERATED union: schema.ttl + instances.ttl)
  - crates/cng/src/jira-data.ttl             (GENERATED copy of ontology.ttl,
    compiled into the `cng` binary via `include_str!` so the CLI needs no
    runtime data-file path)
"""

from __future__ import annotations

import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
PACK_ROOT = Path(__file__).resolve().parent
TICKETS_INDEX = REPO_ROOT / "docs/jira/v26.7.11/tickets/index.md"
SCHEMA_TTL = PACK_ROOT / "schema.ttl"
INSTANCES_TTL = PACK_ROOT / "instances.ttl"
ONTOLOGY_TTL = PACK_ROOT / "ontology.ttl"
CNG_DATA_COPY = REPO_ROOT / "crates/cng/src/jira-data.ttl"

JIRA_NS = "http://seanchatmangpt.github.io/packs/jira-tracking#"

ROW_START_RE = re.compile(r"^\|\s*(\d+)\s*\|", re.MULTILINE)
HEADING_RE = re.compile(r"^#{1,6}\s", re.MULTILINE)
MD_LINK_RE = re.compile(r"\[(.*?)\]\([^)]*\)")
DEP_TOKEN_RE = re.compile(r"\b(\d{3})(?:-(\d{3}))?\b")

# Every status token this pack's closed vocabulary can currently express,
# checked out first-match against the STATUS cell's own text (not the whole
# evidence blob) so a status cell like "PARTIAL (7/10 ALIVE, 3/10 correctly
# BLOCKED...)" resolves to PARTIAL (the cell's own leading claim), not one
# of the parenthetical numbers it mentions in passing.
STATUS_KEYWORDS = ["ALIVE", "PARTIAL", "BLOCKED", "PLANNED", "OVERCLAIMED", "OPEN", "DONE"]
STATUS_KEYWORD_RE = re.compile(r"\b(" + "|".join(STATUS_KEYWORDS) + r")\b")

# Closed-vocabulary normalization (schema.ttl's 5 jira:Status individuals).
# OPEN -> Planned and DONE -> Alive are documented heuristics (schema.ttl's
# jira:Planned comment); every other keyword maps to its literal namesake.
STATUS_MAP = {
    "ALIVE": "Alive",
    "PARTIAL": "Partial",
    "BLOCKED": "Blocked",
    "PLANNED": "Planned",
    "OVERCLAIMED": "Overclaimed",
    "OPEN": "Planned",
    "DONE": "Alive",
}


def turtle_escape(text: str) -> str:
    """Escapes a Python string for a single-line Turtle `"..."` literal.

    Newlines/tabs/carriage returns are collapsed to a space first (the
    caller has already joined a multi-line block on spaces, but this stays
    defensive against any that slip through), then backslash and double
    quote are escaped per the Turtle STRING_LITERAL_QUOTE grammar.
    """
    text = text.replace("\r", " ").replace("\n", " ").replace("\t", " ")
    text = text.replace("\\", "\\\\").replace('"', '\\"')
    return text


def strip_md_link(name: str) -> str:
    """`[Text](url)` -> `Text`; names with no link syntax pass through."""
    return MD_LINK_RE.sub(lambda m: m.group(1), name).strip()


def extract_deps(deps_cell: str) -> list[str]:
    """Extracts dependency ticket ids from a free-text Dependencies cell.

    The source table's Dependencies column is not a clean list: it mixes
    bare ids ("750"), comma lists ("755, 756"), ranges ("750-754"), and
    prose ("Rail A/B (750-754), Rail C (755, 756)"). This extracts every
    3-digit token (this milestone's ids are all in 700-799) and expands
    NNN-NNN ranges inclusively; prose words like "Rail" carry no numeric
    tokens and are dropped by construction, not specially matched.
    Returns a sorted, de-duplicated list of zero-padded-3-digit id strings.
    """
    if deps_cell.strip() in ("", "—", "-", "--"):
        return []
    ids: set[int] = set()
    for m in DEP_TOKEN_RE.finditer(deps_cell):
        lo = int(m.group(1))
        hi = int(m.group(2)) if m.group(2) else lo
        if hi < lo:
            lo, hi = hi, lo
        for n in range(lo, hi + 1):
            ids.add(n)
    return [str(n) for n in sorted(ids)]


def extract_ticket_rows(text: str) -> list[dict]:
    """Real parse of the ticket table(s) in `text`.

    Algorithm:
      1. Find every ticket row boundary (`^\\|\\s*(digits)\\s*\\|`) AND every
         markdown heading boundary in the whole document, merge and sort by
         position.
      2. For each row boundary, the ticket's block is the text from that
         boundary up to the NEXT boundary of either kind — this is the key
         correction over a naive "next row only" bound: without also
         stopping at headings, the LAST ticket in a table absorbs the
         following `##`/`###` section's prose (and, worse, the NEXT table's
         own header/separator row), corrupting the tail-end field split.
         Verified empirically against this file: tickets 770/796/798 (each
         immediately followed by a `##`/`###` heading) parse correctly only
         with this heading-aware bound.
      3. Join the block's physical lines with single spaces (multi-paragraph
         remediation notes become one logical row, matching how this table
         is actually authored — see module docstring).
      4. Split the joined text into exactly 6 logical columns without being
         confused by any stray literal "|" that might appear inside the
         free-text Scope/evidence column: split the FIRST 3 columns
         (id/title/section) from the left (`split("|", 3)`) and the LAST 2
         columns (dependencies/status) from the right (`rsplit("|", 2)`) of
         the remaining text — whatever pipe characters are left in the
         middle, however many, all belong to the evidence column.

    Known limitation, disclosed not hidden: this assumes the title and
    section columns themselves never contain a literal "|" (true for every
    row in the current file, checked by hand during development) and that
    every ticket block's true terminal cell is its own last two "|"-bounded
    segments (also true for all 49 current rows — zero short-row/short-back
    fallback warnings printed during development).
    """
    boundaries: list[tuple[int, str, str | None]] = []
    for m in ROW_START_RE.finditer(text):
        boundaries.append((m.start(), "row", m.group(1)))
    for m in HEADING_RE.finditer(text):
        boundaries.append((m.start(), "heading", None))
    boundaries.sort(key=lambda t: t[0])

    rows: list[dict] = []
    for idx, (pos, kind, tid) in enumerate(boundaries):
        if kind != "row":
            continue
        block_end = len(text)
        if idx + 1 < len(boundaries):
            block_end = boundaries[idx + 1][0]
        block = text[pos:block_end]
        joined = " ".join(line.strip() for line in block.splitlines() if line.strip())
        if joined.endswith("|"):
            joined = joined[:-1].rstrip()
        if joined.startswith("|"):
            joined = joined[1:].lstrip()

        front = joined.split("|", 3)
        if len(front) < 4:
            print(f"WARN: ticket {tid}: row too short to hold id/title/section/rest, skipping")
            continue
        fid, raw_title, raw_section, rest = (p.strip() for p in front)

        back = rest.rsplit("|", 2)
        if len(back) < 3:
            print(f"WARN: ticket {tid}: row too short to hold evidence/deps/status, skipping")
            continue
        raw_evidence, raw_deps, raw_status_cell = (p.strip() for p in back)

        kw = STATUS_KEYWORD_RE.search(raw_status_cell)
        if not kw:
            print(f"WARN: ticket {tid}: no recognized status keyword in {raw_status_cell!r}, skipping")
            continue

        rows.append(
            {
                "id": fid,
                "title": strip_md_link(raw_title),
                "section": raw_section,
                "deps": extract_deps(raw_deps),
                "status": STATUS_MAP[kw.group(1)],
                "evidence": raw_evidence,
            }
        )
    return rows


def render_instances_ttl(rows: list[dict]) -> str:
    lines = [
        "# GENERATED — do not edit. Source: docs/jira/v26.7.11/tickets/index.md",
        "# Regenerate: packs/jira-tracking-pack/make-ontology.py",
        f"# {len(rows)} jira:Ticket individuals, real parse (not hand-transcribed).",
        "@prefix jira: <" + JIRA_NS + "> .",
        "",
    ]
    for row in rows:
        iri = f"jira:PROJ{row['id']}"
        lines.append(f"{iri}")
        lines.append(f'    a jira:Ticket ;')
        lines.append(f'    jira:id "{turtle_escape(row["id"])}" ;')
        lines.append(f'    jira:title "{turtle_escape(row["title"])}" ;')
        lines.append(f'    jira:section "{turtle_escape(row["section"])}" ;')
        lines.append(f"    jira:status jira:{row['status']} ;")
        for dep in row["deps"]:
            lines.append(f"    jira:dependsOn jira:PROJ{dep} ;")
        lines.append(f'    jira:evidence "{turtle_escape(row["evidence"])}" .')
        lines.append("")
    return "\n".join(lines) + "\n"


def main() -> int:
    text = TICKETS_INDEX.read_text()
    rows = extract_ticket_rows(text)
    if len(rows) < 40:
        raise SystemExit(
            f"FATAL: parsed only {len(rows)} tickets from {TICKETS_INDEX} "
            "(expected ~49) — parser likely broken, refusing to write a "
            "truncated instances.ttl. Not silently emitting a partial file."
        )

    instances_ttl = render_instances_ttl(rows)
    INSTANCES_TTL.write_text(instances_ttl)

    schema_ttl = SCHEMA_TTL.read_text()
    union = (
        "# GENERATED union — do not edit. Sources:\n"
        "#   packs/jira-tracking-pack/schema.ttl   (hand-authored vocabulary)\n"
        "#   packs/jira-tracking-pack/instances.ttl (generated ticket data, see below)\n"
        "# Regenerate: packs/jira-tracking-pack/make-ontology.py\n"
        + schema_ttl
        + "\n"
        + instances_ttl
    )
    ONTOLOGY_TTL.write_text(union)
    CNG_DATA_COPY.write_text(
        "# GENERATED — do not edit. Compiled into the `cng` binary via\n"
        "# include_str! (crates/cng/src/jira.rs) so the CLI needs no runtime\n"
        "# data-file path. Byte-identical in content to\n"
        "# packs/jira-tracking-pack/ontology.ttl (this pack's own source of\n"
        "# truth); regenerate both together via\n"
        "# packs/jira-tracking-pack/make-ontology.py.\n"
        + schema_ttl
        + "\n"
        + instances_ttl
    )

    by_status: dict[str, int] = {}
    for row in rows:
        by_status[row["status"]] = by_status.get(row["status"], 0) + 1
    print(f"parsed {len(rows)} tickets from {TICKETS_INDEX.relative_to(REPO_ROOT)}")
    for status, count in sorted(by_status.items()):
        print(f"  {status}: {count}")
    print(f"wrote {INSTANCES_TTL.relative_to(REPO_ROOT)}")
    print(f"wrote {ONTOLOGY_TTL.relative_to(REPO_ROOT)}")
    print(f"wrote {CNG_DATA_COPY.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
