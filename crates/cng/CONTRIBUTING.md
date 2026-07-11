# Contributing to cng

`crates/cng` (v26.9.10, `Cargo.toml`) is the noun-verb CLI realizing `A = μ(O*)`: admitted
PDDL Turtle planning artifacts become one POWL v2 Turtle workflow artifact. This playbook
captures the conventions a v26.7.10 session converged on after running dozens of concurrent
sub-agents against this crate — write it down once so a new agent or developer does not have
to re-derive it by trial and error. Audit trail for the determinism claims in §6:
`../../docs/releases/v26.7.10/GAP_AUDIT.md`.

## Quick reference

| Rule | One-line law |
|------|--------------|
| Module map (§1) | `bench/decomp/` plans, `bench/{dispatch,engine}.rs` execute, `ipc/` generates |
| Concurrent builds (§2) | `CARGO_TARGET_DIR=target/agent-<name>` + one agent per file cluster |
| Refusals (§3) | `Result<T, CngRefusal>` everywhere; every variant gets 3 match arms + 2 tests |
| Artifacts (§4) | Turtle/PDDL/SPARQL live on disk, never as `.rs` string literals |
| Tests (§5) | `chicago-tdd-tools` `test!` macro; typed-struct fixtures over hand-written text |
| Determinism (§6) | no wall clock, no `rand`, `BTreeMap`/`BTreeSet` only in receipt paths |

## 1. Module map

Read the real `pub mod` declarations before trusting this table — it is a snapshot, not the
source of truth. Top-level modules (`grep 'pub mod' src/lib.rs`):

| Path | Role |
|------|------|
| `src/pipeline.rs` | Default non-bench path: import → merge → plan → project → serialize |
| `src/powl.rs` | POWL 2.0 model/serializer AND the `CngRefusal` taxonomy (CNG_R01–R25) |
| `src/shape.rs` | SPARQL structural validator for generated POWL v2 graphs |
| `src/runner.rs` | Adapter to the published `bcinr-powl` runtime (feature `runner`) |
| `src/telemetry_gen.rs` | `ggen`-generated Weaver semantic-convention bindings, do not edit |
| `src/bench/` | Fortune-5 benchmark verbs, feature-gated behind `bench` |

`src/bench/mod.rs`'s own `pub mod`/`mod` lines, by role:

| Path | Role |
|------|------|
| `bench/decomp/` | No-LLM multi-actor goal decomposition (PROJ-702..710); `pub mod` |
| `bench/dispatch.rs` | External workflow dispatch + lawful re-entry (PROJ-618..620) |
| `bench/engine.rs` | Multi-engine execution: `cng engine serve`/`resume` (PROJ-722..724) |
| `bench/ipc/` | Clean-room IPC-style corpus generators (5 domains, see below); `pub mod` |
| `bench/arazzo.rs` | Arazzo step projection for `api-orchestration` dispatch category |
| `bench/roles.rs` | Old-AI (Mycin) role inference + praxis-graphlaw Datalog over the roster |
| `bench/workday.rs` | Single-operator workday loop (PROJ-608), standing-next-action driven |
| `bench/workday_verify.rs` | `pub mod`; byte-identity harness behind `cng-workday-verify` |
| `bench/generate.rs`, `run.rs`, `report.rs`, `verify.rs` | Benchmark campaign lifecycle |
| `bench/manufacture.rs`, `templates.rs`, `hooks.rs`, `audit_replay.rs`, `api_docs.rs` | Support |

`bench/decomp/` is the no-LLM planning surface named in this ticket: it derives goal
decompositions from admitted graph state, manufactures helper/main PDDL via SPARQL CONSTRUCT,
and plans classically — `LLM_CALLS = 0` is structural (see the module doc,
`src/bench/decomp/mod.rs:1`). `bench/dispatch.rs` and `bench/engine.rs` are the multi-engine
execution pair: `dispatch.rs` is the broker choke point that routes an outbound category to a
dispatch adapter; `engine.rs` is the bounded, receipted poll loop each engine process runs
against its filesystem inbox/outbox. `bench/ipc/` generates the five classical-planning
benchmark domains from first principles (no IPC PDDL file copied), one generator module per
domain plus a shared renderer.

## 2. Concurrent-agent-safe build pattern

