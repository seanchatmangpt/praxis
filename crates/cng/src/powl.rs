//! Minimal POWL 2.0 model, projection, and Turtle serialization for the cng
//! CLI. Clean-room implementation of the invariants proven in the praxis
//! test surface (chatman_pddl_to_powl_*): one `ActivityLeaf` per plan op in
//! tape order; the order relation stored pre-closed (transitive closure,
//! `(i, j)` for all `i < j`); deterministic structural IRIs minted from a
//! base IRI (`<base>/n0`, `<base>/n0/c<i>`, `<base>/n0/binding/<i>`);
//! `powl2:derivedFrom` attached to the root model node only. Same inputs
//! produce byte-identical Turtle.

use std::collections::{BTreeMap, BTreeSet};

use bcinr_pddl::Pddl8Tape;

/// POWL 2.0 vocabulary namespace.
pub const POWL2_PREFIX: &str = "https://truex.io/ontology/powl2#";

/// The subset of POWL 2.0 the linear projection can produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Powl {
    /// An activity leaf; `None` is a silent leaf (never produced by the
    /// linear projection, kept for model completeness).
    Leaf(Option<String>),
    /// A strict partial order over child models; `order` is stored
    /// transitively closed.
    PartialOrder {
        children: Vec<Powl>,
        order: BTreeSet<(usize, usize)>,
    },
}

