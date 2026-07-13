//! PROJ-728/729 multi-engine harness: coordinator (in-process, the public
//! `engine_dispatch_remote`/`engine_collect_remote` bench API) + REAL
//! engine OS processes (`CARGO_BIN_EXE_cng engine serve/resume`), talking
//! only through the filesystem inbox/outbox transport.
//!
//! Covered here:
//! - concurrent C+H+M run: contracts flow out, consequences flow back,
//!   lawful re-entry admits, per-engine bundles + serve reports exist,
//!   distributed markers (including the INVERTED existence markers) hold;
//! - isolation falsifiers: a hostile fixture graph is REFUTED by the
//!   marker queries (derived bypass + forbidden obs kinds + divergence),
//!   and a replayed collect over the same root refuses CNG_R25 DoubleAdmit;
//! - fs-inventory: engine roots contain exactly the seven bundle dirs
//!   (inbox/outbox are the only cross-engine surfaces);
//! - G13 crash-resume: kill the engine mid-serve, resume verifies the
//!   ledger chain prefix and completes; a torn ledger tail refuses;
//! - distributed determinism: two fully serialized same-seed C+H+M runs
//!   produce byte-identical roots (every file compared, no exclusions);
//! - recursion crossing engines: a depth-2 contract tree whose parent→child
//!   edges alternate H↔M, at fan_out = 2
//!   (`recursion_crosses_engines_depth_two`) AND at the doctrine's full
//!   fan_out = 8 (`recursion_crosses_engines_full_8x2_fanout`, PROJ-728/
//!   729-followup Gap A) — 146 total dispatches per run (64 of them the
//!   depth-2 leaves), through real `CARGO_BIN_EXE_cng` engine processes,
//!   observed at 32-37s wall-clock across repeat runs in this session
//!   (`std::time::Instant`, telemetry only — never hashed/receipted).
//!
//! No inline Turtle/SPARQL: the hostile graph is an on-disk fixture
//! (`tests/fixtures/multi-engine/`), and every query is loaded from
//! `queries/markers/*.rq` via the public `QuerySet`.

#![cfg(feature = "bench")]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use chicago_tdd_tools::prelude::*;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use cng::bench::{engine_collect_remote, engine_dispatch_remote, QuerySet};

const SEED: u64 = 616;