Cargo takes an exclusive lock on `target/.cargo-lock` scoped to `CARGO_TARGET_DIR`.
Concurrent `just`/`cargo` invocations against the same target dir serialize on that lock —
this looks like a hang, not a crash, and silently doubles wall-clock time (`../../justfile`
lines 8–14 document the same root cause for the whole repo). As of this writing there is no
`cng-test-isolated`/`cng-check-isolated` recipe in `../../justfile` — check
`grep -n cng- ../../justfile` before relying on this section; if those recipes have landed,
prefer them over the raw form below.

Raw isolation pattern: override `CARGO_TARGET_DIR` per agent, mirroring the existing
`cng-test`/`cng-check` recipe bodies (`cargo test -p cng --features bench` /
`cargo check -p cng --features bench --tests`):

```bash
# Agent "payload" gets its own target dir; agent "workday" gets a different one.
# Both can run cng-test-shaped commands at the same time without lock contention.
CARGO_TARGET_DIR=target/agent-payload cargo test -p cng --features bench
CARGO_TARGET_DIR=target/agent-workday cargo check -p cng --features bench --tests
```

The isolated dir trades a slower first build (no shared incremental cache) for true
concurrency. Run `ps aux | grep -E "cargo (test|build|check)"` before starting a new
build/test/check to confirm no stray concurrent invocation already holds a lock.

Isolation only prevents *build-lock* collisions, not *edit* collisions. Pair it with
disjoint-file-surface discipline: before two agents work in the same session, partition the
work so each agent owns a distinct cluster of files (e.g. one agent owns
`bench/decomp/{compose,select}.rs` + `decomp_test.rs`, a second owns `bench/dispatch.rs` +
`dispatch_test.rs`) and neither edits a file the other has claimed. Two agents editing the
same file concurrently is not solved by `CARGO_TARGET_DIR` — the lock only serializes cargo,
not `Edit` calls — so it must be avoided by task assignment, not tooling.

## 3. Typed-refusal convention

Every fallible operation in this crate returns `Result<T, CngRefusal>` — never `.unwrap()`,
`.expect()`, `panic!()`, or a silently-swallowed `.ok()`/`unwrap_or_default()`. `CngRefusal`
(`src/powl.rs:37`) currently runs `CNG_R01` through `CNG_R25`
(`grep -oE 'CNG_R[0-9]+' src/powl.rs | sort -u | tail -1` to confirm the live max before
adding a new one — never guess or leave a gap in the sequence).

Adding `CNG_R26` (the next number) requires three co-located match arms plus two tests:

1. A doc-commented enum variant (`src/powl.rs:37-229`) naming the code and the invariant it
   protects, in the same style as the existing 25 — e.g. `CNG_R25 DoubleAdmit` documents what
   fires it and what it is refused instead of ("never silently re-applied").
2. A `code()` arm (`src/powl.rs:237`): `CngRefusal::YourVariant { .. } => "CNG_R26"`.
3. A `message()` arm (`src/powl.rs:271`): a stable diagnostic string, not per-instance data.
4. A `Display` arm (`src/powl.rs:345` on) if the variant carries fields worth interpolating,
   following the `"{}: {} (field {value})"` shape used by e.g. `DoubleAdmit`.
5. One positive test that never returns `CNG_R26` on the happy path, and one negative test
   that forces exactly `Err(CngRefusal::YourVariant { .. })` — see `src/bench/decomp/
   decomp_test.rs` for the pattern: `forced_inadmissible_candidate_refuses_cng_r21` and
   `cyclic_composed_order_refuses_cng_r21` are two independent negative tests for the SAME
   code, and `single_actor_is_always_candidate_zero` is the positive counterpart proving the
   happy path never trips it.

Never renumber or reuse an existing `CNG_R` code — extend past 25, do not fill an unused gap.

## 4. No-inline-artifacts rule

