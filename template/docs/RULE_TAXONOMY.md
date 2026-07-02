# Rule-ID Taxonomy

A reusable convention for naming lint/validator rule IDs in praxis-generated
projects, sourced from `anti-llm-cheat-lsp/src/rules/*.rs` — a lint pack that
detects fabricated LLM success claims. Each finding carries a code of the
form `ANTI-LLM-<PREFIX>-<NNN>` (three-digit, zero-padded, per-prefix
sequence). Any new praxis-generated lint pack should follow this same
`<PREFIX>-<NNN>` scheme so codes stay greppable and stable across releases.

## Prefixes

| Prefix | Source file | Category | Example ID | Description |
|---|---|---|---|---|
| `DECLARE` | `declare_laws.rs` | Declare-style temporal constraints (Absence/Existence/Response) compiled from project law docs (e.g. AGENTS.md) | `DECLARE-001` | Plain `tower_lsp` reference detected where `Absence(tower_lsp_reference)` is required |
| `ORACLE` | `oracle.rs` | Non-deterministic or unverifiable "oracle" constructs standing in for real checks | `ORACLE-001` | `lazy_static` initialized from an environment variable outside test code |
| `CLAIM` | `claims.rs` | Victory-language / overclaim vocabulary and changelog laundering | `CLAIM-004` | Victory language and overclaim detection (single source of truth for claim vocabulary) |
| `CHANGELOG` | `claims.rs` | A changelog delta presented as if it were full spec coverage | `CHANGELOG-001` | Delta changelog presented as full spec coverage |
| `TRACE` | `trace.rs` | Fabricated or low-fidelity inference/execution trace evidence | `TRACE-001` | `inference_trace.push` called with a constant string literal instead of real data |
| `OCEL` | `ocel_rules.rs` | Missing or malformed OCEL 2.0 process-mining event evidence | `OCEL-001` | Diagnostic emitted without a corresponding OCEL process event |
| `ADMIT` | `ocel_rules.rs` | Admission/fitness reporting without measurement provenance | `ADMIT-001` | Fitness report contains a bare constant with no measurement provenance |
| `RECEIPT` | `receipts.rs` | Cryptographically signed receipts lacking real admission proof | `RECEIPT-001` | Test result reported as `ok` without a corresponding signed receipt |
| `HOLLOW` | `hollow.rs` | Placeholder/stub implementations masquerading as complete code | `HOLLOW-001` | `unimplemented!()` is a placeholder — hollow by law |
| `VERSION` | `version.rs` | Version-law violations (CalVer enforcement, path-dependency versioning) | `VERSION-002` | Path dependency declared with an explicit non-CalVer (SemVer) version |
| `ALT` (aka `DEAD-ALT`) | `dead_alt.rs` | Dead "alternate" implementations left behind after a rewrite | `ALT-001` | Function named with an alt suffix (`_v2`, `_fixed`, `_real`, ...) that is never called |

## Conventions for new lint packs

- **Format**: `ANTI-<PACK>-<PREFIX>-<NNN>` (or `<PACK>-<PREFIX>-<NNN>` if the
  pack has its own namespace). Numbers are per-prefix, sequential, and never
  reused even if a rule is retired.
- **One prefix per detection theme**, not per file — a source file may host
  several prefixes if it evaluates multiple themes (e.g. `claims.rs` hosts
  both `CLAIM` and `CHANGELOG`; `ocel_rules.rs` hosts both `OCEL` and `ADMIT`).
- **Category field**: every diagnostic should also carry a lowercase
  `category` string (e.g. `"oracle"`, `"receipt"`, `"trace"`) so tooling can
  group findings independent of the numeric ID.
- **Message contract**: each rule's doc comment states the exact construct
  being matched (e.g. `// ORACLE-001: lazy_static initialized from env`)
  immediately above the diagnostic it emits, so the mapping from source line
  to rule ID is grep-able.
- **Forbidden implication**: rules that assert a temporal/causal law (as in
  `DECLARE`/`OCEL`) should record the violated implication as a string field
  (e.g. `"DiagnosticEmitted => ProcessEvidenceRecorded"`) for audit trails.

## Applying this to a new project

1. Pick a short pack name for the domain being checked (tests, docs, CI,
   receipts, etc.).
2. Choose one prefix per detection theme, following the table above as a
   template for tone and scope (declaration laws, oracles, claims, traces,
   process evidence, admissions, receipts, hollow stubs, version laws, dead
   code).
3. Number rules sequentially within each prefix starting at `001`.
4. Keep the doc comment directly above each diagnostic emission so the
   taxonomy stays self-documenting from source.
