# Concept Identity Report — Praxis/Corpus Namespace Collisions

Classification pass only. No `Praxis/Corpus/*.lean`, `Praxis/Mathlib/*.lean`, or
`Praxis/CorpusExtra.lean` files were edited as part of producing this report. Method used
to derive ground truth is described below; it deviated from the plan's step 1 because a
batch `lake build` with all 77 candidate orphan imports added hit `.lake/build` cache
corruption (stale/missing `.olean` files under `Mathlib/Order/*` and `Praxis/Corpus/*`,
consistent with a concurrent build process touching the same `.lake/build` tree — this repo
has other autonomous sessions active against the same working copy). That batch build did
surface 4 genuine non-collision compile errors (`thm_total`, `thm_bounded_ground`, `thm_mrr`,
`thm_farkas` — unsolved goals / unknown Mathlib constants, unrelated to naming) before the
cache corruption made further error signal unusable.

Given that, ground truth for collisions was instead derived by a **static, namespace-aware
declaration scan** over every `def`/`structure`/`inductive`/`abbrev`/`class`/`axiom` head in
`Praxis/Corpus/*.lean`, `Praxis/Mathlib/*.lean`, and `Praxis/MFW/*.lean`, tracking
`namespace`/`end` nesting to compute each declaration's actual fully-qualified name (so that,
e.g., two `Praxis.Corpus.EstSweep.N`-and-`Agent8.N` declarations are correctly recognized as
*not* colliding), and stripping `/- ... -/` doc-comment blocks first (an unstripped pass
produced several false positives — see "False positives eliminated" below). This is exactly
the signal Lean's elaborator uses to reject a redeclaration, computed without depending on a
healthy build cache.

## True count

**18 distinct fully-qualified names collide across 2 or more files** (not 28). This is lower
than the 28 figure in the task brief. Two things could reconcile the gap without contradicting
either count: (a) the prior agent's 28 may have been counted before namespace-qualification was
applied (my first, non-namespace-aware pass over cross-file dups found 29 raw names, several of
which — `is`, `to`, `of`, `N`, `D` — evaporate under either namespace-qualification or
doc-comment-stripping, see below); (b) some of the 33 "remaining orphaned" files referenced in
the task brief were not part of the 77-file candidate set this scan covered, because the
77-file "true orphan" set was itself derived from a full import-closure trace off
`Praxis.CorpusExtra` + `Praxis.lean`, which may partition the corpus differently than whatever
list the prior agent worked from. I did not have access to that prior agent's exact 33/28 file
list (no artifact from that session was found on disk), so I could not reproduce its count
directly; the 18 below are independently verified by direct declaration-site inspection, which
is the authority the task asks for regardless of the headline number.

### False positives eliminated from the raw scan

- **`is`, `to`, `of`** (6, 2, and 2 raw hits respectively): every one of these was prose inside
  a `/-- ... -/` or `/- ... -/` doc comment matched by a naive `axiom|structure ... \s+NAME`
  regex against sentences like *"no new axiom **is** needed"*, *"the executor-internal
  structure **of** a derived recovery plan"*, *"no new structure **is** needed"*. None are real
  declarations. Confirmed by reading full context at every site
  (`prop_mainclean.lean:19`, `def_gitlock_op.lean:15`, `thm_sep.lean:19`,
  `prop_notesledger_op.lean:17`, `def_polytope.lean:14`, `def_claim.lean:21` for `is`;
  `ax_restartpolicy.lean:35`, `refusal_simpleoneforone.lean:31` for `of`;
  `def_denialcode.lean:15`, `ax_armstrong.lean:18` for `to`).
- **`N`** (`est_sweep.lean:47` vs `prop_footprint.lean:22`): different namespaces
  (`Praxis.Corpus.EstSweep.N` vs `Agent8.N`) — not a collision.
- **`D`** (`def_earned.lean:27` vs `cor_onedenial.lean:33`): different namespaces
  (`Praxis.Corpus.DefEarned.D` vs `OneDenial.D`) — not a collision.
- **`RestartPolicy.new`** initially looked like a second, spurious in-file collision with
  `RestartPolicy` (both in `ax_restartpolicy.lean`) because a dot-unaware regex truncated
  `RestartPolicy.new` to `RestartPolicy`; fixed by capturing dotted identifiers.