/// Typed refusal algebra for the whole μ pipeline. Release law: for any
/// admitted artifact set, cng either manufactures a valid POWL v2 artifact
/// (with provenance, determinism, validation, and runner evidence) or emits
/// exactly one of these refusals with its stable code. There is no third
/// state — no silent fallback, no placeholder output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CngRefusal {
    /// `CNG_R01` — an input artifact is not valid RDF/Turtle, or a PDDL
    /// literal inside it fails to parse.
    MalformedTtl(String),
    /// `CNG_R02` — no PDDL domain fragment exists in the admitted set.
    MissingDomain(String),
    /// `CNG_R03` — no PDDL problem fragment exists in the admitted set.
    MissingProblem(String),
    /// `CNG_R04` — the merged planning surface admits no plan (empty tape,
    /// unreachable goal, empty grounding).
    PlanUnsolvable(String),
    /// `CNG_R05` — a construct the pipeline does not support (mismatched
    /// domain names, duplicate actions, nested/branching POWL, >64-op tape).
    UnsupportedConstruct(String),
    /// `CNG_R06` — a POWL graph fails parsing or shape validation.
    InvalidPowl(String),
    /// `CNG_R07` — the bcinr-powl runner refused or its execution did not
    /// conform to the projected order.
    RunnerMismatch(String),
    /// `CNG_R08` — repeated manufacture produced different bytes.
    Nondeterminism(String),
    /// `CNG_R09` — the output does not reflect the admitted plan (canned or
    /// detached output suspected).
    HardcodingSuspicion(String),
    /// `CNG_R10` — filesystem input/output was refused by the OS.
    IoRefused(String),
    /// `CNG_R11` — an independent audit replay recomputed a digest that does
    /// not match the recorded one, or a bundle input named by the manifest is
    /// missing/altered. Distinct from `CNG_R08 Nondeterminism` (same-producer
    /// re-manufacture drift): R11 is third-party integrity failure detected
    /// against recorded evidence.
    AuditMismatch(String),
    /// `CNG_R12` — the standing next-action query returned a number of
    /// lawful candidate actions other than exactly one while work remains
    /// at the given logical tick. The single-operator workday loop must
    /// always be able to derive ONE lawful next action from standing; zero
    /// candidates with open work, or more than one, is a governance
    /// failure, never a heuristic choice.
    StandingAmbiguous {
        /// Logical tick at which the standing query was evaluated.
        tick: usize,
        /// Number of candidate rows the standing query returned.
        candidate_count: usize,
    },
    /// `CNG_R13` — an executed workflow transition produced no matching
    /// hook receipt (zero-unreceipted-actuation law: `actuate(c) ⟹ ∃R
    /// (R⊢c)`). The workday hook broker must obtain exactly one graphlaw
    /// `HookReceipt` (hook name == workload category) per transition; a
    /// missing receipt means the actuation was unlawful, never a warning.
    UnreceiptedActuation {
        /// The workflow instance (tick set id) whose transition actuated.
        workflow: String,
        /// The workload category whose hook failed to receipt.
        category: String,
    },
    /// `CNG_R14` — the Dialect Registry failed its closed-shape law
    /// (dialect-registry.shape.ttl: all eight registry fields mandatory,
    /// sh:closed) BEFORE any tick executed. `missing` names either the
    /// absent required field or the undeclared (closedness-violating)
    /// property on `entry`.
    DialectRegistryRefused {
        /// The registry entry IRI that violated the shape.
        entry: String,
        /// The missing required field, or the unexpected property.
        missing: String,
    },
    /// `CNG_R15` — a dispatch contract is incomplete: one or more of the 21
    /// required contract fields (dispatch-shapes.ttl DispatchContractShape)
    /// is missing or empty. Refused BEFORE the contract leaves the broker —
    /// an incomplete contract is never written to the outbox.
    DispatchContractIncomplete {
        /// The dispatch id (or contract IRI) of the incomplete contract.
        dispatch: String,
        /// Comma-separated names of the missing/empty required fields.
        missing: String,
    },
    /// `CNG_R16` — a dispatch state transition outside the lawful 16-state
    /// machine (dispatch-shapes.ttl disp:DispatchState individuals; the
    /// transition table is documented on `bench::dispatch::DispatchState`).
    /// An unlawful transition is a broker bug surfacing as a typed refusal,
    /// never a silent state overwrite.
    DispatchStateUnlawful {
        /// The dispatch id whose state machine was violated.
        dispatch: String,
        /// The current state name.
        from: String,
        /// The requested (unlawful) target state name.
        to: String,
    },
    /// `CNG_R17` — an externally produced consequence failed the lawful
    /// re-entry pipeline at the named stage (in enforced order: provenance →
    /// correlation → authority → structural → semantic). The external result
    /// never touches standing before admission; a refused consequence is
    /// evidence, never input.
    ExternalConsequenceRefused {
        /// The dispatch id the consequence claims to answer.
        dispatch: String,
        /// The re-entry stage that refused (provenance | correlation |
        /// authority | structural | semantic).
        stage: String,
    },
    /// `CNG_R18` — an admitted Arazzo description uses a feature outside the
    /// 80/20 profile (arazzo-shapes.ttl), named explicitly. Refusing by name
    /// is the profile doctrine: unsupported spec surface is REFUSED, never
    /// silently skipped.
    ArazzoProfileRefused {
        /// The refused feature, named (e.g. `criterionType=xpath`).
        feature: String,
    },
    /// `CNG_R19` — a graph-derived closure gate (PROJ-614) found unclosed
    /// evidence at end of run: unreceipted actuations (transitions without a
    /// matching hook receipt in the OCEL evidence graph), unreceipted
    /// dispatches (dispatch_sent without acknowledgement), or
    /// returned-but-unadmitted consequences. The SPARQL evidence graph is
    /// the authority; Rust counters are telemetry only.
    EvidenceGateFailed {
        /// The gate that refused (`unreceipted-actuations` |
        /// `unreceipted-dispatches` | `unadmitted-consequences`).
        gate: String,
        /// The offending graph-derived count (nonzero).
        count: i64,
    },
    /// `CNG_R20` — a v26.7.10 success marker (PROJ-622) evaluated FALSE over
    /// the emitted OCEL/evidence graph. Markers are derived from the on-disk
    /// `queries/markers/*.rq` SELECTs only — never from Rust counters — and
    /// a false marker is a typed refusal with a nonzero exit, never a
    /// warning.
    MarkerFalse {
        /// The marker name (e.g. `AUTONOMIC_LOOP_CLOSED`).
        marker: String,
        /// The marker query's `?value` (0 = proven; anything else refuses).
        value: i64,
    },
    /// `CNG_R21` — a specific decomposition candidate failed a proof
    /// obligation (unsolvable subproblem, interference, unreleased resource,
    /// cyclic composed order) while the caller demanded exactly that
    /// candidate. Ordinary candidate rejection during bounded search is a
    /// typed RESULT (`bench::decomp::DecompositionOutcome` /
    /// `CandidateStatus::Inadmissible` receipts), never this refusal; R21
    /// fires only when an inadmissible candidate would otherwise be selected
    /// or was explicitly forced.
    DecompositionInadmissible {
        /// Canonical candidate id (e.g. `0-single`, or the sorted helper
        /// goal-atom key list).
        candidate: String,
        /// The proof obligation that failed, named.
        reason: String,
    },
    /// `CNG_R22` — the non-interference proof failed on a SELECTED
    /// decomposition: a helper action and a main action with no derived
    /// ordering path between them clobber each other (one's delete effects
    /// intersect the other's protected preconditions). Search must have
    /// filtered such candidates; this is the belt-and-braces gate at
    /// composition/selection time.
    InterferenceDetected {
        /// Ground label of the helper-side action.
        helper_action: String,
        /// Ground label of the main-side action.
        main_action: String,
        /// The clobbered atom (ground label).
        atom: String,
    },
    /// `CNG_R23` — the helper-tape replay found a step whose preconditions
    /// do not hold in the replayed state. The interface state s′ = E(s, π_h)
    /// is a proof obligation, not trust in the planner: a tape that does not
    /// replay lawfully never yields an interface state.
    InterfaceStateMismatch {
        /// 0-based tape step whose precondition failed.
        step: usize,
        /// The missing precondition atom (ground label).
        atom: String,
    },
    /// `CNG_R24` — the resource-release closure gate failed on a selected
    /// decomposition: a resource-classified atom acquired by the helper
    /// remains held in the interface state s′ without any main-side
    /// precondition consuming it. Helpers must release what they acquire.
    ResourceUnreleased {
        /// The unreleased resource atom (ground label).
        resource: String,
        /// The holding side (e.g. `helper`).
        holder: String,
    },
    /// `CNG_R25` — a consequence whose idempotency key was ALREADY admitted
    /// was presented for admission again (replayed/duplicated consequence,
    /// PROJ-721). The durable processed set (`ledger/processed.ttl`) is
    /// checked before every admission; a double admission is a typed
    /// refusal, never a silent re-apply.
    DoubleAdmit {
        /// The dispatch id whose consequence was replayed.
        dispatch: String,
        /// The already-processed idempotency key.
        idempotency_key: String,
    },
}

