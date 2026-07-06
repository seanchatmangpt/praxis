//! Parking — lawful quarantine with a way back.
//!
//! PORTED-FROM: knhk (workspace pkg "genesis" v1.2.0, unpublished)
//!   - /Users/sac/knhk/rust/genesis-etl/src/park.rs (`ParkCause`,
//!     `ParkedDelta`, `ParkManager` — demote over-budget work to a holding
//!     queue with a typed cause, receipted)
//!
//! DELTAS: knhk's park queue is in-memory and has NO re-admission policy —
//! parked work has no way back. This port closes both gaps: entries persist
//! through the crate's blake3 WAL (quarantine survives kill -9) and every
//! entry carries a [`ReAdmission`] policy the executor honors at run
//! boundaries. `RawTriple` payloads become node ids (praxis parks plan
//! nodes, not triples). Path-dep refused: genesis-etl drags
//! rdkafka/oxigraph/otel.
//!
//! SYNC: re-diff against the knhk path above before claiming upstream parity.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::wal::Wal;
use crate::Refusal;

/// Why work was parked.
// PORT(knhk): park.rs `ParkCause` — TickBudgetExceeded/RunLengthExceeded
// kept; the two cache-heuristic causes (L1MissPredicted, HeatBelowThreshold)
// are replaced by praxis-semantic causes (CrashLoop, UpstreamParked): praxis
// parks on lawfulness grounds, not cache forecasts (DIVERGES, documented).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParkCause {
    /// Tick budget exceeded (ticks > 8).
    TickBudgetExceeded,
    /// Run length exceeds the 8-item hot-path limit.
    RunLengthExceeded,
    /// Restart intensity exhausted — the node crash-looped.
    CrashLoop,
    /// A dependency was parked; this node cannot lawfully run.
    UpstreamParked,
}

impl ParkCause {
    /// Human-readable description (receipt register head).
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            ParkCause::TickBudgetExceeded => "tick budget exceeded (ticks > 8)",
            ParkCause::RunLengthExceeded => "run length exceeds limit (run_len > 8)",
            ParkCause::CrashLoop => "restart intensity exhausted (crash loop)",
            ParkCause::UpstreamParked => "upstream dependency parked",
        }
    }
}

/// When parked work may come back. Closes knhk's park-with-no-way-back gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReAdmission {
    /// Re-admit when the node's input hashes change (something upstream
    /// was fixed).
    OnInputChange,
    /// Re-admit automatically after this many supervised runs.
    AfterRuns(u8),
    /// Only a human/authority fact re-admits.
    Manual,
}

/// One parked node, receipted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParkedEntry {
    /// Content id of the parked DAG node.
    pub node_id: String,
    /// Why it was parked.
    pub cause: ParkCause,
    /// The way back.
    pub readmission: ReAdmission,
    /// Supervised-run index at which it was parked.
    pub parked_at_run: u64,
    /// Input-hash fingerprint at park time (for `OnInputChange`).
    pub input_fingerprint: String,
}

/// The holding queue, kill-9 durable.
// PORT(knhk): park.rs `ParkManager` — park/get_parked/parked_count
// semantics; persistence + re-admission are the praxis additions.
#[derive(Debug, Default)]
pub struct ParkManager {
    entries: BTreeMap<String, ParkedEntry>,
}

/// WAL key prefix distinguishing park records from memo records (additive —
/// old logs replay unchanged; memo recovery ignores park keys and vice versa).
const PARK_KEY_PREFIX: &str = "park/v1/";

impl ParkManager {
    /// Empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Park a node. Returns false if it was already parked (idempotent).
    pub fn park(&mut self, entry: ParkedEntry, wal: Option<&mut Wal>) -> Result<bool, Refusal> {
        if self.entries.contains_key(&entry.node_id) {
            return Ok(false);
        }
        if let Some(w) = wal {
            let key = format!("{PARK_KEY_PREFIX}{}", entry.node_id);
            let payload = serde_json::to_vec(&entry).map_err(|e| Refusal::InvalidInput {
                detail: format!("park serialize: {e}"),
            })?;
            w.append(&key, &payload)?;
        }
        self.entries.insert(entry.node_id.clone(), entry);
        Ok(true)
    }

    /// Whether a node is currently parked.
    #[must_use]
    pub fn is_parked(&self, node_id: &str) -> bool {
        self.entries.contains_key(node_id)
    }

    /// The parked entry for a node, if any.
    #[must_use]
    pub fn get(&self, node_id: &str) -> Option<&ParkedEntry> {
        self.entries.get(node_id)
    }

    /// Number of parked entries.
    #[must_use]
    pub fn parked_count(&self) -> usize {
        self.entries.len()
    }

    /// Iterate parked entries (deterministic order).
    pub fn iter(&self) -> impl Iterator<Item = &ParkedEntry> {
        self.entries.values()
    }

    /// Run the re-admission pass for the given supervised-run index:
    /// removes and returns every entry whose policy fires. `Manual` entries
    /// never fire here; `OnInputChange` fires when the current fingerprint
    /// differs from the parked one.
    pub fn readmit(
        &mut self,
        run_index: u64,
        current_fingerprint: impl Fn(&str) -> Option<String>,
    ) -> Vec<ParkedEntry> {
        let due: Vec<String> = self
            .entries
            .values()
            .filter(|e| match e.readmission {
                ReAdmission::AfterRuns(n) => {
                    run_index >= e.parked_at_run.saturating_add(u64::from(n))
                }
                ReAdmission::OnInputChange => {
                    current_fingerprint(&e.node_id).is_some_and(|f| f != e.input_fingerprint)
                }
                ReAdmission::Manual => false,
            })
            .map(|e| e.node_id.clone())
            .collect();
        due.iter()
            .filter_map(|id| self.entries.remove(id))
            .collect()
    }

    /// Manually re-admit one node (the authority path for `Manual`).
    pub fn readmit_manual(&mut self, node_id: &str) -> Option<ParkedEntry> {
        self.entries.remove(node_id)
    }

    /// Rebuild the park set from a WAL (park records only; memo records are
    /// ignored). Quarantine survives machine death.
    pub fn recover(path: &Path) -> Result<Self, Refusal> {
        let (cache, _frames, _torn) = Wal::recover(path)?;
        let mut mgr = Self::new();
        for (key, payload) in cache.iter_raw() {
            if let Some(_node) = key.strip_prefix(PARK_KEY_PREFIX) {
                let entry: ParkedEntry =
                    serde_json::from_slice(payload).map_err(|e| Refusal::InvalidInput {
                        detail: format!("park recover: {e}"),
                    })?;
                mgr.entries.insert(entry.node_id.clone(), entry);
            }
        }
        Ok(mgr)
    }
}
