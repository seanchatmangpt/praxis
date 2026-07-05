/-
def:receipt (00_foundations / projection_thesis) — formalized as a structure.

Original (LaTeX, projection_thesis's fuller version):
  Fix a collision-resistant hash chainH : {0,1}* -> {0,1}^256, write dg(.) = chainH(.).
  For a manufacture with admitted observation x = adm(o), obligation set G with denial
  word d(o), transition identity tau, artifact a = muop(x), replay result Fitness,
  refusal reason r, and implementation version v, the frame is
    fr = <dg(x), dg(G), d(o), tau, dg(a), Fitness, r, v>,
  and the chained receipt advances by h+ = chainH(h- ++ fr).

This is a DEFINITION, not a theorem — there is no proof obligation, only the
requirement that it type-checks as a well-formed Lean structure. `chainH` is
modeled abstractly as an opaque function Digest -> Digest (its cryptographic
properties — collision resistance — are NOT modeled here; that belongs to a
separate axiom/hypothesis a later theorem would take as a parameter, not to
this definition).
-/

/-- A 256-bit digest, modeled abstractly as an opaque type (its internal
    representation is irrelevant to this formalization). -/
axiom Bits256 : Type

abbrev Digest := Bits256

/-- The obligation-battery denial word, transition identity, replay/fitness
    result, refusal reason, and version are all modeled abstractly for now —
    each is its own separate formalization target the graph's dependsOn
    edges would name, not something to invent shortcuts for here. -/
axiom DenialWord : Type
axiom TransitionId : Type
axiom Fitness : Type
axiom RefusalReason : Type
axiom Version : Type

/-- One receipt frame: the eight committed fields named in the LaTeX,
    in the same order. -/
structure Frame where
  dgX : Digest          -- dg(x), digest of the admitted observation
  dgG : Digest          -- dg(G), digest of the obligation set
  denial : DenialWord   -- d(o)
  transition : TransitionId  -- tau
  dgA : Digest          -- dg(a), digest of the artifact
  fitness : Fitness
  reason : RefusalReason
  version : Version

/-- The chain-hash function, modeled abstractly: collision resistance is a
    PROPERTY a theorem about this function would assume as a hypothesis
    (e.g. `CollisionResistant chainH`), not something baked into its type. -/
axiom chainH : Digest → Digest

/-- The chained receipt: advancing from a prior digest `hMinus` by folding in
    a new frame. `chainH (hMinus ++ fr)` in the LaTeX becomes application of
    a two-argument chaining function here, since Digest ++ Frame isn't
    itself a Digest in this abstract model. -/
axiom chainStep : Digest → Frame → Digest

/-- A receipt: the running digest after folding in one frame. -/
structure Receipt where
  hMinus : Digest
  frame : Frame
  hPlus : Digest
  advances : hPlus = chainStep hMinus frame
