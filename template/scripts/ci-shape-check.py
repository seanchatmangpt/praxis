#!/usr/bin/env python3
"""CI shape checker — verifies that all required_jobs in cicd.toml are
present as job IDs in .github/workflows/ci.yml.

Exit codes:
  0  All required jobs found.
  1  One or more required jobs missing from ci.yml.
  2  A required input file was not found or could not be parsed.

Usage (from repo root):
  python3 scripts/ci-shape-check.py
  python3 scripts/ci-shape-check.py --cicd cicd.toml --workflow .github/workflows/ci.yml
"""

import sys
import argparse
import re
from pathlib import Path

# tomllib is stdlib in Python 3.11+; fall back to tomli for older interpreters.
try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ImportError:
        sys.exit(
            "ERROR: Python 3.11+ required (tomllib is stdlib), "
            "or install tomli: pip install tomli"
        )

# ANSI colour helpers — skipped when stdout is not a TTY.
def _colour(code: str, text: str) -> str:
    if sys.stdout.isatty():
        return f"\033[{code}m{text}\033[0m"
    return text

GREEN  = lambda t: _colour("32", t)
RED    = lambda t: _colour("31", t)
YELLOW = lambda t: _colour("33", t)
BOLD   = lambda t: _colour("1",  t)
DIM    = lambda t: _colour("2",  t)

PASS_LABEL = "[PASS]"
FAIL_LABEL = "[FAIL]"
WARN_LABEL = "[WARN]"


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument(
        "--cicd",
        default="cicd.toml",
        help="Path to cicd.toml (default: cicd.toml)",
    )
    p.add_argument(
        "--workflow",
        default=".github/workflows/ci.yml",
        help="Path to ci.yml (default: .github/workflows/ci.yml)",
    )
    p.add_argument(
        "--verbose",
        action="store_true",
        help="Print all jobs found in ci.yml, not just missing ones",
    )
    return p.parse_args()


def load_cicd(path: Path) -> dict:
    """Parse cicd.toml and return the full document as a dict."""
    if not path.exists():
        print(f"ERROR: cicd.toml not found at {path}", file=sys.stderr)
        sys.exit(2)
    try:
        with path.open("rb") as fh:
            return tomllib.load(fh)
    except Exception as exc:
        print(f"ERROR: failed to parse {path}: {exc}", file=sys.stderr)
        sys.exit(2)


def extract_yml_jobs(path: Path) -> set:
    """Extract top-level job IDs from a GitHub Actions workflow YAML file.

    We parse the YAML with a simple line-oriented approach rather than pulling
    in PyYAML (which is not guaranteed on all runner images). Job IDs appear
    as keys at two-space indent directly under the top-level `jobs:` mapping.
    GitHub Actions job IDs must match: [a-zA-Z_][a-zA-Z0-9_-]*
    """
    if not path.exists():
        print(f"ERROR: workflow file not found at {path}", file=sys.stderr)
        sys.exit(2)

    text = path.read_text(encoding="utf-8")

    in_jobs = False
    jobs = set()
    for line in text.splitlines():
        # Detect the top-level `jobs:` key (zero indent).
        if re.match(r"^jobs\s*:", line):
            in_jobs = True
            continue
        if not in_jobs:
            continue
        # A new top-level key resets context.
        if re.match(r"^[a-zA-Z]", line):
            in_jobs = False
            continue
        # Job IDs: exactly two leading spaces, identifier, colon.
        m = re.match(r"^  ([a-zA-Z_][a-zA-Z0-9_-]*):", line)
        if m:
            jobs.add(m.group(1))
    return jobs


def main() -> int:
    args = parse_args()
    cicd_path = Path(args.cicd)
    workflow_path = Path(args.workflow)

    # ── Load cicd.toml ──────────────────────────────────────────────────────
    cicd = load_cicd(cicd_path)
    ci_section = cicd.get("ci", {})
    required_jobs = ci_section.get("required_jobs", [])
    advisory_jobs = ci_section.get("advisory_jobs", [])
    flaky = ci_section.get("flaky", [])
    known_failing = ci_section.get("known_failing", [])

    if not required_jobs:
        print(f"{WARN_LABEL} cicd.toml [ci].required_jobs is empty — nothing to check.")
        return 0

    # ── Load ci.yml ─────────────────────────────────────────────────────────
    yml_jobs = extract_yml_jobs(workflow_path)

    # ── Compare ─────────────────────────────────────────────────────────────
    missing = [j for j in required_jobs if j not in yml_jobs]
    advisory_missing = [j for j in advisory_jobs if j not in yml_jobs]

    # ── Report ──────────────────────────────────────────────────────────────
    print()
    print(BOLD("CI Shape Check"))
    print(BOLD("══════════════"))
    print(f"  cicd.toml  : {cicd_path}")
    print(f"  ci.yml     : {workflow_path}")
    print()

    print(BOLD("Required jobs:"))
    for job in required_jobs:
        if job in yml_jobs:
            mark = GREEN("  ✓")
            print(f"{mark} {job}")
        else:
            mark = RED("  ✗")
            print(f"{mark} {job}  {DIM('<-- MISSING in ci.yml')}")

    if args.verbose and yml_jobs:
        print()
        print(BOLD("All jobs found in ci.yml:"))
        for job in sorted(yml_jobs):
            if job in required_jobs:
                tag = "R"
            elif job in advisory_jobs:
                tag = "A"
            else:
                tag = " "
            print(f"  [{tag}] {job}")
        print(DIM("       R=required  A=advisory"))

    if advisory_missing:
        print()
        print(BOLD("Advisory jobs (informational — not blocking):"))
        for job in advisory_missing:
            print(
                f"  {YELLOW(WARN_LABEL)} {job} listed in [ci].advisory_jobs "
                f"but not found in ci.yml"
            )

    if flaky:
        print()
        print(BOLD("Flaky jobs (tracked, not blocking):") + "  " + ", ".join(flaky))

    if known_failing:
        print()
        print(BOLD("Known failing jobs:") + "  " + ", ".join(known_failing))

    # ── Result ──────────────────────────────────────────────────────────────
    print()
    if missing:
        count = len(missing)
        noun = "job" if count == 1 else "jobs"
        print(
            f"{RED(FAIL_LABEL)} {count} required {noun} missing from ci.yml: "
            + ", ".join(missing)
        )
        print()
        print(
            DIM(
                "  Fix: add the missing job(s) to .github/workflows/ci.yml\n"
                "       and include them in the `needs` list of the ci-success gate."
            )
        )
        print()
        return 1
    else:
        print(
            f"{GREEN(PASS_LABEL)} All {len(required_jobs)} required jobs are "
            f"present in ci.yml."
        )
        print()
        return 0


if __name__ == "__main__":
    sys.exit(main())
