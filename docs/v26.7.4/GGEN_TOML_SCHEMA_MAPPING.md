# `ggen.toml` / Frontmatter Schema Mapping (PROJ-302 extension)

PROJ-302 (no-private-abstraction gate) requires every closed-world predicate
table to have a mapping doc stating, for each field *excluded* from the
crate's closed vocabulary, why no such field exists. This doc covers the two
closed surfaces added under `crates/ggen/schema/`:
`ggen-toml-schema.ttl` (`crate::config::GgenConfig`) and
`frontmatter-schema.ttl` (`crate::template::Frontmatter`).

`ggen.toml` loading is now two-stage, both stages closed:

1. **Structural** — `deny_unknown_fields` deserialization via `star_toml::load_file`/`from_str`, matched field-for-field against `ggen-toml-schema.ttl` (see below).
2. **Semantic** — `crate::config`'s `Project`/`Ontology`/`Templates`/`PackRef`/`GgenConfig` implement `star_toml::Validate`, run via `.validated()` in `GgenConfig::load`/`from_toml_str`. This reuses `star_toml::Validator::check_path`/`check_non_empty` (the same path-traversal/null-byte checks `star_toml` already provides) rather than reimplementing path safety. Every field the TTL marks `ggenspec:pathSafe`/`ggenspec:nonEmpty` has a corresponding negative-case test in `tests/ggen_toml_semantic_validation.rs` — a declared constraint with no enforcing test would itself be exactly the "accepted-but-never-validated" anti-pattern this ticket exists to close, so the constraint declarations are not decorative.

Both TTL files are the source of truth; `tests/ggen_toml_schema_match.rs` and
`tests/frontmatter_schema_match.rs` assert the Rust structs' field sets are
exact matches against them (see `docs/ggen-port-evaluation.md` and this
session's cross-repo research for the full field inventory these decisions
are drawn from).

## `ggen.toml` — fields found in sibling schemas, excluded here

Sibling implementations surveyed: `~/ggen`'s three competing `GgenConfig`/
`GgenManifest` schemas, and `~/unrdf`'s three competing config schemas.

| Field / section (found in siblings) | Excluded because |
|---|---|
| `workspace` (project_config.rs), `inference`/`validation` (manifest/types.rs) | No ticket in this milestone requires multi-crate workspace config or a separate inference/SHACL-validation pipeline stage — SHACL validation is confirmed absent from every sibling implementation's actual code path (LSP and core validator both, independently). Adding the field ahead of the feature would be exactly the "escape-hatch, never validated" anti-pattern found in `manifest/types.rs`'s 12 untyped passthrough fields. |
| `plugins`, `profiles`, `lifecycle` (project_config.rs) | Out of scope per `docs/jira/v26.7.4/tickets/index.md`'s refuse-list doctrine — no PROJ-30x ticket calls for a plugin or profile system. |
| `ai`, `mcp`, `a2a`, `telemetry`, `security`, `performance` sections (schema.rs / project_config.rs) | These are untyped or loosely-typed in every sibling (schema.rs has no `deny_unknown_fields` on any of them); adding them now would import the same unenforced-surface problem this ticket exists to close. `crates/ggen`'s own `otel`/`mcp`/`lsp` features are Cargo-feature-gated, not `ggen.toml`-configured, and stay that way. |
| Inline `gpack.toml`/`package.toml` nesting inside the manifest | Confirmed in research: **no** sibling schema nests a pack manifest inline — all four external schemas plus praxis's own use an external-pointer `PackRef` only. `GgenConfig.packs` here follows that same, unanimous convention. |

## Frontmatter — full field set, each with real, enforced behavior

Sibling implementations surveyed: `~/ggen`'s `Frontmatter` (open, 21 fields)
and `HygenFrontmatter` (4 fields), plus the kgen/unjucks SHACL prior art
(`ontology_catalogue/unjucks/.../template-constraints.ttl`,
`.../kgn/.../injection-shapes.ttl`). Every field found in a sibling schema is
now present in `crate::template::Frontmatter` and `frontmatter-schema.ttl`.
None is accepted without a corresponding implementation — the sequence was
always "implement the capability, then add the field," never the reverse:

| Field | Implementation |
|---|---|
| `sh_before` / `sh_after` | Executed via `sync.rs::run_shell_hook` (`std::process::Command`, cwd = project root), gated by `crate::shell_safety::check_shell_command_safe`'s denylist (adapted from the kgen/unjucks dangerous-command SHACL constraint, since no SPARQL/SHACL engine runs here). Never run during `--dry-run` (a dry run has zero side effects); `sh_after` only runs after a successful `Written`/`Injected` outcome, never after `Skipped`. Non-zero exit is a hard `[FM-SHELL-003]` error. |
| `shape` | `sync.rs::check_shape_files_exist` refuses if any listed path (relative to the project root) does not exist. **Stated limitation**: existence-checked only — no SHACL engine runs in this crate, so listed shapes are not evaluated against rendered output. This is written down, not silently implied as full validation. |
| `determinism` | Typed `Option<bool>` (not an opaque value, unlike `~/ggen`'s `serde_yaml::Value`). When `true`, `sync.rs::check_determinism` renders the template body a second time with the identical context and refuses with `[FM-TPL-009]` if the bytes differ — a real, enforced assertion. |
| `freeze_policy` / `freeze_slots_dir` | `write.rs::check_freeze`/`record_freeze_checksum` implement a real freeze stage inline in `plan_write` (no separate pipeline stage needed): `always` skips once the target exists; `checksum` compares on-disk content against a BLAKE3 checksum recorded under `freeze_slots_dir` the last time ggen wrote the file, skipping (protecting a human edit) on mismatch and otherwise proceeding and re-recording the checksum. |
| `backup` | `write.rs::maybe_backup` copies the existing file to `<path>.bak` before any overwrite (`force` or `inject`). |
| `from` | `sync.rs::parse_template_file` replaces the parsed `Template::body` with the content of the referenced path (resolved relative to the template file's own directory) when frontmatter sets `from:`; frontmatter fields still come from the original file. |

If a future ticket needs a `sh_before`/`sh_after` sandbox beyond the
denylist, or a real SHACL engine for `shape`, extend the corresponding
implementation and update this table in the same commit.
