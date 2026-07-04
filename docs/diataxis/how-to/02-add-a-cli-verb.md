# How to Add a New CLI Verb

This assumes you already know how the pipeline works end to end — if not,
start with [Your First Sync](../tutorials/01-your-first-sync.md).

**Problem:** you want `ggen <noun> <verb>` to exist (e.g. add `gc` under the
existing `doctor` noun) without hand-writing a route file.

## The shortest correct recipe

### 1. Add the `CliCommand` triple to `schema/praxis.ttl`

Every route in `crates/ggen/src/verbs/` is projected from a
`praxis:CliCommand` instance. Look at the existing pattern
(`schema/praxis.ttl:289-294`):

```turtle
praxis:CmdGgenDoctorRun a praxis:CliCommand ;
    praxis:inCrate praxis:CrateGgen ;
    praxis:noun "doctor" ;
    praxis:verb "run" ;
    praxis:handler "handle_doctor" ;
    rdfs:comment "Check lockfile/pack drift, orphaned generated artifacts, and receipt-vs-disk staleness." .
```

Append a sibling instance with a new `praxis:verb` (and a `praxis:handler`
name you will implement in step 4):

```turtle
praxis:CmdGgenDoctorGc a praxis:CliCommand ;
    praxis:inCrate praxis:CrateGgen ;
    praxis:noun "doctor" ;
    praxis:verb "gc" ;
    praxis:handler "handle_doctor_gc" ;
    rdfs:comment "Delete orphaned generated artifacts flagged by `doctor run`." .
```

If your command takes boolean flags, add one `praxis:flag "name"` triple per
flag (see `praxis:CmdGgenSyncRun` at `schema/praxis.ttl:259-266`, which has
two: `dry_run`, `watch`).

### 2. Confirm which `[[generation.rules]]` block covers it

Every existing noun already has its own rule in the root `ggen.toml`, each
scoped by a `FILTER(?noun = "...")` (see `ggen.toml:286-314` for the
`doctor` rule). **Adding a new verb under an existing noun needs no new
rule** — the rule's SPARQL has no `?verb` filter, so it already
`GROUP_CONCAT`s every verb row for that noun into one file. Proof, run from
the repository root with the ontology swapped to a scratch copy that has
the new triple (so nothing in the real repo is touched):

```bash
$ ggen sync --dry-run true --ontology /tmp/praxis-plus-gc.ttl --rule crate-ggen-verbs-doctor
```
```
[Quality Gate: Manifest Schema] ✓
...
All Gates: ✅ PASSED → Proceeding to generation phase

[DRY RUN] Would sync 1 files:
  crates/ggen/src/verbs/doctor.rs (would create)

Inference rules: ["derive-workspace-members (order: 0)"]
Generation rules: ["crate-ggen-verbs-doctor -> crates/ggen/src/verbs/doctor.rs"]
{
  "duration_ms": 3,
  "files": [
    {
      "action": "would create",
      "path": "crates/ggen/src/verbs/doctor.rs",
      "rule": "crate-ggen-verbs-doctor",
      "size_bytes": 0
    }
  ],
  ...
  "status": "success"
}
```

Only if you are adding a **brand-new noun** (one with no existing rule) do
you need a new `[[generation.rules]]` block — copy an existing one (e.g.
`ggen.toml:286-314`), change the `FILTER(?noun = "...")` value, `name`, and
`output_file`, and add the noun to `crate-ggen-verbs-mod`'s output by doing
nothing extra: `templates/crates/ggen/verbs_mod.rs.tera` and the
`crate-ggen-verbs-mod` rule query at `ggen.toml:179-194` already select
`DISTINCT ?noun` with no noun filter, so any noun that has at least one
`CliCommand` row is picked up automatically.

### 3. Run `ggen sync` with the OLD installed binary to regenerate

The binary on `PATH` (`ggen 26.7.2`, installed 2026-07-02) is the one that
still understands this `[[generation.rules]]` manifest shape — do not
rebuild from the current worktree's `crates/ggen` source for this step; a
fresh `cargo build -p ggen` from HEAD produces a binary with a different,
incompatible `ggen.toml` schema (`project`/`ontology`/`packs`/`templates`
only, no `[[generation.rules]]` — see `crates/ggen/src/config.rs:16-29`),
which is mid-migration and cannot parse the root manifest. Regenerate with
the old binary, for real this time, against the actual manifest and
ontology:

