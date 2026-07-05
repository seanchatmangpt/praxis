/-
con:xorf — Let R ⊆ Obs be the reachability fixpoint fact set. A frozen fact
store builds an 8-bit XOR filter over FNV-1a hashes of R, with index mapping
via the fastrange reduction reduce(x,n) = (x * n) / 2^32; membership is gated
by x ∈ B if F(x) = true, else false, with false-positive rate ≈ 0.4% and
false negatives 0.

We model this abstractly in bare Lean 4 core (no mathlib), reusing the
`GroundState` / ground-atom vocabulary from def:ground: the reachability
fixpoint fact set `R` is a `List Atom` of true ground atoms (as would sit in
some `GroundState.atoms`). The construction here packages:

  - an FNV-1a-style hash function `Atom → UInt32`,
  - the fastrange reduction `reduce x n = (x * n) / 2^32` mapping a hash into
    a bucket index range,
  - an 8-bit fingerprint array `B : Array UInt8` (the frozen filter storage),
  - the membership test `F x = true ↔ x ∈ B` gate described informally as
    "false positive rate ≈ 0.4%, false negatives 0" — modeled as a
    `Prop`-valued specification field `noFalseNegatives` rather than proved
    numerically, since the probabilistic bound is a property of the concrete
    hash family, not a fact derivable in bare Lean core.

As a construction, the obligation is only that the definitions type-check;
no theorem is proved here.
-/

/-- The fastrange reduction: maps a 32-bit hash `x` into the range `[0, n)`
    via `reduce(x, n) = (x * n) / 2^32`, computed here in `UInt64` to avoid
    overflow of the intermediate product. -/
def reduce (x n : UInt32) : UInt32 :=
  UInt32.ofNat (((UInt32.toNat x) * (UInt32.toNat n)) / 4294967296)

/-- An 8-bit XOR filter built over a ground atom universe `Atom`. -/
structure XorFilter (Atom : Type) where
  /-- FNV-1a-style hash of a ground atom into 32 bits. -/
  hash        : Atom → UInt32
  /-- number of buckets in the frozen fingerprint table -/
  numBuckets  : UInt32
  /-- the frozen 8-bit fingerprint store -/
  fingerprints : Array UInt8
  /-- fingerprint derived from a hash (the low 8 bits, abstractly) -/
  fingerprint  : UInt32 → UInt8
  /-- membership gate: F(x) = true iff the fingerprint at the reduced bucket
      matches x's own fingerprint -/
  member       : Atom → Bool :=
    fun a =>
      let idx := reduce (hash a) numBuckets
      match fingerprints[(UInt32.toNat idx)]? with
      | some fp => fp == fingerprint (hash a)
      | none    => false
  /-- specification: no false negatives — every atom actually in the frozen
      fact set R is reported as a member by the filter -/
  noFalseNegatives : List Atom → Prop :=
    fun R => ∀ a ∈ R, member a = true

/-- Build the frozen XOR filter from the reachability fixpoint fact set `R`,
    given a hash function, a target bucket count, a fingerprint extractor,
    and the already-constructed fingerprint table (the construction of the
    fingerprint table itself, via XOR-based peeling, is left as the abstract
    input `table` since it depends only on `R`, `hash`, and `numBuckets`). -/
def buildXorFilter
    (Atom : Type)
    (R : List Atom)
    (hash : Atom → UInt32)
    (numBuckets : UInt32)
    (fingerprint : UInt32 → UInt8)
    (table : Array UInt8) :
    XorFilter Atom :=
  { hash := hash
    numBuckets := numBuckets
    fingerprints := table
    fingerprint := fingerprint }
