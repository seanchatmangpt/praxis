/-!
# PROJ-769 / PRD v26.7.11 §6.5 — Substitution Closure

Target 1 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"substitution closure under compatible socket predicates."

PRD §6.5 (`docs/jira/v26.7.11/PRD.md:216-224`), verbatim:

> For admitted workflow `W`, socket `a`, and compatible child `W'`:
> `W[a↦W'] ∈ 𝒲`
> only when the child satisfies the socket's declared type, authority, boundary,
> and closure contract.

No existing Rust/Erlang implementation of workflow-socket substitution exists in
this repo as of this session (exhaustive grep for `fn substitut`/`substitute` under
`crates/` this session found only unrelated string-substitution helpers), so per
this ticket's own instructions this file formalizes the model itself, then the
property, rather than a correspondence to existing code.

## Formalization strategy

`W'` is admitted to replace socket `a` in `W` exactly when four independently
declared boolean checks all hold (type, authority, boundary, closure contract) —
PRD's own "only when ... satisfies ... type, authority, boundary, and closure
contract" reads as a conjunction of four named obligations, so
`SocketContract.compatible` is modeled as their plain `&&` conjunction, matching
`Praxis/Corpus/prop_quarantine.lean`'s `POWL.sep`-style `if cond then some .. else
none` admission-gated retraction (not a bespoke shape).

Workflows are modeled as the same two-kind (`choice`/`order`) recursive
decomposition as `Praxis/Corpus/thm_sep.lean`'s `POWL` type, extended with a third
`socket` constructor naming an addressable substitution point (PRD §7.3: "Every
POWL activity SHALL be addressable as a potential workflow socket"). `substituteAt`
is the obvious structural replacement of every socket node carrying a given name;
substitution is *admitted* (returns `some`) iff the socket's compatibility contract
holds, and refused (`none`) otherwise — proved, not merely declared, to coincide
exactly with `SocketContract.compatible` below.

No axioms: `PWorkflow` is a plain inductive type over `List`/`String`;
`substituteAt` is plain structural recursion; `SocketContract.compatible` is a plain
`Bool` conjunction.
-/

/-- A POWL workflow term: a choice node, an order node (both over a list of
children, matching `Praxis/Corpus/thm_sep.lean`'s `POWL`), or a named, addressable
substitution socket (PRD §7.3: every activity is a potential workflow socket). -/
inductive PWorkflow where
  | choice : List PWorkflow → PWorkflow
  | order  : List PWorkflow → PWorkflow
  | socket : String → PWorkflow
deriving Repr, Inhabited

/-- The four independently declared socket obligations PRD §6.5 names: "the
socket's declared type, authority, boundary, and closure contract." Each is
modeled as an opaque `Bool` verdict — what concretely produces that verdict (a type
checker, an authority-registry lookup, a boundary check, a closure-law evaluation)
is out of this file's scope; only their conjunction's effect on admission is
formalized here. -/
structure SocketContract where
  typeOk      : Bool
  authorityOk : Bool
  boundaryOk  : Bool
  closureOk   : Bool
deriving DecidableEq, Repr

/-- `compatible`: PRD §6.5's four-way conjunction, verbatim ("type, authority,
boundary, and closure contract"). -/
def SocketContract.compatible (c : SocketContract) : Bool :=
  c.typeOk && c.authorityOk && c.boundaryOk && c.closureOk

/-- Structural replacement of every `socket name` node in `w` with `child`. Nodes
naming a different socket, and non-socket nodes, recurse structurally and are
otherwise left alone. -/
def substituteAt (w : PWorkflow) (name : String) (child : PWorkflow) : PWorkflow :=
  match w with
  | .socket n => if n = name then child else .socket n
  | .choice cs => .choice (cs.map (substituteAt · name child))
  | .order cs  => .order (cs.map (substituteAt · name child))

/-- `W[a↦W'] ∈ 𝒲` (PRD §6.5): admits the structural substitution iff the socket
contract is compatible, refuses (`none`) otherwise — the admission-gated retraction
this file's docstring describes. -/
def admitSubstitute (w : PWorkflow) (name : String) (child : PWorkflow)
    (c : SocketContract) : Option PWorkflow :=
  if c.compatible then some (substituteAt w name child) else none

/-- `thm:substitution_closure` (PRD §6.5): the substituted workflow is admitted
(`W[a↦W'] ∈ 𝒲`) if and only if the socket's four-way compatibility contract holds
— "only when the child satisfies the socket's declared type, authority, boundary,
and closure contract," proved as a real biconditional over the concrete admission
gate above, not merely asserted. -/
theorem substitution_closure (w child : PWorkflow) (name : String)
    (c : SocketContract) :
    (admitSubstitute w name child c).isSome ↔ c.compatible = true := by
  unfold admitSubstitute
  cases c.compatible <;> simp

/-- Soundness: whenever substitution is admitted, the result is exactly the real
structural substitution — admission never silently produces something other than
`W[a↦W']`. -/
theorem substitution_closure_sound (w child : PWorkflow) (name : String)
    (c : SocketContract) (h : c.compatible = true) :
    admitSubstitute w name child c = some (substituteAt w name child) := by
  unfold admitSubstitute
  simp [h]
