# Onboarding: From Story to Receipt in 30 Minutes

A first-contact path for a stranger to Praxis. You will read the fable, read the
thesis abstract, run one demo, and verify one cryptographic seal. Every step lists
the command, what you should see, and a time estimate. Steps whose prerequisite is
missing in this working tree are marked `deferred: <reason>` — not invented.

All paths are relative to the repo root (`/Users/sac/praxis`).

---

## Step 1 — Read the fable (8 min)

```sh
open docs/fiction/BOOK_TWO_THE_LAW_OF_WORK.md   # or: less docs/fiction/BOOK_TWO_THE_LAW_OF_WORK.md
```

**You should see:** Book Two of the Primer, *The Law of Work* — the narrative frame
for why work must produce receipts and why refusals are first-class outcomes. Read
for the vocabulary: admit, refuse, receipt, chain.

## Step 2 — Read the Projection Principle abstract (4 min)

```sh
less docs/thesis/projection_thesis.tex    # the abstract is near the top
open docs/thesis/projection_thesis.pdf    # compiled PDF, same content
```

**You should see:** the abstract of the Projection Principle thesis — the formal
claim the fable dramatizes. Read only the abstract; the rest is optional.

## Step 3 — Run the revenue demo (5 min)

The recipe actually present in the `justfile` is named `revenue-demo`:

```sh
just revenue-demo
# runs: cargo run --quiet --features proposer --bin revenue_demo
```

**You should see:** the Day-2 revenue pipe execute end-to-end (propose → plan →
judge → admit) and print a receipt with a deterministic `chain_hash`. First run
includes compile time; budget most of the 5 minutes for cargo.

## Step 4 — Inspect one receipt (3 min)

A real, checked-in receipt:

```sh
less docs/genesis/DAY_2_RECEIPT.md
```

**You should see:** a human-readable receipt for Genesis Day 2, with its manifest
counterpart at `docs/genesis/MANIFEST_DAY_2.json`. Note that the receipt records
what was verified, not what was hoped.

## Step 5 — Inspect one refusal (4 min)

- `docs/fable/FABLE_REFUSALS.md` — the fable's own refusal register: 14 claims
  the story declined to make, each with reason and the evidence that would
  admit it. (Written concurrently with this document; both now exist.)
- `docs/GENESIS.md` — the seven-day program table, including the Day-7 row
  where push/tag/publish were **refused** on a non-quiescent tree — the
  refusal Chapter 14 narrates.

What *does* exist and covers the same ground — the Day-7 push refusal:

```sh
less docs/genesis/DAY_7_RECEIPT.md
```

**You should see:** the section "Publication / irreversible public actions —
REFUSED, receipted". The release gate (`cargo test --workspace --all-features`)
was not green, so the irreversible publish actions were refused and the refusal
itself was receipted — the system's core behavior demonstrated on itself.

## Step 6 — Inspect the Genesis seal (3 min)

```sh
python3 -c "import json; d=json.load(open('docs/genesis/GENESIS_SEAL.json')); \
print(d['seal_algorithm']); print(d['seal_hash'])"
```

**You should see** the seal hash:

```
a194af72faec7a42a125f7b18ba0ae6da00c23bc1486e1bfc893e84d5b2f196d
```

The seal closes the seven-day chain: `days_with_manifest`, per-day entries, and a
topology under a stated `seal_algorithm`. Day 7 provably commits Day 1's hash.

## Step 7 (optional) — The fleet overlap curve receipt (3 min)

```sh
python3 -m json.tool target/synthesis-fleet-receipt.json | head -40
```

**You should see:** `n: 10000` and a `points` array sweeping novelty from 1.0
downward — as `core_hits`/`replayed_nodes` rise, `pipelines_per_sec` climbs
(~1.9k/s at full novelty to ~11.6k/s at novelty 0.1): the overlap curve showing
replay beats recomputation. Note this file lives in `target/` (build output); if
absent on a fresh clone, regenerate via the synthesis fleet run, or skip.

Bonus: `just membrane-demo` drives the same revenue pipe through the MCP server
over raw JSON-RPC (`scripts/membrane_demo.sh`).

---

**Total:** ~30 minutes. You have now read the story, the claim, run the machine,
seen an admission receipt, a refusal receipt, and verified the seal that binds
the week together.
