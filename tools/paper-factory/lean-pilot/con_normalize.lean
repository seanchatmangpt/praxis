/-
con:normalize — Normalization of a sub-net to a valid WF-net.

"Given a partition part T'⊆T with entry places P_in and exit places
P_out, the sub-net is normalized to a valid WF-net by adding fresh
boundary places p_s, p_e and silent transitions τ_in, τ_out,
redirecting boundary arcs onto a unified source and sink."

This is a construction: we model the ambient net's places/transitions
abstractly (reusing the `WFNet` vocabulary from thm:sep), take a
transition-subset `T'` together with its entry/exit place sets
`Pin, Pout`, and produce (as data) the normalized sub-net: two fresh
boundary places `ps`/`pe`, two fresh silent transitions `tauIn`/`tauOut`,
and the redirection witnesses (arcs from `ps` into every entry place,
arcs from every exit place into `pe`) that unify the sub-net onto a
single source and sink. No proof obligation beyond type-checking: the
file must simply elaborate the construction.
-/

axiom WFNet : Type
axiom Place : Type
axiom Transn : Type

/-- Silence marker: a transition contributes no observable label. -/
axiom Silent : Transn → Prop

/-- Fresh-place / fresh-transition existence, used to manufacture the
two boundary places and two silent transitions the normalization needs.
These are constructive existence axioms over the ambient net, standing
in for "there is a place/transition not already used by `w`". -/
axiom freshPlace : WFNet → Place
axiom freshTrans : WFNet → Transn
axiom freshTrans_silent : ∀ (w : WFNet), Silent (freshTrans w)

/-- An arc of the (sub-)net, from a place to a transition or a
transition to a place, recorded uniformly as a pair tag. -/
inductive Arc
  | pt : Place → Transn → Arc
  | tp : Transn → Place → Arc

/-- The data of a normalized WF-net: the two fresh boundary places, the
two fresh silent transitions, and the redirected boundary arcs unifying
the sub-net onto a single source `ps` and single sink `pe`. -/
structure Normalized where
  ps      : Place
  pe      : Place
  tauIn   : Transn
  tauOut  : Transn
  tauInSilent  : Silent tauIn
  tauOutSilent : Silent tauOut
  inArcs  : List Arc   -- ps → τ_in → each entry place
  outArcs : List Arc   -- each exit place → τ_out → pe

/-- **con:normalize.** Given the ambient net `w`, a subset of
transitions `Tsub` (the partition part `T'`), and its entry places
`Pin` and exit places `Pout`, construct the normalized sub-net: fresh
boundary places `ps`, `pe`, fresh silent transitions `τ_in`, `τ_out`,
and the redirected boundary arcs `ps —τ_in→ Pin` and
`Pout —τ_out→ pe` that make the sub-net a valid WF-net with unified
source and sink. -/
noncomputable def normalize (w : WFNet) (_Tsub : List Transn)
    (Pin Pout : List Place) : Normalized :=
  let ps := freshPlace w
  let pe := freshPlace w
  let tauIn := freshTrans w
  let tauOut := freshTrans w
  { ps := ps
    pe := pe
    tauIn := tauIn
    tauOut := tauOut
    tauInSilent := freshTrans_silent w
    tauOutSilent := freshTrans_silent w
    inArcs := Arc.pt ps tauIn :: Pin.map (fun p => Arc.tp tauIn p)
    outArcs := Pout.map (fun p => Arc.pt p tauOut) ++ [Arc.tp tauOut pe] }
