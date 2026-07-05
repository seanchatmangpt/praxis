/- def:logicadm
   prolog8 admits a query, fact block, or rule only if it lies in a bounded,
   decidable Horn fragment: arity ≤ 8, rule body ≤ 8 atoms, ≤ 8 variables,
   proof fan-in ≤ 8; no cut, no dynamic mutation, stratified negation, bounded
   recursion, no runtime text parsing, no side effects, interned terms;
   violations are enumerated by RejectionCode. -/

/-- Reasons a prolog8 construct can be rejected from the bounded Horn fragment. -/
inductive RejectionCode where
  | arityExceeded
  | bodyTooLong
  | tooManyVariables
  | fanInExceeded
  | cutUsed
  | dynamicMutation
  | unstratifiedNegation
  | unboundedRecursion
  | runtimeTextParsing
  | sideEffect
  | nonInternedTerm
  deriving Repr, DecidableEq

/-- The measurable shape of a candidate query, fact block, or rule. -/
structure Prolog8Shape where
  arity        : Nat
  bodyAtoms    : Nat
  variables    : Nat
  proofFanIn   : Nat
  usesCut            : Bool
  dynamicMutation    : Bool
  stratifiedNegation : Bool
  boundedRecursion   : Bool
  runtimeTextParsing : Bool
  sideEffects        : Bool
  internedTerms      : Bool

/-- The bound shared by every dimension of the fragment. -/
def prolog8Bound : Nat := 8

/-- `admits s` holds iff `s` lies in the bounded, decidable Horn fragment that
    prolog8 accepts: every numeric dimension is within `prolog8Bound`, no cut,
    no dynamic mutation, negation is stratified, recursion is bounded, no
    runtime text parsing, no side effects, and all terms are interned. -/
def admits (s : Prolog8Shape) : Prop :=
  s.arity ≤ prolog8Bound ∧
  s.bodyAtoms ≤ prolog8Bound ∧
  s.variables ≤ prolog8Bound ∧
  s.proofFanIn ≤ prolog8Bound ∧
  s.usesCut = false ∧
  s.dynamicMutation = false ∧
  s.stratifiedNegation = true ∧
  s.boundedRecursion = true ∧
  s.runtimeTextParsing = false ∧
  s.sideEffects = false ∧
  s.internedTerms = true

instance (s : Prolog8Shape) : Decidable (admits s) := by
  unfold admits; infer_instance

/-- Every rejected shape is annotated with the specific reasons it fails. -/
def rejections (s : Prolog8Shape) : List RejectionCode :=
  (if s.arity > prolog8Bound then [RejectionCode.arityExceeded] else []) ++
  (if s.bodyAtoms > prolog8Bound then [RejectionCode.bodyTooLong] else []) ++
  (if s.variables > prolog8Bound then [RejectionCode.tooManyVariables] else []) ++
  (if s.proofFanIn > prolog8Bound then [RejectionCode.fanInExceeded] else []) ++
  (if s.usesCut then [RejectionCode.cutUsed] else []) ++
  (if s.dynamicMutation then [RejectionCode.dynamicMutation] else []) ++
  (if ¬ s.stratifiedNegation then [RejectionCode.unstratifiedNegation] else []) ++
  (if ¬ s.boundedRecursion then [RejectionCode.unboundedRecursion] else []) ++
  (if s.runtimeTextParsing then [RejectionCode.runtimeTextParsing] else []) ++
  (if s.sideEffects then [RejectionCode.sideEffect] else []) ++
  (if ¬ s.internedTerms then [RejectionCode.nonInternedTerm] else [])
