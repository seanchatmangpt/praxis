#!/usr/bin/env python3
"""rigor-gap-scanner — finds code that LOOKS complete but isn't wired to anything real.

Three checks, all heuristic (regex/line-based, not a real Rust parser — see LIMITATIONS
at the bottom of this file). Each check targets one specific failure mode found by hand
this session, so the point of this tool is to make that hand-verification mechanical and
repeatable, not to replace judgment on anything it flags.

  1. orphaned-module   — a .rs file that exists on disk but is never reached via any
                          `mod` declaration walked from the crate root. Compiles clean
                          (Rust just silently excludes it), has its own passing tests if
                          run directly, and is functionally dead. This is exactly what
                          PROJ-777/778 turned out to be this session: real, tested code,
                          never `mod`-declared in lib.rs, therefore never part of the
                          binary at all.

  2. zero-caller-pub-fn — a `pub fn` whose only occurrences in the whole tree are its own
                          definition and test code (#[cfg(test)] blocks, tests/ dir,
                          *_test.rs files). Real and tested, unreachable from any
                          production entry point. This is the "island" pattern the
                          Fortune-5 audit found repeatedly this session (graphlaw_authority
                          before its mod-fix, the cng OTel->OCEL->receipt chain, closure.rs/
                          compensation.rs).

  3. claim-without-mechanism — a doc comment using strong enforcement/verification
                          language ("statically reject", "enforce", "verified", "proven",
                          "guarantee", "mechanically enforced") directly above a function
                          whose body has no branching or comparison logic at all (no
                          if/match/==/!=/</> , and/or an unconditional Ok(...)/return as
                          the effective whole body). The doc comment asserts a check is
                          happening; the body shows nothing is. This is the
                          SubKolmogorovBound pattern found in ~/mfact this session:
                          "the Rust compiler's borrow checker statically rejects..." above
                          a function that discards its input and returns unconditionally.

Usage:
  python3 scan.py <path-to-crate-or-repo-root> [--json]

Exits 0 if nothing is flagged, 1 if anything is flagged (so it can gate a `just` recipe).

Full-workspace scan (`scan.py .`) takes ~14s as of this writing (179 findings, 28 crates).
Earlier versions hung past 90s on this repo: check_zero_caller_pub_fns recomputed
strip_test_cfg_blocks per (function, other-file) pair instead of once per file, and the
monorepo root's own stray src/ (an unrelated `my-conforming-project` boilerplate package
declared in the root Cargo.toml) tripped the single-crate detection, making the tool
rescan the entire ~220MB repo -- including vendors/ (188MB of third-party vendored
source) -- as one undifferentiated crate. Both are fixed; see inline comments at the
fix sites for the reasoning.
"""

import argparse
import json
import re
import sys
from pathlib import Path

TEST_DIR_MARKERS = ("/tests/", "/test/")
TEST_FILE_SUFFIXES = ("_test.rs", "_tests.rs")

CLAIM_WORDS = re.compile(
    r"\b(statically reject|mechanically enforc|is enforced|cryptographically verif"
    r"|verified|is proven|guarantee[sd]?|validated at compile.?time"
    r"|compiler.{0,20}reject|borrow checker.{0,20}reject)\b",
    re.IGNORECASE,
)

BRANCH_TOKENS = re.compile(r"\b(if|match|while|for)\b|[=!<>]=|[<>]")


def is_test_path(path: Path) -> bool:
    p = str(path)
    if any(m in p for m in TEST_DIR_MARKERS):
        return True
    if path.name.endswith(TEST_FILE_SUFFIXES):
        return True
    return False


def strip_test_cfg_blocks(text: str) -> str:
    """Remove #[cfg(test)] ... matching-brace blocks so Check 2 doesn't count test-only
    callers as production callers. Heuristic brace matching, not a real parser."""
    out = []
    i = 0
    n = len(text)
    cfg_re = re.compile(r"#\[cfg\(test\)\]")
    while i < n:
        m = cfg_re.search(text, i)
        if not m:
            out.append(text[i:])
            break
        out.append(text[i : m.start()])
        j = text.find("{", m.end())
        if j == -1:
            i = m.end()
            continue
        depth = 1
        k = j + 1
        while k < n and depth > 0:
            if text[k] == "{":
                depth += 1
            elif text[k] == "}":
                depth -= 1
            k += 1
        i = k
    return "".join(out)


EXCLUDED_PATH_MARKERS = ("/target/", "/.git/", "/vendors/", "/vendor/")


def find_rust_files(root: Path):
    return [p for p in root.rglob("*.rs") if not any(m in str(p) for m in EXCLUDED_PATH_MARKERS)]


