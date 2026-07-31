#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'agent-standards: REFUSED: %s\n' "$1" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_text() {
  local path="$1"
  local text="$2"
  grep -Fq -- "$text" "$path" || fail "missing required law in $path: $text"
}

required_files=(
  "AGENTS.md"
  "CHATGPT-CLOUD-AGENTS.md"
  ".claude/rules/_core/absolute.md"
  ".claude/rules/cognition-contracts.md"
  "docs/standing/STANDARDS_DELTA_2026-07-30.md"
  ".github/workflows/agent-standards.yml"
)

for path in "${required_files[@]}"; do
  require_file "$path"
done

require_text "AGENTS.md" '`AGENTS.md` is the sole normative agent document'
require_text "AGENTS.md" '**Zero unreceipted actuation.**'
require_text "AGENTS.md" 'PARTIAL_ALIVE'
require_text "AGENTS.md" 'BUILD_BROKEN'
require_text "AGENTS.md" 'UNKNOWN'
require_text "AGENTS.md" 'UNSUPPORTED'
require_text "AGENTS.md" 'ggen renders; Lean admits; mfact certifies.'
require_text "AGENTS.md" 'Exact-head finalization'
require_text "AGENTS.md" 'unit;'
require_text "AGENTS.md" 'independent verifier report.'

require_text ".claude/rules/_core/absolute.md" 'Preserve'
require_text ".claude/rules/_core/absolute.md" 'Fence'
require_text ".claude/rules/_core/absolute.md" 'Calculus'
require_text ".claude/rules/_core/absolute.md" 'Exclusions'
require_text ".claude/rules/_core/absolute.md" 'Falsifier'
require_text ".claude/rules/_core/absolute.md" 'Operationalization'

require_text ".claude/rules/cognition-contracts.md" 'Observation → Admission → Breed execution'
require_text ".claude/rules/cognition-contracts.md" 'Proposal visibility does not imply downstream authority'
require_text ".claude/rules/cognition-contracts.md" 'Per-breed `ALIVE` does not promote a composed pipeline.'

require_text "CHATGPT-CLOUD-AGENTS.md" 'blob → tree → commit → ref → draft PR'
require_text "CHATGPT-CLOUD-AGENTS.md" 'No unreceipted actuation'

require_text "docs/standing/STANDARDS_DELTA_2026-07-30.md" '**PARTIAL_ALIVE**'
require_text "docs/standing/STANDARDS_DELTA_2026-07-30.md" '2026-07-24 through'
require_text "docs/standing/STANDARDS_DELTA_2026-07-30.md" '## Falsifier'

require_text ".github/workflows/agent-standards.yml" 'bash scripts/verify-agent-standards.sh'

printf 'agent-standards: ALIVE: required contracts and structural laws are present\n'
