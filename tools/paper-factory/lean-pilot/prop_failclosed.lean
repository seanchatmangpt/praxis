-- prop:failclosed
-- When the `signed` feature is enabled, a missing or unreadable signing key is
-- a hard error (`SigningFailed`), not a silent skip.

/-- Hex-encoded strings, represented abstractly as `String`. -/
abbrev HexString := String

/-- A signed receipt: a hex hash, an Ed25519 signature (opaque, represented as a
    hex-encoded string), and the hex verifying key. Self-contained: the signature
    and key are carried alongside the hash they attest to. -/
structure SignedReceipt where
  hHex : HexString
  sig  : HexString
  key  : HexString
  deriving DecidableEq

/-- Outcome of an attempt to produce a (possibly signed) receipt. -/
inductive SignOutcome where
  | ok (r : SignedReceipt)
  | signingFailed
  | skipped
  deriving DecidableEq

/-- Whether the signing key was readable and present. -/
inductive KeyStatus where
  | present (k : HexString)
  | missingOrUnreadable
  deriving DecidableEq

/--
The receipt-signing decision procedure: given whether the `signed` feature is
enabled and the status of the signing key, decide the outcome. When `signed`
is disabled, a receipt is emitted unsigned (`skipped` — no signing attempted).
When `signed` is enabled and the key is present, signing succeeds. When
`signed` is enabled and the key is missing or unreadable, signing must
hard-fail: it can never fall back to `skipped`.
-/
def attemptSign (signedEnabled : Bool) (key : KeyStatus) (hHex : HexString) : SignOutcome :=
  if signedEnabled then
    match key with
    | KeyStatus.present k => SignOutcome.ok ⟨hHex, "sig", k⟩
    | KeyStatus.missingOrUnreadable => SignOutcome.signingFailed
  else
    SignOutcome.skipped

/--
Proposition (fail-closed): when the `signed` feature is enabled and the
signing key is missing or unreadable, the outcome is `signingFailed` — it is
never `skipped` (a silent skip) and never `ok` (a fabricated success).
-/
theorem prop_failclosed (hHex : HexString) :
    attemptSign true KeyStatus.missingOrUnreadable hHex = SignOutcome.signingFailed := by
  rfl

/-- Corollary form: fail-closed is never a silent skip. -/
theorem prop_failclosed_not_skipped (hHex : HexString) :
    attemptSign true KeyStatus.missingOrUnreadable hHex ≠ SignOutcome.skipped := by
  simp [prop_failclosed]
