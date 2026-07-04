# The algebra, geometry, and calculus of ggen

This is the formal foundation for the from-scratch `praxis/crates/ggen`,
independent of any existing implementation (all of which are DROP/REIMPLEMENT
per `ggen-port-evaluation.md`). The point of writing this down before code:
every praxis "iron invariant" that evaluation kept citing (typed `Refusal`,
computed-not-asserted receipts, no wall-clock in hash paths, closed
vocabulary) is not house style — it is the exact condition required for the
algebra below to hold. Violate one and a specific law breaks, provably, not
just "against convention."

ggen's own slogan is `A = μ(O)`: artifacts are a function of an ontology.
Below is what kind of function μ has to be, what space O and A live in, and
what "change" means in that space.

---

## 1. Algebra: μ as a composed morphism in a category of graphs

**Objects.** Let **Ont** be the category whose objects are RDF graphs (sets
of triples/quads over a fixed universe of IRIs, literals, blank nodes) and
whose morphisms `O → O'` are graph homomorphisms that preserve a chosen
closed predicate vocabulary `V` (see §2). "Closed vocabulary" is exactly the
statement that **Ont** is not the category of *all* graphs — it is the full
subcategory whose objects use only predicates in `V`. A module that accepts
arbitrary predicates (every evaluated ggen crate did) isn't working in
**Ont** at all; it's working in the ambient category of all graphs, where
none of the guarantees below apply. This is why "closed vocabulary" is
load-bearing, not aesthetic.

**The pipeline as a composed functor.** ggen's five stages are morphisms
composed in order:

```
μ = μ₅ ∘ μ₄ ∘ μ₃ ∘ μ₂ ∘ μ₁ : Ont → Artifact
```

- `μ₁` (load): embed a Turtle/JSON-LD source into **Ont** — the only stage
  that can fail on *malformed input*, and it must fail as a typed `Refusal`,
  not a panic, because everything downstream assumes a well-typed object of
  **Ont** exists.
- `μ₂` (inference): `Ont → Ont`, a SPARQL `CONSTRUCT` endomorphism. For this
  to be well-behaved it should be a **closure operator**: extensive
  (`O ⊆ μ₂(O)`), monotone (`O ⊆ O' ⟹ μ₂(O) ⊆ μ₂(O')`), and idempotent
  (`μ₂(μ₂(O)) = μ₂(O)`). Idempotence is the algebraic reason "run inference
  twice" must be a no-op — if it isn't, μ₂ isn't a closure, and downstream
  determinism (§1.3) has no foundation.
- `μ₃` (generation): **Ont → Template-Context → Text**, a pure function of
  the graph (SPARQL SELECT projects a relation, Tera renders it). Pure means
  no wall-clock, no ambient mutable state — literally the "no `Utc::now()`
  in the render path" rule, stated algebraically: `μ₃` is a *function*, and
  a function whose output depends on when you called it is not a function
  of its argument, it's a function of `(argument, time)` — a different,
  unstated domain.
- `μ₄` (validation): **Ont → Bool** (SHACL/ASK), defining the feasible
  region (§2).
- `μ₅` (emit): **Text → Filesystem**, plus a **Receipt** — see §1.2.

**The monoid of generation rules.** A `ggen.toml` manifest is a finite set
of generation rules `{r₁, …, rₙ}`, each `rᵢ : Ont → Artifact_i`. Running
`sync` computes `⋃ᵢ rᵢ(O)`. This union is commutative and associative in
rule order *only if* rules don't have side effects on shared mutable state
(e.g. two rules racing to write the same `output_file`) — which is exactly
why ggen.toml rules declare disjoint `output_file`s. The monoid identity is
the empty rule set: `sync` on zero rules writes nothing, changes nothing —
this is the algebraic content of "dry-run" and "no rules matched" both being
`Ok(∅)`, not an error.

### 1.1 Deltas form a group, not just "a diff"

Represent a graph as a finite quad-set. Define `Δ(O, O') = (O' \ O, O \ O')`
— the pair (additions, removals). Deltas compose:

```
Δ(O, O'') = Δ(O, O') ⊕ Δ(O', O'')
```

where `⊕` merges (additions, removals) pairs with cancellation (a quad
added then removed cancels). This makes deltas a **group** under `⊕`, with
identity `(∅, ∅)` and inverse `Δ(O, O')⁻¹ = Δ(O', O)` (swap the pair). This
is not decoration — it is what makes **replay** (§3) well-defined: if
deltas didn't form a group, "reconstruct O from genesis plus an ordered
list of deltas" wouldn't have a unique answer.

### 1.2 The receipt hash must be a homomorphism, not an oracle

A receipt's hash has exactly one algebraically sound definition:

```
H : Ont → Hash,  H(O) = BLAKE3(canonicalize(O))
```

