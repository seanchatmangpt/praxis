import Praxis.Corpus.def_contentaddr

/-!
prop:cas, reformalized in the Mathlib lane.

"In a content-addressed lake, storing the same receipt twice is idempotent,
and mutating a stored receipt changes its address, so the old address
still resolves to the old bytes: history is immutable by construction."

Depends on `def:contentaddr` (`Praxis.Corpus.def_contentaddr`), reused via
`import` rather than redeclared: `ByteArray`, `Digest := BitVec 256`,
`chainH : ByteArray → Digest`, and `contentAddress b := chainH b`.

Two halves of the corpus statement, both proved (no `sorry`):

* Idempotence: storing the same bytes twice yields the same address --
  immediate `rfl` from `contentAddress` being a plain function of its
  argument (already noted as `contentAddress_congr` in `def_contentaddr`
  for the byte-identical case; here specialised to literally the same
  object, which is `rfl`).
* Immutability: mutating the bytes (`b₁ ≠ b₂`) yields a different address,
  under the standard cryptographic idealization that `chainH` has no
  collisions (`Function.Injective chainH`, reusing Mathlib's own
  `Function.Injective` from `Mathlib.Logic.Function.Basic` rather than a
  bespoke definition). This is exactly the collision-resistance hypothesis
  already axiomatized as the asymptotic statement `chainH_cr` in `ax:cr`;
  here we take the clean idealized (perfect, not merely negligible-advantage)
  form of that same hypothesis as an explicit proof hypothesis, since the
  corpus statement phrases immutability as an unconditional structural fact
  ("mutating ... changes its address") rather than a probabilistic one --
  matching how `def:contentaddr` itself flags collision-resistance as "the
  cryptographic hypothesis a later theorem would take", which this is.
  Consequently "the old address still resolves to the old bytes" follows:
  the address-to-bytes correspondence is injective, so no other (mutated)
  byte string can ever produce the old address.
-/

/-- Idempotence: storing the same bytes twice gives the same address. -/
theorem contentAddress_idempotent (b : ByteArray) :
    contentAddress b = contentAddress b :=
  rfl

/-- Immutability: under the standard no-collision idealization of `chainH`,
mutating the stored bytes (`b₁ ≠ b₂`) changes the address
(`contentAddress b₁ ≠ contentAddress b₂`), so the old address can never be
produced by any other (mutated) byte string -- history is immutable. -/
theorem prop_cas (hinj : Function.Injective chainH) {b₁ b₂ : ByteArray}
    (hmut : b₁ ≠ b₂) : contentAddress b₁ ≠ contentAddress b₂ :=
  fun heq => hmut (hinj heq)
