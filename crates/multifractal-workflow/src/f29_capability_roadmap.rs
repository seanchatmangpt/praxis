//! Family F29 -- "Thermodynamic Capability Roadmap" (atlas ticket V12-029).
//!
//! Wire-phase-1 status (this pass): **MIXED, wired**. This module now carries
//! real, compiling Rust logic for the 8-stage pipeline the F29 atlas requires
//! -- `Process Work Functional -> Pressure Clusterer -> Capability Seed
//! Generator -> Counterfactual Sandbox -> Scenario Corpus Replayer -> Work
//! Delta Comparator -> Standing Invariant Gate -> Roadmap Renderer` -- the
//! 9-state lifecycle (`WORK_MEASURED -> PRESSURE_CLUSTERED -> SEED_GENERATED
//! -> CANDIDATE_INJECTED -> CORPUS_REPLAYED -> DELTA_MEASURED ->
//! INVARIANTS_CHECKED -> ROADMAP_EMITTED`, with `REFUSED` reachable only from
//! `PRESSURE_CLUSTERED`/`CORPUS_REPLAYED`, per the atlas L5 `stateDiagram-v2`
//! read in full this session), and a `CapabilityRoadmapRefused` typed refusal
//! fired at every gate. Unlike some sibling MIXED families in this crate,
//! every C1-C8 stage below has a real, hand-written, unit-tested
//! implementation -- there is no domain-specific precedent anywhere in the
//! repo to reuse or adapt (confirmed by this module's own survey: a
//! repo-wide grep for every F29 noun returned zero hits), but the stages
//! themselves (measurement, clustering, content-addressing,
//! structural-identity checking, integer delta, evidence-flag gating,
//! ranking) are generic enough to implement for real over caller-supplied
//! data. Every place a stage's real implementation stops short of a live
//! production feed is disclosed in that stage's own doc comment, not
//! silently implied.
//!
//! ## L1-L3 -- Process Work Functional, Pressure Clusterer, Capability Seed
//! Generator (HAND_WRITE_REQUIRED, new; no domain precedent anywhere in the
//! repo)
//!
//! [`measure_process_work`] (C1) is a real pure function over caller-supplied
//! [`WorkRegion`] measurements. It does **not** consume a live self-play
//! feed: the atlas explicitly cross-references F26 (Public Ontology Self
//! Play) and F09 (MFW Growth Operator) as this stage's real input, and
//! neither family emits a `WorkRegion`-shaped feed anywhere in this repo
//! (grep-confirmed at survey time; both are themselves separately-tracked,
//! in-progress modules in this same crate). [`cluster_high_pressure_regions`]
//! (C2) is a real, deterministic, integer-only O(n log n) clustering
//! function (all regions at or above 1.5x mean ticks, one cluster, sorted
//! descending -- no floating-point arithmetic, matching this repo's
//! determinism discipline). [`generate_capability_seed`] (C3) is real:
//! content-addresses a candidate seed id via BLAKE3 over the cluster's
//! member region ids, and is where the atlas L5 `PRESSURE_CLUSTERED ->
//! REFUSED` ("invalid") edge actually fires -- an empty cluster refuses with
//! [`CapabilityRoadmapRefused::PressureClusterInvalid`] rather than
//! proposing a seed from nothing (matching the atlas L4 refusal sequence,
//! where the boundary check happens at the Capability Seed Generator, not
//! the Clusterer itself).
//!
//! ## L1-L3 -- Counterfactual Sandbox, Scenario Corpus Replayer, Work Delta
//! Comparator (HAND_WRITE_REQUIRED, new; disclosed partial scope)
//!
//! [`inject_candidate_into_sandbox`] (C4) constructs a content-addressed,
//! independent [`CandidateEcosystem`] value (its own BLAKE3 id) from a seed
//! id and a caller-supplied corpus-snapshot digest. This is data-isolation
//! (an independent value sharing no mutable state with its inputs), **not**
//! OS/process-level sandboxing of executable code -- no code-execution
//! sandbox exists anywhere in this repo (grep-confirmed), and this module
//! does not claim one. [`replay_scenario_corpus`] (C5) verifies the
//! candidate corpus run used the identical ordered scenario-id list as the
//! baseline corpus (a real structural-identity check); a mismatch is where
//! the atlas L5 `CORPUS_REPLAYED -> REFUSED` ("authority or conformance
//! failure") edge fires, via
//! [`CapabilityRoadmapRefused::CorpusReplayFailed`]. It does **not** execute
//! the candidate itself to observe a live trace -- no live executor exists
//! anywhere in this repo (grep-confirmed); the candidate's measured
//! post-run work is caller-supplied input, disclosed as PARTIAL relative to
//! a full behavioral replay. [`compare_work_delta`] (C6) is a real
//! integer-arithmetic comparison (`contracted` iff candidate ticks are
//! strictly less than baseline ticks).
//!
//! ## L3 -- Standing Invariant Gate (HAND_WRITE_REQUIRED, real integration
//! against the *shape* of the existing standing schema, not a live parse)
//!
//! [`check_standing_invariants`] (C7) is the survey's flagged integration
//! point: rather than reimplementing invariant-checking, it refuses
//! ([`CapabilityRoadmapRefused::StandingInvariantViolated`]) iff a
//! caller-supplied [`StandingEvidence`] carries any blocking status, using
//! the real `cicd-standing.v1` blocking-status vocabulary
//! (`NON_STANDING`/`QUARANTINED`/`RETIRED` -- confirmed this session by
//! reading `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/standing/model.rs`
//! in full). It deliberately does **not** shell out to `just standing` or
//! parse a live `target/praxis-standing/standing.json` itself: that file
//! does not exist in this checkout (`test -f`, verified this session), and
//! a library function autonomously invoking a build tool would itself risk
//! this repo's own no-concurrent-`just`-invocations build-hygiene rule
//! (`CLAUDE.md`) -- especially with the many other agents' `cargo`/`just`
//! processes already running against this shared `target/` this session
//! (`ps aux`, checked before this pass). Wiring a live standing.json parse
//! into `StandingEvidence` is disclosed follow-on work, not silently
//! pretended-done.
//!
//! ## L1-L3 -- Roadmap Renderer (REUSE_ADAPT for the receipt, hand-written
//! for ranking)
//!
//! [`render_roadmap`] (C8) ranks admitted candidates by `delta_ticks`
//! descending (a real integer sort) and emits a receipted
//! [`RoadmapArtifact`] via `praxis_graphlaw::chatman::abi::Receipt::
//! from_canonical_nquads` -- the same already-tested BLAKE3-over-canonical-
//! N-Quads construction F01's Receipt Emitter reuses (`praxis-graphlaw` is
//! already a dependency of this crate, added for F06; read in full this
//! session, `crates/praxis-graphlaw/src/chatman/abi.rs:217`).
//!
//! ## L6 -- Data & Provenance (GGEN_GENERATABLE)
//!
//! [`PROV_CHAIN`] (from `f29_capability_roadmap_generated.rs`, `include!`d
//! below) is a real, ggen-generated pure data projection of the 8 typed
//! provenance entities the atlas L6 diagram names (`WorkProfile ->
//! PressureCluster -> CapabilitySeed -> CandidateEcosystem -> ScenarioCorpus
//! -> DeltaFReport -> InvariantReport -> RoadmapArtifact`), each chained to
//! its predecessor and tagged with the real Rust function that realizes it
//! (or the caller-supplied boundary it stops at). Generated by `ggen sync`
//! (`crates/ggen`, never the frozen `~/ggen`) from
//! `packs/f29-capability-roadmap-pack/ontology.ttl`'s
//! `mfwroad:ProvChainEntity` individuals via
//! `packs/f29-capability-roadmap-pack/templates/f29_capability_roadmap_generated.rs.tmpl`;
//! two independent fresh runs produced byte-identical output (`shasum -a
//! 256`, confirmed this session), matching F01/F22's own determinism check.
//! This is a pure data catalog, not enforcement --
//! [`provenance_catalog_matches_lifecycle_states`]-equivalent tests below
//! cross-check its entries against the real, hand-written enums so the two
//! cannot silently drift apart.
//!
//! ## L5 -- 9-state lifecycle (HAND_WRITE_REQUIRED, new)
//!
//! [`LifecycleState`] and [`LifecycleState::can_transition_to`] are new: no
//! existing praxis code models this exact machine. The transition table is
//! the literal, exhaustive encoding of the atlas L5 `stateDiagram-v2` (read
//! in full this session, not inferred from the survey's prose summary
//! alone): `REFUSED` reachable only from `PRESSURE_CLUSTERED` ("invalid")
//! and `CORPUS_REPLAYED` ("authority or conformance failure").
//!
//! ## L7 -- Concurrency/chaos (HAND_WRITE_REQUIRED, new, disclosed gap)
//!
//! [`IdempotencyLedger`] is genuinely hand-written, following the identical
//! `BTreeMap`-keyed (never `HashMap`, for deterministic iteration)
//! highest-admitted-sequence-per-correlation-id discipline F01's own L7 gate
//! uses -- the F29 survey found no domain-specific analog, and this
//! generalizes F01's proven pattern rather than inventing a new one. What it
//! does **not** cover (disclosed, not silently skipped): actual
//! cross-process durability -- this ledger is in-memory only in this pass,
//! same disclosed scope boundary as F01's.
//!
//! ## L8 -- Claim ceiling (HAND_WRITE_REQUIRED, new vocabulary)
//!
//! [`claim_ceiling`] is the literal implementation of the atlas L8 diagram's
//! three named claims: `AUTONOMIC_CAPABILITY_ROADMAP_PROVEN`,
//! `COUNTERFACTUAL_CORPUS_REPLAY`, `NO_DECORATIVE_PHYSICS_CLAIM` are `true`
//! only when every named evidence flag in [`ClaimEvidence`] is independently
//! supplied `true` by the caller -- this function never derives evidence
//! itself, only combines it, matching F01's own `claim_ceiling` discipline.
//!
//! Survey-cited and independently-read paths for F29 (informed research from
//! the v26.7.12 family survey handed to this scaffolding session inline,
//! plus the atlas source file and cargo-cicd standing schema this Wire pass
//! read directly rather than relying on the survey's prose summary alone):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F29_capability-roadmap.md (read in full this session)
//! - /Users/sac/capability-map/README.md
//! - /Users/sac/capability-map/PROJECT.md
//! - /Users/sac/capability-map/src/recommend.rs
//! - /Users/sac/praxis/docs/jira/v26.7.11/PATH_TO_100.md
//! - /Users/sac/praxis/packs/jira-tracking-pack/pack.toml
//! - /Users/sac/praxis/packs/f01-standing-algebra-pack/ (structural precedent for this pass's ggen pack)
//! - /Users/sac/praxis/packs/f22-recovery-pack/ (structural precedent for this pass's ggen pack)
//! - /Users/sac/praxis/crates/multifractal-workflow/src/f01_standing_algebra.rs (structural precedent for lifecycle/refusal/idempotency/replay/claim-ceiling shape)
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/abi.rs (Receipt::from_canonical_nquads, read in full this session)
//! - /Users/sac/cargo-cicd/docs/reference/standing-schema.md
//! - /Users/sac/cargo-cicd/crates/cargo-cicd-core/src/standing/model.rs (read in full this session for the real cicd-standing.v1 status vocabulary)
//! - /Users/sac/praxis/justfile (`standing` recipe, lines 160-171)

