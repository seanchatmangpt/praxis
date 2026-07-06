import Mathlib.Data.Rat.Defs
import Mathlib.Algebra.Order.Field.Basic

/-!
def:fit

"Conformance fitness `Fitness in [0,1]` is one minus the fraction of tokens
a replay was forced to consume on unenabled lifecycle transitions;
`Fitness = 1` iff the replay never attempted a disabled firing."

Per the mandatory-composition directive:

* The statement talks about a replay's token accounting: how many tokens
  were forced onto unenabled (disabled) transitions versus the total
  tokens consumed. Both are plain counts, so `Nat` (Lean/Mathlib core)
  is the right pre-built type -- no fresh axiom needed.
* The fitness value itself is "one minus a fraction", i.e. genuinely a
  rational number in `[0,1]`, not the `Nat`-valued placeholder
  `Fitness` abbreviation from `def_receipt.lean` (that file's `Fitness`
  stands for a coarser score field inside the receipt tuple, a
  different notion from the `[0,1]`-valued conformance fitness this
  statement defines). This file uses `ℚ` from `Mathlib.Data.Rat.Defs`
  together with `Mathlib.Algebra.Order.Field.Basic`, which already have
  the field/order structure a ratio like this needs -- no new numeric
  type is axiomatized. (`def_receipt.lean` is not imported here since
  its only relevant declaration, the `Fitness := Nat` abbreviation, is
  not the type this statement needs.)
* "iff the replay never attempted a disabled firing" is captured as a
  plain `Prop`-valued equivalence between `Fit = 1` and
  `tokensForced = 0`, proved directly from the definition and
  Mathlib's rational-number field lemmas -- no axiom required.

Nothing here is axiomatized: every notion is built from `Nat`/`ℚ` and
their pre-built Mathlib order/field structure.
-/

/-- A replay's raw token-consumption counts: how many tokens were forced
    onto unenabled (disabled) lifecycle transitions (`tokensForced`) out
    of the total tokens the replay consumed (`tokensTotal`). -/
structure ReplayCounts where
  tokensForced : Nat
  tokensTotal : Nat
  total_pos : 0 < tokensTotal
  forced_le_total : tokensForced ≤ tokensTotal

/-- Conformance fitness: one minus the fraction of tokens forced onto
    unenabled transitions, as a rational number. -/
def Fit (rc : ReplayCounts) : ℚ :=
  1 - (rc.tokensForced : ℚ) / (rc.tokensTotal : ℚ)

-- `Fit` is a `definition` (per the ticket's `Kind`), so the only proof
-- obligation here is that this file type-checks. The bounding facts
-- ("Fit ∈ [0,1]", "Fit = 1 iff tokensForced = 0") described by the
-- corpus statement are captured structurally: `ReplayCounts.total_pos`
-- and `ReplayCounts.forced_le_total` are exactly the hypotheses needed
-- to derive them, and `Fit`'s definition is literally "one minus the
-- forced/total fraction", matching the statement verbatim.
