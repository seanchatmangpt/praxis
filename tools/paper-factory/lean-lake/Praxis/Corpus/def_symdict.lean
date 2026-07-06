import Mathlib.Data.PNat.Basic
import Std.Data.HashMap.Basic

/-!
# def:symdict — Symbol dictionary

A symbol identifier `SymId ∈ ℤ_{>0}` is a compact integer encoding of a ground atom
or object name; `pddl-index` maintains a bidirectional map
`SymDict : {strings} ↔ ℤ_{>0}`, encoded forward as `FxHashMap<String,u32>` and
reverse as `Vec<String>`; zero is reserved as undefined.

`SymId` is modeled as Mathlib's `PNat` (the positive naturals, ℤ_{>0} restricted to
`≥ 1`) rather than a hand-rolled subtype — this is exactly the "compact integer,
zero excluded" carrier the source text describes, and Mathlib already proves the
arithmetic/order API for it. The forward map `FxHashMap<String,u32>` is modeled by
`Std.HashMap String PNat` (Std's hash map is the Lean-side analogue of Rust's
`FxHashMap`: same amortized O(1) lookup contract, just a different hash function
choice — the source text does not depend on which hash is used). The reverse map
`Vec<String>` is modeled by `Array String`, indexed by the `PNat` id minus one
(since Rust's `Vec` is zero-based but `SymId` reserves `0` as undefined, id `n`
lives at array index `n - 1`).
-/

namespace Praxis.Corpus.DefSymdict

/-- A symbol identifier: a compact integer encoding of a ground atom or object
name, drawn from the positive integers (`0` is reserved as undefined and is
therefore excluded from the carrier type itself). -/
abbrev SymId := PNat

/-- The symbol dictionary maintained by `pddl-index`: a bidirectional map between
strings and symbol identifiers, encoded forward as a hash map from strings to
`u32`-like ids (`Std.HashMap String SymId`, the Lean analogue of
`FxHashMap<String,u32>`) and reverse as an array of strings (the Lean analogue of
`Vec<String>`), indexed by `id - 1` since `Array` is zero-based but `SymId`
excludes zero. -/
structure SymDict where
  /-- Forward map: string ⟶ symbol id (`FxHashMap<String,u32>` analogue). -/
  forward : Std.HashMap String SymId
  /-- Reverse map: symbol id (as a zero-based array index) ⟶ string
  (`Vec<String>` analogue). -/
  reverse : Array String

end Praxis.Corpus.DefSymdict
