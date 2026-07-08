# Datafrog Crate Audit: Architecture & Insights for Praxis-GraphLaw

## A. Architecture Overview

Datafrog is a **lightweight, embeddable Datalog engine** optimized for integration into other Rust programs. It explicitly rejects heavy abstractions and runtime machinery, instead using simple data structures (sorted vectors) and straightforward iteration control exposed to the user.

### Core Representation

**Relations** are static, sorted vectors of distinct tuples (`Relation<T>` = `Vec<T>` where `T: Ord`). Each relation maintains a single invariant: tuples are **sorted and deduplicated**.

**Variables** are the dynamic mutable counterparts. They track tuples across three lifecycle stages:
1. **`to_add`**: Relations queued for introduction (awaiting next `changed()` call)
2. **`recent`**: Tuples newly added and available for one iteration cycle
3. **`stable`**: Accumulated tuples from all prior iterations (kept for deduplication)

The stable section uses a **logarithmic batching strategy**: tuples are stored in a `Vec<Relation<Tuple>>` where each batch is roughly double the size of the previous. This ensures O(log n) batches while avoiding frequent full merges.

### Execution Model

**No built-in scheduler.** The user explicitly writes a loop:
```rust
while iteration.changed() {
    // Apply rules: variable.from_join(...), variable.from_map(...), etc.
}
```

The `Iteration` context tracks all registered variables. On each `changed()` call:
1. Recent tuples are merged into stable (with opportunistic batch consolidation)
2. Pending (to_add) tuples are filtered against stable to remove duplicates
3. If any variable has non-empty recent, return true; else false

This implements **eager materialization with delta semantics**: tuples flow through the system once per iteration, and all derivations accumulate.

### Variable Lifecycle Example

```
Initial: insert(vec![T1, T2])
  to_add = [T1, T2]
  recent = []
  stable = []

After changed() #1:
  to_add = []
  recent = [T1, T2]  <-- used in iteration 1
  stable = [T1, T2]

After changed() #2 (no new tuples added):
  recent = []
  stable = [T1, T2]
  Returns false (iteration terminates)
```

## B. Core Algorithms

### Join: Gallop-Based Merge

Datafrog implements **binary sorted-list joins** using the **gallop search** (`join::gallop` in `src/join.rs`), an exponential binary search strategy:

```rust
fn gallop<T>(mut slice: &[T], mut cmp: impl FnMut(&T) -> bool) -> &[T] {
    if !slice.is_empty() && cmp(&slice[0]) {
        let mut step = 1;
        while step < slice.len() && cmp(&slice[step]) {
            slice = &slice[step..];
            step <<= 1;  // Exponential: 1, 2, 4, 8, ...
        }
        step >>= 1;
        while step > 0 {
            if step < slice.len() && cmp(&slice[step]) {
                slice = &slice[step..];
            }
            step >>= 1;  // Binary search phase
        }
        slice = &slice[1..];
    }
    slice
}
```

**Key insight**: gallop is optimal when the result set is clustered (many consecutive items match), but degrades to O(log n) per advance. This is ideal for Datalog where keys often cluster by stratum or access pattern.

### Join Delta Strategy

When joining two variables, datafrog performs **semi-naive evaluation** without explicit rule ordering:
```
join_delta(input1: Variable, input2: Variable) produces:
  1. join(recent1, stable2)  -- new facts from input1 against all input2
  2. join(stable1, recent2)  -- all input1 against new facts from input2
  3. join(recent1, recent2)  -- new facts from both inputs
```

This ensures **all new derivations** are caught without reconsidering old facts from both sides. The contract: at least one input must be a Variable (not both Relations) because Relations have no "recent" state.

### Leapjoin: Treefrog Multi-Way Joins