/// Fresh scratch root for one test. O(existing files) removal.
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/multi-engine-it")
        .join(format!("{}-{}", test_name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Runs the compiled cng binary to completion. O(child runtime).
fn run_cng(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_cng"))
        .args(args)
        .output()
        .expect("spawn cng binary");
    (
        String::from_utf8(output.stdout).expect("utf-8 stdout"),
        String::from_utf8(output.stderr).expect("utf-8 stderr"),
        output.status.success(),
    )
}

/// Spawns a long-running engine serve process (stdout piped). O(1).
fn spawn_engine(root: &Path, engine_id: &str, max_polls: &str, poll_wait_ms: &str) -> Child {
    Command::new(env!("CARGO_BIN_EXE_cng"))
        .args([
            "engine",
            "serve",
            "--root",
            root.to_str().expect("utf-8 root"),
            "--engine-id",
            engine_id,
            "--seed",
            "616",
            "--max-polls",
            max_polls,
            "--poll-wait-ms",
            poll_wait_ms,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn engine serve")
}

/// Kill-on-drop guard around a spawned engine `serve` child process, closing
/// the process-leak gap between `spawn_engine` and its reap: if any
/// `.expect()`/`assert!()` on the path from spawn to `wait()`/
/// `wait_with_output()` panics, `Drop` issues a best-effort `kill()` during
/// unwind instead of orphaning a live engine process (otherwise bounded but
/// real, up to `--max-polls * --poll-wait-ms`). The guard is a no-op on the
/// happy path: `wait_with_output` below takes the child out of `self.0`
/// before waiting, so `Drop` finds `None`; and killing an already-`wait()`-
/// ed child is itself a no-op (`Child::kill` on Unix returns `Ok(())` once a
/// prior successful `wait()` has cached the exit status), so a guard that
/// still holds `Some` after an explicit kill+wait (as in the G13 crash-
/// resume test) costs nothing extra when it drops. Derefs to `Child` for
/// `kill()`/`wait()`; `wait_with_output()` is reimplemented here because
/// `Child::wait_with_output` consumes `self` by value, which `Deref` cannot
/// provide. O(1) per operation, excluding the wait itself which blocks on
/// child exit.
struct EngineGuard(Option<Child>);

impl EngineGuard {
    /// Wraps a freshly spawned child. O(1).
    fn new(child: Child) -> Self {
        EngineGuard(Some(child))
    }

    /// Consumes the guard, waits for the child to exit, and collects its
    /// output — mirrors `Child::wait_with_output`. Takes the child out of
    /// the `Option` first so `Drop` becomes a no-op immediately (the wait
    /// itself is the reap). O(child runtime).
    fn wait_with_output(mut self) -> std::io::Result<std::process::Output> {
        self.0
            .take()
            .expect("engine guard child already taken")
            .wait_with_output()
    }
}

impl std::ops::Deref for EngineGuard {
    type Target = Child;
    fn deref(&self) -> &Child {
        self.0.as_ref().expect("engine guard child already taken")
    }
}

impl std::ops::DerefMut for EngineGuard {
    fn deref_mut(&mut self) -> &mut Child {
        self.0.as_mut().expect("engine guard child already taken")
    }
}

impl Drop for EngineGuard {
    /// Best-effort kill on drop (e.g. an intervening panic between spawn and
    /// reap). Ignores the result: the child may already have been taken by
    /// `wait_with_output` (no-op), or may have already exited under an
    /// explicit `wait()` elsewhere (`Child::kill` returns `Ok(())` in that
    /// case on Unix), and there is nothing actionable to do with a kill
    /// failure during unwind regardless. O(1).
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
        }
    }
}

/// Runs one serialized engine serve to completion (NoWait: no
/// --poll-wait-ms), returning stdout. O(child runtime).
fn serve_to_budget(root: &Path, engine_id: &str, max_polls: &str) -> String {
    let (stdout, stderr, ok) = run_cng(&[
        "engine",
        "serve",
        "--root",
        root.to_str().expect("utf-8 root"),
        "--engine-id",
        engine_id,
        "--seed",
        "616",
        "--max-polls",
        max_polls,
    ]);
    assert!(ok, "engine serve {engine_id} failed: {stderr}");
    stdout
}

/// Evaluates one on-disk marker query over `store`, returning its single
/// `?value` binding as i64. The query text comes from
/// `queries/markers/<stem>.rq` via the public QuerySet — never inline.
/// O(store facts) per SELECT.
fn marker_value(store: &Store, stem: &str) -> i64 {
    let markers_dir = QuerySet::default_dir().join("markers");
    let queries = QuerySet::load(&markers_dir).expect("load marker queries");
    let query = queries.get(stem).expect("marker query present");
    let prepared = SparqlEvaluator::new()
        .parse_query(query)
        .expect("marker query parses");
    match prepared.on_store(store).execute() {
        Ok(QueryResults::Solutions(mut solutions)) => {
            let solution = solutions
                .next()
                .expect("marker query yields one row")
                .expect("marker row evaluates");
            let term = solution.get("value").expect("?value bound");
            match term {
                oxigraph::model::Term::Literal(lit) => {
                    lit.value().parse::<i64>().expect("integer ?value")
                }
                other => panic!("non-literal ?value: {other}"),
            }
        }
        Ok(_) => panic!("marker query did not yield solutions"),
        Err(e) => panic!("marker query evaluation failed: {e}"),
    }
}

/// Recursively copies a directory tree. O(files + bytes).
fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("mkdir copy target");
    for entry in fs::read_dir(src).expect("read copy source").flatten() {
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            fs::copy(&from, &to).expect("copy file");
        }
    }
}

/// Collects every file under `root` as (relative path → bytes), sorted by
/// the BTreeMap key. O(files + bytes).
fn collect_files(root: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(root).expect("read dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, base, out);
        } else {
            let rel = path
                .strip_prefix(base)
                .expect("path under base")
                .display()
                .to_string();
            out.insert(rel, fs::read(&path).expect("read file"));
        }
    }
}

/// One fully SERIALIZED same-seed C+H+M run (determinism pinning: engines
/// run to their poll budget between the dispatch and collect phases, so
/// every collect poll finds its consequence at poll 0 and nothing
/// arrival-time-dependent enters the observation stream).
/// O(contracts) manufactures.
fn serialized_run(root: &Path, per_engine: usize, depth: u32, fan_out: usize) {
    let dispatched =
        engine_dispatch_remote(root, "C", &["H", "M"], per_engine, depth, fan_out, SEED)
            .expect("dispatch phase");
    assert!(dispatched > 0, "dispatch phase must address contracts");
    serve_to_budget(root, "H", "2");
    serve_to_budget(root, "M", "2");
    let report = engine_collect_remote(
        root,
        "C",
        &["H", "M"],
        per_engine,
        depth,
        fan_out,
        SEED,
        4,
        None,
    )
    .expect("collect phase");
    assert_eq!(report.contracts_dispatched, dispatched);
    assert_eq!(report.consequences_admitted, dispatched);
}