- **Local, single-file collisions** (e.g. `Atom`/`Rule`/`Hygiene` all inside one
  `def_logicadm.lean`, `Claim`'s 4 `def`s in one `def_claim.lean`) are not namespace
  collisions at all — Lean allows any number of distinct top-level names in one file; my
  first unfiltered pass conflated "declared more than once in this repo" with "declared more
  than once as the *same* name," which only the cross-file, same-namespace subset actually is.

## The 18 collisions

---

### `Digest`

- `Praxis/Corpus/def_walframe.lean:29` — `abbrev Digest := BitVec 256`
- `Praxis/Corpus/con_merklecell.lean:31` — `abbrev Digest := BitVec 256`
- `Praxis/Corpus/def_fable.lean:46` — `abbrev Digest := BitVec 256`
- `Praxis/Corpus/def_receipt.lean:40` — `abbrev Digest := BitVec 256`
- `Praxis/Corpus/def_contentaddr.lean:33` — `abbrev Digest := BitVec 256`
- `Praxis/Mathlib/DefReceipt.lean:48` — `abbrev Digest := BitVec 256`

**Classification: `SAME_CONCEPT_SAME_MEANING`**

All six sites are the byte-identical abbreviation `BitVec 256`. Every non-`DefReceipt.lean`
site's doc comment explicitly says it is reusing `DefReceipt.lean`'s choice (e.g.
`def_walframe.lean:19`: *"`Digest := BitVec 256` -- reused verbatim from `DefReceipt.lean`
rather than re-declaring a second 256-bit digest type"*; `def_contentaddr.lean`,
`con_merklecell.lean` carry the same language). The intent recorded in every file is explicit
non-duplication; what actually happened is mechanical copy-paste of the one-line `abbrev`
instead of an `import` + reference.

**Resolution law:** merge to canonical. `Praxis/Mathlib/DefReceipt.lean`'s `Digest` is the
canonical declaration (it is the one every other site's own comment cites as the source of
truth). Every other site should `import Praxis.Mathlib.DefReceipt` and drop its local
`abbrev Digest`.

---

### `chainH`

- `Praxis/Corpus/def_walframe.lean:31` — `axiom chainH : String → Digest`
- `Praxis/Corpus/con_merklecell.lean:41` — `axiom chainH : List Digest → Digest`
- `Praxis/Corpus/def_receipt.lean:48` — `axiom chainH : String → Digest`
- `Praxis/Corpus/def_contentaddr.lean:37` — `axiom chainH : ByteArray → Digest`
- `Praxis/Corpus/con_commit.lean:23` — `axiom chainH : Payload → BitVec 256`
- `Praxis/Corpus/con_fablechain.lean:54` — `axiom chainH : String → Digest`
- `Praxis/Mathlib/DefReceipt.lean:69` — `axiom chainH : Digest → Digest`

**Classification: `SAME_CONCEPT_DIFFERENT_REPRESENTATION`**

Every site's doc comment identifies this as the *same* real-world primitive: an axiomatized
BLAKE3-style collision-resistant hash (`con_fablechain.lean:24-25`: *"the *same* kind of
primitive as `Praxis.Mathlib.DefReceipt`'s `chainH`/`chainStep`"*; `con_commit.lean:10`:
*"matching the `chainH` used elsewhere"*). But the argument type genuinely differs by what is
being hashed at that call site — a serialized string, a raw byte array, a payload, a list of
child digests (Merkle aggregation), or (in the canonical `DefReceipt.lean`) a digest-to-digest
step. These are not interchangeable signatures; a `String → Digest` axiom cannot stand in for
a `List Digest → Digest` Merkle-aggregation axiom.

**Resolution law:** canonical object + conversion map. Keep one base axiom
(`Praxis.Mathlib.DefReceipt.chainH : Digest → Digest`, or better, retype it to the most general
useful domain — `ByteArray → Digest`, since every use case reduces to "hash these bytes") and
express every other site's variant as a `def` composed from it plus a concrete, computable
encoding function (the pattern `def_chain.lean` already demonstrates: `bitVecToBytesLE` +
reused `chainH` instead of a fresh axiom — see the `chainStep` entry below). Concretely:
`con_merklecell`'s `List Digest → Digest` becomes `fun ds => chainH (serialize ds)`;
`con_commit`'s `Payload → BitVec 256` becomes `fun p => chainH (serializePayload p)`; etc.
This turns 6 independent axioms (6 unverifiable cryptographic assumptions) into 1 axiom + 5
computable wrappers.

---

### `Frame`

- `Praxis/Corpus/refusal_nomeasuredticks.lean:30` — `abbrev Frame := Nat`
- `Praxis/Corpus/def_receipt.lean:51` — `structure Frame where theta : Theta; dg : Digest`
- `Praxis/Corpus/def_frame.lean:39` — `structure Frame where instruction_id : BitVec 64;
  fired_mask : BitVec 64; denial : BitVec 8; obj_refs : ...; ts_ns : ...; activity_idx : ...;
  node_kind : NodeKind; prior_hash : BitVec 256` (128-byte AST/execution-trace record)
- `Praxis/Mathlib/DefReceipt.lean:59` — `structure Frame where dgX dgG : Digest; denial :
  DenialWord; transition : TransitionId; dgA : Digest; fitness : Fitness; reason :
  RefusalReason; version : Version`

**Classification: `DIFFERENT_CONCEPT_SAME_NAME`** (3-way), with one internal
`SAME_CONCEPT_DIFFERENT_REPRESENTATION` pair inside that

There are three genuinely distinct English senses of "frame" here:

1. `refusal_nomeasuredticks.lean`'s `Frame := Nat` is a **planner tick index** (a cost-accounting
   time step), unrelated to receipts or execution traces — its own doc comment
   (`refusal_nomeasuredticks.lean:12-20`) frames it purely as the domain of `declaredTicks : Frame
   → Ticks`.
