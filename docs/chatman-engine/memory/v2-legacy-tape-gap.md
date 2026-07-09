# V2-to-Legacy Tape Gap

**Summary**: No v2-to-legacy `PowlTape` conversion exists in bcinr-powl; a bridge must build
both tape forms from source ops.

**Source evidence**: This session's audit of bcinr-powl; no conversion function between the v2
and legacy `PowlTape` representations was found.

**Why it matters**: Assuming a converter exists leads to code that silently produces one tape
form and mislabels it, or to duplicate `PowlTape` definitions (a static-gate failure).

**Future instruction**: When both tape forms are needed, generate each directly from the
source operations. Do not write a v2-to-legacy converter ad hoc, and never duplicate the
`PowlTape` type.
