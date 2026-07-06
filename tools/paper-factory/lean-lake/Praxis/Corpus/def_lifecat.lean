import Mathlib.CategoryTheory.PathCategory.Basic

/-!
# def:lifecat

`Life` is the free category on the linear quiver
`Raw --j--> Val --a--> Admd --r--> Rcpt`,
with four objects and three generating arrows `judge`, `admit`, `receipt`.

We realize the quiver on a four-object inductive type with exactly the three
generating edges (`raw ⟶ val`, `val ⟶ admd`, `admd ⟶ rcpt`) via `Quiver.mk`, and
then take `Life` to be Mathlib's free path category `CategoryTheory.Paths` on that
quiver -- reusing Mathlib's existing free-category construction (`Quiver.Path`,
`CategoryTheory.Paths.categoryPaths`) rather than hand-rolling composition/identity/
associativity from scratch.
-/

namespace Praxis.Corpus

open CategoryTheory

/-- The four objects of the linear quiver underlying `Life`. -/
inductive LifeObj : Type
  | raw
  | val
  | admd
  | rcpt
deriving DecidableEq

open LifeObj

/-- The three generating arrows `judge : raw ⟶ val`, `admit : val ⟶ admd`,
`receipt : admd ⟶ rcpt`, and no others. -/
inductive LifeHom : LifeObj → LifeObj → Type
  | judge : LifeHom raw val
  | admit : LifeHom val admd
  | receipt : LifeHom admd rcpt

instance lifeQuiver : Quiver LifeObj := ⟨LifeHom⟩

/-- `Life` is the free category on the linear quiver
`Raw --j--> Val --a--> Admd --r--> Rcpt`. -/
def Life : Type := CategoryTheory.Paths LifeObj

noncomputable instance : Category Life :=
  CategoryTheory.Paths.categoryPaths LifeObj

end Praxis.Corpus