use std::collections::BTreeMap;

use praxis_graphlaw::chatman::abi::Receipt;

include!("f29_capability_roadmap_generated.rs");

/// The 8-state Thermodynamic Capability Roadmap lifecycle plus the `Refused`
/// escape state (F29 atlas L5). `Refused` is reachable only from
/// `PressureClustered` or `CorpusReplayed` -- see
/// [`LifecycleState::can_transition_to`], the literal, exhaustive encoding
/// of the atlas `stateDiagram-v2` (not merely a comment about it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    WorkMeasured = 0,
    PressureClustered = 1,
    SeedGenerated = 2,
    CandidateInjected = 3,
    CorpusReplayed = 4,
    DeltaMeasured = 5,
    InvariantsChecked = 6,
    RoadmapEmitted = 7,
    Refused = 8,
}

impl LifecycleState {
    /// Whether `self -> next` is a legal Capability Roadmap edge.
    ///
    /// Literal encoding of the atlas F29-L5 `stateDiagram-v2`: a single
    /// linear chain `WORK_MEASURED -> ... -> ROADMAP_EMITTED`, plus exactly
    /// two refusal edges (`PRESSURE_CLUSTERED -> REFUSED`,
    /// `CORPUS_REPLAYED -> REFUSED`).
    ///
    /// # Complexity
    /// O(1).
    #[must_use]
    pub fn can_transition_to(self, next: LifecycleState) -> bool {
        use LifecycleState::{
            CandidateInjected, CorpusReplayed, DeltaMeasured, InvariantsChecked, PressureClustered,
            Refused, RoadmapEmitted, SeedGenerated, WorkMeasured,
        };
        matches!(
            (self, next),
            (WorkMeasured, PressureClustered)
                | (PressureClustered, SeedGenerated)
                | (PressureClustered, Refused)
                | (SeedGenerated, CandidateInjected)
                | (CandidateInjected, CorpusReplayed)
                | (CorpusReplayed, DeltaMeasured)
                | (CorpusReplayed, Refused)
                | (DeltaMeasured, InvariantsChecked)
                | (InvariantsChecked, RoadmapEmitted)
        )
    }

    /// One-hot replay token bit for this state (L6/L8), mirroring F01's
    /// `token_bit`. `Refused` deliberately has no bit: it exits the lawful
    /// pipeline rather than continuing it.
    ///
    /// # Complexity
    /// O(1).
    fn token_bit(self) -> u64 {
        match self {
            LifecycleState::WorkMeasured => 1 << 0,
            LifecycleState::PressureClustered => 1 << 1,
            LifecycleState::SeedGenerated => 1 << 2,
            LifecycleState::CandidateInjected => 1 << 3,
            LifecycleState::CorpusReplayed => 1 << 4,
            LifecycleState::DeltaMeasured => 1 << 5,
            LifecycleState::InvariantsChecked => 1 << 6,
            LifecycleState::RoadmapEmitted => 1 << 7,
            LifecycleState::Refused => 0,
        }
    }
}

