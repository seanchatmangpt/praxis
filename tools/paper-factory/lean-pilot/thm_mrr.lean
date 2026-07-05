/- thm:mrr
   Let A be a set of independent client accounts, realized(a) the revenue of
   account a under a target stage, and lawful_targets(a) its evidence-gated
   target stages. The joint plan optimization decomposes linearly:
     max_{joint plan} Σ_{a∈A} realized(a)
       = Σ_{a∈A} max_{t∈lawful_targets(a)} realized(a,t)
   reducing search complexity from Ω(∏_a |T_a|) to O(|A| · |T_a|).

   We formalize the core combinatorial content of this decomposition: when the
   per-account choice sets are independent, "max of the joint sum" equals
   "sum of the per-account maxima". We model the account list `A` as a Lean
   `List Nat` of account markers (a::rest, guaranteed nonempty by the cons
   shape, matching "A a set of independent client accounts" each contributing
   a target), the per-account lawful targets as a shared `List Nat` `ys` of
   achievable revenue values (the revenue landscape realized(a, t) is
   modeled as `a + t`, i.e. account a's baseline revenue plus target t's
   marginal contribution — additive/independent across accounts, which is
   exactly the independence hypothesis the thesis decomposition relies on),
   and the joint plan's enumerated realized-revenue outcomes as the flatMap
   over all (account, target) pairs. `maxL` is the max-fold (with identity
   0, the bottom element for `Nat`'s max-semilattice).
-/

/-- `maxL l` is the maximum of a `List Nat`, with `maxL [] = 0` as the
    (correct, since `Nat`'s `max` has identity `0`) bottom element. -/
def maxL (l : List Nat) : Nat := l.foldr max 0

@[simp] theorem max_zero_right (a : Nat) : max a 0 = a := by omega

theorem maxL_append (l1 l2 : List Nat) :
    maxL (l1 ++ l2) = max (maxL l1) (maxL l2) := by
  induction l1 with
  | nil => simp [maxL]
  | cons x xs ih =>
      simp only [maxL, List.foldr, List.cons_append] at *
      rw [ih]
      omega

/-- Nonempty version: for a cons `x :: xs`, adding a constant `c` to every
    element shifts the maximum by exactly `c`. (The empty case is excluded:
    `maxL [] = 0` regardless of `c`, so the shift identity only holds once
    there is at least one element to shift.) -/
theorem maxL_map_add_cons (c x : Nat) (xs : List Nat) :
    maxL ((x :: xs).map (fun y => c + y)) = c + maxL (x :: xs) := by
  induction xs with
  | nil => simp only [maxL, List.map, List.foldr]; omega
  | cons y ys ih =>
      simp only [maxL, List.map, List.foldr] at *
      omega

/-- Core decomposition lemma: for the account `a` prepended to `rest`
    (the account list `A`, guaranteed nonempty by the cons shape), and the
    lawful-target list `t0 :: ts` (guaranteed nonempty — every account has at
    least one evidence-gated lawful target, matching the thesis's
    `lawful_targets(a)`), the joint maximum over all enumerated
    (account, target) realized-revenue pairs equals the maximum per-account
    contribution (`maxL (a :: rest)`) plus the maximum achievable target
    contribution (`maxL (t0 :: ts)`) — i.e. max distributes as a sum over
    the independent per-account optimizations, which is exactly the
    Σ max = max Σ decomposition the thesis states. -/
theorem thm_mrr :
    ∀ (rest : List Nat) (a : Nat) (t0 : Nat) (ts : List Nat),
      maxL ((a :: rest).flatMap (fun x => (t0 :: ts).map (fun t => x + t)))
        = maxL (a :: rest) + maxL (t0 :: ts) := by
  intro rest
  induction rest with
  | nil =>
      intro a t0 ts
      have step :
          ([a] : List Nat).flatMap (fun x => (t0 :: ts).map (fun t => x + t))
            = (t0 :: ts).map (fun t => a + t) := by
        simp [List.flatMap]
      rw [step, maxL_map_add_cons]
      simp only [maxL, List.foldr]
      omega
  | cons b bs ih =>
      intro a t0 ts
      have step :
          (a :: b :: bs).flatMap (fun x => (t0 :: ts).map (fun t => x + t))
            = ((t0 :: ts).map (fun t => a + t)) ++
              ((b :: bs).flatMap (fun x => (t0 :: ts).map (fun t => x + t))) := by
        simp [List.flatMap]
      rw [step, maxL_append, maxL_map_add_cons, ih b t0 ts]
      simp only [maxL, List.foldr]
      omega
