/-
def:walframe

A cache frame journaling the memoized DAG nodes is serialized as a
length-prefixed frame:
  WAL_Frame = ⟨ length, chainH(payload), payload ⟩
allowing recovery to discard torn frames.

We model the payload as a byte-string (`List Bool` for bit-level generality
would be heavier than needed; we use `List Nat` to stand in for a byte
sequence), the hash `chainH` as an opaque function from payloads to a hash
type, and the frame as a structure pairing a declared length, the hash of
the payload, and the payload itself.
-/

abbrev Payload := List Nat

abbrev Hash := Nat

/-- Opaque chaining hash function over payloads (abstract stand-in for
`chainH` from the thesis; its concrete definition lives elsewhere). -/
def chainH (p : Payload) : Hash :=
  p.foldl (fun acc b => acc * 31 + b + 1) 0

/-- A length-prefixed write-ahead-log frame: declared length, hash of the
payload, and the payload itself. Recovery can discard a frame as "torn"
if `length` does not match the actual payload length or the hash does not
match `chainH payload`. -/
structure WAL_Frame where
  length  : Nat
  hash    : Hash
  payload : Payload

/-- Construct a well-formed WAL frame from a payload, filling in the
length and hash fields consistently. -/
def WAL_Frame.mk_from (p : Payload) : WAL_Frame :=
  { length := p.length, hash := chainH p, payload := p }

/-- A frame is intact (not torn) iff its declared length and hash agree
with the payload it carries. -/
def WAL_Frame.intact (f : WAL_Frame) : Prop :=
  f.length = f.payload.length ∧ f.hash = chainH f.payload
