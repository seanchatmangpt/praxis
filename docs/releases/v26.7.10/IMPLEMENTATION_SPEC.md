# v26.7.10 Implementation Specification (PROJ-601..605)

Executable spec for the five v26.7.10 tickets. Written so an implementer can follow it
mechanically with zero information loss: every edit names its file, anchor text, exact new code,
and acceptance command. Read `PRD.md` and `ARD.md` in this directory first for the why; this
document is only the how. Baseline commit: the tree as of `6944e3d`.

Global rules that bind every ticket below (violations are the bug):

1. Never run `cargo` directly — use `just` recipes (`just cng-test`, `just cng-bench-build`,
   `just cng-bench ...`, `just cng-bench-verify <dir>`). If no recipe fits, add one to `justfile`.
2. No `.unwrap()`, `.expect()`, or `panic!()` in `crates/cng/src/**` (tests may use `expect`).
   Every failure path returns `CngRefusal`.
3. No inline SPARQL or Turtle strings in Rust. Queries live in `.rq` files; graph data lives in
   `.ttl` files. Tests use chicago-tdd-tools `test!` macros and file-driven fixtures.
4. No wall clock in any hashed/digested value. `Instant` is telemetry-only.
5. `BTreeMap`/`BTreeSet` in every digest path; never `HashMap` iteration order.
6. Commit messages cite the exact verification command and its result.

Recommended implementation order: 601 → 605 → 603 → 602 → 604. (602 depends on 601's relative
keys and 605's refusal variant; 603's manifest is consumed by 602.)

---

## PROJ-601 — digests.json path-portability fix

### Problem (exact current behavior)

`crates/cng/src/bench.rs` records replay digests keyed by the path string of each set directory
as it appeared at `run` time:

```rust
// bench.rs:1565-1569 (inside run())
let replay_digests: BTreeMap<String, String> = outcomes
    .iter()
    .filter(|r| r.outcome.refusal_code.is_none())
    .map(|r| (r.dir.display().to_string(), r.outcome.powl_digest.clone()))
    .collect();
```

and `verify()` resolves those keys verbatim:

```rust
// bench.rs:2060-2065 (inside verify())
let sample: Vec<(PathBuf, String)> = digests
    .iter()
    .enumerate()
    .filter(|(i, _)| i % usize::max(1, sample_every) == 0)
    .map(|(_, (path, digest))| (PathBuf::from(path), digest.clone()))
    .collect();
```

If the benchmark directory is copied to another path/machine/CWD, `PathBuf::from(path)` points
at a location that no longer exists; `manufacture_set` then returns refusal outcomes and replay
silently fails instead of replaying.

### Change 1 — write bench_dir-relative keys

In `run()`, replace the `replay_digests` construction (the exact block quoted above) with:

```rust
// Per-set digest map keyed RELATIVE to bench_dir so the whole directory
// is relocatable: `benchmark verify` rejoins keys against its own --dir.
// Sets outside bench_dir would be a generator bug; refuse loudly.
let replay_digests: BTreeMap<String, String> = outcomes
    .iter()
    .filter(|r| r.outcome.refusal_code.is_none())
    .map(|r| {
        let rel = r.dir.strip_prefix(bench_dir).map_err(|_| {
            CngRefusal::HardcodingSuspicion(format!(
                "set dir {} is not under bench dir {}; digests.json keys must be \
                 bench-dir-relative for portable replay",
                r.dir.display(),
                bench_dir.display()
            ))
        })?;
        Ok((rel.display().to_string(), r.outcome.powl_digest.clone()))
    })
    .collect::<Result<_, CngRefusal>>()?;
```

Notes: `bench_dir` is the `&Path` parameter of `run()`. The in-run replay comparison a few lines
below (`replay_digests.get(&dir.display().to_string())` at bench.rs:1572) must be updated to the
same relative form:

```rust
let rel_key = dir
    .strip_prefix(bench_dir)
    .map(|p| p.display().to_string())
    .unwrap_or_else(|_| dir.display().to_string());
if let Some(expected) = replay_digests.get(&rel_key) {
```

