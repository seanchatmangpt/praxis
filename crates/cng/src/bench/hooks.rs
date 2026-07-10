//! Workday hook broker (PROJ-612/613): lawful actuation through the
//! praxis-graphlaw knowledge-hooks engine, plus the Dialect Registry gate.
//!
//! Actuation law (zero-unreceipted-actuation): every executed workflow
//! transition of the single-operator workday is actuated through EXACTLY
//! one hook of `crates/cng/hooks/workday-pack.ttl` (hook name == workload
//! category) and must yield that hook's `HookReceipt`; a missing receipt is
//! `CNG_R13 UnreceiptedActuation`, never a warning.
//!
//! Registry gate (PROJ-613): before any tick executes, the Dialect
//! Registry (`crates/cng/hooks/dialect-registry.ttl`) is validated against
//! its closed shape (`dialect-registry.shape.ttl`). Validation choice —
//! documented per ticket: cng validates the registry with SPARQL structural
//! queries over oxigraph (`registry-missing-fields.rq`,
//! `registry-closed-violations.rq`, both SHAPE-DRIVEN: the required-field
//! list and closedness law are read from the loaded shape graph's
//! `sh:property`/`sh:path`/`sh:closed` declarations, never duplicated),
//! following the existing `shape.rs validate_powl_store` pattern. graphlaw's
//! `validate_shacl` was not chosen because the SPARQL route yields the typed
//! `(entry, field)` diagnostic that `CNG_R14 DialectRegistryRefused`
//! requires, and its support for `sh:closed`/`sh:or` on this shape is
//! UNVERIFIED; the shape file remains the single source of the law either
//! way. Any violation refuses `CNG_R14` naming the entry and field.
//!
//! Determinism: hook receipts (`delta_hash`, `idempotency_key`,
//! `delta_quads`) are content-derived BLAKE3 values from graphlaw's
//! canonicalized (sorted, bnode-c14n) delta serialization; each actuation
//! runs in a FRESH `TripleStore` seeded from the once-loaded pack text, so
//! a transition's delta covers exactly that transition's actuation fact.
//! Verdict records accumulate in transition order (Kahn schedule inside
//! each materialization is deterministic); `run_hook_hash()` digests them
//! for the evidence chain. No wall clock anywhere.

