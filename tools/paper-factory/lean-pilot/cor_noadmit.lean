/-
cor:noadmit — No algorithm admits an observation by deciding a non-trivial
semantic property of what it denotes; any total admission procedure decides a
syntactic, decidable surrogate instead.

This is a direct corollary of `thm:rice` (see thm_rice.lean, reused verbatim
below). An "admission procedure" is any total Boolean decider `f : Obs → Bool`.
If it decided a semantic, non-trivial property (the property `fun o => f o =
true` is itself `Semantic` and `NonTrivial`), Rice's theorem would be
contradicted, since `f` witnesses `Decidable'` of exactly that property. Hence
no total decider can be deciding a semantic non-trivial property: whatever
property a total `f` decides must fail `Semantic` or `NonTrivial` — i.e. it is
a syntactic, decidable surrogate, not the semantic property itself.
-/

axiom Obs : Type
axiom ObsMeansWhatItPurports : Obs → Prop

axiom Sim : Obs → Obs → Prop
axiom Sim_refl : ∀ o, Sim o o
axiom Sim_symm : ∀ {o1 o2 : Obs}, Sim o1 o2 → Sim o2 o1
axiom Sim_trans : ∀ {o1 o2 o3 : Obs}, Sim o1 o2 → Sim o2 o3 → Sim o1 o3

axiom Halts : Obs → Prop
axiom Halts_undecidable : ¬ ∃ f : Obs → Bool, ∀ o, Halts o ↔ f o = true

axiom bot : Obs
axiom bot_not_halts : ¬ Halts bot
axiom smn : Obs → Obs → Obs
axiom smn_sim_bot : ∀ o oT, ¬ Halts o → Sim (smn o oT) bot
axiom smn_sim_oT  : ∀ o oT, Halts o → Sim (smn o oT) oT

def Semantic (P : Obs → Prop) : Prop :=
  ∀ o1 o2, Sim o1 o2 → (P o1 ↔ P o2)

def NonTrivial (P : Obs → Prop) : Prop :=
  (∃ o, P o) ∧ (∃ o, ¬ P o)

def Decidable' (P : Obs → Prop) : Prop :=
  ∃ f : Obs → Bool, ∀ o, P o ↔ f o = true

theorem rice_core (P : Obs → Prop) (hsem : Semantic P)
    (hbot : ¬ P bot) (oT : Obs) (hoT : P oT) : ¬ Decidable' P := by
  intro ⟨f, hf⟩
  apply Halts_undecidable
  refine ⟨fun x => f (smn x oT), fun x => ?_⟩
  constructor
  · intro hx
    have hsimT : Sim (smn x oT) oT := smn_sim_oT x oT hx
    have hiff : P (smn x oT) ↔ P oT := hsem _ _ hsimT
    exact (hf (smn x oT)).mp (hiff.mpr hoT)
  · intro hx
    apply Classical.byContradiction
    intro hxnh
    have hsimBot : Sim (smn x oT) bot := smn_sim_bot x oT hxnh
    have hiff : P (smn x oT) ↔ P bot := hsem _ _ hsimBot
    exact hbot (hiff.mp ((hf (smn x oT)).mpr hx))

theorem rice (P : Obs → Prop) (hsem : Semantic P) (hnt : NonTrivial P) :
    ¬ Decidable' P := by
  rcases Classical.em (P bot) with hbot | hbot
  · obtain ⟨_, oF, hoF⟩ := hnt
    have hsemNeg : Semantic (fun o => ¬ P o) := by
      intro o1 o2 hsim
      constructor
      · intro h1 h2
        exact h1 ((hsem o1 o2 hsim).mpr h2)
      · intro h1 h2
        exact h1 ((hsem o1 o2 hsim).mp h2)
    have hcore := rice_core (fun o => ¬ P o) hsemNeg (fun h => h hbot) oF hoF
    intro ⟨f, hf⟩
    apply hcore
    refine ⟨fun o => !f o, fun o => ?_⟩
    constructor
    · intro hnp
      cases hfo : f o with
      | false => simp [hfo]
      | true => exact absurd ((hf o).mpr hfo) hnp
    · intro hb hp
      have hft : f o = true := (hf o).mp hp
      simp [hft] at hb
  · obtain ⟨oT, hoT⟩ := hnt.1
    exact rice_core P hsem hbot oT hoT

/-- cor:noadmit. An admission procedure is any total Boolean decider on `Obs`.
No such procedure can be simultaneously deciding a property that is
`Semantic` (respects meaning-equivalence) and `NonTrivial` — i.e. it cannot be
"deciding a non-trivial semantic property of what it denotes". Concretely, for
any total `f : Obs → Bool`, the property it literally decides (`fun o => f o
= true`) cannot be both `Semantic` and `NonTrivial`; whatever `f` computes is
therefore forced to be a syntactic, decidable surrogate for that semantic
property, not the semantic property itself. -/
theorem no_admit (f : Obs → Bool) :
    ¬ (Semantic (fun o => f o = true) ∧ NonTrivial (fun o => f o = true)) := by
  rintro ⟨hsem, hnt⟩
  exact rice (fun o => f o = true) hsem hnt ⟨f, fun o => Iff.rfl⟩