(The `unwrap_or_else` fallback is acceptable here because the refuse-on-escape already happened
when building the map; a non-prefixed dir at this point can only match nothing, which counts as
a replay miss, never a false pass.)

### Change 2 — rejoin keys in verify()

In `verify()`, replace the `sample` construction (quoted above) with:

```rust
let sample: Vec<(PathBuf, String)> = digests
    .iter()
    .enumerate()
    .filter(|(i, _)| i % usize::max(1, sample_every) == 0)
    .map(|(_, (path, digest))| {
        let candidate = PathBuf::from(path);
        // v26.7.10 digests are bench_dir-relative; pre-v26.7.10 files may
        // hold absolute or CWD-relative keys. Rejoin relative keys against
        // bench_dir; leave absolute keys as-is (legacy compatibility).
        let resolved = if candidate.is_absolute() {
            candidate
        } else {
            bench_dir.join(&candidate)
        };
        (resolved, digest.clone())
    })
    .collect();
```

Additionally, immediately after building `sample`, refuse instead of silently replaying nothing
when the resolved paths do not exist:

```rust
let missing: Vec<String> = sample
    .iter()
    .filter(|(dir, _)| !dir.is_dir())
    .map(|(dir, _)| dir.display().to_string())
    .collect();
if !missing.is_empty() {
    return Err(CngRefusal::AuditMismatch(format!(
        "digest keys resolve to {} missing set dir(s) under {}; first: {}",
        missing.len(),
        bench_dir.display(),
        missing[0]
    )));
}
```

`AuditMismatch` is added by PROJ-605. If implementing 601 before 605, use
`CngRefusal::IoRefused` here and switch it to `AuditMismatch` when 605 lands.

### Tests

Add to `crates/cng/tests/` a new file `cng_bench_portability.rs`, gated on the bench feature
(`#![cfg(feature = "bench")]`), using `chicago_tdd_tools::prelude::*` and `test!`:

1. `test!(digests_keys_are_bench_dir_relative, { ... })` — generate a tiny benchmark
   (`cng::bench::generate` with workers=6400, depth=2, or the smallest values `generate`
   accepts; read `benchmark_generate` in `main.rs:512` for the exact function and defaults),
   run it, read `<dir>/results/digests.json`, assert no key starts with `/` and no key contains
   the scratch prefix.
2. `test!(verify_replays_after_directory_move, { ... })` — generate + run into scratch dir A,
   recursively copy A to sibling scratch dir B (std `fs`, no shell), delete A, call
   `cng::bench::verify(&B, 1, 2)`, assert `replay_passes == replayed` and `replayed > 0`.

Scratch dirs go under `target/chatman/cng-tests/portability/` following the existing pattern in
`cng_pipeline.rs:73-79`. Full runs are slow; use the smallest generator parameters that produce
at least 2 sets.

### Acceptance

```bash
just cng-test                       # all suites incl. the 2 new tests pass
just cng-bench-build                # exit 0
# manual end-to-end: generate+run into X, cp -R X Y, rm -rf X, then:
just cng-bench-verify Y             # REPLAY_RESULT=n/n with n > 0
```

---

## PROJ-605 — `CNG_R11 AuditMismatch` refusal variant

### Change — extend the enum

File `crates/cng/src/powl.rs`. Three mechanical edits, mirroring the existing 10 variants:

1. In `pub enum CngRefusal` (powl.rs:37-63), append after the `IoRefused` variant:

```rust
    /// `CNG_R11` — an independent audit replay recomputed a digest that does
    /// not match the recorded one, or a bundle input named by the manifest is
    /// missing/altered. Distinct from `CNG_R08 Nondeterminism` (same-producer
    /// re-manufacture drift): R11 is third-party integrity failure detected
    /// against recorded evidence.
    AuditMismatch(String),
```

2. In `fn code()` (powl.rs:70-83), append the arm:

```rust
            CngRefusal::AuditMismatch(_) => "CNG_R11",
```

