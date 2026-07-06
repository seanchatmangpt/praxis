# The biggest math in Praxis, explained without assuming you know any math

This is a companion doc set to [`docs/diataxis`](../diataxis/README.md) (which documents
the `ggen` tool). This one documents the *mathematical* side of the project: the theorems
in the "Math Manufacturing" thesis (`docs/thesis/`) that are actually the load-bearing
ideas behind Praxis's design, explained for a reader with no math background.

It follows the same [Diataxis](https://diataxis.fr) structure:

- **Explanation** (`explanation/`) — plain-English retellings of the five biggest results,
  one per thesis paper, in the order you'd want to read them in. Start here.
- **Reference** (`reference/`) — a flat table of every "biggest" theorem: what it says in
  one sentence, whether it's been machine-checked by the Lean 4 proof assistant (and in
  which lane — see below), and exactly where to find the real statement and proof.
- **Tutorials** (`tutorials/`) — a hands-on walkthrough that lets you verify one of these
  theorems yourself, on your own machine, with zero math background required.

## The one idea that ties all five together

Every result here answers some version of the same question: **"why should anyone trust
a decision this system made, without a human checking it by hand?"** The five papers
answer it for five different kinds of decision — refusing bad input, sealing a receipt,
choosing a plan, running at scale — but the shape of the answer is always the same:
*don't try to prove the system understood the request; prove instead that its mechanical
check is airtight, and that the check's own failure would be independently, cheaply
detectable.*

## A word on "machine-checked"

Some of these theorems are proved on paper, in the `.tex` files, by a human mathematician
(with named lemmas, hypotheses, and proof steps, like any math paper). Separately, a
subset of the corpus's theorem *statements* have also been translated into the Lean 4
formal language and checked by Lean's own kernel — a computer program that will refuse
to accept a proof with a gap in it, no matter how convincing the English argument reads.
Where a theorem below has been Lean-checked, that's stated explicitly, in which lane
(there are two: a "bare core" lane and a "Mathlib" lane, see
[reference/00-biggest-theorems-table.md](reference/00-biggest-theorems-table.md) for what
that distinction means), and where it hasn't (or where it was tried and failed), that's
stated just as explicitly. Nothing here claims a stronger form of verification than what
was actually run.

## Contents

### Explanation — start here, in order

| # | File | The paper it covers | The one idea |
|---|------|----------------------|--------------|
| 0 | [explanation/00-why-nothing-can-understand-you.md](explanation/00-why-nothing-can-understand-you.md) | `00_foundations.tex` | No program can decide what an input *means* — so admission has to work on syntax, never semantics. |
| 1 | [explanation/01-the-algebra-of-refusal.md](explanation/01-the-algebra-of-refusal.md) | `01_admission_algebra.tex` | Refusal reasons form an algebra (like addition or set-union) — so denials combine predictably instead of arbitrarily. |
| 2 | [explanation/02-receipts-that-cannot-lie.md](explanation/02-receipts-that-cannot-lie.md) | `02_receipt_cryptography.tex` | A tampered receipt chain either breaks the hash (astronomically unlikely) or gets caught at the exact record it was tampered. |
| 3 | [explanation/03-proving-a-plan-is-impossible.md](explanation/03-proving-a-plan-is-impossible.md) | `03_planning_geometry.tex` | You can *prove*, not just fail to find, that no valid plan reaches a goal — with a short, checkable certificate. |
| 4 | [explanation/04-trust-at-planetary-scale.md](explanation/04-trust-at-planetary-scale.md) | `04_projection_and_scale.tex` | The same admission check that works for one agent works, unchanged, for 64 at a time, at near-zero extra cost. |

### Reference — look things up

| File | Covers |
|------|--------|
| [reference/00-biggest-theorems-table.md](reference/00-biggest-theorems-table.md) | Every theorem discussed above: one-sentence claim, Lean-verification status in both lanes, exact `file:line` citations |

### Tutorials — do it yourself

| File | What you'll do |
|------|-----------------|
| [tutorials/00-verify-a-theorem-yourself.md](tutorials/00-verify-a-theorem-yourself.md) | Install nothing but Lean's toolchain, run one command, and watch the computer itself confirm a real theorem — no math required to follow the steps |

## What's *not* in scope here

This doc set covers the five "biggest" results — the ones each paper is actually built
around. It is not a catalog of all 202 statement labels in the corpus (see
`tools/paper-factory/lean-lake/mathlib_migration_receipts.jsonl` for that full list) — most
of those are smaller supporting lemmas, type definitions, or engineering-side properties
that don't need a plain-English writeup of their own to be useful.
