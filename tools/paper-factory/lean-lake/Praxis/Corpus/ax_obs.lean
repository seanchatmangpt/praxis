import Mathlib.Data.Fintype.Basic

/-!
Label: ax:obs

There is a set `Obs`, the observation space, whose elements are arbitrary finite
records; `Obs` carries no decidable semantics.

`Obs` is kept as an `axiom` (opaque type), not a Mathlib composition: the source
statement deliberately leaves the internal shape of "finite records" and the
semantics of observations unspecified/abstract, and explicitly asserts that no
decidable semantics exists on it. There is no single pre-built Mathlib type that
is simultaneously (a) a stand-in for an arbitrary/opaque finite-record format and
(b) equipped with no decidable equality/semantics by construction -- picking any
concrete Mathlib encoding (e.g. `List (String × ByteArray)`) would smuggle in a
decidable semantics the thesis explicitly says not to assume. This mirrors the
justification style of `ObsSimEquivalence.lean`'s abstract observation space.
-/

axiom Obs : Type
