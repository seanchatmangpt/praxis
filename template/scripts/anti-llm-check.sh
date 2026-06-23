#!/usr/bin/env bash
# anti-llm-check.sh — AI integrity gate for the praxis house style.
#
# Reads anti-llm.toml to discover human-only patterns, then checks whether
# recent AI-authored commits touched files containing those patterns.
#
# Exit codes:
#   0  No violations found (warnings may still be printed).
#   1  One or more patterns were modified by an AI-attributed commit.
#
# Usage (from repo root):
#   bash scripts/anti-llm-check.sh
#   bash scripts/anti-llm-check.sh --since HEAD~20
#   bash scripts/anti-llm-check.sh --warn-only
#
# Requirements: bash, git, python3 (stdlib only), grep

set -euo pipefail

# ── Defaults ────────────────────────────────────────────────────────────────

CICD_TOML="${ANTI_LLM_TOML:-anti-llm.toml}"
# Look back this many commits for AI-authored commits.  Override with --since.
SINCE="${ANTI_LLM_SINCE:-HEAD~50}"
# When --warn-only is passed, always exit 0 even if violations found.
WARN_ONLY=0

# ── Argument parsing ─────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --since)
            SINCE="$2"; shift 2 ;;
        --since=*)
            SINCE="${1#--since=}"; shift ;;
        --warn-only)
            WARN_ONLY=1; shift ;;
        --toml)
            CICD_TOML="$2"; shift 2 ;;
        -h|--help)
            sed -n '2,20p' "$0" | sed 's/^# \?//'
            exit 0 ;;
        *)
            echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

# ── Colour helpers ───────────────────────────────────────────────────────────

if [[ -t 1 ]]; then
    RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'
    BOLD=$'\033[1m'; DIM=$'\033[2m'; RESET=$'\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; BOLD=''; DIM=''; RESET=''
fi

pass()  { echo "${GREEN}[PASS]${RESET} $*"; }
fail()  { echo "${RED}[FAIL]${RESET} $*"; }
warn()  { echo "${YELLOW}[WARN]${RESET} $*"; }
info()  { echo "${DIM}      $*${RESET}"; }
title() { echo; echo "${BOLD}$*${RESET}"; }

# ── Helpers ──────────────────────────────────────────────────────────────────

# Parse [[rules.patterns]] blocks from anti-llm.toml using Python's tomllib.
# Outputs lines of: PATTERN_NAME<TAB>SEARCH_PATTERN<TAB>FILE_GLOBS<TAB>MESSAGE
parse_patterns() {
    python3 - "$CICD_TOML" <<'PYEOF'
import sys, re

try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        sys.exit("ERROR: Python 3.11+ required for tomllib (or install tomli)")

toml_path = sys.argv[1]
try:
    with open(toml_path, "rb") as fh:
        data = tomllib.load(fh)
except FileNotFoundError:
    # No anti-llm.toml → nothing to check
    sys.exit(0)
except Exception as e:
    print(f"ERROR parsing {toml_path}: {e}", file=sys.stderr)
    sys.exit(1)

patterns = data.get("rules", {}).get("patterns", [])
for p in patterns:
    name    = p.get("name", "unnamed")
    pattern = p.get("pattern", "")
    files   = p.get("files", ["**/*"])
    message = p.get("message", "Human review required.")
    # Encode as tab-separated; join file globs with comma
    print(f"{name}\t{pattern}\t{','.join(files)}\t{message}")
PYEOF
}

# Parse [exemptions] files list.
parse_exemptions() {
    python3 - "$CICD_TOML" <<'PYEOF'
import sys

try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        sys.exit(0)

toml_path = sys.argv[1]
try:
    with open(toml_path, "rb") as fh:
        data = tomllib.load(fh)
except Exception:
    sys.exit(0)

for f in data.get("exemptions", {}).get("files", []):
    print(f)
PYEOF
}

# ── Identify AI-authored commits ─────────────────────────────────────────────

title "Anti-LLM Integrity Check"
echo "${BOLD}══════════════════════════════${RESET}"
echo "  manifest  : $CICD_TOML"
echo "  scan-from : $SINCE"
echo "  warn-only : $([ $WARN_ONLY -eq 1 ] && echo yes || echo no)"

# git log can fail if SINCE is outside history; fall back gracefully.
AI_COMMITS=()
if ! AI_COMMIT_LINES=$(git log "${SINCE}..HEAD" \
        --format="%H %s" \
        --grep="Co-Authored-By: Claude\|Co-authored-by: Claude\|Co-Authored-By: Copilot\|Co-authored-by: Copilot\|Co-Authored-By: GPT\|Co-authored-by: GPT" 2>/dev/null); then
    AI_COMMIT_LINES=""
fi

# Also find commits where the author name looks like an AI tool.
if ! AI_AUTHOR_LINES=$(git log "${SINCE}..HEAD" \
        --format="%H %s" \
        --author="Claude\|Copilot\|GPT\|github-actions" 2>/dev/null); then
    AI_AUTHOR_LINES=""
