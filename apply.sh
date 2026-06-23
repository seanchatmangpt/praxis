#!/usr/bin/env bash
# apply.sh — Apply praxis house standardisation layer into an existing Rust repo.
#
# Usage:
#   praxis/apply.sh <TARGET_REPO> [OPTIONS]
#
# Arguments:
#   TARGET_REPO   Path to the target Rust repository (must contain Cargo.toml)
#
# Options:
#   --dry-run         Preview what would be copied without writing anything
#   --check           Like --dry-run but exits 1 if anything is missing/stale (CI gate)
#   --force           Overwrite existing files (default: skip if already present)
#   --wasm            Also layer template-wasm/ files (auto-detected if wasm32 found)
#   --integration     Also copy template-integration/ CI workflow
#   --mcp             Also copy template-mcp/ CI workflow
#   --no-audit        Skip the anti-pattern audit section
#   -h, --help        Show this help
#
# Hygiene files copied from template/ (relative paths preserved):
#   deny.toml, typos.toml, rustfmt.toml, rust-toolchain.toml, SECURITY.md,
#   .editorconfig, .gitignore, justfile, cliff.toml, cicd.toml, anti-llm.toml,
#   LICENSE-MIT, LICENSE-APACHE, CONTRIBUTING.md, CHANGELOG.md,
#   .github/workflows/ci.yml, .github/workflows/release.yml,
#   .github/dependabot.yml, .github/pull_request_template.md
#
# Cargo.toml is NEVER overwritten. Missing [workspace.lints]/[lints] and
# [profile.release] sections are printed as actionable patches instead.
#
# Anti-pattern audit (after file copy):
#   ANTI-1  .cargo/config.toml RUSTFLAGS lint suppression
#   ANTI-2  Nightly toolchain without date pin
#   ANTI-3  proc-macro-error (RUSTSEC-2024-0370) in Cargo.toml/Cargo.lock
#   ANTI-5  Missing [workspace.lints] inheritance in workspace member crates
#   WASM-1  strip = true in [profile.release] (corrupts WASM binaries — BUG-1)
#   WASM-2  getrandom missing js feature for wasm32 target
#   WASM-3  console_error_panic_hook not initialised
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE="$SCRIPT_DIR/template"
TEMPLATE_WASM="$SCRIPT_DIR/template-wasm"
TEMPLATE_INTEGRATION="$SCRIPT_DIR/template-integration"
TEMPLATE_MCP="$SCRIPT_DIR/template-mcp"

# ── Argument parsing ──────────────────────────────────────────────────────────
TARGET=""
FORCE=0
DRY=0
CHECK=0
OPT_WASM=0
OPT_INTEGRATION=0
OPT_MCP=0
OPT_NO_AUDIT=0

for arg in "$@"; do
  case "$arg" in
    --force)        FORCE=1 ;;
    --dry-run)      DRY=1 ;;
    --check)        DRY=1; CHECK=1 ;;
    --wasm)         OPT_WASM=1 ;;
    --integration)  OPT_INTEGRATION=1 ;;
    --mcp)          OPT_MCP=1 ;;
    --no-audit)     OPT_NO_AUDIT=1 ;;
    -h|--help)
      # Print the header comment block (lines 2-N, stopping at first non-comment)
      awk 'NR==1 { next } /^[^#]/ { exit } { gsub(/^# ?/, ""); print }' "$0"
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
  echo "Usage: $0 <TARGET_REPO> [--dry-run] [--force] [--wasm] [--integration] [--mcp]" >&2
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

# ── Detect workspace vs single-crate ─────────────────────────────────────────
IS_WORKSPACE=0
if grep -q '^\[workspace\]' "$TARGET/Cargo.toml" 2>/dev/null; then
  IS_WORKSPACE=1
fi

# ── Auto-detect WASM ─────────────────────────────────────────────────────────
WASM_AUTODETECTED=0
if [[ $OPT_WASM -eq 0 ]]; then
  if grep -rq 'wasm32\|wasm-bindgen\|wasm_bindgen' "$TARGET/Cargo.toml" 2>/dev/null || \
     find "$TARGET/src" -name "*.rs" 2>/dev/null \
       | xargs grep -l 'wasm_bindgen\|target_arch.*wasm' 2>/dev/null | grep -q .; then
    OPT_WASM=1
    WASM_AUTODETECTED=1
  fi