```bash
$ ggen sync --dry-run false --rule crate-ggen-verbs-doctor
```
```
[Quality Gate: DMAIC Phase 5: Control] ✓

All Gates: ✅ PASSED → Proceeding to generation phase

ℹ Generating 1 files...

✓ Generated 1 files in 3ms
  1 inference rules, 1 generation rules
  725 total bytes written
{
  "duration_ms": 3,
  "files": [
    { "action": "created", "path": "./crates/ggen/src/verbs/doctor.rs",
      "rule": "crate-ggen-verbs-doctor", "size_bytes": 725 }
  ],
  "files_synced": 1,
  "generation_rules_executed": 1,
  "inference_rules_executed": 1,
  "receipt_path": ".ggen/receipts/latest.json",
  "status": "success"
}
```

(Output above captured from an isolated scratch copy of the manifest,
template, and a `praxis.ttl` with the `CmdGgenDoctorGc` triple added, to
avoid writing into the real tree — the resulting `doctor.rs` gained both
routes, verb-sorted:)

```rust
/// Delete orphaned generated artifacts flagged by `doctor run`.
#[clap_noun_verb_macros::verb("gc", "doctor")]
fn doctor_gc() -> Result<serde_json::Value> {
    crate::verbs::handlers::handle_doctor_gc()
}

/// Check lockfile/pack drift, orphaned generated artifacts, and receipt-vs-disk staleness.
#[clap_noun_verb_macros::verb("run", "doctor")]
fn doctor_run() -> Result<serde_json::Value> {
    crate::verbs::handlers::handle_doctor()
}
```

Against the real repository, drop `--ontology` and just run
`ggen sync --dry-run false --rule crate-ggen-verbs-doctor` (or a full
`ggen sync --dry-run false` with no `--rule` filter).

### 4. Add the hand-written handler

The generated route only calls out to `crate::verbs::handlers`
(`templates/crates/ggen/verbs_noun.rs.tera:13`). Add the real logic next to
the other `handle_*` functions in `crates/ggen/src/verbs/handlers.rs` (the
existing handlers start at lines 36, 56, 98, 149, 353):

```rust
/// `ggen doctor gc` — delete orphaned generated artifacts flagged by
/// `doctor run`.
///
/// # Errors
/// Any I/O failure removing an orphan is mapped to a `NounVerbError`
/// execution error, exiting non-zero.
pub fn handle_doctor_gc() -> Result<serde_json::Value> {
    let root = project_root()?;
    // ...
    Ok(serde_json::json!({ "removed": [] }))
}
```

The function name must match the `praxis:handler` string from step 1
exactly — the template interpolates it verbatim
(`templates/crates/ggen/verbs_noun.rs.tera:13`:
``crate::verbs::handlers::{{ row.handler }}(...)``), so a typo there is a
compile error in the generated file, not a silent no-op.

## Never hand-edit the generated route file

Every route file this recipe touches carries this header, generated by the
`verbs_noun.rs.tera` template itself
(`templates/crates/ggen/verbs_noun.rs.tera:1-4`):

```
//! `{{ results.0.noun }}` noun — routes GENERATED by `ggen sync` from
//! `schema/praxis.ttl` (`praxis:CliCommand` instances). Do not edit by hand:
//! routes are a pure projection of the ontology (mode = Overwrite); logic
//! lives in `crate::verbs::handlers`.
```

`crates/ggen/src/verbs/mod.rs` carries the equivalent warning from its own
template (`templates/crates/ggen/verbs_mod.rs.tera:1-5`): "Do not edit by
hand — add a `CliCommand` instance to the ontology instead." Both files are
generated with `mode = "Overwrite"` (`ggen.toml:194`, `:224` etc.) — any
hand edit is silently discarded on the next `ggen sync`.

## Gotchas

- **Rule already covers your noun vs. needs a new one.** Check `ggen.toml`
  for an existing `[[generation.rules]]` block whose SPARQL
  `FILTER(?noun = "...")` matches your noun before writing a new block —
  duplicating a rule for the same noun produces two competing writers of
  the same `output_file`.
- **Flags are booleans only.** The old binary's manifest-driven CLI takes
  `--flag <BOOL>` form for its own top-level flags (e.g.
  `--dry-run true`, not a bare `--dry-run`); the *generated* per-verb flags
  declared via `praxis:flag` become plain `bool` parameters
  (`templates/crates/ggen/verbs_noun.rs.tera:12`), so don't assume the
  clap-derived binary and the ontology-declared verb flags share the same
  parsing convention.
- **Handler name mismatches fail at compile time, not at `sync` time.**
  `ggen sync` will happily generate a call to a handler that doesn't exist
  yet in `handlers.rs`; you'll find out from `cargo build`, not from the
  sync output.
- **`crate-ggen-verbs-mod`'s query has no noun filter** — a new noun is
  wired into `mod.rs` automatically the moment its first `CliCommand`
  triple lands in the ontology, before you've written a rule for its route
  file. Running `ggen sync` at that point generates a `pub mod <noun>;`
  that doesn't resolve yet — add the noun's own rule and route template in
  the same change.
