# synth/v1 — the one-page wire format

Everything a foreign agent needs to drive the synthesis pipeline
(`synth run` / `synth solve` on the CLI; same payload over the membrane).
The 8-caps make the schema small enough to specify completely here.

## Terms and atoms

A **term** is a string: `"?N"` is variable *N* (N < 8); anything else is an
interned constant. An **atom** is `["pred", ["t1", ...]]` — arity ≤ 8.
Facts must be ground (no variables).

## The payload

```json
{
  "synth": "v1",
  "facts":        [["raw", ["o1"]]],
  "rules":        [{"head": ["ready", ["?0"]],
                    "body": [["raw", ["?0"]]],
                    "neg":  []}],
  "capabilities": [{"name": "work", "params": 1,
                    "pre": [["ready", ["?0"]]],
                    "add": [["done", ["?0"]]],
                    "del": [],
                    "cost": 1}],
  "goal":         [["done", ["o1"]]],
  "horizon":      4,
  "constraints":  [],
  "solver":       "solver8"
}
```

| field | rule |
|---|---|
| `facts` | ground atoms; ≤ 10⁸ total tuples |
| `rules` | stratified Horn; every head/negated variable bound by a positive body atom; ≤ 8 vars |
| `capabilities` | effect variables bound by preconditions; declared, never ordered by you |
| `goal` | patterns (variables allowed) |
| `horizon` | ≤ 16 steps |
| `constraints` | ≤ 64 (8×8); see kinds below |
| `solver` | `"solver8"` (propagating, certifies unsat) or `"brute"` (the differential oracle) |

## The eight constraint kinds

```json
{"kind": "Before",     "a": "x", "b": "y"}   // every x before every y
{"kind": "After",      "a": "x", "b": "y"}   // sugar for Before(y,x)
{"kind": "NotLater",   "a": "x", "k": 2}     // x at step < k
{"kind": "NotEarlier", "a": "x", "k": 3}     // x at step >= k
{"kind": "Excludes",   "a": "x", "b": "y"}   // never both
{"kind": "Requires",   "a": "x", "b": "y"}   // x present => y present
{"kind": "AtMost",     "a": "x", "n": 1}     // occurrence cap
{"kind": "Budget",     "max": 10}            // total cost bound
```

## What comes back

`synth solve` → `{"status": "solved", "saturation": {…, "fixpoint_hash"},
"plan": {"steps": [{"capability", "binding"}], "cost", "receipt":
{"nodes_explored", "problem_hash", "plan_hash"}}}` — the solver discovered
both the **order** and the **parameter bindings**; you declared, it derived.

`synth run` additionally executes the plan as a content-addressed DAG
(BLAKE3 per-node outputs, memoized replay) and admits it through six
refinements — returning one `SynthesisReceipt` whose `chain` commits the
entire run.

Refusals are results, not errors:

```json
{"status": "refused",
 "refusal": {"UnsatProof": {
   "detail": "mandatory capability 'gate-check' requires 'authorization', which the initial state lacks and no capability produces",
   "core": ["MissingFact(authorization)"],
   "replayed": false}}}
```

An `UnsatProof` core is a **certificate**: re-propagating the core alone
reproduces the impossibility — verify it without searching. Dead ends are
shared, not re-derived.

## Conformance

The lawobject fixture: the five capabilities in
`crates/praxis-synthesis/tests/common/mod.rs` under `solver8`, horizon 6,
must yield exactly `supply-evidence → clear-obligations → judge → admit →
receipt`, cost 5, all bound to `o1`. If your reimplementation disagrees,
it is wrong — or you have found a bug worth a receipt.
