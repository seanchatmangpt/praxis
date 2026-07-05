/-
Label: refusal:simpleoneforone
Kind: refusal

No SimpleOneForOne; intensity > 8 is refused, not clamped: derived plans have
static step sets, and `RestartPolicy.new 9 _` returns a Refusal -- the byte
governor applies to recovery itself, and silently clamping a stated policy
would be the executor editing the operator's law.

This is formalized as a definition (no proof obligation beyond type-checking):
`RestartPolicy.new` is total, and for intensity > 8 it always yields `.refusal`,
never a clamped `.valid` policy with some other intensity substituted in.
-/

/-- A restart policy outcome: either a validly-constructed policy carrying
    its exact requested intensity, or an explicit refusal. -/
inductive RestartOutcome where
  | valid (intensity : Nat) (stepSet : List Nat)
  | refusal
  deriving Repr, DecidableEq

/-- The maximum intensity the byte governor permits. -/
def maxIntensity : Nat := 8

/-- `RestartPolicy.new` constructs a restart policy for a requested intensity
    and a static step set. If the intensity exceeds `maxIntensity`, the
    governor refuses outright -- it never clamps the intensity down to a
    permitted value and silently substitutes a different policy. -/
def RestartPolicy.new (intensity : Nat) (stepSet : List Nat) : RestartOutcome :=
  if intensity > maxIntensity then
    .refusal
  else
    .valid intensity stepSet

/-- No SimpleOneForOne clamping: requesting intensity 9 (which exceeds
    `maxIntensity = 8`) always yields a refusal, for any step set -- it is
    never silently rewritten into a `.valid` policy at some clamped
    intensity. -/
example (stepSet : List Nat) : RestartPolicy.new 9 stepSet = .refusal := by
  simp [RestartPolicy.new, maxIntensity]
