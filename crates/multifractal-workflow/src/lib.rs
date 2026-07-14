//! `multifractal-workflow` -- v26.7.12 architecture atlas scaffolding crate.
//!
//! One module per architecture family (30 families x 8 lenses, per the v26.7.12
//! mermaid atlas at `/Users/sac/Downloads/v26.7.12_mermaid_atlas/`). This crate was
//! scaffolded in a single Wire-phase-0 pass from a 30-family research survey handed to
//! that scaffolding session inline (not itself a checked-in repo doc); every module body
//! is an honestly-labeled placeholder -- see each module's own doc comment for its
//! family's survey verdict and cited paths. No family's real admission, lifecycle,
//! receipt, or refusal logic is implemented yet; that is later Wire-phase work, informed
//! per-family by the REUSE_ADAPT / GGEN_GENERATABLE / HAND_WRITE_REQUIRED breakdown the
//! survey already produced for each family.
//!
//! Ticket numbering: `V12-0XX` follows the atlas's own per-family ticket pattern
//! (`V12-0` + zero-padded family number). This was directly confirmed in the survey text
//! for most families (e.g. F01/V12-001, F09/V12-009, F20/V12-020, F30/V12-030) and applied
//! uniformly to the remaining families by the same pattern; treat any not spelled out
//! verbatim in the family's own doc comment as inferred-by-pattern, not independently
//! re-confirmed against the atlas file itself.
//!
//! | Module | Family | Name | Ticket | Verdict |
//! |---|---|---|---|---|
//! | `f01_standing_algebra` | F01 | Standing Algebra | V12-001 | MIXED |
//! | `f02_observation_admission` | F02 | Observation Admission | V12-002 | MIXED |
//! | `f03_semantic_contraction` | F03 | Semantic Contraction | V12-003 | MIXED |
//! | `f04_dialect_registry` | F04 | GraphLaw Dialect Registry | V12-004 | MIXED |
//! | `f05_datalog_closure` | F05 | Datalog Closure | V12-005 | MIXED |
//! | `f06_n3_quarantine` | F06 | N3 Quarantine and Refinement | V12-006 | MIXED |
//! | `f07_shape_admission` | F07 | SHACL and ShEx Admission | V12-007 | MIXED |
//! | `f08_pddl_planning` | F08 | PDDL Planning and Action-Hook Binding | V12-008 | MIXED |
//! | `f09_mfw_growth` | F09 | MFW Growth Operator | V12-009 | MIXED |
//! | `f10_powl_geometry` | F10 | POWL Recursive Process Geometry | V12-010 | MIXED |
//! | `f11_bcinr_runtime` | F11 | BCINR Local Runtime | V12-011 | MIXED |
//! | `f12_external_cut` | F12 | POWL External Cut and Projection | V12-012 | ALREADY_BUILT |
//! | `f13_arazzo_artifact` | F13 | Arazzo Generated Artifact | V12-013 | MIXED |
//! | `f14_wasm4pm_arazzo` | F14 | wasm4pm Arazzo Compiler | V12-014 | MIXED |
//! | `f15_air_transition_core` | F15 | AIR Single Semantic Core (shared OTP/AtomVM transition machine) | V12-015 | MIXED |
//! | `f16_otp_runner` | F16 | Erlang OTP Outer Runner | V12-016 | MIXED |
//! | `f17_atomvm_runtime` | F17 | AtomVM Edge Runtime | V12-017 | MIXED |
//! | `f18_broker_law` | F18 | Broker and Zero Unreceipted Actuation | V12-018 | MIXED |
//! | `f19_hooks` | F19 | Hooks and Action-Capability Resolution | V12-019 | MIXED |
//! | `f20_external_dispatch` | F20 | External Dispatch and Re-admission | V12-020 | ALREADY_BUILT |
//! | `f21_parent_child_closure` | F21 | Parent-Child Closure | V12-021 | MIXED |
//! | `f22_compensation` | F22 | Compensation and Recovery | V12-022 | MIXED |
//! | `f23_otel_rdf` | F23 | OpenTelemetry RDF Admission | V12-023 | MIXED |
//! | `f24_ocel_construct` | F24 | OCEL CONSTRUCT Capitalization | V12-024 | MIXED |
//! | `f25_receipts_replay` | F25 | Receipts and Replay | V12-025 | MIXED |
//! | `f26_ontology_self_play` | F26 | Public Ontology Self Play | V12-026 | MIXED |
//! | `f27_western_electric` | F27 | Western Electric Process Signal | V12-027 | MIXED |
//! | `f28_multi_breed_science` | F28 | Multi-Breed Executable Process Science | V12-028 | MIXED |
//! | `f29_capability_roadmap` | F29 | Thermodynamic Capability Roadmap | V12-029 | MIXED |
//! | `f30_ggen_release_state` | F30 | GGEN Dynamic Project State and Release Admission | V12-030 | MIXED |

pub mod f01_standing_algebra;
pub mod f02_observation_admission;
pub mod f03_semantic_contraction;
pub mod f04_dialect_registry;
pub mod f05_datalog_closure;
pub mod f06_n3_quarantine;
pub mod f07_shape_admission;
pub mod f08_pddl_planning;
pub mod f09_mfw_growth;
pub mod f10_powl_geometry;
pub mod f11_bcinr_runtime;
pub mod f12_external_cut;
pub mod f13_arazzo_artifact;
pub mod f14_wasm4pm_arazzo;
pub mod f15_air_transition_core;
pub mod f16_otp_runner;
pub mod f17_atomvm_runtime;
pub mod f18_broker_law;
pub mod f19_hooks;
pub mod f20_external_dispatch;
pub mod f21_parent_child_closure;
pub mod f22_compensation;
pub mod f23_otel_rdf;
pub mod f24_ocel_construct;
pub mod f25_receipts_replay;
pub mod f26_ontology_self_play;
pub mod f27_western_electric;
pub mod f28_multi_breed_science;
pub mod f29_capability_roadmap;
pub mod f30_ggen_release_state;

/// F31 -- "Org Merge" (not one of the 30 atlas families; new work). Closes the
/// first real, working instance of
/// `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` item 14 (multi-org
/// first-merge bootstrap): two independently F02-admitted organization graphs
/// fused into one re-validated merged graph, with real deterministic
/// identifier-collision detection and a disclosed union-of-shapes governance
/// rule. See the module's own doc comment for the full design and its honest
/// scope boundary.
pub mod f31_org_merge;

pub mod crown_external;
/// Composed crown-witness pipelines (real production callers that chain multiple families'
/// real entry points end to end). `crown_local` drives the LOCAL witness prefix
/// F02 -> F03 -> F08 -> F09 -> F10 in one real call; `crown_external` drives the EXTERNAL witness
/// tail F10 -> F12 -> F13 -> F14 -> F15 in one real call, stopping honestly at the F15 -> F16
/// Erlang OTP-runner boundary.
pub mod crown_local;

/// Standalone Wire-phase-0 analysis module, **not** part of the 30-family v26.7.12 atlas
/// above and not counted toward any family's ticket/verdict table or crown-witness edge
/// count. Implements the failure-process framework (t_err/t_lock/t_obs, fix window,
/// observability lag) from arXiv:2607.09510 (Zhao et al., "Failure as a Process: An
/// Anatomy of CLI Coding Agent Trajectories") against this repo's own commit history,
/// applied to this repo's real F18->F19 crown-witness overclaim as a worked example. No
/// caller wires this module into `crown_local`/`crown_external` as of this writing --
/// disclosed, not hidden. See the module's own doc comment for full scope and limitations.
pub mod trajectory_failure_process;
