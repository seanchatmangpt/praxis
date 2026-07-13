# GGEN Parity

Parity verdict for `crates/ggen` (this repo's ggen implementation) against the frozen
reference at `/Users/sac/ggen`, which per this repo's PRD may be reviewed but never used
or imported. The verdict is grounded in a 5-agent survey run this session: one agent
built a template usage census across every real `.tmpl` file in the repo, four surveyed
specific subsystems (receipts, law/validation, schema drift, determinism) against the
reference. `crates/ggen` implements every mechanism the pack corpus actually exercises,
exceeds the reference on receipt-chain tamper detection and refusal discipline, and
deliberately omits a defined set of reference features for which the census below shows
zero real call sites. Three blocker items opened this session to close remaining gaps are
partially resolved — see "Blockers resolved this session" for the exact, verified split
between committed and still-in-flight work.

## Measured template usage

118 real `.tmpl` files across 35 packs plus `templates/` in `crates/ggen/`:

| Frontmatter field | Files using it | Role |
|---|---|---|
| `to` | 118/118 | required, universal |
| `force` | 88/118 | overwrite gate |
| `sparql` | 83/118 | named SELECT map plus body loop — the workhorse mechanism |
| `determinism` | 50/118 | double-render byte-equality check |
| `skip_empty` | 34/118 | suppress output when the query binds nothing |
| `skip_if` | 12/118 | conditional skip guard |

Tera constructs actually used in template bodies: `for`/`endfor` (202/204 blocks),
`if`/`endif` (103/118), `set` (34), `set_global` (12) — only loops, conditionals, and
variable assignment. Filters in use: `json_encode`, `split`, `replace`, `filter`,
`default`, `pascal_case`, `snake_case`, `lower`, `upper`, `shouty_snake_case`, `length`,
`addslashes` — all Tera builtins plus the small case-conversion set.

22 of the 118 files use SPARQL pattern B: the query text itself is the template body
(`to: *.rq`), compiled elsewhere via `include_str!`. Packs prefer this over the
`construct:`/`select:` frontmatter mechanism, which the census shows zero real use of.

Zero measured usage anywhere in the corpus: `construct:` frontmatter, injection
(`inject`/`before`/`after`/`at_line`), `when:` (ASK guard), `from:`, `vars:`,
`sh_before`/`sh_after`, `backup:`, `shape:` (SHACL-on-output),
`freeze_policy`/`freeze_slots_dir`.

## Where crates/ggen exceeds the reference

- Receipt system: a chained BLAKE3 receipt log (`.ggen-v2/receipt-log.jsonl`) with
  tamper detection — refuses to extend a corrupted head (`FM-CHAIN-9`) — and full-history
  verification (`ggen receipt history`). The reference writes a single-shot sha256
  receipt per sync with no chain and no tamper detection.
- Law/validation engine: defaults to GraphLaw (N3/Datalog materialization plus SHACL
  plus closed-vocabulary denials) via `praxis-graphlaw`, with a documented Oxigraph-only
  fallback that typed-refuses all law ops instead of silently no-op'ing. The reference's
  inference is CONSTRUCT-rule materialization only, no Datalog and no denials.
- Schema-reflection tests (`ggen_toml_schema_match.rs`, `frontmatter_schema_match.rs`):
  `Frontmatter` and `GgenConfig` are kept byte-exact against their TTL schema docs via
  `schemars` reflection, enforced as CI-grade tests. No equivalent was found in the
  reference during this session's survey.
- `extra_ontologies` / `lock=false` pack-union mechanisms (this session's own additions,
  commit `c0e7cd71` and commit `4f1c4428`): declared in `ggen.toml`, tracked in the
  content hash and input closure. The reference's `OntologyConfig.imports` is a
  roughly comparable but differently scoped manifest-level mechanism.

Determinism double-render (`check_determinism`) is a real, enforced re-render-and-compare
assertion in `crates/ggen`; the reference's Stage 4.5 coherence gate is analogous but
differently scoped — this item is a parity match, not an advantage either way.

## Deferred by design (zero measured demand)

Every item below is confirmed `UNSUPPORTED-by-design` against the 118-template census,
not an oversight:

- `ggen init` scaffold generator — every consumer already has a hand-authored `ggen.toml`.
- Injection engine (`inject`/`before`/`after`/`at_line`, prepend/append/eof_last) — every
  real template is a from-scratch `to:` write; nothing here injects into hand-written files.
- Marketplace, registry, trust tiers, Ed25519-signed lockfile, semver package versioning —
  packs are exclusively local-path or git-pinned; none has needed search/install/publish.
- Reverse-sync (code back to RDF, the mu-inverse direction) — no consumer round-trips
  generated code back into the graph.
- LSP/MCP/A2A/framework(LangChain)/Six-Sigma tooling — no consumer of this crate is an
  editor, an agent-to-agent bridge, or a manufacturing-quality pipeline.
- The reference's larger Tera filter/function set (camel/kebab/train/pluralize/ordinalize,
  `schema_to_rust`/go/java/elixir/typescript, and more) — the 6 filters plus 6 sparql-helper
  functions `crates/ggen` ships cover 100% of measured usage across 118 templates.
