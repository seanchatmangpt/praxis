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
        }
    }
}

impl std::fmt::Display for CngRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
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
mod tests {
    use super::*;
    use bcinr_pddl::Pddl8Tape;
    use chicago_tdd_tools::prelude::*;
    use oxigraph::model::NamedNode;
    use oxigraph::store::Store;

    /// Parses serializer output into an in-memory store so assertions run
    /// over the parsed graph via `crate::shape::validate_powl_store` and the
    /// typed `quads_for_pattern` API — never substring matching on Turtle
    /// and never inline SPARQL strings.
    fn store_from_turtle(turtle: &str) -> Store {
        let store = Store::new().expect("in-memory store must construct");
        store
            .load_from_slice(
                oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle),
                turtle.as_bytes(),
            )
            .expect("serializer output must be valid Turtle");
        store
    }

    /// Objects of `<subject_iri> <predicate_iri> ?o` in the default graph,
    /// via the typed pattern API. O(matches).
    fn objects_of(store: &Store, subject_iri: &str, predicate_iri: &str) -> Vec<String> {
        let subject = NamedNode::new(subject_iri).expect("test subject IRI must parse");
        let predicate = NamedNode::new(predicate_iri).expect("test predicate IRI must parse");
        store
            .quads_for_pattern(
                Some(subject.as_ref().into()),
                Some(predicate.as_ref()),
                None,
                None,
            )
            .map(|quad| quad.expect("quad must decode").object.to_string())
            .collect()
    }

    /// Count of quads carrying `<predicate_iri>` anywhere in the store, via
    /// the typed pattern API. O(matches).
    fn predicate_count(store: &Store, predicate_iri: &str) -> usize {
        let predicate = NamedNode::new(predicate_iri).expect("test predicate IRI must parse");
        store
            .quads_for_pattern(None, Some(predicate.as_ref()), None, None)
            .count()
    }

    test!(empty_tape_refuses_plan_unsolvable, {
        let empty = Pddl8Tape { ops: vec![] };
        match project_tape_to_powl(&empty) {
            Err(refusal @ CngRefusal::PlanUnsolvable(_)) => {
                assert_eq!(refusal.code(), "CNG_R04");
                assert!(!refusal.message().is_empty());
            }
            other => panic!("expected PlanUnsolvable, got {other:?}"),
        }
    });

    test!(audit_mismatch_refusal_has_stable_code, {
        let refusal = CngRefusal::AuditMismatch("digest drift".to_string());
        assert_eq!(refusal.code(), "CNG_R11");
        assert_eq!(refusal.message(), "digest drift");
        assert_eq!(format!("{refusal}"), "CNG_R11: digest drift");
    });

    test!(provenance_serializer_emits_one_source_per_leaf, {
        let model = Powl::PartialOrder {
            children: vec![
                Powl::Leaf(Some("a(x)".to_string())),
                Powl::Leaf(Some("b(x)".to_string())),
            ],
            order: [(0usize, 1usize)].into_iter().collect(),
        };
        let sources = vec!["urn:blake3:aa".to_string(), "urn:blake3:bb".to_string()];
        let turtle = powl_to_turtle_with_provenance(&model, "urn:t", Some("urn:src"), &sources)
            .expect("aligned provenance must serialize");
        let store = store_from_turtle(&turtle);
        let prov_iri = format!("{PROV_PREFIX}wasDerivedFrom");
        for (idx, expected_source) in sources.iter().enumerate() {
            assert_eq!(
                objects_of(&store, &format!("urn:t/n0/c{idx}"), &prov_iri),
                vec![format!("<{expected_source}>")],
                "leaf {idx} must carry exactly its own source's provenance"
            );
        }
        assert_eq!(predicate_count(&store, &prov_iri), sources.len());
        // Misaligned provenance refuses (UnsupportedConstruct, CNG_R05).
        match powl_to_turtle_with_provenance(&model, "urn:t", None, &sources[..1]) {
            Err(r @ CngRefusal::UnsupportedConstruct(_)) => assert_eq!(r.code(), "CNG_R05"),
            other => panic!("expected UnsupportedConstruct, got {other:?}"),
        }
    });

    test!(turtle_is_deterministic_and_derived_from_is_root_only, {
        let model = Powl::PartialOrder {
            children: vec![
                Powl::Leaf(Some("a(x)".to_string())),
                Powl::Leaf(Some("b(x)".to_string())),
            ],
            order: [(0usize, 1usize)].into_iter().collect(),
        };
        // Determinism: whole-output byte equality (String equality, not
        // substring matching).
        let a = powl_to_turtle(&model, "urn:t", Some("urn:src"));
        let b = powl_to_turtle(&model, "urn:t", Some("urn:src"));
        assert_eq!(a, b, "same inputs must serialize byte-identically");
        // Root-only provenance, asserted over the parsed graph.
        let store = store_from_turtle(&a);
        let derived_iri = format!("{POWL2_PREFIX}derivedFrom");
        assert_eq!(predicate_count(&store, &derived_iri), 1);
        assert_eq!(
            objects_of(&store, "urn:t/n0", &derived_iri),
            vec!["<urn:src>".to_string()],
            "the single powl2:derivedFrom triple must sit on the root"
        );
    });

    /// Builds a synthetic tape op with a given `(schema_name, label)`; the
    /// action's preconditions/effects are irrelevant to projection.
    fn tape_op(index: u8, pred_mask: u64, schema_name: &str) -> bcinr_pddl::Pddl8TapeOp {
        bcinr_pddl::Pddl8TapeOp {
            index,
            label: format!("{schema_name}()"),
            pred_mask,
            action: bcinr_pddl::Pddl8GroundAction {
                schema_name: schema_name.to_string(),
                label: format!("{schema_name}()"),
                preconditions: vec![],
                add_effects: vec![],
                del_effects: vec![],
            },
        }
    }

    /// Three artifacts, tape order A,A,B,C — a run of consecutive same-source
    /// ops followed by two single-op phases from distinct artifacts.
    fn three_phase_tape_and_sources() -> (Pddl8Tape, BTreeMap<String, String>) {
        let tape = Pddl8Tape {
            ops: vec![
                tape_op(0, 0, "act_a1"),
                tape_op(1, 1, "act_a2"),
                tape_op(2, 2, "act_b1"),
                tape_op(3, 4, "act_c1"),
            ],
        };
        let mut sources = BTreeMap::new();
        sources.insert("act_a1".to_string(), "urn:blake3:aa".to_string());
        sources.insert("act_a2".to_string(), "urn:blake3:aa".to_string());
        sources.insert("act_b1".to_string(), "urn:blake3:bb".to_string());
        sources.insert("act_c1".to_string(), "urn:blake3:cc".to_string());
        (tape, sources)
    }

    test!(
        hierarchical_projection_groups_consecutive_same_source_runs,
        {
            let (tape, sources) = three_phase_tape_and_sources();
            let (model, phase_sources) =
                project_tape_to_powl_hierarchical(&tape, &sources).expect("must project");

            assert_eq!(
                phase_sources,
                vec![
                    "urn:blake3:aa".to_string(),
                    "urn:blake3:bb".to_string(),
                    "urn:blake3:cc".to_string(),
                ]
            );
            let Powl::PartialOrder { children, order } = &model else {
                panic!("expected root PartialOrder");
            };
            assert_eq!(children.len(), 3, "3 phases: [a1,a2], [b1], [c1]");
            assert_eq!(order.len(), 3, "C(3,2) root-level precedence pairs");
            let Powl::PartialOrder {
                children: phase0_leaves,
                order: phase0_order,
            } = &children[0]
            else {
                panic!("expected phase 0 to be a PartialOrder");
            };
            assert_eq!(phase0_leaves.len(), 2, "phase 0 groups act_a1 and act_a2");
            assert_eq!(phase0_order.len(), 1, "C(2,2) intra-phase precedence pair");
            let Powl::PartialOrder {
                children: phase1_leaves,
                ..
            } = &children[1]
            else {
                panic!("expected phase 1 to be a PartialOrder");
            };
            assert_eq!(phase1_leaves.len(), 1, "phase 1 is the lone act_b1 op");
        }
    );

    test!(hierarchical_projection_refuses_empty_tape, {
        let empty = Pddl8Tape { ops: vec![] };
        match project_tape_to_powl_hierarchical(&empty, &BTreeMap::new()) {
            Err(refusal @ CngRefusal::PlanUnsolvable(_)) => {
                assert_eq!(refusal.code(), "CNG_R04");
            }
            other => panic!("expected PlanUnsolvable, got {other:?}"),
        }
    });

    test!(hierarchical_projection_refuses_untracked_action, {
        let tape = Pddl8Tape {
            ops: vec![tape_op(0, 0, "act_unknown")],
        };
        match project_tape_to_powl_hierarchical(&tape, &BTreeMap::new()) {
            Err(refusal @ CngRefusal::HardcodingSuspicion(_)) => {
                assert_eq!(refusal.code(), "CNG_R09");
            }
            other => panic!("expected HardcodingSuspicion, got {other:?}"),
        }
    });

    test!(phase_provenance_serializer_emits_one_source_per_phase, {
        let (tape, sources) = three_phase_tape_and_sources();
        let (model, phase_sources) =
            project_tape_to_powl_hierarchical(&tape, &sources).expect("must project");

        let turtle =
            powl_to_turtle_with_phase_provenance(&model, "urn:t", Some("urn:src"), &phase_sources)
                .expect("aligned phase provenance must serialize");
        let store = store_from_turtle(&turtle);

        // The nested model passes the crate's own structural validator —
        // this doubles as the shape.rs regression test for hierarchical
        // output (root Model + 4 PartialOrders + 4 labelled leaves + 7
        // bindings: 3 root-level, 4 leaf-level).
        let report =
            crate::shape::validate_powl_store(&store, true).expect("nested model must validate");
        assert_eq!(report.models, 1);
        assert_eq!(report.partial_orders, 4, "root + 3 phase PartialOrders");
        assert_eq!(report.activity_leaves, 4, "all 4 tape ops are leaves");
        assert_eq!(
            report.child_bindings, 7,
            "3 phase bindings + 4 leaf bindings"
        );
        assert_eq!(report.derived_from, 1);

        // One prov:wasDerivedFrom per phase node (n0/c0, n0/c1, n0/c2), each
        // pointing at that phase's contributing source IRI — asserted with
        // the typed pattern API over the parsed graph.
        let prov_iri = format!("{PROV_PREFIX}wasDerivedFrom");
        for (phase_idx, expected_source) in phase_sources.iter().enumerate() {
            assert_eq!(
                objects_of(&store, &format!("urn:t/n0/c{phase_idx}"), &prov_iri),
                vec![format!("<{expected_source}>")],
                "phase {phase_idx} must carry exactly its own source's provenance"
            );
        }
        assert_eq!(
            predicate_count(&store, &prov_iri),
            3,
            "exactly one prov:wasDerivedFrom triple per phase"
        );
    });

    test!(phase_provenance_serializer_refuses_flat_model, {
        // A flat model (top-level children are Leaf, not PartialOrder) is not
        // a hierarchical shape — refuses CNG_R05, points callers at the flat
        // provenance function instead.
        let flat_model = Powl::PartialOrder {
            children: vec![
                Powl::Leaf(Some("a(x)".to_string())),
                Powl::Leaf(Some("b(x)".to_string())),
            ],
            order: [(0usize, 1usize)].into_iter().collect(),
        };
        let sources = vec!["urn:blake3:aa".to_string(), "urn:blake3:bb".to_string()];
        match powl_to_turtle_with_phase_provenance(&flat_model, "urn:t", None, &sources) {
            Err(r @ CngRefusal::UnsupportedConstruct(_)) => assert_eq!(r.code(), "CNG_R05"),
            other => panic!("expected UnsupportedConstruct, got {other:?}"),
        }
    });

    test!(
        phase_provenance_serializer_refuses_misaligned_source_count,
        {
            let (tape, sources) = three_phase_tape_and_sources();
            let (model, phase_sources) =
                project_tape_to_powl_hierarchical(&tape, &sources).expect("must project");
            match powl_to_turtle_with_phase_provenance(
                &model,
                "urn:t",
                None,
                &phase_sources[..phase_sources.len() - 1],
            ) {
                Err(r @ CngRefusal::UnsupportedConstruct(_)) => assert_eq!(r.code(), "CNG_R05"),
                other => panic!("expected UnsupportedConstruct, got {other:?}"),
            }
        }
    );

    test!(
        existing_flat_functions_are_unaffected_by_hierarchical_additions,
        {
            // Regression guard: the pre-existing flat projection/serialization
            // shape is unchanged after adding the hierarchical siblings, verified
            // by the crate's own structural validator over the parsed output —
            // no substring matching, no inline query strings.
            let tape = Pddl8Tape {
                ops: vec![tape_op(0, 0, "a"), tape_op(1, 1, "b")],
            };
            let model = project_tape_to_powl(&tape).expect("flat projection");
            let turtle = powl_to_turtle(&model, "urn:t", Some("urn:src"));
            let store = store_from_turtle(&turtle);

            let report =
                crate::shape::validate_powl_store(&store, true).expect("flat model must validate");
            assert_eq!(report.models, 1);
            assert_eq!(report.partial_orders, 1, "flat model has one PartialOrder");
            assert_eq!(
                report.activity_leaves, 2,
                "both flat tape ops must serialize as labelled ActivityLeafs"
            );
            assert_eq!(report.child_bindings, 2);
            assert_eq!(report.precedes, 1, "C(2,2) = 1 closed order pair");
            assert_eq!(report.derived_from, 1);
        }
    );
}
