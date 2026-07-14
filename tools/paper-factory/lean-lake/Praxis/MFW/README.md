# Praxis/MFW

Multifractal Workflow (MFW) configuration object `X = (G, W, g, H, R)` and its
transition `Φ`, formalized as definitions only — no theorems, no `sorry` — grounded
in existing `Praxis/Corpus` types wherever a genuine fit exists.

## Files

- `State.lean` — the `MFWConfig` structure.
- `Operators.lean` — the eight operators `C, ρ, Π, M, ⊙, E, α, K`.
- `Transition.lean` — the `MFWResult` outcome type and the transition `Φ`.

## (a) Field grounding: `G, W, g, H, R`

| Field | Reused corpus type / new | Source file |
|---|---|---|
| `G` — admitted graph state | Reused: `Praxis.Corpus.DefGround.GroundedState D Ob` | `Praxis/Corpus/def_ground.lean` |
| `W` — recursive workflow geometry | Reused: `Praxis.Corpus.POWL (GroundAction D Ob)` | `Praxis/Corpus/def_powl.lean` |
| `g` — required continuation goal | **New**: `MFWGoal D Ob := Finset (GroundAtom D Ob)` | `Praxis/MFW/State.lean` (built from the reused `GroundAtom`) |
| `H` — admitted capability/hook surface | Reused: `Praxis.Corpus.GateBattery (GroundAction D Ob)` | `Praxis/Corpus/def_astgate.lean` |
| `R` — current receipt chain | Reused: `List Praxis.Receipt` | `Praxis/Core.lean` |

`g` is the one newly-invented piece: no corpus type carries a PDDL-style goal
(`def_ground`'s `Problem` bundles an object universe, an initial grounded state, and a
discount factor, but has no goal field, and no other corpus file defines one). It is
built entirely from the already-reused `GroundAtom D Ob` type rather than an invented
bespoke goal syntax.

`R` uses `Praxis.Receipt` (`Praxis/Core.lean`) rather than the more literal
`Praxis.Corpus.def_chain.Ledger`: `def_chain.lean` transitively imports
`Praxis.Corpus.def_frame`'s global `structure Frame`, which collides with
`Praxis.Mathlib.DefReceipt`'s own global `structure Frame` the moment both land in one
root import graph — and `Praxis.lean` already imports `Praxis.Mathlib.DefReceipt`.
This is a pre-existing naming collision between two `Praxis/Corpus`/`Praxis/Mathlib`
files that this purely-additive pass must not touch (fixing it would mean editing
files under `Praxis/Corpus/` or `Praxis/Mathlib/`, which is out of scope here);
`Praxis.Receipt` is the genuine, already-top-level-wired stand-in that avoids
re-triggering it.

## (b) Operator grounding: `C, ρ, Π, M, ⊙, E, α, K`

| Operator | Kind | Justification |
|---|---|---|
| `C` (closure) | real `def` | `Saturation T ∪ G.trueAtoms`, reusing `Praxis.Corpus.DefSaturation.Saturation` (`= OrderHom.lfp T`, Mathlib's pre-built least fixpoint) unioned with `G`'s own facts. |
| `ρ` (residue) | real `def` | Plain `Set` difference `goal \ closure`. |
| `Π` (planning, exported as `piPlan`) | `axiom` | Stands for a real external PDDL/STRIPS planner (e.g. Fast Downward, or this repo's own `cng` PDDL-TTL pipeline) searching a combinatorial action space — no Mathlib/Lean-core term models heuristic search. |
| `M` (manufacture) | `axiom` | Stands for the real external POWL manufacture process consulting scheduling/cost/hook-realizability metadata this definitional pass's bare `MFWPlan` (`List (GroundAction D Ob)`) does not carry; a trivial structural wrapper would misrepresent a real manufacturing process as free recursion. |
| `⊙` (`attach`) | real `def` | Structural append to a `POWL` node's `children : List (POWL A)` field (`def_powl.lean`'s own list-position addressing scheme); an `activity` leaf is first promoted to a two-child `partialOrder`. |
| `E` (execution) | `axiom` | Stands for real broker-controlled, side-effecting execution (actual network calls / process spawns / hardware interaction) — genuinely outside Lean's computational reach. |
| `α` (admission) | `axiom` | Stands for the real external admission/parsing pipeline turning an actually-returned `Praxis.Observation` into vetted ground atoms; depends on real signature/receipt/authority verification not carried by this definitional pass. |
| `K` (capitalization) | real `def` | `Finset` union on `G.trueAtoms` — structurally exactly what a SPARQL `CONSTRUCT` does (insert constructed triples into the target graph); no SPARQL engine exists in this corpus to invoke, but the structural content is fully computable. |

`Entails` (`C(G) ⊨ g`) and `Admissible` (`C(G), g` admissible in principle) are two
further real `def`s in `Operators.lean` used by `Φ`: `Entails` is `goal ⊆ closure`;
`Admissible` is `goal ⊆ Saturation T` (the domain's full reachable closure,
independent of `G`'s current contents) — deliberately weaker than `Entails`, so that a
goal can be `Admissible` (reachable in principle) without yet being entailed by the
current closure (the `Continue` case), while a goal that fails `Admissible` can never
become true no matter how much further planning/execution runs (the `Refused` case).

`MFWResult` (`Closed | Refused | Continue`) is modeled after this corpus's existing
tri-/n-state result pattern: `Praxis.Corpus.DefQueryResult.QueryResult`'s
`Answered`/`Denied`/`Invalid` and `Disposition`'s four-way
`Completed`/`Parked`/`SkippedBy`/`GaveUp` (`prop_totalaccounting.lean`).

`Φ` itself is a genuine `def` (`noncomputable`, since `Entails`/`Admissible` are
classically-decided `Set`-level `Prop`s): pure case analysis and composition of the
eight operators above, even though some of the values it composes (`piPlan`, `M`, `E`,
`α`) are axiomatized. Case analysis over an axiomatized value's *type* is still
constructive dispatch, not itself an assumption.

## Axiom-allowlist gate

`AXIOM_ALLOWLIST.md`'s regression gate (`just praxis-lean-axiom-gate`) is explicitly
scoped to `tools/paper-factory/lean-lake/Praxis/Corpus/*.lean` (per its own
"Recomputed from scratch" grep command and disclosure header). `Praxis/MFW/*.lean`
files live outside `Praxis/Corpus/` entirely, so the four axioms introduced here
(`Π`/`piPlan`, `M`, `E`, `α`) are out of that gate's scope by construction — no
`Praxis/Corpus/` file was touched, and no entry was added to `AXIOM_ALLOWLIST.md`. If
this gate's scope is ever widened to cover `Praxis/MFW/`, the four axioms above (with
the one-sentence justifications in the table) are the complete, honest inventory to
add.

## Scope

Definitions only — no theorems attempted this pass; the Autonomous Resolution Crown
Theorem (termination + no-unreceipted-actuation) is future work once this object has
been reviewed.
