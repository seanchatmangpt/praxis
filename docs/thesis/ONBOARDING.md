# Onboarding — 30 minutes to a working mental model

For a new engineer. By the end you will have built the `praxis` CLI, run one verb
from each of the four core nouns (**law**, **plan**, **receipt**, **propose**),
read a receipt, verified a tamper-evident chain, and seen exactly where the
thesis math lives in the code. Every command below is real and traceable to a
paper result or a `#[verb]` in the tree.

Read [`README.md`](./README.md) first for the one-paragraph summary of
`A = μ(O*)`. Repo root is `/Users/sac/praxis`.

---

## 0. Clone and build (about 5 min)

```bash
cd /Users/sac/praxis           # already cloned in this environment
cargo build                    # default features = ["typestate"]
```

The default binary is `target/debug/my-conforming-project` (the package name in
`Cargo.toml`; the docs call it `praxis`). Set an alias so the rest of this guide
reads cleanly:

```bash
export BIN=target/debug/my-conforming-project
$BIN --version
```

To exercise the signing and OCEL paths later, build with more features:

```bash
cargo build --features law-signed,law-ocel   # ed25519 receipts + OCEL export
```

The full surface (`repl,lsp,mcp,ggen,proposer,...`) is `--features all-features`;
you do not need it for this walkthrough.

---

## 1. `law` — the admission gate (BRCE B1: `A = μ(O*)`)

The `law` noun runs the Raw → Validated → Admitted → Receipt pipeline. Verbs are
in `src/verbs/law.rs`. First, watch the gate **refuse** an observation that
carries an unmet obligation — a refusal is an *output*, not an exception (this is
the Admission Algebra of Part I; `crates/praxis-core/src/refusal.rs`):

```bash
$BIN law judge \
  --payload '{"value":{"action":"wire-transfer","amount":250000},"obligations":[{"type":"blocking_constraint","reason":"dual-control approval absent"}]}' \
  --law default --format json
```

Expect `"verdict": "halted"` with the unmet obligation inspectable in the JSON.

Now the **same action with its evidence obligation met** passes judgment and is
admitted (this is `x = adm(o) ≠ ⊥`, the precondition of B1):

```bash
LAWFUL='{"value":{"action":"wire-transfer","amount":250000,"actor":"alice"},"obligations":[{"type":"evidence_required","evidence_type":"dual-control"}],"evidence":["dual-control"]}'

$BIN law judge  --payload "$LAWFUL" --law default   --format json   # -> "validated"
$BIN law admit  --payload "$LAWFUL" --policy default --format json   # -> "admitted"
```

**Math shown:** BRCE B1, `00_foundations.tex` `def:brce`; admission monoid /
refusal composition, `crates/praxis-core/src/refusal.rs` (`compose_denials`).

---

## 2. `receipt` — manufacture leaves a hash-committed proof (B3)

Ask `law receipt` to run the full judge → admit → receipt pipeline and emit a
BLAKE3 chain hash that commits to payload + previous hash + meta. Fixed
`ts_ns`/`instruction_id` make it deterministic — re-run it and the hash is
identical:

```bash
RCPT="${LAWFUL%\}},\"ts_ns\":42,\"instruction_id\":7,\"activity_idx\":2,\"node_kind\":0}"
$BIN law receipt --payload "$RCPT" --format json      # note the 64-hex chain_hash
$BIN law receipt --payload "$RCPT" --format json      # same input -> same chain_hash
```

**Read one receipt.** Issue a small persisted lifecycle into a scratch ledger,
then show it:

```bash
mkdir -p /tmp/praxis-onboard/receipts && cd /tmp/praxis-onboard
$BIN receipt issue --payload "$RCPT" --format json    # writes receipts/*.jsonl
$BIN receipt show --last 0                            # print the ledger
cd /Users/sac/praxis
```

Each line is a `ReceiptRecord` — inspect its fields in
`crates/praxis-core/src/receipt_record.rs` (`payload_hash_hex`,
`prev_chain_hash_hex`, `chain_hash_hex`, `andon`).

**Math shown:** Receipt totality B3 and the Faithful Projection Theorem
(Part IV `thm:faithful`, Part II cryptography). Chain fold:
`src/chain.rs` `fold()` = `BLAKE3(prev_hex_bytes ‖ event_bytes)`.

---

## 3. Verify a chain — and watch a one-byte tamper get caught

`receipt validate` recomputes the chain, checks linkage/monotonicity, and runs
POWL token-replay conformance (the marking-geometry check of Part III). Against
the untampered ledger it passes:

