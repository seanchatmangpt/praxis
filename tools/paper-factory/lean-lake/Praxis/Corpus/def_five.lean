/-!
# def:five — The Chatman equation's five objects

"The Chatman equation relates five objects: the observation space `Obs`, the
admitted space `Adm`, the manufacturing morphism `mu`, the artifact/action
space `Act`, and the receipt space `Rec`."

This is a bare structural definition bundling the five named components: three
carrier types (`Obs`, `Adm`, `Act`), a fourth carrier type (`Rec`), and the
manufacturing morphism `mu` relating the admitted space to the artifact space.
No concrete instantiation is prescribed by the source text, so the carriers
are left as abstract `Type`s (composed from core `Type`, not axiomatized) and
`mu` is a plain function type between two of them — nothing here needs an
`axiom`: a `structure` bundling existing `Type`s and a function field is
exactly the pre-built composition Mathlib/core already provides.
-/

universe u

structure ChatmanFive where
  /-- The observation space. -/
  Obs : Type u
  /-- The admitted space. -/
  Adm : Type u
  /-- The artifact/action space. -/
  Act : Type u
  /-- The receipt space. -/
  Rec : Type u
  /-- The manufacturing morphism from the admitted space to the artifact space. -/
  mu : Adm → Act