/// F29's typed refusal taxonomy (atlas: `CapabilityRoadmapRefused`). Every
/// variant carries concrete offending context, never a bare generic
/// message; every variant is REAL per [`REFUSAL_CATALOG`] (cross-checked by
/// [`refusal_catalog_matches_enum_variants`] below).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CapabilityRoadmapRefused {
    /// C3 boundary check: the pressure cluster handed to the Capability Seed
    /// Generator was empty (atlas L5: `PRESSURE_CLUSTERED -> REFUSED`,
    /// "invalid").
    #[error("PressureClusterInvalid: cluster {cluster_id:?} has no member regions; refusing to propose a capability seed from an empty cluster")]
    PressureClusterInvalid { cluster_id: String },
    /// C6 boundary check: the candidate corpus run did not replay the
    /// identical ordered scenario corpus as the baseline (atlas L5:
    /// `CORPUS_REPLAYED -> REFUSED`, "authority or conformance failure").
    #[error("CorpusReplayFailed: candidate scenario ids {candidate_scenario_ids:?} do not match baseline {baseline_scenario_ids:?}")]
    CorpusReplayFailed {
        baseline_scenario_ids: Vec<String>,
        candidate_scenario_ids: Vec<String>,
    },
    /// C7 Standing Invariant Gate: a candidate that would otherwise be
    /// admitted carries one or more blocking `cicd-standing.v1` statuses.
    #[error(
        "StandingInvariantViolated: blocking standing statuses present: {blocking_statuses:?}"
    )]
    StandingInvariantViolated { blocking_statuses: Vec<String> },
    /// An attempted lifecycle transition is not one of the legal edges in
    /// [`LifecycleState::can_transition_to`].
    #[error("illegal Capability Roadmap lifecycle transition {from:?} -> {to:?}")]
    IllegalLifecycleTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
    /// L7: a duplicate or stale (out-of-order / post-restart re-delivered)
    /// event was refused by the idempotency+correlation gate rather than
    /// silently re-admitted.
    #[error("duplicate or stale event refused (correlation_id={correlation_id:?}): {reason}")]
    DuplicateOrStaleEvent {
        correlation_id: String,
        reason: String,
    },
    /// L6/L8: replaying the recorded lifecycle chain did not reconstruct an
    /// equivalent consequence to the receipted one.
    #[error(
        "replay of the receipted roadmap chain did not reconstruct an equivalent consequence: {reason}"
    )]
    ReplayEquivalenceFailed { reason: String },
    /// The underlying `praxis_graphlaw::chatman::abi::Receipt` construction
    /// refused (e.g. empty/unsorted canonical N-Quads material); the
    /// original refusal's message is preserved verbatim in `reason`.
    #[error("receipt emission refused: {0}")]
    ReceiptEmissionRefused(String),
}

/// One measured process-work region (C1 input). Stands in for what the
/// atlas describes as "self-play and realized MFW work profiles" -- F26 and
/// F09 do not yet emit a feed shaped like this anywhere in the repo, so
/// this pass's [`measure_process_work`] is real and tested against
/// caller-supplied regions, disclosed as UNVERIFIED against a live
/// production feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRegion {
    pub region_id: String,
    pub measured_ticks: u64,
}

/// C1 output: aggregate process-work statistics over a set of
/// [`WorkRegion`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkProfile {
    pub regions: Vec<WorkRegion>,
    pub total_ticks: u64,
    pub mean_ticks: u64,
}

/// C1 Process Work Functional: aggregates `regions` into a [`WorkProfile`].
/// An empty `regions` slice yields a zeroed profile (not a refusal -- an
/// empty measurement window is a legitimate, if uninteresting, observation;
/// [`generate_capability_seed`] is where "nothing to act on" becomes a
/// refusal, per the atlas's own boundary-check placement).
///
/// # Complexity
/// O(n), n = `regions.len()` (one pass to sum, integer division for the
/// mean -- no floating-point arithmetic).
#[must_use]
pub fn measure_process_work(regions: &[WorkRegion]) -> WorkProfile {
    let total_ticks: u64 = regions.iter().map(|r| r.measured_ticks).sum();
    let mean_ticks = if regions.is_empty() {
        0
    } else {
        total_ticks / regions.len() as u64
    };
    WorkProfile {
        regions: regions.to_vec(),
        total_ticks,
        mean_ticks,
    }
}

/// C2 output: a cluster of recurring high-work regions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PressureCluster {
    pub cluster_id: String,
    pub member_region_ids: Vec<String>,
    pub cluster_ticks: u64,
}

/// C2 Pressure Clusterer: groups every region at or above 1.5x the
/// profile's mean ticks into a single cluster, ordered by `measured_ticks`
/// descending. Deliberately integer-only (`mean + mean / 2`, not `mean *
/// 1.5`) to avoid floating-point arithmetic in this pipeline, matching this
/// repo's determinism discipline.
///
/// # Complexity
/// O(n log n), n = `profile.regions.len()` (one sort).
#[must_use]
pub fn cluster_high_pressure_regions(profile: &WorkProfile) -> PressureCluster {
    let threshold = profile.mean_ticks + profile.mean_ticks / 2;
    let mut members: Vec<&WorkRegion> = profile
        .regions
        .iter()
        .filter(|r| r.measured_ticks >= threshold && threshold > 0)
        .collect();
    members.sort_by(|a, b| b.measured_ticks.cmp(&a.measured_ticks));
    let cluster_ticks: u64 = members.iter().map(|r| r.measured_ticks).sum();
    let member_region_ids: Vec<String> = members.iter().map(|r| r.region_id.clone()).collect();
    let cluster_id = hex::encode(blake3::hash(member_region_ids.join(",").as_bytes()).as_bytes());
    PressureCluster {
        cluster_id,
        member_region_ids,
        cluster_ticks,
    }
}

/// C3 output: a proposed capability seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySeed {
    pub seed_id: String,
    pub cluster_id: String,
    pub description: String,
}

/// C3 Capability Seed Generator: proposes a content-addressed
/// [`CapabilitySeed`] from `cluster`. This is where the atlas L5
/// `PRESSURE_CLUSTERED -> REFUSED` ("invalid") edge fires: a cluster with no
/// member regions is refused rather than seeding a candidate from nothing.
///
/// # Errors
/// [`CapabilityRoadmapRefused::PressureClusterInvalid`] if
/// `cluster.member_region_ids` is empty.
///
/// # Complexity
/// O(m), m = `cluster.member_region_ids.len()` (one BLAKE3 pass over the
/// joined ids plus `description`).
pub fn generate_capability_seed(
    cluster: &PressureCluster,
    description: impl Into<String>,
) -> Result<CapabilitySeed, CapabilityRoadmapRefused> {
    if cluster.member_region_ids.is_empty() {
        return Err(CapabilityRoadmapRefused::PressureClusterInvalid {
            cluster_id: cluster.cluster_id.clone(),
        });
    }
    let description = description.into();
    let seed_material = format!("{}|{}", cluster.member_region_ids.join(","), description);
    let seed_id = hex::encode(blake3::hash(seed_material.as_bytes()).as_bytes());
    Ok(CapabilitySeed {
        seed_id,
        cluster_id: cluster.cluster_id.clone(),
        description,
    })
}

/// C4 output: an isolated candidate ecosystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateEcosystem {
    pub ecosystem_id: String,
    pub seed_id: String,
    pub corpus_snapshot_digest: String,
}