```bash
ABS_BIN=/Users/sac/praxis/$BIN
( cd /tmp/praxis-onboard && "$ABS_BIN" receipt validate --format json )
```

`receipt validate` defaults to the ledger in the current directory, so run it
from the folder that holds `receipts/` (hence the subshell `cd`).

For the full "flip one byte in a copy of the `.jsonl` store and see validation
reject it at the chain-integrity stage" demo, run the narrated end-to-end script,
which also does the tamper attack for you:

```bash
cargo build                                   # ensure the binary exists
PRAXIS_BIN=$BIN bash scripts/walkthrough.sh   # exits 0 iff every claim holds
```

The tamper logic (mutate one hex digit of `payload_hash_hex`, expect rejection)
is Step 4 of `scripts/walkthrough.sh`; the narrative is `docs/WALKTHROUGH.md`.

**Math shown:** tamper-evidence = collision commitment (Part II); replay
conformance fitness = 1 (B4) via `PowlReplayVerifier` in
`crates/praxis-core/src/replay_adapter.rs`.

*(ed25519 signature verification is a real verb, `law verify-signature` in
`src/verbs/law.rs`, but is gated behind `--features law-signed` and expects a
`PRAXIS_SIGNING_KEY`; skip unless you built with that feature.)*

---

## 4. `plan` — grounding and solving (Planning Geometry)

The `plan` noun (`src/verbs/plan.rs`) solves classical/temporal PDDL8 problems
and can execute a plan through the admission gate to a receipt. The
self-contained self-test manufactures and solves the `lawobject` domain (requires
the `ggen` feature):

```bash
cargo run --features ggen --bin my-conforming-project -- plan lawobject --format json
```

If you did not build `ggen`, inspect the verbs instead — `plan route`,
`plan solve`, `plan analyze`, `plan execute` all take a single JSON `--payload`:

```bash
$BIN plan solve --help
```

**Math shown:** grounding is bounded (Part III `thm:bounded-ground`); the marking
polytope `m = m0 + N·x` and Farkas separation (`def:polytope`, `thm:farkas`).

---

## 5. `propose` — proposals are observations, not authority

The `propose` noun (`src/verbs/propose.rs`, `--features proposer`) emits ranked
proposals. The key invariant to internalize: **a proposal is an untrusted
observation** — it must itself re-enter the admission gate before anything acts
on it (this is why it is a *noun beside* `law`, not inside it). Quickest look:

```bash
cargo run --quiet --features proposer -- propose goal --help
# or the packaged demo:
just revenue-demo        # runs the revenue_demo binary end-to-end
```

**Math shown:** admission is the only authority (`A = μ(O*)`); everything else,
including a proposer's ranking, is an `O` that must pass `adm` first.

---

## 6. Where the math lives — a 2-minute tour

Open these in order; each is small and re-exported from
`crates/praxis-core/src/lib.rs`:

- `crates/praxis-core/src/law.rs` — `LawObject`, `Obligation`, `Admit`, `Judge`.
  This *is* `adm` and the obligation battery (BRCE B1).
- `crates/praxis-core/src/refusal.rs` — the admission monoid: `compose_denials`,
  `denial_lane`. (Part I, Admission Algebra.)
- `crates/praxis-core/src/quarantine.rs` — `RiceQuarantine`: undecidable
  observations refused at the boundary (Part 0 `thm:rice`).
- `crates/praxis-core/src/receipt_record.rs` + `src/chain.rs` — the receipt and
  its BLAKE3 chain. (Part II; Faithful Projection, Part IV.)
- `crates/praxis-core/src/replay_adapter.rs` — `PowlReplayVerifier` /
  `PowlReplayFrame`: conformance replay and the node/token status projection.
  (Part III polytope; the status-byte view the keystone scales in Part V.)

If a claim here ever stops matching the code, the code is the source of truth —
fix forward and update this file. One brief known gap: the `agent8` /
`crates/agent8` fleet-projection crate described in the keystone (Part V) and in
`workflows/genesis/day4.js` has **not** landed under `crates/` yet; see the
honesty note in [`README.md`](./README.md).

---

## Checklist

- [ ] `cargo build` succeeds; `$BIN --version` prints.
- [ ] `law judge` halts an unlawful payload and validates a lawful one.
- [ ] `law receipt` emits a 64-hex `chain_hash`, identical on re-run.
- [ ] `receipt show` prints a ledger you issued.
- [ ] `scripts/walkthrough.sh` exits 0 (chain verified, tamper caught).
- [ ] You have opened `law.rs`, `refusal.rs`, `receipt_record.rs`, and
      `replay_adapter.rs` and matched each to a paper result.
