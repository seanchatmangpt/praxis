# cng Cheatsheet

Quick-reference command index for the `cng` noun-verb CLI (`crates/cng/src/main.rs`).
17 verbs across 5 nouns: `plan`, `workflow`, `benchmark`, `engine`, `evidence`. For the
product thesis, ontology mapping, and refusal algebra, see `crates/cng/README.md`; this
doc is only the command surface, one line per verb, with a real invocation you can
copy-paste.

Every example below uses fixtures that already exist in this repo — no invented paths.
`plan`/`workflow` examples run against `crates/cng/plans/joseph/` (13-phase, 26-file
famine-management plan). `benchmark`/`engine`/`evidence` examples run against the `bench`
feature (`--features bench`) and use fresh `--out`/`--root` directories since those verbs
generate their own corpora. `plan decompose` uses an on-disk negative fixture
(`tests/fixtures/decomp-negative/`) since no positive fixture ships on disk (positive
decomposition cases are template-rendered at test time, not fixture files).

## Quick reference

| Noun | Verb | One-line |
|------|------|----------|
| `plan` | `import` | List/validate importable PDDL Turtle artifacts in a directory |
| `plan` | `admit` | Parse + structurally merge an artifact set (no planning) |
| `plan` | `generate` | Merge + plan once; print the BLAKE3 plan id and step labels |
| `plan` | `decompose` | No-LLM multi-actor goal decomposition (`bench` feature) |
| `workflow` | `project` | Project the combined plan into POWL v2 Turtle (inline, no file write) |
| `workflow` | `export` | Full manufacture; write + shape-validate one POWL v2 Turtle artifact |
| `workflow` | `inspect` | Parse + shape-validate an existing POWL v2 Turtle file |
| `workflow` | `doctor` | Report toolchain surface + run cheap self-checks |
| `workflow` | `validate` | Run the projected workflow on the bcinr-powl runtime (`runner`) |
| `workflow` | `evidence` | Full proof chain + release manifest (`runner` feature) |
| `benchmark` | `generate` | Generate a Fortune-5-scale worker/workload corpus (`bench`) |
| `benchmark` | `run` | Admit, manufacture, validate, replay a benchmark corpus (`bench`) |
| `benchmark` | `workday` | Single-operator deterministic logical-tick day (`bench`) |
| `benchmark` | `verify` | Independent replay/re-validation of recorded digests (`bench`) |
| `engine` | `serve` | Run one Chatman Engine worker's receipted poll loop (`bench`) |
| `engine` | `resume` | Resume a worker after crash/kill; verify receipt-chain tail (`bench`) |
| `evidence` | `replay` | Independent auditor replay from a self-contained bundle (`bench`) |

`runner` is a default feature (needs the nightly toolchain pinned by this workspace).
`bench` is off by default; build/run with `--features bench` (pulls in
`praxis-graphlaw`, `wasm4pm-cognition`, `bcinr-pddl`).

## Global flags

