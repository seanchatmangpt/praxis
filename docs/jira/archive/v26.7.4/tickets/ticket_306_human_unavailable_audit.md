# Ticket: Human-Unavailable Execution Audit — CLOSED

## Title
Prove `fire_hooks`/`replay_firing` never block on live human/interactive input (PROJ-306) — **STATUS: CLOSED**

## Description
The vision doc's "post-RQ runtime requires admitted authority, not live human interaction"
claim is currently true by absence (there is no interactive code in praxis-synthesis at all —
`tests/no_llm_runtime.rs` already tripwires on LLM dependency/symbols), but that absence has
never been asserted as its OWN claim distinct from "no LLM." This ticket extends the existing
tripwire pattern (do not invent a new mechanism) to also assert absence of interactive/blocking
symbols: `std::io::stdin`, any `dialoguer`/`inquire`/similar interactive-prompt crate, any
`std::thread::sleep`-based polling-for-human-input loop.

Combined with PROJ-303's authority-anchor requirement, this closes the vision claim precisely:
execution proceeds from admitted authority declared IN THE GRAPH (checked at firing time),
never from a live prompt to a human waiting at a terminal.

## Acceptance Criteria
- `tests/no_llm_runtime.rs` (or a new `tests/human_unavailable.rs` sitting alongside it) grows
  an additional assertion: no `stdin`, no known interactive-prompt crate name, appears in
  `crates/praxis-synthesis/src/**/*.rs` or `Cargo.toml`'s `[dependencies]`.
- The test passes on the current tree without any code change (expected outcome, since no such
  dependency exists) — this ticket is confirmation-plus-tripwire, not new functionality.
- A one-line addition to `docs/v26.7.3/DEFINITION_OF_DONE.md` or a new v26.7.4 doc citing this
  test as the evidence for "human-unavailable execution," so the claim has a named test, not
  just an assertion in prose.

## Dependencies
PROJ-303 (the authority-anchor refusal is what makes "doesn't need a human" mean something
concrete — without it, "no interactive code" alone would just mean "nothing happens," not
"authorized action happens without a human").

## Verification Mechanism
1. `cargo test -p praxis-synthesis --test no_llm_runtime` (extended) — green.
2. `grep -rn "stdin\|dialoguer\|inquire" crates/praxis-synthesis/src/ Cargo.toml` — empty
   (confirms the test's assertion by hand as well).
3. `cargo test -p praxis-synthesis` full suite green.

## Resolution
Extended `tests/no_llm_runtime.rs` with `source_and_deps_contain_no_interactive_human_symbols`,
reusing the existing allowlist-section-tracking and comment-stripping scan pattern rather than
inventing a new mechanism, and checking `["stdin", "dialoguer", "inquire"]` against both the
`Cargo.toml` `[dependencies]` section and every `src/**/*.rs` file. No code change was needed
in `src/` — the test passed on the first run, confirming the claim by absence. Added gate 18b
to `docs/v26.7.3/DEFINITION_OF_DONE.md` citing this test as evidence. All three verification
commands run and green/empty; see the ticket-closing report for exact output.