2. `def_frame.lean`'s `Frame` is the **execution/AST-trace record** (`def:frame`'s actual
   subject): a fixed 128-byte binary layout of `instruction_id`/`fired_mask`/`obj_refs`/
   `node_kind`/etc. — a hardware/wire-format concept, nothing to do with receipt chaining.
3. `def_receipt.lean`'s `Frame` (`{theta, dg}`) and `DefReceipt.lean`'s `Frame`
   (`{dgX, dgG, denial, transition, dgA, fitness, reason, version}`) are both the
   **receipt-envelope frame** from `def:receipt` — the thing `chainStep` folds over — but at two
   different levels of elaboration of the same spec: `def_receipt.lean`'s is a minimal 2-field
   placeholder (`theta`, one digest), `DefReceipt.lean`'s is the fully-elaborated 8-field
   version matching the "six opaque fields, four composed" account in its own module doc. This
   pair is `SAME_CONCEPT_DIFFERENT_REPRESENTATION` (same role in the receipt chain, different
   granularity — see also the `Receipt` entry, which has the same v1/v2 relationship).

**Resolution law:** distinct qualified names reflecting actual role, for the 3-way split:
- `refusal_nomeasuredticks.lean`'s tick-index sense → rename to `TickFrame` or fold into
  `Ticks`-adjacent naming (it is already just `Nat`; the name `Frame` here is arguably
  unnecessary).
- `def_frame.lean`'s execution-trace record → rename to `TraceFrame` or `ExecFrame` (it is the
  literal subject of `def:frame`, so this is the strongest claim to keep the bare name if only
  one candidate must keep it).