For N-way joins with multiple relations, datafrog uses **treefrog leapjoin** (McSherry's technique, `src/treefrog.rs`). Instead of materializing intermediate Cartesian products:

1. **Estimate counts** for each leaper (relation/filter)
2. **Propose from the smallest** leaper first
3. **Intersect** remaining leapers with proposed values

```rust
pub fn leapjoin<Tuple, Val, Result>(
    source: &[Tuple],
    mut leapers: impl Leapers,
    mut logic: impl FnMut(&Tuple, &Val) -> Result,
) -> Relation<Result> {
    for tuple in source {
        // Find leaper with minimum proposed count
        let min_index = leapers.for_each_count(tuple, ...)
                               .min_by_key(|count| count)
                               .index;
        // Leaper proposes values; others intersect
        leapers.propose(tuple, min_index, &mut values);
        leapers.intersect(tuple, min_index, &mut values);
        // Apply logic to all proposed values
        for val in values.drain(..) {
            result.push(logic(&tuple, &val));
        }
    }
}
```

**Leapers** are composable join operators:
- **ExtendWith**: propose values from a relation keyed by tuple
- **ExtendAnti**: propose values from complement of a relation (anti-join)
- **FilterWith/FilterAnti**: restrict to tuples present/absent in relation
- **PrefixFilter**: arbitrary predicate on tuple
- **ValueFilter**: arbitrary predicate on proposed value

Treefrog avoids materializing the cross-product of all relations, making multi-way joins much more efficient than sequential binary joins.

### Rule Ordering & Stratification

**Datafrog does NOT implement automatic stratification.** Users must:
1. Order rules manually (positive rules first, then negation-dependent rules)
2. OR handle stratification externally (as praxis-graphlaw does via Bellman-Ford)
3. OR rely on `Variable::variable_indistinct()` (non-terminating guarantee dropped)

## C. Data Structures

### Relation<T>

```rust
pub struct Relation<Tuple> {
    pub elements: Vec<Tuple>,  // Always sorted, deduplicated
}
```

Operations:
- **`merge(other)`**: O(|self| + |other|) merge-sort with deduplication
- **`from_join(input1, input2, logic)`**: binary sorted join
- **`from_antijoin(input1, input2, logic)`**: anti-join
- **`from_map(input, logic)`**: map operation
- **`from_leapjoin(source, leapers, logic)`**: treefrog multi-way join

### Variable<T>

```rust
pub struct Variable<Tuple> {
    pub distinct: bool,
    pub stable: Rc<RefCell<Vec<Relation<Tuple>>>>,  // Log-structured batches
    pub recent: Rc<RefCell<Relation<Tuple>>>,       // Current iteration
    pub to_add: Rc<RefCell<Vec<Relation<Tuple>>>>,  // Pending insertion
}
```

Key methods:
- **`insert(relation)`**: queue relation for next iteration
- **`extend(iterator)`**: queue iterator-collected relation
- **`from_join(var1, var2/rel, logic)`**: delta join + insert
- **`from_leapjoin(source, leapers, logic)`**: treefrog + insert
- **`complete()`**: consume variable, flatten batches, return final Relation
- **`changed()`**: advance lifecycle (called by Iteration context)

**Ref-counting via Rc<RefCell<_>>** allows Variables to be cloned and shared across multiple rule scopes without lifetime constraints, at the cost of runtime borrow-check overhead.

## D. Benchmarks & Performance

### Provided Benchmarks

Datafrog includes property-based tests (via proptest) for:
- **Reachability**: transitive closure over 100x100 grid of edges
- **Sum-join**: multi-way join reducing via closure logic
- **Leapfrog vs. sequential joins**: validates equivalence
- **Filter operations**: intersection, set-minus via filters

Test generators use edge counts ~100-500, node ranges 0-100. **No large-scale benchmarks** (e.g., 10^5+ tuples, deep graphs).

### Performance Characteristics

**Strengths**:
- Gallop search minimizes comparisons on clustered data
- Logarithmic batch structure avoids O(n) stable-list merges per iteration
- No allocations in inner loops (reuses Vec with pre-allocated capacity)
- Sorted representation enables cache-friendly iteration

**Weaknesses**:
- Rc<RefCell<_>> adds indirection and per-access borrow overhead
- No indexing beyond sort order (no hash tables, no B-trees)
- Requires deduplication on every merge (O(n) pass)
- `from_join` with two Variables re-examines all stable tuples each iteration (though efficiently via gallop)

### Measurement Approach

Datafrog provides `Iteration::record_stats_to(writer)` for CSV logging:
```csv
Variable,Round,Stable count,Recent count
reachable,1,10,50
reachable,2,60,20
reachable,3,80,0  # Terminated
```

This enables **per-variable progress tracking** but requires instrumentation in user code (no built-in profiler).

## E. Design Constraints & Limitations

### Explicitly NOT Supported

1. **Negation as Failure (NAF)**
   - No implicit stratification or well-founded semantics
   - Users must manually order rules or use external stratification
   - Praxis-graphlaw implements this via explicit Bellman-Ford analysis

2. **Aggregation**
   - No built-in sum, count, min, max over variable groups
   - Must be expressed via explicit accumulation rules
   - Praxis-graphlaw has explicit Aggregate support with stratification

3. **Choice Points / Non-Determinism**
   - All rules are deterministic (no choice logic, no disjunction in heads)
   - Head of a rule has exactly one triple pattern

4. **Query Execution**
   - No interactive query language (SQL, SPARQL, Datalog syntax)
   - Users hand-write Rust code to define rules
   - Must manually extract results via `variable.complete()`

### Pragmatic Constraints

5. **Sorting Requirement**
   - All tuples must implement `Ord`
   - Non-comparable types (e.g., floats with NaN) cannot be used
   - Forces users to normalize keys (e.g., intern strings)

6. **Termination Guarantees**
   - Only guaranteed if `distinct=true` for all variables
   - With `distinct=false`, loops may not terminate (used for performance trade-offs)
   - No automatic cycle detection or stratification of recursive predicates

7. **Stratification Cycles**
   - Rules forming cycles through negation/aggregation cause non-termination
   - No static analyzer warns about this
   - Example: `p(X) :- not q(X)` and `q(X) :- not p(X)` produces oscillation

### Design Philosophy

From Frank McSherry's blog: **"Little enough in the way of someone coming to understand how it all works."**

Constraints are features:
- No magic scheduling or optimizer (user controls iteration)
- No hidden state or complex runtime (everything in Rc<RefCell<_>>)
- Single representation (sorted vectors) for all data
- Simplicity for **educational clarity and embeddability**, not maximal performance

## F. High-Signal Learnings for Praxis (Post-PROJ-401)

### 1. Gallop Algorithm for Indexed Joins

**Applicability**: After PROJ-401 symbol interning, praxis-graphlaw's join operations could benefit from gallop search on sorted symbol lists.

**Current state**: Praxis likely uses linear search or no search in join paths. Gallop is O(log n) but optimal on clustered keys (which RDF triples are, grouped by subject/predicate).

**Action**: When joining large triple indexes, profile whether switching from sequential search to gallop in `tripleindex.rs` improves join latency.

### 2. Logarithmic Batch Structure for Deduplication

**Applicability**: Praxis's deduplication of derived triples could adopt datafrog's log-structured batching rather than full merges.

**Current state**: Praxis likely deduplicates derived triples against all prior derivations on each rule iteration. At large scale (millions of triples), this is O(n^2) in aggregate.

**Action**: For PROJ-401 and beyond, consider batching derived triples and only merging/deduplicating against batches of similar size. Cuts deduplication cost from O(n^2) to O(n log n).

### 3. Delta Iteration Formalization

**Applicability**: Praxis's semi-naive evaluation could explicitly structure rule application as datafrog does: always work with (recent, stable) pairs.

**Current state**: Praxis likely applies rules to all accumulated facts, not just newly derived ones.

**Action**: Formalize the rule engine to track "recent" vs. "stable" facts per variable/predicate. Reduces redundant rule applications.

### 4. Multi-Way Join via Leapjoin

**Applicability**: Praxis rules with 3+ triple patterns in the body could use treefrog leapjoin instead of sequential binary joins.

**Example rule**:
```
?derivedTriple a :Result :-
  ?subject ?predicate ?object ,
  ?predicate rdfs:range ?range ,
  ?object a ?range .
```

Current: join((subject, predicate, object), (predicate, range)) → intermediate, then join(intermediate, (object, range))

With leapjoin: propose from smallest set, intersect others; no intermediate materialization.

**Action**: For hook bodies with 3+ bound patterns, emit leapjoin instead of sequential joins. Measure improvement on knowledge_hooks_e2e.rs.

### 5. Interned Symbol Sorting for Cache Efficiency

**Applicability**: After PROJ-401 symbol interning, symbols become u64 integers. Gallop search and sorted vectors operate on cache-friendly integers, not heap-allocated strings.

**Current state**: Praxis likely sorts by term objects (pointers/strings), poor cache locality.

**Action**: Once PROJ-401 lands, re-sort internals to use symbol u64 values as join keys. Measure cache-miss rate and throughput.

### 6. Hook Eligibility Sets via Closure Analysis

**Applicability**: Datafrog doesn't compute rule dependencies, but praxis's Knowledge Hooks (PROJ-307) require knowing which rules could be triggered by new facts.

**Current state**: Hooks.rs likely computes eligible rules via full scan.

**Action**: Adopt datafrog's dependency-tracking pattern (track which predicates appear in rule heads/bodies) to precompute eligibility sets. Avoids O(n) lookup for hook dispatch.

## G. Benchmark Comparator Suitability

### Can Datafrog Be a Reference for Praxis Performance?

**Partially yes**, with caveats.

**Suitable comparisons**:
- Basic join efficiency (sorted list join speed vs. praxis's tripleindex joins)
- Deduplication cost (merge + sort + dedup overhead)
- Rule iteration count (how many passes to reach fixpoint)

**Not suitable**:
- RDF-specific reasoning (RDFS, OWL reasoning rules) — datafrog has no type hierarchy
- Receipt/replay guarantees — datafrog does not track provenance
- Hook semantics — datafrog has no notion of "when-derived" callbacks
- Negation performance — datafrog bans it; praxis has stratified negation

### Benchmark Plan

To use datafrog as a reference:
1. **Translate 1-2 example problems** from praxis test suite to datafrog (e.g., closure rules, property chains)
2. **Run both on same data** (e.g., 10k triple closure)
3. **Measure**: iterations to fixpoint, memory peak, wall-clock time per rule application
4. **Interpret**: if datafrog is faster, investigate whether gallop/batching/leapjoin improvements are applicable to praxis

Example:
```rust
// Datafrog version: transitive closure of :derivedFrom
let triples: Relation<(TripleId, TripleId)> = ...;
let mut iter = Iteration::new();
let derived = iter.variable("derivedFrom");
derived.insert(triples);
while iter.changed() {
    derived.from_join(&derived, &triples, |&k, &v1, &v2| (v1, v2));
}
```

Compare praxis hooks implementation on same rule set.

## H. Future Backend Feasibility

### Could Datafrog Be an Optional Praxis Backend?

**Short answer**: Not without substantial changes to both systems.

**Barriers**:

1. **Receipt/Provenance Model**
   - Datafrog: no receipt tracking
   - Praxis: all derivations must have BLAKE3 receipts (Invariant #2)
   - Datafrog would need to track (derivation, receipt) pairs through rules

2. **Knowledge Hooks**
   - Datafrog: static rule set, no "when-derived" callbacks
   - Praxis: hooks fire when specific predicates derive facts
   - Would need to instrument leapjoin to emit hook events

3. **Stratification**
   - Datafrog: user-managed or external
   - Praxis: automatic Bellman-Ford-based stratification
   - Datafrog provides the join/iterate primitives; praxis adds stratification wrapper

4. **Negation & Aggregation**
   - Datafrog: explicitly banned
   - Praxis: supported via stratification
   - Datafrog engine would need extensions

5. **Closed Vocabularies**
   - Praxis (Invariant #4): unknown predicates refused by name
   - Datafrog: no vocabulary tracking
   - Would need validation layer outside datafrog

### Potential Hybrid Approach

**Instead of full backend replacement:**

1. **Use datafrog's join/merge algorithms**: Port gallop, leapjoin to praxis-graphlaw
2. **Keep praxis's rule engine**: Stratification, receipt tracking, hooks, vocabulary checks
3. **Benchmark comparison**: Measure datafrog on pure Datalog subset of praxis rules

This avoids rewriting praxis's core logic while gaining algorithmic improvements.

### Contract for Optional Backend

**If praxis were to support datafrog-based evaluation for specific workloads:**

```
Scope: Positive-only Datalog rules (no negation, no aggregation)
Input: Stratified rule set + initial facts + (optional) hook triggers
Output: Final facts + proof receipts + hook events

Datafrog would:
  1. Enforce @Ord on facts (u64 symbol IDs)
  2. Track derivation path for receipts
  3. Emit hook events on predicate derivation
  4. Return facts + receipts
Praxis would:
  1. Validate rules pre-execution
  2. Handle stratification
  3. Integrate results into knowledge graph
  4. Dispatch hook callbacks
```

---

## Summary

**Datafrog excels at**:
- Simplicity and understandability
- Efficient sorted-list joins (gallop, leapjoin)
- Logarithmic deduplication via batching
- Embeddability in other systems

**Praxis can benefit from**:
1. Gallop algorithm for tripleindex join paths
2. Log-structured batching in deduplication
3. Formalized delta iteration (recent vs. stable)
4. Leapjoin for multi-way rule bodies
5. Closure analysis for hook eligibility

**Unsuitable as full backend** due to receipt tracking, hooks, stratification, and vocabulary requirements. **Suitable as algorithmic reference** for join and merge optimizations.
