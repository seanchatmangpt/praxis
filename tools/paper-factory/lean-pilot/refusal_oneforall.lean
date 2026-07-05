/-
Label: refusal:oneforall
Kind: refusal

No OneForAll: OTP justifies one-for-all by shared mutable fate between
siblings. In an acyclic data-flow plan, same-depth siblings are
independent by construction -- the coupling one-for-all encodes is
inexpressible. The variant does not appear in the Strategy enum at all;
an exhaustive-match test makes its future addition a compilation event,
forcing the doctrine conversation.

We reuse the `Strategy` enum from def:earned (RestForOne / OneForOne).
The refusal is witnessed here by an exhaustive match over `Strategy`
that names both constructors and no others: if a `OneForAll` variant
were ever added to the enum, this match would stop type-checking
(non-exhaustive match), which is exactly the "compilation event" the
source text describes. The function below is total precisely because
`OneForAll` is not, and cannot be, one of the cases.
-/

/-- The supervision strategy enum, as in def:earned: only RestForOne and
OneForOne are members. No `OneForAll` constructor exists. -/
inductive Strategy where
  | RestForOne
  | OneForOne

/-- Exhaustive match witnessing that `Strategy` has exactly two
constructors. Adding a `OneForAll` case to `Strategy` would make this
definition fail to type-check (non-exhaustive match), turning the
enum's extension into a forced compilation event rather than a silent
addition. -/
def Strategy.isRestForOne (s : Strategy) : Bool :=
  match s with
  | Strategy.RestForOne => true
  | Strategy.OneForOne  => false

/-- Sanity check: the exhaustive match above type-checks and computes,
confirming `Strategy` currently admits only these two cases. -/
example : Strategy.isRestForOne Strategy.RestForOne = true := rfl
example : Strategy.isRestForOne Strategy.OneForOne = false := rfl
