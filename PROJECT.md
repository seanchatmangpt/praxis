# Project: praxis-graphlaw Knowledge Hooks

## Architecture
This project implements first-class Knowledge Hook capabilities directly inside the low-level `praxis-graphlaw` reasoner.

1. **Parser & Registry**: Parsed natively from loaded graph triples matching `?s rdf:type kh:Hook`. Validates hook packs against a core SHACL Law Pack at load time (constitutional gating).
2. **First-Class Triggers**: Evaluates condition kinds including Datalog, Delta, Threshold, Count, Window, SHACL, ShEx, N3, and SPARQL (ASK/SELECT).
3. **Pure Action Projections**: Evaluates declarative SPARQL CONSTRUCT actions projecting into `kh:addQuad` / `kh:deleteQuad` graph deltas. Prevents/refuses host-level side-effects.
4. **BLAKE3 Receipts**: Serializes projected deltas to a sorted canonical N-Quads format and hashes them using BLAKE3 to generate deterministic receipts.
5. **Fixpoint Integration**: Hook evaluation runs inside the `Reasoner::materialize` stratum fixpoint loop, feeding projected additions back into the current reasoning cycle and rolling back on refusal.

## Code Layout
- `crates/praxis-graphlaw/src/hooks.rs`: Knowledge hooks representation, registry, extraction, validation, trigger evaluation, delta projection, and receipt generation.
- `crates/praxis-graphlaw/src/reasoner/mod.rs`: Fixpoint integration of hook evaluations.
- `crates/praxis-graphlaw/src/lib.rs`: Expose hook registration and gating on `TripleStore`.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: Hook Registry & Gating | Parse `kh:Hook` from triples, validate against SHACL Law Pack, refuse side-effects | none | COMPLETE |
| 2 | M2: Trigger Dialects & SPARQL | Implement condition evaluators, add SPARQL ASK / SELECT support | M1 | COMPLETE |
| 3 | M3: Pure Actions & CONSTRUCT | SPARQL CONSTRUCT actions projecting `kh:addQuad` / `kh:deleteQuad` | M2 | COMPLETE |
| 4 | M4: Canonical N-Quads & BLAKE3 | Sorted N-Quads serialization and BLAKE3 receipt generation | M3 | COMPLETE |
| 5 | M5: Reasoner Fixpoint Integration | Integrate hook evaluation & delta feedback loop in `Reasoner::materialize` | M4 | COMPLETE |
| 6 | M6: E2E Verification & Hardening | Opaque-box E2E tests, boundary checks, and adversarial hardening | M5 | IN_PROGRESS |

## Interface Contracts
### `TripleStore` ↔ `Reasoner`
- `TripleStore::load_hook_pack` -> parses, validates, and stores hooks.
- `Reasoner::materialize` evaluates triggers in-loop and returns/asserts projected deltas.
