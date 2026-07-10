# rwai-bench — Fortune-5 Autonomic Recursive Workflow benchmark methodology

## What is measured (MEASURED_CNG_RESULT)

Every number in the benchmark results comes from executing the REAL cng
manufacture chain — no mocks, no bypass, no simulated success:

`.ttl` import (oxigraph Turtle admission) → SPARQL classification →
structural fragment merge → bcinr-pddl grounding + bounded-BFS planning →
POWL v2 projection → provenance serialization → SPARQL shape validation →
bcinr-powl compile admission + branchless scheduler execution with per-tick
order-conformance → BLAKE3 receipts → replay verification.

Commands (`cng` built with `--features bench`, release profile):

```bash
cng benchmark generate --out DIR --workers N --depth 5   # deterministic corpus
cng benchmark run --dir DIR                              # measured execution
cng benchmark verify --dir DIR                           # independent replay
```

## Worker representation

Workers are never counters. `generate` materializes every represented worker
as RDF facts (`ex:wN a ex:Worker ; ex:role … ; ex:department … ;
ex:standing ex:admitted`) in partitioned roster `.ttl` artifacts (5,000
workers per partition). `run` parses every partition through oxigraph
(INPUT_TRIPLES is the parsed triple count) and executes a real role-inference
SPARQL SELECT per partition. Every workload artifact set is attributed to a
worker IRI drawn from the roster.

## Workload

12 enterprise categories (email routing, calendar change, invoice matching,
purchase-order approval, expense review, HR notice, customer request,
logistics event, compliance check, document request, software delivery,
admission request). Each artifact set = 2 domain-fragment `.ttl` + 2
problem-fragment `.ttl` files rendered from committed templates with seeded
names (splitmix64) — many-to-one manufacture per workflow, structurally
merged by the pipeline. ~1% of sets omit the closing problem fragment: these
refuse `CNG_R03` and are counted as bounded human admissions, never silent
fallbacks.

## Recursion

An 8-ary tree of machine-generated artifact sets, 5 levels deep
(1+8+64+512+4096 = 4,681 workflow nodes; 4,096 leaf workflows at level 5 =
8⁴ nodes hosting 8⁵ = 32,768 level-5 activities). Attachment is derived from
`ex:attachesWorkflow` triples in each node's admitted graph via SPARQL — the
runner discovers children from the graph, never from directory listing, and
no child workflow is hand-authored. Reported separately: logical workflow
nodes (declared), materialized POWL nodes (actually manufactured), executed
transitions (ops fired on the bcinr-powl scheduler), validated transitions
(shape-valid), receipted transitions (BLAKE3).

## Timing and resources

Wall-clock timing exists only in the benchmark harness, never in the
manufacture path (digests contain no time). Latency percentiles are computed
over per-set end-to-end manufacture times; throughput uses the manufacture
window only. Peak RSS and CPU time come from `/usr/bin/time -l` wrappers.
Network traffic is zero by construction (single-host, in-process). BLAKE3
throughput is measured by hashing a 64 MiB deterministic buffer.
WASM execution time is not exercised in this benchmark: the measured path is
the native engine; WASM parity of the logic-tick substrate is tracked
separately in the Chatman Engine workstream and is not claimed here.

## Scale strategy

Roster partitions materialize EVERY represented worker at EVERY scale (5M
workers = 1,000 partitions = 5M worker fact-sets = 20M roster triples
parsed). Workload sets scale at workers/100 (capped at 50,000 sets = 200,000
artifact files) — the per-set cost distribution is measured directly, so
daily-volume totals for larger workloads are arithmetic on measured rates
and are labeled as such wherever shown; they are never presented as
measurements.

## LLM comparison (MODELED_LLM_COMPARISON)

The LLM-agent comparison is a MODEL, not a benchmark result, and is labeled
as such everywhere. Declared assumptions (documented in the generated
report): an equivalent agent architecture spends ≥3 LLM calls per workflow
step (plan/act/verify), ~2,000 input + 500 output tokens per call, at
published API prices ($3/M input, $15/M output — Claude Sonnet class);
recursion multiplies calls by the same 8ⁿ node counts the deterministic
engine executes. RWAI cost converts measured CPU-seconds to dollars at
$0.05/vCPU-hour (on-demand cloud rate). Both models are stated per million
workflows and annually for a 5M-worker enterprise at 8 workflows/worker/day
× 250 days.

## Integrity

Determinism: same corpus → byte-identical POWL digests (replay + independent
`benchmark verify` pass). Anti-hardcoding: seeded names propagate into every
generated POWL; changing the seed changes every digest. Refusals are typed
(`CNG_R01`–`CNG_R10`). The no-inline-TTL guard covers the benchmark code:
templates and generated corpora are `.ttl` artifacts; generated roster/
workload Turtle has the same status as serializer output and is consumed
only back through oxigraph.
