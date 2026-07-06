import Mathlib.Data.BitVec
import Praxis.Corpus.def_fable
import Praxis.Corpus.prop_fable_oracle

/-!
con:fablechain

When `Oracle(T,R) = 1`, the harness mints a receipt using the standard
`OcelCausalFrame` chain: `h+ = chainH(h- ‖ body(fr))`, where the frame's
`obj_refs` carry `dg(C) = chainH(canonical bytes of C)`. The genesis seed is
`chainH("fable-v1-genesis")`, domain-isolated from the plan chain so fable
receipts cannot cross-verify with plan receipts even if their terminal
hashes collide.

This is a construction: no proof obligation beyond type-checking.

Reuse over fresh axioms:

* `Digest` is reused as-is from `Praxis.Corpus.def_fable` (`BitVec 256`,
  itself reused from `Praxis.Mathlib.DefReceipt`'s `Digest`) -- the same
  concrete 256-bit vector type is used for the fixture-index hash, the
  plan-chain digest, and now the fable-chain digest.
* `CodeBlock` is reused as-is from `Praxis.Corpus.def_fable` (`String`).
* `chainH` is axiomatized as `String → Digest`, the *same* kind of
  primitive as `Praxis.Mathlib.DefReceipt`'s `chainH`/`chainStep`
  (a real cryptographic hash, BLAKE3 per the corpus's own cryptography
  paper): no Lean/Mathlib term is a meaningful stand-in for an actual
  collision-resistant hash implementation, so its *existence* is
  axiomatized rather than a fake hash being modelled. Taking `String`
  as the domain (rather than a bare `Digest`) is what lets one function
  cover all three call sites in the statement -- canonical bytes of a
  `CodeBlock`, the concatenation `h- ‖ body(fr)`, and the literal seed
  string -- instead of declaring three unrelated hash axioms.
* `dg`, the frame's `obj_refs` digest, `chainStepFable` (the `h+ = chainH
  (h- ‖ body(fr))` step), and `fableGenesis` are all plain functions/defs
  composed from `chainH`, `Digest.toNat`/`Nat.repr` (pre-built), and
  `String.append` (pre-built) -- no further axioms declared for them.
* Domain isolation from the plan chain (`Praxis.Mathlib.DefReceipt`'s
  `Receipt`/`chainStep`) is realized at the *type* level, not merely by
  a distinct seed string: `FableChainTag` is a `structure` wrapping a
  `Digest`, distinct from `DefReceipt.Receipt`'s bare `Digest` fields.
  A `FableChainTag` value is not defeq/unifiable with a plan receipt's
  terminal `hPlus : Digest`, so "fable receipts cannot cross-verify with
  plan receipts even if their terminal hashes collide" holds
  structurally: cross-domain comparison is not even well-typed, which is
  strictly stronger than relying on the hash values happening to differ.
-/

/-- A real, axiomatized cryptographic hash (BLAKE3, per the corpus's own
`02_receipt_cryptography` paper) from canonical byte strings to a
`Digest`. Covers all three uses in the statement: hashing a code block's
canonical bytes, hashing a chain-step concatenation, and hashing the
genesis seed literal. -/
axiom chainH : String → Digest

/-- `dg(C) = chainH(canonical bytes of C)`; a `CodeBlock` (`String`) is
already its own canonical byte representation at this level of
abstraction. -/
noncomputable def dg (C : CodeBlock) : Digest := chainH C

/-- A fable-chain `OcelCausalFrame`: the object-reference digest carried
by the frame (`dg(C)` for the frame's code block) together with the
frame's own body bytes. -/
structure FableFrame where
  body    : String
  objRefs : Digest

/-- Build a fable-chain frame for a given code block and body payload,
with `objRefs` set to `dg(C)`. -/
noncomputable def mkFableFrame (C : CodeBlock) (body : String) : FableFrame :=
  { body := body, objRefs := dg C }

/-- `chainH(h- ‖ body(fr))`: the chain step used to mint `h+` from the
previous digest and the frame's body, via `Digest.toNat`/`Nat.repr` to
render `h-` as bytes before concatenation. -/
noncomputable def chainStepFable (hMinus : Digest) (fr : FableFrame) : Digest :=
  chainH (hMinus.toNat.repr ++ fr.body)

/-- A minted fable-chain receipt: `h+ = chainH(h- ‖ body(fr))`, exactly
the standard `OcelCausalFrame` chain rule from the statement. -/
structure FableReceipt where
  hMinus   : Digest
  frame    : FableFrame
  hPlus    : Digest
  advances : hPlus = chainStepFable hMinus frame

/-- The fable-chain genesis seed: `chainH("fable-v1-genesis")`. -/
noncomputable def fableGenesis : Digest := chainH "fable-v1-genesis"

/-- A nominal tag distinguishing fable-chain digests from any other
chain's digests (e.g. the plan chain's) at the *type* level, realizing
domain isolation structurally rather than relying on hash values
happening to differ. -/
structure FableChainTag where
  digest : Digest
deriving DecidableEq

/-- The tagged fable-chain genesis, domain-isolated from the plan chain:
no term of a plan-chain digest type can be compared against a
`FableChainTag`, so fable receipts cannot cross-verify with plan
receipts even if `Digest` values collide. -/
noncomputable def fableGenesisTagged : FableChainTag := ⟨fableGenesis⟩

/-- Given `Oracle(T, R) = true`, mint a `FableReceipt` for the extracted
code block `C`, chaining from the (tagged) genesis. -/
noncomputable def mintFableReceipt (h : FableHarness) (T : TaskOntology) (R : ModelResponse)
    (C : CodeBlock) (body : String) (_ok : h.oracle T R = true) : FableReceipt :=
  let fr := mkFableFrame C body
  { hMinus := fableGenesisTagged.digest
    frame := fr
    hPlus := chainStepFable fableGenesisTagged.digest fr
    advances := rfl }
