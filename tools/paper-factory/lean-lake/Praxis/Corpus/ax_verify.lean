import Mathlib.Data.BitVec

/-!
ax:verify -- key sensitivity of `Verify` (Ed25519-style signature
verification), split out of `prop_intauth.lean`.

`Verify : BitVec 512 → String → BitVec 256 → Bool` is itself declared as
an `axiom` (an opaque predicate standing for a real cryptographic
primitive; Mathlib has no model of Ed25519 or of signature schemes in
general, matching the precedent of treating BLAKE3 as an opaque,
unmodeled hash in `Praxis/Mathlib/DefReceipt.lean`).

Because `Verify` is introduced as an axiom with *no* defining equations
-- it is not `def`-ed in terms of any computation Lean can unfold --
nothing beyond `Verify`'s type can be derived about it from inside Lean.
In particular `verify_key_sensitive` (the fact that a genuine signature
scheme rejects at least one wrong key for some message) is not a
theorem *of* `Verify`: it is part of *what it means* for `Verify` to
model a real, non-degenerate signature scheme in the first place, in
the same way `chainH_cr` (`ax_cr.lean`) is not a theorem about
`msgHash` but an assumed security property of it. A genuine proof
attempt was made composing only from `Verify`'s bare type signature and
found nothing to induct or case on: `Verify` has no constructors, no
recursor, and no other axiom fixes its behavior, so `verify_key_sensitive`
must remain a second, independent axiom rather than a derived lemma.

This is why it is reclassified out of `prop_intauth.lean` (a `prop_`
file, whose own content should be limited to what it actually proves)
and into this `ax_*.lean` file, following the naming convention of
`ax_cr.lean`/`ax_obs.lean`/`ax_refusal.lean`. `prop_intauth.lean`
imports this file and references `verify_key_sensitive` by name rather
than declaring it.
-/

/-- Opaque Ed25519-style signature verification: `Verify sig hHex key` is
`true` iff `sig` is a valid signature over `hHex` under `key`. -/
axiom Verify : BitVec 512 → String → BitVec 256 → Bool

/-- A real verification predicate must be sensitive to the key: there is
some signature/message pair that verifies under one key but not another.
This is what makes "checking against the wrong key" meaningfully weaker
than "checking against the right key". Kept as an axiom (not derived):
see the module doc comment above for why this cannot be proved from
`Verify`'s bare axiomatized type. -/
axiom verify_key_sensitive :
    ∃ (sig : BitVec 512) (hHex : String) (k1 k2 : BitVec 256),
      k1 ≠ k2 ∧ Verify sig hHex k1 = true ∧ Verify sig hHex k2 = false