impl CngRefusal {
    /// Stable machine-readable refusal code.
    ///
    /// # Complexity
    /// O(1).
    pub fn code(&self) -> &'static str {
        match self {
            CngRefusal::MalformedTtl(_) => "CNG_R01",
            CngRefusal::MissingDomain(_) => "CNG_R02",
            CngRefusal::MissingProblem(_) => "CNG_R03",
            CngRefusal::PlanUnsolvable(_) => "CNG_R04",
            CngRefusal::UnsupportedConstruct(_) => "CNG_R05",
            CngRefusal::InvalidPowl(_) => "CNG_R06",
            CngRefusal::RunnerMismatch(_) => "CNG_R07",
            CngRefusal::Nondeterminism(_) => "CNG_R08",
            CngRefusal::HardcodingSuspicion(_) => "CNG_R09",
            CngRefusal::IoRefused(_) => "CNG_R10",
            CngRefusal::AuditMismatch(_) => "CNG_R11",
            CngRefusal::StandingAmbiguous { .. } => "CNG_R12",
            CngRefusal::UnreceiptedActuation { .. } => "CNG_R13",
            CngRefusal::DialectRegistryRefused { .. } => "CNG_R14",
            CngRefusal::DispatchContractIncomplete { .. } => "CNG_R15",
            CngRefusal::DispatchStateUnlawful { .. } => "CNG_R16",
            CngRefusal::ExternalConsequenceRefused { .. } => "CNG_R17",
            CngRefusal::ArazzoProfileRefused { .. } => "CNG_R18",
            CngRefusal::EvidenceGateFailed { .. } => "CNG_R19",
            CngRefusal::MarkerFalse { .. } => "CNG_R20",
            CngRefusal::DecompositionInadmissible { .. } => "CNG_R21",
            CngRefusal::InterferenceDetected { .. } => "CNG_R22",
            CngRefusal::InterfaceStateMismatch { .. } => "CNG_R23",
            CngRefusal::ResourceUnreleased { .. } => "CNG_R24",
            CngRefusal::DoubleAdmit { .. } => "CNG_R25",
        }
    }

    /// The refusal's diagnostic message.
    ///
    /// # Complexity
    /// O(1).
    pub fn message(&self) -> &str {
        match self {
            CngRefusal::MalformedTtl(m)
            | CngRefusal::MissingDomain(m)
            | CngRefusal::MissingProblem(m)
            | CngRefusal::PlanUnsolvable(m)
            | CngRefusal::UnsupportedConstruct(m)
            | CngRefusal::InvalidPowl(m)
            | CngRefusal::RunnerMismatch(m)
            | CngRefusal::Nondeterminism(m)
            | CngRefusal::HardcodingSuspicion(m)
            | CngRefusal::IoRefused(m)
            | CngRefusal::AuditMismatch(m) => m,
            CngRefusal::StandingAmbiguous { .. } => {
                "standing next-action query returned an ambiguous candidate set; \
                 exactly one lawful action is required while work remains"
            }
            CngRefusal::UnreceiptedActuation { .. } => {
                "executed transition produced no matching hook receipt; \
                 zero-unreceipted-actuation law violated"
            }
            CngRefusal::DialectRegistryRefused { .. } => {
                "dialect registry entry violates the closed registry shape; \
                 all eight registry fields are mandatory and no others are lawful"
            }
            CngRefusal::DispatchContractIncomplete { .. } => {
                "dispatch contract is missing required fields; all 21 contract \
                 fields are mandatory before the contract may leave the broker"
            }
            CngRefusal::DispatchStateUnlawful { .. } => {
                "dispatch state transition is outside the lawful 16-state machine"
            }
            CngRefusal::ExternalConsequenceRefused { .. } => {
                "external consequence refused during lawful re-entry; the result \
                 never touches standing before admission"
            }
            CngRefusal::ArazzoProfileRefused { .. } => {
                "arazzo description uses a feature outside the 80/20 profile; \
                 unsupported features are refused by name"
            }
            CngRefusal::EvidenceGateFailed { .. } => {
                "graph-derived closure gate found unclosed evidence; the SPARQL \
                 evidence graph refutes end-of-run closure"
            }
            CngRefusal::MarkerFalse { .. } => {
                "success marker evaluated false over the evidence graph; markers \
                 are SPARQL-derived and a false marker refuses the run"
            }
            CngRefusal::DecompositionInadmissible { .. } => {
                "demanded decomposition candidate failed a proof obligation; an \
                 inadmissible candidate is never selected or forced"
            }
            CngRefusal::InterferenceDetected { .. } => {
                "non-interference proof failed on the selected decomposition; \
                 concurrent segments must not clobber each other's protected \
                 preconditions"
            }
            CngRefusal::InterfaceStateMismatch { .. } => {
                "helper-tape replay found a step whose preconditions do not hold; \
                 the interface state s' is a proof obligation, never trusted"
            }
            CngRefusal::ResourceUnreleased { .. } => {
                "resource-release closure failed: a resource acquired by the \
                 helper remains held in the interface state without a consuming \
                 main precondition"
            }
            CngRefusal::DoubleAdmit { .. } => {
                "consequence idempotency key was already admitted; a replayed \
                 consequence is refused, never silently re-applied"
            }
        }
    }
}

