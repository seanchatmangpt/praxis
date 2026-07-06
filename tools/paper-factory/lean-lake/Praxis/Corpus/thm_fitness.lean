import Praxis.Corpus.def_net
import Praxis.Corpus.def_replay
import Praxis.Corpus.prop_safe

/-!
# thm:fitness

"Replaying a sequence of frames against the net either (i) completes with no
violation, in which case it is a genuine firing sequence and `Fitness = 1`; or
(ii) halts at the first frame whose preset is not contained in the current
marking, returning a coordinate-localized witness of the earliest disabled
transition."

We reuse `Praxis.Corpus.DefNet.Net`/`Marking`/`Net.enabled` and
`Praxis.Corpus.DefReplay.ReplayState`/`Net.replayFrame`/`Net.replayTrace`
directly (invariant 6): no new net or replay representation is introduced,
only a `Fitness` reading of the trace's `Option` result and a dichotomy
theorem about `Net.replayTrace`.

`Fitness` is the paper's `0/1` verdict: `1` exactly when the whole trace
replays without rejection (`Net.replayTrace` returns `some _`), `0` otherwise.
No axiom is introduced for it: it is a two-line definition on top of
`Option.isSome`, which core/Mathlib already provides.

The theorem itself is a genuine proof obligation, done by list induction on
the frame sequence, mirroring `Net.replayTrace`'s own recursion: either the
whole trace succeeds (`Fitness = 1`), or there is a *first* frame at which
`Net.replayFrame` returns `none`, and by unfolding `Net.replayFrame` that
first failure is either an invalid node bit (`frame = none`) or a coordinate
`i` at which the named transition's preset exceeds the current marking
(`¬ N.pre t i ≤ s.enabledTokens i`), i.e. the "coordinate-localized witness of
the earliest disabled transition" from the paper. `not_forall` (classical,
since the `Pi` order on `Fin p → ℕ` need not be decidable for arbitrary `T`)
turns the negation of `N.enabled` (itself `∀ i, N.pre t i ≤ m i`, `Pi.le_def`)
into that witness coordinate — no axiom stands in for this step.
-/

namespace Praxis.Corpus.ThmFitness

open Praxis.Corpus.DefNet
open Praxis.Corpus.DefReplay

universe u

variable {p : ℕ} {T : Type u} [Fintype T]

/-- The paper's `0/1` fitness verdict on a (possibly rejected) replay result:
`1` iff the trace replayed with no violation. -/
def Fitness (r : Option (ReplayState p T)) : ℕ :=
  if r.isSome then 1 else 0

@[simp] theorem fitness_some (s : ReplayState p T) : Fitness (some s) = 1 := rfl

@[simp] theorem fitness_none : Fitness (p := p) (T := T) none = 0 := rfl

/-- Main dichotomy: replaying `frames` from `s₀` against `N` either (i)
completes, giving a state `s` with `Fitness = 1`; or (ii) there is a first
frame (`frames = pre ++ frame :: rest`) at which the prefix `pre` has
successfully replayed to some state `s`, the whole trace is rejected
(`Fitness = 0`), and `frame` is itself the witness of the violation: either
an invalid node bit, or a transition `t` with a coordinate `i` at which `t`'s
preset is not contained in `s`'s marking. -/
theorem replay_dichotomy (N : Net p T) :
    ∀ (s₀ : ReplayState p T) (frames : List (Option T)),
      (∃ s, Net.replayTrace N s₀ frames = some s ∧ Fitness (some s) = 1) ∨
      ∃ (pre rest : List (Option T)) (frame : Option T) (s : ReplayState p T),
        frames = pre ++ frame :: rest ∧
        Net.replayTrace N s₀ pre = some s ∧
        Net.replayTrace N s₀ frames = none ∧
        Fitness (Net.replayTrace N s₀ frames) = 0 ∧
        (frame = none ∨
          ∃ t i, frame = some t ∧ ¬ N.pre t i ≤ s.enabledTokens i) := by
  intro s₀ frames
  induction frames generalizing s₀ with
  | nil => exact Or.inl ⟨s₀, rfl, rfl⟩
  | cons frame rest ih =>
    rcases hrf : Net.replayFrame N s₀ frame with _ | s1
    · -- the very first frame already fails: `pre = []`, witness is `frame`
      refine Or.inr ⟨[], rest, frame, s₀, rfl, rfl, ?_, ?_, ?_⟩
      · show Net.replayTrace N s₀ (frame :: rest) = none
        unfold Net.replayTrace
        rw [hrf]
      · show Fitness (Net.replayTrace N s₀ (frame :: rest)) = 0
        unfold Net.replayTrace
        rw [hrf]
        rfl
      · cases frame with
        | none => exact Or.inl rfl
        | some t =>
          right
          unfold Net.replayFrame at hrf
          simp only [Option.bind_some] at hrf
          by_cases hen : N.enabled s₀.enabledTokens t
          · simp [hen] at hrf
          · unfold Net.enabled at hen
            rw [Pi.le_def] at hen
            obtain ⟨i, hi⟩ := not_forall.mp hen
            exact ⟨t, i, rfl, hi⟩
    · rcases ih s1 with ⟨s, hs, hfit⟩ | ⟨pre', rest', frame', s', heq, hpre', hnone', hfit', hwit⟩
      · refine Or.inl ⟨s, ?_, hfit⟩
        show Net.replayTrace N s₀ (frame :: rest) = some s
        unfold Net.replayTrace
        rw [hrf]
        exact hs
      · refine Or.inr ⟨frame :: pre', rest', frame', s', ?_, ?_, ?_, ?_, hwit⟩
        · rw [heq]; rfl
        · show Net.replayTrace N s₀ (frame :: pre') = some s'
          unfold Net.replayTrace
          rw [hrf]
          exact hpre'
        · show Net.replayTrace N s₀ (frame :: rest) = none
          unfold Net.replayTrace
          rw [hrf]
          exact hnone'
        · show Fitness (Net.replayTrace N s₀ (frame :: rest)) = 0
          unfold Net.replayTrace
          rw [hrf]
          exact hfit'


end Praxis.Corpus.ThmFitness
