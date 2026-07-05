/-
prop:conserve — If every action drawing on φ both decrements it by r_j at
start and increments it by r_j at end, the net effect of each completed
action on φ is zero, and the terminal valuation equals the initial:
ν_end(φ) = ν0(φ); scheduling redistributes when φ is held, never how much
exists.

We formalize this using def:balance's `Draw` / `freeLevel` machinery: if a
time `t` lies outside every draw's active interval (i.e. all draws have
"completed" relative to `t`), then the free level at `t` equals the initial
level `v0` — the net effect of each completed action is zero, so nothing
is lost or gained overall.
-/

structure Draw where
  start  : Int
  stop   : Int
  rate   : Int
  rate_nonneg : 0 ≤ rate

/-- Whether time `t` lies in the half-open interval `[d.start, d.stop)`. -/
def Draw.active (d : Draw) (t : Int) : Bool :=
  decide (d.start ≤ t) && decide (t < d.stop)

/-- The rate contributed by a draw at time `t`: its rate if active, else 0. -/
def Draw.contribution (d : Draw) (t : Int) : Int :=
  if d.active t then d.rate else 0

/-- The free level of a fluent at time `t`, given its initial level `v0`
and the list of draws acting on it. -/
def freeLevel (v0 : Int) (draws : List Draw) (t : Int) : Int :=
  v0 - (draws.map (fun d => d.contribution t)).foldl (· + ·) 0

/-- A single completed draw (one whose active interval does not contain `t`)
contributes nothing: the decrement at start is exactly cancelled by the
increment at end, so its net effect is zero. -/
theorem Draw.completed_contribution_zero (d : Draw) (t : Int)
    (h : d.active t = false) : d.contribution t = 0 := by
  unfold Draw.contribution
  rw [h]
  rfl

/-- prop:conserve — if every draw in `draws` has completed by time `t`
(none is active at `t`), the free level at `t` equals the initial level
`v0`: the terminal valuation equals the initial one, ν_end(φ) = ν0(φ).
Scheduling only redistributes *when* φ is held during the draws' active
windows, never how much exists once they have all completed. -/
theorem freeLevel_conserve (v0 : Int) (draws : List Draw) (t : Int)
    (hcompleted : ∀ d ∈ draws, d.active t = false) :
    freeLevel v0 draws t = v0 := by
  unfold freeLevel
  have hsum : (draws.map (fun d => d.contribution t)).foldl (· + ·) 0 = 0 := by
    have hzero : draws.map (fun d => d.contribution t) = draws.map (fun _ => (0 : Int)) := by
      apply List.map_congr_left
      intro d hd
      exact d.completed_contribution_zero t (hcompleted d hd)
    rw [hzero]
    clear hzero hcompleted
    induction draws with
    | nil => rfl
    | cons a as ih =>
      simpa using ih
  rw [hsum]
  simp
