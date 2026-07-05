/-
Label: lineage:armstrong
Kind: lineage

Erlang/OTP made three moves that survive every re-examination: processes are
isolated so failure cannot spread by memory; supervisors -- not the failing
code -- own the recovery decision; and restart strategies plus a restart
intensity turn `let it crash' into a bounded, structured discipline. What OTP
does not do is derive the tree: a human writes the supervision hierarchy, and
the crash space lives in the programmer's head.

This is a historical/lineage statement, not a formalizable mathematical
claim. We record its structural content as a bare Lean 4 (core-only)
data type: the three properties OTP established, plus the one thing it
leaves to the human (the supervision tree is not derived).
-/

/-- The three structural moves OTP is credited with. -/
structure OTPMoves where
  isolatesFailure   : Prop  -- processes are isolated: failure cannot spread by memory
  supervisorOwnsRecovery : Prop  -- supervisors, not failing code, decide recovery
  boundedRestart    : Prop  -- restart strategy + restart intensity bounds "let it crash"

/-- What OTP explicitly does *not* provide: derivation of the supervision tree. -/
structure OTPGap where
  humanWritesTree : Prop  -- the supervision hierarchy is authored by a human
  crashSpaceInHead : Prop  -- the crash space is not externalized, only in the programmer's head

/-- The lineage claim bundles the three moves together with the acknowledged gap. -/
structure ArmstrongLineage where
  moves : OTPMoves
  gap   : OTPGap

#check @ArmstrongLineage