impl std::fmt::Display for CngRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CngRefusal::StandingAmbiguous {
                tick,
                candidate_count,
            } => write!(
                f,
                "{}: {} (tick {tick}, {candidate_count} candidates)",
                self.code(),
                self.message()
            ),
            CngRefusal::UnreceiptedActuation { workflow, category } => write!(
                f,
                "{}: {} (workflow {workflow}, category {category})",
                self.code(),
                self.message()
            ),
            CngRefusal::DialectRegistryRefused { entry, missing } => write!(
                f,
                "{}: {} (entry {entry}, field {missing})",
                self.code(),
                self.message()
            ),
            CngRefusal::DispatchContractIncomplete { dispatch, missing } => write!(
                f,
                "{}: {} (dispatch {dispatch}, missing {missing})",
                self.code(),
                self.message()
            ),
            CngRefusal::DispatchStateUnlawful { dispatch, from, to } => write!(
                f,
                "{}: {} (dispatch {dispatch}, {from} -> {to})",
                self.code(),
                self.message()
            ),
            CngRefusal::ExternalConsequenceRefused { dispatch, stage } => write!(
                f,
                "{}: {} (dispatch {dispatch}, stage {stage})",
                self.code(),
                self.message()
            ),
            CngRefusal::ArazzoProfileRefused { feature } => {
                write!(f, "{}: {} (feature {feature})", self.code(), self.message())
            }
            CngRefusal::EvidenceGateFailed { gate, count } => write!(
                f,
                "{}: {} (gate {gate}, count {count})",
                self.code(),
                self.message()
            ),
            CngRefusal::MarkerFalse { marker, value } => write!(
                f,
                "{}: {} (marker {marker}, value {value})",
                self.code(),
                self.message()
            ),
            CngRefusal::DecompositionInadmissible { candidate, reason } => write!(
                f,
                "{}: {} (candidate {candidate}, reason {reason})",
                self.code(),
                self.message()
            ),
            CngRefusal::InterferenceDetected {
                helper_action,
                main_action,
                atom,
            } => write!(
                f,
                "{}: {} (helper {helper_action}, main {main_action}, atom {atom})",
                self.code(),
                self.message()
            ),
            CngRefusal::InterfaceStateMismatch { step, atom } => write!(
                f,
                "{}: {} (step {step}, atom {atom})",
                self.code(),
                self.message()
            ),
            CngRefusal::ResourceUnreleased { resource, holder } => write!(
                f,
                "{}: {} (resource {resource}, holder {holder})",
                self.code(),
                self.message()
            ),
            CngRefusal::DoubleAdmit {
                dispatch,
                idempotency_key,
            } => write!(
                f,
                "{}: {} (dispatch {dispatch}, key {idempotency_key})",
                self.code(),
                self.message()
            ),
            _ => write!(f, "{}: {}", self.code(), self.message()),
        }
    }
}

