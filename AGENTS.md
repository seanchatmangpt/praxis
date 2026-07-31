# Praxis — Authoritative Agent Contract (v26.7.30)

This file is the repository-wide operating contract for every coding agent,
regardless of model, vendor, editor, shell, or orchestration system.

`AGENTS.md` is the sole normative agent document for this repository. Other
agent files are compatibility pointers or path-scoped refinements. A more
specific `AGENTS.md` below a working directory overrides this file only for
that subtree and may not weaken these laws.

Hosted agents using the GitHub connector or an ephemeral cloud shell must also
read `CHATGPT-CLOUD-AGENTS.md`.

The compact manufacturing law is:

> `A = μ(O*)`, where `O*` is admitted, aligned, complete-enough, grounded, and
> bounded observation; `μ` is a lawful transformation; and `A` is an artifact
> with standing. Every actuation must emit a recomputable receipt `R`.

The hard invariant is:

> **Zero unreceipted actuation.**

---

## 1. Instruction precedence

When instructions conflict, use this order:

1. Platform and safety constraints.
2. The user's current request and explicit scope.
3. The nearest path-specific `AGENTS.md`.
4. This root `AGENTS.md`.
5. Current source, manifests, build recipes, tests, and generated ledgers.
6. Other documentation and historical notes.

The live repository outranks stale summaries. Unfamiliar state is not invalid
state. Record drift instead of silently choosing the convenient source.

---

## 2. Law-state execution model

Every task follows this state machine:

1. **Parse** — identify objects, boundaries, requested outcome, exclusions, and
   acceptance commands.
2. **Orient** — inspect tools, permissions, mounts, repository doctrine,
   current branch, working tree, and remote state.
3. **Resolve** — bind the exact base ref and commit SHA.
4. **Materialize** — obtain the exact local tree by the strongest available
   transport.
5. **Admit or refuse** — construct `O*`; reject ambiguous, stale, malformed,
   unauthorized, or incomplete observations with a typed reason.
6. **Diagnose and repair** — preserve the existing architecture before making
   a bounded change.
7. **Actuate** — change only admitted objects through an authorized path.
8. **Receipt** — record exact inputs, outputs, commands, hashes, commit, and
   observed results.
9. **Replay or hook** — prove the receipt recomputes or that the relevant hook
   rejects tampering.

No layer may convert a refusal into success. Wrappers, CLIs, UIs, generators,
or documentation may not override the real boundary.

---

## 3. Claims may not exceed evidence

Keep these facts distinct:

- **Observed** — source, metadata, logs, or artifacts were read.
- **Executed** — a command ran against the claimed checkout or artifact.
- **Changed** — repository state was modified.
- **Verified** — an independent check recomputed or exercised the claim.
- **Inferred** — a conclusion follows from observed evidence.
- **Blocked** — a required boundary could not be reached.

Reading a test proves declaration, not execution. A green workflow proves only
its exact commit and command surface. A receipt file proves bytes exist, not
that its digest recomputes. A coherent diagram does not prove a runtime edge.

Use the standing lattice exactly:

- **PARTIAL_ALIVE** — a real checkpoint executed, with named missing closure.
- **ALIVE** — the claimed boundary executed and its required receipt verified
  in the current session.
- **BLOCKED** — a required boundary is known and unreachable; name the failed
  edge and the minimal missing capability.
- **BUILD_BROKEN** — the exact tree was materialized, but the build or required
  verifier fails.
- **UNKNOWN** — the observation was not obtained or is stale. `UNKNOWN` is not
  admitted evidence.
- **UNSUPPORTED** — the capability is intentionally outside the contract.
  `UNSUPPORTED` is not refusal and must not be represented as failure.

Never promote a checkpoint to crown closure. Never require crown completeness
from a checkpoint claim.

---

## 4. Chesterton fence and refutation discipline

Before replacing, deleting, or refuting an existing mechanism:

