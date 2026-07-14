#!/usr/bin/env bash
# Axiom disclosure gate for tools/paper-factory/lean-lake/Praxis/Corpus.
#
# Fails if any `axiom` declaration exists outside ax_*.lean files that is not
# documented in AXIOM_ALLOWLIST.md. Does NOT fail on axioms already listed
# there (including class-(b) ones) -- the point is disclosure of new axioms,
# not prohibition of documented ones. Files currently owned by another
# in-progress reclassification effort are excluded from enforcement; see the
# "Excluded, in-progress" section of AXIOM_ALLOWLIST.md.
set -euo pipefail

CORPUS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/Praxis/Corpus" && pwd)"
ALLOWLIST="$(dirname "${BASH_SOURCE[0]}")/AXIOM_ALLOWLIST.md"

# Files another agent is actively reproving/reclassifying; excluded from
# this gate until that work lands and the inventory is recomputed.
EXCLUDED_FILES=(
  "prop_intauth.lean"
  "refusal_simpleoneforone.lean"
  "ref_curve.lean"
  "lineage_armstrong.lean"
  "def_obsauth.lean"
  "def_body.lean"
  "prop_topology.lean"
)

is_excluded() {
  local f="$1"
  for ex in "${EXCLUDED_FILES[@]}"; do
    [[ "$f" == "$ex" ]] && return 0
  done
  return 1
}

# Extract real axiom declarations: "<file>:<line>:axiom <Name>", filtering
# out ax_*.lean (designated axiom home) and prose false positives (doc
# comments where a sentence happens to start with the word "axiom").
declare -a offenders=()

while IFS= read -r line; do
  file="${line%%:*}"
  rest="${line#*:}"
  lineno="${rest%%:*}"
  content="${rest#*:}"

  # Skip designated axiom files.
  [[ "$file" == ax_* ]] && continue

  # Skip excluded, in-progress files.
  is_excluded "$file" && continue

  # Extract the identifier immediately following "axiom ". Real
  # declarations use a Lean identifier (letters/digits/_/.); prose false
  # positives are followed by english words like "is", "needed", etc. and
  # are filtered by requiring the next token to look like a Lean identifier
  # AND not be one of the known English stopwords that appear in this
  # corpus's doc-comment prose.
  name="$(sed -E 's/^[[:space:]]*axiom[[:space:]]+//' <<<"$content" | sed -E "s/^([A-Za-z_][A-Za-z0-9_.']*).*/\\1/")"

  case "$name" in
    is|needed.|introduced.|declared.)
      continue
      ;;
  esac

  offenders+=("${file}:${lineno}:${name}")
done < <(grep -rn "^\s*axiom\s" "$CORPUS_DIR"/*.lean | sed "s|^$CORPUS_DIR/||")

echo "Found ${#offenders[@]} in-scope axiom declarations (excluding ax_*.lean and the 7 in-progress files)."

missing=0
for entry in "${offenders[@]}"; do
  file="${entry%%:*}"
  rest="${entry#*:}"
  lineno="${rest%%:*}"
  name="${rest#*:}"

  # An axiom is "documented" if its file:line and name both appear on the
  # same allowlist table row.
  if ! grep -F "\`${file}:${lineno}\`" "$ALLOWLIST" | grep -qF "\`${name}\`"; then
    echo "UNDOCUMENTED AXIOM: ${file}:${lineno}: ${name}"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo ""
  echo "Gate FAILED: one or more axioms are not documented in AXIOM_ALLOWLIST.md."
  echo "Add a row (file:line, name, class a/b, justification) before merging,"
  echo "or move the declaration into an ax_*.lean file."
  exit 1
fi

echo "Gate PASSED: every in-scope axiom is documented in AXIOM_ALLOWLIST.md."
