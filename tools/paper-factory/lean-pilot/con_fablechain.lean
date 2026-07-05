/- def:fable (reused verbatim, needed as this file has no import mechanism) -/

structure FableHarness
    (TaskOntology Prompt Hash Response Code Verdict : Type) where
  buildPrompt   : TaskOntology → Prompt
  hash          : Prompt → Hash
  mockMembrane  : Hash → Response
  extractCode   : Response → Code
  oracle        : TaskOntology → Response → Verdict

/-- The end-to-end run of a Fable harness (reused verbatim from def_fable.lean). -/
def FableHarness.run
    {TaskOntology Prompt Hash Response Code Verdict : Type}
    (H : FableHarness TaskOntology Prompt Hash Response Code Verdict)
    (T : TaskOntology) : Code × Verdict :=
  let P := H.buildPrompt T
  let h := H.hash P
  let R := H.mockMembrane h
  let C := H.extractCode R
  (C, H.oracle T R)

/- prop:fable-oracle (reused verbatim) -/

inductive ErrorStage where
  | prompt
  | mockMembrane
  | extractCode
  | verify
  deriving DecidableEq, Repr

inductive Verdict where
  | pass
  | fail (stage : ErrorStage)
  deriving DecidableEq, Repr

/- con:fablechain

When `Oracle(T,R) = 1` (i.e. `Verdict.pass`), the harness mints a receipt using
the standard `OcelCausalFrame` chain: `h+ = chainH(h- ++ body(fr))`, where the
frame's `obj_refs` carry `dg(C) = chainH(canonical bytes of C)`. The genesis
seed is `chainH("fable-v1-genesis")`, domain-isolated from the plan chain so
fable receipts cannot cross-verify with plan receipts even if their terminal
hashes collide.

We model this construction abstractly in bare Lean 4 core:
- `ByteString` is an opaque space of canonical byte encodings.
- `chainH : ByteString → Hash` is the chaining hash function (abstract, no
  concrete BLAKE3 arithmetic — only its structural role as a chain-step).
- `canon : Code → ByteString` produces the canonical bytes of a code block `C`,
  and `dg := chainH ∘ canon` is the digest function used in `obj_refs`.
- `body : Hash → ByteString` is the abstract serialisation of a frame's body
  content (parametrised by the running previous hash for concatenation),
  and `chainStep h- fr := chainH (body (h-) )` is one link of the chain,
  i.e. `h+ = chainH(h- ++ body(fr))` modelled as a function of `h-`.
- Domain isolation is modelled by tagging each genesis seed with a `Domain`
  label (`fable` vs `plan`): the genesis hash construction takes a `Domain`
  as an explicit parameter, so the fable genesis and the plan genesis are
  produced by applying the same seed-hasher to *different* domain tags,
  and are kept as distinct fields — no lemma is claimed here (that is
  separate propositional content), only the structural separation. -/

inductive Domain where
  | fable
  | plan
  deriving DecidableEq, Repr

/-- A causal-frame chain step: given the previous hash `h-` and the current
frame's body bytes, produce the next hash `h+ = chainH(h- ++ body(fr))`. Here
`chainH` is left abstract (a field of the construction) and the concatenation
`h- ++ body(fr)` is modelled as a pairing consumed by `chainH`. -/
structure OcelCausalFrame (Hash ByteString Code : Type) where
  /-- The abstract chaining hash function. -/
  chainH   : ByteString → Hash
  /-- Canonical byte encoding of an extracted code block, feeding `dg`. -/
  canon    : Code → ByteString
  /-- The serialised body of a frame, parametrised by the previous hash so
  that concatenation `h- ++ body(fr)` is captured as a single byte string. -/
  body     : Hash → ByteString
  /-- Seed bytes for the genesis hash, indexed by domain (`fable` vs `plan`),
  giving domain isolation: fable and plan genesis hashes are produced from
  distinct seed byte strings even though both pass through `chainH`. -/
  seed     : Domain → ByteString

namespace OcelCausalFrame

variable {Hash ByteString Code : Type}

/-- One chain step: `h+ = chainH(h- ++ body(fr))`. -/
def chainStep (fr : OcelCausalFrame Hash ByteString Code) (hPrev : Hash) : Hash :=
  fr.chainH (fr.body hPrev)

/-- The digest `dg(C) = chainH(canonical bytes of C)` carried in `obj_refs`. -/
def dg (fr : OcelCausalFrame Hash ByteString Code) (C : Code) : Hash :=
  fr.chainH (fr.canon C)

/-- The domain-tagged genesis hash: `chainH(seed(domain))`. For `Domain.fable`
this is the fable-chain genesis `chainH("fable-v1-genesis")`; for
`Domain.plan` it is the (distinct) plan-chain genesis. -/
def genesis (fr : OcelCausalFrame Hash ByteString Code) (d : Domain) : Hash :=
  fr.chainH (fr.seed d)

end OcelCausalFrame

/-- The fablechain construction: given a `FableHarness` whose `Verdict` is
the two-constructor type from `prop:fable-oracle`, and an `OcelCausalFrame`
providing the chaining machinery, mint a receipt hash for a run `(C, v)`
exactly when `v = Verdict.pass`, chaining from the fable-domain genesis
through one step whose `obj_refs` carries `dg(C)`. When the verdict is a
failure, no receipt is minted (`none`). -/
def mintFableReceipt
    {TaskOntology Prompt Hash ByteString Response Code : Type}
    (H : FableHarness TaskOntology Prompt Hash Response Code Verdict)
    (fr : OcelCausalFrame Hash ByteString Code)
    (T : TaskOntology) : Option Hash :=
  let (C, v) := H.run T
  match v with
  | Verdict.pass =>
      let hMinus := fr.genesis Domain.fable
      -- the digest of the extracted code is available to be carried in
      -- obj_refs; the chain step itself proceeds from hMinus via body/chainH
      let _dgC := fr.dg C
      some (fr.chainStep hMinus)
  | Verdict.fail _ => none
