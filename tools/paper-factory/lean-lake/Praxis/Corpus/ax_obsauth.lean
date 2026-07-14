import Praxis.Corpus.ax_refusal
import Praxis.Mathlib.ObsSimEquivalence

/-!
`def:obsauth`'s domain-specific external-provenance axioms, split out of `def_obsauth.lean`.

* `Obs` -- the raw observation space. Left abstract for the same reason as `Adm` in
  `ax_refusal`: the thesis never fixes what an observation *is*, only its role in the
  authoritative/untrusted distinction.
* `HasChainedReceipt` -- the predicate "was produced by an admitted actuation with a
  chained receipt". This stands in for real receipt-chaining behavior implemented outside
  Lean (the receipt/chain machinery in `crates/praxis-graphlaw`): the statement does not
  reduce it to any existing concrete structure, so it is left as an abstract predicate on
  `Obs` rather than invented as concrete data.
* `GProp` -- the obligation battery `G_prop` the receipt chain must satisfy. Like
  `HasChainedReceipt`, its content is a domain-specific set of proposer obligations
  enforced by the external system, not something Mathlib provides a stand-in for.
* `admProp` -- the proposer's admission map, retracting `Obs` onto `Adm_prop`. Existence
  of this classification map is domain-specific (it depends on how untrusted observations
  are actually refused by the external system), so it is left as an axiomatized total
  function; totality itself is witnessed literally by the imported `AdmissionResult`'s
  `Sum` type.

All four are genuine external-system axioms (they model behavior of code outside this
Lean corpus, not placeholders for something provable in-Lean), which is why they live
here rather than being asserted inline in `def_obsauth.lean`.
-/

/-- The provenance predicate: `o` was produced by an admitted actuation
with a chained receipt. Stands for the real receipt-chaining behavior implemented
outside Lean (praxis-graphlaw's receipt/chain machinery); left abstract since the
statement does not reduce it to any existing concrete encoding. -/
axiom HasChainedReceipt : Obs → Prop

/-- The obligation battery `G_prop` a receipt chain must satisfy for the
proposer. Stands for a domain-specific, externally-enforced set of proposer
obligations; left abstract for the same reason as `HasChainedReceipt`. -/
axiom GProp : Obs → Prop

/-- The proposer's admission map `adm_prop`, retracting `Obs` onto
`Adm_prop` by treating every inbound observation as untrusted until its
receipt chain satisfies `G_prop`. Existence of this classification map is
domain-specific (it depends on how untrusted observations are actually
refused by the external system), so it is left as an axiomatized total function;
totality itself is witnessed literally by the imported `AdmissionResult`'s
`Sum` type, exactly as in `ax_refusal`. -/
axiom admProp : Obs → AdmissionResult
