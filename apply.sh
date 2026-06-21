#!/usr/bin/env bash
# apply.sh — Apply affidavit house standardization layer into an existing Rust repo.
#
# Usage:
#   praxis/apply.sh <TARGET_REPO> [--dry-run] [--force]
#
# Arguments:
#   TARGET_REPO   Path to the target Rust repository (must contain Cargo.toml)
#
# Flags:
#   --dry-run     Preview what would be copied without writing anything
#   --force       Overwrite existing files (default: skip if file already exists)
#   -h, --help    Show this help
#
# Hygiene files copied (relative paths preserved):
#   deny.toml, typos.toml, .editorconfig, rustfmt.toml, rust-toolchain.toml,
#   SECURITY.md, .github/workflows/ci.yml, .github/workflows/release.yml,
#   .github/dependabot.yml
#
# Cargo.toml is NEVER overwritten — a diff of missing [lints] and
# profile.release sections is printed instead.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE="$SCRIPT_DIR/template"

# ── Argument parsing ──────────────────────────────────────────────────────────
TARGET=""
FORCE=0
DRY=0

for arg in "$@"; do
  case "$arg" in
    --force)   FORCE=1 ;;
    --dry-run) DRY=1 ;;
    -h|--help)
      grep '^#' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    -*)
      echo "error: unknown flag: $arg" >&2
      echo "Run with --help for usage." >&2
      exit 2
      ;;
    *)
      if [[ -n "$TARGET" ]]; then
        echo "error: unexpected argument '$arg' (TARGET_REPO already set to '$TARGET')" >&2
        exit 2
      fi
      TARGET="$arg"
      ;;
  esac
done

# ── Validate inputs ───────────────────────────────────────────────────────────
if [[ -z "$TARGET" ]]; then
  echo "error: TARGET_REPO argument is required." >&2
  echo "Usage: $0 <TARGET_REPO> [--dry-run] [--force]" >&2
  exit 2
fi

if [[ ! -d "$TARGET" ]]; then
  echo "error: target '$TARGET' is not a directory or does not exist." >&2
  exit 1
fi

if [[ ! -f "$TARGET/Cargo.toml" ]]; then
  echo "error: '$TARGET' does not appear to be a Rust project (no Cargo.toml found)." >&2
  exit 1
fi

if [[ ! -d "$TEMPLATE" ]]; then
  echo "error: template directory not found at '$TEMPLATE'." >&2
  exit 1
fi

TARGET="$(cd "$TARGET" && pwd)"

# ── Files to copy ─────────────────────────────────────────────────────────────
HYGIENE_FILES=(
  deny.toml
  typos.toml
  rustfmt.toml
  rust-toolchain.toml
  SECURITY.md
  .github/workflows/ci.yml
  .github/workflows/release.yml
  .github/dependabot.yml
)

# .editorconfig lives in template root — handle separately (may not exist there)
OPTIONAL_FILES=(
  .editorconfig
)

# ── Summary counters ──────────────────────────────────────────────────────────
declare -a APPLIED_LIST=()
declare -a SKIPPED_LIST=()
declare -a WOULD_APPLY_LIST=()
declare -a MISSING_LIST=()

# ── Helper: process one file ──────────────────────────────────────────────────
process_file() {
  local rel="$1"
  local src="$TEMPLATE/$rel"
  local dst="$TARGET/$rel"

  if [[ ! -f "$src" ]]; then
    MISSING_LIST+=("$rel")
    return
  fi

  if [[ -e "$dst" && $FORCE -eq 0 ]]; then
    SKIPPED_LIST+=("$rel")
    return
  fi

  if [[ $DRY -eq 1 ]]; then
    WOULD_APPLY_LIST+=("$rel")
    return
  fi

  mkdir -p "$(dirname "$dst")"
  cp "$src" "$dst"
  APPLIED_LIST+=("$rel")
}

# ── Mode banner ───────────────────────────────────────────────────────────────
if [[ $DRY -eq 1 ]]; then
  echo "==> DRY RUN — no files will be written"
fi
echo "==> Applying house boilerplate: $TEMPLATE -> $TARGET"
echo

# ── Process hygiene files ─────────────────────────────────────────────────────
for rel in "${HYGIENE_FILES[@]}"; do
  process_file "$rel"
done

for rel in "${OPTIONAL_FILES[@]}"; do
  process_file "$rel"
done

# ── Cargo.toml: diff missing sections (never overwrite) ───────────────────────
echo "==> Cargo.toml (never overwritten — diff of missing sections below)"
echo

TARGET_CARGO="$TARGET/Cargo.toml"
TEMPLATE_CARGO="$TEMPLATE/Cargo.toml"

if [[ ! -f "$TEMPLATE_CARGO" ]]; then
  echo "    [!] template/Cargo.toml not found — skipping Cargo.toml diff"