- `def_receipt.lean` / `DefReceipt.lean`'s receipt-envelope sense → resolve per the
  `SAME_CONCEPT_DIFFERENT_REPRESENTATION` law: canonicalize on `DefReceipt.lean`'s 8-field
  `Frame` (or `ReceiptFrame` if `Frame` is reserved for #2 above) with a `theta`/`dg`-shaped
  smart constructor for callers that only need `def_receipt.lean`'s minimal view.

---

### `Fitness`

- `Praxis/Corpus/def_receipt.lean:44` — `abbrev Fitness := Nat`
- `Praxis/Mathlib/DefReceipt.lean:57` — `abbrev Fitness := Nat`

**Classification: `SAME_CONCEPT_SAME_MEANING`**

Identical abbreviation, identical role (a replay/fitness score in the receipt tuple), and
`def_receipt.lean:25` explicitly says *"a fitness score ... (matches `DefReceipt.lean`)"*.
(Note: `def_seqtoken.lean` also declares a `Fitness` — a `structure` with a Q16.16 fixed-point
encoding — but it lives inside `namespace Praxis.Corpus` alongside a `Marking`, and its
fully-qualified name is `Praxis.Corpus.Fitness`... **correction during verification**: it does
not collide with the two `Fitness` above because those two are unnamespaced top-level `Fitness`
in their files' global scope, while `def_seqtoken.lean`'s is `Praxis.Corpus.Fitness` — the scan
above already excludes it from this pairing for exactly that reason. `thm_fitness.lean`'s
`def Fitness` is also namespace-isolated. Flagging here only so the follow-up agent doesn't
assume the two 3rd-party `Fitness` hits it will find via grep are part of this same collision —
they are not, per the namespace-aware scan.)

**Resolution law:** merge to canonical. `DefReceipt.lean`'s `Fitness` is canonical (cited by
name in `def_receipt.lean`'s own comment); `def_receipt.lean` should import it instead of
re-declaring.

---

### `Obs`

- `Praxis/Corpus/ax_obs.lean:20` — `axiom Obs : Type`
- `Praxis/Corpus/ax_obsauth.lean:30` — `axiom Obs : Type`
- `Praxis/Mathlib/ObsSimEquivalence.lean:38` — `axiom Obs : Type`

**Classification: `SAME_CONCEPT_SAME_MEANING`**

All three are the identical opaque "raw observation space" primitive. `ax_obsauth.lean:6-8`
explicitly ties its `Obs` to the same abstraction reason as `Adm` in `ax_refusal.lean`, and
`ax_obs.lean:16` explicitly says its own justification *"mirrors the justification style of
`ObsSimEquivalence.lean`'s abstract observation space."* `ObsSimEquivalence.lean`'s own header
(lines 1-2) describes itself as the *"Shared `Obs`/`Sim` layer (used by `thm:rice`, `def:adm`,
`def:mu`)"* — i.e. it is explicitly designed to be the one shared declaration these other files
should import, not re-declare.

**Resolution law:** merge to canonical. `Praxis/Mathlib/ObsSimEquivalence.lean`'s `Obs` is
canonical per its own stated role as the shared layer; `ax_obs.lean` and `ax_obsauth.lean`
should import it and drop their local axioms.

---

### `Verify`

- `Praxis/Corpus/ax_verify.lean:37` — `axiom Verify : BitVec 512 → String → BitVec 256 → Bool`
- `Praxis/Corpus/prop_intauth.lean:37` — `axiom Verify : BitVec 512 → String → BitVec 256 → Bool`

**Classification: `LEGACY_ALIAS_COLLISION`**

