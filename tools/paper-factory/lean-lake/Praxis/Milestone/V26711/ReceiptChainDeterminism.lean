/-!
# PROJ-769 / PRD v26.7.11 §15 — Receipt-Chain Head Determinism

Target 9 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"receipt-chain head determinism over an ordered event corpus."

PRD §15 (`docs/jira/v26.7.11/PRD.md:690-726`), in relevant part:

> Every workflow execution SHALL extend a BLAKE3-linked receipt chain.
> ... prior receipt head ...
> Replay SHALL: (1) resolve the AIR artifact by digest; (2) restore the admitted
> initial state; (3) apply the admitted ordered event corpus; (4) recompute state
> and command digests; (5) verify receipt-head equivalence.
> Replay mismatch SHALL be a typed refusal or build failure; it SHALL NOT be
> logged and ignored.

## Formalization strategy

The receipt chain is folded over an *ordered* event corpus (`Praxis/Corpus/
con_commit.lean`/`def_receipt.lean` model the same "chain a digest forward through
a frame" shape via an axiomatized `chainH`; this file does not repeat that axiom —
see the note below). `chainHeadRec genesis events` folds a caller-supplied `extend`
step function left-to-right through `events`, starting from `genesis`.

The content-bearing theorem is `chainHeadRec_append`: the chain head over `p ++ s`
is exactly the chain head over `s` *starting from* the chain head over `p` alone.
This is precisely what makes replay (steps 2-5 above) sound: replaying the admitted
prefix `p` up to receipt head `R_n` and then continuing with the remainder `s`
reproduces the same head as processing the whole corpus `p ++ s` in one pass —
determinism is not "the same computation run twice gives the same answer" (true of
any pure function and not worth a theorem on its own), it is this
compositional/prefix property, which is what "crash after `R_n`, replay through
`R_n`, continue" (acceptance scenario 19.6) actually depends on.
`chainHeadRec_deterministic` and `crash_replay_continue` below state the
determinism and replay-specific corollaries by name.

No new axioms: `extend` is a universally quantified function parameter (standing in
for the real BLAKE3-based chain step,
`wasm4pm_compat::hash::blake3_combined`/`crates/praxis-graphlaw`'s receipt paths),
not an assumed-to-exist global axiom — deliberately different from
`Praxis/Corpus/con_commit.lean`/`def_receipt.lean`'s `axiom chainH`, chosen
specifically to keep this ticket's new-axiom count at zero against
`crates/praxis-lean/tests/rail_h_existing_corpus_audit.rs`'s
`KNOWN_UNAUDITED_AXIOM_COUNT` regression guard (which scans this whole `Praxis/`
directory tree, including this file's own directory). The theorems below hold for
*any* concrete `extend` a real BLAKE3 chain step could denote, which is a strictly
more general (and equally sound) statement than fixing one axiomatized `chainH`.
-/

variable {Digest Event : Type}

/-- Folds `extend` left-to-right through `events`, starting at `genesis` — the
receipt-chain head after processing an ordered event corpus. -/
def chainHeadRec (extend : Digest → Event → Digest) (genesis : Digest) :
    List Event → Digest
  | [] => genesis
  | e :: es => chainHeadRec extend (extend genesis e) es

/-- `thm:receipt_chain_prefix_composition`: the chain head over `p ++ s` equals the
chain head over `s`, started from the chain head over `p` alone. This is the real
content of "receipt-chain head determinism over an ordered event corpus": replaying
a prefix and continuing reproduces exactly the same head as one full pass. -/
theorem chainHeadRec_append (extend : Digest → Event → Digest) (genesis : Digest)
    (p s : List Event) :
    chainHeadRec extend genesis (p ++ s) =
      chainHeadRec extend (chainHeadRec extend genesis p) s := by
  induction p generalizing genesis with
  | nil => simp [chainHeadRec]
  | cons e es ih =>
      simp only [List.cons_append, chainHeadRec]
      exact ih (extend genesis e)

/-- `thm:receipt_chain_determinism`: two computations of the chain head over the
*same* genesis and the *same* ordered event corpus are identical. Named explicitly
because acceptance scenario 19.6 depends on exactly this fact holding for replay's
own recomputation, not merely on `chainHeadRec` happening to be a function. -/
theorem chainHeadRec_deterministic (extend : Digest → Event → Digest)
    (genesis : Digest) (executed replayed : List Event) (h : executed = replayed) :
    chainHeadRec extend genesis executed = chainHeadRec extend genesis replayed := by
  rw [h]

/-- `crash_replay_continue` (acceptance scenario 19.6): given a crash after
processing prefix `p` (reaching head `chainHeadRec extend genesis p`), replaying
`p` and continuing with the remaining events `s` reproduces exactly the same head as
an uninterrupted run over `p ++ s` — "the supervisor SHALL restore execution
machinery, replay admitted events through `R_n`, reproduce the state digest, and
continue without changing workflow semantic identity." -/
theorem crash_replay_continue (extend : Digest → Event → Digest) (genesis : Digest)
    (p s : List Event) (replayedPrefixHead : Digest)
    (hreplay : replayedPrefixHead = chainHeadRec extend genesis p) :
    chainHeadRec extend genesis (p ++ s) = chainHeadRec extend replayedPrefixHead s := by
  rw [hreplay]
  exact chainHeadRec_append extend genesis p s
