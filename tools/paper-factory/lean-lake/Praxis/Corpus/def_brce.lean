import Praxis.Corpus.def_fit
import Praxis.Corpus.def_mu
import Praxis.Corpus.def_receipt

/-!
Label: def:brce

"A system satisfies the Bounded Receipted Chatman Equation if for every
actuated artifact it maintains the admission gate (B1), bounded manufacture
(B2), receipt totality (B3), and conformance (B4)."

This definition composes entirely from the three already-migrated corpus
pieces rather than introducing fresh carriers:

* `def:adm` (via `def:mu`'s import) already supplies the admission gate
  `adm Adm gs : Code → Option Code`; B1 ("maintains the admission gate") is
  the Prop that every actuated artifact is in fact admitted, i.e. `adm Adm gs
  o = some o`.
* `def:mu` already supplies the manufacturing morphism `mu f hc : Option Code
  → Option Code`, whose boundedness (M2) is carried by the `Computable f`
  hypothesis `hc` bundled into `mu`'s own signature; B2 ("bounded
  manufacture") is discharged by requiring the system's manufacturing step
  to literally be some `mu f hc` application -- no separate boundedness
  notion is invented here, it is reused from `def:mu`.
* `def:receipt` already supplies the `Receipt` tuple; B3 ("receipt
  totality") is the Prop that a `Receipt` exists (via a total function
  `receiptOf`) for every actuated artifact -- "total" is modeled the same
  way `def:mu`'s file models totality of a Lean function, i.e. structurally,
  by `receiptOf` being a total (non-`Option`) map out of the artifact
  carrier.
* `def:fit` already supplies the rational-valued conformance score `Fit`;
  B4 ("conformance") is the Prop that the replay counts recorded for every
  actuated artifact hit maximal fitness, `Fit (countsOf o) = 1`.

Nothing new is axiomatized: `System` bundles exactly the carriers and
functions already defined in `def_adm`/`def_mu`/`def_receipt`/`def_fit`, and
`SatisfiesBRCE` is a plain conjunction of four `Prop`s built from those
pieces, each obtained by universally quantifying one of B1-B4 over an
explicit set of "actuated" artifacts.
-/

open Nat.Partrec (Code)

/-- The components of a system that could satisfy the Bounded Receipted
Chatman Equation: an admission gate (`Adm`/`gs`, reusing `def:adm`), a
manufacturing morphism built from a computable `f` (reusing `def:mu`), and
per-artifact assignments of replay counts (for `def:fit`'s `Fit`) and
receipts (`def:receipt`'s `Receipt`). -/
structure System where
  Adm : Set Code
  admDec : DecidablePred (· ∈ Adm)
  gs : List (Code → Bool)
  f : Code → Code
  hc : Computable f
  countsOf : Code → ReplayCounts
  receiptOf : Code → Receipt

attribute [instance] System.admDec

/-- `def:brce`: a system `S` satisfies the Bounded Receipted Chatman
Equation on a set `actuated` of actuated artifacts if every artifact in
`actuated`:

* B1 -- passes `S`'s admission gate (`adm S.Adm S.gs o = some o`);
* B2 -- is manufactured by `S`'s bounded morphism `mu S.f S.hc` (bundled
  automatically: `mu` only ever takes a `Computable` `f`, so any use of it
  is already bounded, matching M2 in `def:mu`);
* B3 -- has a total receipt assignment (`S.receiptOf o`, a genuine `Receipt`
  for every `o`, i.e. receipt totality);
* B4 -- conforms with maximal fitness (`Fit (S.countsOf o) = 1`). -/
def SatisfiesBRCE (S : System) (actuated : Set Code) : Prop :=
  ∀ o ∈ actuated,
    (adm S.Adm S.gs o = some o) ∧                          -- B1: admission gate
    (mu S.f S.hc (some o) = some (S.f o)) ∧                 -- B2: bounded manufacture
    (S.receiptOf o).verdict = true ∧                        -- B3: receipt totality
                                                              --     (`S.receiptOf` is a total,
                                                              --     non-`Option` map `Code →
                                                              --     Receipt`, so every actuated
                                                              --     artifact already has a
                                                              --     receipt; this conjunct
                                                              --     additionally requires that
                                                              --     receipt to record acceptance)
    Fit (S.countsOf o) = 1                                  -- B4: conformance