/// C4 Counterfactual Sandbox: injects `seed` into isolation by constructing
/// a content-addressed [`CandidateEcosystem`] value that shares no mutable
/// state with `seed` or `corpus_snapshot_digest` -- data-isolation, not
/// OS/process-level sandboxing of executable code (no such executor exists
/// anywhere in this repo; see module doc comment). Always succeeds: the
/// atlas L5 diagram has no refusal edge leaving `CANDIDATE_INJECTED`'s
/// predecessor state for this stage.
///
/// # Complexity
/// O(1) plus one BLAKE3 pass over the two input ids.
#[must_use]
pub fn inject_candidate_into_sandbox(
    seed: &CapabilitySeed,
    corpus_snapshot_digest: &str,
) -> CandidateEcosystem {
    let material = format!("{}|{}", seed.seed_id, corpus_snapshot_digest);
    let ecosystem_id = hex::encode(blake3::hash(material.as_bytes()).as_bytes());
    CandidateEcosystem {
        ecosystem_id,
        seed_id: seed.seed_id.clone(),
        corpus_snapshot_digest: corpus_snapshot_digest.to_string(),
    }
}

/// C5 output: a scenario corpus replay record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCorpusReplay {
    pub ecosystem_id: String,
    pub scenario_ids: Vec<String>,
    pub candidate_total_ticks: u64,
}

/// C5 Scenario Corpus Replayer: verifies `candidate_scenario_ids` is
/// structurally identical (same ids, same order) to `baseline_scenario_ids`
/// -- the "identical scenario corpus" the atlas invariant requires -- and,
/// if so, records `candidate_total_ticks` (caller-measured; no live
/// executor exists in this repo to produce it directly, see module doc
/// comment). This is where the atlas L5 `CORPUS_REPLAYED -> REFUSED`
/// ("authority or conformance failure") edge fires on mismatch.
///
/// # Errors
/// [`CapabilityRoadmapRefused::CorpusReplayFailed`] if the two scenario-id
/// lists differ in length, order, or content.
///
/// # Complexity
/// O(n), n = `baseline_scenario_ids.len()` (one elementwise comparison).
pub fn replay_scenario_corpus(
    ecosystem: &CandidateEcosystem,
    baseline_scenario_ids: &[String],
    candidate_scenario_ids: &[String],
    candidate_total_ticks: u64,
) -> Result<ScenarioCorpusReplay, CapabilityRoadmapRefused> {
    if baseline_scenario_ids != candidate_scenario_ids {
        return Err(CapabilityRoadmapRefused::CorpusReplayFailed {
            baseline_scenario_ids: baseline_scenario_ids.to_vec(),
            candidate_scenario_ids: candidate_scenario_ids.to_vec(),
        });
    }
    Ok(ScenarioCorpusReplay {
        ecosystem_id: ecosystem.ecosystem_id.clone(),
        scenario_ids: candidate_scenario_ids.to_vec(),
        candidate_total_ticks,
    })
}

/// C6 output: whether candidate work contracted relative to baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaFReport {
    pub baseline_total_ticks: u64,
    pub candidate_total_ticks: u64,
    pub delta_ticks: i64,
    pub contracted: bool,
}

/// C6 Work Delta Comparator: real integer-arithmetic comparison of
/// `baseline_total_ticks` vs. `replay.candidate_total_ticks`. `contracted`
/// is `true` iff candidate ticks are strictly less than baseline ticks
/// (`delta_ticks` is `baseline - candidate`, so `contracted == (delta_ticks
/// > 0)`).
///
/// # Complexity
/// O(1).
#[must_use]
pub fn compare_work_delta(
    replay: &ScenarioCorpusReplay,
    baseline_total_ticks: u64,
) -> DeltaFReport {
    let delta_ticks = baseline_total_ticks as i64 - replay.candidate_total_ticks as i64;
    DeltaFReport {
        baseline_total_ticks,
        candidate_total_ticks: replay.candidate_total_ticks,
        delta_ticks,
        contracted: delta_ticks > 0,
    }
}

/// Caller-supplied standing evidence for [`check_standing_invariants`) (C7).
/// Field vocabulary matches the real `cicd-standing.v1` schema's blocking
/// statuses (`NON_STANDING`, `QUARANTINED`, `RETIRED` --
/// `StandingStatus` in `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/
/// standing/model.rs`, read in full this session), but this module does not
/// itself parse a live `standing.json` -- see module doc comment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StandingEvidence {
    pub blocking_statuses: Vec<String>,
}

/// C7 output: whether standing/actuation invariants are preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantReport {
    pub ecosystem_id: String,
    pub standing_preserved: bool,
}

/// C7 Standing Invariant Gate: refuses any candidate whose `evidence`
/// carries a blocking standing status, even if [`compare_work_delta`]
/// reported `contracted == true` -- the atlas CTQ's explicit "a candidate
/// that improves one metric by creating unreceipted actuation elsewhere
/// must be refused" rule.
///
/// # Errors
/// [`CapabilityRoadmapRefused::StandingInvariantViolated`] if
/// `evidence.blocking_statuses` is non-empty.
///
/// # Complexity
/// O(1) (the evidence is pre-computed by the caller; this only inspects
/// `blocking_statuses.is_empty()`).
pub fn check_standing_invariants(
    ecosystem: &CandidateEcosystem,
    evidence: &StandingEvidence,
) -> Result<InvariantReport, CapabilityRoadmapRefused> {
    if !evidence.blocking_statuses.is_empty() {
        return Err(CapabilityRoadmapRefused::StandingInvariantViolated {
            blocking_statuses: evidence.blocking_statuses.clone(),
        });
    }
    Ok(InvariantReport {
        ecosystem_id: ecosystem.ecosystem_id.clone(),
        standing_preserved: true,
    })
}

/// One ranked entry in a rendered [`RoadmapArtifact`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoadmapCandidate {
    pub seed: CapabilitySeed,
    pub delta: DeltaFReport,
}

/// C8 output: the final receipted, ranked roadmap.
#[derive(Debug, Clone)]
pub struct RoadmapArtifact {
    pub ranked: Vec<RoadmapCandidate>,
    pub receipt: Receipt,
}

