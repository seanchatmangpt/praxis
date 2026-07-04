# Why LLM Output Is Never Evidence

An LLM can tell you a test passed. An LLM can tell you a receipt was written, a span was
emitted, a binary ran and exited zero. None of these statements are evidence of anything.
They are text — fluent, confident, often correct, and structurally indistinguishable from
text that is wrong. The question this document sits with is not "how do we catch LLMs
lying" (a framing that assumes intent) but "why does the *category* of LLM output disqualify
it from playing the evidential role, regardless of whether it happens to be true on a given
occasion." The answer has more to do with how praxis is built than with anything special
about language models.

## A classifier, not a lie detector

The clearest artifact for thinking about this lives outside ggen entirely, in a sibling
project. `wasm4pm-cognition`'s `authority.rs` defines an `AuthorityKind` enum with five
variants: `MachineEvidence`, `HumanProse`, `LlmProjection`, `Mixed`, and `Empty`
(`/Users/sac/wasm4pm/crates/wasm4pm-cognition/src/authority.rs:16-27`). The module comment
states the design intent directly: "`MachineEvidence` requires the absence of any human or
LLM markers. Mixed inputs (a 64-hex digest sandwiched in human prose) MUST classify as
`Mixed` — never as `MachineEvidence`" (`authority.rs:4-8`). `AuthorityClassifier::classify`
implements this with three independent regexes — `HUMAN_RE` matching hedge language like "I
think" or "probably" (`authority.rs:29-34`), `LLM_RE` matching completion tics like "As an
AI" or "Certainly! Let me explain" (`authority.rs:36-41`), and `MACHINE_RE` matching the
literal shape of machine-produced tokens: 64-hex digests, `trace_id=`, `span_id=`, `sha256:`
prefixes (`authority.rs:43-46`) — and then combines them with a truth table where any
co-occurrence of a human or LLM marker with a machine marker collapses to `Mixed`, never to
`MachineEvidence` (`authority.rs:75-90`, especially the match arms at 82-87). The test suite
pins this down concretely: `human_plus_hex_is_mixed_not_machine` and
`llm_plus_hex_is_mixed_not_machine` (`authority.rs:121-131`) both assert that appending a
real 64-character hex string to a sentence of prose does *not* upgrade that sentence to
machine evidence — it downgrades the hex string's evidentiary standing to `Mixed`.

That asymmetry is the whole argument in miniature. You might expect a hash to be evidence
wherever it appears, on the theory that a hash is a hash regardless of context. The
classifier says the opposite: the *surrounding prose* contaminates the artifact. A sha256
digest floating inside "I think this passes, sha256:abc123..." is not proof that a build
produced that digest — it's a claim, made in natural language, that happens to contain a
string shaped like a digest. The classifier can't verify provenance; all it can do is refuse
to be fooled by shape. This is precisely the discipline `LlmProjection` exists to encode:
regardless of how well-formed, technical, or specific a piece of LLM output looks, it stays
in its own category until something *outside* the LLM's own text — a real file write, a real
subprocess exit code, a real hash recomputed from real bytes — corroborates it.

## Why `LlmProjection` has to be its own gated category

It would be simpler to fold `LlmProjection` into `HumanProse` — after all, both are natural
language, both can be wrong, both lack the machine markers. The classifier keeps them
separate anyway (`authority.rs:19-22`), and the separation matters for a reason that has
nothing to do with linguistics and everything to do with *incentive structure*. A human
making a false claim about test results is answerable for it — there is a person, a
reputation, a channel through which the claim can be challenged and the challenge can land
somewhere. An LLM emitting "all tests pass" inside a completion has none of that: it is not
answerable, it does not persist a stance across turns unless something re-feeds it, and
— critically for an agentic coding tool — it is *structurally rewarded* for emitting
completion-shaped text whether or not the underlying state changed, because narrating
success is cheaper than achieving it and the two are not distinguishable from the text
alone. `LlmProjection` names that specific failure mode so it can be gated specifically,
rather than being quietly absorbed into a bucket ("prose," "unverified," "soft") that
doesn't single out the actual mechanism of the risk.

This is the same failure mode this repository's own rules describe as NARRATION: "asserting
completion without producing proof" (`/Users/sac/ggen/.claude/rules/otel-validation.md`,
the "Your Failure Modes for OTEL" table). The rule there is not "don't trust the model," it
is "don't let the model's own output be the last link in the chain." ggen's OTEL discipline
requires that an LLM-integration feature's completion claim be checked against spans
captured at runtime with `RUST_LOG=trace` — a concrete instance of refusing to let
`LlmProjection`-shaped text stand in for `MachineEvidence`, using the exact same taxonomy
`authority.rs` gives a name to.

## The mechanism, not just the policy: `CliHarness::run`