impl std::error::Error for CngRefusal {}

/// Projects a PDDL plan tape into a POWL 2.0 `PartialOrder`: one
/// `Leaf(Some(label))` per op in tape order, order relation transitively
/// closed over the total order.
///
/// # Errors
/// `CNG_R04 PlanUnsolvable` for an empty tape.
///
/// # Complexity
/// O(n²) in tape length (the closed order relation has C(n, 2) pairs).
pub fn project_tape_to_powl(tape: &Pddl8Tape) -> Result<Powl, CngRefusal> {
    if tape.ops.is_empty() {
        return Err(CngRefusal::PlanUnsolvable(
            "empty PDDL plan tape: no ops to project into a POWL workflow".to_string(),
        ));
    }
    let children: Vec<Powl> = tape
        .ops
        .iter()
        .map(|op| Powl::Leaf(Some(op.label.clone())))
        .collect();
    let mut order = BTreeSet::new();
    // O(n²): store the transitive closure of the total order.
    for i in 0..children.len() {
        for j in (i + 1)..children.len() {
            order.insert((i, j));
        }
    }
    Ok(Powl::PartialOrder { children, order })
}

/// Projects a PDDL plan tape into a *hierarchical* POWL 2.0 model: ops are
/// grouped into phases by contributing source artifact (`action_sources`,
/// `AdmittedSurface::action_sources`), a phase being a maximal run of
/// tape-adjacent ops sharing the same source. The root is a `PartialOrder`
/// over one child `PartialOrder` per phase (in tape order); each phase is
/// itself a `PartialOrder` over that phase's `Leaf` ops, order transitively
/// closed at both levels. This is the first 8→8² hierarchical instance — no
/// new semantic authority, nesting is derived purely from existing
/// provenance data. Sibling to [`project_tape_to_powl`], which stays flat.
///
/// Returns the model plus `phase_sources`: one artifact source IRI per
/// top-level phase child, in phase order, for later provenance attachment
/// via [`powl_to_turtle_with_phase_provenance`].
///
/// # Errors
/// `CNG_R04 PlanUnsolvable` for an empty tape. `CNG_R09 HardcodingSuspicion`
/// if a tape op's action has no contributing source artifact — the model
/// would be detached from its inputs.
///
/// # Complexity
/// O(n) to group ops into phases, plus O(n² ) total across all levels to
/// store the transitively closed order relations (same asymptotic bound as
/// the flat projection, split across phase and root levels).
pub fn project_tape_to_powl_hierarchical(
    tape: &Pddl8Tape,
    action_sources: &BTreeMap<String, String>,
) -> Result<(Powl, Vec<String>), CngRefusal> {
    if tape.ops.is_empty() {
        return Err(CngRefusal::PlanUnsolvable(
            "empty PDDL plan tape: no ops to project into a hierarchical POWL workflow".to_string(),
        ));
    }

    // Group into maximal tape-adjacent runs sharing the same source artifact.
    let mut phases: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, op) in tape.ops.iter().enumerate() {
        let source = action_sources
            .get(&op.action.schema_name)
            .cloned()
            .ok_or_else(|| {
                CngRefusal::HardcodingSuspicion(format!(
                    "plan op {:?} has no contributing source artifact in the admitted \
                     surface; hierarchical output would be detached from its inputs",
                    op.action.schema_name
                ))
            })?;
        match phases.last_mut() {
            Some((last_source, indices)) if *last_source == source => indices.push(i),
            _ => phases.push((source, vec![i])),
        }
    }

    let phase_sources: Vec<String> = phases.iter().map(|(source, _)| source.clone()).collect();

    let phase_children: Vec<Powl> = phases
        .into_iter()
        .map(|(_, indices)| {
            let leaves: Vec<Powl> = indices
                .iter()
                .map(|&i| Powl::Leaf(Some(tape.ops[i].label.clone())))
                .collect();
            let n = leaves.len();
            let mut order = BTreeSet::new();
            for i in 0..n {
                for j in (i + 1)..n {
                    order.insert((i, j));
                }
            }
            Powl::PartialOrder {
                children: leaves,
                order,
            }
        })
        .collect();

    let n = phase_children.len();
    let mut root_order = BTreeSet::new();
    for i in 0..n {
        for j in (i + 1)..n {
            root_order.insert((i, j));
        }
    }

    Ok((
        Powl::PartialOrder {
            children: phase_children,
            order: root_order,
        },
        phase_sources,
    ))
}

