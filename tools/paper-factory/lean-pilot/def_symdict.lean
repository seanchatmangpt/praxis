/-
def:symdict

A symbol identifier `SymId ∈ ℤ_{>0}` is a compact integer encoding of a ground
atom or object name; `pddl-index` maintains a bidirectional map
`SymDict : {strings} ↔ ℤ_{>0}`, encoded forward as `FxHashMap<String,u32>` and
reverse as `Vec<String>`; zero is reserved as undefined.
-/

/-- A symbol identifier is a positive natural number encoding of a ground
atom or object name. Zero is reserved as the "undefined" sentinel and is
excluded from `SymId` by construction. -/
def SymId := { n : Nat // n > 0 }

/-- The bidirectional symbol dictionary maintained by `pddl-index`.
`forward` is the `FxHashMap<String,u32>` side: given a string, look up its
encoded `SymId` (as a raw `Nat`, `0` meaning undefined/absent).
`reverse` is the `Vec<String>` side: given a `SymId`'s raw index, recover the
original string. -/
structure SymDict where
  /-- Forward map: string ↦ encoded id, with `0` reserved for "undefined". -/
  forward : String → Nat
  /-- Reverse map: encoded id (as a `Vec` index) ↦ original string. -/
  reverse : List String
  /-- Zero is never assigned as a real encoding. -/
  zero_reserved : ∀ s : String, forward s = 0 ∨ forward s > 0
