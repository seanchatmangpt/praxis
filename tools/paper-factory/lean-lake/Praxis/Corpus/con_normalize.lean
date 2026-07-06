import Praxis.Corpus.thm_sep

/-!
Label: con:normalize

"Given a partition part $T'\subseteq T$ with entry places $P_{\text{in}}$ and
exit places $P_{\text{out}}$, the sub-net is normalized to a valid WF-net by
adding fresh boundary places $p_s,p_e$ and silent transitions
$\tau_{\text{in}},\tau_{\text{out}}$, redirecting boundary arcs onto a
unified source and sink."

Formalization strategy: a Petri/workflow net is realized as a plain record
`Net` of finite carriers -- `places : List String`, `transitions : List
String`, `arcs : List (String × String)` (an arc being a place/transition or
transition/place pair, encoded generically as a pair of node-name strings,
matching the string-labelled node style already used by `POWL`/`def_adm` in
this corpus). This is entirely data: `List`/`String`/`Prod`, no axioms.

The construction `normalize` takes a sub-net `T'` (itself a `Net`), its
entry places `Pin` and exit places `Pout` (both `List String`), and produces
the normalized WF-net by:
  * adding two fresh boundary places `p_s`, `p_e` to `places`,
  * adding two fresh silent transitions `tau_in`, `tau_out` to `transitions`,
  * redirecting every entry place `p ∈ Pin` onto the unified source via the
    arc `(p_s, tau_in)` then `(tau_in, p)` (source flows in through the
    silent transition to each entry place), and every exit place `p ∈ Pout`
    onto the unified sink via `(p, tau_out)` then `(tau_out, p_e)`,
  * keeping the sub-net's own internal arcs unchanged.

Freshness of `p_s`, `p_e`, `tau_in`, `tau_out` relative to the sub-net's
existing carriers is recorded as a `Prop`-valued side-condition
`FreshBoundary`, exactly the "hypothesis, not proof obligation" pattern
used for e.g. `Adm`'s `DecidablePred` side-condition in `def:adm` -- since
this label's `Kind` is `construction`, the only requirement is that the
construction type-checks, not that any theorem about it is proved here.

No axioms: `Net` and `normalize` are plain data built from `List`, `String`,
and `Prod`, mirroring the inductive/list-based style of `POWL` (`thm:sep`)
and the `List`/`Option`-based style of `def:adm`.
-/

/-- A workflow net as a plain finite-carrier record: place names, transition
names, and arcs (encoded generically as ordered pairs of node names, since a
Petri-net arc always connects a place to a transition or vice versa). -/
structure Net where
  places      : List String
  transitions : List String
  arcs        : List (String × String)
deriving Inhabited

/-- Side-condition recording that the four fresh boundary names
`p_s, p_e, tau_in, tau_out` do not already occur among the sub-net's places
or transitions. Not a proof obligation for this `construction`-kind label:
it is the hypothesis under which `normalize` is meant to be applied. -/
def FreshBoundary (T' : Net) (ps pe tauIn tauOut : String) : Prop :=
  ps ∉ T'.places ∧ pe ∉ T'.places ∧
  tauIn ∉ T'.transitions ∧ tauOut ∉ T'.transitions

/-- `con:normalize`: given a sub-net `T'` with entry places `Pin` and exit
places `Pout`, produce the normalized WF-net obtained by adding fresh
boundary places `ps`/`pe` and fresh silent transitions `tauIn`/`tauOut`, and
redirecting every entry place's boundary arc through `tauIn` from the
unified source `ps`, and every exit place's boundary arc through `tauOut`
into the unified sink `pe`. The sub-net's own internal arcs are kept as-is. -/
def normalize (T' : Net) (Pin Pout : List String)
    (ps pe tauIn tauOut : String) : Net :=
  { places      := ps :: pe :: T'.places
    transitions := tauIn :: tauOut :: T'.transitions
    arcs        :=
      T'.arcs
        ++ (ps, tauIn) :: (Pin.map (fun p => (tauIn, p)))
        ++ (Pout.map (fun p => (p, tauOut))) ++ [(tauOut, pe)] }