/// Serializes a POWL model as Turtle with deterministic structural IRIs.
/// The root node is `<base>/n0`, typed `powl2:Model`; `derived_from`, when
/// present, attaches exactly one `powl2:derivedFrom` triple to the root.
///
/// # Complexity
/// O(n + |order|) over model nodes plus the pre-closed order relation.
pub fn powl_to_turtle(model: &Powl, base_iri: &str, derived_from: Option<&str>) -> String {
    let base_iri = base_iri.trim_end_matches('/');
    let mut out = String::new();
    out.push_str("@prefix powl2: <");
    out.push_str(POWL2_PREFIX);
    out.push_str("> .\n");
    out.push_str("@prefix base: <");
    out.push_str(base_iri);
    out.push_str("/> .\n\n");

    let root_path = "n0";
    out.push_str(&format!("<{base_iri}/{root_path}> a powl2:Model .\n"));
    if let Some(source_iri) = derived_from {
        out.push_str(&format!(
            "<{base_iri}/{root_path}> powl2:derivedFrom <{source_iri}> .\n"
        ));
    }
    emit_powl_node(model, base_iri, root_path, &mut out);
    out
}

/// Recursively emits Turtle triples for `model` at `<base_iri>/<path>`;
/// children live at `/c<i>` with `ChildBinding`s at `/binding/<i>`.
///
/// # Complexity
/// O(n) in the subtree size plus O(|order|) per `PartialOrder`.
fn emit_powl_node(model: &Powl, base_iri: &str, path: &str, out: &mut String) {
    match model {
        Powl::Leaf(None) => {
            out.push_str(&format!(
                "<{base_iri}/{path}> a powl2:Leaf, powl2:SilentLeaf .\n"
            ));
        }
        Powl::Leaf(Some(label)) => {
            out.push_str(&format!(
                "<{base_iri}/{path}> a powl2:Leaf, powl2:ActivityLeaf ;\n"
            ));
            out.push_str(&format!(
                "  powl2:activityLabel \"{}\" .\n",
                escape_turtle_literal(label)
            ));
        }
        Powl::PartialOrder { children, order } => {
            out.push_str(&format!("<{base_iri}/{path}> a powl2:PartialOrder .\n"));
            for (idx, child) in children.iter().enumerate() {
                let child_path = format!("{path}/c{idx}");
                let binding_path = format!("{path}/binding/{idx}");
                out.push_str(&format!(
                    "<{base_iri}/{path}> powl2:hasChild <{base_iri}/{binding_path}> .\n"
                ));
                out.push_str(&format!(
                    "<{base_iri}/{binding_path}> a powl2:ChildBinding ;\n  powl2:childIndex {idx} ;\n  powl2:childModel <{base_iri}/{child_path}> .\n"
                ));
                emit_powl_node(child, base_iri, &child_path, out);
            }
            for (i, j) in order.iter() {
                out.push_str(&format!(
                    "<{base_iri}/{path}/binding/{i}> powl2:precedes <{base_iri}/{path}/binding/{j}> .\n"
                ));
            }
        }
    }
}