/// C8 Roadmap Renderer: ranks `candidates` by `delta.delta_ticks` descending
/// (largest work contraction first; a real integer sort, no floating-point
/// scoring) and emits a receipted [`RoadmapArtifact`] via
/// `Receipt::from_canonical_nquads` -- reused, not reimplemented (see module
/// doc comment).
///
/// # Errors
/// [`CapabilityRoadmapRefused::ReceiptEmissionRefused`] if the underlying
/// receipt construction refuses (empty/unsorted material, per
/// `Receipt::from_canonical_nquads`'s own contract).
///
/// # Complexity
/// O(n log n) for the ranking sort plus O(bytes) for the receipt's BLAKE3
/// digest, n = `candidates.len()`.
pub fn render_roadmap(
    mut candidates: Vec<RoadmapCandidate>,
    witness: &str,
    replay_hint: &str,
    canon_nquads: &str,
) -> Result<RoadmapArtifact, CapabilityRoadmapRefused> {
    candidates.sort_by(|a, b| b.delta.delta_ticks.cmp(&a.delta.delta_ticks));
    let subject = format!(
        "urn:multifractal-workflow:f29:roadmap:{}",
        candidates
            .iter()
            .map(|c| c.seed.seed_id.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    let receipt = Receipt::from_canonical_nquads(&subject, witness, replay_hint, canon_nquads)
        .map_err(|e| CapabilityRoadmapRefused::ReceiptEmissionRefused(e.to_string()))?;
    Ok(RoadmapArtifact {
        ranked: candidates,
        receipt,
    })
}

/// One case moving through the Capability Roadmap pipeline: `Process Work
/// Functional -> Pressure Clusterer -> Capability Seed Generator ->
/// Counterfactual Sandbox -> Scenario Corpus Replayer -> Work Delta
/// Comparator -> Standing Invariant Gate -> Roadmap Renderer`. Every stage
/// is an explicit, typed, potentially-refusing method; there is no path
/// that advances `state` without going through one of them (mirrors F01's
/// `StandingCase` structure, generalized to F29's 8-stage machine).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoadmapCase {
    state: LifecycleState,
    transitions: Vec<(LifecycleState, LifecycleState)>,
}

impl Default for RoadmapCase {
    fn default() -> Self {
        Self::new()
    }
}

impl RoadmapCase {
    /// Opens a new case at `WorkMeasured` (the atlas's `[*] ->
    /// WORK_MEASURED` initial edge -- there is no separate pre-measurement
    /// state in the L5 diagram).
    ///
    /// # Complexity
    /// O(1).
    #[must_use]
    pub fn new() -> Self {
        RoadmapCase {
            state: LifecycleState::WorkMeasured,
            transitions: Vec::new(),
        }
    }

    /// Current lifecycle state.
    #[must_use]
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// The lawful transition history recorded so far, in order taken. Feeds
    /// [`replay_capability_roadmap_chain`]/[`verify_receipt_head`].
    #[must_use]
    pub fn transitions(&self) -> &[(LifecycleState, LifecycleState)] {
        &self.transitions
    }

    /// Advances `state` to `next` iff it is a legal edge, recording the
    /// transition for later replay.
    ///
    /// # Complexity
    /// O(1).
    fn transition(&mut self, next: LifecycleState) -> Result<(), CapabilityRoadmapRefused> {
        if !self.state.can_transition_to(next) {
            return Err(CapabilityRoadmapRefused::IllegalLifecycleTransition {
                from: self.state,
                to: next,
            });
        }
        self.transitions.push((self.state, next));
        self.state = next;
        Ok(())
    }

    /// `WorkMeasured -> PressureClustered`, after C2 has run.
    ///
    /// # Errors
    /// [`CapabilityRoadmapRefused::IllegalLifecycleTransition`] if this case
    /// is not `WorkMeasured`.
    pub fn advance_to_pressure_clustered(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::PressureClustered)
    }

    /// `PressureClustered -> SeedGenerated`, after C3 has run successfully.
    ///
    /// # Errors
    /// [`CapabilityRoadmapRefused::IllegalLifecycleTransition`] if this case
    /// is not `PressureClustered`.
    pub fn advance_to_seed_generated(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::SeedGenerated)
    }

    /// `SeedGenerated -> CandidateInjected`, after C4 has run.
    pub fn advance_to_candidate_injected(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::CandidateInjected)
    }

    /// `CandidateInjected -> CorpusReplayed`, after C5 has run successfully.
    pub fn advance_to_corpus_replayed(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::CorpusReplayed)
    }

    /// `CorpusReplayed -> DeltaMeasured`, after C6 has run.
    pub fn advance_to_delta_measured(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::DeltaMeasured)
    }

    /// `DeltaMeasured -> InvariantsChecked`, after C7 has run successfully.
    pub fn advance_to_invariants_checked(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::InvariantsChecked)
    }

    /// `InvariantsChecked -> RoadmapEmitted`, after C8 has run.
    pub fn advance_to_roadmap_emitted(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::RoadmapEmitted)
    }

    /// Exits to `Refused` from `PressureClustered` or `CorpusReplayed` (the
    /// only two legal `Refused` predecessors, atlas L5).
    ///
    /// # Errors
    /// [`CapabilityRoadmapRefused::IllegalLifecycleTransition`] from any
    /// other state.
    pub fn refuse(&mut self) -> Result<(), CapabilityRoadmapRefused> {
        self.transition(LifecycleState::Refused)
    }
}

/// L7: an atomic idempotency+correlation gate (HAND_WRITE_REQUIRED, no
/// domain-specific analog found for F29; generalizes F01's own
/// `IdempotencyLedger` pattern). Tracks the highest admitted sequence
/// number per correlation id in a `BTreeMap` (never `HashMap`, for
/// deterministic iteration); admits an event iff its sequence number is
/// strictly newer than the last admitted one for that correlation id,
/// refusing duplicates and stale/out-of-order re-delivery. Same disclosed
/// scope boundary as F01's: this ledger is in-memory only in this pass,
/// not yet wired to a persisted store surviving a real process restart.
#[derive(Debug, Clone, Default)]
pub struct IdempotencyLedger {
    seen: BTreeMap<String, u64>,
}

impl IdempotencyLedger {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        IdempotencyLedger::default()
    }

    /// Admits `(correlation_id, sequence)` iff it is strictly newer than any
    /// previously admitted sequence for that correlation id.
    ///
    /// # Errors
    /// [`CapabilityRoadmapRefused::DuplicateOrStaleEvent`] if `sequence` is
    /// less than or equal to the last admitted sequence for
    /// `correlation_id`.
    ///
    /// # Complexity
    /// O(log n), n = distinct correlation ids seen so far.
    pub fn admit(
        &mut self,
        correlation_id: &str,
        sequence: u64,
    ) -> Result<(), CapabilityRoadmapRefused> {
        if let Some(prev) = self.seen.get(correlation_id).copied() {
            if sequence <= prev {
                return Err(CapabilityRoadmapRefused::DuplicateOrStaleEvent {
                    correlation_id: correlation_id.to_string(),
                    reason: format!(
                        "sequence {sequence} is not newer than the last admitted sequence \
                         {prev} for this correlation id"
                    ),
                });
            }
        }
        self.seen.insert(correlation_id.to_string(), sequence);
        Ok(())
    }
}

/// L6/L8: replays a recorded lawful transition history through
/// [`bcinr_powl_receipt::replay::PowlReplayVerifier`] and returns the real
/// conformance metrics, or the first violation encountered. A lawful,
/// in-order Capability Roadmap chain (`WorkMeasured -> ... ->
/// RoadmapEmitted`) replays to `fitness == 0x0001_0000` (1.0, Q16.16),
/// matching F01's own `replay_standing_chain` precedent (`bcinr-powl-receipt`
/// is already a dependency of this crate, added for F01).
///
/// # Errors
/// The first `ReplayViolation` encountered (out-of-order or unknown-node
/// transition).
///
/// # Complexity
/// O(t) over `transitions`, t = chain length (each frame is O(1)).
pub fn replay_capability_roadmap_chain(
    transitions: &[(LifecycleState, LifecycleState)],
) -> Result<
    bcinr_powl_receipt::conformance::ConformanceMetrics,
    bcinr_powl_receipt::replay::ReplayViolation,
> {
    use bcinr_powl_receipt::replay::{PowlReplayFrame, PowlReplayVerifier};
    let mut verifier = PowlReplayVerifier::new(LifecycleState::WorkMeasured.token_bit());
    for (idx, (from, to)) in transitions.iter().enumerate() {
        let frame = PowlReplayFrame {
            node_id: *to as u32,
            node_bit: to.token_bit(),
            required_tokens: from.token_bit(),
            produces_tokens: to.token_bit(),
            activity: format!("{from:?}->{to:?}"),
            ts_ns: idx as u64,
            object_ids: Vec::new(),
        };
        verifier.replay_frame(&frame)?;
    }
    Ok(verifier.finalize())
}

