import Praxis.Corpus.def_signed

/-!
prop:intauth

Integrity (`verify_chain_hash`) checks `s` against the embedded key `k`;
authenticity (`verify_chain_hash_with_key`) checks `s` against an
externally pinned key `k star`. The first is necessary but not sufficient
for the second.

Composition over axiomatization:

* We reuse `SignedReceipt` from `def:signed` (`Praxis/Corpus/def_signed.lean`)
  rather than redefining the triple `⟨hHex, sig, key⟩`.
* `Verify : BitVec 512 → String → BitVec 256 → Bool` models Ed25519
  signature verification (signature, hashed message, key ↦ accept/reject).
  This is kept as an `axiom` (an opaque predicate, not a defined function)
  because it stands for a real cryptographic primitive (Ed25519 verify);
  Mathlib has no model of Ed25519 or of signature schemes in general, so
  there is no pre-built equivalent to compose from -- matching the
  precedent in `Praxis/Mathlib/DefReceipt.lean` of treating a concrete
  cryptographic hash as an opaque, unmodeled operation.
* The only extra fact assumed about `Verify` is `verify_key_sensitive`:
  a genuine signature scheme must be able to reject a wrong key for some
  signed message (otherwise it verifies nothing). This is the minimal,
  scheme-agnostic property needed to witness "not sufficient" below; it is
  not a bespoke axiom invented for convenience, but the defining
  correctness property of any signature verification predicate.

`verify_chain_hash` checks the signature against the key embedded in the
receipt itself; `verify_chain_hash_with_key` checks it against an
independently supplied (pinned) key `kStar`.
-/

/-- Opaque Ed25519-style signature verification: `Verify sig hHex key` is
`true` iff `sig` is a valid signature over `hHex` under `key`. -/
axiom Verify : BitVec 512 → String → BitVec 256 → Bool

/-- A real verification predicate must be sensitive to the key: there is
some signature/message pair that verifies under one key but not another.
This is what makes "checking against the wrong key" meaningfully weaker
than "checking against the right key". -/
axiom verify_key_sensitive :
    ∃ (sig : BitVec 512) (hHex : String) (k1 k2 : BitVec 256),
      k1 ≠ k2 ∧ Verify sig hHex k1 = true ∧ Verify sig hHex k2 = false

/-- Integrity: verify `s` against the key embedded in the receipt. -/
noncomputable def verify_chain_hash (r : SignedReceipt) : Bool :=
  Verify r.sig r.hHex r.key

/-- Authenticity: verify `s` against an externally pinned key `kStar`. -/
noncomputable def verify_chain_hash_with_key (r : SignedReceipt) (kStar : BitVec 256) : Bool :=
  Verify r.sig r.hHex kStar

/-- Necessary: if the pinned key is exactly the key the receipt already
carries, passing the authenticity check forces the integrity check to
pass too (both reduce to the very same `Verify` call). -/
theorem intauth_necessary (r : SignedReceipt) (kStar : BitVec 256)
    (hk : kStar = r.key) (h : verify_chain_hash_with_key r kStar = true) :
    verify_chain_hash r = true := by
  simpa [verify_chain_hash, verify_chain_hash_with_key, hk] using h

/-- Not sufficient: there is a receipt whose embedded-key (integrity)
check passes while the externally-pinned-key (authenticity) check fails,
i.e. integrity alone does not entail authenticity. -/
theorem intauth_not_sufficient :
    ∃ (r : SignedReceipt) (kStar : BitVec 256),
      verify_chain_hash r = true ∧ verify_chain_hash_with_key r kStar = false := by
  obtain ⟨sig, hHex, k1, k2, hne, h1, h2⟩ := verify_key_sensitive
  refine ⟨⟨hHex, sig, k1⟩, k2, ?_, ?_⟩
  · simpa [verify_chain_hash] using h1
  · simpa [verify_chain_hash_with_key] using h2

/-- prop:intauth: integrity (`verify_chain_hash`) is necessary but not
sufficient for authenticity (`verify_chain_hash_with_key`). -/
theorem intauth :
    (∀ (r : SignedReceipt) (kStar : BitVec 256), kStar = r.key →
      verify_chain_hash_with_key r kStar = true → verify_chain_hash r = true) ∧
    (∃ (r : SignedReceipt) (kStar : BitVec 256),
      verify_chain_hash r = true ∧ verify_chain_hash_with_key r kStar = false) :=
  ⟨intauth_necessary, intauth_not_sufficient⟩
