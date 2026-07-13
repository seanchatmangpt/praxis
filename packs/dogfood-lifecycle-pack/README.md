# Dogfood Lifecycle Pack

Operation Dogfood (v26.7.13): the machinery by which Multifractal Workflow governs its own
Claude Code session lifecycle. Every tool call the operator makes becomes an admitted RDF
observation that `ggen` can validate and receipt — MFW as customer zero, not MFW as an artifact
generator.

This is the crown-I bootstrap seed: MFW's own `ggen` validates MFW's own session capture,
driven by MFW's own hook, on MFW's own session.

## The loop

```text
tool call  --PostToolUse hook-->  session-<id>.ttl  --ggen graph validate --files-->  receipt
 (capture)     (one dfl:ToolEvent      (admitted RDF          (parse validation)      (blake3
               node per event)          observation)                                  digest)
```

1. **Capture** — `hooks/dogfood-lifecycle-capture.sh` (a PostToolUse hook) appends one
   `dfl:ToolEvent` Turtle node per tool event to `.cargo-cicd/lifecycle/session-<id>.ttl`,
   content-addressing the tool input/result with real blake3 digests (`urn:blake3:<hex>`).
2. **Validate** — `hooks/dogfood-lifecycle-session-end.sh` runs
   `ggen graph validate --files <session.ttl>` over every captured log.
3. **Receipt** — the same script appends a content-addressed validation receipt per log to
   `.cargo-cicd/lifecycle/receipts.jsonl`.

## Files

| Path | Role |
|---|---|
| `ontology.ttl` | Session-lifecycle vocabulary (PROV-O / DCTERMS / SKOS / OWL-Time + disclosed `dfl:` terms) |
| `shapes.ttl` | SHACL shape a tool-event node must satisfy (session ref, tool name, ordering, outcome, agent, used/generated) |
| `fixtures/session-good.ttl` | Well-formed sample session log |
| `fixtures/session-malformed.ttl` | Deliberately broken sample (the parse falsifier) |
| `hooks/dogfood-lifecycle-capture.sh` | PostToolUse capture hook (canonical copy; the live install lives in `.claude/hooks/`) |
| `hooks/dogfood-lifecycle-session-end.sh` | Session-end validator + receipt |

## Installation (local)

The repository gitignores `.claude/` (developer-local config), so the wiring is installed
locally, not committed. Copy the hooks and add the PostToolUse matcher to `.claude/settings.json`:

```json
{
  "matcher": "Bash|Edit|Write|Read|Grep|Glob|Task|WebFetch|WebSearch",
  "hooks": [
    { "type": "command", "command": "<repo>/.claude/hooks/dogfood-lifecycle-capture.sh" }
  ]
}
```

Run the validator at session end: `bash .claude/hooks/dogfood-lifecycle-session-end.sh`.

## Verification (live, this session)

The hook fired on real tool calls (`Write`, `Bash`) and captured them to
`session-1f9798ec-f62d-48bb-80a0-e9817fafdb71.ttl`; the session-end validator parsed all logs
(exit 0); a malformed log failed (exit 1):

```text
$ ggen graph validate --files fixtures/session-good.ttl              # -> exit 0, 67 quads
$ ggen graph validate --files fixtures/session-malformed.ttl         # -> exit 1, names the file
$ bash hooks/dogfood-lifecycle-session-end.sh                        # -> VALID, receipts appended
$ (inject a malformed session log) -> session-end                    # -> exit 1 (fail-closed)
```

## Scope and named follow-ups

- `ggen graph validate --files` performs Turtle **parse** validation today, not SHACL. The
  `shapes.ttl` constraints bite once the `--files X --shapes Y` SHACL layer lands.
- The receipt is a content-addressed digest binding, not yet the chained `praxis-core`
  `ReceiptStore` envelope with `ggen receipt verify`.
- The closed tool-name scheme covers nine worker tools
  (`Bash Edit Write Read Grep Glob Task WebFetch WebSearch`); other tools are not captured.
- Outcome is the tool-level result (`dfl:Ok` unless the tool itself errored/was blocked); the
  underlying command exit lives inside the content-addressed `prov:generated` payload.

## Fence

Observation only. No class, predicate, or individual is named `authorize` / `permit` / `grant`
/ `actuate` / `execute`; `dfl:Blocked` records that a call did not complete, never a statement
about authority. Admission, permission, and receipts are separate governed surfaces.

## See also

- `ontology.ttl` — the vocabulary and its disclosed minted-term justifications
- `docs/releases/v26.7.13/` — the Operation Dogfood release documents
- `crates/ggen/src/verbs/handlers.rs` — `handle_graph_validate` (the multi-file validator)
