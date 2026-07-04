# The Delta Algebra of Receipts

## Why a delta at all

`crates/ggen/src/graph.rs` builds `DeterministicGraph` around a BLAKE3
`state_hash()` computed over "sorted canonical N-Quads" (crates/ggen/src/graph.rs:86-99).
A hash by itself only answers one question — *are these two states identical?*
It cannot tell you *what changed*, and praxis needs that answer constantly:
receipts have to describe a transition, not just certify an endpoint. That is
the job of `Delta`:

```rust
pub struct Delta {
    pub additions: Vec<String>,
    pub deletions: Vec<String>,
}
```

(crates/ggen/src/graph.rs:104-110). Additions and deletions are stored as
already-canonicalized N-Quads strings — the same strings that feed
`state_hash` — so a `Delta` is, structurally, a claim about set difference
between two canonical projections of RDF state, not a diff over some
incidental serialization.

## Why deltas want to be a group

Once you have "the difference between two states" as a first-class value,
you immediately want to do arithmetic with it: undo a delta, chain two
deltas into one, ask whether a chain of deltas nets to nothing. Group theory
is the right lens because a receipt chain is, mechanically, exactly a
sequence of composition operations that has to stay associative and
invertible or the whole audit trail becomes untrustworthy.

`Delta` supplies the three operations a group needs:

- **Identity** — `Delta::default()` (derived `Default`, crates/ggen/src/graph.rs:104),
  the empty additions/deletions pair, tested for via `is_empty()`
  (crates/ggen/src/graph.rs:216-220: `additions.is_empty() && deletions.is_empty()`).
- **Inverse** — `Delta::inverse()` swaps additions and deletions
  (crates/ggen/src/graph.rs:172-180). The doc comment states the intended law
  directly: "applying `self` then `self.inverse()` is a net no-op"
  (crates/ggen/src/graph.rs:172-174).