fi

# Merge and deduplicate.
mapfile -t AI_COMMITS < <(printf '%s\n%s\n' "$AI_COMMIT_LINES" "$AI_AUTHOR_LINES" \
    | grep -v '^$' | sort -u)

echo
if [[ ${#AI_COMMITS[@]} -eq 0 ]]; then
    info "No AI-attributed commits found in range ${SINCE}..HEAD"
    info "(checked Co-Authored-By trailers and AI author names)"
    echo
    pass "No AI commits to audit."
    exit 0
fi

echo "${BOLD}AI-attributed commits (${#AI_COMMITS[@]}):${RESET}"
for commit_line in "${AI_COMMITS[@]}"; do
    sha="${commit_line%% *}"
    subject="${commit_line#* }"
    info "${sha:0:12}  ${subject}"
done

# ── Load patterns ─────────────────────────────────────────────────────────────

echo
echo "${BOLD}Human-only patterns (from $CICD_TOML):${RESET}"

mapfile -t RAW_PATTERNS < <(parse_patterns 2>/dev/null || true)

if [[ ${#RAW_PATTERNS[@]} -eq 0 ]]; then
    info "No [[rules.patterns]] found in $CICD_TOML — nothing to enforce."
    echo
    pass "No patterns configured."
    exit 0
fi

# Load exemptions.
mapfile -t EXEMPTIONS < <(parse_exemptions 2>/dev/null || true)

# Print pattern summary.
for raw in "${RAW_PATTERNS[@]}"; do
    IFS=$'\t' read -r name pattern files message <<< "$raw"
    info "[$name]  pattern: $(printf '%q' "$pattern")  files: $files"
done

# ── Check each AI commit for human-only pattern touches ──────────────────────

echo
echo "${BOLD}Scanning AI commits for human-only pattern touches:${RESET}"

VIOLATIONS=0
TOTAL_CHECKS=0

for commit_line in "${AI_COMMITS[@]}"; do
    sha="${commit_line%% *}"
    subject="${commit_line#* }"

    # Files changed in this commit.
    mapfile -t CHANGED_FILES < <(git diff-tree --no-commit-id -r --name-only "$sha" 2>/dev/null || true)

    for raw in "${RAW_PATTERNS[@]}"; do
        IFS=$'\t' read -r name pattern files message <<< "$raw"

        for changed_file in "${CHANGED_FILES[@]}"; do
            # Skip exempted files.
            is_exempt=0
            for exemption in "${EXEMPTIONS[@]}"; do
                # Simple glob match using case.
                # Convert glob wildcards to something testable with bash [[ ]].
                exempt_glob="${exemption//\*\*/DOUBLSTAR}"
                exempt_glob="${exempt_glob//\*/[^/]*}"
                exempt_glob="${exempt_glob//DOUBLSTAR/.*}"
                if [[ "$changed_file" =~ ^$exempt_glob$ ]]; then
                    is_exempt=1
                    break
                fi
            done
            [[ $is_exempt -eq 1 ]] && continue

            # Check if this changed file matches any of the pattern's file globs.
            IFS=',' read -ra file_globs <<< "$files"
            file_matches=0
            for glob in "${file_globs[@]}"; do
                glob="${glob// /}"  # trim spaces
                # Convert glob to regex for bash matching.
                glob_regex="${glob//\*\*/DOUBLSTAR}"
                glob_regex="${glob_regex//\*/[^/]*}"
                glob_regex="${glob_regex//DOUBLSTAR/.*}"
                if [[ "$changed_file" =~ ^${glob_regex}$ ]]; then
                    file_matches=1
                    break
                fi
            done
            [[ $file_matches -eq 0 ]] && continue

            # The file matches — now check if the changed content contains the pattern.
            TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
            if git show "${sha}:${changed_file}" 2>/dev/null \
                    | grep -qE "$pattern" 2>/dev/null; then
                VIOLATIONS=$((VIOLATIONS + 1))
                warn "Pattern [$name] found in AI commit ${sha:0:12}"
                info "  file   : $changed_file"
                info "  commit : $subject"
                info "  reason : $message"
                echo
            fi
        done
    done
done

# ── Summary ───────────────────────────────────────────────────────────────────

echo
echo "${BOLD}Summary${RESET}"
echo "${BOLD}═══════${RESET}"
info "AI commits scanned  : ${#AI_COMMITS[@]}"
info "Pattern checks run  : $TOTAL_CHECKS"
info "Violations detected : $VIOLATIONS"
echo

if [[ $VIOLATIONS -eq 0 ]]; then
    pass "No human-only patterns found in AI-authored commits."
    exit 0
fi

if [[ $WARN_ONLY -eq 1 ]]; then
    warn "$VIOLATIONS violation(s) detected — running in warn-only mode (exit 0)."
    warn "Remove --warn-only to enforce as a hard failure."
    exit 0
fi

fail "$VIOLATIONS violation(s) detected. Human review required before merging."
info "Run with --warn-only to downgrade to a warning."
echo
exit 1
