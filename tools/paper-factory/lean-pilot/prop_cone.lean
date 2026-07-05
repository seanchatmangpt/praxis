/-
prop:cone

Every marking m reachable from m₀ satisfies the state equation for some
x ∈ Z_{≥0}^{|T|}, hence lies in the integer points of m₀ + cone(N) intersected
with Z_{≥0}^p; the state equation is necessary for reachability but not
sufficient.

We formalize the "necessary" half: reachability (the reflexive-transitive
closure of single-transition firing, built on def:incidence's Net/Marking/
incidence apparatus) implies existence of a Parikh vector x witnessing the
state equation m = m₀ + N x. (The "not sufficient" half is a claim that a
converse can fail, not itself a proposition with a proof obligation here — it
is definitional commentary in the source text.)

Built on the same standalone (no-import) core-Lean apparatus as def:incidence.
-/

def Marking (p : Nat) : Type := Fin p → Nat

structure Net (p n : Nat) where
  pre  : Fin n → Marking p
  post : Fin n → Marking p

def IMarking (p : Nat) : Type := Fin p → Int

def Net.incidence {p n : Nat} (N : Net p n) : Fin n → IMarking p :=
  fun t i => (Int.ofNat (N.post t i)) - (Int.ofNat (N.pre t i))

def Parikh (n : Nat) : Type := Fin n → Nat

def incidenceSumAux {p n : Nat} (N : Net p n) (x : Parikh n) (i : Fin p) : Nat → Int
  | 0 => 0
  | k + 1 =>
    if h : k < n then
      let t : Fin n := ⟨k, h⟩
      incidenceSumAux N x i k + (Int.ofNat (x t)) * (N.incidence t i)
    else
      incidenceSumAux N x i k

def Net.incidenceSum {p n : Nat} (N : Net p n) (x : Parikh n) (i : Fin p) : Int :=
  incidenceSumAux N x i n

def Net.stateEquation {p n : Nat} (N : Net p n) (m₀ m : Marking p) (x : Parikh n) : Prop :=
  ∀ i : Fin p, (Int.ofNat (m i)) = (Int.ofNat (m₀ i)) + N.incidenceSum x i

/-- The zero Parikh vector (no transitions fired). -/
def zeroParikh (n : Nat) : Parikh n := fun _ => 0

/-- The Parikh vector `x` with one extra firing of transition `t`. -/
def updateParikh {n : Nat} (x : Parikh n) (t : Fin n) : Parikh n :=
  fun s => if s.val = t.val then x s + 1 else x s

/-- Reachability: `m` is reached from `m₀` by a (possibly empty) firing
    sequence, where each step fires an enabled transition `t`
    (`pre t ≤` current marking) and updates the marking by `pre`/`post`. -/
inductive Reachable {p n : Nat} (N : Net p n) (m₀ : Marking p) : Marking p → Prop
  | base : Reachable N m₀ m₀
  | step : ∀ {m : Marking p} (t : Fin n),
      Reachable N m₀ m →
      (∀ i, N.pre t i ≤ m i) →
      Reachable N m₀ (fun i => (m i - N.pre t i) + N.post t i)

theorem finEq {n : Nat} {a b : Fin n} (h : a.val = b.val) : a = b := by
  cases a; cases b; cases h; rfl

