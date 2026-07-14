# Bribery-Compliance-Case CLI Runbook

Runbook for `crown-bribery-case`, the Solvane Global bribery-compliance case driver in
`crates/multifractal-workflow/src/bin/crown-bribery-case.rs`. Covers: how to start a run, the
terminal states (closed vs. each refusal type), where evidence lands on disk, and how to
reconstruct/verify a run. Companion to the adversarial refusal suite in
`crates/multifractal-workflow/tests/bribery_case_adversarial.rs`.

Last verified: 2026-07-13 (this session), against `crown-bribery-case` as it exists on this
branch today. See "Standing" at the end for the exact chain scope this reflects.

## 1. Starting a case run

```bash
CARGO_TARGET_DIR=target/agent-<your-agent-name> \
  cargo run -p multifractal-workflow --bin crown-bribery-case -- --run-id <run-id>
```

(`just multifractal-workflow-run-crown-bribery-case-isolated <your-agent-name> -- --run-id
<run-id>` is the `just`-wrapped equivalent, per this repo's no-bare-cargo build discipline.)

`--run-id` defaults to `bribery-case-run-1` if omitted. `--help`/`-h` prints usage and exits 0.
Any unrecognized extra argument is a usage refusal (exit 2) before any pipeline stage runs. The
case content itself (`case.ttl`, `hook.ttl`, `shapes.ttl`, `pddl-domain.ttl`, all under
`crates/multifractal-workflow/fixtures/bribery-case/`) is fixed and embedded at compile time
(`include_str!`) — this CLI does not currently accept a caller-supplied case file (unlike its
sibling `crown-local-cli`, which does take a file argument; see `docs/` note in section 5).

Live-verified this session (fresh run, `--run-id adversarial-runbook-verify-1`):

```
[F02 admission]      state=Admitted subject=https://cases.solvane-global.example.org/case/BRB-2026-0417 triples=67 receipt_hash=5d9d5fa29c21e18b161f498545a3d4de401a5e226fe19177f634805136e60300
[Knowledge Hook]     derived 3 sc:hasObligation triples ... ["assess-policy-violation", "verify-contractor-authorization-level", "verify-transaction-authenticity"]
[mfg::manufacture]   domain_bytes=3404 problem_bytes=1351 graph_hash=7a0abca471096da1d3d53fc37ec3149102abf8d319337a0ab4f45f9d263127f6
[F08 plan]           ops=10 goal_reached=true steps=["supply-evidence", "clear-authorization-obligation", "supply-evidence", "clear-policy-obligation", "supply-evidence", "clear-transaction-obligation", "close-obligations", "judge", "admit", "receipt"]
[F09/F10 growth]     leaves=10 partial_orders=2 choices=0 child_bindings=11 turtle_len=11572
[F13 Arazzo]         bytes=2818 arazzo_digest=5a7d32f0afe343dd1ec19ecd527e6d5de249dd756cbac73c41b1de5eae33f61a air_digest_hex=41c8ac00fa89e9961a1c7e5afa66df9c2b542651b450c8728fe310673d10c34c
[F14 compile]        workflows=1 steps=10 air_digest_hex=41c8ac00fa89e9961a1c7e5afa66df9c2b542651b450c8728fe310673d10c34c
crown-bribery-case: OK -- full chain (F02 -> hook -> F08 -> F09/F10 -> F13 -> F14) composed for real
```

Exit code `0`.

## 2. Terminal states

| State | Exit code | Meaning |
|---|---|---|
| `OK` (closed) | 0 | Every stage (F02 admission, hook obligation derivation, F08 PDDL8 planning, F09/F10 growth+geometry, F13 Arazzo manufacture, F14 AIR compile) succeeded; `goal_reached=true` in the F08 plan receipt. |
| `usage error: ...` | 2 | Bad CLI args (before any pipeline stage ran). |
| `could not write <path>: ...` | 2 | Filesystem I/O failure writing an artifact. |
| `internal admission policy invalid: ...` | 2 | This CLI's own fixed `AdmissionPolicy` failed to construct (defensive; SHACL literal is hand-verified). |
| `case.ttl is malformed: ...` | 2 | Local Turtle-parse pre-flight failure. |
| `hook.ttl failed to load as a kh: hook pack: ...` | 2 | Hook-pack parse/validate failure. |
| `F02 admission refused: ...` | 1 | One of F02's 6 real admission gates refused (Identity/Provenance/Authority/Shape/Semantic/Ledger — see `ObservationAdmissionRefused`). |
| `hook.ttl derived zero sc:hasObligation triples ...` | 1 | The cross-border trigger pattern did not match the admitted case graph. |
| `hook.ttl declares no sc:requiresEvidenceType fact for obligation ...` | 1 | **Scenario 3** (missing evidence): an obligation was derived but its evidence-type catalog entry is absent — see `tests/bribery_case_adversarial.rs::scenario3_*`. |
| `mfg::manufacture (RDF pdl: instance data -> PDDL8 text) failed: ...` | 1 | RDF→PDDL8 manufacture refused (bound violation, malformed instance data). |
| `F08 planning refused: ...` | 1 | `NoAdmissiblePlan` (no plan reachable — includes **Scenario 5**, an unbound Action-Hook capability) or `Underlying` (a real `Pddl8Error`, including **Scenario 6**'s `BoundExceeded` — see below; never silently folded into `NoAdmissiblePlan`). |
| `F09/F10 growth refused: ...` | 1 | Growth/geometry stage refusal. |
| `F13 Arazzo manufacture refused: ...` / `F14 Arazzo->AIR compile refused: ...` | 1 | Projection/compile stage refusal. |

There is currently **no distinct `BLOCKED` terminal RDF/JSON artifact** written on refusal —
refusal is reported via nonzero exit + a `Display`-formatted message on stderr, and no artifact
for the failed stage (or any stage after it) is written. This is a disclosed gap relative to
`pddl-domain.ttl`'s own `raw -> validated -> admitted -> receipted -> blocked` lifecycle
vocabulary (which the domain models internally via a real `block-for-missing-evidence` PDDL
action) — nothing in `crown-bribery-case.rs` currently projects that internal PDDL lifecycle
state back out to a `08-case-closure.ttl`-style artifact. See "Standing" below.

## 3. Where evidence lands on disk

Every run writes real intermediate/final artifacts under `target/crown-bribery-case/<run-id>/`
(a fixed relative path from the process's current working directory — **not** affected by
`CARGO_TARGET_DIR`, which only controls where the compiled binary/build cache lives):

| File | Stage | Content |
|---|---|---|
| `01-admitted-case.ttl` | F02 | The admitted case Turtle (verbatim `case.ttl`). |
| `02-derived-obligations.ttl` | Knowledge Hook | The real `sc:hasObligation` triples the hook derived. |
| `03-pddl-problem-fragment.ttl` | Stage-2 projector | The runtime `pdl:Problem` RDF fragment (obligations + evidence-type `:init` atoms). |
| `04-pddl-domain.pddl` / `04-pddl-problem.pddl` | `mfg::manufacture` | Real, bound-checked PDDL8 STRIPS text. |
| `05-plan-tape.json` | F08 | The real grounded plan tape (`Pddl8Tape`, JSON-serialized). |
| `06-powl-v2-model.ttl` | F09/F10 | Real POWL v2 geometry Turtle. |
| `07-arazzo-artifact.json` | F13 | The real Arazzo 1.1.0 JSON workflow document. |
| `07-arazzo-receipt.json` | F13 | `{source_powl_digest_hex, external_cut_identity, sparql_projection_digest_hex, tera_template_digest_hex, arazzo_digest_hex, compiler_version, air_digest_hex}` — see example below. |

Example `07-arazzo-receipt.json` (this session's live run):

```json
{
  "source_powl_digest_hex": "cdaa6a43d00d528b32a6527785b98a582e5bc5ad71882ab05026983a4e4bec18",
  "external_cut_identity": "urn:mfw:crown-bribery-case/arazzo/n0",
  "sparql_projection_digest_hex": "78a9cd3541992e37a3a03584d7fbca82d0648527b42124a8a0c79287fe341019",
  "tera_template_digest_hex": "635b333bad701da4dd428e8bc61a225caf53b6f88ff9515482d8ae2782f33de3",
  "arazzo_digest_hex": "5a7d32f0afe343dd1ec19ecd527e6d5de249dd756cbac73c41b1de5eae33f61a",
  "compiler_version": "26.7.12",
  "air_digest_hex": "41c8ac00fa89e9961a1c7e5afa66df9c2b542651b450c8728fe310673d10c34c"
}
```

**No OCEL/F24 evidence, no F18 Broker receipt, and no F25 replay receipt are written by
`crown-bribery-case` today.** Those exist as real, tested library code
(`f18_broker_law::Broker`, `f24_ocel_construct`, `f25_receipts_replay`) and are wired end-to-end
for a *different* fixture by the sibling binary `crown-local-cli` (see section 5) and by
`crown_external.rs`'s own (Erlang-`escript`-dependent, `#[ignore]`-gated) test suite — but
nothing in `crown-bribery-case.rs` itself calls them yet. This is the disclosed Stage-3 gap; see
"Standing".

## 4. Reconstructing and verifying a run

There is **no dedicated `crown-bribery-case verify`/`replay` subcommand today** (disclosed gap,
not a broken command — no such command exists to be broken). Two real mechanisms substitute:

**(a) Re-run-and-diff (determinism check).** Run twice with different `--run-id`s, then diff any
artifact after normalizing the run-id substring; a genuine run must be byte-identical:

```bash
cargo run -p multifractal-workflow --bin crown-bribery-case -- --run-id verify-a
cargo run -p multifractal-workflow --bin crown-bribery-case -- --run-id verify-b
diff <(sed 's/verify-a/RUN/g' target/crown-bribery-case/verify-a/07-arazzo-receipt.json) \
     <(sed 's/verify-b/RUN/g' target/crown-bribery-case/verify-b/07-arazzo-receipt.json)
```

Live-verified this session: `verify-a`/`verify-b` (named `adversarial-runbook-verify-1`/`-2`
above) produced byte-identical `07-arazzo-receipt.json` after run-id normalization — `diff`
exit 0, `IDENTICAL after run-id normalization` printed.

**(b) Library-level replay/verify (`f25_receipts_replay`).** The real digest-fold +
independent-replay + equivalence-comparison machinery a future `verify` subcommand would wrap is
exercised directly (no CLI integration yet) by
`tests/bribery_case_adversarial.rs::scenario2_tampered_material_is_detected_and_refused_on_independent_replay`,
which builds a real `Receipt` from this case's own live artifacts (admitted case Turtle, derived
obligations Turtle, domain/hook-catalog Turtle) via `receipt_builder::build`, then calls
`independent_verifier::verify` against a scratch, one-byte-flipped copy of a material and
confirms `ReceiptReplayRefused::EquivalenceMismatch` naming the exact digest kind that diverged.
Run it directly:

```bash
cargo test -p multifractal-workflow --test bribery_case_adversarial -- --nocapture
```

## 5. Related binary: `crown-local-cli`

`crown-local-cli` (same crate, `src/bin/crown-local-cli.rs`) drives a *different* fixture
("email needs a response") through the **entire LOCAL crown witness**
(`F02 -> F03 -> F08 -> F09 -> F10 -> F11 -> F18 -> F19 -> F02(re-admit) -> F24 -> F21 -> F25`),
including a real `f18_broker_law::Broker` receipt, F24 OCEL construction, and F25 replay —
no Erlang/`escript` dependency. It is architecturally the closest existing precedent for what a
Stage-3 `crown-bribery-case` extension would look like, but it is not the bribery-case fixture
and does not read `case.ttl`/`hook.ttl`. See that binary's own `--help` output for its input
contract.

## 6. Standing (end-to-end, as of this session)

**PARTIAL.** `crown-bribery-case` genuinely, verifiably composes F02 admission through F14 AIR
compile (live-verified this session, byte-identical across two independent runs). The
Stage-3 external tail this task's brief described (F15 real-Erlang-`escript` dispatch -> F18
Broker -> F20 external dispatch -> F02 re-admission -> F21/F24/F25) is **BLOCKED**: a prior
session's attempt to extend `crown-bribery-case.rs` with that tail was interrupted by plan mode
before any code was written (see `/Users/sac/.claude/plans/sequential-cooking-metcalfe-agent-a3fb26280347f16e8.md`
for the full, still-unimplemented design). `crown-bribery-case.rs` contains zero references to
`f18_broker_law`, `f16_otp_runner`, `f20_external_dispatch`, or `f25_receipts_replay` today
(confirmed by grep this session). The 7-scenario adversarial suite in this runbook's companion
test file therefore exercises real, tested, in-process library code for the Broker/replay
scenarios (1, 2, 4b, 7) rather than the CLI binary itself for those specific scenarios — each
test's own doc comment states which category it falls into. Scenarios 3, 4a, 5, and 6 run
directly against the real, wired CLI chain (F02, hook/evidence-catalog projection, F08
`run_pipeline`).

## See Also

- `crates/multifractal-workflow/tests/bribery_case_adversarial.rs` — the 7-scenario adversarial
  refusal suite this runbook accompanies.
- `crates/multifractal-workflow/tests/bribery_case_fixture.rs` — Stage-1 F02/hook live
  verification (this file's own `build_policy`/`derive_obligations` helpers mirror it).
- `crates/multifractal-workflow/src/bin/crown-bribery-case.rs` — the CLI itself (see its own
  module doc for the exact chain driven and file-by-file RDF lifecycle authority).
- `crates/multifractal-workflow/src/crown_external.rs` — the real, independently-tested (but not
  yet CLI-wired) F15-F25 external witness tail this runbook's Standing section refers to.
