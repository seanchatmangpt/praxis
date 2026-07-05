/-
Label: def:fragile
Kind: definition

A precondition predicate `p` of capability `c` is fragile iff no capability
in the problem produces `p`. The initial state is a one-time gift: if a
fragile fact is lost mid-run, nothing in the plan can lawfully re-produce
it -- whereas a fact with even one producer is recoverable by restarting
that producer.

We model capabilities and facts abstractly by types `Capability` and
`Fact`, with a `Produces` relation `Capability → Fact → Prop` recording
that a capability's add-effects include a given fact. A fact `p` is
`Fragile` iff no capability produces it.
-/

variable {Capability Fact : Type}

/-- `Produces c p` : capability `c` produces (adds) fact `p`. -/
def Produces (produces : Capability → Fact → Prop) (c : Capability) (p : Fact) : Prop :=
  produces c p

/-- A fact `p` is fragile relative to a `produces` relation iff no
capability in the problem produces it. -/
def Fragile (produces : Capability → Fact → Prop) (p : Fact) : Prop :=
  ¬ ∃ c, produces c p