/// L6/L8: verifies that replaying `transitions` reconstructs a consequence
/// equivalent to `receipt` -- the receipt head and the replay must agree,
/// not merely both exist. "Equivalent" means: replay is lawful (fitness ==
/// 1.0, Q16.16) *and* the receipt's own digest recomputes (`Receipt::
/// verify`).
///
/// # Errors
/// [`CapabilityRoadmapRefused::ReplayEquivalenceFailed`] if replay is a
/// `ReplayViolation`, if fitness is not `0x0001_0000`, or if
/// `receipt.verify()` fails.
///
/// # Complexity
/// O(t) for the replay plus O(bytes) for the receipt digest recompute.
pub fn verify_receipt_head(
    transitions: &[(LifecycleState, LifecycleState)],
    receipt: &Receipt,
) -> Result<bcinr_powl_receipt::conformance::ConformanceMetrics, CapabilityRoadmapRefused> {
    let metrics = replay_capability_roadmap_chain(transitions).map_err(|violation| {
        CapabilityRoadmapRefused::ReplayEquivalenceFailed {
            reason: format!("{violation:?}"),
        }
    })?;
    if metrics.fitness != 0x0001_0000 {
        return Err(CapabilityRoadmapRefused::ReplayEquivalenceFailed {
            reason: format!(
                "replay fitness {:#010x} is not 1.0; the roadmap chain does not reconstruct \
                 equivalently",
                metrics.fitness
            ),
        });
    }
    receipt
        .verify()
        .map_err(|e| CapabilityRoadmapRefused::ReplayEquivalenceFailed {
            reason: e.to_string(),
        })?;
    Ok(metrics)
}

/// L8: the independently-supplied evidence flags [`claim_ceiling`] combines,
/// matching the atlas L8 diagram's five inputs (`SRC`, `TEST`, `REACH`,
/// `CHAOS`, `RECEIPT`). None of these ever defaults to `true`; every field
/// must be set by the caller from real evidence gathered elsewhere, never
/// derived by `claim_ceiling` itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClaimEvidence {
    pub source_and_graph_evidence_present: bool,
    pub unit_and_negative_fixtures_present: bool,
    pub production_reachability_trace_present: bool,
    pub chaos_recovery_evidence_present: bool,
    pub receipt_replay_equivalence_verified: bool,
}

/// L8: the only three claims this family may ever emit, matching the atlas
/// L8 diagram's `M1`/`M2`/`M3` exactly:
/// `AUTONOMIC_CAPABILITY_ROADMAP_PROVEN`, `COUNTERFACTUAL_CORPUS_REPLAY`,
/// `NO_DECORATIVE_PHYSICS_CLAIM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimCeiling {
    pub autonomic_capability_roadmap_proven: bool,
    pub counterfactual_corpus_replay: bool,
    pub no_decorative_physics_claim: bool,
}

