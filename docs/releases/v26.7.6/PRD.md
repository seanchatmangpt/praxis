# Praxis v26.7.6 "After Neon" — Product Requirements Document

Status: DRAFT, tied to `RELEASE_CONTROL.md` (single control surface). Every claim
in this document cites a file, test, or receipt in this repository. Rows without
evidence are marked PLANNED or UNKNOWN, never asserted.

## 1. Product summary

Praxis is verified AI engineering infrastructure: a factory that turns language
into admitted, receipted technical artifacts. Work does not enter the system by
assertion. It is admitted by law-state (`crates/praxis-graphlaw`, surfaced as
`ggen law`), planned (`plan` verb, `src/verbs/plan.rs`), executed as workflow
(POWL tape, `src/plan_run.rs:67 compile_plan_to_powl`), gauged by kernels and
tests (`crates/praxis-lean`, `cargo test`), and evidenced by computed BLAKE3
genesis-folded receipts (`crates/ggen/tests/receipt_chain_e2e.rs`,
`src/verbs/receipt.rs`).

## 2. Post-cyberpunk frame

The cyberpunk premise was that technology outruns institutions; the neon is the
glare of systems nobody can audit. After Neon is the civic phase: the same
generative capacity, placed under law. Praxis is not a model and not an agent —
it is admission, standing, gauge, and receipt for machine-made work. The
deliverable of this release is not more output; it is *standing*: every object
typed, located, and receipted, every refusal named.

## 3. Customer problem

AI systems produce code, plans, and proofs faster than any human review process
can absorb. The output has volume but no standing: no admission criteria, no
chain of evidence, no way for a buyer, auditor, or downstream agent to
distinguish verified work from plausible text. Teams either re-review everything
(losing the speed) or trust the output (losing the assurance).

## 4. Product position

**Verified AI Engineering Infrastructure.** Praxis sits between generation and
acceptance: the law-state layer that admits, the planner that sequences, the
workflow engine that executes, the gauges that check, and the receipt chain that
preserves evidence. Competitors ship generation; Praxis ships standing.

## 5. Core equation

```
A = mu(O*)
```

Admitted artifacts `A` are the image of the objective space `O*` under the
manufacturing operator `mu` — the deterministic pipeline law-state → plan →
workflow → factory → gauge → receipt (`RELEASE_CONTROL.md` Sec. 2 architecture
loop). Nothing is in `A` that did not pass through `mu`; `mu` refuses forward
with typed `Refusal` variants rather than degrading (`src/lib.rs`).

## 6. Doctrine: Combinatorial Maximalism

Global closure across admitted axes: every object in the system is typed,
located, and receipted, and the frontier of what is admitted / refused /
unevaluated is itself a first-class receipt. This is implemented, not aspirational:
the `frontier` verb (`src/verbs/frontier.rs`, subcommands `matrix`, `summary`,
`counts`) emits the Lane 10 combinatorial-maximalist frontier receipt, with
matrix construction shared with `tests/frontier_matrix.rs` so the CLI and the
test suite cannot drift on what counts as admitted, refused, or unevaluated.

## 7. Primary release goal

Close the loop natively: the graph law engine (`crates/praxis-graphlaw`, roxi
clean-room adoption) replaces the frozen external `ggen-graph` coupling
(`Cargo.toml:52,80`), and the whole loop — admit → law → plan → workflow →
factory → gauge → receipt → report — runs as one deterministic command with
byte-identical receipts across two consecutive runs (`RELEASE_CONTROL.md` exit
criterion 3).

## 8. MVP definition

The MVP is the smallest surface that manufactures standing end to end:

1. `ggen law load|derive|validate|explain|export` against `[law].rules` and
   `[law].shapes` (IMPLEMENTED — see `CLI.md`).
2. `plan solve|execute` producing a POWL tape and receipted execution
   (`src/plan_run.rs`, `tests/plan_run_e2e.rs`).
3. `praxis-l4 l4 verify|no-sorry|reconcile|report` as the kernel gauge
   (`crates/praxis-lean`).
4. `receipt validate` + `ggen receipt verify|history` as the evidence chain.
5. One full-loop fixture proven deterministic
   (`tests/plan_run_e2e.rs::two_runs_identical_chain_hashes`).

## 9. Personas

- **Founder-operator.** Runs the factory alone; needs `just verify-all` as the
  single DoD gate (`justfile`) and receipts that survive their own absence.
- **AI agent.** Consumes the CLI programmatically; every command exposes
  `--introspect` (JSON Schema for tool-calling) and `--structured-errors`
  (observed on every binary help in this release: `ggen`, `praxis-l4`,
  `my-conforming-project`).
- **Adversarial reviewer.** Assumes every claim is false until a test or receipt
  says otherwise; served by the receipt chain (`ggen receipt history` recomputes
  the BLAKE3 chain) and `law explain` (full derived-triple diff).
- **Enterprise buyer.** Needs provenance and refusal semantics, not demos:
  closed vocabularies with unknown predicates refused by name
  (`docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`), no wall clock in any receipt path.

