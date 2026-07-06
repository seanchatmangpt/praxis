# anti-llm-cheat-lsp Invocation Policy

`anti-llm-cheat-lsp` is a separate repo/binary (`/Users/sac/anti-llm-cheat-lsp`)
that enforces `ANTI-LLM-STANDING-000..006` (see `CLAIM_RULES.md`) plus its
other diagnostic families over this repo's source and docs. This document
states exactly how to invoke it against praxis and where its config
currently stands.

## Invocation

The binary is `anti-llm-cheat-lsp` (crate name matches). Its CLI is
noun/verb, not a single flat `scan` command:

```sh
# One-shot scan of a directory tree, from the anti-llm-cheat-lsp repo:
cargo run --quiet --bin anti-llm-cheat-lsp -- server scan --dir /Users/sac/praxis

# Or, once installed on PATH (cargo install --path /Users/sac/anti-llm-cheat-lsp):
anti-llm-cheat-lsp server scan --dir /Users/sac/praxis
```

Verified 2026-07-06: `cargo run --quiet --bin anti-llm-cheat-lsp -- --help`
lists `server` (with `serve`/`scan` sub-verbs), `receipt`, `affi` as the
top-level nouns — there is no bare `scan` verb at the root.

Via an editor's LSP client, the same binary runs as
`anti-llm-cheat-lsp server serve` over stdio; the editor is responsible for
launching it per its LSP client configuration (no praxis-specific
integration is checked in for this yet).

## Where `index_path` comes from

`ANTI-LLM-STANDING-000..006` only run when the scanned repo has a
`[standing]` table in its `anti.toml`
(`/Users/sac/anti-llm-cheat-lsp/src/config.rs`, `StandingConfig`). Fields:

```toml
[standing]
index_path = "target/praxis-standing/standing.json"  # default; relative to the scan root
max_index_age_secs = 86400                            # default; 24h freshness window
```

**Current state**: praxis does not yet have an `anti.toml` at its repo root
(checked 2026-07-06 — `find /Users/sac/praxis -maxdepth 1 -iname anti.toml`
returns nothing). Standing-claim enforcement (`002`–`006`) is therefore not
yet active for this repo; only the purely textual `ANTI-LLM-STANDING-001`
(unscoped-claim detection) would fire, and only once `anti-llm-cheat-lsp` is
actually run against this tree (nothing runs it automatically today). To
opt in: add an `anti.toml` at the praxis repo root with at minimum:

```toml
[meta]
name = "praxis"
version = "1"

[standing]
index_path = "target/praxis-standing/standing.json"
```

This is a config file to add in a follow-up pass, not part of this ticket's
scope (which builds the standing surface the LSP would consume, not the LSP
wiring itself).

## Current exemption paths

`[surface].non_blocking_path_prefixes` is the shared exemption list every
standing claim-diagnostic (except the bare index-property checks `000`/
`006`) honors via `config.surface_is_non_blocking`. Praxis has no `anti.toml`
yet, so no exemption prefixes are configured. When one is added, historical/
archived docs likely to carry stale readiness language
(`docs/jira/`, `docs/releases/*/NO_TERMINAL_BLOCKERS.md`-style ledgers once
superseded) are the expected candidates, mirroring the
`["docs/jira/", "docs/archive/"]` example already documented in
`anti-llm-cheat-lsp/src/config.rs`.