def check_orphaned_modules(crate_root: Path, results: list):
    """Walk mod declarations from lib.rs/main.rs; report .rs files under src/ that are
    never reached. Handles `mod x;`, `pub mod x;`, and a simple `#[path = "..."]`
    override immediately before a mod declaration. Does not resolve mod declarations
    nested inside an inline `mod x { ... }` block deeper than one level of indirection
    from a file it already visited -- see LIMITATIONS."""
    src = crate_root / "src"
    if not src.is_dir():
        return
    entry = None
    for candidate in ("lib.rs", "main.rs"):
        if (src / candidate).is_file():
            entry = src / candidate
            break
    if entry is None:
        return

    mod_re = re.compile(r'(?:#\[path\s*=\s*"([^"]+)"\]\s*)?(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*[;{]')

    visited = set()
    queue = [entry]
    while queue:
        f = queue.pop()
        if f in visited or not f.is_file():
            continue
        visited.add(f)
        try:
            text = f.read_text(errors="replace")
        except OSError:
            continue
        base_dir = f.parent if f.name in ("mod.rs", "lib.rs", "main.rs") else f.parent / f.stem
        for path_override, name in mod_re.findall(text):
            if path_override:
                candidate = f.parent / path_override
                if candidate.is_file():
                    queue.append(candidate)
                continue
            c1 = base_dir / f"{name}.rs"
            c2 = base_dir / name / "mod.rs"
            if c1.is_file():
                queue.append(c1)
            elif c2.is_file():
                queue.append(c2)

    all_files = set(find_rust_files(src))
    orphaned = sorted(all_files - visited, key=str)
    for f in orphaned:
        results.append(
            {
                "check": "orphaned-module",
                "path": str(f.relative_to(crate_root)),
                "detail": "never reached via any `mod` declaration walked from the crate root "
                "(lib.rs/main.rs) -- present on disk, absent from the compiled binary",
            }
        )


def check_zero_caller_pub_fns(crate_root: Path, results: list):
    fn_re = re.compile(r"^\s*(?:///[^\n]*\n\s*)*pub\s+fn\s+(\w+)", re.MULTILINE)
    files = [f for f in find_rust_files(crate_root) if not is_test_path(f)]
    file_texts = {}
    for f in files:
        try:
            file_texts[f] = f.read_text(errors="replace")
        except OSError:
            continue

    # Precompute each file's production-only text ONCE. Previously this was recomputed
    # (via strip_test_cfg_blocks, an O(len(text)) brace-matching pass) for every single
    # (function, other-file) pair -- O(functions x files x file-length) -- which hung for
    # 3+ minutes on this workspace's larger crates. Precomputing drops it to O(files).
    stripped_texts = {f2: strip_test_cfg_blocks(t2) for f2, t2 in file_texts.items()}

    all_text_for_test_dirs = ""
    for f in find_rust_files(crate_root):
        if is_test_path(f):
            try:
                all_text_for_test_dirs += f.read_text(errors="replace")
            except OSError:
                pass

    for f, text in file_texts.items():
        for m in fn_re.finditer(text):
            name = m.group(1)
            if name in ("new", "default", "main", "fmt", "clone", "from", "into"):
                continue  # trait-method-shaped names, too noisy to be useful signal
            # `(?<!fn )` excludes the definition's own `fn name(` from matching itself as
            # a call -- relies on cargo-fmt's single-space `fn name` convention (enforced
            # repo-wide), which is cheaper and correctness-preserving unlike the previous
            # before/after-def-line split, which sliced the *stripped* text using byte
            # offsets computed against the *original* text -- wrong whenever a
            # #[cfg(test)] block preceded the function in the same file.
            call_re = re.compile(r"(?<!fn )\b" + re.escape(name) + r"\s*\(")
            prod_calls = 0
            for stripped in stripped_texts.values():
                prod_calls += len(call_re.findall(stripped))
                if prod_calls > 0:
                    break
            if prod_calls > 0:
                continue
            test_calls = len(call_re.findall(all_text_for_test_dirs))
            if test_calls == 0:
                continue  # not even test-exercised; a different (likely fine) situation, skip
            results.append(
                {
                    "check": "zero-caller-pub-fn",
                    "path": str(f.relative_to(crate_root)),
                    "detail": f"`pub fn {name}` has {test_calls} test-only caller(s) and zero "
                    "production callers anywhere else in this tree",
                }
            )