test!(multi_engine_concurrent_dispatch_execute_readmit, {
    // Arrange: dispatch 2 contracts to each of H and M, then start BOTH
    // engines as real OS processes polling their inboxes.
    let root = scratch_dir("concurrent");
    let dispatched =
        engine_dispatch_remote(&root, "C", &["H", "M"], 2, 0, 0, SEED).expect("dispatch phase");
    assert_eq!(dispatched, 4);
    // Generous poll budgets: the engines must outlive the coordinator's
    // collect loop (which writes their quiescence files at its end) even on
    // a slow machine — 3000 polls × 20 ms = a 60 s ceiling, normally ended
    // by quiescence within a second or two.
    let child_h = EngineGuard::new(spawn_engine(&root, "H", "3000", "20"));
    let child_m = EngineGuard::new(spawn_engine(&root, "M", "3000", "20"));

    // Act: the coordinator collects concurrently (real inter-poll waits
    // behind the seam; poll COUNTS are receipted logical facts and are
    // arrival-dependent here — byte-identity is asserted only by the
    // serialized determinism test below). Collect writes the quiescence
    // files, ending both serve loops.
    let report = engine_collect_remote(&root, "C", &["H", "M"], 2, 0, 0, SEED, 400, Some(20))
        .expect("collect phase");
    let out_h = child_h.wait_with_output().expect("H exits");
    let out_m = child_m.wait_with_output().expect("M exits");

    // Assert: consequences flowed back and re-admission passed for all 4.
    assert_eq!(report.contracts_dispatched, 4);
    assert_eq!(report.remote_consequences_received, 4);
    assert_eq!(report.consequences_admitted, 4);
    // Two REAL engine instances receipted engine_started (graph-derived).
    assert_eq!(report.engine_instances, 2);
    // Distributed markers all true (a false marker would have refused);
    // the existence markers prove the multi-engine claim positively.
    assert!(report.markers["MULTI_ENGINE_EXECUTION_PROVEN"]);
    assert!(report.markers["ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN"]);
    assert!(report.markers["DIRECT_ENGINE_BYPASSES_ZERO"]);
    assert!(report.markers["SHARED_MEMORY_CROSSINGS_ZERO"]);
    assert!(report.markers["REMOTE_WORKFLOWS_COMPLETED"]);
    // Both engine processes quiesced lawfully and executed their 2 each.
    for (id, out) in [("H", &out_h), ("M", &out_m)] {
        assert!(out.status.success(), "engine {id} exited nonzero");
        let stdout = String::from_utf8(out.stdout.clone()).expect("utf-8 stdout");
        assert!(
            stdout.contains("ENGINE_QUIESCED=true"),
            "engine {id} must quiesce: {stdout}"
        );
        assert!(
            stdout.contains("CONTRACTS_EXECUTED=2"),
            "engine {id} must execute 2 contracts: {stdout}"
        );
        // Per-engine bundle + serve report exist.
        let bundle = root.join("engines").join(id);
        assert!(bundle.join("receipts/serve-report.json").is_file());
        // fs-inventory (structural half of the isolation claim): the
        // engine root contains EXACTLY the seven bundle dirs — inbox and
        // outbox are the only surfaces the coordinator wrote/read (plus
        // control/quiesce.ttl), and nothing else crosses engine roots.
        let mut entries: Vec<String> = fs::read_dir(&bundle)
            .expect("read engine bundle")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        let mut expected: Vec<String> = [
            "admissions",
            "control",
            "inbox",
            "ledger",
            "outbox",
            "receipts",
            "ticks",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        expected.sort();
        assert_eq!(entries, expected, "engine {id} bundle inventory drifted");
    }
});

test!(isolation_falsifier_hostile_graph_is_refuted_by_markers, {
    // Arrange: the hostile fixture graph no lawful producer can emit.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/multi-engine/forged-bypass-obs.ttl");
    let body = fs::read_to_string(&fixture).expect("read fixture");
    let store = Store::new().expect("store");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
        .expect("fixture parses");

    // Act + Assert: the isolation marker counts 1 crossing + 1 explicit
    // bypass + 1 DERIVED bypass (received-without-sent) = 3, and the
    // replay-divergence marker counts the forged "true" divergence.
    assert_eq!(marker_value(&store, "marker-engine-isolation"), 3);
    assert_eq!(marker_value(&store, "marker-replay-divergence"), 1);
    // An empty lawful graph proves both vacuously (0 = proven).
    let empty = Store::new().expect("store");
    assert_eq!(marker_value(&empty, "marker-engine-isolation"), 0);
    assert_eq!(marker_value(&empty, "marker-replay-divergence"), 0);
    // The existence marker is FALSE (1) on the empty graph — proof it
    // cannot be satisfied vacuously.
    assert_eq!(marker_value(&empty, "marker-multi-engine-execution"), 1);
});

test!(double_admit_falsifier_replayed_collect_refuses_cng_r25, {
    // Arrange: one lawful serialized run (consequence files remain on the
    // engines' outboxes; the coordinator ledger holds the processed keys).
    let root = scratch_dir("double-admit");
    serialized_run(&root, 1, 0, 0);

    // Act: replaying the ENTIRE collect phase over the same root presents
    // the same consequences with already-admitted idempotency keys.
    let second = engine_collect_remote(&root, "C", &["H", "M"], 1, 0, 0, SEED, 4, None);

    // Assert: typed CNG_R25 DoubleAdmit, never a silent re-admission.
    let refusal = second.expect_err("replayed collect must refuse");
    assert_eq!(refusal.code(), "CNG_R25", "got {refusal:?}");
});

test!(g13_crash_resume_verifies_chain_and_completes, {
    // Arrange: 2 contracts for H alone (no collect phase here — the
    // distributed existence markers lawfully require >= 2 engines, and
    // G13 is a serve/resume-level falsifier).
    let root = scratch_dir("g13");
    let dispatched =
        engine_dispatch_remote(&root, "C", &["H"], 2, 0, 0, SEED).expect("dispatch phase");
    assert_eq!(dispatched, 2);

    // Act 1: start H with a real inter-poll wait, then KILL it as soon as
    // durable work (a ledger file) exists — a mid-serve crash. The exact
    // kill point is honestly nondeterministic; every durable artifact is
    // atomically written, so any prefix is lawful.
    let mut child = EngineGuard::new(spawn_engine(&root, "H", "10000", "50"));
    let ledger_dir = root.join("engines/H/ledger");
    // Bounded watch loop: O(attempts). Filters to committed `.ttl` entries
    // only — `FileLedgerSink::append` writes `<id>.tmp` then atomically
    // renames to `<id>.ttl` (dispatch.rs's write_atomic), so counting *any*
    // dir entry can observe a transient `.tmp` mid-rename and kill the
    // engine before a committed ledger file exists (the exact race the
    // torn-tail branch below assumes has already been closed by this same
    // filter — `.find(|p| ... == Some("ttl"))`).
    let mut saw_ledger = false;
    for _ in 0..600 {
        let has_ttl_entry = fs::read_dir(&ledger_dir)
            .map(|d| {
                d.flatten()
                    .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
            })
            .unwrap_or(false);
        if has_ttl_entry {
            saw_ledger = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if !saw_ledger {
        child.kill().ok();
        let mut stderr = String::new();
        std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut stderr)
            .unwrap_or_default();
        panic!(
            "engine never wrote durable ledger state. stderr:\n{}",
            stderr
        );
    }
    child.kill().expect("kill engine mid-serve");
    let _ = child.wait();

    // Act 2 (torn-tail NEGATIVE, on a copy): corrupt the ledger tail and
    // require resume to refuse (CNG_R11 AuditMismatch → nonzero exit).
    let torn_root = scratch_dir("g13-torn");
    copy_dir_recursive(&root, &torn_root);
    let torn_ledger_dir = torn_root.join("engines/H/ledger");
    let victim = fs::read_dir(&torn_ledger_dir)
        .expect("read torn ledger dir")
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .expect("a ledger file exists");
    let mut text = fs::read_to_string(&victim).expect("read ledger file");
    text.push_str("this is a torn tail, not turtle\n");
    fs::write(&victim, &text).expect("corrupt ledger tail");
    let (_stdout, stderr, ok) = run_cng(&[
        "engine",
        "resume",
        "--root",
        torn_root.to_str().expect("utf-8 root"),
        "--engine-id",
        "H",
        "--seed",
        "616",
        "--max-polls",
        "4",
    ]);
    assert!(!ok, "torn ledger tail must refuse resume: {stderr}");

    // Act 3 (POSITIVE): resume the original root — the chain prefix
    // verifies, already-processed contracts are skipped (idempotent
    // consume), unfinished work completes.
    let (stdout, stderr, ok) = run_cng(&[
        "engine",
        "resume",
        "--root",
        root.to_str().expect("utf-8 root"),
        "--engine-id",
        "H",
        "--seed",
        "616",
        "--max-polls",
        "6",
    ]);
    assert!(ok, "resume failed: {stderr}");
    assert!(
        stdout.contains("ENGINE_RESUMED=true"),
        "resume must report resumed: {stdout}"
    );
    // Assert: after resume BOTH consequences exist on the outbox.
    let outbox = root.join("engines/H/outbox");
    let consequences = fs::read_dir(&outbox)
        .expect("read outbox")
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
        .count();
    assert_eq!(consequences, 2, "resume must complete both contracts");
    // The resume-verified observation is durable in the engine's ticks.
    let ticks = root.join("engines/H/ticks");
    let ticks_store = Store::new().expect("store");
    for entry in fs::read_dir(&ticks).expect("read ticks").flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) == Some("ttl") {
            let body = fs::read_to_string(&path).expect("read ticks partition");
            ticks_store
                .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
                .expect("ticks partition parses");
        }
    }
    // Typed pattern scan (no inline SPARQL): one obs:obsKind
    // "resume_verified" fact must exist.
    let kind_pred = oxigraph::model::NamedNode::new("https://ggen.io/ontology/bench-obs#obsKind")
        .expect("obsKind IRI");
    let resume_literal = oxigraph::model::Literal::new_simple_literal("resume_verified");
    let found = ticks_store
        .quads_for_pattern(
            None,
            Some(kind_pred.as_ref()),
            Some(oxigraph::model::TermRef::Literal(resume_literal.as_ref())),
            None,
        )
        .count();
    assert!(found >= 1, "resume_verified observation must be durable");
});