use std::fs;
use std::path::{Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::NamedNodeRef;
use oxigraph::store::Store;

use praxis_graphlaw::hooks::{hook_hash, HookVerdictRecord};
use praxis_graphlaw::term::Triple;
use praxis_graphlaw::TripleStore;

use crate::powl::CngRefusal;

use super::roles::select_rows;
use super::templates::QuerySet;
use super::RWAI_PREFIX;

/// The evidence a single lawful actuation returns to the workday loop.
#[derive(Debug)]
pub(super) struct HookActuationReceipt {
    /// Hook name (== workload category slug).
    pub(super) hook_name: String,
    /// BLAKE3 hash of the canonicalized delta quads.
    pub(super) delta_hash: String,
    /// Content-derived idempotency key (BLAKE3 over a domain-separated
    /// prefix + delta hash).
    pub(super) idempotency_key: String,
}

/// Broker owning the once-loaded workday hook pack and the accumulated
/// hook verdict records of the day. See the module docs for the laws it
/// enforces.
#[derive(Debug)]
pub(super) struct WorkdayHookBroker {
    /// Full Turtle texts of the hook packs, read from disk exactly once,
    /// in the fixed admission order (graphlaw admits at most 12 hooks per
    /// pack, so the 14 category hooks ship as two packs).
    pack_ttls: Vec<String>,
    /// Sorted `kh:name` values of the pack's hooks (oxigraph pattern scan).
    hook_names: Vec<String>,
    /// Every `HookVerdictRecord` of the day, in actuation order.
    verdicts: Vec<HookVerdictRecord>,
    /// Telemetry: successful actuations (receipts obtained).
    actuations: usize,
}

impl WorkdayHookBroker {
    /// Default hooks directory: `<CARGO_MANIFEST_DIR>/hooks`.
    pub(super) fn default_hooks_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("hooks")
    }

    /// Constructs the broker: validates the Dialect Registry against its
    /// closed shape (CNG_R14 gate — runs BEFORE any tick), reads the hook
    /// pack once, admits it through graphlaw's `load_hook_pack` (SHACL
    /// `kh:HookShape` law) to fail early, and scans the hook names.
    ///
    /// # Errors
    /// `CNG_R14 DialectRegistryRefused` on any registry shape violation;
    /// `CNG_R01/R05/R10` for unreadable or unadmittable pack/registry
    /// artifacts.
    ///
    /// # Complexity
    /// O(registry triples) load + two SELECTs, plus O(pack triples) for
    /// one pack admission and one name scan.
    pub(super) fn new(
        registry_path: &Path,
        shape_path: &Path,
        pack_paths: &[PathBuf],
        queries: &QuerySet,
    ) -> Result<Self, CngRefusal> {
        validate_dialect_registry(registry_path, shape_path, queries)?;

        // # Complexity
        // O(total pack bytes) reads, in the fixed pack order.
        let mut pack_ttls = Vec::with_capacity(pack_paths.len());
        for pack_path in pack_paths {
            pack_ttls.push(fs::read_to_string(pack_path).map_err(|e| {
                CngRefusal::IoRefused(format!("read hook pack {}: {e}", pack_path.display()))
            })?);
        }

        // Early admission through the real graphlaw gate (kh:HookShape
        // SHACL law + keyword sweep + compile + Kahn schedule): a pack
        // that cannot be admitted refuses at workday start, not mid-day.
        let mut probe = TripleStore::new();
        for pack_ttl in &pack_ttls {
            probe.load_hook_pack(Path::new(pack_ttl)).map_err(|e| {
                CngRefusal::UnsupportedConstruct(format!("hook pack admission: {e}"))
            })?;
        }

        // Hook name scan via the typed oxigraph pattern API (no SPARQL).
        let store = Store::new()
            .map_err(|e| CngRefusal::IoRefused(format!("hook pack store construction: {e}")))?;
        for pack_ttl in &pack_ttls {
            store
                .load_from_slice(
                    RdfParser::from_format(RdfFormat::Turtle),
                    pack_ttl.as_bytes(),
                )
                .map_err(|e| CngRefusal::MalformedTtl(format!("hook pack parse: {e}")))?;
        }
        let name_pred = NamedNodeRef::new("http://seanchatmangpt.github.io/praxis/kh#name")
            .map_err(|e| CngRefusal::MalformedTtl(format!("kh:name IRI: {e}")))?;
        // # Complexity
        // O(hooks) pattern scan + O(h log h) sort.
        let mut hook_names = Vec::new();
        for quad in store.quads_for_pattern(None, Some(name_pred), None, None) {
            let quad =
                quad.map_err(|e| CngRefusal::MalformedTtl(format!("hook name scan: {e}")))?;
            hook_names.push(super::manufacture::term_value(&quad.object));
        }
        hook_names.sort();
        hook_names.dedup();

        Ok(WorkdayHookBroker {
            pack_ttls,
            hook_names,
            verdicts: Vec::new(),
            actuations: 0,
        })
    }

    /// Sorted hook names of the admitted pack (for HookStanding emission).
    pub(super) fn hook_names(&self) -> &[String] {
        &self.hook_names
    }

    /// Telemetry: number of successful (receipted) actuations so far.
    pub(super) fn actuations(&self) -> usize {
        self.actuations
    }

    /// Actuates one executed transition through its category hook: fresh
    /// `TripleStore` seeded from the once-loaded pack, one actuation fact
    /// `(ex:tx-<workflow>-<seq>, ex:actuates-<category>, "t<tick>")`,
    /// materialize, and collect EXACTLY the matching `HookReceipt`.
    ///
    /// # Errors
    /// `CNG_R13 UnreceiptedActuation` when no receipt with hook name ==
    /// `category` and a non-empty delta hash exists after materialization
    /// (e.g. the category's hook is absent from the pack).
    ///
    /// # Complexity
    /// O(pack triples) per actuation (pack re-admission into the fresh
    /// store) + one materialization over O(pack + 1) facts.
    pub(super) fn actuate(
        &mut self,
        workflow: &str,
        category: &str,
        tick: usize,
        seq: usize,
    ) -> Result<HookActuationReceipt, CngRefusal> {
        let mut ts = TripleStore::new();
        // # Complexity
        // O(total pack triples) re-admission into the fresh store.
        for pack_ttl in &self.pack_ttls {
            ts.load_hook_pack(Path::new(pack_ttl)).map_err(|e| {
                CngRefusal::UnsupportedConstruct(format!("hook pack admission: {e}"))
            })?;
        }
        ts.add(Triple::from(
            format!("<{RWAI_PREFIX}tx-{workflow}-{seq}>"),
            format!("<{RWAI_PREFIX}actuates-{category}>"),
            format!("\"t{tick}\""),
        ));
        ts.materialize().map_err(|e| {
            CngRefusal::UnsupportedConstruct(format!(
                "hook materialization for {workflow}/{category}: {e}"
            ))
        })?;
        let receipts = ts.get_hook_receipts();
        let receipt = receipts
            .iter()
            .find(|r| r.hook_name == category && !r.delta_hash.is_empty())
            .ok_or_else(|| CngRefusal::UnreceiptedActuation {
                workflow: workflow.to_string(),
                category: category.to_string(),
            })?;
        // Verdict records (fired AND not-fired, engine schedule order —
        // deterministic Kahn) accumulate in actuation order for the
        // run-level hook hash.
        self.verdicts.extend(ts.verdicts.iter().cloned());
        self.actuations += 1;
        Ok(HookActuationReceipt {
            hook_name: receipt.hook_name.clone(),
            delta_hash: receipt.delta_hash.clone(),
            idempotency_key: receipt.idempotency_key.clone(),
        })
    }

    /// Run-level hook digest over every accumulated verdict record, in
    /// actuation order (graphlaw `hook_hash`). Folded into the workday
    /// evidence chain digest — see `workday.rs` chain-composition docs.
    ///
    /// # Complexity
    /// O(verdicts) serialization + one hash.
    pub(super) fn run_hook_hash(&self) -> Result<String, CngRefusal> {
        hook_hash(&self.verdicts)
            .map_err(|e| CngRefusal::UnsupportedConstruct(format!("hook_hash: {e}")))
    }
}

