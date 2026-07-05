/-
prop:dictcost

With symbol dictionary encoding, a ground atom p(o_1,...,o_k) is a tuple of
k+1 SymId values; equality reduces to integer comparison and hash-set
membership to a single hash over 4(k+1) bytes, so the grounding loop's cost
with encoding replaces O(|T|*k*N) string comparisons with O(1)-time integer
counterparts.

We formalize the structural core of this claim in bare Lean 4 core, reusing
`SymId` from def:symdict. A ground atom is represented as a `List SymId` of
length k+1 (predicate symbol id followed by k object ids). Comparing two such
encoded atoms costs one O(1) integer comparison per component, i.e.
`encodedCost atom = atom.length`. By contrast, comparing the underlying
strings costs at least one unit per byte of each component's string, i.e.
`stringCost atom len = sum of len s` where `len s ≥ 1` for every symbol `s`.
We prove `encodedCost atom ≤ stringCost atom len` under that minimal-length
hypothesis: the encoded, O(1)-per-symbol cost never exceeds the string-based
cost, witnessing that dictionary encoding replaces per-byte string comparison
with O(1) integer comparison.
-/

/-- A symbol identifier is a positive natural number encoding of a ground
atom or object name (reused from def:symdict). -/
def SymId := { n : Nat // n > 0 }

/-- Cost of comparing an encoded ground atom: one O(1) integer comparison
per `SymId` component. A ground atom `p(o_1,...,o_k)` is encoded as a list of
`k+1` `SymId`s, so this is `k+1`, independent of any symbol's string length. -/
def encodedCost (atom : List SymId) : Nat := atom.length

/-- Cost of comparing the same atom via its underlying strings: the sum,
over each component, of that component's string length (`len s`). -/
def stringCost (atom : List SymId) (len : SymId → Nat) : Nat :=
  (atom.map len).foldr (· + ·) 0

/-- Each component contributes at least one unit to `stringCost` once we
unfold one `cons`. -/
theorem stringCost_cons (a : SymId) (rest : List SymId) (len : SymId → Nat) :
    stringCost (a :: rest) len = len a + stringCost rest len := by
  simp [stringCost]

/-- prop:dictcost (structural core): the encoded, O(1)-per-symbol comparison
cost of a ground atom never exceeds the string-based comparison cost, as
long as every symbol's underlying string is nonempty (`len s ≥ 1`). Hence
dictionary encoding replaces per-byte string comparison with O(1) integer
comparison. -/
theorem encodedCost_le_stringCost
    (atom : List SymId) (len : SymId → Nat)
    (hlen : ∀ s, s ∈ atom → 1 ≤ len s) :
    encodedCost atom ≤ stringCost atom len := by
  induction atom with
  | nil => simp [encodedCost, stringCost]
  | cons a rest ih =>
    have ha : 1 ≤ len a := hlen a (by simp)
    have hrest : ∀ s, s ∈ rest → 1 ≤ len s :=
      fun s hs => hlen s (by simp [hs])
    have hstep : encodedCost rest ≤ stringCost rest len := ih hrest
    have hcons : stringCost (a :: rest) len = len a + stringCost rest len :=
      stringCost_cons a rest len
    simp only [encodedCost, List.length_cons] at *
    omega