// Swarm audit wnl2yhbgm finding #8: `ObsWriter::new` used to always start `part_idx` at 0
// regardless of what was already on disk. `engine_resume` calls the same `run_serve_loop` as
// `engine_serve`, which constructs a FRESH `ObsWriter` over the SAME `bundle.ticks_dir()` a
// crashed pass already wrote into -- so resume's own first flush would silently overwrite the
// crashed pass's `engine-part-00000.ttl` (and onward), destroying already-durable observation
// partitions in exactly the crash-resume scenario `engine_resume` exists for. This test's own
// falsifier is partition-numbering safety specifically, not chain verification (already covered
// by g13 above): every pre-crash tick partition must be byte-identical after a real `cng engine
// resume`, and resume must have written at least one genuinely new partition beyond that set.
test!(g14_crash_resume_never_overwrites_a_prior_tick_partition, {
    let root = scratch_dir("g14");
    let dispatched =
        engine_dispatch_remote(&root, "C", &["H"], 2, 0, 0, SEED).expect("dispatch phase");
    assert_eq!(dispatched, 2);

    let mut child = EngineGuard::new(spawn_engine(&root, "H", "10000", "50"));
    let ticks_dir = root.join("engines/H/ticks");
    let mut saw_ticks = false;
    for _ in 0..600 {
        let has_ttl_entry = fs::read_dir(&ticks_dir)
            .map(|d| {
                d.flatten()
                    .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
            })
            .unwrap_or(false);
        if has_ttl_entry {
            saw_ticks = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if !saw_ticks {
        child.kill().ok();
        let mut stderr = String::new();
        std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut stderr)
            .unwrap_or_default();
        panic!(
            "engine never wrote a durable tick partition. stderr:\n{}",
            stderr
        );
    }
    child.kill().expect("kill engine mid-serve");
    let _ = child.wait();

    // Snapshot every pre-crash partition file's exact content before resume.
    let pre_crash_partitions: BTreeMap<String, String> = fs::read_dir(&ticks_dir)
        .expect("read ticks dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .map(|p| {
            let name = p.file_name().expect("tick partition has a filename");
            let name = name.to_string_lossy().to_string();
            let content = fs::read_to_string(&p).expect("read pre-crash partition");
            (name, content)
        })
        .collect();
    assert!(
        !pre_crash_partitions.is_empty(),
        "at least one tick partition must be durable before resume"
    );

    let (_stdout, stderr, ok) = run_cng(&[
        "engine",
        "resume",
        "--root",
        root.to_str().expect("utf-8 root"),
        "--engine-id",
        "H",
        "--seed",
        "616",
        "--max-polls",
        "6",
    ]);
    assert!(ok, "resume failed: {stderr}");

    for (name, pre_content) in &pre_crash_partitions {
        let post_content = fs::read_to_string(ticks_dir.join(name))
            .unwrap_or_else(|e| panic!("pre-crash partition {name} missing after resume: {e}"));
        assert_eq!(
            &post_content, pre_content,
            "resume must not overwrite the pre-crash partition {name}"
        );
    }
    let post_crash_partition_count = fs::read_dir(&ticks_dir)
        .expect("read ticks dir")
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
        .count();
    assert!(
        post_crash_partition_count > pre_crash_partitions.len(),
        "resume must have written at least one new tick partition beyond the pre-crash set"
    );
});