## 10. Functional requirements

| # | Requirement | Evidence surface |
|---|---|---|
| F1 | Law-state load/derive/validate/explain/export | `ggen law` subcommands (verified via `--help`, 2026-07-06) |
| F2 | Admission of JSON law objects: judge, admit, receipt, promote | root CLI `law` verb (`src/verbs/law.rs`, dispatches to `praxis_core::DefaultLaw`) |
| F3 | Plan route/solve/analyze/execute | root CLI `plan` verb (verified via `--help`) |
| F4 | Plan → POWL compilation and receipted execution | `src/plan_run.rs:67,89` |
| F5 | Factory pipeline resolve/enrich/extract/render/write | `ggen sync run` (verified via `--help`) |
| F6 | Kernel gauge: verify, no-sorry, reconcile, report, index-build | `praxis-l4 l4` subcommands (verified via `--help`) |
| F7 | Receipt chain verification | `ggen receipt verify|history`; `receipt validate` (root CLI) |
| F8 | Frontier receipt | `frontier matrix|summary|counts` |
| F9 | Graph/ontology validation against praxis vocabularies | `ggen graph validate` |
| F10 | Environment health | `ggen doctor run`, root `doctor` verb |

## 11. Non-functional requirements

1. **Determinism.** No wall clock in any hash/receipt path (`ts_ns=0` pattern);
   two consecutive full-loop runs must produce byte-identical receipts
   (`tests/plan_run_e2e.rs::two_runs_identical_chain_hashes` — currently FAILING
   pending signing-key provisioning; see Sec. 14).
2. **Typed refusal completeness.** No panics, no silent defaults; every error a
   `Refusal` variant extended in `src/lib.rs`, never a parallel enum.
3. **Computed evidence.** Receipts BLAKE3, genesis-folded, never asserted-in.
4. **Closed vocabularies.** `wf:`, `hook:`, `prayer-kernel:`, `agent:`; unknown
   predicates refused by name.
5. **Frozen dependency surface.** `praxis-synthesis` deps exactly pddl-index,
   chatman-common, blake3, serde, serde_json, thiserror
   (`crates/praxis-synthesis/tests/no_llm_runtime.rs`).
6. **Factory hygiene.** `crates/ggen`: `#![deny(unsafe_code)]`,
   `#![deny(clippy::print_stdout)]` (`crates/ggen/src/lib.rs`).

## 12. Out of scope

- The standalone `~/ggen` repository (frozen by user decision; `INVENTORY.md`).
- LLM runtime inside `praxis-synthesis` (forbidden by invariant 5).
- Market/planner sales reporting (Phase 3b, not in this release's exit criteria).
- The MISSING `tower-lsp-max` lineage and its `lsp-max` path patch
  (`Cargo.toml:153`) — tracked as a coupling, not a deliverable.
- New subsystems where a const table suffices (invariant 6).

## 13. Day-one finish plan

1. Provision `PRAXIS_SIGNING_KEY` for the test environment so
   `tests/plan_run_e2e.rs` (`full_loop_after_neon_fixture`,
   `two_runs_identical_chain_hashes`) can run to verdict — both currently refuse
   with the typed signing error, observed 2026-07-06.
2. Run `just verify-all`; capture output into `RELEASE_CONTROL.md` Sec. 8.
3. Land the graphlaw-through-ggen e2e proof (exit criterion 2).
4. Record the two-run byte-identical receipt pair (exit criterion 3).
5. Complete the 15-doc set in `docs/releases/v26.7.6/` (exit criterion 6).

## 14. Acceptance criteria

The seven exit criteria from `RELEASE_CONTROL.md` Sec. 5, verbatim in substance:

| # | Criterion | Proof required |
|---|---|---|
| 1 | `just verify-all` green | command output captured in RELEASE_CONTROL.md receipts section |
| 2 | graphlaw live in ggen with e2e proof | passing e2e test exercising praxis-graphlaw through the ggen factory |
| 3 | One-command full-loop demo, deterministic across 2 runs | byte-identical receipts from two consecutive runs |
| 4 | Breeds/algorithms admitted with a generated artifact | artifact + receipt tied to `BREED_MODULE_MAP` (`crates/praxis-synthesis/src/breeds.rs:15`) |
| 5 | Full command surface typed-refusal-complete | refusal tests per command in RELEASE_CONTROL.md Sec. 3 |
| 6 | 15 release docs in `docs/releases/v26.7.6/` | file count |
| 7 | Receipt chain verifies | receipt-chain verification output |

Known failing evidence at time of writing: `cargo test -p my-conforming-project
--test plan_run_e2e` — 1 passed, 2 failed (`full_loop_after_neon_fixture`,
`two_runs_identical_chain_hashes`), both refusing on missing
`PRAXIS_SIGNING_KEY`/`PRAXIS_SIGNING_KEY_FILE` (observed 2026-07-06). The
failure is a typed refusal, not a panic in product code; provisioning the key is
step 1 of the day-one finish plan.
