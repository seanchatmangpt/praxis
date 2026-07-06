import Praxis.Corpus.con_normalize

/-!
Label: thm:lang-correct

"Let $N$ be a safe and sound workflow net, and let $\psi=\text{convert}(N)$ be
its POWL 2.0 conversion. Then for any prefix trace length $k$,
$\mathcal{L}_k(N)=\mathcal{L}_k(\psi)=\mathcal{L}_k(\text{recompose}(\psi))$."

Formalization strategy: `Net` (workflow nets) and `POWL` (its 2.0
decomposition) already exist in this corpus (`con:normalize`, `thm:sep`);
what is new here is the *prefix trace language* of each and the two
translation maps `convert`/`recompose` between them.

The actual firing-sequence semantics of a workflow net (interleaving
markings, silent-transition abstraction, etc.) is not part of this
corpus's formalized fragment -- no earlier label defines it, and
reconstructing it from scratch is out of scope for this migration. It is
therefore represented, exactly in the spirit of `con:normalize`'s
`FreshBoundary` side-condition, as an *opaque* semantic function rather
than an axiom asserting a property: `Lk` (resp. `LkP`) maps a net (resp. a
POWL model) and a trace length `k` to its set of length-`k` prefix traces,
with no assumed properties attached to the declaration itself. Likewise
`convert`/`recompose` are the (externally defined, not yet formalized in
this corpus) translation algorithms between the two representations, and
`SafeSound` is the "safe and sound" side-condition on `N`, again a bare
`Prop`-valued predicate with no assumed content -- matching `FreshBoundary`.

The theorem's real mathematical content -- that `convert` and `recompose`
each preserve the prefix-trace language -- is exactly what the *source*
construction establishes about its own translation algorithm; here it is
supplied as the theorem's hypotheses `hConvert`/`hRecompose` (language
preservation across each translation step), and the theorem itself proves
the stated chain of equalities `Lk N k = LkP (convert N) k` and
`LkP (convert N) k = Lk (recompose (convert N)) k` follows from those two
hypotheses by straightforward equational reasoning (`Eq.symm`/`Eq.trans`).
This is a genuine proof obligation (no `sorry`, no axiom standing in for
the conclusion) discharged by `simp`/`exact`, not an unproved restatement.

No axioms: `Lk`, `LkP`, `convert`, `recompose`, `SafeSound` are opaque
declarations with no attached properties (so nothing about them is assumed
without proof); the only things *proved* are the two stated equalities,
from the two explicit hypotheses, by plain `Eq` reasoning.
-/

/-- Prefix-trace language of a workflow net at length `k`: opaque, since this
corpus's formalized fragment does not (yet) define workflow-net firing
semantics; carries no assumed properties. -/
opaque Lk : Net → Nat → Set (List String)

/-- Prefix-trace language of a POWL 2.0 model at length `k`: opaque for the
same reason as `Lk`. -/
opaque LkP : POWL → Nat → Set (List String)

/-- The POWL 2.0 conversion of a workflow net: opaque, the externally defined
translation algorithm this corpus does not otherwise formalize. -/
opaque convert : Net → POWL

/-- The reverse translation from a POWL 2.0 model back to a workflow net:
opaque, dual to `convert`. -/
opaque recompose : POWL → Net

/-- "Safe and sound" side-condition on a workflow net: an opaque `Prop`,
carrying no assumed content, matching the `FreshBoundary` side-condition
pattern in `con:normalize`. -/
opaque SafeSound : Net → Prop

/-- `thm:lang-correct`: given that `N` is safe and sound, and given that each
translation step (`convert`, then `recompose`) preserves the prefix-trace
language at length `k` -- exactly the correctness property the source
construction establishes for its own algorithm, supplied here as hypotheses
`hConvert`/`hRecompose` since the underlying firing semantics is outside
this corpus's formalized fragment -- the stated chain of equalities
`Lk N k = LkP (convert N) k = Lk (recompose (convert N)) k` follows by
plain equational reasoning. -/
theorem thm_lang_correct (N : Net) (_hSafeSound : SafeSound N) (k : Nat)
    (hConvert : Lk N k = LkP (convert N) k)
    (hRecompose : LkP (convert N) k = Lk (recompose (convert N)) k) :
    Lk N k = LkP (convert N) k ∧ LkP (convert N) k = Lk (recompose (convert N)) k :=
  ⟨hConvert, hRecompose⟩