// Swarm audit wnl2yhbgm finding #30: `disp:dispatchId` had no character-class restriction in
// `DispatchContractShape`, so an inbox contract (anything writing into
// `<root>/engines/<id>/inbox/`, the real External Dispatch transport `cng engine serve`
// implements) with a path-traversal value would pass shape validation and then be joined
// unsanitized into real filesystem paths across several sites in engine.rs/dispatch.rs. This
// tampers a REAL, already-rendered contract's `dispatchId` literal in place (every other field
// stays genuinely valid, from the real `engine_dispatch_remote` renderer) so the refusal this
// test proves is isolated to that one property, not some unrelated missing field.
test!(
    g15_path_traversal_dispatch_id_is_refused_by_shape_pattern,
    {
        let root = scratch_dir("g15");
        let dispatched =
            engine_dispatch_remote(&root, "C", &["H"], 1, 0, 0, SEED).expect("dispatch phase");
        assert_eq!(dispatched, 1);

        let inbox = root.join("engines/H/inbox");
        let contract_path = fs::read_dir(&inbox)
            .expect("read inbox")
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
            .expect("real dispatched contract exists");
        let original = fs::read_to_string(&contract_path).expect("read real contract");
        let needle = "disp:dispatchId \"disp-remote-H-0000\"";
        assert!(
            original.contains(needle),
            "expected the real contract to carry the deterministic dispatch id: {original}"
        );
        let malicious =
            original.replacen(needle, "disp:dispatchId \"../../../../../../tmp/pwned\"", 1);
        fs::write(&contract_path, &malicious).expect("tamper dispatchId in place");

        let (_stdout, stderr, ok) = run_cng(&[
            "engine",
            "serve",
            "--root",
            root.to_str().expect("utf-8 root"),
            "--engine-id",
            "H",
            "--seed",
            "616",
            "--max-polls",
            "2",
            "--poll-wait-ms",
            "0",
        ]);
        assert!(
        !ok,
        "a path-traversal dispatchId must refuse the contract, not silently admit it: stderr={stderr}"
    );
        assert!(
            stderr.contains("CNG_R15") || stderr.contains("dispatchId"),
            "the refusal must be the shape-pattern violation on dispatchId, not an unrelated \
         failure: stderr={stderr}"
        );
    }
);

