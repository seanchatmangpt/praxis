import Mathlib.Data.Fin.Basic
import Mathlib.Order.Fin.Basic
import Mathlib.Order.WithBot
import Mathlib.Data.Prod.Lex

/-!
# def:cost — Router cost vector

The router's `CostVector` is the tuple
`(c0,...,c5) = (overline{admitted}, risk, attention_seconds, tokens, latency, switches)`;
the order is lexicographic, cheapest-first, and the refused vector
`refused() = (unadmitted, 255, ∞, ∞, ∞, 255)` is the top element.

Rather than hand-rolling a 6-way lexicographic order and re-proving it is a
`LinearOrder`, we compose it from pieces Mathlib already provides:

* the lexicographic order combinator `α ×ₗ β` together with its
  `LinearOrder` instance (`Mathlib.Data.Prod.Lex`), nested five times to
  get a 6-tuple lexicographic order for free (no lemmas re-proved here);
* `c0 = overline{admitted}` (the *negated* admission flag) as `Bool`, whose
  built-in order `false < true` makes "not admitted" strictly more expensive
  than "admitted", matching the overline in the source;
* `risk` and `switches`, both bytes in the source text, as `Fin 256`, whose
  `LinearOrder` is already derived by Mathlib;
* `attention_seconds`, `tokens`, `latency` as `WithTop ℕ`, so that each can
  carry the literal `∞` used in `refused()`, again with a `LinearOrder`
  instance already provided by Mathlib (`Mathlib.Order.WithBot`).
-/

namespace Praxis.Corpus.DefCost

/-- The router's cost vector `(c0,...,c5)`, ordered lexicographically and
cheapest-first via nested `Lex` products (each factor's own `LinearOrder`
supplied by Mathlib; the nesting is the only new "definition" here). -/
def CostVector : Type :=
  Bool ×ₗ (Fin 256 ×ₗ (WithTop ℕ ×ₗ (WithTop ℕ ×ₗ (WithTop ℕ ×ₗ Fin 256))))

noncomputable instance : LinearOrder CostVector := by
  unfold CostVector
  infer_instance

/-- The refused vector `refused() = (unadmitted, 255, ∞, ∞, ∞, 255)`, the top
element of `CostVector` under the lexicographic order above. -/
noncomputable def refused : CostVector :=
  (toLex
    (true,
      toLex
        ((255 : Fin 256),
          toLex
            ((⊤ : WithTop ℕ),
              toLex
                ((⊤ : WithTop ℕ),
                  toLex ((⊤ : WithTop ℕ), (255 : Fin 256)))))) : CostVector)

end Praxis.Corpus.DefCost