1. Preserve the strongest current statement `S`.
2. Identify the fence: why the mechanism exists, which invariant it protects,
   and which consumers depend on it.
3. Reconstruct `O*` from the exact tree and live evidence.
4. Apply the lawful transformation `A = μ(O*)`.
5. State exclusions and unsupported surfaces.
6. State a falsifier that would disprove the new claim.
7. Extend only after equivalence or intentional non-equivalence is proven.

Adjacency is not refutation. A refutation must match the same objects,
morphisms, admission rules, closure boundary, actuation path, receipts,
replay behavior, exclusions, and failure semantics.

---

## 5. Repository and git transport

Resolve the exact base SHA before editing. Preferred transport order:

1. verified existing checkout;
2. exact-SHA archive;
3. clone or fetch;
4. bundle;
5. workflow artifact;
6. Git tree/blob reconstruction;
7. dependency-closed sparse tree.

One failed transport edge does not imply repository access is blocked. Use the
next lawful edge and report the transport used.

Before local edits:

```bash
git status -sb
git diff --stat
git diff -- <intended-paths>
git rev-parse HEAD
```

Rules:

- preserve unrelated user and agent changes;
- use a dedicated worktree or branch when collision is possible;
- never use `git add .` in a mixed tree;
- stage explicit paths and inspect the staged diff;
- do not rebase shared fleet branches;
- do not hand-resolve generated outputs;
- default publication to a draft pull request;
- bind every receipt and PR report to the exact base and head SHAs.

An exact tree without `.git` is sufficient for inspection and local
verification. Publication may use blob → tree → commit → ref → PR when a
normal push path is unavailable.

---

## 6. Rust and implementation invariants

Violation is a defect, not a review preference.

1. **No panics or silent defaults in fallible code.** Ban `.unwrap()`,
   `.expect()`, `panic!()`, `.ok()`, and `.unwrap_or_default()` unless an
   assertion-only test context proves they cannot hide a runtime failure.
2. **Typed refusals.** Every expected failure crosses the public boundary as a
   stable typed refusal with at least one positive and one negative test.
3. **Receipts are computed.** Use BLAKE3 over stable canonical bytes; never
   assert or fabricate digests.
4. **No ambient time in deterministic paths.** Use graph-carried OWL-Time or
   explicit logical time. Ban wall-clock values from hash, ordering, planning,
   replay, and receipt identity paths.
5. **Deterministic collections and serialization.** Prefer `BTreeMap`,
   `BTreeSet`, explicitly sorted vectors, fixed seeds, and stable encoding.
6. **Closed vocabularies.** Unknown predicates, operation kinds, breed IDs,
   and status values are refused by name.
7. **No hidden quadratic behavior.** Document asymptotic bounds on critical
   algorithms and benchmark the relevant hot path.
8. **Unsafe code is exceptional.** Every `unsafe` block requires a `// SAFETY:`
   proof and an owning verifier.
9. **No TODO/FIXME contract gaps.** Implement, type the block, or record the
   work as a governed open item outside executable success paths.
10. **No documentation-only enforcement.** Critical policy must have a
    structural checker, test, hook, or CI gate.

---

## 7. Generated-surface law

Generated files are projections of admitted sources. They are never primary
inputs and must not be hand-edited.

The governing relation is:

> admitted graph/spec → generator → deterministic filesystem projection →
> receipt → replay verification

For every generated surface:

- identify the admitted source;
- identify the generator command and version;
- run the generator rather than editing output;
- verify idempotence with a second run;
- refuse unexpected drift;
- preserve the receipt chain;
- keep runtime outputs out of version control unless the repository doctrine
  explicitly governs them as committed fixtures.

The architecture slogan is binding where those components apply:

> **ggen renders; Lean admits; mfact certifies.**

A generator existing in source does not prove a generated artifact is current.
A committed generated file does not prove replay unless regeneration is clean.

---

## 8. Cognition and composition contracts

Cognition breeds, planners, hooks, and process-mining algorithms are governed
by `.claude/rules/cognition-contracts.md`.