/// Validates the Dialect Registry against its closed shape: loads registry
/// + shape into one oxigraph store and runs the two shape-driven structural
/// SELECTs (see module docs for why SPARQL, not graphlaw `validate_shacl`).
///
/// # Errors
/// `CNG_R14 DialectRegistryRefused` naming the first violating
/// (entry, field) pair — missing required fields checked before
/// closedness violations.
///
/// # Complexity
/// O(registry + shape triples) load + two SELECTs over the fixed graph.
fn validate_dialect_registry(
    registry_path: &Path,
    shape_path: &Path,
    queries: &QuerySet,
) -> Result<(), CngRefusal> {
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("registry store construction: {e}")))?;
    // # Complexity
    // O(file bytes) per artifact; two artifacts.
    for path in [registry_path, shape_path] {
        let turtle = fs::read_to_string(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| {
                CngRefusal::MalformedTtl(format!("registry load {}: {e}", path.display()))
            })?;
    }
    for query_name in ["registry-missing-fields", "registry-closed-violations"] {
        let rows = select_rows(&store, queries.get(query_name)?)?;
        if let Some(row) = rows.first() {
            let bound = |var: &str| -> Result<String, CngRefusal> {
                row.get(var).cloned().ok_or_else(|| {
                    CngRefusal::MalformedTtl(format!("{query_name}.rq row missing ?{var}"))
                })
            };
            return Err(CngRefusal::DialectRegistryRefused {
                entry: bound("entry")?,
                missing: bound("field")?,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "hooks_test.rs"]
mod hooks_test;