- **Composition** — `Delta::compose()` (crates/ggen/src/graph.rs:182-214) takes
  `self` (applied first) and `other` (applied second) and produces the single
  delta that has the same net effect. It is not naive concatenation: an
  addition in `self` that `other` immediately deletes must vanish from the
  composite, and symmetrically for a deletion in `self` undone by an addition
  in `other` (crates/ggen/src/graph.rs:184-186, implemented at
  crates/ggen/src/graph.rs:196-208 by filtering each side's additions/deletions
  against the other side's opposing set before unioning into `BTreeSet`s).

Whether this is *actually* a group — not just group-shaped — is an empirical
claim about the code, and the codebase tests it as one rather than asserting
it in prose. `crates/ggen/src/graph.rs:399-408` (`compose_with_inverse_is_empty_and_hashes_as_empty`)
checks `d.compose(&d.inverse())` is empty and hashes identically to the
identity element. `crates/ggen/src/graph.rs:410-414` (`inverse_is_an_involution`)
checks `d.inverse().inverse() == d`. `crates/ggen/src/graph.rs:416-425`
(`compose_cancels_crosswise_and_keeps_survivors_sorted`) checks that
crosswise cancellation actually fires and that a genuine survivor (`z`) is
retained. These are hand-picked examples, though — three fixed deltas prove
the algebra behaves correctly on three fixed inputs, nothing more.

The property test in `crates/ggen/tests/combinatorial_matrix.rs:378-410`
(`delta_laws`) is what turns "these three examples pass" into "this holds
across a swept space of RDF triple combinations." It generates two random
small graphs (`a_idx`, `b_idx`, each 0–8 triples drawn from a 4×2×4 term
space, crates/ggen/tests/combinatorial_matrix.rs:379-380), computes
`Delta::compute(&ga, &gb)` (crates/ggen/tests/combinatorial_matrix.rs:394),
applies it to `ga`, and asserts the resulting hash equals `gb`'s hash
(crates/ggen/tests/combinatorial_matrix.rs:395-400) — that is the
group-action law, "applying the delta between two states reaches the target
state," checked against the same `state_hash` the receipts actually sign.
It then asserts `d.compose(&d.inverse()).is_empty()` and
`d.inverse().compose(&d).is_empty()` (crates/ggen/tests/combinatorial_matrix.rs:401-409)
— both one-sided inverse laws, over however many cases proptest draws for
this run, not just the three curated ones in `graph.rs`. Running it locally:

```
$ cargo test -p ggen --test combinatorial_matrix delta_laws -- --nocapture
running 1 test
test delta_laws ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s
```

and the three hand-written unit tests in `graph.rs` itself:

```
$ cargo test -p ggen --lib graph:: -- --nocapture
running 4 tests
test graph::tests::empty_delta_is_empty ... ok
test graph::tests::inverse_is_an_involution ... ok
test graph::tests::compose_cancels_crosswise_and_keeps_survivors_sorted ... ok
test graph::tests::compose_with_inverse_is_empty_and_hashes_as_empty ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.00s
```

Neither run is proof in the mathematical sense — proptest samples a finite
number of cases per invocation, it does not exhaust the space — but it is
the strongest evidence a test suite can offer for a would-be algebraic law,
which is why the property test exists at all rather than resting on the
three fixed examples.

## Why the hash must be a homomorphism on the canonicalization quotient

The Chatman Equation's premise is that a receipt's hash *stands for* the
state it describes: if two graphs are "the same" for every purpose the
system cares about, their receipts must be bit-identical, and if they are
different, the hashes must (with overwhelming probability) differ too. That
requirement only makes sense relative to an equivalence relation — RDF has
no canonical serialization, so "the same graph" really means "the same graph
up to blank-node relabeling and triple ordering." `graph.rs`'s module doc
states this quotient explicitly: "graphs without blank nodes are sorted
lexicographically by their N-Quads string form; graphs with blank nodes go
through a bounded color-refinement pass ... and blank nodes are relabelled
`c14n{i}` in signature order so isomorphic graphs hash identically regardless
of blank-node labels" (crates/ggen/src/graph.rs:7-11). `state_hash()` then
hashes the join of those canonical lines (crates/ggen/src/graph.rs:95-99),
and `Delta::hash()` hashes the same canonical strings, sorted, with `+`/`-`
prefixes (crates/ggen/src/graph.rs:222-240).

Call the canonicalization function `c` (RDF graph → canonical string form)
and the hash function `h`. The system needs `h ∘ c` to be well-defined on
the quotient by isomorphism — i.e. `c` must actually collapse every
insertion-order and blank-node-labeling variant of "the same graph" to one
representative, or `h` will assign different hashes to graphs a human (and
every downstream SPARQL query) would call identical. This is exactly
*homomorphism* in the algebraic sense relevant here: the map from the
delta group to hash-space must respect structure, so that composing two
deltas and hashing equals some deterministic function of hashing them
separately — at minimum, it must at least send the group identity (the empty
delta) to a fixed, reproducible value, which is what
`compose_with_inverse_is_empty_and_hashes_as_empty` (crates/ggen/src/graph.rs:399-408)
actually checks: `net.hash() == Delta::default().hash()`. If canonicalization
were not a true quotient map — if it left some insertion-order or
blank-node-naming residue in the string form — the hash would stop being a
function of graph *state* and would silently become a function of graph
*history*, which is precisely the failure this project treats as
unacceptable: a receipt is supposed to prove what the state *is*, and if the
proof depends on incidental construction order, it proves nothing
reproducible. The insertion-order property test
(`crates/ggen/tests/combinatorial_matrix.rs:340-373`, unnamed inline test
using a deterministic Fisher–Yates shuffle) exists for exactly this reason:
it builds the same triples in two different insertion orders and asserts
`state_hash()` agrees (crates/ggen/tests/combinatorial_matrix.rs:368-372).

## The GROUP_CONCAT incident as a projection that was not a function

This session's build surfaced a concrete case of a projection failing to be
well-defined, in the ggen-generation layer that sits above `graph.rs`
(the SPARQL rules in `ggen.toml` that generate `crates/ggen/src/verbs/*.rs`
from the ontology). Several rules aggregate a command's flags with
`GROUP_CONCAT(?flag; separator=",")`
(e.g. /Users/sac/praxis/ggen.toml:201, :231, :261, :291). `GROUP_CONCAT` over
a `GROUP BY` has no defined row order unless one is imposed — SPARQL
aggregation, like SQL's, iterates the grouped rows in whatever order the
query engine's internal plan produces them, which is a legitimate
implementation freedom the SPARQL spec grants and oxigraph is free to
exercise however its optimizer prefers. Naively, that makes
`?noun ?verb ?handler ?comment -> flags string` *not a function*: the same
input tuple can legally project to `"a,b,c"` on one evaluation and `"c,a,b"`
on another, and a code generator that embeds that string in a `.rs` file
would then produce a different byte-for-byte output — and a different
receipt hash — for a completely unchanged ontology, purely as an artifact of
query-plan nondeterminism.

The fix visible in `ggen.toml` is to force a deterministic row order before
the aggregation ever sees the rows, using a nested `SELECT` with its own
`ORDER BY ?flag` (/Users/sac/praxis/ggen.toml:203-217 for the `sync` rule,
structurally identical at :233-247 for `graph`):

```
SELECT ?noun ?verb ?handler ?comment (GROUP_CONCAT(?flag; separator=",") AS ?flags)
WHERE {
  {
    SELECT ?cmd ?noun ?verb ?handler ?comment ?flag
    WHERE {
      ...
      OPTIONAL { ?cmd praxis:flag ?flag }
      FILTER(?noun = "sync")
    }
    ORDER BY ?flag
  }
}
GROUP BY ?noun ?verb ?handler ?comment
ORDER BY ?verb
```

The inner `ORDER BY ?flag` pins the sequence the outer `GROUP_CONCAT` walks,
so the aggregation becomes a genuine function of the ontology's flag set
rather than of query-plan happenstance; the outer `ORDER BY ?verb` does the
analogous job for row order across the whole result set, which matters
because `graph.rs`'s own canonicalization strategy assumes there is a
single, sortable, deterministic string form to hash against
(crates/ggen/src/graph.rs:7-11, :262-268) — an unordered aggregate anywhere
upstream of that hash reintroduces exactly the nondeterminism the
`c14n{i}` / lexicographic-sort machinery in `graph.rs` was built to
eliminate. It is the same failure mode at a different layer: `graph.rs`
solves it for blank-node identity and quad ordering inside the RDF store;
the `ORDER BY` subqueries in `ggen.toml` solve the same class of problem one
layer up, where a SPARQL aggregate function is the thing that must be made
into an actual mathematical function before its output is safe to hash.

## What this buys, and what it does not

Treating deltas as a group gives praxis composable, invertible transition
receipts: two applied deltas can be folded into one without replaying
intermediate states, and any receipt chain can be walked backward by
composing inverses, because `compose` and `inverse` are proven (by property
test, not just by construction) to interact the way group axioms require.
It does not, by itself, protect against a non-deterministic *input* to the
algebra — if the canonicalization step or an upstream SPARQL projection
hands `Delta::compute` two different string sets for what a human would
call "the same" query result, the group laws still hold perfectly (compose,
inverse, and hash all behave consistently on whatever strings they are
given) while the receipts built on top silently stop meaning what everyone
assumes they mean. The algebra is only as trustworthy as the canonicalization
and query-ordering discipline feeding it — which is why both live under the
same invariant, and why a fix to one (the `ORDER BY` subquery) is really
enforcing the same law as the other (the `c14n{i}` blank-node canonicalization
and lexicographic quad sort in `graph.rs`): a projection from state to string
must be a function before any hash built on top of it can mean anything.
