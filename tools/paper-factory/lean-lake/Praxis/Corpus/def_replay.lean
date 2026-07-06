import Praxis.Corpus.def_net

/-!
# def:replay

The verifier holds a marking `m` (`enabled_tokens`) initialized to the entry
token, and accumulators `replayed`, `fitted`, `enabled_not_taken`; replaying a
frame rejects an invalid node bit, requires `m ≥ m⁻_t`, records
enabled-but-unconsumed tokens, fires `m ← (m & ¬m⁻_t) | m⁺_t`, and sets the
node bit in `replayed` and `fitted`.

We reuse `Praxis.Corpus.DefNet.Net`, `Marking`, `Net.enabled`, and `Net.fire`
directly (invariant 6): no new net/marking representation is introduced here,
only the verifier's bookkeeping state on top of it. As in `prop:safe`, the
paper's "bitset" `enabled_tokens` and the branchless update
`(enabled_tokens & ¬m⁻_t) | m⁺_t` are exactly `Net.fire`'s existing
`m i - N.pre t i + N.post t i` on the 1-bounded (safe) coordinates that
`prop:safe` already relates to the bitwise form; we do not re-derive that
identity here, we just invoke `Net.fire`/`Net.enabled` as-is and let the
already-proved `prop:safe` lemmas be the bridge to the bitwise reading
whenever a caller needs it.

An "invalid node bit" (a frame that names no real transition to replay) is
modeled as `none : Option T`; a present transition is `some t`. Replaying a
frame is total and partial-in-effect: it rejects (`none`) both an invalid
node bit and a valid node bit that is not enabled (`¬ N.enabled m t`,
i.e. `m ≥ m⁻_t` fails), matching "requires `m ≥ m⁻_t`" as a rejection
condition rather than an unchecked precondition — no `axiom` is needed to
assert it away.

`enabled_not_taken` accumulates (via Boolean `||`, matching the paper's
accumulator semantics) every transition other than the one just fired that
was enabled at the pre-firing marking, i.e. tokens that were enabled but not
consumed by this frame. Membership uses core's `Bool`/`ite` plus
`Classical.propDecidable` for the (in general undecidable-without-choice,
since `T` and `≤` on `Fin p → ℕ` are only assumed `Fintype`/no `Decidable`
instance is threaded through) predicates `N.enabled m t'` and `t' = t`; this
is standard `Classical` use for a `Prop`-valued definition, not an axiom
standing in for the paper's content.
-/

namespace Praxis.Corpus.DefReplay

open Praxis.Corpus.DefNet
open Classical

universe u

variable {p : ℕ} {T : Type u} [Fintype T]

/-- The verifier's running state while replaying frames against a `Net`:
the current marking `enabled_tokens` (initialized by the caller to the entry
token), and the three accumulators `replayed`, `fitted`, `enabled_not_taken`,
each a `T`-indexed `Bool` (a finite bitset over transitions, matching the
paper's per-node bits). -/
structure ReplayState (p : ℕ) (T : Type u) [Fintype T] where
  /-- the current marking, i.e. the paper's `enabled_tokens` bitset -/
  enabledTokens : Marking p
  /-- nodes whose bit has been set as replayed -/
  replayed : T → Bool
  /-- nodes whose bit has been set as fitted -/
  fitted : T → Bool
  /-- nodes seen enabled at some marking but whose token was not consumed
  by the frame that made that marking current -/
  enabledNotTaken : T → Bool

/-- Replay one frame naming node `frame` against verifier state `s` on net
`N`. An invalid node bit (`none`) is rejected outright. A named node `t`
(`some t`) that is not enabled at the current marking (`¬ N.enabled
s.enabledTokens t`, i.e. `m ≥ m⁻_t` fails) is also rejected. Otherwise: every
other transition enabled at the pre-firing marking is folded into
`enabled_not_taken`, the marking fires (`Net.fire`, i.e. the paper's
`(enabled_tokens & ¬m⁻_t) | m⁺_t`), and `t`'s bit is set in both `replayed`
and `fitted`. -/
noncomputable def Net.replayFrame (N : Net p T) (s : ReplayState p T)
    (frame : Option T) : Option (ReplayState p T) :=
  frame.bind fun t =>
    if N.enabled s.enabledTokens t then
      some
        { enabledTokens := N.fire s.enabledTokens t
          replayed := fun t' => if t' = t then true else s.replayed t'
          fitted := fun t' => if t' = t then true else s.fitted t'
          enabledNotTaken := fun t' =>
            if t' ≠ t ∧ N.enabled s.enabledTokens t' then
              true
            else
              s.enabledNotTaken t' }
    else
      none

/-- Replaying a whole trace: fold `Net.replayFrame` over a list of frames
starting from an initial state (whose `enabledTokens` the caller sets to the
entry token), short-circuiting to `none` as soon as any frame is rejected —
matching the paper's verifier, which halts replay on the first invalid or
unenabled frame. -/
noncomputable def Net.replayTrace (N : Net p T) (s₀ : ReplayState p T) :
    List (Option T) → Option (ReplayState p T)
  | [] => some s₀
  | frame :: rest =>
    match Net.replayFrame N s₀ frame with
    | none => none
    | some s₁ => Net.replayTrace N s₁ rest

end Praxis.Corpus.DefReplay
