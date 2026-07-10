//! Minimal POWL 2.0 model, projection, and Turtle serialization for the cng
//! CLI. Clean-room implementation of the invariants proven in the praxis
//! test surface (chatman_pddl_to_powl_*): one `ActivityLeaf` per plan op in
//! tape order; the order relation stored pre-closed (transitive closure,
//! `(i, j)` for all `i < j`); deterministic structural IRIs minted from a
//! base IRI (`<base>/n0`, `<base>/n0/c<i>`, `<base>/n0/binding/<i>`);
//! `powl2:derivedFrom` attached to the root model node only. Same inputs
//! produce byte-identical Turtle.

use std::collections::BTreeSet;

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
            | CngRefusal::IoRefused(m) => m,
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

    #[test]
    fn empty_tape_refuses_plan_unsolvable() {
        let empty = Pddl8Tape { ops: vec![] };
        match project_tape_to_powl(&empty) {
            Err(refusal @ CngRefusal::PlanUnsolvable(_)) => {
                assert_eq!(refusal.code(), "CNG_R04");
                assert!(!refusal.message().is_empty());
            }
            other => panic!("expected PlanUnsolvable, got {other:?}"),
        }
    }

    #[test]
    fn provenance_serializer_emits_one_source_per_leaf() {
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
        assert!(turtle.contains("<urn:t/n0/c0> prov:wasDerivedFrom <urn:blake3:aa> ."));
        assert!(turtle.contains("<urn:t/n0/c1> prov:wasDerivedFrom <urn:blake3:bb> ."));
        // Misaligned provenance refuses (UnsupportedConstruct, CNG_R05).
        match powl_to_turtle_with_provenance(&model, "urn:t", None, &sources[..1]) {
            Err(r @ CngRefusal::UnsupportedConstruct(_)) => assert_eq!(r.code(), "CNG_R05"),
            other => panic!("expected UnsupportedConstruct, got {other:?}"),
        }
    }

    #[test]
    fn turtle_is_deterministic_and_derived_from_is_root_only() {
        let model = Powl::PartialOrder {
            children: vec![
                Powl::Leaf(Some("a(x)".to_string())),
                Powl::Leaf(Some("b(x)".to_string())),
            ],
            order: [(0usize, 1usize)].into_iter().collect(),
        };
        let a = powl_to_turtle(&model, "urn:t", Some("urn:src"));
        let b = powl_to_turtle(&model, "urn:t", Some("urn:src"));
        assert_eq!(a, b);
        assert_eq!(a.matches("powl2:derivedFrom").count(), 1);
        assert!(a.contains("<urn:t/n0> powl2:derivedFrom <urn:src> ."));
    }
}
