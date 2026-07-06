import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic

/-!
# def:logicadm — Admission fragment for `prolog8`

`prolog8` admits a query, fact, or rule only if it lies in a bounded, decidable Horn
fragment: arity ≤ 8, rule body ≤ 8 atoms, ≤ 8 variables, proof fan-in ≤ 8; no cut, no
dynamic mutation, stratified negation, bounded recursion, no runtime text parsing, no
side effects, interned terms. Violations are enumerated by `RejectionCode`.

We fix the bound `8` as a `Nat` literal (no Mathlib abstraction needed for a literal
constant), model atoms/rules as finite-arity/finite-body records bounded by `List.length`
constraints (reusing `List` and its Mathlib-free `length`, since the thesis text gives no
concrete term syntax to typecheck against — the atom/term payload is left abstract), and
give the admission check as a `Prop`-valued (in fact decidable, via `decide`-friendly
`Nat.decLe`) conjunction of the bound predicates plus a `Bool` structural-hygiene flag
(no cut / no dynamic mutation / stratified negation / bounded recursion / no runtime
parsing / no side effects / interned terms) that the source leaves as an abstract
opaque discipline rather than something with an invented concrete syntax to check.
`RejectionCode` enumerates exactly the ways admission can fail, one constructor per
disjunct of the negated admission condition.
-/

namespace Praxis.Corpus.DefLogicAdm

/-- The fragment bound: arity, body length, variable count, and proof fan-in are all
capped at `8`. -/
def bound : Nat := 8

/-- An atom: a predicate symbol name together with its argument arity. Argument terms
themselves are left abstract (`Term`), since the thesis fixes no concrete term syntax. -/
structure Atom (Term : Type) where
  /-- Predicate symbol. -/
  pred : String
  /-- Argument terms. -/
  args : List Term

/-- Arity of an atom: the number of argument terms. -/
def Atom.arity {Term : Type} (a : Atom Term) : Nat := a.args.length

/-- A Horn rule: a head atom, a body of atoms (the conjunction of premises), and the
list of distinct variables occurring in the rule (variables are abstracted as `Var`,
since no concrete variable syntax is fixed by the source). -/
structure Rule (Term Var : Type) where
  /-- Rule head. -/
  head : Atom Term
  /-- Rule body: a conjunction of atoms. -/
  body : List (Atom Term)
  /-- Distinct variables occurring in the rule. -/
  vars : List Var

/-- Structural hygiene discipline required of every admitted clause: no cut, no
dynamic mutation, stratified negation, bounded recursion, no runtime text parsing, no
side effects, and interned terms. The thesis text names these as qualitative
engineering disciplines with no single concrete formal syntax fixed for them in this
source, so each is carried as an abstract `Bool` flag (`true` = discipline holds)
rather than invented as a bespoke concrete check. -/
structure Hygiene where
  /-- No `cut` operator used. -/
  noCut : Bool
  /-- No dynamic mutation (assert/retract) of the fact/rule base. -/
  noDynamicMutation : Bool
  /-- Negation, where present, is stratified. -/
  stratifiedNegation : Bool
  /-- Recursion, where present, is bounded. -/
  boundedRecursion : Bool
  /-- No runtime parsing of text into terms. -/
  noRuntimeTextParsing : Bool
  /-- No side effects during evaluation. -/
  noSideEffects : Bool
  /-- All terms are interned (hash-consed / canonicalized). -/
  internedTerms : Bool

/-- All seven hygiene disciplines hold. -/
def Hygiene.holds (h : Hygiene) : Prop :=
  h.noCut = true ∧ h.noDynamicMutation = true ∧ h.stratifiedNegation = true ∧
    h.boundedRecursion = true ∧ h.noRuntimeTextParsing = true ∧
    h.noSideEffects = true ∧ h.internedTerms = true

/-- Proof fan-in of a rule: the number of body atoms feeding a single derivation step
(the same quantity as the body length, per the source's Horn-clause reading of
"proof fan-in"). -/
def Rule.fanIn {Term Var : Type} (r : Rule Term Var) : Nat := r.body.length

/-- A rule lies in the bounded Horn fragment iff its head arity, body length, variable
count, and fan-in are all `≤ bound`. -/
def Rule.inFragment {Term Var : Type} (r : Rule Term Var) : Prop :=
  r.head.arity ≤ bound ∧ r.body.length ≤ bound ∧ r.vars.length ≤ bound ∧ r.fanIn ≤ bound

/-- A bare query or fact (no body) lies in the bounded fragment iff its arity and
variable count are `≤ bound`. -/
def Atom.inFragment {Term Var : Type} (a : Atom Term) (vars : List Var) : Prop :=
  a.arity ≤ bound ∧ vars.length ≤ bound

/-- The reasons `prolog8` may refuse a query, fact, or rule: each constructor names one
disjunct of the negated admission condition. -/
inductive RejectionCode where
  /-- Arity exceeds the bound. -/
  | arityExceeded : RejectionCode
  /-- Rule body length exceeds the bound. -/
  | bodyExceeded : RejectionCode
  /-- Variable count exceeds the bound. -/
  | varsExceeded : RejectionCode
  /-- Proof fan-in exceeds the bound. -/
  | fanInExceeded : RejectionCode
  /-- A `cut` operator was used. -/
  | cutUsed : RejectionCode
  /-- The fact/rule base was mutated dynamically. -/
  | dynamicMutation : RejectionCode
  /-- Negation present is not stratified. -/
  | unstratifiedNegation : RejectionCode
  /-- Recursion present is not bounded. -/
  | unboundedRecursion : RejectionCode
  /-- Runtime text was parsed into terms. -/
  | runtimeTextParsed : RejectionCode
  /-- A side effect occurred during evaluation. -/
  | sideEffect : RejectionCode
  /-- Terms were not interned. -/
  | termsNotInterned : RejectionCode
  deriving DecidableEq, Repr

/-- Full admission of a rule into `prolog8`: it lies in the bounded Horn fragment and
all hygiene disciplines hold. -/
def Rule.admitted {Term Var : Type} (r : Rule Term Var) (h : Hygiene) : Prop :=
  r.inFragment ∧ h.holds

end Praxis.Corpus.DefLogicAdm
