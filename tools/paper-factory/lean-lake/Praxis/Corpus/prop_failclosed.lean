import Mathlib.Data.BitVec
import Praxis.Corpus.def_signed

/-!
prop:failclosed

"When the `signed` feature is enabled, a missing or unreadable signing
key is a hard error (`SigningFailed`), not a silent skip."

Composition over axiomatization:

* We reuse `SignedReceipt` from `Praxis.Corpus.def_signed` verbatim
  (imported, not redefined) for the success shape.
* The signing key's presence/absence is modeled by `Option (BitVec 256)`
  -- `none` stands for "missing or unreadable", exactly matching
  `Option`'s built-in semantics, so no new opaque "key lookup result"
  type is introduced.
* `SignOutcome` is a two-constructor inductive: either a produced
  `SignedReceipt`, or `SigningFailed`. This is the smallest possible
  encoding of "hard error, not silent skip" -- there is no third,
  silently-degraded constructor available to return, so the *type*
  itself rules out a silent skip; the proposition below additionally
  pins down the *value* returned in the missing-key case.
* `signAttempt` is a total, decidable function (a `match`), not an
  axiom: whether the feature is enabled and whether the key is present
  are both concrete, decidable facts already available to us as
  `Bool`/`Option`, so composing them needs no new axiom.

No axioms are introduced.
-/

inductive SignOutcome where
  | signed (r : SignedReceipt)
  | SigningFailed

/-- Attempt to produce a signed receipt. When `enabled` is `true` and
`key` is `none` (missing or unreadable), the attempt hard-fails with
`SigningFailed`; it never silently falls back to an unsigned or
default-keyed receipt. -/
def signAttempt (enabled : Bool) (key : Option (BitVec 256)) (hHex : String)
    (sig : BitVec 512) : SignOutcome :=
  match enabled, key with
  | true, none => .SigningFailed
  | true, some k => .signed ⟨hHex, sig, k⟩
  | false, none => .SigningFailed
  | false, some k => .signed ⟨hHex, sig, k⟩

/-- prop:failclosed -- when the `signed` feature is enabled and the
signing key is missing or unreadable (`none`), the outcome is exactly
`SigningFailed`, for every hash digest and signature value: a hard
error, never a silent skip. -/
theorem prop_failclosed (hHex : String) (sig : BitVec 512) :
    signAttempt true none hHex sig = SignOutcome.SigningFailed := by
  simp [signAttempt]