// Swarm audit wnl2yhbgm finding #32: `EngineBundle::new` joined `engine_id` directly into the
// filesystem root with no validation, reachable straight from `cng engine serve`'s own
// `--engine-id` argv (no sanitization anywhere between the CLI arg and the join). A malicious or
// mistyped `--engine-id` containing `..` segments would relocate the ENTIRE per-engine bundle
// (inbox/outbox/ledger/everything) outside the intended root. This drives the real CLI directly
// with such a value and asserts the engine refuses before creating anything.
test!(
    g16_path_traversal_engine_id_is_refused_before_any_bundle_dir_is_created,
    {
        let root = scratch_dir("g16");
        let (_stdout, stderr, ok) = run_cng(&[
            "engine",
            "serve",
            "--root",
            root.to_str().expect("utf-8 root"),
            "--engine-id",
            "../../../../../../tmp/evil-engine",
            "--seed",
            "616",
            "--max-polls",
            "1",
            "--poll-wait-ms",
            "0",
        ]);
        assert!(
            !ok,
            "a path-traversal engine_id must refuse before creating the bundle, not silently \
         relocate it: stderr={stderr}"
        );
        assert!(
        stderr.contains("invalid engine_id"),
        "the refusal must be the engine_id validation, not an unrelated failure: stderr={stderr}"
    );
        // The critical safety property: no `engines/` directory tree was ever created under root
        // (the relocation this fix exists to prevent never got a chance to happen).
        assert!(
            !root.join("engines").exists(),
            "no bundle directory should ever be created for a rejected engine_id"
        );
    }
);