def check_claim_without_mechanism(crate_root: Path, results: list):
    doc_fn_re = re.compile(
        r"((?:^[ \t]*///.*\n)+)^[ \t]*(?:pub\s+)?fn\s+(\w+)[^{]*\{", re.MULTILINE
    )
    for f in find_rust_files(crate_root):
        if is_test_path(f):
            continue
        try:
            text = f.read_text(errors="replace")
        except OSError:
            continue
        for m in doc_fn_re.finditer(text):
            doc, name = m.group(1), m.group(2)
            if not CLAIM_WORDS.search(doc):
                continue
            body_start = m.end()
            depth = 1
            i = body_start
            n = len(text)
            while i < n and depth > 0:
                if text[i] == "{":
                    depth += 1
                elif text[i] == "}":
                    depth -= 1
                i += 1
            body = text[body_start:i]
            has_branching = bool(BRANCH_TOKENS.search(body))
            is_short = len(body.strip().splitlines()) <= 4
            if not has_branching:
                results.append(
                    {
                        "check": "claim-without-mechanism",
                        "path": str(f.relative_to(crate_root)),
                        "detail": f"`fn {name}` doc comment uses enforcement/verification "
                        f"language but the body has no if/match/while/for or comparison "
                        f"operator anywhere ({'short body, ' if is_short else ''}nothing is "
                        f"actually checked)",
                    }
                )


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("root", type=Path, help="crate or repo root to scan (must contain src/)")
    ap.add_argument("--json", action="store_true", help="emit JSON instead of text")
    args = ap.parse_args()

    root = args.root.resolve()
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        sys.exit(2)

    results = []
    # If the target itself has src/, treat it as one crate. Otherwise scan every
    # immediate crate-shaped subdirectory (has its own src/) one level down, plus
    # any Cargo workspace member globs under crates/ and apps/ -- kept simple on
    # purpose; see LIMITATIONS.
    crate_roots = []
    # `root` is only its own crate when it's genuinely a single crate (e.g. invoked as
    # `scan.py crates/praxis-graphlaw`). A monorepo root can *also* have its own stray
    # src/ (this repo's root Cargo.toml declares an unrelated `my-conforming-project`
    # boilerplate package) -- treating that as a crate_root when scanning the whole
    # workspace makes find_rust_files(crate_root) walk the ENTIRE repo as "this crate's
    # source", redundantly rescanning every real crate under crates/*, apps/*, template*,
    # playground/, etc. as one undifferentiated blob (misattributing cross-crate calls,
    # and inflating cost ~10x on top of the per-crate scans below). A monorepo root is
    # identified by having its own crates/ or apps/ directory.
    looks_like_monorepo_root = (root / "crates").is_dir() or (root / "apps").is_dir()
    if (root / "src").is_dir() and not looks_like_monorepo_root:
        crate_roots.append(root)
    for parent in ("crates", "apps", "."):
        base = root / parent if parent != "." else root
        if base.is_dir():
            for child in base.iterdir():
                if child.is_dir() and (child / "src").is_dir() and child not in crate_roots:
                    crate_roots.append(child)

    for cr in crate_roots:
        check_orphaned_modules(cr, results)
        check_zero_caller_pub_fns(cr, results)
        check_claim_without_mechanism(cr, results)

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        if not results:
            print(f"rigor-gap-scanner: 0 findings across {len(crate_roots)} crate(s) under {root}")
        else:
            by_check = {}
            for r in results:
                by_check.setdefault(r["check"], []).append(r)
            for check, items in sorted(by_check.items()):
                print(f"\n=== {check} ({len(items)}) ===")
                for it in items:
                    print(f"  {it['path']}: {it['detail']}")
            print(f"\nrigor-gap-scanner: {len(results)} finding(s) across {len(crate_roots)} crate(s)")

    sys.exit(1 if results else 0)


if __name__ == "__main__":
    main()

# LIMITATIONS (read before trusting a clean run):
# - Regex/line-based, not a real Rust AST parser. It can miss module resolution through
#   #[path] attributes with expressions, macro-generated mod declarations, or `mod` blocks
#   nested more than one level deep inside an inline `mod x { ... }` that itself isn't a
#   separate file. False negatives are more likely than false positives on check 1.
# - Check 2's "zero production callers" is purely textual (`name(`) -- it will miss calls
#   made only via a trait object, a function pointer stored in a struct field, or dynamic
#   dispatch, and it will false-flag any function whose name collides with an unrelated
#   identifier elsewhere. Always read the flagged file before concluding it's really dead.
# - Check 3's "no branching" heuristic will false-flag legitimately simple pure functions
#   that just don't need branches (a getter, a constant). The check only means something
#   in combination with a doc comment making a strong enforcement claim -- read both
#   together, not the flag alone.
# - This tool finds candidates for a human (or an adversarial-review agent) to look at.
#   It is not a proof of absence of the pattern, and a clean run is not itself a claim
#   that end-to-end wiring is correct -- exactly the trap this tool exists to catch,
#   applied to itself.