SPARQL, Turtle, and PDDL text never live as string literals inside `.rs` source. They live on
disk — `queries/*.rq` (benchmark, loaded at runtime) and `src/queries/*.rq` (library, loaded
via `include_str!`), `templates/*.ttl` and `templates/*.pddl`, `ontologies/*.ttl` — or are
built through typed Rust structs (`Pddl8Domain`, `Pddl8Problem`, etc. from `bcinr_pddl`) that
a renderer serializes. This is mechanically enforced, not a style preference:
`tests/no_inline_ttl_guard.rs` greps every `.rs` file under `src/` and `tests/` for an
assembled-from-parts `@prefix`, `(define (domain`, `(define (problem`, or
`ceng:pddlDomain """` needle and fails the build if any file (other than the one lawful
Turtle-prefix emitter, `src/powl.rs`'s serializer) contains one; a second test in the same
file forbids inline `SELECT ?`/`CONSTRUCT {` text anywhere. Run it directly when adding new
artifact-touching code:

```bash
CARGO_TARGET_DIR=target/agent-yourname cargo test -p cng --test no_inline_ttl_guard
```

If you need a new query or template, add the `.rq`/`.ttl`/`.pddl` file under the matching
directory and load it with `include_str!` or a runtime file read — never paste the text into
a `format!()` call.

## 5. Test conventions

Use `chicago-tdd-tools`' `test!` macro (`use chicago_tdd_tools::prelude::*;`), not raw
`#[test]`, for anything exercising `CngRefusal`/receipt/OCEL behavior — see
`src/bench/decomp/decomp_test.rs:381`
(`kitchen_two_chain_selects_a_two_actor_decomposition`) for a full Arrange/Act/Assert example
that drives production code and asserts on a typed outcome, not a loose `matches!`.

Prefer typed-struct fixtures over hand-written PDDL/Turtle text wherever the test does not
need to exercise a real parser. `src/bench/decomp/decomp_test.rs`'s `kitchen_domain()`/
`kitchen_problem()` helpers (lines 72, 118) build a `Pddl8Domain`/`Pddl8Problem` directly from
`bcinr_pddl` structs — no PDDL text, no parse step, no risk of a fixture silently drifting
from what the struct fields actually mean.

Use on-disk fixture pairs only where the test's whole point is that a real file format
parses (or is correctly refused). `tests/fixtures/decomp-negative/` is the convention to
match: one `<scenario>.domain.pddl` + `<scenario>.problem.pddl` pair per negative case (e.g.
`actor-lacks-capability.domain.pddl` / `actor-lacks-capability.problem.pddl`), named for the
law the pair is meant to violate, never a generic `fixture1.pddl`.

## 6. Determinism invariants

No wall clock (`SystemTime`, `Instant::now`, `std::time`) may enter any receipt/digest path;
the one lawful exception in the whole audited surface is a `std::thread::sleep` behind
`RealTimeWait` in `bench/dispatch.rs`, structurally prevented from reaching a digest. No
`rand`/`random`/`thread_rng` — all pseudo-randomness is `splitmix64`-seeded (see
`src/main.rs`, `src/bench/{generate,run,workday,mod}.rs` for the pattern). No raw `HashMap`/
`HashSet` feeding output order in any receipt/digest path — `BTreeMap`/`BTreeSet` only, so
iteration order is structurally irrelevant rather than incidentally stable.

Do not re-derive this from scratch: `../../docs/releases/v26.7.10/GAP_AUDIT.md` §4
("Determinism/Replay Risks") is the audit trail for this session's sweep of
`src/bench/{decomp,ipc,engine.rs,arazzo.rs}` plus `dispatch.rs`/`workday.rs` against exactly
these patterns (`SystemTime|Instant::now|std::time`, `rand|random|thread_rng`,
`HashMap|HashSet`, `f32|f64`, `process::id|thread::current`) — its "Invariants checked and
confirmed clean" list is the confirmed-clean baseline to extend, not re-audit, the next time
this scope changes. Cite it in a PR instead of re-running the sweep by hand.

## References

- `README.md` — crate overview and CLI usage
- `BENCHMARK.md` — Fortune-5 benchmark campaign (feature `bench`)
- `src/powl.rs` — `CngRefusal` taxonomy, source of truth for §3
- `tests/no_inline_ttl_guard.rs` — mechanical enforcement for §4
- `../../docs/releases/v26.7.10/GAP_AUDIT.md` — determinism audit trail for §6
- `../../justfile` — `cng-*` recipes and the `CARGO_TARGET_DIR` isolation note (§2)
- `../../.claude/rules/praxis-rust-discipline.md` — repo-wide invariant verification checklist