Byte-identical signature and doc comment. `ax_verify.lean:4-5,25-28` states outright: *"key
sensitivity of `Verify` ... split out of `prop_intauth.lean`"* and *"`prop_intauth.lean` imports
this file and references `verify_key_sensitive` by name rather than declaring it"* — i.e.
`ax_verify.lean` is a deliberate, already-documented refactor extracting `Verify` and
`verify_key_sensitive` out of `prop_intauth.lean` into the project's `ax_*.lean`
axiom-disclosure convention (see `AXIOM_ALLOWLIST.md`'s note that `ax_*.lean` files are "the
designated home for axioms"). `prop_intauth.lean` simply has not yet had its local copy deleted
and the corresponding `import` added — this is an in-flight, partially-applied refactor, not
two independent authors converging on the same idea.

**Resolution law:** deprecate the older one, apply the already-documented compatibility path.
Delete `Verify`/`verify_key_sensitive` from `prop_intauth.lean` and add
`import Praxis.Corpus.ax_verify` — exactly what `ax_verify.lean`'s own comment already
prescribes.

---

### `verify_key_sensitive`

- `Praxis/Corpus/ax_verify.lean:45` — same existential statement
- `Praxis/Corpus/prop_intauth.lean:43` — same existential statement

**Classification: `LEGACY_ALIAS_COLLISION`** — identical situation and resolution as `Verify`
above (same two files, same in-flight split-out refactor, same fix).

---

### `RestartPolicy`

- `Praxis/Corpus/ax_restartpolicy.lean:37` — `axiom RestartPolicy : Type`
- `Praxis/Corpus/refusal_simpleoneforone.lean:33` — `axiom RestartPolicy : Type`

### `RestartPolicy.new`

- `Praxis/Corpus/ax_restartpolicy.lean:46` — `axiom RestartPolicy.new (intensity : Nat) (aux :
  Type) (a : aux) : Except RefusalReason RestartPolicy`
- `Praxis/Corpus/refusal_simpleoneforone.lean:42` — identical signature

### `no_simple_one_for_one`

- `Praxis/Corpus/ax_restartpolicy.lean:57` — same statement
- `Praxis/Corpus/refusal_simpleoneforone.lean:50` — same statement

**Classification (all three): `LEGACY_ALIAS_COLLISION`**

Same pattern as `Verify`/`verify_key_sensitive`: `refusal_simpleoneforone.lean` is the original
`refusal_*.lean` file with the axioms declared inline; `ax_restartpolicy.lean` is the
`ax_*.lean`-convention split-out home, containing a verbatim copy of `RestartPolicy`,
`RefusalReason` (see separately below), `RestartPolicy.new`, and `no_simple_one_for_one`, with
`refusal_simpleoneforone.lean:17-18` explicitly noting `RefusalReason := String, matching the
same composition already used in Praxis/Mathlib/DefReceipt.lean` — i.e. it already knows about
and defers to the canonical source, but the file itself hasn't been converted to an `import`.

**Resolution law:** deprecate `refusal_simpleoneforone.lean`'s inline copies; have it
`import Praxis.Corpus.ax_restartpolicy` and reference the axioms by name, matching the
`ax_verify.lean` / `prop_intauth.lean` precedent above.

---

### `RefusalReason`

- `Praxis/Corpus/ax_restartpolicy.lean:41` — `abbrev RefusalReason : Type := String`
- `Praxis/Corpus/refusal_simpleoneforone.lean:37` — `abbrev RefusalReason : Type := String`
- `Praxis/Mathlib/DefReceipt.lean:56` — `abbrev RefusalReason := String`

**Classification: `SAME_CONCEPT_SAME_MEANING`**

Identical one-line abbreviation; both non-canonical sites' doc comments explicitly cite
`DefReceipt.lean`'s `RefusalReason := String` by name as "the identical concept"
(`refusal_simpleoneforone.lean:17-18`) or "the same composition"
(`ax_restartpolicy.lean:39-40`).

**Resolution law:** merge to canonical. `DefReceipt.lean`'s `RefusalReason` is canonical (cited
by both other sites); `ax_restartpolicy.lean` and `refusal_simpleoneforone.lean` should import
it. Note this folds into, and should be resolved together with, the `RestartPolicy` /
`Verify` legacy-alias cleanups above since they are the same two source files.

---

### `Receipt`

- `Praxis/Corpus/def_receipt.lean:66` — `structure Receipt where verdict : Verdict; hPlus :
  Digest; fitness : Fitness; reason : Reason` (4 fields)
- `Praxis/Mathlib/DefReceipt.lean:72` — `structure Receipt where hMinus : Digest; frame :
  Frame; hPlus : Digest; advances : hPlus = chainStep hMinus frame` (chain-envelope + proof
  obligation)

**Classification: `SAME_CONCEPT_DIFFERENT_REPRESENTATION`**