/// PROV-O namespace used for per-element source provenance.
pub const PROV_PREFIX: &str = "http://www.w3.org/ns/prov#";

/// Serializes a POWL model as Turtle with deterministic structural IRIs AND
/// per-leaf source provenance: leaf i (the `powl2:ActivityLeaf` at
/// `<base>/n0/c<i>`) gets one `prov:wasDerivedFrom <leaf_sources[i]>`
/// triple, preserving which imported artifact contributed each workflow
/// element. The root keeps its single `powl2:derivedFrom` provenance triple.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` when `leaf_sources` does not align with
/// the model's top-level children (only the flat linear projection shape is
/// supported).
///
/// # Complexity
/// O(n + |order|) over model nodes plus the pre-closed order relation.
pub fn powl_to_turtle_with_provenance(
    model: &Powl,
    base_iri: &str,
    derived_from: Option<&str>,
    leaf_sources: &[String],
) -> Result<String, CngRefusal> {
    let child_count = match model {
        Powl::PartialOrder { children, .. } => children.len(),
        Powl::Leaf(_) => 1,
    };
    if leaf_sources.len() != child_count {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "leaf provenance list has {} entries but the model has {child_count} \
             top-level elements; per-element provenance requires the flat linear shape",
            leaf_sources.len()
        )));
    }
    let base = base_iri.trim_end_matches('/');
    let mut out = String::new();
    out.push_str("@prefix prov: <");
    out.push_str(PROV_PREFIX);
    out.push_str("> .\n");
    out.push_str(&powl_to_turtle(model, base_iri, derived_from));
    for (idx, source_iri) in leaf_sources.iter().enumerate() {
        let subject = match model {
            Powl::PartialOrder { .. } => format!("{base}/n0/c{idx}"),
            Powl::Leaf(_) => format!("{base}/n0"),
        };
        out.push_str(&format!(
            "<{subject}> prov:wasDerivedFrom <{source_iri}> .\n"
        ));
    }
    Ok(out)
}

