-- def:signed
-- A signed receipt is the triple SignedReceipt = ⟨h_hex, s, k⟩ where
-- s = Sign_sk(h_hex) is an Ed25519 signature and k the hex verifying key;
-- it is self-contained since s and k travel with h_hex.

/-- Hex-encoded strings, represented abstractly as `String`. -/
abbrev HexString := String

/-- A signed receipt: a hex hash, an Ed25519 signature (opaque, represented as a
    hex-encoded string), and the hex verifying key. Self-contained: the signature
    and key are carried alongside the hash they attest to. -/
structure SignedReceipt where
  hHex : HexString
  sig  : HexString
  key  : HexString