else
  # Extract [lints] block from template (stop before next top-level section)
  LINTS_BLOCK=$(awk '
    /^\[lints\]/ { found=1; next }
    found && /^\[/ && !/^\[lints\./ { found=0 }
    found { print }
  ' "$TEMPLATE_CARGO")

  # Extract [profile.release] block from template
  RELEASE_BLOCK=$(awk '
    /^\[profile\.release\]/ { found=1; next }
    found && /^\[/ && !/^\[profile\.release/ { found=0 }
    found { print }
  ' "$TEMPLATE_CARGO")

  HAS_LINTS=$(grep -c '^\[lints\]' "$TARGET_CARGO" 2>/dev/null || true)
  HAS_RELEASE=$(grep -c '^\[profile\.release\]' "$TARGET_CARGO" 2>/dev/null || true)

  CARGO_DIFF_NEEDED=0

  if [[ "${HAS_LINTS:-0}" -eq 0 ]]; then
    CARGO_DIFF_NEEDED=1
    echo "    MISSING [lints] section. Add to $TARGET_CARGO:"
    echo "    ─────────────────────────────────────────────"
    if [[ -n "$LINTS_BLOCK" ]]; then
      echo "$LINTS_BLOCK" | sed 's/^/    /'
    else
      cat <<'LINTS_FALLBACK' | sed 's/^/    /'
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
pedantic = "warn"
unwrap_used = "warn"
expect_used = "warn"
LINTS_FALLBACK
    fi
    echo "    ─────────────────────────────────────────────"
    echo
  else
    echo "    [lints] section already present — OK"
  fi

  if [[ "${HAS_RELEASE:-0}" -eq 0 ]]; then
    CARGO_DIFF_NEEDED=1
    echo "    MISSING [profile.release] section. Add to $TARGET_CARGO:"
    echo "    ─────────────────────────────────────────────"
    if [[ -n "$RELEASE_BLOCK" ]]; then
      echo "$RELEASE_BLOCK" | sed 's/^/    /'
    else
      cat <<'RELEASE_FALLBACK' | sed 's/^/    /'
[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
RELEASE_FALLBACK
    fi
    echo "    ─────────────────────────────────────────────"
    echo
  else
    echo "    [profile.release] section already present — OK"
  fi

  if [[ $CARGO_DIFF_NEEDED -eq 0 ]]; then
    echo "    Cargo.toml is up to date — nothing to add."
  fi
fi

echo

# ── Summary table ─────────────────────────────────────────────────────────────
print_col() {
  local label="$1"
  local -n _list="$2"
  local color="$3"
  local reset="\033[0m"
  if [[ ${#_list[@]} -eq 0 ]]; then
    printf "${color}%-14s${reset} (none)\n" "$label"
  else
    printf "${color}%-14s${reset} %s\n" "$label" "${_list[0]}"
    for item in "${_list[@]:1}"; do
      printf "%-14s %s\n" "" "$item"
    done
  fi
}

echo "┌─────────────────────────────────────────────────────────────────┐"
echo "│                        SUMMARY                                  │"
echo "├──────────────┬──────────────────────────────────────────────────┤"

GREEN="\033[0;32m"
YELLOW="\033[0;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
RESET="\033[0m"

printf "│ ${GREEN}%-12s${RESET} │ %-48s│\n" "APPLIED" "${#APPLIED_LIST[@]} file(s)"
for f in "${APPLIED_LIST[@]}"; do
  printf "│ %-12s │   %-46s│\n" "" "$f"
done

printf "│ ${YELLOW}%-12s${RESET} │ %-48s│\n" "SKIPPED" "${#SKIPPED_LIST[@]} file(s) (already exist; use --force to overwrite)"
for f in "${SKIPPED_LIST[@]}"; do
  printf "│ %-12s │   %-46s│\n" "" "$f"
done

if [[ $DRY -eq 1 ]]; then
  printf "│ ${CYAN}%-12s${RESET} │ %-48s│\n" "WOULD_APPLY" "${#WOULD_APPLY_LIST[@]} file(s)"
  for f in "${WOULD_APPLY_LIST[@]}"; do
    printf "│ %-12s │   %-46s│\n" "" "$f"
  done
fi

if [[ ${#MISSING_LIST[@]} -gt 0 ]]; then
  printf "│ ${RED}%-12s${RESET} │ %-48s│\n" "NOT_IN_TPL" "${#MISSING_LIST[@]} file(s) missing from template"
  for f in "${MISSING_LIST[@]}"; do
    printf "│ %-12s │   %-46s│\n" "" "$f"
  done
fi

echo "└──────────────┴──────────────────────────────────────────────────┘"
echo

if [[ $DRY -eq 1 ]]; then
  echo "Dry run complete. Run without --dry-run to apply changes."
else
  echo "Done. applied=${#APPLIED_LIST[@]} skipped=${#SKIPPED_LIST[@]}"
fi

exit 0