- `construct:` top-level inference rules and SHACL-validated `shape:` frontmatter: the
  fields exist and `construct:` does run once in `Enrich`, but `shape:` is
  existence-checked only, not SHACL-run against output. Zero real templates exercise
  either path, so this is deferred rather than a currently-observed defect.

## Blockers resolved this session

Three blocker workstreams were opened this session against the gaps above. Status below
distinguishes committed-and-tested from still-uncommitted or unverified work; nothing here
is rounded up.

- lock=false pack opt-out — mechanism `ALIVE`: `PackRef::Path` gained a `lock: bool` field
  (commit `4f1c4428`, `feat(ggen): add lock:bool opt-out on PackRef::Path`), fixing a
  spurious `FM-PACK-008` content-hash-mismatch refusal on the sibling-checkout
  `standing-pack` (its `ontology.ttl` is regenerated by every `just standing` run, so
  content-hash pinning is the wrong contract for it). That commit's own message records
  this session's verification: `pack_e2e` 12/12, `ggen_toml_schema_match` 5/5,
  `graph_config_test` 8/8, `ggen_toml_semantic_validation` 7/7, `fmt-check-pkg` clean.
  Applying `lock = false` to `ggen.toml`'s `standing-pack` entry itself is present in the
  working tree but `UNVERIFIED` as committed — `git diff -- ggen.toml` shows it staged as
  an uncommitted change as of this writing.
- 3 pack migrations to `extra_ontologies` (`ocel-bench-pack`, `arazzo-pack`,
  `jira-tracking-pack`) — `PARTIAL`. The `extra_ontologies` mechanism itself and its first
  consumer (`togaf-adm-pack`) are committed (`c0e7cd71`). The three migrations named in
  this session's plan are present in the working tree — `ggen.toml` declares
  `extra_ontologies` for all three, and `packs/arazzo-pack/make-ontology.sh` and
  `packs/ocel-bench-pack/make-ontology.sh` are deleted — but none of this is committed yet,
  and this repo's task tracker still shows the item `in_progress`. No result was found for
  the planned verification (sync idempotence, `cross_pack_matrix`, `framework_packs_e2e`).
- Version bump, binary reinstall, and live recipe verification — `BLOCKED`. `crates/
  ggen/Cargo.toml` shows an uncommitted bump to `26.7.13`, but the installed binary
  (`~/.cargo/bin/ggen`, dated Jul 9) still reports `ggen 26.7.4` — `just install-ggen` has
  not completed successfully this session. The verification agent's own handoff was: "I'll
  stop polling and let the Monitor task (bwhxa6kgh) deliver its notification when the
  install-ggen build completes." No clean `ggen sync run`, and no
  `chatman-sync-verify`/`jira-tracking-verify`/`standing` recipe result, is present in this
  session's evidence. Treat this item as not resolved, not merely slow.

## Disclosed limitations

- `EXPECTED_FACTORY_HEAD` risk: `clients/autonomic-platform/tests/run-evidence-pass.mjs`
  pins a literal chain-head hash from `.ggen-v2/receipt-log.jsonl` as
  `EXPECTED_FACTORY_HEAD`, checked against `ggen receipt history`'s live output. Any
  legitimate `ggen sync run` that extends the chain — exactly what the pending clean sync
  and the three pack migrations above would do — moves the head and goes stale against
  this pinned constant; the evidence-pass test then fails in a way indistinguishable from
  real tamper detection unless a maintainer knows to re-pin it. As of this check the
  working chain head (`e12a2d2c...c72`) still matches the pinned value, only because the
  blocked clean sync above has not yet run to completion — this risk is disclosed, not
  yet triggered.
- Everything under "Blockers resolved this session" beyond the two cited commits is
  uncommitted working-tree state; both source tasks are still `in_progress` in this
  session's tracker, not `completed`.
- `construct:`/`shape:` partial wiring (noted above) is a real functional gap relative to
  a fully SHACL-validated `shape:` pipeline — it is currently unobserved only because no
  template in the 118-file corpus exercises it yet.

## See also

- `crates/ggen/schema/*.ttl` — the TTL schema docs the frontmatter/config reflection
  tests above are kept byte-exact against.
- Pack census methodology: 5-agent survey run this session against all 118 `.tmpl` files
  and the `/Users/sac/ggen` reference tree.