fi

# ── Summary counters ──────────────────────────────────────────────────────────
declare -a APPLIED_LIST=()
declare -a SKIPPED_LIST=()
declare -a WOULD_APPLY_LIST=()
declare -a MISSING_TPL_LIST=()

# ── Helper: process one file ──────────────────────────────────────────────────
process_file() {
  local rel="$1"
  local src_base="${2:-$TEMPLATE}"
  local src="$src_base/$rel"
  local dst="$TARGET/$rel"

  if [[ ! -f "$src" ]]; then
    MISSING_TPL_LIST+=("$rel (not in ${src_base##*/})")
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
  if [[ $CHECK -eq 1 ]]; then
    echo "==> CHECK MODE — will exit 1 if any file is missing or would change"
  else
    echo "==> DRY RUN — no files will be written"
  fi
fi
echo "==> Applying praxis house boilerplate: $TEMPLATE -> $TARGET"
if [[ $IS_WORKSPACE -eq 1 ]]; then
  echo "    Repo type: WORKSPACE (found [workspace] in Cargo.toml)"
else
  echo "    Repo type: SINGLE CRATE"
fi
if [[ $OPT_WASM -eq 1 ]]; then
  if [[ $WASM_AUTODETECTED -eq 1 ]]; then
    echo "    WASM: auto-detected (wasm32/wasm_bindgen references found)"
  else
    echo "    Mode: --wasm"
  fi
fi
[[ $OPT_INTEGRATION -eq 1 ]] && echo "    Mode: --integration"
[[ $OPT_MCP -eq 1 ]]         && echo "    Mode: --mcp"
echo

# ── Core hygiene files (always applied) ───────────────────────────────────────
HYGIENE_FILES=(
  deny.toml
  typos.toml
  rustfmt.toml
  rust-toolchain.toml
  SECURITY.md
  .editorconfig
  .gitignore
  justfile
  cliff.toml
  cicd.toml
  anti-llm.toml
  LICENSE-MIT
  LICENSE-APACHE
  CONTRIBUTING.md
  CHANGELOG.md
  .github/workflows/ci.yml
  .github/workflows/release.yml
  .github/dependabot.yml
  .github/pull_request_template.md
)

for rel in "${HYGIENE_FILES[@]}"; do
  process_file "$rel"
done

# ── WASM variant files (override base where applicable) ───────────────────────
if [[ $OPT_WASM -eq 1 && -d "$TEMPLATE_WASM" ]]; then
  # These files from template-wasm/ supersede the base template equivalents
  for rel in .cargo/config.toml .github/workflows/ci.yml justfile; do
    process_file "$rel" "$TEMPLATE_WASM"
  done
fi

# ── Integration test variant ──────────────────────────────────────────────────
if [[ $OPT_INTEGRATION -eq 1 && -d "$TEMPLATE_INTEGRATION" ]]; then
  process_file ".github/workflows/integration.yml" "$TEMPLATE_INTEGRATION"
fi

# ── MCP variant ───────────────────────────────────────────────────────────────
if [[ $OPT_MCP -eq 1 && -d "$TEMPLATE_MCP" ]]; then
  process_file ".github/workflows/ci.yml" "$TEMPLATE_MCP"
fi

echo

# ── Cargo.toml: diff missing sections (never overwrite) ───────────────────────
echo "==> Cargo.toml audit (never overwritten — missing sections shown as patches)"
echo

TARGET_CARGO="$TARGET/Cargo.toml"
TEMPLATE_CARGO="$TEMPLATE/Cargo.toml"
TEMPLATE_WS_CARGO="$TEMPLATE/Cargo.workspace.toml"
CARGO_DIFF_NEEDED=0

# Choose reference template based on workspace vs single-crate
REF_CARGO="$TEMPLATE_CARGO"
if [[ $IS_WORKSPACE -eq 1 && -f "$TEMPLATE_WS_CARGO" ]]; then
  REF_CARGO="$TEMPLATE_WS_CARGO"
fi

if [[ ! -f "$REF_CARGO" ]]; then
  echo "    [!] Reference template not found at '$REF_CARGO' — skipping Cargo.toml diff"
else
  # ── [workspace.lints] / [lints] section ─────────────────────────────────
  if [[ $IS_WORKSPACE -eq 1 ]]; then
    LINTS_PATTERN='^\[workspace\.lints\]'
    LINTS_LABEL="[workspace.lints]"
    LINTS_BLOCK=$(awk '
      /^\[workspace\.lints\]/ { found=1; next }
      found && /^\[/ && !/^\[workspace\.lints\./ { found=0 }
      found { print }
    ' "$REF_CARGO")
  else
    LINTS_PATTERN='^\[lints\]'
    LINTS_LABEL="[lints]"
    LINTS_BLOCK=$(awk '
      /^\[lints\]/ { found=1; next }
      found && /^\[/ && !/^\[lints\./ { found=0 }
      found { print }
    ' "$REF_CARGO")
  fi

  HAS_LINTS=$(grep -c "$LINTS_PATTERN" "$TARGET_CARGO" 2>/dev/null || true)

  if [[ "${HAS_LINTS:-0}" -eq 0 ]]; then
    CARGO_DIFF_NEEDED=1
    echo "    MISSING $LINTS_LABEL section. Add to Cargo.toml:"
    echo "    ─────────────────────────────────────────────"
    if [[ -n "$LINTS_BLOCK" ]]; then
      echo "$LINTS_BLOCK" | sed 's/^/    /'
    else
      if [[ $IS_WORKSPACE -eq 1 ]]; then
        cat <<'LINTS_WS_FALLBACK' | sed 's/^/    /'
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unreachable_pub = "warn"
unexpected_cfgs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
unwrap_used = "warn"
expect_used = "warn"
LINTS_WS_FALLBACK
      else
        cat <<'LINTS_FALLBACK' | sed 's/^/    /'
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unreachable_pub = "warn"
unexpected_cfgs = "warn"

[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
unwrap_used = "warn"
expect_used = "warn"
LINTS_FALLBACK
      fi
    fi
    echo "    ─────────────────────────────────────────────"
    echo
  else
    echo "    $LINTS_LABEL section present — OK"
  fi

  # ── [profile.release] section ────────────────────────────────────────────
  RELEASE_BLOCK=$(awk '
    /^\[profile\.release\]/ { found=1; next }
    found && /^\[/ && !/^\[profile\.release/ { found=0 }
    found { print }
  ' "$REF_CARGO")

  HAS_RELEASE=$(grep -c '^\[profile\.release\]' "$TARGET_CARGO" 2>/dev/null || true)

  if [[ "${HAS_RELEASE:-0}" -eq 0 ]]; then
    CARGO_DIFF_NEEDED=1
    echo "    MISSING [profile.release] section. Add to Cargo.toml:"
    echo "    ─────────────────────────────────────────────"
    if [[ -n "$RELEASE_BLOCK" ]]; then
      echo "$RELEASE_BLOCK" | sed 's/^/    /'
    else
      cat <<'RELEASE_FALLBACK' | sed 's/^/    /'
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
# strip omitted: wasm32 requires wasm-opt for stripping (strip = true corrupts WASM binaries)
RELEASE_FALLBACK
    fi
    echo "    ─────────────────────────────────────────────"
    echo
  else
    echo "    [profile.release] section present — OK"
  fi

  # ── Workspace: check each member for lints.workspace = true (ANTI-5) ─────
  if [[ $IS_WORKSPACE -eq 1 ]]; then
    echo
    echo "    Checking member crates for 'lints.workspace = true' (ANTI-5)..."
    declare -a MEMBERS_WITHOUT_LINTS=()
    while IFS= read -r member_cargo; do
      [[ "$member_cargo" == "$TARGET_CARGO" ]] && continue
      # A member inherits workspace lints when it has a [lints] table with workspace = true
      if ! grep -q 'workspace\s*=\s*true' "$member_cargo" 2>/dev/null; then
        MEMBERS_WITHOUT_LINTS+=("${member_cargo#$TARGET/}")
      fi
    done < <(find "$TARGET" -name "Cargo.toml" \
                -not -path "$TARGET/Cargo.toml" \
                -not -path "*/target/*" 2>/dev/null | sort)

    if [[ ${#MEMBERS_WITHOUT_LINTS[@]} -gt 0 ]]; then
      CARGO_DIFF_NEEDED=1
      echo "    [WARN] ANTI-5: ${#MEMBERS_WITHOUT_LINTS[@]} member crate(s) missing 'lints.workspace = true':"
      for m in "${MEMBERS_WITHOUT_LINTS[@]}"; do
        echo "      - $m"
      done
      echo "    Fix: add to each member Cargo.toml:"
      echo "      [lints]"
      echo "      workspace = true"
      echo
    else
      echo "    All member crates inherit workspace lints — OK"
    fi
  fi

  if [[ $CARGO_DIFF_NEEDED -eq 0 ]]; then
    echo "    Cargo.toml is up to date — nothing to add."
  fi
fi

echo

# ── Hygiene file divergence check ─────────────────────────────────────────────
echo "==> Hygiene file divergence check (existing files vs template)"
echo

DIVERGED_COUNT=0
DIFF_CHECK_FILES=(deny.toml typos.toml rustfmt.toml .editorconfig rust-toolchain.toml)

for rel in "${DIFF_CHECK_FILES[@]}"; do
  src="$TEMPLATE/$rel"
  dst="$TARGET/$rel"
  if [[ -f "$src" && -f "$dst" ]]; then
    if ! diff -q "$src" "$dst" >/dev/null 2>&1; then
      DIVERGED_COUNT=$((DIVERGED_COUNT + 1))
      echo "    [DIFF] $rel diverges from template (--force to overwrite):"
      diff --unified=2 "$src" "$dst" 2>/dev/null | head -25 | sed 's/^/      /' || true
      echo
    fi
  fi
done

if [[ $DIVERGED_COUNT -eq 0 ]]; then
  echo "    All checked hygiene files match the template."
fi
echo

# ── Anti-pattern audit ────────────────────────────────────────────────────────
AUDIT_WARNINGS=0

if [[ $OPT_NO_AUDIT -eq 0 ]]; then
  echo "==> Anti-pattern audit"
  echo

  # ANTI-1: .cargo/config.toml RUSTFLAGS lint suppression
  CARGO_CFG="$TARGET/.cargo/config.toml"
  if [[ -f "$CARGO_CFG" ]]; then
    # Match -A clippy::, --allow clippy::, -A warnings, --allow warnings
    if grep -E '"-A"|"--allow"|rustflags' "$CARGO_CFG" 2>/dev/null \
       | grep -qiE 'clippy|warnings'; then
      AUDIT_WARNINGS=$((AUDIT_WARNINGS + 1))
      echo "    [WARN] ANTI-1: .cargo/config.toml appears to suppress lints via RUSTFLAGS:"
      grep -n -E '"-A"|"--allow"' "$CARGO_CFG" 2>/dev/null | grep -iE 'clippy|warn' \
        | sed 's/^/      /' || true
      echo "    Fix: move lint allows to [workspace.lints.clippy] with justification comments."
      echo "    Ref: survey/01-SECOND-WAVE.md §5 ANTI-1"
      echo
    else
      echo "    [OK]   ANTI-1: .cargo/config.toml present, no RUSTFLAGS lint suppression"
      echo
    fi
  else
    echo "    [OK]   ANTI-1: No .cargo/config.toml (no RUSTFLAGS suppression risk)"
    echo
  fi

  # ANTI-2: Bare nightly toolchain (unpinned)
  TOOLCHAIN_FILE="$TARGET/rust-toolchain.toml"
  [[ ! -f "$TOOLCHAIN_FILE" ]] && TOOLCHAIN_FILE="$TARGET/rust-toolchain"
  if [[ -f "$TOOLCHAIN_FILE" ]]; then
    if grep -q 'channel\s*=\s*"nightly"[^-]' "$TOOLCHAIN_FILE" 2>/dev/null || \
       grep -E 'channel\s*=\s*"nightly"\s*$' "$TOOLCHAIN_FILE" 2>/dev/null | grep -q .; then
      AUDIT_WARNINGS=$((AUDIT_WARNINGS + 1))
      echo "    [WARN] ANTI-2: rust-toolchain pins bare 'nightly' (unpinned — CI breaks on new lints):"
      grep -n 'channel' "$TOOLCHAIN_FILE" | sed 's/^/      /'
      echo "    Fix: pin to 'nightly-YYYY-MM-DD' or switch to stable '1.82.0'."
      echo "    Ref: survey/01-SECOND-WAVE.md §5 ANTI-2"
      echo
    elif grep -q 'channel.*nightly-[0-9]' "$TOOLCHAIN_FILE" 2>/dev/null; then
      echo "    [INFO] ANTI-2: Pinned nightly toolchain — verify nightly features are justified:"
      grep -n 'channel' "$TOOLCHAIN_FILE" | sed 's/^/      /'
      echo "    Ensure CLAUDE.md documents which nightly feature(s) require it."
      echo
    else
      echo "    [OK]   ANTI-2: Toolchain is pinned stable or absent"
      echo
    fi
  else
    echo "    [INFO] ANTI-2: No rust-toolchain.toml — toolchain is uncontrolled"
    echo "    Fix: apply.sh will copy from template (run without --dry-run)."
    echo
  fi

  # ANTI-3: proc-macro-error (RUSTSEC-2024-0370 — unmaintained)
  declare -a PROC_MACRO_ERROR_FILES=()
  for candidate in \
    "$TARGET/Cargo.toml" \
    "$TARGET/Cargo.lock" \
    "$TARGET"/crates/*/Cargo.toml \
    "$TARGET"/*/Cargo.toml; do
    [[ -f "$candidate" ]] || continue
    # Match proc-macro-error but NOT proc-macro-error2
    if grep -E 'proc-macro-error[^2"]' "$candidate" 2>/dev/null | grep -q .; then
      PROC_MACRO_ERROR_FILES+=("${candidate#$TARGET/}")
    fi
  done
  if [[ ${#PROC_MACRO_ERROR_FILES[@]} -gt 0 ]]; then
    AUDIT_WARNINGS=$((AUDIT_WARNINGS + 1))
    echo "    [WARN] ANTI-3: 'proc-macro-error' found (RUSTSEC-2024-0370 — unmaintained):"
    for f in "${PROC_MACRO_ERROR_FILES[@]}"; do
      echo "      - $f"
      grep -n -E 'proc-macro-error[^2"]' "$TARGET/$f" 2>/dev/null | sed 's/^/        /' || true
    done
    echo "    Fix: migrate to 'proc-macro-error2' (maintained fork) or 'manyhow'."
    echo "    Ref: survey/01-SECOND-WAVE.md §5 ANTI-3"
    echo
  else
    echo "    [OK]   ANTI-3: No proc-macro-error (RUSTSEC-2024-0370) found"
    echo
  fi

  # WASM checks (only when WASM detected)
  if [[ $OPT_WASM -eq 1 ]]; then
    # WASM-1: strip = true in [profile.release] corrupts WASM binaries (BUG-1)
    if awk '
      /^\[profile\.release\]/ { in_section=1; next }
      in_section && /^\[/ { in_section=0 }
      in_section && /strip\s*=\s*true/ { found=1; exit }
      END { exit !found }
    ' "$TARGET_CARGO" 2>/dev/null; then
      AUDIT_WARNINGS=$((AUDIT_WARNINGS + 1))
      echo "    [WARN] WASM-1: [profile.release] has 'strip = true' — CORRUPTS WASM binaries!"
      grep -n 'strip' "$TARGET_CARGO" | sed 's/^/      /'
      echo "    Fix: remove 'strip = true' from [profile.release]. Use 'just build-opt'"
      echo "    (wasm-opt -Os) for size reduction. Native stripping is via linker/packager."
      echo "    Ref: survey/01-SECOND-WAVE.md BUG-1"
      echo
    else
      echo "    [OK]   WASM-1: No 'strip = true' in [profile.release]"
      echo
    fi

    # WASM-2: getrandom missing js feature for wasm32
    if grep -q 'getrandom' "$TARGET_CARGO" 2>/dev/null; then
      if ! grep -A 10 "cfg(target_arch.*wasm32" "$TARGET_CARGO" 2>/dev/null \
           | grep -q 'getrandom.*js'; then
        echo "    [INFO] WASM-2: 'getrandom' dependency found but may be missing 'js' feature"
        echo "    for wasm32 targets. Expected in Cargo.toml:"
        printf '      [target.'"'"'cfg(target_arch = "wasm32")'"'"'.dependencies]\n'
        echo "      getrandom = { version = \"0.2\", features = [\"js\"] }"
        echo
      else
        echo "    [OK]   WASM-2: getrandom has 'js' feature for wasm32 target"
        echo
      fi
    fi

    # WASM-3: console_error_panic_hook initialisation
    if [[ -d "$TARGET/src" ]]; then
      if ! grep -rq 'console_error_panic_hook' "$TARGET/src" 2>/dev/null; then
        echo "    [INFO] WASM-3: 'console_error_panic_hook::set_once()' not found in src/"
        echo "    Fix: call it in #[wasm_bindgen(start)] for browser-visible panics."
        echo
      else
        echo "    [OK]   WASM-3: console_error_panic_hook initialised"
        echo
      fi
    fi
  fi

  # Audit summary
  if [[ $AUDIT_WARNINGS -eq 0 ]]; then
    echo "    Anti-pattern audit: no warnings."
  else
    echo "    Anti-pattern audit: $AUDIT_WARNINGS warning(s) — see details above."
  fi
  echo
fi

# ── Summary table ─────────────────────────────────────────────────────────────
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
RESET="\033[0m"

echo "┌─────────────────────────────────────────────────────────────────┐"
echo "│                        SUMMARY                                  │"
echo "├──────────────┬──────────────────────────────────────────────────┤"

printf "│ ${GREEN}%-12s${RESET} │ %-48s│\n" "APPLIED" "${#APPLIED_LIST[@]} file(s)"
for f in "${APPLIED_LIST[@]}"; do
  printf "│ %-12s │   %-46s│\n" "" "$f"
done

printf "│ ${YELLOW}%-12s${RESET} │ %-48s│\n" "SKIPPED" "${#SKIPPED_LIST[@]} file(s) (exist; --force to overwrite)"
for f in "${SKIPPED_LIST[@]}"; do
  printf "│ %-12s │   %-46s│\n" "" "$f"
done

if [[ $DRY -eq 1 ]]; then
  printf "│ ${CYAN}%-12s${RESET} │ %-48s│\n" "WOULD_APPLY" "${#WOULD_APPLY_LIST[@]} file(s)"
  for f in "${WOULD_APPLY_LIST[@]}"; do
    printf "│ %-12s │   %-46s│\n" "" "$f"
  done
fi

if [[ ${#MISSING_TPL_LIST[@]} -gt 0 ]]; then
  printf "│ ${RED}%-12s${RESET} │ %-48s│\n" "NOT_IN_TPL" "${#MISSING_TPL_LIST[@]} file(s) not in template"
  for f in "${MISSING_TPL_LIST[@]}"; do
    printf "│ %-12s │   %-46s│\n" "" "$f"
  done
fi

echo "└──────────────┴──────────────────────────────────────────────────┘"
echo

# ── Final status ──────────────────────────────────────────────────────────────
if [[ $DRY -eq 1 ]]; then
  if [[ $CHECK -eq 1 ]]; then
    # --check mode: fail if anything is pending
    _issues=0
    [[ ${#WOULD_APPLY_LIST[@]} -gt 0 ]] && _issues=$((_issues + ${#WOULD_APPLY_LIST[@]}))
    [[ $CARGO_DIFF_NEEDED -eq 1 ]] && _issues=$((_issues + 1))
    [[ $AUDIT_WARNINGS -gt 0 ]] && _issues=$((_issues + AUDIT_WARNINGS))
    if [[ $_issues -gt 0 ]]; then
      echo "check: FAIL — $_issues issue(s) pending (files to apply: ${#WOULD_APPLY_LIST[@]}, Cargo.toml patches: ${CARGO_DIFF_NEEDED:-0}, audit: ${AUDIT_WARNINGS:-0})"
      exit 1
    else
      echo "check: PASS — repo is fully up to date with praxis template."
      exit 0
    fi
  else
    echo "Dry run complete. Run without --dry-run to apply changes."
    echo "Tip: --force overwrites existing hygiene files with template versions."
  fi
else
  echo "Done. applied=${#APPLIED_LIST[@]} skipped=${#SKIPPED_LIST[@]}"
  if [[ $AUDIT_WARNINGS -gt 0 ]]; then
    echo "NOTICE: $AUDIT_WARNINGS anti-pattern warning(s) require manual action (see audit above)."
  fi
fi

exit 0
