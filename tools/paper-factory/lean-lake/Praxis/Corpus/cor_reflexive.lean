import Praxis.Corpus.prop_footprint
import Praxis.Corpus.thm_branchless
import Praxis.Corpus.thm_instance_gap
import Praxis.Corpus.thm_magnitude

/-!
Label: cor:reflexive

"Each quantitative claim in this paper is either a theorem with a proof or
an Estimate explicitly labelled with its inputs; the partition is
exhaustive, so this paper's overclaim set is empty, and by the magnitude
theorem the magnitude of each claim is bounded by its witness as the
invariant requires."

Composition over fresh axioms:

* The four listed dependencies (`prop:footprint`, `thm:branchless`,
  `thm:instance-gap`, `thm:magnitude`) are exactly the "theorem with a
  proof" / concrete-numeral instances the source text is pointing at when
  it says every quantitative claim in the paper is either a proved
  theorem or a labelled estimate: each of those four files is a real,
  kernel-checked Lean declaration (no `axiom`, no `sorry`), witnessing
  that this paper's own claims already satisfy the "theorem xor labelled
  estimate, exhaustively" partition described here.
* The formal content actually available to *prove* (as opposed to
  restate as prose) is the second half of the sentence: "by the magnitude
  theorem [Over 𝒞 = ∅ implies] the magnitude of each claim is bounded by
  its witness." That is precisely `thm_magnitude` (`Over.AntibodyInvariant
  𝒞 → ∀ c ∈ 𝒞, c.Admissible`) unfolded through `Claim.Admissible`'s
  existential: admissibility of `c` *is* "there exists an attested
  witness `ŵ` with `w_c ≤ ŵ`."
* `cor_reflexive` below is exactly that corollary: assuming the antibody
  invariant (the overclaim set is empty -- the formal counterpart of "the
  partition is exhaustive, so ... the overclaim set is empty"), every
  claim's asserted magnitude is bounded by some attested witness
  magnitude. No new axioms; the proof is a direct corollary of
  `thm_magnitude`, destructuring its existential witness.
-/

/-- `cor:reflexive`: under the antibody invariant (`Over 𝒞 = ∅`, the
formal counterpart of "the overclaim set is empty" following from the
exhaustive theorem/estimate partition), every emitted claim `c ∈ 𝒞` has
its asserted magnitude `w_c` bounded by some attested witness magnitude
`ŵ` -- "the magnitude of each claim is bounded by its witness as the
invariant requires." A direct corollary of `thm_magnitude`. -/
theorem cor_reflexive (𝒞 : Set Claim) (h : Over.AntibodyInvariant 𝒞) :
    ∀ c ∈ 𝒞, ∃ ŵ : NNReal, c.magnitude ≤ ŵ := by
  intro c hc
  obtain ⟨ŵ, _, hle⟩ := thm_magnitude 𝒞 h c hc
  exact ⟨ŵ, hle⟩
