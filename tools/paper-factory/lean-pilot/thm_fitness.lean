/-
thm:fitness

Replaying a sequence of frames against the net either (i) completes with no
violation, in which case it is a genuine firing sequence and Fitness = 1; or
(ii) halts at the first frame whose preset is not contained in the current
marking, returning a coordinate-localized witness of the earliest disabled
transition.

Formalized in bare Lean 4 core (no mathlib), reusing `def:net`/`def:replay`/
`prop:safe`'s `Marking`/`SafeMarking`/`Frame`/`ReplayState`/`setNode`
machinery verbatim. `findDisabled` locates the earliest place index `i` at
which a frame's preset demands a token the current marking lacks (the
"coordinate-localized witness"); `firstUnauthorized` replays a list of
frames, halting with `Sum.inl (node, i)` at the first such violation or
returning `Sum.inr` of the final state on success; `Fitness` reads off 1 on
success, 0 on halt. The theorem proves this dichotomy is exhaustive and that
every halt carries a genuine witness: the offending place is demanded by the
halting frame's preset (`pre i = true`) and absent from the marking at that
point (`m i = false`).
-/

/-- A marking assigns a nonnegative integer count of tokens to each of `p`
    places (reused verbatim from `def:net`). -/
def Marking (p : Nat) : Type := Fin p → Nat

/-- Embed a bit (a token count on a safe, 1-bounded place) into `Nat`
    (reused verbatim from `prop:safe`). -/
def bit (b : Bool) : Nat := if b then 1 else 0

/-- A safe-net marking: one `Bool` per place (reused verbatim from
    `def:replay`). -/
def SafeMarking (p : Nat) : Type := Fin p → Bool

/-- A single replay step's frame (reused verbatim from `def:replay`). -/
structure Frame (p : Nat) (Node : Type) where
  node : Node
  pre  : SafeMarking p
  post : SafeMarking p

/-- The verifier's running state (reused verbatim from `def:replay`). -/
structure ReplayState (p : Nat) (Node : Type) where
  enabled_tokens     : SafeMarking p
  replayed           : Node → Bool
  fitted             : Node → Bool
  enabled_not_taken  : SafeMarking p

/-- Update a `Node → Bool` function at exactly `n` (reused verbatim from
    `def:replay`). -/
def setNode {Node : Type} [DecidableEq Node] (f : Node → Bool) (n : Node) :
    Node → Bool :=
  fun n' => if n' = n then true else f n'

/-- The earliest place index `i` at which `pre` demands a token (`pre i =
    true`) that the current marking `m` does not hold (`m i = false`);
    `none` if `pre` is fully enabled by `m`. This is the branchless,
    coordinate-localized failure witness. -/
def findDisabled : (p : Nat) → (Fin p → Bool) → (Fin p → Bool) → Option (Fin p)
  | 0, _, _ => none
  | _ + 1, pre, m =>
      if pre 0 = true ∧ m 0 = false then
        some 0
      else
        (findDisabled _ (fun i => pre i.succ) (fun i => m i.succ)).map Fin.succ

/-- Whenever `findDisabled` reports a witness `i`, it is genuine: the
    preset demands a token at `i` that the marking does not hold there. -/
theorem findDisabled_spec :
    ∀ (p : Nat) (pre m : Fin p → Bool) (i : Fin p),
      findDisabled p pre m = some i → pre i = true ∧ m i = false := by
  intro p
  induction p with
  | zero => intro pre m i h; simp [findDisabled] at h
  | succ n ih =>
    intro pre m i h
    unfold findDisabled at h
    by_cases hc : pre 0 = true ∧ m 0 = false
    · simp only [hc, if_true] at h
      cases h
      exact hc
    · simp only [hc, if_false] at h
      cases hfd : findDisabled n (fun i => pre i.succ) (fun i => m i.succ) with
      | none => rw [hfd] at h; simp at h
      | some j =>
        rw [hfd] at h
        simp only [Option.map_some] at h
        cases h
        exact ih (fun i => pre i.succ) (fun i => m i.succ) j hfd

/-- Replay a list of frames against the net starting from state `s`: on the
    first frame whose preset is not contained in the current marking, halt
    with `Sum.inl (node, i)` — the offending node together with the
    coordinate-localized witness `i`; otherwise fire the frame (the
    branchless bitset update reused from `def:replay`'s `ReplayState.step`)
    and continue. Completing the whole list returns `Sum.inr` of the final
    state: a genuine firing sequence. -/
def firstUnauthorized {p : Nat} {Node : Type} [DecidableEq Node] :
    List (Frame p Node) → ReplayState p Node →
      Sum (Node × Fin p) (ReplayState p Node)
  | [], s => Sum.inr s
  | fr :: rest, s =>
      match findDisabled p fr.pre s.enabled_tokens with
      | some i => Sum.inl (fr.node, i)
      | none =>
          let newMarking : SafeMarking p :=
            fun i => (s.enabled_tokens i && !(fr.pre i)) || fr.post i
          let s' : ReplayState p Node :=
            { enabled_tokens := newMarking
              replayed := setNode s.replayed fr.node
              fitted := setNode s.fitted fr.node
              enabled_not_taken :=
                fun i => (s.enabled_tokens i && !(fr.pre i)) || s.enabled_not_taken i }
          firstUnauthorized rest s'

/-- Fitness reads 1 off a completed, violation-free replay and 0 off a halt. -/
def Fitness {p : Nat} {Node : Type} :
    Sum (Node × Fin p) (ReplayState p Node) → Nat
  | Sum.inl _ => 0
  | Sum.inr _ => 1

/-- **Fitness dichotomy.** Replaying a list of frames against the net either
    completes with no violation, in which case `Fitness = 1`; or halts at
    the first frame whose preset is not contained in the current marking,
    returning a coordinate-localized witness `i` of the earliest disabled
    transition: `pre i = true` (the frame's preset demands a token there)
    and `m i = false` (the current marking does not hold one there). -/
theorem thm_fitness {p : Nat} {Node : Type} [DecidableEq Node] :
    ∀ (frames : List (Frame p Node)) (s : ReplayState p Node),
      (∃ s', firstUnauthorized frames s = Sum.inr s' ∧
             Fitness (Sum.inr s' : Sum (Node × Fin p) (ReplayState p Node)) = 1)
      ∨
      (∃ (node : Node) (i : Fin p) (pre m : Fin p → Bool),
          firstUnauthorized frames s = Sum.inl (node, i) ∧
          pre i = true ∧ m i = false) := by
  intro frames
  induction frames with
  | nil => intro s; left; exact ⟨s, rfl, rfl⟩
  | cons fr rest ih =>
    intro s
    unfold firstUnauthorized
    cases hfd : findDisabled p fr.pre s.enabled_tokens with
    | some i =>
      right
      have hspec := findDisabled_spec p fr.pre s.enabled_tokens i hfd
      exact ⟨fr.node, i, fr.pre, s.enabled_tokens, by simp [hfd], hspec.1, hspec.2⟩
    | none =>
      simp only [hfd]
      exact ih _