Both are formalizations of the same source quote ("def:receipt": *"the receipt is the `<= CL`-
chunk tuple (verdict, h+, Fitness, reason)"* per `def_receipt.lean:8-9`), and `def_receipt.lean`
titles itself identically to `DefReceipt.lean` ("def:receipt[, reformalized in the Mathlib
lane]"). `def_receipt.lean`'s version is a flat 4-field "bare-core" tuple matching the literal
quote; `DefReceipt.lean`'s version elaborates the receipt as a chain-step envelope
(`hMinus`/`frame`/`hPlus`/proof-carrying `advances` field) consistent with its own `Frame`'s
8-field expansion (digests X/G/A, denial, transition, fitness, reason, version — a strict
superset of the info in `def_receipt.lean`'s `theta`/`dg` `Frame`). This is the same v1
(minimal, literal-quote) vs. v2 (elaborated, chain-integrated) relationship documented for
`Frame` above — not a coincidence, since `Receipt` is built directly from `Frame` in both
files.

**Resolution law:** canonical object + conversion map. Treat `DefReceipt.lean`'s `Receipt` as
canonical (it is chain-integrated and proof-carrying, a strictly stronger formalization);
express `def_receipt.lean`'s 4-tuple as a projection/view function
`toBareReceipt : DefReceipt.Receipt → def_receipt.Receipt` if the bare-core view is still
needed downstream, otherwise delete `def_receipt.lean`'s `Receipt` and repoint its callers.

---

### `chainStep`

- `Praxis/Corpus/def_chain.lean:38` — `noncomputable def chainStep (hMinus : Digest) (fr :
  Frame) : Digest := chainH (bitVecToBytesLE 32 hMinus ++ bitVecToBytesLE 99 (body fr))`
- `Praxis/Mathlib/DefReceipt.lean:70` — `axiom chainStep : Digest → Frame → Digest`

**Classification: `SAME_CONCEPT_DIFFERENT_REPRESENTATION`**

`def_chain.lean`'s own doc comment (lines 5-23) states this is `def:chain`, "reformalized in the
Mathlib lane," explicitly built by composing `def_body.lean`'s `Frame`/`body`,
`def_contentaddr.lean`'s `chainH`, and a new concrete `bitVecToBytesLE` helper — i.e. it is a
deliberate *concrete* derivation of the same chaining step `DefReceipt.lean` leaves as an
opaque `axiom`. Both compute "the next commitment from the previous one plus a frame"; one is
axiomatized (abstract top-level receipt model), the other is fully worked out (concrete corpus
derivation one level down). Note this also depends on which `Frame` is in scope (`def_chain.lean`
imports `def_body.lean`'s `Frame`, not `DefReceipt.lean`'s) — this collision is entangled with
the `Frame` collision above and cannot be resolved independently of it.

**Resolution law:** canonical object + conversion map, sequenced after the `Frame` resolution.
Once `Frame` has a settled canonical shape, either (a) prove `DefReceipt.lean`'s `axiom
chainStep` is discharged by `def_chain.lean`'s concrete `chainStep` (turning the axiom into a
theorem — the strongest outcome), or (b) if the two `Frame`s remain genuinely distinct
representations, keep both `chainStep`s but qualify them (`DefReceipt.chainStep` /
`ChainCorpus.chainStep`) and add an explicit compatibility lemma relating them if one is ever
meant to refine the other.

---

### `POWL`

- `Praxis/Corpus/def_powl.lean:31` — `inductive POWL (A : Type u) : Type u` — 3 constructors
  (`activity : A → POWL A`, `partialOrder : List (POWL A) → (Nat → Nat → Prop) → POWL A`, and a
  third `choiceGraph` constructor referenced by `prop_local.lean`'s pattern match), polymorphic
  over an activity alphabet `A`, with an explicit precedence relation `prec` on child positions.
- `Praxis/Corpus/thm_sep.lean:38` — `inductive POWL` — 2 constructors (`choice : List POWL →
  POWL`, `order : List POWL → POWL`), monomorphic (no alphabet parameter), no precedence
  relation, `deriving Inhabited`.

**Classification: `SAME_CONCEPT_DIFFERENT_REPRESENTATION`**

Both explicitly claim to formalize `def:powl`/POWL 2.0 (`thm_sep.lean:5-6`: *"a recursive
decomposition into a POWL~2.0 model whose every node is a choice graph ... or a partial order
..."*), but `thm_sep.lean` deliberately does **not** import `def_powl.lean` — its own comment
(lines 27-29) says *"No axioms: `POWL` is a plain inductive type over `List`, matching the style
of `def:adm`'s plain data-level composition"* with no reference to reusing `def_powl.lean`'s
existing type, even though one already exists. `thm_sep.lean`'s version is a self-contained
simplification (drops the activity-alphabet polymorphism and the `prec` relation, since
`thm:sep`'s statement doesn't need them) built independently for local proof convenience rather
than composed from the canonical definition.

**Resolution law:** canonical object + conversion map. `def_powl.lean`'s `POWL A` is canonical
(it is literally `def:powl`, the ticket this type is named for). `thm_sep.lean` should either
(a) import `def_powl.lean` and instantiate `A` with whatever concrete alphabet `thm:sep` needs,
recovering `choice`/`order` as the `partialOrder`/`choiceGraph` constructors with a trivial
`prec`, or (b) if the simplification is load-bearing for the proof, rename its local type to
`SepPOWL` and add a forgetful map `def_powl.POWL A → SepPOWL` documenting the relationship.

---

### `POWL.arity`

- `Praxis/Corpus/thm_sep.lean:45` — `def POWL.arity : POWL → Nat` (member of `thm_sep.lean`'s
  own local 2-constructor `POWL`)
- `Praxis/Corpus/prop_local.lean:33` — `def arity {A : Type u} : POWL A → Nat` inside
  `namespace POWL` (member of `def_powl.lean`'s imported 3-constructor `POWL A`)

**Classification: `SAME_CONCEPT_DIFFERENT_REPRESENTATION`** (downstream consequence of the
`POWL` collision above)

Same role in both — "number of immediate children" — but necessarily typed against two
different `POWL` types, so this collision cannot be fixed independently; it is entirely a
byproduct of the `POWL` type-level split. `prop_local.lean` correctly imports `def_powl.lean`
and extends its namespace (the intended pattern); `thm_sep.lean` re-derives the same member
name on its own parallel type.

**Resolution law:** resolves automatically once `POWL` above is unified — if `thm_sep.lean`
switches to importing `def_powl.lean`'s `POWL A`, its local `POWL.arity` becomes redundant with
`prop_local.lean`'s and should be deleted in favor of it.

---

### `Marking`

- `Praxis/Corpus/def_reachcone.lean:33` — `abbrev Marking := Fin p → ℤ` (inside `namespace
  Praxis.Corpus`, parametrized over `p : ℕ`; integer-valued Petri-net marking vector, general
  reachability-cone setting)
- `Praxis/Corpus/def_seqtoken.lean:31` — `def Marking : Type := LifeObj → Bool` (inside
  `namespace Praxis.Corpus`; boolean-valued marking specific to the 4-place safe net over
  `LifeObj`'s `raw`/`val`/`admd`/`rcpt` places)

**Classification: `SAME_CONCEPT_DIFFERENT_REPRESENTATION`**

Both are "a Petri-net marking" (an assignment of token state to places), which is the reason
they collide under the shared `Praxis.Corpus` namespace — but they model different net classes:
`def_reachcone.lean`'s is a general integer-vector marking for an arbitrary `p`-place net
(reachability-cone algebra, needs `ℤ`-valued counts for the cone construction);
`def_seqtoken.lean`'s is a safe-net (1-bounded) marking specialized to exactly 4 named places
reusing `LifeObj`, where `Bool` suffices because no place ever holds more than one token. Neither
doc comment references the other; this looks like independent formalization of two different
Petri-net-shaped source concepts that happen to share the generic name "Marking," not a
copy-paste duplicate.

**Resolution law:** canonical object + conversion map. Keep `def_reachcone.lean`'s `Marking`
(the general `Fin p → ℤ` vector) as the canonical net-marking representation, since a safe net
is the special case `∀ i, marking i ∈ {0, 1}`. Rename `def_seqtoken.lean`'s to
`SafeMarking := LifeObj → Bool` and, if useful, add a coercion
`SafeMarking → Marking (Fintype.card LifeObj)` via `Bool.toNat`, rather than leaving both
anchored to the same bare name.

---

### `Praxis.Corpus.RefCurve.NoveltyCurveUnderFaultsWithheld` and
### `Praxis.Corpus.RefCurve.noveltyCurveUnderFaultsWithheld`

- `Praxis/Corpus/ref_curve.lean:27,32` — `axiom NoveltyCurveUnderFaultsWithheld : Prop` /
  `axiom noveltyCurveUnderFaultsWithheld : NoveltyCurveUnderFaultsWithheld`, inside
  `namespace Praxis.Corpus.RefCurve`
- `Praxis/Corpus/ax_curve.lean:36,45` — identical two declarations, same namespace

**Classification: `LEGACY_ALIAS_COLLISION`**

The two files are near-verbatim duplicates of each other's prose and declarations.
`ax_curve.lean`'s only substantive addition over `ref_curve.lean` is an explicit `DISCLOSURE:
this file declares an unproven, empirical placeholder axiom` header and slightly expanded
"UNPROVEN, by design" doc comments — i.e. `ax_curve.lean` is `ref_curve.lean` re-homed into the
`ax_*.lean` axiom-disclosure convention (the same convention documented in
`AXIOM_ALLOWLIST.md` and already seen driving the `ax_verify.lean`/`ax_restartpolicy.lean`
splits above). `ref_curve.lean` (the `ref_*.lean`-named file) is the pre-convention original;
`ax_curve.lean` is its designated replacement.

**Resolution law:** deprecate the older one, propose a compatibility path. Delete both axioms
from `ref_curve.lean` and have it `import Praxis.Corpus.ax_curve`, re-exporting the
`Praxis.Corpus.RefCurve` namespace's two names from the canonical `ax_curve.lean` — matching the
`ax_verify.lean` precedent exactly.

---

## Summary table

| Name | Sites | Classification |
|---|---|---|
| `Digest` | 6 | SAME_CONCEPT_SAME_MEANING |
| `chainH` | 7 | SAME_CONCEPT_DIFFERENT_REPRESENTATION |
| `Frame` | 4 (3-way concept split) | DIFFERENT_CONCEPT_SAME_NAME (+ 1 internal SAME_CONCEPT_DIFFERENT_REPRESENTATION pair) |
| `Fitness` | 2 | SAME_CONCEPT_SAME_MEANING |
| `Obs` | 3 | SAME_CONCEPT_SAME_MEANING |
| `Verify` | 2 | LEGACY_ALIAS_COLLISION |
| `verify_key_sensitive` | 2 | LEGACY_ALIAS_COLLISION |
| `RestartPolicy` | 2 | LEGACY_ALIAS_COLLISION |
| `RestartPolicy.new` | 2 | LEGACY_ALIAS_COLLISION |
| `no_simple_one_for_one` | 2 | LEGACY_ALIAS_COLLISION |
| `RefusalReason` | 3 | SAME_CONCEPT_SAME_MEANING |
| `Receipt` | 2 | SAME_CONCEPT_DIFFERENT_REPRESENTATION |
| `chainStep` | 2 | SAME_CONCEPT_DIFFERENT_REPRESENTATION |
| `POWL` | 2 | SAME_CONCEPT_DIFFERENT_REPRESENTATION |
| `POWL.arity` | 2 | SAME_CONCEPT_DIFFERENT_REPRESENTATION (downstream of `POWL`) |
| `Marking` | 2 | SAME_CONCEPT_DIFFERENT_REPRESENTATION |
| `Praxis.Corpus.RefCurve.NoveltyCurveUnderFaultsWithheld` | 2 | LEGACY_ALIAS_COLLISION |
| `Praxis.Corpus.RefCurve.noveltyCurveUnderFaultsWithheld` | 2 | LEGACY_ALIAS_COLLISION |

18 distinct names, 0 `UNKNOWN_SEMANTIC_IDENTITY`, 0 `GENERATED_NAME_COLLISION` (no evidence any
of these came from a ggen/packs template — every site carries a hand-written, individually
reasoned doc comment, not template boilerplate; `packs/` was not searched further since no
`.tmpl` naming pattern matches any of the 18 colliding identifiers).

## Notes for the follow-up (resolution) agent

- Six of the 18 names (`Verify`, `verify_key_sensitive`, `RestartPolicy`, `RestartPolicy.new`,
  `no_simple_one_for_one`, and both `NoveltyCurveUnderFaultsWithheld` variants) share one
  mechanical fix: delete the duplicate declarations from the non-`ax_*.lean` file and add an
  `import` of the corresponding `ax_*.lean` file. This resolves 3 of the 18 collision *groups*
  in one mechanical pass (`ax_verify.lean`↔`prop_intauth.lean`,
  `ax_restartpolicy.lean`↔`refusal_simpleoneforone.lean`, `ax_curve.lean`↔`ref_curve.lean`).
- `Digest`, `Fitness`, `Obs`, `RefusalReason` are a second mechanical batch: each has one
  canonical site already self-identified by every other site's doc comment; fix is
  delete-and-import, no design decision required.
- `Frame` is the one genuinely hard case: it needs a 3-way rename decision (tick-index vs.
  execution-trace vs. receipt-envelope) before anything downstream of it
  (`Receipt`, `chainStep`, and transitively anything importing `def_frame.lean`/
  `DefReceipt.lean`) can be resolved. Do this one first; `Receipt` and `chainStep`'s
  resolutions are gated on its outcome.
- `POWL`/`POWL.arity` and `Marking` need a maintainer decision about whether the
  parallel/independent formalizations are intentional simplifications worth keeping distinct
  (rename) or accidental duplication worth collapsing (merge) — I found supporting evidence for
  "intentional simplification for a specific proof's needs" in both cases, but did not find an
  explicit statement ruling out the alternative, so treat my SAME_CONCEPT_DIFFERENT_REPRESENTATION
  call as a recommendation, not a certainty on a par with the LEGACY_ALIAS_COLLISION calls above
  (which are backed by literal doc-comment cross-references admitting the split).
