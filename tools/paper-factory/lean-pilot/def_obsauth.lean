/-
def:obsauth

Let `Obs` be the raw observation space; an observation `o ∈ Obs` is
authoritative (`o ∈ Obs*`) if it was produced by an admitted actuation
with a chained receipt; all other observations are untrusted; the
proposer's admission map `adm_prop` retracts `Obs` onto `Adm_prop` by
treating every inbound observation as untrusted until its receipt chain
satisfies the obligation battery `G_prop`.

We model this abstractly: given a raw observation space `Obs`, a type of
admitted actuations `Adm`, a type of receipts `Receipt`, and a predicate
`chained : Receipt → Prop` picking out receipts that are properly chained
(genesis-folded), an observation is authoritative iff it comes tagged
with an admitted actuation and a chained receipt for it. The obligation
battery is a predicate `G` on receipts; the admission map retracts `Obs`
onto the authoritative subtype by sending every observation either to
its witness (if its receipt satisfies `G` and is chained) or to `none`
(untrusted, no silent promotion).
-/

section ObsAuth

variable {Obs Adm Receipt : Type}
variable (chained : Receipt → Prop)
variable (G : Receipt → Prop)          -- obligation battery G_prop
variable (producedBy : Obs → Adm → Receipt → Prop)
                                        -- o was produced by admitted a via receipt r

/-- An observation `o` is authoritative if some admitted actuation `a` and
receipt `r` produced it, `r` is chained, and `r` satisfies the obligation
battery `G`. This is `Obs*`. -/
def Authoritative (o : Obs) : Prop :=
  ∃ (a : Adm) (r : Receipt), producedBy o a r ∧ chained r ∧ G r

/-- The authoritative subtype `Obs*`, a subtype of `Obs`. -/
def ObsStar := {o : Obs // Authoritative (chained := chained) (G := G) (producedBy := producedBy) o}

/-- The proposer's admission map: total on `Obs`, retracting every
observation to `some` authoritative witness when its receipt chain
satisfies `G`, or `none` when untrusted — never a silent default. -/
def adm_prop (o : Obs)
    (dec : ∀ o, Decidable (Authoritative (chained := chained) (G := G) (producedBy := producedBy) o)) :
    Option (ObsStar chained G producedBy) :=
  if h : Authoritative (chained := chained) (G := G) (producedBy := producedBy) o then
    some ⟨o, h⟩
  else
    none

end ObsAuth
