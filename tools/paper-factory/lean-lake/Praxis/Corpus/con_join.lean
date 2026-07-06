import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic
import Mathlib.Data.Finset.Pi
/-!
# con:join — Grounding with type constraints as relational join

Grounding with type constraints is a relational join: for a schema parameter `v` of
type `τ`, the admissible objects are `{o : TypeIndex::satisfies(SymId(o), SymId(τ))}`,
walking the parent chain to depth bounded by the height of `T`, reducing effective
branching from `N^k` to `∏ᵢ |{o : type(o) ≤ τᵢ}|`.

We model `TypeIndex::satisfies` abstractly as a decidable relation `Satisfies : Obj →
Ty → Prop` on a finite object carrier `Obj` and finite type carrier `Ty` (no concrete
parent-chain-walking algorithm is fixed by the source text, only its extensional
effect — restricting candidates to those satisfying the type). Given that relation,
the *admissible objects for one parameter* are exactly Mathlib's `Finset.filter` of
the satisfies-predicate over `Finset.univ` (reusing `Fintype`/`Finset` rather than a
hand-rolled subset type), and the *admissible objects for a schema's whole parameter
list* (the "join" proper, reducing `N^k` to `∏ᵢ |{o : type(o) ≤ τᵢ}|`) is exactly
Mathlib's `Finset.pi`/dependent product over the per-parameter filtered sets, whose
cardinality is definitionally that product by `Finset.card_pi`.
-/

namespace Praxis.Corpus.ConJoin

/-- Abstract type-index satisfaction relation: `Satisfies o τ` holds when object `o`'s
symbol-indexed type is a subtype of (walks up the parent chain to) `τ`'s symbol-indexed
type. The parent-chain-walking algorithm itself is left abstract; only this
extensional relation is needed for the join construction. -/
class TypeIndex (Obj Ty : Type) where
  Satisfies : Obj → Ty → Prop
  [dec : DecidablePred (fun p : Obj × Ty => Satisfies p.1 p.2)]

variable {Obj Ty : Type} [Fintype Obj] [DecidableEq Obj] [DecidableEq Ty] [TypeIndex Obj Ty]

/-- The admissible objects for a single typed parameter `τ`: exactly the objects
satisfying `TypeIndex.Satisfies · τ`, computed as a `Finset.filter` over the finite
object carrier (Mathlib's `Fintype`/`Finset.filter`, not a hand-rolled subset). -/
noncomputable def admissible (τ : Ty) : Finset Obj :=
  haveI : DecidablePred (fun o : Obj => TypeIndex.Satisfies o τ) := Classical.decPred _
  Finset.univ.filter (fun o => TypeIndex.Satisfies o τ)

/-- The relational join for a schema's whole typed parameter list `τs : List Ty`:
the dependent product, over each parameter's `admissible` set, of a choice of object —
i.e. `Finset.pi` on the list of admissible sets. This is the grounding-candidate space
whose size is `∏ᵢ |admissible τᵢ|` rather than the naive `N^k` (all objects raised to
the parameter count). -/
noncomputable def groundingJoin (τs : List Ty) :
    Finset (∀ i ∈ τs.toFinset, Obj) :=
  τs.toFinset.pi (fun τ => admissible (Obj := Obj) τ)

end Praxis.Corpus.ConJoin
