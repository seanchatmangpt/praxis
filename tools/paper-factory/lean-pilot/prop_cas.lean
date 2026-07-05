/-
Label: prop:cas
Kind: proposition

Statement: In a content-addressed lake, storing the same receipt twice is
idempotent, and mutating a stored receipt changes its address, so the old
address still resolves to the old bytes: history is immutable by
construction.

We reuse `def:contentaddr`'s model: objects are bitstrings `Bytes`, and
`content_address b` is the hex digest `chainH b` (opaque, deterministic).

A lake is modeled as a `Store : String → Option Bytes`, mapping an address to
the object stored there (if any). `put s b` stores `b` at its content
address, leaving every other address of `s` untouched.

We prove:
  1. `put_idempotent`   — storing the same object twice yields the same store.
  2. `old_address_persists` — if a *different* object `b'` (with a distinct
     content address) is subsequently stored, the original object `b` is
     still found at its own address `content_address b`: mutating via a
     different object cannot alter what the old address resolves to.
-/

/-- A stored object, represented as an arbitrary bitstring (same as
    `def:contentaddr`). -/
abbrev Bytes := List Bool

/-- An abstract deterministic hash function into a hex-encoded digest string. -/
opaque chainH : Bytes → String

/-- The content address of a stored object `b`. -/
def content_address (b : Bytes) : String := chainH b

/-- A content-addressed lake: a partial map from address to stored bytes. -/
def Store := String → Option Bytes

/-- Store `b` at its content address, leaving all other addresses of `s`
    unchanged. -/
def put (s : Store) (b : Bytes) : Store :=
  fun a => if a = content_address b then some b else s a

/-- `prop:cas` (part 1) — storing the same receipt twice is idempotent. -/
theorem put_idempotent (s : Store) (b : Bytes) :
    put (put s b) b = put s b := by
  funext a
  simp only [put]
  by_cases h : a = content_address b
  · rw [if_pos h, if_pos h]
  · rw [if_neg h, if_neg h]

/-- `prop:cas` (part 2) — mutating the lake by storing a *different* object
    `b'` (whose content address differs from `b`'s) does not disturb the
    old address: it still resolves to the old bytes `b`. History at
    `content_address b` is immutable under such a mutation. -/
theorem old_address_persists (s : Store) (b b' : Bytes)
    (h : content_address b' ≠ content_address b) :
    put (put s b) b' (content_address b) = some b := by
  unfold put
  rw [if_neg h.symm, if_pos rfl]