where `canonicalize` is a *pure function of the quad-set* (sort quads into
a total order, e.g. by (subject, predicate, object, graph) with blank-node
labels alpha-renamed to a canonical scheme). Two properties this buys, both
of which the evaluated codebases broke:

- **Well-definedness on the quotient.** RDF graphs are *sets* of triples;
  the in-memory/on-disk representation may list them in any order.
  `canonicalize` must map every representation of the same graph to the
  same sorted sequence — i.e. `H` factors through the quotient by
  permutation. Skip canonicalization and `H` isn't a function of the graph
  at all, it's a function of incidental serialization order.
- **`H` is a pure function, full stop — no second hidden argument.** The
  moment `H`'s implementation folds in `Utc::now()` (found in 5 modules
  across ggen-graph/ggen-core/cpmp during evaluation), `H` secretly has
  signature `Ont × Time → Hash`. Two syntactically identical calls now
  produce different outputs, which means `H` is not a hash *of the
  ontology* — it's a hash of `(ontology, clock reading)`, and "receipt
  proves O produced this artifact" stops being true, because the receipt
  no longer determines O (many different clock readings produce the same
  H alongside different real content, and worse, the same O produces
  different H's at different times, breaking reproducibility outright).
  "No wall-clock in any hash path" is the precise algebraic requirement
  that H remain a function of one argument.

**Asserted-vs-computed** is the same law from the caller's side: if a
`Receipt` struct lets a caller *set* `input_hashes`/`signature` fields
directly (found in `ggen-core/receipt/receipt_impl.rs`), the type system no
longer enforces `H = BLAKE3(canonicalize(O))` — it allows `H` to be any
string at all, i.e. an unconstrained function, which is not a hash of
anything. The fix is a type-level one: a `Receipt` constructor should take
`&Ont` and compute the hash internally; there should be no code path that
constructs a `Receipt` from a caller-supplied hash string.

### 1.3 Chain composition: hashes as a monoid homomorphism into a chain

A receipt chain (`prev_chain_hash → chain_hash`) is:

```
chain_hash(Rₙ) = H(record_n ‖ chain_hash(Rₙ₋₁)),  chain_hash(R₀) = genesis
```

This is a monoid homomorphism from `(sequence of records, concatenation)`
into `(Hash, ‖-then-hash)`. Tamper-detection (`recompute and compare`,
exactly the pattern in `praxis-core/receipt_validator.rs`'s
`check_chain_recompute`) works *because* this is a homomorphism: any
alteration to record `i` changes `chain_hash(Rᵢ)`, which — because every
later hash is a function of the previous one — necessarily changes every
`chain_hash(Rⱼ)` for `j ≥ i`. If the hash function weren't pure (§1.2),
this propagation property doesn't hold and tamper detection is unsound.

---

## 2. Geometry: the feasible region and the metric

**State space.** Fix the closed vocabulary `V`. The space of all
well-formed ontologies is the (huge but discrete) set `Graphs(V)` of finite
quad-sets over `V`. Think of it as the vertex set of a lattice ordered by
`⊆` (graph inclusion) — the same lattice a closure operator (§1, μ₂) acts
on.

**SHACL/ASK validation carves out a feasible region.** `μ₄ : Graphs(V) →
{valid, invalid}` partitions the space into a feasible subset `F ⊆
Graphs(V)` and its complement. Only `O ∈ F` may proceed to `μ₅`. This is
the geometric content of "validation, not just generation" — `μ` is really
`μ₅ ∘ (μ₄ restricted to F) ∘ μ₃ ∘ μ₂ ∘ μ₁`, a **partial function**, and the
`Refusal` it returns off `F` is not an error path bolted onto a total
function — it is the honest description of `μ`'s actual domain.

**A metric via symmetric difference.** Define
`d(O, O') = |O △ O'| = |O ∖ O'| + |O' ∖ O|` (the size of the delta from
§1.1). This is a genuine metric (non-negative, symmetric, triangle
inequality via set inclusion-exclusion, zero iff equal). Two uses:

- **Determinism as a Lipschitz-like statement.** For μ to be worth trusting
  incrementally (§3), small `d(O, O')` should produce small, localized
  changes in the generated artifact set — not because of a formal bound
  the code enforces, but because generation rules are typically scoped to
  specific SPARQL patterns; a delta that doesn't touch a rule's pattern
  shouldn't touch that rule's output. This is the geometric justification
  for **incremental regeneration**: only re-run rule `rᵢ` if its query's
  result set actually changed under the delta, not on every `sync`.
- **Drift as distance from a reference point.** "Documentation drift" /
  "code drift" is literally `d(actual_source_AST, μ₃(O))  > 0` — a
  nonzero distance between the generated artifact and the artifact
  actually shipped. Drift-prevention machinery (BLAKE3 document hashing,
  AST comparison — see the earlier docs-RFC work) is exactly "monitor
  `d(·,·)` and refuse when it's nonzero," the same metric reused at a
  different pair of points (rendered text vs. compiled AST, rather than
  two ontology snapshots).

**Closure operators are topological closures.** μ₂'s three properties
(extensive, monotone, idempotent — §1) are precisely the Kuratowski closure
axioms (minus the `cl(∅)=∅` and `cl(A∪B)=cl(A)∪cl(B)` axioms, which don't
generally hold for SPARQL CONSTRUCT and aren't needed here). This gives a
concrete test for whether an inference rule is well-formed: check
`μ₂(μ₂(O)) = μ₂(O)` on real data before shipping the rule, not just "it ran
without erroring."

---

## 3. Calculus: differential and integral generation

**Differentiation = delta.** `d/dt` in this world is the delta operator
`Δ(O, O')` from §1.1 — the difference between two ontology snapshots. It's
already a first-class value (a pair of quad-sets), not a derived
side-effect of diffing two files.

**Integration = replay.** The inverse operation reconstructs a later state
from an earlier one plus an ordered sequence of deltas:

```
O_n = O_0 ⊕ Δ_1 ⊕ Δ_2 ⊕ … ⊕ Δ_n
```

This is a discrete Riemann sum / the discrete fundamental theorem of
calculus: `O_n - O_0 = Σ Δᵢ`, and crucially it must not depend on *when*
each `Δᵢ` was applied, only on their order and content — which is exactly
why receipts must not embed wall-clock time (§1.2): a proof system for
"replaying the deltas reproduces `O_n`" cannot depend on a quantity (time)
that isn't itself one of the `Δᵢ`. If a timestamp needs to be part of the
record, it must enter as **declared graph data** (an OWL-Time literal
that's part of some `Δᵢ`'s addition set) — never read from the wall clock
at hash-computation time. This is precisely praxis's rule #3 restated in
calculus terms: time is data (something differentiated/integrated over),
never an ambient variable the integral secretly depends on.

**Directional derivative = incremental generation.** `μ₃` applied to a
full `O` from scratch is expensive; the useful operation is the directional
derivative of `μ` along a delta:

```
∂μ/∂Δ (O) ≈ μ(O ⊕ Δ) − μ(O)   (as a set of changed artifacts, not a real number)
```

Concretely: for each rule `rᵢ`, compute whether `Δ`'s added/removed quads
intersect `rᵢ`'s SPARQL query pattern; if not, `rᵢ`'s output is unchanged
and can be skipped entirely. This is the formal justification for
`ggen sync --stage <rule>` and for building a dependency graph from
queries to output files (a rule "depends on" exactly the predicates its
`WHERE` clause mentions) — the chain rule of this calculus is: the
sensitivity of the whole pipeline's output to a change in one predicate is
the composition of each stage's local sensitivity to that predicate, which
is computable statically from the SPARQL query text without running
anything.

**The chain rule across μ₁…μ₅.** Because `μ = μ₅∘μ₄∘μ₃∘μ₂∘μ₁`, a change at
`μ₁` (a different Turtle load) propagates through every later stage. This
is why μ₂'s idempotence (§1) matters for the calculus too: if inference
weren't idempotent, "small change in, run inference, run generation" would
not have a stable meaning — the same input delta could produce different
outputs depending on how many times inference happened to run, which
breaks the well-definedness of `∂μ/∂Δ` itself.

---

## 4. Summary: why the invariants are the theory, not house style

| praxis "iron invariant" | algebraic/geometric/calculus statement it enforces |
|---|---|
| Typed `Refusal`, never panic | `μ` is a **partial function** on `Graphs(V)`, restricted to the feasible region `F` (§2); a `Refusal` is the honest value outside `F`, a panic is an undefined one. |
| Receipts computed, never asserted | The receipt hash `H` must be the pure function `BLAKE3∘canonicalize` (§1.2); a settable field breaks `H`'s well-definedness entirely. |
| No wall-clock in any hash/receipt path | `H` and the chain/replay algebra (§1.3, §3) require purity and time-independence; wall-clock input silently changes `H`'s domain and breaks reproducibility and the fundamental-theorem identity `O_n − O_0 = ΣΔᵢ`. |
| Closed vocabulary tables | **Ont** is defined as graphs over a fixed predicate set `V`; this is the actual object of study, not an incidental filter — code that accepts arbitrary predicates is not implementing ggen's algebra, it's implementing something else that happens to share code. |

This table is the acceptance criterion for the from-scratch
`praxis/crates/ggen`: a module is done not when it compiles and passes
tests, but when it can be checked against the specific law above it's
supposed to satisfy — idempotence of `μ₂`, purity of `H`, closure of `V`,
partiality of `μ` restricted to `F`. The porting roadmap in
`ggen-port-evaluation.md` should be read against this document: every
REIMPLEMENT-DIFFERENTLY verdict there is, precisely, "this module's code
does not satisfy the law its position in the algebra requires."