/// Computes the L8 claim ceiling from independently-supplied evidence. All
/// three claims require every [`ClaimEvidence`] flag to be `true`;
/// `counterfactual_corpus_replay` additionally re-checks
/// `receipt_replay_equivalence_verified` explicitly (redundant with the
/// combined check today, but kept as its own named condition so a future
/// loosening of the combined gate cannot silently loosen this specific
/// claim too -- same discipline as F01's `claim_ceiling`).
///
/// # Complexity
/// O(1).
#[must_use]
pub fn claim_ceiling(evidence: ClaimEvidence) -> ClaimCeiling {
    let all_present = evidence.source_and_graph_evidence_present
        && evidence.unit_and_negative_fixtures_present
        && evidence.production_reachability_trace_present
        && evidence.chaos_recovery_evidence_present
        && evidence.receipt_replay_equivalence_verified;
    ClaimCeiling {
        autonomic_capability_roadmap_proven: all_present,
        counterfactual_corpus_replay: all_present && evidence.receipt_replay_equivalence_verified,
        no_decorative_physics_claim: all_present,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_regions() -> Vec<WorkRegion> {
        vec![
            WorkRegion {
                region_id: "region-a".to_string(),
                measured_ticks: 100,
            },
            WorkRegion {
                region_id: "region-b".to_string(),
                measured_ticks: 10,
            },
            WorkRegion {
                region_id: "region-c".to_string(),
                measured_ticks: 90,
            },
        ]
    }

    #[test]
    fn measure_process_work_computes_total_and_mean() {
        let profile = measure_process_work(&sample_regions());
        assert_eq!(profile.total_ticks, 200);
        assert_eq!(profile.mean_ticks, 66); // integer division: 200 / 3
        assert_eq!(profile.regions.len(), 3);
    }

    #[test]
    fn measure_process_work_handles_empty_input() {
        let profile = measure_process_work(&[]);
        assert_eq!(profile.total_ticks, 0);
        assert_eq!(profile.mean_ticks, 0);
    }

    #[test]
    fn cluster_high_pressure_regions_groups_above_threshold_descending() {
        let profile = measure_process_work(&sample_regions());
        // mean = 66, threshold = 66 + 33 = 99: only region-a (100) clears it.
        let cluster = cluster_high_pressure_regions(&profile);
        assert_eq!(cluster.member_region_ids, vec!["region-a".to_string()]);
        assert_eq!(cluster.cluster_ticks, 100);
    }

    #[test]
    fn cluster_high_pressure_regions_empty_profile_yields_empty_cluster() {
        let profile = measure_process_work(&[]);
        let cluster = cluster_high_pressure_regions(&profile);
        assert!(cluster.member_region_ids.is_empty());
    }

    #[test]
    fn generate_capability_seed_refuses_empty_cluster() {
        let empty_cluster = PressureCluster {
            cluster_id: "empty".to_string(),
            member_region_ids: Vec::new(),
            cluster_ticks: 0,
        };
        let result = generate_capability_seed(&empty_cluster, "candidate description");
        assert!(matches!(
            result,
            Err(CapabilityRoadmapRefused::PressureClusterInvalid { .. })
        ));
    }

    #[test]
    fn generate_capability_seed_succeeds_on_nonempty_cluster_and_is_deterministic() {
        let cluster = PressureCluster {
            cluster_id: "cluster-1".to_string(),
            member_region_ids: vec!["region-a".to_string()],
            cluster_ticks: 100,
        };
        let seed1 = generate_capability_seed(&cluster, "desc").expect("must succeed");
        let seed2 = generate_capability_seed(&cluster, "desc").expect("must succeed");
        assert_eq!(
            seed1.seed_id, seed2.seed_id,
            "seed id must be deterministic"
        );
        assert!(!seed1.seed_id.is_empty());
    }

    #[test]
    fn inject_candidate_into_sandbox_is_content_addressed_and_deterministic() {
        let seed = CapabilitySeed {
            seed_id: "seed-1".to_string(),
            cluster_id: "cluster-1".to_string(),
            description: "desc".to_string(),
        };
        let eco1 = inject_candidate_into_sandbox(&seed, "corpus-digest-abc");
        let eco2 = inject_candidate_into_sandbox(&seed, "corpus-digest-abc");
        assert_eq!(eco1.ecosystem_id, eco2.ecosystem_id);
        let eco3 = inject_candidate_into_sandbox(&seed, "corpus-digest-xyz");
        assert_ne!(
            eco1.ecosystem_id, eco3.ecosystem_id,
            "different corpus snapshot must yield a different isolated ecosystem id"
        );
    }

    #[test]
    fn replay_scenario_corpus_refuses_on_scenario_mismatch() {
        let eco = CandidateEcosystem {
            ecosystem_id: "eco-1".to_string(),
            seed_id: "seed-1".to_string(),
            corpus_snapshot_digest: "digest".to_string(),
        };
        let baseline = vec!["scn-1".to_string(), "scn-2".to_string()];
        let candidate = vec!["scn-1".to_string(), "scn-3".to_string()];
        let result = replay_scenario_corpus(&eco, &baseline, &candidate, 50);
        assert!(matches!(
            result,
            Err(CapabilityRoadmapRefused::CorpusReplayFailed { .. })
        ));
    }

    #[test]
    fn replay_scenario_corpus_succeeds_on_identical_corpus() {
        let eco = CandidateEcosystem {
            ecosystem_id: "eco-1".to_string(),
            seed_id: "seed-1".to_string(),
            corpus_snapshot_digest: "digest".to_string(),
        };
        let scenarios = vec!["scn-1".to_string(), "scn-2".to_string()];
        let replay = replay_scenario_corpus(&eco, &scenarios, &scenarios, 50)
            .expect("identical corpus must replay successfully");
        assert_eq!(replay.candidate_total_ticks, 50);
    }

    #[test]
    fn compare_work_delta_flags_contraction() {
        let replay = ScenarioCorpusReplay {
            ecosystem_id: "eco-1".to_string(),
            scenario_ids: vec!["scn-1".to_string()],
            candidate_total_ticks: 40,
        };
        let delta = compare_work_delta(&replay, 100);
        assert_eq!(delta.delta_ticks, 60);
        assert!(delta.contracted);
    }

    #[test]
    fn compare_work_delta_flags_non_contraction() {
        let replay = ScenarioCorpusReplay {
            ecosystem_id: "eco-1".to_string(),
            scenario_ids: vec!["scn-1".to_string()],
            candidate_total_ticks: 150,
        };
        let delta = compare_work_delta(&replay, 100);
        assert_eq!(delta.delta_ticks, -50);
        assert!(!delta.contracted);
    }

    #[test]
    fn check_standing_invariants_refuses_on_blocking_status() {
        let eco = CandidateEcosystem {
            ecosystem_id: "eco-1".to_string(),
            seed_id: "seed-1".to_string(),
            corpus_snapshot_digest: "digest".to_string(),
        };
        let evidence = StandingEvidence {
            blocking_statuses: vec!["QUARANTINED".to_string()],
        };
        let result = check_standing_invariants(&eco, &evidence);
        assert!(matches!(
            result,
            Err(CapabilityRoadmapRefused::StandingInvariantViolated { .. })
        ));
    }

    #[test]
    fn check_standing_invariants_admits_clean_evidence() {
        let eco = CandidateEcosystem {
            ecosystem_id: "eco-1".to_string(),
            seed_id: "seed-1".to_string(),
            corpus_snapshot_digest: "digest".to_string(),
        };
        let evidence = StandingEvidence::default();
        let report = check_standing_invariants(&eco, &evidence).expect("clean evidence must admit");
        assert!(report.standing_preserved);
    }

    #[test]
    fn render_roadmap_ranks_by_delta_descending() {
        let make_candidate = |seed_id: &str, delta_ticks: i64| RoadmapCandidate {
            seed: CapabilitySeed {
                seed_id: seed_id.to_string(),
                cluster_id: "cluster-1".to_string(),
                description: "desc".to_string(),
            },
            delta: DeltaFReport {
                baseline_total_ticks: 100,
                candidate_total_ticks: (100 - delta_ticks) as u64,
                delta_ticks,
                contracted: delta_ticks > 0,
            },
        };
        let candidates = vec![
            make_candidate("seed-low", 10),
            make_candidate("seed-high", 90),
            make_candidate("seed-mid", 50),
        ];
        let artifact = render_roadmap(
            candidates,
            "witness-1",
            "replay-hint-1",
            "<urn:a> <urn:b> <urn:c> .\n",
        )
        .expect("render_roadmap must succeed on valid canonical N-Quads");
        let ranked_ids: Vec<&str> = artifact
            .ranked
            .iter()
            .map(|c| c.seed.seed_id.as_str())
            .collect();
        assert_eq!(ranked_ids, vec!["seed-high", "seed-mid", "seed-low"]);
    }

    #[test]
    fn full_pipeline_reaches_roadmap_emitted_and_receipt_verifies() {
        let regions = sample_regions();
        let profile = measure_process_work(&regions);
        let cluster = cluster_high_pressure_regions(&profile);

        let mut case = RoadmapCase::new();
        assert_eq!(case.state(), LifecycleState::WorkMeasured);
        case.advance_to_pressure_clustered()
            .expect("must advance to PressureClustered");

        let seed = generate_capability_seed(&cluster, "reduce region-a pressure")
            .expect("nonempty cluster must generate a seed");
        case.advance_to_seed_generated()
            .expect("must advance to SeedGenerated");

        let ecosystem = inject_candidate_into_sandbox(&seed, "corpus-snapshot-digest");
        case.advance_to_candidate_injected()
            .expect("must advance to CandidateInjected");

        let scenarios = vec!["scn-1".to_string(), "scn-2".to_string()];
        let replay = replay_scenario_corpus(&ecosystem, &scenarios, &scenarios, 40)
            .expect("identical corpus must replay");
        case.advance_to_corpus_replayed()
            .expect("must advance to CorpusReplayed");

        let delta = compare_work_delta(&replay, profile.total_ticks);
        assert!(delta.contracted);
        case.advance_to_delta_measured()
            .expect("must advance to DeltaMeasured");

        let invariant_report = check_standing_invariants(&ecosystem, &StandingEvidence::default())
            .expect("clean evidence must admit");
        assert!(invariant_report.standing_preserved);
        case.advance_to_invariants_checked()
            .expect("must advance to InvariantsChecked");

        let artifact = render_roadmap(
            vec![RoadmapCandidate { seed, delta }],
            "witness-full",
            "replay-hint-full",
            "<urn:a> <urn:b> <urn:c> .\n",
        )
        .expect("render_roadmap must succeed");
        case.advance_to_roadmap_emitted()
            .expect("must advance to RoadmapEmitted");
        assert_eq!(case.state(), LifecycleState::RoadmapEmitted);

        let metrics = verify_receipt_head(case.transitions(), &artifact.receipt)
            .expect("lawful chain + valid receipt must verify");
        assert_eq!(metrics.fitness, 0x0001_0000);
    }

    #[test]
    fn refused_only_reachable_from_pressure_clustered_or_corpus_replayed() {
        // From WorkMeasured: illegal.
        let mut wm_case = RoadmapCase::new();
        assert!(matches!(
            wm_case.refuse(),
            Err(CapabilityRoadmapRefused::IllegalLifecycleTransition {
                from: LifecycleState::WorkMeasured,
                to: LifecycleState::Refused,
            })
        ));

        // From PressureClustered: legal.
        let mut pc_case = RoadmapCase::new();
        pc_case
            .advance_to_pressure_clustered()
            .expect("must advance");
        assert!(pc_case.refuse().is_ok());
        assert_eq!(pc_case.state(), LifecycleState::Refused);

        // From CorpusReplayed: legal.
        let mut cr_case = RoadmapCase::new();
        cr_case
            .advance_to_pressure_clustered()
            .expect("must advance");
        cr_case.advance_to_seed_generated().expect("must advance");
        cr_case
            .advance_to_candidate_injected()
            .expect("must advance");
        cr_case.advance_to_corpus_replayed().expect("must advance");
        assert!(cr_case.refuse().is_ok());
        assert_eq!(cr_case.state(), LifecycleState::Refused);

        // From SeedGenerated: illegal (not a named refusal edge).
        let mut sg_case = RoadmapCase::new();
        sg_case
            .advance_to_pressure_clustered()
            .expect("must advance");
        sg_case.advance_to_seed_generated().expect("must advance");
        assert!(matches!(
            sg_case.refuse(),
            Err(CapabilityRoadmapRefused::IllegalLifecycleTransition {
                from: LifecycleState::SeedGenerated,
                to: LifecycleState::Refused,
            })
        ));
    }

    #[test]
    fn idempotency_ledger_admits_strictly_increasing_sequence() {
        let mut ledger = IdempotencyLedger::new();
        assert!(ledger.admit("corr-1", 1).is_ok());
        assert!(ledger.admit("corr-1", 2).is_ok());
        assert!(ledger.admit("corr-2", 1).is_ok());
    }

    #[test]
    fn idempotency_ledger_refuses_duplicate_event() {
        let mut ledger = IdempotencyLedger::new();
        ledger.admit("corr-1", 5).expect("first admit must succeed");
        let result = ledger.admit("corr-1", 5);
        assert!(matches!(
            result,
            Err(CapabilityRoadmapRefused::DuplicateOrStaleEvent { .. })
        ));
    }

    #[test]
    fn idempotency_ledger_refuses_stale_redelivery_after_restart() {
        // Simulates a process restart re-delivering an older event: a fresh
        // admit() call against the same durable ledger state must still
        // refuse, since the ledger (not in-process memory) is what's
        // consulted.
        let mut ledger = IdempotencyLedger::new();
        ledger.admit("corr-1", 10).expect("must admit sequence 10");
        let stale_redelivery = ledger.admit("corr-1", 3);
        assert!(matches!(
            stale_redelivery,
            Err(CapabilityRoadmapRefused::DuplicateOrStaleEvent { .. })
        ));
    }

    #[test]
    fn replay_capability_roadmap_chain_out_of_order_is_violation() {
        // Seeds the verifier with only WorkMeasured's token enabled. A frame
        // requiring PressureClustered's token (i.e. one that never went
        // through WorkMeasured -> PressureClustered first) must be
        // rejected, not silently accepted.
        let transitions = [(
            LifecycleState::PressureClustered,
            LifecycleState::SeedGenerated,
        )];
        let result = replay_capability_roadmap_chain(&transitions);
        assert!(matches!(
            result,
            Err(bcinr_powl_receipt::replay::ReplayViolation::TokenNotEnabled { .. })
        ));
    }

    #[test]
    fn claim_ceiling_is_false_unless_all_evidence_present() {
        let none = claim_ceiling(ClaimEvidence::default());
        assert!(!none.autonomic_capability_roadmap_proven);
        assert!(!none.counterfactual_corpus_replay);
        assert!(!none.no_decorative_physics_claim);

        let partial = claim_ceiling(ClaimEvidence {
            source_and_graph_evidence_present: true,
            unit_and_negative_fixtures_present: true,
            production_reachability_trace_present: true,
            chaos_recovery_evidence_present: true,
            receipt_replay_equivalence_verified: false,
        });
        assert!(!partial.autonomic_capability_roadmap_proven);

        let full = claim_ceiling(ClaimEvidence {
            source_and_graph_evidence_present: true,
            unit_and_negative_fixtures_present: true,
            production_reachability_trace_present: true,
            chaos_recovery_evidence_present: true,
            receipt_replay_equivalence_verified: true,
        });
        assert!(full.autonomic_capability_roadmap_proven);
        assert!(full.counterfactual_corpus_replay);
        assert!(full.no_decorative_physics_claim);
    }

    /// Cross-checks the ggen-generated [`PROV_CHAIN`]'s `pipeline_stage`
    /// entries against the real, hand-written stage functions by name, so
    /// the generated catalog and the enforcement code cannot silently drift
    /// apart (same discipline as F01's `provenance_catalog_matches_
    /// lifecycle_states`).
    #[test]
    fn provenance_chain_is_eight_entities_in_order() {
        assert_eq!(PROV_CHAIN.len(), 8);
        for (i, entry) in PROV_CHAIN.iter().enumerate() {
            assert_eq!(entry.chain_order as usize, i + 1);
        }
        assert_eq!(PROV_CHAIN[0].derived_from, "none");
        for i in 1..PROV_CHAIN.len() {
            assert_eq!(PROV_CHAIN[i].derived_from, PROV_CHAIN[i - 1].name);
        }
    }

    /// The generated 9-entry lifecycle catalog's state names match the real
    /// `LifecycleState` enum's `Debug` names exactly.
    #[test]
    fn lifecycle_catalog_matches_lifecycle_state_enum() {
        let known_state_names: std::collections::BTreeSet<String> = [
            LifecycleState::WorkMeasured,
            LifecycleState::PressureClustered,
            LifecycleState::SeedGenerated,
            LifecycleState::CandidateInjected,
            LifecycleState::CorpusReplayed,
            LifecycleState::DeltaMeasured,
            LifecycleState::InvariantsChecked,
            LifecycleState::RoadmapEmitted,
            LifecycleState::Refused,
        ]
        .iter()
        .map(|s| screaming_snake_case(&format!("{s:?}")))
        .collect();
        assert_eq!(LIFECYCLE_STATE_CATALOG.len(), 9);
        for entry in LIFECYCLE_STATE_CATALOG {
            assert!(
                known_state_names.contains(entry.name),
                "LIFECYCLE_STATE_CATALOG entry {:?} is not a real LifecycleState variant",
                entry.name
            );
        }
    }

    /// The generated refusal catalog's variant names match the real
    /// `CapabilityRoadmapRefused` enum's variant names exactly (7 variants).
    #[test]
    fn refusal_catalog_matches_enum_variants() {
        let known_variant_names: std::collections::BTreeSet<&str> = [
            "PressureClusterInvalid",
            "CorpusReplayFailed",
            "StandingInvariantViolated",
            "IllegalLifecycleTransition",
            "DuplicateOrStaleEvent",
            "ReplayEquivalenceFailed",
            "ReceiptEmissionRefused",
        ]
        .into_iter()
        .collect();
        assert_eq!(REFUSAL_CATALOG.len(), 7);
        for entry in REFUSAL_CATALOG {
            assert!(
                known_variant_names.contains(entry.name),
                "REFUSAL_CATALOG entry {:?} is not a real CapabilityRoadmapRefused variant",
                entry.name
            );
        }
    }

    /// Converts a Rust `Debug`-style `PascalCase` name (e.g. `WorkMeasured`)
    /// to the ontology's `SCREAMING_SNAKE_CASE` convention (e.g.
    /// `WORK_MEASURED`) for cross-checking against [`LIFECYCLE_STATE_CATALOG`].
    fn screaming_snake_case(pascal: &str) -> String {
        let mut out = String::new();
        for (i, c) in pascal.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_uppercase());
        }
        out
    }
}