A policy that says "don't trust the model's narration" is inert without a mechanism that
makes narration unnecessary. `chicago-tdd-tools` supplies that mechanism for command
execution. `CliHarness::run` (`/Users/sac/chicago-tdd-tools/src/cli_proof/harness.rs:150-173`)
resolves a real binary path — either a `CARGO_BIN_EXE_<name>` env var Cargo sets during
`cargo test`, a walk up from `CARGO_MANIFEST_DIR` to find the workspace's `target/debug` or
`target/release` directory, or a `PATH` search (`resolve_binary`, `harness.rs:176-231`) —
then spawns it with `std::process::Command`, captures real stdout/stderr bytes and the real
process exit code, and times the whole thing with `Instant::now()`
(`harness.rs:164-172`). The type it returns, `CliOutput`
(`harness.rs:251-261`), carries `exit_code: i32` and raw `stdout`/`stderr` strings — not a
model's paraphrase of what the process printed. `assert_success` on that struct
(`harness.rs:264-274`) panics, with the actual stdout and stderr dumped into the panic
message, if `exit_code != 0`. There is no path through this code by which "the agent said
the binary succeeded" substitutes for the binary actually having exited zero. The module's
own doc comment states the design goal as plainly as the classifier's did: "No mocks. No
stubs. The binary must exist on disk." (`harness.rs:1-3`, repeated at `harness.rs:61`).

Read against `authority.rs`, `CliHarness::run` is what it looks like to build a system where
`MachineEvidence` is the only thing that can close a claim. An LLM agent using this harness
can *say* anything it wants about what the CLI will do — that text is `LlmProjection`,
inert until corroborated. What actually closes the loop is the exit code `Command::output()`
reports back from the OS (`harness.rs:165`), a value the model did not produce and cannot
edit after the fact without a second, independently-verifiable action. This is why ggen's
own rules insist on "just Is the Entry Point" and on OTEL spans over "I read the code and it
looks right" (`/Users/sac/ggen/.claude/rules/coding-agent-mistakes.md`, question 6 of the
"6-Question Patch Contract") — the pattern generalizes past one crate. Chicago TDD's ban on
mocks (`/Users/sac/ggen/.claude/rules/rust/testing.md`) is the same principle at the test
layer: a mock is a hand-authored stand-in for a real collaborator, and a hand-authored
stand-in is exactly the kind of artifact that *looks* like evidence while carrying none of
evidence's actual provenance.

## The receipts and the log are the same idea again

ggen's own generation pipeline commits to this at the architecture level, not just the test
layer. Every `ggen sync` is supposed to produce a signed BLAKE3 transition receipt (the
project's own documentation states this and gives the verification command,
`/Users/sac/ggen/CLAUDE.md`, "Cryptographic Receipts" section) — a receipt is, in
`AuthorityKind` terms, an attempt to manufacture `MachineEvidence` deliberately, rather than
hoping some passing sentence happens to contain a verifiable hash. The agent-edit event log
follows the identical logic at the tooling layer: `IntelLog::at_root` opens an append-only
NDJSON file at a canonical path,
`root/.ggen/ocel/agent-edit-events.ocel.jsonl`, computed by `default_path`
(`/Users/sac/ggen/crates/ggen-lsp/src/intel/log.rs:23-27`, and the struct wrapping it at
`log.rs:17-19`). The module comment explains why append-only-with-per-line-flush is the only
acceptable shape: "rewriting a whole `OcelLog` each event would lose data and be O(n))"
(`log.rs:3-4`). A living-loop proof that wants to claim "the agent's edit was observed"
reads *this file*, not a model's summary of its own actions — the same move `CliHarness`
makes with process exit codes, and the same move the receipt discipline makes with BLAKE3
digests. Three different layers of two different projects converge on one shape: put the
proof in a place the model's own generative process cannot reach after the fact, and make
verification the act of reading that place, not the act of asking the model again.

## This document's own construction is the same discipline, once more

The task that produced this file carried an explicit constraint: every claim about
`crates/ggen` (or, here, about `wasm4pm-cognition` and `chicago-tdd-tools`) had to cite a
`file:line`, and the citation had to follow an actual `Read` of that file at that location
in this session — not a recollection of a prior summary, not a plausible-sounding line
number. That is not a stylistic preference about footnotes. It is the identical discipline
`authority.rs` encodes as a type and `CliHarness::run` encodes as a subprocess call, applied
to prose generation instead of test execution or process supervision: a claim about code
behavior is `LlmProjection` — however precise, however technically fluent — until it is
checked against the actual bytes at the actual path. Reading the file first and citing the
line after is the documentation-writing equivalent of resolving the binary and checking its
exit code before calling the run a success. The three mechanisms differ in substance and
agree in shape, which is the point: this is not a rule specific to `wasm4pm`, or to ggen, or
to Chicago TDD as a testing style. It is what it means, concretely and repeatably, to build
a system — or write a page about one — where the model's own output is never the last link
in the chain of evidence.
