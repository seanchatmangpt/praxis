/-
def:replay

The verifier holds a marking m (`enabled_tokens`), initialized to the entry
token, and accumulators `replayed`, `fitted`, `enabled_not_taken`; replaying a
frame rejects an invalid node bit, requires m ≥ m⁻, records enabled-but-
unconsumed tokens, fires m ← (m & ¬m⁻) | m⁺, and sets the node bit in
`replayed` and `fitted`.

Formalized in bare Lean 4 core (no mathlib), reusing `prop:safe`'s `Marking`
machinery specialized to safe (1-bounded) nets: a place's token count is a
`Bool`, embedded into `Nat` by `bit`. `Node` indexes the finite set of nodes
(transitions) a log frame may reference; a frame is valid when its node bit
lies in the net's node set.
-/

/-- A marking assigns a nonnegative integer count of tokens to each of `p`
    places (reused verbatim from `prop:safe`/`def:net`). -/
def Marking (p : Nat) : Type := Fin p → Nat

/-- Embed a bit (a token count on a safe, 1-bounded place) into `Nat`
    (reused verbatim from `prop:safe`). -/
def bit (b : Bool) : Nat := if b then 1 else 0

/-- A safe-net marking: one `Bool` per place (token present / absent). -/
def SafeMarking (p : Nat) : Type := Fin p → Bool

/-- A single replay step's frame: a claimed node together with that node's
    delete-effect (`pre`) and add-effect (`post`) bitmasks on the `p`
    places. -/
structure Frame (p : Nat) (Node : Type) where
  node : Node
  pre  : SafeMarking p
  post : SafeMarking p

/-- The verifier's running state: current marking, and the three
    accumulators `replayed`, `fitted`, `enabled_not_taken`, each a subset of
    places (`SafeMarking p`) folded per node into a record indexed by
    `Node`. -/
structure ReplayState (p : Nat) (Node : Type) where
  enabled_tokens     : SafeMarking p
  replayed           : Node → Bool
  fitted             : Node → Bool
  enabled_not_taken  : SafeMarking p

-- Decidable equality on `Node` is needed to update a single node's bit in
-- `replayed`/`fitted` while leaving all others unchanged.
variable {Node : Type} [DecidableEq Node]

/-- Initialize the verifier at the entry token: `entry` marks the single
    initially-present place, no node has been replayed or fitted yet, and no
    enabled-but-unconsumed tokens have been recorded. -/
def ReplayState.init {p : Nat} (entry : SafeMarking p) : ReplayState p Node :=
  { enabled_tokens := entry
    replayed := fun _ => false
    fitted := fun _ => false
    enabled_not_taken := fun _ => false }

/-- Update a `Node → Bool` function at exactly `n`, setting it to `true`. -/
def setNode (f : Node → Bool) (n : Node) : Node → Bool :=
  fun n' => if n' = n then true else f n'

/-- The enabling test on a safe place: `pre` may only demand a token where
    `m` already holds one (single-place instance of `Net.enabled`, reused
    from `prop:safe`). -/
def enabledAt {p : Nat} (m pre : SafeMarking p) : Prop :=
  ∀ i : Fin p, pre i = true → m i = true

/-- Replaying one frame against the verifier state and the net's finite set
    of valid nodes `validNodes`:
    - rejects if `frame.node` is not a valid node (returns `none`);
    - otherwise requires `enabledAt m frame.pre` (`m ≥ m⁻`, checked
      separately, as `Prop`, by the caller via `enabledAt`);
    - records places that are enabled but not consumed by this frame's
      delete-effect into `enabled_not_taken`;
    - fires the branchless bitset update
      `(m && !pre) || post` per place (single-place instance of
      `safe_fire_eq_bitset`, reused from `prop:safe`);
    - sets `frame.node`'s bit in both `replayed` and `fitted`. -/
def ReplayState.step {p : Nat} (validNodes : Node → Bool)
    (s : ReplayState p Node) (frame : Frame p Node) :
    Option (ReplayState p Node) :=
  if validNodes frame.node = false then
    none
  else
    let m := s.enabled_tokens
    let newEnabledNotTaken : SafeMarking p :=
      fun i => (m i && !(frame.pre i)) || s.enabled_not_taken i
    let newMarking : SafeMarking p :=
      fun i => (m i && !(frame.pre i)) || frame.post i
    some
      { enabled_tokens := newMarking
        replayed := setNode s.replayed frame.node
        fitted := setNode s.fitted frame.node
        enabled_not_taken := newEnabledNotTaken }