3. In `fn message()` (powl.rs:89-102), add `AuditMismatch(m)` to the single or-pattern list
   (append `| CngRefusal::AuditMismatch(m)` before the final `=> m`).

No other match sites exist that require updating (the enum is only matched exhaustively in
those two methods; everything else matches specific variants). Verify with:
`grep -rn "match .*CngRefusal\|CngRefusal::" crates/cng/src/ | grep -v "::A" | head -50` and by
letting the compiler confirm.

### Tests

In the existing `#[cfg(test)] mod tests` of `powl.rs`, add:

```rust
test!(audit_mismatch_refusal_has_stable_code, {
    let refusal = CngRefusal::AuditMismatch("digest drift".to_string());
    assert_eq!(refusal.code(), "CNG_R11");
    assert_eq!(refusal.message(), "digest drift");
    assert_eq!(format!("{refusal}"), "CNG_R11: digest drift");
});
```

The end-to-end negative test (tamper → refuse) lands with PROJ-602 below, since it needs the
replay verb to exercise R11 for real.

### Acceptance

```bash
just cng-test    # includes the new unit test
```

Update the refusal-code table in `crates/cng/README.md` (it documents CNG_R01–R10; append the
R11 row with the same one-line trigger style as the others).

---

## PROJ-603 — evidence bundle manifest

### Goal

One JSON file, `<bench_dir>/results/evidence-manifest.json`, naming the BLAKE3 digest of every
input and output an auditor needs, written by `run()` after all other results files. Sorted
keys (use `BTreeMap`/struct with fields in fixed order), BLAKE3 only, no timestamps.

### Struct (add to bench.rs, near `RunReport` at bench.rs:590)

```rust
/// Evidence bundle manifest: every input and output digest an independent
/// auditor needs to replay this run from the bundle directory alone.
/// All digests are `blake3:<hex>`. No timestamps, no absolute paths.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EvidenceManifest {
    /// Always "MEASURED_CNG_RESULT"; the manifest describes a measured run.
    pub measurement_class: String,
    /// Schema version for forward compatibility. This release: 1.
    pub schema_version: u32,
    /// BLAKE3 over the sorted concatenation of every obs/*.ttl file's bytes
    /// (sort by bench_dir-relative path, concatenate path + NUL + bytes).
    pub obs_digest: String,
    /// Per-query digests, keyed by file stem (e.g. "ocel-events.construct").
    pub query_digests: BTreeMap<String, String>,
    /// Digests of the ontology inputs the queries were generated from.
    /// Keys: "ocel2.ttl", "bench-obs.ttl" (file names only, no paths).
    pub ontology_digests: BTreeMap<String, String>,
    /// BLAKE3 of crates/cng/rules/bench-roles.dl as loaded for this run.
    pub rules_digest: String,
    /// Copied from RunReport: digest of the sorted OCEL N-Triples evidence.
    pub ocel_graph_digest: String,
    /// Copied from RunReport: digest of the ordered SELECT result rows.
    pub sparql_result_digest: String,
    /// Copied from RunReport: chained per-set POWL evidence digest.
    pub evidence_chain_digest: String,
    /// Exact command an auditor runs from the bundle's parent directory.
    pub replay_command: String,
    /// Reserved for a future signing decision (ed25519 exists in
    /// praxis-core/src/signing.rs but is deliberately unwired here).
    /// MUST be emitted as an empty array; never fabricate a signature.
    pub signatures: Vec<String>,
}
```

### Population rules (inside `run()`, after `digests.json` is written at bench.rs:1959-1962)

- `obs_digest`: walk `<bench_dir>/obs/` recursively, collect `.ttl` paths, sort by
  bench_dir-relative path string, feed `blake3::Hasher` with, per file:
  relative-path bytes, one `0u8`, file bytes. Format as `format!("blake3:{}", hash.to_hex())`.
- `query_digests`: the `QuerySet` already holds every query's text keyed by stem
  (bench.rs:159-213). Add a method to `QuerySet`:

