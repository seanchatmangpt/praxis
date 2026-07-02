# Day 7 Receipt — The Genesis Release & Chain Closure

**Date:** 2026-07-02
**Program:** GENESIS Day 7 — version + changelog, publish the lawful subset, Book Three (*The First Week*) written from the receipts, and close the seven-day chain so Day 7 provably commits Day 1's hash.

> A receipt that hides its gaps is not a receipt; it is a press release. This one reports what the week actually sealed, not what the program hoped it would.

## What the release shipped

- **Version:** `26.7.2` (CalVer; bumped from `26.6.30`, monotonic and distinct from `wasm4pm 26.7.1`). Recorded in root `Cargo.toml`.
- **Changelog:** `CHANGELOG.md` — Keep-a-Changelog format, generated from the week's conventional commits, with working-tree-only lines explicitly marked `(working tree)`.
- **Frontier matrix — FINAL** (`target/frontier-report.json`, matrix `cphy-frontier`):
  - **286 cells**, **30 evaluated** (Admitted/Executed + Refused), **coverage 0.105**, **pass_rate 1.0**, **0 failures**.
  - Every capability source explored this week is a cell: Admitted with the socket it landed in, or Refused with reason + salvage (e.g. `bytestar` refused — C stubs/dormant, design ported; `unibit` refused — dirty tree, admit4 semantics ported; `stpnt` refused — no license, taxonomy ported). Refusals are receipted cells, not omissions.
- **Book Three of the Primer — *The First Week*:** `docs/fiction/BOOK_THREE_THE_FIRST_WEEK.md`. Mythic-technical-nonfiction, written strictly from `O*` (the admitted, receipted record) and not from memory (`O`). It states its own honesty constraint in the prologue: two of seven days sealed, and the four unsealed days are narrated as silences rather than invented. Prior publication cross-referenced: *The Chatman Equation* (v1.2.0, Nov 9 2025); C-era prehistory `bytestar`, refused this week.

## Final verification (build & test, one last time)

Command: `cargo build --workspace --all-features` then `cargo test --workspace --all-features` (the exact gate the task names).

- **Build:** `BUILD_EXIT=0` — the entire workspace compiles green with `--all-features`.
- **Tests (parallel, the default):** `TEST_EXIT=101` — **one** failure: `my-conforming-project` `ops::tests::receipt_validate_on_clean_ledger_is_ok` (`src/ops.rs:1502`, `assert_eq! left=false right=true`).
- **Same test in isolation** (`--exact`, `--test-threads=1`): **passes**.
- **Whole `my-conforming-project` lib suite single-threaded:** **127 passed, 0 failed.**

