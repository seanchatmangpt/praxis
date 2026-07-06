import Praxis.Corpus.def_tax
import Praxis.Corpus.def_denialcode

/-!
# def:decoder

Let `L = {Adml, c_1, ..., c_7} ⊆ Deny` be the clean word together with the seven named
single-lane constants; the lane decoder `scnop_ℓ : Deny ⇀ Scnset` is
`scenario_for_denial_lane`: it returns `None` on `Adml`, returns the matching denial-lane
scenario on each `c_i`, and returns `None` on every word not in `L`.

`Deny` is `DenialPolarity` (`def:denialcode`), `Scnset` is `RefusalScenario` (`def:tax`).
Since `DenialPolarity` already derives `DecidableEq`, the partial function is just a
finite case split via `if`/`then`/`else` on that decidable equality -- no new Mathlib
machinery is needed beyond what `def:denialcode` and `def:tax` already bring in.
-/

open DenialPolarity RefusalScenario

/-- The lane decoder `scnop_ℓ : Deny ⇀ Scnset`, modeled as `Deny → Option Scnset`:
`None` on the clean word `Adml`, the matching denial-lane scenario on each of the seven
named single-lane constants `c_1, ..., c_7`, and `None` on every other word. -/
def scenario_for_denial_lane (d : DenialPolarity) : Option RefusalScenario :=
  if d = DenialPolarity.Adml then none
  else if d = DenialPolarity.lane1 then some RefusalScenario.lane1Denial
  else if d = DenialPolarity.lane2 then some RefusalScenario.lane2Denial
  else if d = DenialPolarity.lane3 then some RefusalScenario.lane3Denial
  else if d = DenialPolarity.lane4 then some RefusalScenario.lane4Denial
  else if d = DenialPolarity.lane5 then some RefusalScenario.lane5Denial
  else if d = DenialPolarity.lane6 then some RefusalScenario.lane6Denial
  else if d = DenialPolarity.lane7 then some RefusalScenario.lane7Denial
  else none
