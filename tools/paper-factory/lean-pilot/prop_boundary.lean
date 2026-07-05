/-
prop:boundary

Under def:obsauth, praxis-proposer enforces im(mu_op_prop) ⊆ Adm_prop: no
proposal is manufactured from an unadmitted observation; the generic
Domain<D> proposer evaluates its Obligation battery on every inbound
record before calling Admit::admit, and a record failing any obligation
is refused and never reaches manufacturing.

We prove the abstract counterpart: whenever `adm_prop` returns `some x`
for an observation `o`, `o` was in fact `Authoritative` (i.e. lies in
`Adm_prop`), and the witness `x` carries exactly that observation. So
the image of `adm_prop` restricted to the `some` branch lands entirely
inside the authoritative subtype — nothing not admitted is manufactured.
-/

section ObsAuth

variable {Obs Adm Receipt : Type}
variable (chained : Receipt → Prop)
variable (G : Receipt → Prop)
variable (producedBy : Obs → Adm → Receipt → Prop)

def Authoritative (o : Obs) : Prop :=
  ∃ (a : Adm) (r : Receipt), producedBy o a r ∧ chained r ∧ G r

def ObsStar := {o : Obs // Authoritative (chained := chained) (G := G) (producedBy := producedBy) o}

def adm_prop (o : Obs)
    (dec : ∀ o, Decidable (Authoritative (chained := chained) (G := G) (producedBy := producedBy) o)) :
    Option (ObsStar chained G producedBy) :=
  if h : Authoritative (chained := chained) (G := G) (producedBy := producedBy) o then
    some ⟨o, h⟩
  else
    none

/-- prop:boundary — the boundary theorem: every observation that the
proposer's admission map actually manufactures a proposal from (i.e.
every `o` with `adm_prop o dec = some x`) is authoritative, and the
manufactured witness `x` is exactly `o` tagged with its authoritativeness
proof. Nothing unadmitted is ever manufactured. -/
theorem boundary (o : Obs)
    (dec : ∀ o, Decidable (Authoritative (chained := chained) (G := G) (producedBy := producedBy) o))
    (x : ObsStar chained G producedBy)
    (hx : adm_prop chained G producedBy o dec = some x) :
    Authoritative (chained := chained) (G := G) (producedBy := producedBy) o ∧ x.val = o := by
  unfold adm_prop at hx
  split at hx
  · rename_i h
    injection hx with hx
    subst hx
    exact ⟨h, rfl⟩
  · exact absurd hx (by simp)

end ObsAuth
