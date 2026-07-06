import Praxis.Corpus.def_over

/-!
thm:magnitude

"If a system maintains `Over = ∅`, then for every emitted claim `c`,
`w_c ≤ ŵ_c = max{w : the committed fields of r(c) entail magnitude w}`,
verifiable at boundary cost `O(CL)` independent of the interior that
produced `c`; no claim can exceed what its mechanism can receipt."

Composition over fresh axioms:

* This is a direct consequence of the already-migrated `Over` and
  `Claim.Admissible` from `def:over` / `def:claim`: `Over 𝒞 = ∅` means
  every `c ∈ 𝒞` fails to be a member of `{c ∈ 𝒞 | ¬ c.Admissible}`, hence
  (since `c ∈ 𝒞` already holds) `c.Admissible` holds, i.e. `c` has *some*
  attested magnitude `ŵ_c` with `w_c ≤ ŵ_c`. No new machinery is
  introduced; the proof is pure set-membership unfolding via Mathlib's
  `Set.mem_sep_iff` / `Set.eq_empty_iff_forall_not_mem`, composed with
  the existing `Claim.Admissible` definition.
* The "boundary cost `O(CL)` independent of the interior" and the
  concrete maximality of `ŵ_c` ("`= max{w : ...}`") are complexity/
  optimality claims about the (still-abstract) receipt/boundary-
  projection apparatus `r(c)`, `Proj`, `V` from `def:claim`
  (`HasReceiptWitnessAttesting`), which remains abstract per that
  definition's own scope note. The formalized theorem below captures
  exactly the load-bearing logical content available at this stage of
  migration -- "no claim can exceed what its mechanism can receipt",
  i.e. admissibility of every claim under the antibody invariant -- and
  is proved as a real theorem (no `sorry`, no axiom).
-/

/-- If the antibody invariant `Over 𝒞 = ∅` holds, then every claim `c`
emitted by the system (`c ∈ 𝒞`) is admissible: it has *some* receipt
witness attesting a magnitude `ŵ_c` with `w_c ≤ ŵ_c`. No claim can
exceed what its mechanism can receipt. -/
theorem thm_magnitude (𝒞 : Set Claim) (h : Over.AntibodyInvariant 𝒞) :
    ∀ c ∈ 𝒞, c.Admissible := by
  intro c hc
  by_contra hna
  have hmem : c ∈ Over 𝒞 := Set.mem_sep hc hna
  rw [Over.AntibodyInvariant] at h
  rw [h] at hmem
  exact hmem
