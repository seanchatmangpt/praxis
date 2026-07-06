import Praxis.Corpus.def_adm
import Mathlib.Data.List.Lex

/-!
Label: def:costvector

"Assign a plan $\bm c=(c_0,c_1,\dots,c_k)$, $c_0\in\{0,1\}$ the unadmitted
indicator ($0=$ admitted), $c_1,\dots,c_k$ bounded secondary costs. Order by
lexicographic $\preceq_{\mathrm{lex}}$."

`c_0 \in \{0,1\}` (the unadmitted indicator, `0` = admitted) is realized as
`Bool` (`false` = admitted = `0`, `true` = unadmitted = `1`), matching the
`Option`/success-failure encoding already used for `adm` in `def:adm`
(admission succeeds to `some o`, i.e. the "0" branch). The secondary costs
`c_1,\dots,c_k` are "bounded" finite-length sequences of naturals, realized as
`List ℕ`. The pair `(c_0, c_1,\dots,c_k)` is the product `Bool × List ℕ`.

Lexicographic order `\preceq_{\mathrm{lex}}` is realized directly by
Mathlib's `Prod.Lex`/`List.Lex` machinery: `Bool`'s own `<` already orders
`false < true`, i.e. admitted before unadmitted, and `List.Lex (· < ·)`
supplies the lexicographic order on the secondary-cost tail. `CostVector.lt`
combines them exactly as `Prod.Lex` does (compare `c0` first; only compare
tails when `c0` fields are equal), so no bespoke order relation is
hand-built.

No axioms: this is a plain data-level definition composed from `Bool`,
`List ℕ`, and Mathlib's prebuilt `List.Lex` lexicographic order.
-/

/-- `def:costvector`: a cost vector `c = (c_0, c_1, …, c_k)`. `c0 = false`
means admitted (the "0" branch of `def:adm`'s `some o`), `c0 = true` means
unadmitted (the "`Rfsl`"/`none` branch); `cs` holds the bounded secondary
costs `c_1, …, c_k`. -/
structure CostVector where
  c0 : Bool
  cs : List Nat
  deriving DecidableEq, Repr

namespace CostVector

/-- Lexicographic order on `CostVector`: compare the unadmitted indicator
`c0` first (`false` i.e. admitted precedes `true` i.e. unadmitted, via
`Bool`'s standard order), and only compare the secondary-cost tails `cs`
(via Mathlib's `List.Lex` on `Nat`'s order) when the indicators agree. -/
instance : LT CostVector where
  lt a b := a.c0 < b.c0 ∨ (a.c0 = b.c0 ∧ List.Lex (· < ·) a.cs b.cs)

instance : DecidableRel (α := CostVector) (· < ·) := fun a b =>
  inferInstanceAs (Decidable (a.c0 < b.c0 ∨ (a.c0 = b.c0 ∧ List.Lex (· < ·) a.cs b.cs)))

instance : LE CostVector where
  le a b := a = b ∨ a < b

end CostVector