```rust
    /// BLAKE3 digest per loaded query, keyed by stem. O(total bytes).
    pub fn digests(&self) -> BTreeMap<String, String> {
        self.queries
            .iter()
            .map(|(k, v)| (k.clone(), format!("blake3:{}", blake3::hash(v.as_bytes()).to_hex())))
            .collect()
    }
```

  IMPORTANT: `run()` must also COPY the loaded query files into
  `<bench_dir>/queries/<stem>.rq` (write the exact in-memory text, not a re-read) so the bundle
  is self-contained; the auditor replays with the bundled queries, not the repo checkout's.
- `ontology_digests`: hash the two files
  `crates/praxis-graphlaw/ontologies/core/ocel2.ttl` and `.../bench-obs.ttl`, resolved via
  `env!("CARGO_MANIFEST_DIR")` + `../praxis-graphlaw/ontologies/core/` — AND copy both into
  `<bench_dir>/ontology/` for the same self-containment reason. If a file is missing, refuse
  `IoRefused` naming the path.
- `rules_digest`: hash the same `bench-roles.dl` text the run loaded (bench.rs:1463-1468 already
  reads it into `rules_text`; hash that string, and copy it to `<bench_dir>/rules/bench-roles.dl`).
- The three output digests are copied from the already-computed locals that feed `RunReport`.
- `replay_command`: the literal string
  `"cng evidence replay --bundle <this directory>"` — write it with the placeholder text, not an
  absolute path (no absolute paths anywhere in the manifest).
- Serialize with `serde_json::to_string_pretty`, write to
  `<bench_dir>/results/evidence-manifest.json`, mapping errors to `IoRefused` exactly like the
  neighboring `digests.json` write.

Add to `benchmark_run` in `main.rs` (after the `DERIVED_SCALE_PATH` println):

```rust
    println!(
        "EVIDENCE_MANIFEST_PATH={}/results/evidence-manifest.json",
        report.bench_dir
    );
```

### Tests

In `cng_bench_portability.rs` (same file as 601's tests):

```rust
test!(evidence_manifest_is_complete_and_relative, { ... });
```

Assertions after a small generate+run: the manifest file parses into `EvidenceManifest` (the
struct derives `Deserialize` for exactly this reason); `signatures` is empty;
`query_digests.len() >= 15`; `ontology_digests` has exactly the two expected keys; no string
field in the serialized JSON contains the scratch dir's absolute prefix (read the raw JSON text
and assert `!text.contains(scratch_prefix)`); the bundled copies exist
(`<dir>/queries/`, `<dir>/ontology/`, `<dir>/rules/bench-roles.dl`).

### Acceptance

```bash
just cng-test
just cng-bench-build
# small run, then:
python3 -c "import json,sys; m=json.load(open('<dir>/results/evidence-manifest.json'));
assert m['signatures']==[] and m['schema_version']==1; print('MANIFEST_OK')"
```

---

## PROJ-602 — `cng evidence replay` verb (independent auditor)

### Semantics