Minimum closure for a breed or algorithm claim:

- registry identity and dispatcher reachability;
- public CLI/API/WASM surface or a typed reason for absence;
- admitted input and bounded output;
- stable refusal codes;
- deterministic trace under fixed input and seed;
- one positive case;
- one typed negative case;
- one invariant/property case;
- projection-authority test;
- composition-state test when used downstream;
- receipt and replay/tamper rejection.

A catalog entry is not execution. Per-breed validity is not composed-pipeline
validity. Proposal visibility is not authority. Observation is not actuation.

---

## 9. Real-boundary and anti-theater testing

Primary evidence paths may not be replaced by mocks, stubs, fabricated
telemetry, fake receipts, hardcoded success, or synthetic proof artifacts.

Use Chicago-style tests at the owning boundary. Unit tests are permitted for
pure logic, but they cannot support a boundary or closure claim by themselves.

A proof-oriented test must have teeth:

1. run the real boundary;
2. preserve exit status and outputs;
3. recompute hashes and receipt chains;
4. corroborate across multiple surfaces where applicable: execution,
   telemetry, state, process, and causality;
5. corrupt a disposable copy or mutate the implementation;
6. observe the verifier reject it;
7. restore valid state and replay.

A printed pass count followed by a non-zero exit, crash, or teardown failure is
not a clean pass. Skipped is neither passed nor failed.

---

## 10. Verification ladder

Run the narrowest owning verifier first, repair failures, then expand:

1. unit;
2. integration;
3. end-to-end;
4. chaos or tamper;
5. stress;
6. benchmark;
7. independent verifier report.

`ALIVE` requires observed execution. A command name does not prove its scope;
inspect the recipe or manifest before treating it as evidence.

Repository starting points remain:

```bash
just test-changed
just verify-all
```

These commands are not universal proof. Add the exact package, target,
feature, runtime, generated-surface, and artifact checks required by the diff.

---

## 11. Gall checkpoint discipline

Complex systems must grow from working systems. Each increment must preserve a
working receiptable path.

For every checkpoint:

- define its admitted input and emitted artifact;
- name the boundary it crosses;
- emit a receipt;
- state its standing without inflating downstream closure;
- preserve replay safety;
- make retries idempotent;
- isolate one-use finalizers and remove them after successful exact-head use;
- expose bounded failure receipts instead of swallowing finalizer failures.

Exact-head finalization must verify the intended commit before actuation and
must refuse if the head moved. Consolidation is not complete until the merged
head passes the owning verifier and the finalizer's receipt is replayable.

---

## 12. Release and capability standing

Capability standing is manufactured from evidence, not manually assigned.
Release-law control must bind:

- exact package identity and version;
- exact commit;
- generated and source artifact hashes;
- runtime boundary evidence;
- receipt-chain verification;
- clean-install or clean-consumer evidence where applicable;
- typed refusal ordering;
- logical time and deterministic serialization;
- an exact-head finalizer or equivalent controlled actuator.

No release or capability is `ALIVE` from enumeration alone. A certificate is
not closure unless its embedded hashes recompute against the exact artifacts
and commit.

---

## 13. Security and artifact hygiene

Never commit credentials, private keys, `.env` files, PII, host-specific
secrets, unredacted sensitive logs, accidental large outputs, or runtime
receipts that expose local paths.

Validate data crossing host, subprocess, WASM, network, RDF, planner, and
plugin boundaries. Malformed, recursive, oversized, unauthorized, or
adversarial inputs must produce typed refusals rather than panic, truncation,
or false success.

---

## 14. Required final receipt

Every implementation report and pull request must state:

```text
State:
Repository:
Base ref and SHA:
Head branch and SHA:
Files changed:
Commands actually executed:
Validation observed:
Tamper/falsifier result:
Commands not executed:
Known exclusions:
Receipt or artifact locations:
```

No unreceipted actuation. No closure claim without exact-boundary evidence.