**Root cause (receipted, not papered over):** `with_receipt_noun_signing_key()` (`src/ops.rs:1434`) mutates the **process-global** env var `PRAXIS_SIGNING_KEY` and returns a `Mutex` guard, but that mutex only serializes tests that *opt in* by calling the helper. Under `law-signed` (pulled in by `--all-features`) a parallel test that reads/writes the same global env var races with it, so the clean-ledger validate intermittently sees an unsigned/mis-signed state and returns `ok=false`. It is a **test-isolation defect** in sibling-owned receipt tests, surfaced by the non-quiescent parallel run — **not** a logic regression, and it does not touch the manifest hashes, the chain links, or the seal (this closer added only docs + JSON). `src/ops.rs` is in the live-edited set (tasks #47/#49/#51/#54 `in_progress`); the correct fix is a sibling refactor (inject the signing key rather than mutate a global env var, or make every signing-dependent test hold the shared guard), not a closer's edit to a file being restructured concurrently. Recommended fix: add the `with_receipt_noun_signing_key()` guard to every receipt test that issues under `law-signed`, or thread the key through `receipt_issue_payload` instead of the environment.

**Consequence for the release:** the stated green-test gate is **not met** under the exact command (`cargo test --workspace --all-features`), so the irreversible release actions below are refused on that ground as well as the dirty-tree ground.

## Publication / irreversible public actions — REFUSED, receipted

Consistent with the standing rules (dry-run before publish; refusal-receipt when the precondition fails) and with the Day 1 / Day 2 / Day 7-release doctrine, the **irreversible public actions were refused**, not executed:

| Action | State | Reason (salvage) |
|---|---|---|
| `git push` | **REFUSED** | Working tree is **non-quiescent**: sibling agents editing live (tasks #47, #49, #51, #54 `in_progress`), dozens of modified/untracked files unrelated to this seal. CHANGELOG's own precondition requires "a committed, quiescent, build-green tree." Salvage: additive artifacts committed locally; push is a one-command follow-up once the tree quiesces. |
| release tag `v26.7.2` | **REFUSED** | Tagging a "Genesis release, chain sealed" over a **2-of-7** chain would assert a completeness the record does not have. Salvage: version + changelog are in the tree; the tag is a follow-up over the closed, quiescent state. |
| `cargo publish` | **REFUSED** | Not run on a dirty tree; publish is irreversible and registry-visible. Salvage: dry-run is the documented next step; per-crate publishability already assessed in the Day 1 publication matrix. |

These refusals are the doctrine working, not the doctrine failing. Silent execution of an irreversible action on an unprovable tree is the thing the whole architecture exists to prevent.

## The chain closure

**Method (matches Days 1–2 exactly):** `manifest_hash = blake3(json.dumps(obj, sort_keys=True, separators=(",",":")))` with the `manifest_hash` field removed. The **seal** is `seal_hash = blake3(canonical_json(days_array))` over the ordered `[{day, manifest_hash}]` list — the same canonicalization, applied one level up.

**Manifests read:** `MANIFEST_DAY_1.json`, `MANIFEST_DAY_2.json`. Manifests for Days 3–7 **do not exist** — they were never emitted.

**Per-link verification (every link checked):**

| Day | `manifest_hash` | Recomputed | `prev_day_hash` | Expected | Link |
|---|---|---|---|---|---|
| 1 | `f6ec2387…5fa5dba` | **GENUINE** | `0`×64 | `0`×64 (genesis anchor) | **INTACT** |
| 2 | `cb184872…ea4e8c9` | **GENUINE** | `f6ec2387…5fa5dba` | Day 1 `manifest_hash` | **INTACT** |

Both stated hashes were independently recomputed from their own canonical JSON and **match** — the manifests are genuine, not asserted. Both existing links are **cryptographically intact**.

**GENESIS_SEAL** (`docs/genesis/GENESIS_SEAL.json`):

```
seal_hash  = 9c666317edb61ace94d6cc7cc5114a15effd80caa596dbe370191ee2e8dfd34f
algorithm  = blake3
days       = [ {day:1, f6ec2387…}, {day:2, cb184872…} ]
```

The seal reproduces from:
`blake3('[{"day":1,"manifest_hash":"f6ec2387…5fa5dba"},{"day":2,"manifest_hash":"cb184872…ea4e8c9"}]')`.

Because the seal is computed over Day 1's hash, **Day 7's seal provably commits Day 1's hash** — the week-as-one-object property holds, over the two links that genuinely sealed.

## Honest closing assessment — program vs. reality

| Day | GENESIS.md program | What the record shows |
|---|---|---|
| 1 | Foundation & publication + genesis manifest | **Sealed.** Manifest `f6ec2387…`. Narrative sections of the Day-1 receipt left `_Pending final phase._` — a receipted gap, not a hidden one. |
| 2 | Revenue physics end-to-end + RevTAC v0 + MRR | **Sealed.** Manifest `cb184872…`, full Day-2 receipt, `revenue-demo` live, 486 tests green at receipt time. |
| 3 | Adversarial hardening (fuzz + mutation) | **Work landed, not sealed.** `crates/praxis-core/tests/{fuzz_boundaries,mutation_chain}.rs`, `powl2-decompose` present; no `MANIFEST_DAY_3.json`. Task #49 still `in_progress`. |
| 4 | The Membrane ships (MCP+ productized) | **Work landed, not sealed.** MCP lawobject server + cache present (Lane 8b tasks done); no external-agent transcript, no `MANIFEST_DAY_4.json`. |
| 5 | Manufacture at scale (ggen amplified) | **Partial, not sealed.** Corpus ingest task #47 `in_progress`; no manufacture-ratio receipt, no `MANIFEST_DAY_5.json`. |
| 6 | Mission physics beyond revenue (church pack) | **Work landed, not sealed.** Church-operations pack done (#46), mission substrate (#50); `tests/two_domains.rs` + `docs/MISSION_PHYSICS.md` still pending; no `MANIFEST_DAY_6.json`. |
| 7 | The Genesis release + closed chain + Book Three | **Release phase + Book Three delivered; chain closed over what exists.** Version/changelog/frontier-final present; Book Three written; seal computed. Irreversible public actions (push, tag, publish) **refused** on a non-quiescent tree. No `MANIFEST_DAY_7.json` emitted. |

**What the seven days actually delivered:** a working, receipted revenue-physics substrate (observe→propose→goal→plan→admit→receipt, proposal-hash-bound), a generic mission substrate with a second domain pack, a capability frontier matrix at pass_rate 1.0 with every refusal receipted, an MCP membrane, adversarial-hardening scaffolding, three books of the Primer, and **two cryptographically sealed, mutually-chained daily manifests**.

**What remains:** manifests for Days 3–7 (the days did work but never sealed); the Day-3 mutation/fuzz kill-report; the Day-4 external-agent-through-the-membrane transcript; the Day-5 manufacture-ratio receipt; `tests/two_domains.rs` + `docs/MISSION_PHYSICS.md`; and the irreversible release actions (push, tag `v26.7.2`, `cargo publish`) — all deferred pending a committed, quiescent, build-green tree.

**Is the chain intact?** **Yes, over its sealed length — and it is honest about that length.** Two links, both genuine, both intact, sealed into one object whose hash commits the genesis anchor. It is a two-link chain presented as a two-link chain. The program asked for seven; the week sealed two; this receipt refuses to fabricate the missing five. That refusal — not a false "all seven done" — is the actual payoff of the doctrine.

## Chain

- **Sealed days:** 1 → 2 (Day 1 `prev = 0`×64 genesis anchor; Day 2 `prev =` Day 1 `manifest_hash`).
- **Seal:** `9c666317edb61ace94d6cc7cc5114a15effd80caa596dbe370191ee2e8dfd34f` (`docs/genesis/GENESIS_SEAL.json`).
- **Unsealed:** Days 3, 4, 5, 6, 7 — receipted as gaps above, not hidden.
