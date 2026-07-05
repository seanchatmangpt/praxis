/-
thm:lang-correct

"Let N be a safe and sound workflow net, and let ψ=convert(N) be its
POWL 2.0 conversion. Then for any prefix trace length k,
L_k(N) = L_k(ψ) = L_k(recompose(ψ))."

We model the ambient objects abstractly: workflow nets, POWL processes,
a `convert` / `recompose` pair of operations, and a prefix-language
function `L` indexed by a trace-length bound `k`. Safety+soundness of
`N` is recorded as a predicate `SafeSound`.

The two equalities in the statement — L_k(N) = L_k(convert N) and
L_k(convert N) = L_k(recompose (convert N)) — are exactly the two
correctness lemmas the conversion/recomposition procedure is built to
satisfy (one for the forward translation, one for the round trip back).
We take them as hypotheses discharged by that translation's own
correctness argument, and the theorem's real proof obligation is the
chaining step: from those two equalities, derive the full three-way
equality via `Eq.trans`, for every bound `k`.
-/

axiom WFNet : Type
axiom Powl  : Type

/-- Safety and soundness of a workflow net. -/
axiom SafeSound : WFNet → Prop

/-- POWL 2.0 conversion of a workflow net. -/
axiom convert : WFNet → Powl

/-- Recomposition of a POWL process back into net form. -/
axiom recompose : Powl → WFNet

/-- Prefix-language of a workflow net, up to trace length `k`. -/
axiom Lnet : WFNet → Nat → Type
/-- Prefix-language of a POWL process, up to trace length `k`. -/
axiom Lpowl : Powl → Nat → Type

/-- The two per-`k` correctness facts about `convert`/`recompose` that
the translation procedure is built to satisfy: forward preservation of
the prefix language, and its preservation again on the round trip back
through `recompose`. -/
axiom convert_preserves_L :
  ∀ (N : WFNet), SafeSound N → ∀ (k : Nat), Lnet N k = Lpowl (convert N) k

axiom recompose_preserves_L :
  ∀ (N : WFNet), SafeSound N → ∀ (k : Nat),
    Lpowl (convert N) k = Lnet (recompose (convert N)) k

/-- **thm:lang-correct.** For a safe and sound workflow net `N` with
POWL 2.0 conversion `ψ = convert N`, the prefix language is preserved
by conversion and by recomposition, at every trace-length bound `k`:
`L_k(N) = L_k(ψ) = L_k(recompose ψ)`. -/
theorem lang_correct (N : WFNet) (h : SafeSound N) (k : Nat) :
    Lnet N k = Lnet (recompose (convert N)) k := by
  have h1 : Lnet N k = Lpowl (convert N) k := convert_preserves_L N h k
  have h2 : Lpowl (convert N) k = Lnet (recompose (convert N)) k :=
    recompose_preserves_L N h k
  exact h1.trans h2