/// Serializes a *hierarchical* POWL model (as produced by
/// [`project_tape_to_powl_hierarchical`]) as Turtle with deterministic
/// structural IRIs AND per-phase source provenance: phase i (the
/// `powl2:PartialOrder` at `<base>/n0/c<i>`) gets one
/// `prov:wasDerivedFrom <phase_sources[i]>` triple, preserving which
/// imported artifact contributed that phase's ops. Leaf-level provenance
/// stays implied transitively via phase membership — attaching it directly
/// is out of scope for the hierarchical increment. The root keeps its
/// single `powl2:derivedFrom` provenance triple. Does not alter or replace
/// [`powl_to_turtle_with_provenance`]; existing flat callers are unaffected.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` when `model`'s top level is not a
/// `PartialOrder` whose every child is itself a `PartialOrder` (i.e. the
/// model is flat, not hierarchical), or when `phase_sources.len()` does not
/// match the top-level phase count.
///
/// # Complexity
/// O(n + |order|) over model nodes plus the pre-closed order relations.
pub fn powl_to_turtle_with_phase_provenance(
    model: &Powl,
    base_iri: &str,
    derived_from: Option<&str>,
    phase_sources: &[String],
) -> Result<String, CngRefusal> {
    let Powl::PartialOrder { children, .. } = model else {
        return Err(CngRefusal::UnsupportedConstruct(
            "hierarchical provenance requires a root PartialOrder of phase PartialOrders; \
             found a bare Leaf model"
                .to_string(),
        ));
    };
    if !children
        .iter()
        .all(|child| matches!(child, Powl::PartialOrder { .. }))
    {
        return Err(CngRefusal::UnsupportedConstruct(
            "hierarchical provenance requires every top-level child to be a phase \
             PartialOrder; found a flat (leaf-only) model — use \
             powl_to_turtle_with_provenance instead"
                .to_string(),
        ));
    }
    if phase_sources.len() != children.len() {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "phase provenance list has {} entries but the model has {} top-level phases",
            phase_sources.len(),
            children.len()
        )));
    }

    let base = base_iri.trim_end_matches('/');
    let mut out = String::new();
    out.push_str("@prefix prov: <");
    out.push_str(PROV_PREFIX);
    out.push_str("> .\n");
    out.push_str(&powl_to_turtle(model, base_iri, derived_from));
    for (idx, source_iri) in phase_sources.iter().enumerate() {
        out.push_str(&format!(
            "<{base}/n0/c{idx}> prov:wasDerivedFrom <{source_iri}> .\n"
        ));
    }
    Ok(out)
}

/// Escapes a string for use inside a double-quoted Turtle literal.
///
/// # Complexity
/// O(len).
fn escape_turtle_literal(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
#[path = "powl_test.rs"]
mod powl_test;