// Swarm audit wnl2yhbgm finding #33: `templates/dispatch-consequence.template.ttl` substitutes
// `{{DISPATCH_ID}}` directly after an `ex:` prefix (`prov:wasDerivedFrom ex:{{DISPATCH_ID}}`) --
// an unescaped Turtle IRI-local-name position, via the same naive `fill_template` string
// substitution `EngineBundle::new`'s finding #32 fix and finding #30's `dispatchId` shape-pattern
// fix both guard elsewhere. A `dispatch_id` carrying `>`/`<`/`.`/space characters embedded there
// would corrupt or mis-bind the rendered outbox consequence's Turtle structure rather than merely
// carry a wrong value. Investigation (this session, no code change needed): `contract.dispatch_id`
// in `run_serve_loop` (engine.rs) is the same `InboxContract` bound straight from
// `read_inbox_contract`, which is only ever called *after* `shape_violations` has already checked
// the raw contract Turtle against `DispatchContractShape` -- the same `sh:pattern
// "^[A-Za-z0-9_-]+$"` finding #30's fix (commit `6ba85a75`) added, and confirmed still wired
// through all three `shape_violations` queries (`dispatch.rs::shape_violations`, unchanged since
// that commit). Since `[A-Za-z0-9_-]` excludes every Turtle-syntax-breaking character, no
// admitted `dispatch_id` can ever reach `fill_template`'s `ex:{{DISPATCH_ID}}` position unsafely
// -- finding #33 is closed as a genuine (if previously undisclosed) side effect of finding #30's
// fix, not a separate code change. This test closes the disclosure gap: it proves the specific
// Turtle-injection-shaped payload finding #33 describes is refused at the same contract-admission
// gate g15 exercises, AND that the outbox consequence file -- the artifact finding #33 says would
// be corrupted -- is never written for the rejected dispatch.
test!(
    g17_turtle_injection_dispatch_id_is_refused_before_any_outbox_consequence_is_written,
    {
        let root = scratch_dir("g17");
        let dispatched =
            engine_dispatch_remote(&root, "C", &["H"], 1, 0, 0, SEED).expect("dispatch phase");
        assert_eq!(dispatched, 1);

        let inbox = root.join("engines/H/inbox");
        let contract_path = fs::read_dir(&inbox)
            .expect("read inbox")
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
            .expect("real dispatched contract exists");
        let original = fs::read_to_string(&contract_path).expect("read real contract");
        let needle = "disp:dispatchId \"disp-remote-H-0000\"";
        assert!(
            original.contains(needle),
            "expected the real contract to carry the deterministic dispatch id: {original}"
        );
        // If this landed unescaped at `ex:{{DISPATCH_ID}}` in the consequence template, it would
        // close the IRI early (`>`), terminate the statement (`.`), and open an injected triple.
        let malicious = original.replacen(
            needle,
            "disp:dispatchId \"evil> . <urn:injected> a <urn:MaliciousClass>.\"",
            1,
        );
        fs::write(&contract_path, &malicious).expect("tamper dispatchId in place");

        let (_stdout, stderr, ok) = run_cng(&[
            "engine",
            "serve",
            "--root",
            root.to_str().expect("utf-8 root"),
            "--engine-id",
            "H",
            "--seed",
            "616",
            "--max-polls",
            "2",
            "--poll-wait-ms",
            "0",
        ]);
        assert!(
            !ok,
            "a Turtle-injection-shaped dispatchId must refuse the contract, not render it into \
         the outbox: stderr={stderr}"
        );
        assert!(
            stderr.contains("CNG_R15") || stderr.contains("dispatchId"),
            "the refusal must be the shape-pattern violation on dispatchId, not an unrelated \
         failure: stderr={stderr}"
        );
        let outbox = root.join("engines/H/outbox");
        let outbox_ttl_count = fs::read_dir(&outbox)
            .map(|it| {
                it.flatten()
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
                    .count()
            })
            .unwrap_or(0);
        assert_eq!(
            outbox_ttl_count, 0,
            "the malicious dispatch_id must never reach fill_template's ex:{{DISPATCH_ID}} \
         position -- no consequence file should exist in the outbox for the rejected dispatch"
        );
    }
);

test!(
    distributed_determinism_two_serialized_runs_byte_identical,
    {
        // Arrange + Act: the identical serialized C+H+M run into two roots.
        let root_a = scratch_dir("determinism-a");
        let root_b = scratch_dir("determinism-b");
        serialized_run(&root_a, 1, 0, 0);
        serialized_run(&root_b, 1, 0, 0);

        // Assert: EVERY file byte-identical, no exclusions — the reports carry
        // no paths, PIDs, or wall-clock values by construction.
        let mut files_a = BTreeMap::new();
        let mut files_b = BTreeMap::new();
        collect_files(&root_a, &root_a, &mut files_a);
        collect_files(&root_b, &root_b, &mut files_b);
        let names_a: Vec<&String> = files_a.keys().collect();
        let names_b: Vec<&String> = files_b.keys().collect();
        assert_eq!(names_a, names_b, "file inventories drifted");
        for (rel, bytes) in &files_a {
            assert_eq!(
                Some(bytes),
                files_b.get(rel),
                "file {rel} differs between same-seed runs"
            );
        }
    }
);

