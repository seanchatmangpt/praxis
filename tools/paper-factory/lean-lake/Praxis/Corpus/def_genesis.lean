import Praxis.Corpus.def_contentaddr

/-!
def:genesis, reformalized in the Mathlib lane.

"The genesis chain value is $\Genesis=\chainH(\bm 0_{32})$; a run may
additionally bind a domain seed
$\Genesis_{\mathrm{dom}}=\chainH(\text{project-version-genesis})$ so chains
from distinct projects or versions cannot cross-verify."

Composed from pre-built pieces rather than fresh axioms:

* Reuses `Digest := BitVec 256` and the one genuine hash axiom `chainH`
  from `Praxis.Corpus.def_contentaddr` (`def:contentaddr`) rather than
  redeclaring them -- both concepts are the same BLAKE3 chain hash from
  the corpus.
* $\bm 0_{32}$, the 32-byte all-zero string, is built from core's
  `ByteArray.mk` over `Array.replicate 32 (0 : UInt8)`, not a fresh axiom --
  it is a fully concrete, computable value.
* The domain seed string `"project-version-genesis"` is encoded to bytes
  via core's `String.toUTF8`, again fully concrete and computable, no
  axiom needed.
* `chainH` itself remains the one genuine axiom (already justified in
  `def_contentaddr.lean`): a real cryptographic hash function with no
  Mathlib/Lean-core implementation, out of scope for this pilot to
  construct.
-/

/-- The 32-byte all-zero string $\bm 0_{32}$, fully concrete. -/
def zeros32 : ByteArray :=
  ByteArray.mk (Array.replicate 32 (0 : UInt8))

/-- The genesis chain value $\Genesis = \chainH(\bm 0_{32})$. -/
noncomputable def genesis : Digest :=
  chainH zeros32

/-- The domain seed string, UTF-8 encoded. -/
def domainSeedBytes : ByteArray :=
  String.toUTF8 "project-version-genesis"

/-- The domain-bound genesis value
$\Genesis_{\mathrm{dom}} = \chainH(\text{project-version-genesis})$. -/
noncomputable def genesisDom : Digest :=
  chainH domainSeedBytes
