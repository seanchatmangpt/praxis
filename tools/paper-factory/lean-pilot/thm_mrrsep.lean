/-
thm:mrrsep

"If the accounts in A are independent, then
  MRR = Σ_{a ∈ A} max_{s ∈ lawful_targets(a)} realized(a,s),
reducing the joint plan search from exponential to linear in |lawful_targets(a)|."

We model (building on def:mrr's `sumRevenue`/`MRR` shape, specialised to Nat-valued
outcomes so the separability content can be proved without mathlib):

- `maxL l` is the maximum of a list of Nats (matching MRR's `foldr max 0`).
- `NEList` is a *nonempty* list of Nat outcomes (a value together with a tail),
  modelling one account's `lawful_targets` image under `realized(a,·)` — nonempty
  because every account has at least one lawful target.
- `cartSums targets` is the list of sums over the *joint* (cartesian-product) plan
  space for independent accounts — exponential in the number of accounts,
  `∏ |lawful_targets(a)|`.
- `sumMaxes targets` is `Σ_a max_s realized(a,s)`, computable in one linear pass.

`mrrsep` proves `maxL (cartSums targets) = sumMaxes targets`: the maximum over the
exponential joint plan space equals the linear sum of per-account maxima, i.e.
under independence MRR collapses from the exponential search to the linear one.
-/

def maxL (l : List Nat) : Nat := l.foldr max 0

@[simp] theorem maxL_nil : maxL ([] : List Nat) = 0 := rfl

theorem maxL_cons (x : Nat) (xs : List Nat) : maxL (x :: xs) = max x (maxL xs) := rfl

/-- Bare-core replacement for `List.bind` (not available without mathlib). -/
def flatMap {A B : Type} (l : List A) (f : A → List B) : List B :=
  l.foldr (fun a acc => f a ++ acc) []

@[simp] theorem flatMap_nil {A B : Type} (f : A → List B) : flatMap ([] : List A) f = [] := rfl

@[simp] theorem flatMap_cons {A B : Type} (x : A) (xs : List A) (f : A → List B) :
    flatMap (x :: xs) f = f x ++ flatMap xs f := rfl

/-- A nonempty list of Nat outcomes: `(v, tail)` represents `v :: tail`. -/
def NEList := Nat × List Nat

def NEList.toList (l : NEList) : List Nat := l.1 :: l.2

@[simp] theorem NEList.toList_mk (v : Nat) (tl : List Nat) :
    NEList.toList (v, tl) = v :: tl := rfl

def NEList.maxV (l : NEList) : Nat := maxL l.toList

/-- The joint (cartesian-product) plan space over independent accounts, collapsed
to the sums achievable by each joint plan. Exponential in the number of accounts. -/
def cartSums (targets : List NEList) : List Nat :=
  targets.foldr (fun ts acc => flatMap ts.toList (fun v => acc.map (fun r => v + r))) [0]

@[simp] theorem cartSums_nil : cartSums ([] : List NEList) = [0] := rfl

@[simp] theorem cartSums_cons (ts : NEList) (rest : List NEList) :
    cartSums (ts :: rest) =
      flatMap ts.toList (fun v => (cartSums rest).map (fun r => v + r)) := rfl

/-- The linear-pass sum of per-account maxima. -/
def sumMaxes (targets : List NEList) : Nat :=
  (targets.map NEList.maxV).foldr (· + ·) 0

@[simp] theorem sumMaxes_nil : sumMaxes ([] : List NEList) = 0 := rfl

@[simp] theorem sumMaxes_cons (ts : NEList) (rest : List NEList) :
    sumMaxes (ts :: rest) = ts.maxV + sumMaxes rest := rfl

theorem maxL_append (A B : List Nat) : maxL (A ++ B) = max (maxL A) (maxL B) := by
  induction A with
  | nil =>
    simp only [List.nil_append, maxL_nil]
    omega
  | cons x xs ih =>
    simp only [List.cons_append, maxL_cons]
    rw [ih]
    omega

theorem add_max_distrib (c a b : Nat) : c + max a b = max (c + a) (c + b) := by omega

theorem maxL_map_add (c : Nat) (L : List Nat) (hL : L ≠ []) :
    maxL (L.map (fun r => c + r)) = c + maxL L := by
  induction L with
  | nil => exact absurd rfl hL
  | cons x xs ih =>
    cases xs with
    | nil =>
      simp only [List.map_cons, List.map_nil, maxL_cons, maxL_nil]
      omega
    | cons y ys =>
      have hih : maxL ((y :: ys).map (fun r => c + r)) = c + maxL (y :: ys) := ih (by simp)
      conv_lhs => rw [List.map_cons, maxL_cons, hih]
      conv_rhs => rw [maxL_cons]
      omega

theorem maxL_bind_add (l : List Nat) (hl : l ≠ []) (L : List Nat) (hL : L ≠ []) :
    maxL (flatMap l (fun v => L.map (fun r => v + r))) = maxL l + maxL L := by
  induction l with
  | nil => exact absurd rfl hl
  | cons x xs ih =>
    cases xs with
    | nil =>
      rw [flatMap_cons, flatMap_nil, List.append_nil, maxL_map_add x L hL, maxL_cons, maxL_nil]
      omega
    | cons y ys =>
      have hih : maxL (flatMap (y :: ys) (fun v => L.map (fun r => v + r)))
          = maxL (y :: ys) + maxL L := ih (by simp)
      conv_lhs => rw [flatMap_cons, maxL_append, maxL_map_add x L hL, hih]
      conv_rhs => rw [maxL_cons]
      omega

/-- The nonemptiness invariant: the cartesian-product sums are always nonempty,
since every account contributes a nonempty (`NEList`) set of targets. -/
theorem cartSums_ne_nil (targets : List NEList) : cartSums targets ≠ [] := by
  induction targets with
  | nil => simp
  | cons ts rest ih =>
    rw [cartSums_cons, show ts.toList = ts.1 :: ts.2 from rfl, flatMap_cons]
    rcases hcs : cartSums rest with _ | ⟨r0, rs⟩
    · exact absurd hcs ih
    · simp

theorem mrrsep (targets : List NEList) : maxL (cartSums targets) = sumMaxes targets := by
  induction targets with
  | nil => rfl
  | cons ts rest ih =>
    have hts : ts.toList ≠ [] := by
      rw [show ts.toList = ts.1 :: ts.2 from rfl]; simp
    have hrest : cartSums rest ≠ [] := cartSums_ne_nil rest
    rw [cartSums_cons, sumMaxes_cons, maxL_bind_add ts.toList hts (cartSums rest) hrest, ih]
    rfl