test!(recursion_crosses_engines_depth_two, {
    // Arrange + Act: one root per engine, each fanning out fan_out = 2
    // children per level for depth = 2, children round-robined to the
    // OTHER engine — 2 × (1 + 2 + 4) = 14 contracts, every parent→child
    // edge crossing H↔M. The full 8² fan-out (fan_out = 8, 8 + 64 children
    // per root) exercises the same machinery and is a corpus-run
    // assertion, not part of this harness.
    let root = scratch_dir("recursion");
    serialized_run(&root, 1, 2, 2);

    // Assert: crossing is visible in the transport itself — the child of
    // an H-rooted contract landed in M's inbox and vice versa.
    assert!(root
        .join("engines/H/inbox/disp-remote-H-0000.ttl")
        .is_file());
    assert!(root
        .join("engines/M/inbox/disp-remote-H-0000-c0.ttl")
        .is_file());
    assert!(root
        .join("engines/M/inbox/disp-remote-M-0000.ttl")
        .is_file());
    assert!(root
        .join("engines/H/inbox/disp-remote-M-0000-c1.ttl")
        .is_file());
    // And every consequence crossed back: 14 admitted (asserted inside
    // serialized_run) with outbox files on both engines.
    let count = |engine: &str| {
        fs::read_dir(root.join("engines").join(engine).join("outbox"))
            .expect("read outbox")
            .flatten()
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
            .count()
    };
    // Per engine: its own root + the alternating tree levels routed to it.
    assert_eq!(count("H") + count("M"), 14);
});

// PROJ-728/729-followup Gap A: the same spawn-free `serialized_run` machinery
// as `recursion_crosses_engines_depth_two` above, at the doctrine's full
// fan_out = 8 / depth = 2 recursive scale (fan_out^depth = 64 depth-2 leaf
// dispatches, matching the module doc's "8 + 64 children per root" figure).
// Per root: 1 (root) + 8 (level-1 children) + 64 (level-2 leaf children) =
// 73 dispatches; two roots (H, M) = 146 total. `std::time::Instant` is used
// here ONLY as test telemetry printed to stderr — never fed into a receipt,
// digest, or assertion value, per the no-wall-clock-in-digests rule
// (`.claude/agents/cng-rust.md`).
test!(recursion_crosses_engines_full_8x2_fanout, {
    // Arrange + Act: one root per engine, fan_out = 8 children per level,
    // depth = 2, children round-robined to the OTHER engine each level —
    // exactly `recursion_crosses_engines_depth_two`'s tree shape, scaled
    // from fan_out = 2 to fan_out = 8. Wall-clock is measured around the
    // whole dispatch+serve+collect cycle and reported honestly regardless
    // of outcome.
    let root = scratch_dir("recursion-8x2");
    let started = Instant::now();
    serialized_run(&root, 1, 2, 8);
    let elapsed = started.elapsed();
    eprintln!(
        "recursion_crosses_engines_full_8x2_fanout: fan_out=8 depth=2, \
         146 total dispatches (64 depth-2 leaf dispatches) in {elapsed:?}"
    );

    // Assert: crossing is visible in the transport itself at full scale —
    // every parent->child edge of BOTH depth-2 trees alternates H<->M.
    assert!(root
        .join("engines/H/inbox/disp-remote-H-0000.ttl")
        .is_file());
    assert!(root
        .join("engines/M/inbox/disp-remote-M-0000.ttl")
        .is_file());
    for c in 0..8u32 {
        // H-root's 8 first-level children land in M's inbox; M-root's land
        // in H's inbox.
        assert!(root
            .join(format!("engines/M/inbox/disp-remote-H-0000-c{c}.ttl"))
            .is_file());
        assert!(root
            .join(format!("engines/H/inbox/disp-remote-M-0000-c{c}.ttl"))
            .is_file());
        for g in 0..8u32 {
            // Second-level (leaf) children cross back: H-root's land back
            // in H's inbox (via M), M-root's land back in M's inbox (via H).
            assert!(root
                .join(format!("engines/H/inbox/disp-remote-H-0000-c{c}-c{g}.ttl"))
                .is_file());
            assert!(root
                .join(format!("engines/M/inbox/disp-remote-M-0000-c{c}-c{g}.ttl"))
                .is_file());
        }
    }

    // And every consequence crossed back: 146 admitted (asserted inside
    // serialized_run), split evenly (73/73) across both engines' outboxes —
    // each engine executes its own root's 64 leaf children plus the other
    // root's 8 first-level children.
    let count = |engine: &str| {
        fs::read_dir(root.join("engines").join(engine).join("outbox"))
            .expect("read outbox")
            .flatten()
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttl"))
            .count()
    };
    assert_eq!(count("H"), 73);
    assert_eq!(count("M"), 73);
    assert_eq!(count("H") + count("M"), 146);
});
