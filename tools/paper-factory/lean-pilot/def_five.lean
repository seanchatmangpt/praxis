/-
def:five — The Chatman equation relates five objects: the observation space `Obs`,
the admitted space `Adm`, the manufacturing morphism `μ`, the artifact/action space
`Act`, and the receipt space `Rec`.

We formalize this as a structure bundling the four carrier types together with the
manufacturing morphism relating them (`μ : Obs → Adm → Act → Rec`, i.e. a morphism
that, given an observation and an admitted witness, produces an artifact/action
paired with a receipt).
-/

structure ChatmanEquation where
  Obs : Type u
  Adm : Type u
  Act : Type u
  Rec : Type u
  μ   : Obs → Adm → Act × Rec