Every verb accepts these (from `clap-noun-verb`, not from the verb's own parameters):

| Flag | Effect |
|------|--------|
| `--format <fmt>` | `json` (default), `json-pretty`, `yaml`, `table`, `plain`, `tsv`, `quiet` |
| `--select <query>` | Project nested JSON output (JSONPath / key selection / JMESPath) |
| `--introspect` | Print every verb's JSON-Schema signature (LLM tool-calling manifest) |
| `--structured-errors` | Emit errors as `StructuredError` JSON, not a plain refusal string |
| `--autonomic` | Enable autonomic mode (implies `--structured-errors`) |
| `-h, --help` / `-V, --version` | Standard clap help/version |

`cng --introspect` is the fastest way to get every verb's exact parameter names and
types without opening `main.rs` — worth reaching for before this doc goes stale.

## `plan` — import, admit, generate one combined plan

```bash
just cng-run plan import --dir crates/cng/plans/joseph
just cng-run plan admit --dir crates/cng/plans/joseph
just cng-run plan generate --dir crates/cng/plans/joseph
```

`plan decompose` needs the `bench` feature and two raw `.pddl` files (not the Turtle
convention the other `plan`/`workflow` verbs use):

```bash
just cng-bench plan decompose \
  --domain crates/cng/tests/fixtures/decomp-negative/actor-lacks-capability.domain.pddl \
  --problem crates/cng/tests/fixtures/decomp-negative/actor-lacks-capability.problem.pddl \
  --out target/agent-<name>/decompose-demo
```

This particular fixture is a negative case (`approved(?d)`'s only achiever needs a
capability fact no action ever grants) — it demonstrates the typed-refusal path
(`CNG_R05 UnsupportedConstruct`, `NoAdmissibleDecomposition` outcome), not a
successful multi-actor split. Swap in your own admitted `.pddl` pair for a `Selected`
outcome.

## `workflow` — project, export, inspect, validate, evidence

```bash
just cng-run workflow project --dir crates/cng/plans/joseph
just cng-run workflow export --dir crates/cng/plans/joseph \
  --out target/agent-<name>/joseph.powl.ttl
just cng-run workflow inspect --file target/agent-<name>/joseph.powl.ttl
just cng-run workflow doctor
just cng-run workflow validate --dir crates/cng/plans/joseph
just cng-run workflow evidence --dir crates/cng/plans/joseph \
  --out target/agent-<name>/joseph.powl.ttl
```

`export` and `evidence` also accept `--base-iri <iri>` (default
`urn:chatman:powl:cng`) and `--derived-from <iri>` (adds `prov:wasDerivedFrom` +
requires `powl2:derivedFrom` on shape-validation); `evidence` additionally accepts
`--seed <str>` (echoed into the manifest as `pddl_fixture_seed`, not consumed
otherwise). Live-verified output on `plans/joseph`: 20 activity leaves, 190 `precedes`
pairs, `validated=true conformant=true executed_ops=20`.

## `benchmark` — Fortune-5-scale corpus generate/run/workday/verify

```bash
just cng-bench-build   # release build once (bench feature pulls in LTO-heavy deps)

just cng-bench benchmark generate --out target/agent-<name>/corpus --workers 5000000
just cng-bench benchmark run --dir target/agent-<name>/corpus
just cng-bench-verify target/agent-<name>/corpus   # re-verifies the SAME corpus dir

# workday is a separate, standalone single-operator benchmark (own --out dir):
just cng-workday --out target/agent-<name>/workday --seed 42 --ticks 32
```

`benchmark generate` flags: `--out` `--workers` (required); `--sets`, `--depth`
(default 5), `--seed` (default 42), `--refusal-per-mille` (default 10) all optional.
`benchmark run` flags: `--dir` (required); `--threads` (default: available
parallelism), `--replay-per-mille` (default 20), `--queries-dir` optional. `benchmark
workday` flags: `--out` (required); `--seed` (default 42), `--ticks` (default 32),
`--refusal-per-mille` (default 125). `benchmark verify` flags: `--dir` (required);
`--sample-every` (default 50), `--threads` (default: available parallelism).

## `engine` — one worker's poll loop, serve/resume

```bash
just cng-engine-serve --root target/agent-<name>/engines --engine-id H --seed 42 --max-polls 64
just cng-engine-resume --root target/agent-<name>/engines --engine-id H --seed 42 --max-polls 64
```

Both take `--root` `--engine-id` (required); `--seed` (default 42), `--max-polls`
(default 64), `--poll-wait-ms` optional (the inter-poll sleep; never enters a digest).
`resume` reloads the durable dispatch ledger tail and refuses `CNG_R11` on a torn tail
before continuing the same serve loop.

## `evidence` — independent auditor replay

```bash
# `benchmark run`'s corpus dir IS the self-contained evidence bundle (it writes
# results/evidence-manifest.json there; `benchmark workday`'s results/workday-report.json
# is a DIFFERENT file and is NOT an evidence-replay bundle):
just cng-bench benchmark generate --out target/agent-<name>/corpus --workers 5000000
just cng-bench benchmark run --dir target/agent-<name>/corpus
just cng-evidence-replay target/agent-<name>/corpus
```

`evidence replay` takes one required `--bundle <dir>` and re-derives digests from the
bundle's own recorded observations/queries with no producer-side state — a tampered
observation or query file refuses `CNG_R11 AuditMismatch`.

## Common `just` recipes

| Recipe | What it does |
|--------|---------------|
| `just cng-test` | `cargo test -p cng` — unit + integration + negative-fixture + boundary suite |
| `just cng-workday *args` | Release-build `benchmark workday` run (see above) |
| `just cng-multi-engine` | Real multi-process engine harness: coordinator + OS-process engines |
| `just cng-test-isolated <name> <bin> *args` | Run one test binary in `target/agent-<name>` |
| `just cng-check-isolated <name>` | `cargo check --tests` in `target/agent-<name>` |

`cng-run`/`cng-bench` wrap `cargo run`; everything else in this table wraps `cargo
test`/`cargo check`. See the "Isolated-target cargo recipes" block in `justfile` for
why `target/agent-<name>` exists (concurrent-agent lock contention on `target/`) and
`just cng-clean-isolated <name>` / `just cng-clean-all-isolated` to reclaim the disk
afterward.

## See Also

- `crates/cng/README.md` — product thesis, ontology mapping, typed refusal algebra
- `crates/cng/completions/cng.bash` — static bash completion script (regenerate manually)
- `crates/cng/Cargo.toml` — feature gates (`runner` default, `bench`, `otel-live`)
- `justfile` — every `cng-*` recipe, including the isolated-target-dir concurrent pattern