A party holding ONLY a copied bundle directory (produced by PROJ-601+603's `run`) re-derives
the OCEL evidence graph from the bundled observations + bundled queries and compares digests
against the bundled manifest. No repo checkout state, no producer memory. Exit 0 on agreement;
typed `CNG_R11` refusal (nonzero exit) on any mismatch.

### New function (bench.rs)

```rust
/// Report of an independent auditor replay from a self-contained bundle.
#[derive(Debug, serde::Serialize)]
pub struct AuditReplayReport {
    pub bundle_dir: String,
    pub obs_files_hashed: usize,
    pub obs_digest_match: bool,
    pub queries_verified: usize,
    pub ocel_graph_digest_match: bool,
    pub recomputed_ocel_graph_digest: String,
    pub expected_ocel_graph_digest: String,
}

/// Independent auditor replay: recomputes evidence from bundle files only.
///
/// Steps: (1) parse results/evidence-manifest.json; (2) re-hash obs/*.ttl and
/// compare obs_digest; (3) re-hash queries/*.rq against query_digests;
/// (4) load obs/*.ttl into a fresh store, run the six bundled
/// ocel-*.construct queries, serialize sorted N-Triples, BLAKE3, compare to
/// ocel_graph_digest. Any disagreement or missing input refuses CNG_R11.
///
/// # Complexity
/// O(obs bytes + evidence triples log-sorted).
pub fn audit_replay(bundle_dir: &Path) -> Result<AuditReplayReport, CngRefusal> { ... }
```

Implementation constraints, in order:

1. Read manifest; a missing/unparsable manifest is `AuditMismatch` (the bundle is not auditable),
   not `IoRefused` — message must name the expected path.
2. Recompute `obs_digest` with the identical algorithm as PROJ-603 (factor that algorithm into a
   shared private fn `fn obs_dir_digest(bench_dir: &Path) -> Result<String, CngRefusal>` used by
   both `run()` and `audit_replay()` — one implementation, zero drift). Mismatch →
   `AuditMismatch` naming both digests.
3. Load queries from `<bundle>/queries/` via the existing `QuerySet::load`. For every key in
   `manifest.query_digests`, the loaded set must contain the stem AND its recomputed digest must
   equal the manifest's. Missing or drifted → `AuditMismatch` naming the stem.
4. Rebuild the observation store: `Store::new()`, then load every `<bundle>/obs/**/*.ttl` sorted
   by relative path, using the same `RdfParser::from_format(RdfFormat::Turtle)` pattern used
   elsewhere in bench.rs. Parse failure → `AuditMismatch` (tampered bundle), naming the file.
5. Materialize OCEL: run exactly the six construct stems, in this fixed order:
   `ocel-events.construct`, `ocel-objects.construct`, `ocel-e2o.construct`,
   `ocel-o2o-sockets.construct`, `ocel-receipts.construct`, `ocel-log.construct`, inserting
   results into a fresh evidence store. REUSE the existing construct-execution helper in
   bench.rs (`run_construct` at bench.rs:~1132 and the materialization loop at ~1755-1763);
   factor if needed rather than duplicating.
6. Serialize the evidence store with the same sorted-N-Triples routine `run()` uses
   (bench.rs:~1765-1781 — factor into a shared
   `fn evidence_digest(store: &Store) -> Result<String, CngRefusal>` so both paths share bytes).
7. Compare to `manifest.ocel_graph_digest`. Equal → Ok(report). Different → `AuditMismatch`
   with BOTH digests in the message.

### New verb (main.rs)

Follow the exact shape of `benchmark_verify` (main.rs:616-631):

```rust
/// Independent auditor replay from a self-contained evidence bundle.
#[cfg(feature = "bench")]
#[verb("replay", "evidence")]
fn evidence_replay(bundle: String) -> Result<cng::bench::AuditReplayReport> {
    let report = cng::bench::audit_replay(Path::new(&bundle)).map_err(to_cli_error)?;
    println!("AUDIT_OBS_DIGEST_MATCH={}", report.obs_digest_match);
    println!("AUDIT_QUERIES_VERIFIED={}", report.queries_verified);
    println!(
        "AUDIT_OCEL_GRAPH_DIGEST_MATCH={}",
        report.ocel_graph_digest_match
    );
    println!("AUDIT_RESULT=CONFORMANT");
    Ok(report)
}
```

Note the existing `#[verb("evidence", "workflow")]` at main.rs:376 — the noun/verb pair here is
`("replay", "evidence")`, i.e. CLI `cng evidence replay --bundle <dir>`. Confirm argument order
against how clap-noun-verb macros in this file map `#[verb(a, b)]` to `cng <b> <a>`; mirror
whichever convention `benchmark verify` actually exposes (it is invoked as
`cng benchmark verify` with `#[verb("verify", "benchmark")]`).

### Justfile recipe

Append next to `cng-bench-verify` (justfile:~191):

```make
# Independent auditor replay from a self-contained bundle (no producer state).
cng-evidence-replay bundle:
    timeout 3600s cargo run -q --release -p cng --features bench --bin cng -- evidence replay --bundle {{bundle}}
```

### Tests (the tamper negative proof — this is PROJ-605's end-to-end test too)

In `cng_bench_portability.rs`:

1. `test!(audit_replay_conformant_on_untouched_bundle, { ... })` — small generate+run, then
   `audit_replay(&dir)` returns Ok with both match flags true.
2. `test!(audit_replay_refuses_tampered_observation, { ... })` — copy the bundle, open ONE
   `obs/*.ttl` file, append a single comment line (`\n# tampered\n` — a comment changes bytes
   without breaking Turtle parsing, so the failure MUST come from the digest, not the parser),
   call `audit_replay`, assert `Err(CngRefusal::AuditMismatch(_))` and `code() == "CNG_R11"`.
3. `test!(audit_replay_refuses_tampered_query, { ... })` — same but modify one
   `queries/*.rq` byte; assert R11 naming that stem.

### Acceptance

```bash
just cng-test
just cng-bench-build
# end-to-end on a moved copy:
just cng-evidence-replay <copied-bundle-dir>    # prints AUDIT_RESULT=CONFORMANT, exit 0
# tamper one obs file, rerun: exit nonzero, stderr contains CNG_R11
```

---

## PROJ-604 — close remaining inline-SPARQL sites; extend the guard

### Current inline sites (exhaustive; verified by grep at baseline)

1. `crates/cng/src/pipeline.rs:135` — `select_literal` builds
   `format!("SELECT ?v WHERE {{ ?s <{predicate}> ?v }}")`.
2. `crates/cng/src/shape.rs` — six sites inside `validate_powl_store`:
   the `class_count` closure (:71-78), the `pred_count` closure (:79-84), and four standalone
   queries at :119 (`bindings_missing_index`), :130 (`bindings_missing_model`),
   :143 (`unlabelled_leaves`), :156 (`bad_precedes`).

### Design: embedded query files with `include_str!`, plus template substitution

These two modules are the publishable library surface (no `queries/` dir ships with a library
call), so runtime-dir loading is wrong here. The lawful mechanism is: each query lives in its
own `.rq` file under `crates/cng/src/queries/` (new directory, part of the crate source tree),
embedded at compile time via `include_str!`, with `{param}` placeholders substituted by
`str::replace` (never `format!` on a SPARQL string literal). The SPARQL text itself lives only
in `.rq` files; Rust holds file references and parameter names.

Create these seven files (contents = the current inline strings verbatim, with a header
comment naming inputs/outputs and the placeholder tokens):

| File (under `crates/cng/src/queries/`)        | Placeholders           |
|-----------------------------------------------|------------------------|
| `select-literal.rq`                           | `{PREDICATE}`          |
| `shape-class-count.rq`                        | `{PREFIX}`, `{CLASS}`  |
| `shape-pred-count.rq`                         | `{PREFIX}`, `{PRED}`   |
| `shape-binding-missing-index.rq`              | `{PREFIX}`             |
| `shape-binding-missing-model.rq`              | `{PREFIX}`             |
| `shape-unlabelled-leaves.rq`                  | `{PREFIX}`             |
| `shape-bad-precedes.rq`                       | `{PREFIX}`             |

Example — `shape-class-count.rq`:

```sparql
# shape-class-count.rq — structural validator query (compiled in via include_str!).
# Placeholders: {PREFIX} = POWL2 vocabulary IRI prefix, {CLASS} = local class name.
SELECT ?s WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{PREFIX}{CLASS}> }
```

Rust side, e.g. in shape.rs:

```rust
const Q_CLASS_COUNT: &str = include_str!("queries/shape-class-count.rq");
// ...
let class_count = |class: &str| -> Result<usize, CngRefusal> {
    solution_count(
        store,
        &Q_CLASS_COUNT
            .replace("{PREFIX}", POWL2_PREFIX)
            .replace("{CLASS}", class),
    )
};
```

Apply the same pattern to all seven sites. `select_literal` in pipeline.rs substitutes
`{PREDICATE}` the same way. Behavior must be byte-identical: the substituted query strings must
equal the old `format!` outputs exactly (the header comment lines are fine — SPARQL comments are
ignored by the parser — but keep the query body character-identical).

### Guard extension

`crates/cng/tests/no_inline_ttl_guard.rs` currently has `no_inline_sparql_in_bench` scanning
only `src/bench.rs`. Replace that test with a whole-src scan:

```rust
#[test]
fn no_inline_sparql_in_rust_sources() {
    // SPARQL text lives in .rq files (crates/cng/queries/ at runtime for the
    // benchmark; crates/cng/src/queries/ via include_str! for the library).
    // Any SELECT/CONSTRUCT text in a .rs file reintroduces Rust as semantic
    // authority. Needles are assembled from parts so this guard file can
    // never match its own patterns.
    let select_needle = format!("{} ?", "SELECT");
    let construct_needle = format!("{} {{", "CONSTRUCT");

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_files(&crate_root.join("src"), &mut files);
    rs_files(&crate_root.join("tests"), &mut files);

    let mut violations = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", file.display()));
        for (needle, kind) in [
            (&select_needle, "inline SPARQL SELECT"),
            (&construct_needle, "inline SPARQL CONSTRUCT"),
        ] {
            if content.contains(needle.as_str()) {
                violations.push(format!("{}: {kind}", file.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "query-authority boundary violated — SPARQL text must live in .rq files:\n{}",
        violations.join("\n")
    );
}
```

`rs_files` only collects `.rs`, so the new `src/queries/*.rq` files are not scanned by the
guard (correct — they are the lawful home). If any test still trips the needle, that test has
a remaining inline query and must be converted to load from a fixture `.rq` (the pattern in
`crates/cng/tests/fixtures/queries/` already exists — reuse it).

### Tests / acceptance

No new behavioral tests needed beyond the guard itself — the existing suites
(`cng_negative_fixtures::invalid_powl_refuses_cng_r06_via_shape_validation`, `cng_pipeline`,
powl.rs unit tests) already exercise `validate_powl_store` and `select_literal`; they must pass
unchanged, proving byte-identical query behavior.

```bash
just cng-test          # guard now green across all of src/ and tests/
just cng-bench-build   # bench feature still compiles (bench.rs untouched here)
```

---

## Final verification ladder (run after all five tickets, in this order)

```bash
just cng-test                                        # 1. all suites
just cng-bench-build                                 # 2. release build
just cng-bench benchmark generate --out <X> --workers 10000 --depth 2
just cng-bench benchmark run --dir <X>               # 3. campaign, note digests
just cng-bench benchmark run --dir <X>               # 4. byte-identical re-run
just cng-bench-verify <X>                            # 5. producer-side verify
cp -R <X> <Y> && rm -rf <X>                          # 6. relocate bundle
just cng-bench-verify <Y>                            # 7. PROJ-601 proof
just cng-evidence-replay <Y>                         # 8. PROJ-602/603 proof
# 9. tamper: append "# x" to one <Y>/obs/*.ttl, rerun step 8 -> CNG_R11, nonzero
just verify-all                                      # 10. repo DoD gate
```

Every step's exit code and key output lines go into the closing commit message. Update
`docs/releases/v26.7.10/RELEASE_CONTROL.md` rows for PROJ-601..605 from PLANNED to ALIVE only
with those citations attached, and update `crates/cng/BENCHMARK.md`'s replay section to describe
the auditor path.

## References

- `docs/releases/v26.7.10/PRD.md` — capability table and scope rationale
- `docs/releases/v26.7.10/ARD.md` — architecture, manifest schema rationale, CNG_R11 spec
- `docs/jira/v26.7.10/tickets/PROJ-601.md` .. `PROJ-605.md` — ticket stubs this spec details
- `crates/cng/BENCHMARK.md` — evidence authority chain being extended
- `.claude/rules/no-overclaiming.md` — status vocabulary for the closing report
