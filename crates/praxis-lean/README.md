# praxis-lean

`praxis-lean` / `praxis-l4` is the 80/20 Lean 4 integration crate for Praxis math manufacturing.

It wraps the Lean 4 ecosystem as a deterministic admission layer:

```text
Praxis RDF corpus
→ Lean 4 / Lake kernel check
→ no-sorry / no-unauthorized-axiom gate
→ receipt JSONL
→ RDF label ↔ Lean declaration index
→ ggen report
→ LaTeX/PDF publication artifact
```

## Core law

```text
Verified(s) ⇔ KernelAccepts(s) ∧ NoSorry(s) ∧ NoUnauthorizedAxiom(s)
```

Agent reports are never authority. The real Lean 4 kernel is authority.

## Commands

```bash
praxis-l4 init --root tools/paper-factory/lean-pilot
praxis-l4 verify --root tools/paper-factory/lean-pilot --receipts formalization_receipts_v2.jsonl
praxis-l4 no-sorry --root tools/paper-factory/lean-pilot
praxis-l4 reconcile --index praxis-lean-index.json --receipts formalization_receipts_v2.jsonl
praxis-l4 report --index praxis-lean-index.json --receipts formalization_receipts_v2.jsonl --out report.json
```

## Workspace integration targets

Feature gates are present for:

- `ggen`
- `clap-noun-verb`
- `chicago-tdd-tools`
- `wasm4pm-compat`

The default build is standalone and uses `clap`. Inside Praxis, swap CLI registration to `clap-noun-verb`, route fixtures through `chicago-tdd-tools`, emit reports through `ggen`, and enforce bounded/refusal constants through `wasm4pm-compat`.
