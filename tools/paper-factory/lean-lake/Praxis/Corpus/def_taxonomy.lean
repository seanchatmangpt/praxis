import Praxis.Corpus.con_denial

/-!
Label: def:taxonomy

"A refusal taxonomy is a pair $(S,\mathrm{cat})$ where $S$ is a finite set of
concrete refusal scenarios and $\mathrm{cat}:S\to C$ is a total map onto the
eight-element category set $C$."

`C`, the eight-element category set, is `Fin 8` -- Mathlib/core's standard
finite type of exactly eight elements, not a hand-rolled enum or new axiom.

`S`, "a finite set of concrete refusal scenarios", is any `Fintype S` --
core's standard finiteness typeclass, giving `S` a `Finset.univ : Finset S`
and decidable finiteness, matching "$S$ is a finite set" without introducing
a bespoke finiteness axiom.

`\mathrm{cat}:S\to C`, "a total map", is simply a function `S → Fin 8`: every
function in Lean/core's type theory is total by construction, so totality
needs no extra hypothesis.

The pair $(S,\mathrm{cat})$ is packaged as a structure `Taxonomy` bundling a
finite scenario type together with its categorizing map into `Fin 8`, reusing
core's `Fintype` and `Fin` rather than redefining finiteness or an
eight-element enumeration from scratch. This is a `definition`: it packages
the LaTeX's data with no new axioms and no proof obligation beyond this file
type-checking.
-/

/-- The eight-element category set `C` from the LaTeX statement. -/
abbrev RefusalCategory : Type := Fin 8

/-- A refusal taxonomy: a finite set `S` of concrete refusal scenarios
together with a total categorizing map `cat : S → C` onto the eight-element
category set `RefusalCategory`. -/
structure Taxonomy where
  /-- The (finite) type of concrete refusal scenarios. -/
  S : Type
  /-- `S` is finite, witnessing "a finite set of concrete refusal scenarios". -/
  finS : Fintype S
  /-- The total categorizing map `cat : S → C`. -/
  cat : S → RefusalCategory
