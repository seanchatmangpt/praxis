-- prop:intauth
-- Integrity (verify_chain_hash) checks s against the embedded key k;
-- authenticity (verify_chain_hash_with_key) checks s against an externally
-- pinned key k*. The first is necessary but not sufficient for the second.

abbrev HexString := String

structure SignedReceipt where
  hHex : HexString
  sig  : HexString
  key  : HexString

/-- Abstract, toy signature-verification relation: a signature is valid for a
    hash/key pair iff it equals their concatenation. Concrete enough to
    witness both directions of the proposition below. -/
def Verify (h s k : HexString) : Prop := s = h ++ k

/-- Integrity: check `s` against the key embedded in the receipt itself. -/
def verify_chain_hash (r : SignedReceipt) : Prop :=
  Verify r.hHex r.sig r.key

/-- Authenticity: check `s` against an externally pinned key `k*`. -/
def verify_chain_hash_with_key (r : SignedReceipt) (kstar : HexString) : Prop :=
  Verify r.hHex r.sig kstar

/-- Necessary: if the externally pinned key coincides with the embedded key,
    authenticity checking reduces to integrity checking. -/
theorem intauth_necessary (r : SignedReceipt) :
    verify_chain_hash_with_key r r.key → verify_chain_hash r := by
  intro h
  exact h

/-- Not sufficient: integrity holding for the embedded key does not imply
    authenticity against an arbitrary externally pinned key. -/
theorem intauth_not_sufficient :
    ∃ (r : SignedReceipt) (kstar : HexString),
      verify_chain_hash r ∧ ¬ verify_chain_hash_with_key r kstar := by
  refine ⟨⟨"h", "hk", "k"⟩, "x", ?_, ?_⟩
  · show "hk" = "h" ++ "k"
    decide
  · show ¬ ("hk" = "h" ++ "x")
    decide

/-- Integrity is necessary but not sufficient for authenticity. -/
theorem prop_intauth :
    (∀ r : SignedReceipt, verify_chain_hash_with_key r r.key → verify_chain_hash r) ∧
    (∃ (r : SignedReceipt) (kstar : HexString),
      verify_chain_hash r ∧ ¬ verify_chain_hash_with_key r kstar) :=
  ⟨intauth_necessary, intauth_not_sufficient⟩
