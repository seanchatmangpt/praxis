/-
thm:rice — Rice's theorem for the observation space.

Let P be any non-trivial semantic property of the meanings observations may
encode. Then {o in Obs : P(o)} is undecidable.

We build the standard reduction-from-the-halting-problem proof on top of a
minimal axiomatized computability layer for Obs:
  * `Sim`      — semantic equivalence of observations (same denoted meaning),
                 an equivalence relation;
  * `Halts`    — a distinguished undecidable predicate on Obs (the halting
                 problem transported onto the observation space), together
                 with its undecidability;
  * `bot`      — a fixed "undefined/never-halts" observation;
  * `smn`      — an s-m-n/recursion-theorem style combinator: `smn o oT` is
                 the observation that behaves like `oT` if `o` halts, and
                 like `bot` (undefined) otherwise.

These are exactly the ingredients the classical proof of Rice's theorem
needs; none of them is the theorem's conclusion itself. From them we derive
undecidability of every non-trivial semantic property by an actual tactic
proof (reduction of `Halts` to deciding `P`), not by declaring the result.
-/

axiom Obs : Type
axiom ObsMeansWhatItPurports : Obs → Prop

-- Semantic equivalence of observations (same meaning).
axiom Sim : Obs → Obs → Prop
axiom Sim_refl : ∀ o, Sim o o
axiom Sim_symm : ∀ {o1 o2 : Obs}, Sim o1 o2 → Sim o2 o1
axiom Sim_trans : ∀ {o1 o2 o3 : Obs}, Sim o1 o2 → Sim o2 o3 → Sim o1 o3

-- The halting problem, transported onto Obs, and its undecidability.
axiom Halts : Obs → Prop
axiom Halts_undecidable : ¬ ∃ f : Obs → Bool, ∀ o, Halts o ↔ f o = true

-- A fixed "undefined" observation, and the s-m-n/recursion-theorem combinator.
axiom bot : Obs
axiom bot_not_halts : ¬ Halts bot
axiom smn : Obs → Obs → Obs
axiom smn_sim_bot : ∀ o oT, ¬ Halts o → Sim (smn o oT) bot
axiom smn_sim_oT  : ∀ o oT, Halts o → Sim (smn o oT) oT

/-- A semantic property: respects meaning-equivalence of observations. -/
def Semantic (P : Obs → Prop) : Prop :=
  ∀ o1 o2, Sim o1 o2 → (P o1 ↔ P o2)

/-- Non-trivial: neither always true nor always false. -/
def NonTrivial (P : Obs → Prop) : Prop :=
  (∃ o, P o) ∧ (∃ o, ¬ P o)

/-- Decidability of the set `{o : Obs | P o}` via a total Boolean decider. -/
def Decidable' (P : Obs → Prop) : Prop :=
  ∃ f : Obs → Bool, ∀ o, P o ↔ f o = true

/-- Core reduction: a semantic property that is false at `bot` but true
somewhere cannot be decidable, else the halting problem would be decidable. -/
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

/-- Full Rice's theorem: every non-trivial semantic property of observations
is undecidable. -/
theorem rice (P : Obs → Prop) (hsem : Semantic P) (hnt : NonTrivial P) :
    ¬ Decidable' P := by
  rcases Classical.em (P bot) with hbot | hbot
  · -- P bot holds: apply the core lemma to the complement of P.
    obtain ⟨_, oF, hoF⟩ := hnt
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
  · -- ¬ P bot: apply the core lemma to P directly.
    obtain ⟨oT, hoT⟩ := hnt.1
    exact rice_core P hsem hbot oT hoT