theorem incidenceSumAux_update {p n : Nat} (N : Net p n) (x : Parikh n) (t : Fin n) (i : Fin p) :
    ∀ k : Nat, incidenceSumAux N (updateParikh x t) i k
      = incidenceSumAux N x i k + (if t.val < k then N.incidence t i else 0)
  | 0 => by
      have h0 : ¬ (t.val < 0) := Nat.not_lt_zero _
      simp [incidenceSumAux, h0]
  | k + 1 => by
      have ih := incidenceSumAux_update N x t i k
      show incidenceSumAux N (updateParikh x t) i (k + 1)
        = incidenceSumAux N x i (k + 1) + (if t.val < k + 1 then N.incidence t i else 0)
      unfold incidenceSumAux
      by_cases h : k < n
      · simp only [dif_pos h]
        by_cases hk : k = t.val
        · have hteq : (⟨k, h⟩ : Fin n) = t := finEq hk
          have hupdval : updateParikh x t (⟨k, h⟩ : Fin n) = x (⟨k, h⟩ : Fin n) + 1 := by
            show (if k = t.val then x (⟨k, h⟩ : Fin n) + 1 else x (⟨k, h⟩ : Fin n))
              = x (⟨k, h⟩ : Fin n) + 1
            rw [if_pos hk]
          rw [ih, hupdval, hteq]
          have e1 : ¬ (t.val < k) := by omega
          have e2 : t.val < k + 1 := by omega
          rw [if_neg e1, if_pos e2]
          have hcast : (Int.ofNat (x t + 1) : Int) = Int.ofNat (x t) + 1 := rfl
          rw [hcast, Int.add_mul, Int.one_mul]
          omega
        · have hupdval : updateParikh x t (⟨k, h⟩ : Fin n) = x (⟨k, h⟩ : Fin n) := by
            show (if k = t.val then x (⟨k, h⟩ : Fin n) + 1 else x (⟨k, h⟩ : Fin n))
              = x (⟨k, h⟩ : Fin n)
            rw [if_neg hk]
          rw [ih, hupdval]
          by_cases hlt : t.val < k
          · have e1 : t.val < k + 1 := by omega
            rw [if_pos hlt, if_pos e1]
            omega
          · have e1 : ¬ (t.val < k + 1) := by omega
            rw [if_neg hlt, if_neg e1]
            omega
      · simp only [dif_neg h]
        exact ih

theorem incidenceSum_update {p n : Nat} (N : Net p n) (x : Parikh n) (t : Fin n) (i : Fin p) :
    N.incidenceSum (updateParikh x t) i = N.incidenceSum x i + N.incidence t i := by
  unfold Net.incidenceSum
  rw [incidenceSumAux_update N x t i n]
  rw [if_pos t.isLt]

theorem incidenceSumAux_zero {p n : Nat} (N : Net p n) (i : Fin p) :
    ∀ k : Nat, incidenceSumAux N (zeroParikh n) i k = 0
  | 0 => rfl
  | k + 1 => by
      have ih := incidenceSumAux_zero N i k
      unfold incidenceSumAux
      by_cases h : k < n
      · simp only [dif_pos h]
        rw [ih]
        show (0 : Int) + (Int.ofNat (zeroParikh n (⟨k, h⟩ : Fin n))) * N.incidence (⟨k, h⟩ : Fin n) i = 0
        unfold zeroParikh
        simp
      · simp only [dif_neg h]
        exact ih

theorem incidenceSum_zero {p n : Nat} (N : Net p n) (i : Fin p) :
    N.incidenceSum (zeroParikh n) i = 0 := incidenceSumAux_zero N i n

/-- **prop:cone.** Every marking reachable from `m₀` satisfies the state
    equation for some Parikh vector `x`. -/
theorem reachable_state_equation {p n : Nat} (N : Net p n) (m₀ mF : Marking p)
    (h : Reachable N m₀ mF) : ∃ x : Parikh n, N.stateEquation m₀ mF x := by
  induction h with
  | base =>
      refine ⟨zeroParikh n, ?_⟩
      intro i
      rw [incidenceSum_zero]
      omega
  | step t hprev henabled ih =>
      rename_i m
      obtain ⟨x, hx⟩ := ih
      refine ⟨updateParikh x t, ?_⟩
      intro i
      show Int.ofNat (m i - N.pre t i + N.post t i)
        = Int.ofNat (m₀ i) + N.incidenceSum (updateParikh x t) i
      have hle := henabled i
      have hsub : Int.ofNat (m i - N.pre t i + N.post t i)
          = Int.ofNat (m i) - Int.ofNat (N.pre t i) + Int.ofNat (N.post t i) := by
        omega
      rw [hsub, hx i, incidenceSum_update]
      unfold Net.incidence
      omega
