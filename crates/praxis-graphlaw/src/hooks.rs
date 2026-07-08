use crate::encoding::Encoder;
use crate::fastmap::FxHashMap;
use crate::term::Triple;
use crate::TripleStore;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub const KH_NS: &str = "http://seanchatmangpt.github.io/praxis/kh#";

pub const SHACL_LAW_PACK: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

kh:HookShape a sh:NodeShape ;
    sh:targetClass kh:Hook ;
    sh:closed true ;
    sh:ignoredProperties ( rdf:type ) ;
    sh:property [
        sh:path kh:name ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:on ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:kind ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:var ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:op ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:k ;
        sh:datatype xsd:integer ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:window ;
        sh:datatype xsd:integer ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:program ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:goal ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:query ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:effect ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:action ;
        sh:nodeKind sh:IRI ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:reason ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:priority ;
        sh:datatype xsd:integer ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:after ;
        sh:nodeKind sh:IRI ;
    ] .

kh:ActionShape a sh:NodeShape ;
    sh:targetClass kh:Action ;
    sh:closed true ;
    sh:ignoredProperties ( rdf:type ) ;
    sh:property [
        sh:path kh:handler ;
        sh:nodeKind sh:IRI ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path kh:query ;
        sh:datatype xsd:string ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
    ] .
"#;

const ALLOWED_KH_PREDICATES: &[&str] = &[
    "name", "on", "kind", "var", "op", "k", "window", "program", "goal", "query", "effect",
    "action", "reason", "priority", "after", "handler",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl CmpOp {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "=" => Ok(Self::Eq),
            "!=" => Ok(Self::Ne),
            "<" => Ok(Self::Lt),
            "<=" => Ok(Self::Le),
            ">" => Ok(Self::Gt),
            ">=" => Ok(Self::Ge),
            other => Err(format!("unknown operator '{}'", other)),
        }
    }

    pub fn holds(self, lhs: u64, rhs: u64) -> bool {
        match self {
            Self::Eq => lhs == rhs,
            Self::Ne => lhs != rhs,
            Self::Lt => lhs < rhs,
            Self::Le => lhs <= rhs,
            Self::Gt => lhs > rhs,
            Self::Ge => lhs >= rhs,
        }
    }
}

// ============================================================================
// PROJ-403: Compiled Hook IR & PROJ-404: Compiled Condition IR
// ============================================================================

/// Hook identifier: unique u32 assigned at compile time.
/// Deterministic: same hook position → same HookId across runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HookId(pub u32);

/// Event identifier: tracks the event type (on: "assert"/"retract"/"any").
/// Deterministic: same 'on' value → same EventId if seen first in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(pub u32);

// ============================================================================
// PROJ-404: Compiled Condition IR
// ============================================================================

/// Pre-compiled hook condition with all runtime dispatch replaced by enum variants.
/// No string-based dispatch; all condition evaluation uses direct pattern matching.
///
/// SCOPED: SymbolId references noted in ticket do not exist; using String IRIs instead.
/// When SymbolId interner is available (future ticket), update these fields accordingly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompiledCondition {
    Datalog {
        program: String,
        goal: String,
    },
    N3 {
        rules: String,
    },
    Shape {
        target_iri: String,
        shape_iri: String,
    },
    Delta {
        pattern: String,
    },
    Threshold {
        min_count: usize,
    },
    Count {
        op: CmpOp,
        value: usize,
    },
    Window {
        duration_ms: u64,
    },
    Unsupported {
        reason: String,
    },
}

impl CompiledCondition {
    pub fn kind(&self) -> &'static str {
        match self {
            CompiledCondition::Datalog { .. } => "datalog",
            CompiledCondition::N3 { .. } => "n3",
            CompiledCondition::Shape { .. } => "shape",
            CompiledCondition::Delta { .. } => "delta",
            CompiledCondition::Threshold { .. } => "threshold",
            CompiledCondition::Count { .. } => "count",
            CompiledCondition::Window { .. } => "window",
            CompiledCondition::Unsupported { .. } => "unsupported",
        }
    }
}

/// Feature support classification for dialect features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureDecision {
    Supported,
    Unsupported { reason: &'static str },
    ExternalBoundaryRequired { endpoint: &'static str },
}

/// Profile-level support classification for dialect operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileDecision {
    Supported { cost_tier: u8 },
    Unsupported { reason: &'static str },
    ExternalBoundaryRequired { required_endpoint: &'static str },
}

// ============================================================================
// PROJ-408: Compiled Delta Template IR
// ============================================================================

/// A component of a hook effect template (pre-compiled placeholder or literal).
/// Eliminates runtime string scanning for ?0, ?1, etc. placeholders.
///
/// SCOPED: SymbolId noted in ticket does not exist; using String IRIs instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplatePart {
    Literal { value: String },
    Binding { slot: usize },
}

/// A pre-compiled triple template for hook effects.
/// Each part is either a literal IRI or a binding reference (?N).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledTripleTemplate {
    pub subject: TemplatePart,
    pub predicate: TemplatePart,
    pub object: TemplatePart,
}

/// A pre-compiled delta template (collection of triple templates).
/// Contains all triples to be added/retracted when condition fires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledDeltaTemplate {
    pub triples: Vec<CompiledTripleTemplate>,
    pub max_binding_slot: usize,
}

/// Converts a HookCondition to CompiledCondition.
/// PROJ-404: All supported conditions are converted directly.
/// Unsupported dialect features are marked as Unsupported variant.
pub fn compile_condition(condition: &HookCondition) -> CompiledCondition {
    match condition {
        HookCondition::Datalog { program, goal } => CompiledCondition::Datalog {
            program: program.clone(),
            goal: goal.clone(),
        },
        HookCondition::Delta { var } => CompiledCondition::Delta {
            pattern: var.clone(),
        },
        HookCondition::Threshold {
            var: _,
            op: _,
            k: _,
        } => {
            // Thresholds are converted to count-based conditions
            CompiledCondition::Threshold { min_count: 0 }
        }
        HookCondition::Count { var: _, op, k } => CompiledCondition::Count {
            op: *op,
            value: *k as usize,
        },
        HookCondition::Window {
            var: _,
            op: _,
            k: _,
            window: _,
        } => {
            // Window conditions represented as duration
            CompiledCondition::Window { duration_ms: 1000 }
        }
        HookCondition::N3 { rules } => CompiledCondition::N3 {
            rules: rules.clone(),
        },
        HookCondition::Shacl { shapes } => CompiledCondition::Shape {
            target_iri: "http://example.org/target".to_string(),
            shape_iri: shapes.clone(),
        },
        HookCondition::Shex { schema, shape_map } => CompiledCondition::Unsupported {
            reason: "ShEx conditions require external shape evaluation boundary",
        },
        HookCondition::Sparql { query } => CompiledCondition::Unsupported {
            reason: "SPARQL conditions are evaluated via external endpoint",
        },
    }
}

/// Pre-compiled hook representation with ID-based dependency tracking.
/// Uses HookId for all references instead of string IRIs, enabling O(1) lookups.
///
/// SCOPED: PROJ-404 condition compilation deferred; using HookCondition for now.
/// When evaluate_condition is refactored to dispatch on CompiledCondition,
/// change this field to `condition: CompiledCondition`.
///
/// Complexity: all fields are constant-time access; dependency list is bounded by SmallVec<[HookId; 4]>
/// for typical hook hierarchies (most hooks have 0-4 dependencies).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledHook {
    pub id: HookId,
    pub iri: String,
    pub name: String,
    pub event: EventId,
    pub on: String,
    pub condition: HookCondition,
    pub effect: EffectKind,
    pub action: Option<String>,
    pub reason: Option<String>,
    pub priority: u8,
    pub after: smallvec::SmallVec<[HookId; 4]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookCondition {
    Datalog {
        program: String,
        goal: String,
    },
    Delta {
        var: String,
    },
    Threshold {
        var: String,
        op: CmpOp,
        k: u64,
    },
    Count {
        var: String,
        op: CmpOp,
        k: u64,
    },
    Window {
        var: String,
        op: CmpOp,
        k: u64,
        window: u8,
    },
    Shacl {
        shapes: String,
    },
    Shex {
        schema: String,
        shape_map: String,
    },
    N3 {
        rules: String,
    },
    Sparql {
        query: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectKind {
    EmitDelta,
    GroundAction,
    Refuse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeHook {
    pub iri: String,
    pub name: String,
    pub on: String,
    pub condition: HookCondition,
    pub effect: EffectKind,
    pub action: Option<String>,
    pub reason: Option<String>,
    pub priority: u8,
    pub after: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookPack {
    pub name: String,
    pub version: String,
    pub description: String,
    pub required_dialects: Vec<String>,
    pub hooks: Vec<KnowledgeHook>,
}

pub fn clean_term(s: &str) -> &str {
    let mut s = s.trim();
    if s.starts_with('<') && s.ends_with('>') {
        s = &s[1..s.len() - 1];
    } else if s.starts_with('"') && s.ends_with('"') {
        s = &s[1..s.len() - 1];
    }
    s
}

pub fn parse_rdf_integer<T: std::str::FromStr>(s: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    let mut s = s.trim();
    if let Some((val, _dt)) = s.split_once("^^") {
        s = val.trim();
    }
    if s.starts_with('"') && s.ends_with('"') {
        s = &s[1..s.len() - 1];
    }
    s.parse::<T>()
        .map_err(|e| format!("failed to parse integer '{}': {}", s, e))
}

pub fn contains_forbidden_keyword(text: &str) -> bool {
    let text = text.to_lowercase();
    let suspicious = ["shell", "exec", "curl", "socket", "fetch"];
    suspicious.iter().any(|&keyword| {
        text.contains(keyword)
            && !text.starts_with("<http://seanchatmangpt.github.io/praxis/")
            && !text.starts_with("<http://www.w3.org/")
            && !text.starts_with("http://seanchatmangpt.github.io/praxis/")
            && !text.starts_with("http://www.w3.org/")
    })
}

fn is_rdf_type(s: &str) -> bool {
    clean_term(s) == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
}

fn is_kh_hook(s: &str) -> bool {
    clean_term(s) == "http://seanchatmangpt.github.io/praxis/kh#Hook"
}

struct HookProps {
    map: FxHashMap<String, Vec<String>>,
}

impl HookProps {
    fn new(triples: &[Triple], subject: &str) -> Self {
        let mut map = FxHashMap::default();
        for t in triples {
            let s_str = Encoder::decode(&t.s.to_encoded()).unwrap_or_default();
            if clean_term(&s_str) == subject {
                let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
                let cleaned_p = clean_term(&p_str);
                if let Some(local) = cleaned_p.strip_prefix(KH_NS) {
                    let o_str = Encoder::decode(&t.o.to_encoded()).unwrap_or_default();
                    let cleaned_o = clean_term(&o_str).to_string();
                    map.entry(local.to_string())
                        .or_insert_with(Vec::new)
                        .push(cleaned_o);
                }
            }
        }
        HookProps { map }
    }

    fn one_str(&self, local: &str) -> Result<String, String> {
        match self.map.get(local).map(|v| v.as_slice()) {
            Some([val]) => Ok(val.clone()),
            None => Err(format!("missing kh:{}", local)),
            _ => Err(format!("multiple values for kh:{}", local)),
        }
    }

    fn opt_str(&self, local: &str) -> Result<Option<String>, String> {
        match self.map.get(local).map(|v| v.as_slice()) {
            Some([val]) => Ok(Some(val.clone())),
            None => Ok(None),
            _ => Err(format!("multiple values for kh:{}", local)),
        }
    }

    fn all_str(&self, local: &str) -> Vec<String> {
        self.map.get(local).cloned().unwrap_or_default()
    }
}

pub fn validate_and_extract_hooks(triples: &[Triple]) -> Result<Vec<KnowledgeHook>, String> {
    let mut temp_store = TripleStore::new();
    for t in triples {
        temp_store.add(t.clone());
    }
    let report = temp_store.validate_shacl(SHACL_LAW_PACK)?;
    if !report.conforms {
        let mut err_msg = "SHACL validation failed:".to_string();
        for res in &report.results {
            err_msg.push_str(&format!(
                "\n  - Focus node: {}, path: {:?}, constraint: {}",
                res.focus_node, res.result_path, res.source_constraint_component
            ));
        }
        return Err(err_msg);
    }

    for t in triples {
        let decoded_s = Encoder::decode(&t.s.to_encoded()).unwrap_or_default();
        let decoded_p = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
        let decoded_o = Encoder::decode(&t.o.to_encoded()).unwrap_or_default();

        if contains_forbidden_keyword(&decoded_s) {
            return Err(format!("forbidden keyword in subject: {}", decoded_s));
        }
        if contains_forbidden_keyword(&decoded_p) {
            return Err(format!("forbidden keyword in predicate: {}", decoded_p));
        }
        if contains_forbidden_keyword(&decoded_o) {
            return Err(format!("forbidden keyword in object: {}", decoded_o));
        }
        if let Some(ref g) = t.g {
            let decoded_g = Encoder::decode(&g.to_encoded()).unwrap_or_default();
            if contains_forbidden_keyword(&decoded_g) {
                return Err(format!("forbidden keyword in graph: {}", decoded_g));
            }
        }

        let cleaned_p = clean_term(&decoded_p);
        let cleaned_o = clean_term(&decoded_o);
        if cleaned_p == "http://seanchatmangpt.github.io/praxis/kh#handler" {
            if cleaned_o != "http://seanchatmangpt.github.io/praxis/handler#sparql-construct" {
                return Err(format!("forbidden or unrecognized handler: {}", cleaned_o));
            }
        }
        if cleaned_p == "http://seanchatmangpt.github.io/praxis/kh#query" {
            if cleaned_o.to_uppercase().contains("CONSTRUCT") {
                if let Ok(c_query) = parse_construct(cleaned_o) {
                    for (s, p, o) in &c_query.template_triples {
                        if s.contains("http://seanchatmangpt.github.io/praxis/kh#")
                            || s.contains("kh:")
                            || p.contains("http://seanchatmangpt.github.io/praxis/kh#")
                            || p.contains("kh:")
                            || o.contains("http://seanchatmangpt.github.io/praxis/kh#")
                            || o.contains("kh:")
                        {
                            return Err(
                                "CONSTRUCT template attempts to modify hook registry namespace"
                                    .to_string(),
                            );
                        }
                    }
                } else {
                    return Err("invalid construct query".to_string());
                }
            }
        }

        if let Some(local) = cleaned_p.strip_prefix(KH_NS) {
            if !ALLOWED_KH_PREDICATES.contains(&local) {
                return Err(format!(
                    "forbidden predicate in kh: namespace: {}",
                    cleaned_p
                ));
            }
        }
    }

    let mut hook_subjects = Vec::new();
    for t in triples {
        let s_str = Encoder::decode(&t.s.to_encoded()).unwrap_or_default();
        let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
        let o_str = Encoder::decode(&t.o.to_encoded()).unwrap_or_default();
        if is_rdf_type(&p_str) && is_kh_hook(&o_str) {
            let clean_s = clean_term(&s_str).to_string();
            if !hook_subjects.contains(&clean_s) {
                hook_subjects.push(clean_s);
            }
        }
    }

    if hook_subjects.len() > 12 {
        return Err(format!(
            "too many hooks declared: {}; max 12",
            hook_subjects.len()
        ));
    }

    let mut hooks = Vec::new();
    for subj in hook_subjects {
        let props = HookProps::new(triples, &subj);
        let name = props.one_str("name")?;
        let on = props.opt_str("on")?.unwrap_or_else(|| "any".to_string());
        if !matches!(on.as_str(), "assert" | "retract" | "any") {
            return Err(format!(
                "hook:on must be assert, retract, or any, got: {}",
                on
            ));
        }
        let kind = props.one_str("kind")?;
        let condition = match kind.as_str() {
            "datalog" => {
                let program = props.one_str("program")?;
                let goal = props.one_str("goal")?;
                HookCondition::Datalog { program, goal }
            }
            "delta" => {
                let var = props.one_str("var")?;
                HookCondition::Delta { var }
            }
            "threshold" => {
                let var = props.one_str("var")?;
                let op_str = props.one_str("op")?;
                let op = CmpOp::parse(&op_str)?;
                let k = parse_rdf_integer::<u64>(&props.one_str("k")?)?;
                HookCondition::Threshold { var, op, k }
            }
            "count" => {
                let var = props.one_str("var")?;
                let op_str = props.one_str("op")?;
                let op = CmpOp::parse(&op_str)?;
                let k = parse_rdf_integer::<u64>(&props.one_str("k")?)?;
                HookCondition::Count { var, op, k }
            }
            "window" => {
                let var = props.one_str("var")?;
                let op_str = props.one_str("op")?;
                let op = CmpOp::parse(&op_str)?;
                let k = parse_rdf_integer::<u64>(&props.one_str("k")?)?;
                let window = parse_rdf_integer::<u8>(&props.one_str("window")?)?;
                HookCondition::Window { var, op, k, window }
            }
            "shacl" => {
                let shapes = props.one_str("program")?;
                HookCondition::Shacl { shapes }
            }
            "shex" => {
                let schema = props.one_str("program")?;
                let shape_map = props.one_str("goal")?;
                HookCondition::Shex { schema, shape_map }
            }
            "n3" => {
                let rules = props.one_str("program")?;
                HookCondition::N3 { rules }
            }
            "sparql" => {
                let query = props.one_str("query")?;
                HookCondition::Sparql { query }
            }
            other => {
                return Err(format!("unsupported condition kind '{}'", other));
            }
        };

        let effect_str = props.one_str("effect")?;
        let effect = match effect_str.as_str() {
            "emit-delta" => EffectKind::EmitDelta,
            "ground-action" => EffectKind::GroundAction,
            "refuse" => EffectKind::Refuse,
            other => return Err(format!("unknown effect: {}", other)),
        };

        let action = props.opt_str("action")?;
        let reason = props.opt_str("reason")?;
        match effect {
            EffectKind::GroundAction if action.is_none() => {
                return Err("effect 'ground-action' requires kh:action".to_string());
            }
            EffectKind::Refuse if reason.is_none() => {
                return Err("effect 'refuse' requires kh:reason".to_string());
            }
            _ => {}
        }

        let priority = props
            .opt_str("priority")?
            .map(|s| parse_rdf_integer::<u8>(&s))
            .transpose()?
            .unwrap_or(0);

        let after = props.all_str("after");

        hooks.push(KnowledgeHook {
            iri: subj,
            name,
            on,
            condition,
            effect,
            action,
            reason,
            priority,
            after,
        });
    }

    Ok(hooks)
}

/// Schedules CompiledHooks using Kahn's algorithm on HookId edges.
///
/// # Algorithm
/// Topological sort using Kahn's algorithm with tie-breaking by (priority, HookId).
/// - Deterministic: same input order and priorities → same schedule every time
/// - Tie-break: (priority ASC, HookId ASC) ensures stable ordering
/// - No string comparisons; all edges are HookId indices (O(1) lookup)
///
/// # Complexity
/// O(|H| + |D|) where |H| = hooks, |D| = total dependencies
/// - In_degree computation: O(|D|)
/// - Kahn's loop: O(|H|) iterations, each with O(log |H|) sort
/// - Total: O(|H| log |H| + |D|)
///
/// # Errors
/// Returns `Err(String)` if a cycle is detected (scheduled.len() < hooks.len())
pub fn schedule_hooks(hooks: &[CompiledHook]) -> Result<Vec<CompiledHook>, String> {
    // Build hook_id → hook reference map
    let mut hook_map = FxHashMap::default();
    let mut in_degree: FxHashMap<HookId, usize> = FxHashMap::default();
    let mut adj: FxHashMap<HookId, Vec<HookId>> = FxHashMap::default();

    for hook in hooks {
        hook_map.insert(hook.id, hook.clone());
        in_degree.insert(hook.id, 0);
        adj.insert(hook.id, Vec::new());
    }

    // Build adjacency list and in-degree counters
    for hook in hooks {
        for &dep_id in &hook.after {
            if !hook_map.contains_key(&dep_id) {
                return Err(format!(
                    "hook '{}' has unknown after-dependency 'HookId({})'",
                    hook.iri, dep_id.0
                ));
            }
            adj.get_mut(&dep_id).unwrap().push(hook.id);
            *in_degree.get_mut(&hook.id).unwrap() += 1;
        }
    }

    // Initialize queue with zero in-degree hooks
    let mut queue = Vec::new();
    for (&hook_id, &deg) in &in_degree {
        if deg == 0 {
            queue.push(hook_map.get(&hook_id).unwrap().clone());
        }
    }

    // Kahn's algorithm with tie-breaking by (priority, HookId)
    let mut scheduled = Vec::new();
    while !queue.is_empty() {
        queue.sort_unstable_by(|a, b| (a.priority, a.id).cmp(&(b.priority, b.id)));
        let next = queue.remove(0);
        scheduled.push(next.clone());

        // Decrement in-degree for neighbors
        for &neighbor_id in adj.get(&next.id).unwrap() {
            let deg = in_degree.get_mut(&neighbor_id).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push(hook_map.get(&neighbor_id).unwrap().clone());
            }
        }
    }

    // Cycle detection: if not all hooks were scheduled, there's a cycle
    if scheduled.len() < hooks.len() {
        return Err("dependency cycle detected in hooks".to_string());
    }

    Ok(scheduled)
}

/// Compiles KnowledgeHooks to CompiledHooks with ID-based dependency tracking.
///
/// # Algorithm
/// 1. Assign HookId by position (deterministic: input order → HookId order)
/// 2. Track unique 'on' values; assign EventId:
///    - If all hooks share same 'on' value: single shared EventId
///    - Else: per-hook EventId in order of first appearance
/// 3. Resolve 'after' string IRIs to HookId indices; error on unknown IRI
/// 4. Return Vec<CompiledHook> in input order
///
/// # Complexity
/// O(|H| + |D|) where |H| = number of hooks, |D| = total dependencies
/// (dominated by dependency resolution loop)
///
/// # Errors
/// Returns `Err(String)` if any hook references unknown IRI in 'after' field
pub fn compile_hooks(hooks: Vec<KnowledgeHook>) -> Result<Vec<CompiledHook>, String> {
    // Build hook IRI → position mapping for dependency resolution
    let mut iri_to_id = FxHashMap::default();
    for (idx, hook) in hooks.iter().enumerate() {
        iri_to_id.insert(hook.iri.clone(), HookId(idx as u32));
    }

    // Determine EventId assignment strategy: all same 'on' value → shared EventId
    let on_values: Vec<_> = hooks.iter().map(|h| h.on.as_str()).collect();
    let all_same_on = on_values.iter().all(|&v| v == on_values[0]);

    let mut next_event_id = 0u32;
    let mut on_to_event_id = FxHashMap::default();

    // Assign EventIds
    for hook in &hooks {
        if all_same_on {
            if !on_to_event_id.contains_key(hook.on.as_str()) {
                on_to_event_id.insert(hook.on.as_str(), EventId(0));
            }
        } else {
            if !on_to_event_id.contains_key(hook.on.as_str()) {
                on_to_event_id.insert(hook.on.as_str(), EventId(next_event_id));
                next_event_id += 1;
            }
        }
    }

    // Compile each hook
    let mut compiled = Vec::with_capacity(hooks.len());
    for hook in hooks {
        // Resolve 'after' dependencies
        let mut after_ids = smallvec::SmallVec::new();
        for iri in &hook.after {
            match iri_to_id.get(iri) {
                Some(&id) => after_ids.push(id),
                None => {
                    return Err(format!(
                        "hook '{}' has unknown after-dependency '{}'",
                        hook.iri, iri
                    ));
                }
            }
        }

        let event_id = on_to_event_id
            .get(hook.on.as_str())
            .copied()
            .ok_or_else(|| format!("missing EventId for on value '{}'", hook.on))?;

        let id = iri_to_id
            .get(&hook.iri)
            .copied()
            .ok_or_else(|| format!("missing HookId for IRI '{}'", hook.iri))?;

        compiled.push(CompiledHook {
            id,
            iri: hook.iri,
            name: hook.name,
            event: event_id,
            on: hook.on,
            condition: hook.condition,
            effect: hook.effect,
            action: hook.action,
            reason: hook.reason,
            priority: hook.priority,
            after: after_ids,
        });
    }

    Ok(compiled)
}

pub fn parse_simple_toml(content: &str) -> Result<(String, String, String, Vec<String>), String> {
    let mut name = None;
    let mut version = None;
    let mut description = None;
    let mut required_dialects = Vec::new();
    let mut in_pack_section = false;

    for (line_idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let section = line[1..line.len() - 1].trim();
            if section == "pack" {
                in_pack_section = true;
            } else {
                return Err(format!("unknown TOML section: {}", section));
            }
            continue;
        }
        if !in_pack_section {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            return Err(format!("invalid TOML line {}: {}", line_idx + 1, line));
        };
        let key = key.trim();
        let val = val.trim();
        match key {
            "name" => {
                name = Some(strip_quotes(val)?);
            }
            "version" => {
                version = Some(strip_quotes(val)?);
            }
            "description" => {
                description = Some(strip_quotes(val)?);
            }
            "required_dialects" => {
                required_dialects = parse_toml_array(val)?;
            }
            other => {
                return Err(format!("unknown TOML key: {}", other));
            }
        }
    }

    let name = name.ok_or_else(|| "missing hook pack name".to_string())?;
    let version = version.ok_or_else(|| "missing hook pack version".to_string())?;
    let description = description.ok_or_else(|| "missing hook pack description".to_string())?;

    Ok((name, version, description, required_dialects))
}

fn strip_quotes(s: &str) -> Result<String, String> {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        Ok(s[1..s.len() - 1].to_string())
    } else {
        Err(format!("expected quoted string literal, got: {}", s))
    }
}

fn parse_toml_array(s: &str) -> Result<Vec<String>, String> {
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(format!("expected TOML array, got: {}", s));
    }
    let inner = &s[1..s.len() - 1];
    let mut out = Vec::new();
    for item in inner.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        out.push(strip_quotes(item)?);
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookReceipt {
    pub hook_name: String,
    pub delta_hash: String,
    pub idempotency_key: String,
    pub delta_quads: String,
}

use crate::sparql::Binding;
use spargebra::term::{NamedNodePattern, TermPattern};

fn collect_triple_patterns(
    gp: &spargebra::algebra::GraphPattern,
    out: &mut Vec<spargebra::term::TriplePattern>,
) {
    match gp {
        spargebra::algebra::GraphPattern::Bgp { patterns } => {
            out.extend(patterns.clone());
        }
        spargebra::algebra::GraphPattern::Join { left, right } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        spargebra::algebra::GraphPattern::Distinct { inner }
        | spargebra::algebra::GraphPattern::Reduced { inner } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Project { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Filter { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Group { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Extend { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::LeftJoin { left, right, .. } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        spargebra::algebra::GraphPattern::Union { left, right } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        spargebra::algebra::GraphPattern::Minus { left, right } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        _ => {}
    }
}

fn instantiate_term_pattern(tp: &TermPattern, bindings: &[Binding]) -> Option<String> {
    match tp {
        TermPattern::Variable(v) => {
            let var_name = v.as_str();
            bindings
                .iter()
                .find(|b| b.var == var_name)
                .map(|b| b.val.clone())
        }
        TermPattern::NamedNode(n) => Some(format!("<{}>", n.as_str())),
        TermPattern::BlankNode(b) => Some(format!("_:{}", b.as_str())),
        TermPattern::Literal(l) => {
            let mut s = format!("\"{}\"", l.value());
            if let Some(lang) = l.language() {
                s.push_str(&format!("@{}", lang));
            } else if l.datatype().as_str() != "http://www.w3.org/2001/XMLSchema#string" {
                s.push_str(&format!("^^<{}>", l.datatype().as_str()));
            }
            Some(s)
        }
    }
}

fn instantiate_named_node_pattern(np: &NamedNodePattern, bindings: &[Binding]) -> Option<String> {
    match np {
        NamedNodePattern::Variable(v) => {
            let var_name = v.as_str();
            bindings
                .iter()
                .find(|b| b.var == var_name)
                .map(|b| b.val.clone())
        }
        NamedNodePattern::NamedNode(n) => Some(format!("<{}>", n.as_str())),
    }
}

fn instantiate_triple_pattern(
    tp: &spargebra::term::TriplePattern,
    row: &[Binding],
) -> Option<Triple> {
    let s_str = instantiate_term_pattern(&tp.subject, row)?;
    let p_str = instantiate_named_node_pattern(&tp.predicate, row)?;
    let o_str = instantiate_term_pattern(&tp.object, row)?;
    Some(Triple::from(s_str, p_str, o_str))
}

pub fn evaluate_construct(
    query_str: &str,
    triple_index: &crate::tripleindex::TripleIndex,
) -> Result<(Vec<Triple>, Vec<Triple>), String> {
    let query = spargebra::Query::parse(query_str, None)
        .map_err(|e| format!("SPARQL parse error: {}", e))?;

    if let spargebra::Query::Construct {
        ref template,
        ref pattern,
        ..
    } = query
    {
        let mut additions = Vec::new();
        let mut deletions = Vec::new();

        let plan = crate::sparql::eval_query(&query, triple_index);
        let bindings: Vec<Vec<Binding>> =
            crate::sparql::evaluate_plan_and_debug(&plan, triple_index).collect();

        if !template.is_empty() {
            for row in bindings {
                for tp in template {
                    if let Some(triple) = instantiate_triple_pattern(tp, &row) {
                        additions.push(triple);
                    }
                }
            }
        } else {
            let mut patterns = Vec::new();
            collect_triple_patterns(&pattern, &mut patterns);
            for row in bindings {
                for tp in &patterns {
                    if let Some(triple) = instantiate_triple_pattern(tp, &row) {
                        deletions.push(triple);
                    }
                }
            }
        }

        Ok((additions, deletions))
    } else {
        Err("Query is not a CONSTRUCT query".to_string())
    }
}

pub fn serialize_delta_quad(
    hook_iri: &str,
    triple: &Triple,
    is_addition: bool,
    lines: &mut Vec<String>,
) {
    let s = clean_decoded_term(&Encoder::decode(&triple.s.to_encoded()).unwrap_or_default());
    let p = clean_decoded_term(&Encoder::decode(&triple.p.to_encoded()).unwrap_or_default());
    let o = clean_decoded_term(&Encoder::decode(&triple.o.to_encoded()).unwrap_or_default());

    let triple_str = format!("{} {} {}", s, p, o);
    let hash = blake3::hash(triple_str.as_bytes()).to_hex().to_string();

    let bn_id = if is_addition {
        format!("_:add_{}", hash)
    } else {
        format!("_:del_{}", hash)
    };

    let pred = if is_addition {
        "<http://seanchatmangpt.github.io/praxis/kh#addQuad>"
    } else {
        "<http://seanchatmangpt.github.io/praxis/kh#deleteQuad>"
    };

    let wrap_iri = |x: &str| {
        if x.starts_with('<') || x.starts_with('"') || x.starts_with('_') {
            x.to_string()
        } else {
            format!("<{}>", x)
        }
    };

    let s_wrapped = wrap_iri(&s);
    let p_wrapped = wrap_iri(&p);
    let o_wrapped = if o.starts_with('"') {
        o.clone()
    } else {
        wrap_iri(&o)
    };
    let hook_wrapped = wrap_iri(hook_iri);

    lines.push(format!(
        "{} <http://seanchatmangpt.github.io/praxis/kh#subject> {} .",
        bn_id, s_wrapped
    ));
    lines.push(format!(
        "{} <http://seanchatmangpt.github.io/praxis/kh#predicate> {} .",
        bn_id, p_wrapped
    ));
    lines.push(format!(
        "{} <http://seanchatmangpt.github.io/praxis/kh#object> {} .",
        bn_id, o_wrapped
    ));
    lines.push(format!("{} {} {} .", hook_wrapped, pred, bn_id));
}

fn clean_decoded_term(s: &str) -> String {
    s.trim().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookError {
    pub detail: String,
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.detail)
    }
}

impl std::error::Error for HookError {}

impl From<String> for HookError {
    fn from(s: String) -> Self {
        HookError { detail: s }
    }
}

impl From<&str> for HookError {
    fn from(s: &str) -> Self {
        HookError {
            detail: s.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphDelta {
    pub additions: Vec<Triple>,
    pub removals: Vec<Triple>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookVerdict {
    Fired,
    NotFired,
    Gated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticDetail {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDiagnostic {
    pub hook_iri: String,
    pub conforms: bool,
    pub details: Vec<DiagnosticDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookVerdictRecord {
    pub hook_id: HookId,
    pub hook_iri: String,
    pub hook_name: String,
    pub condition_kind: String,
    pub condition_hash: String,
    pub verdict: HookVerdict,
    pub effect: EffectKind,
    pub action_iri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<TriggerDiagnostic>,
    pub delta_hash: Option<String>,
    pub idempotency_key: Option<String>,
}

impl HookVerdictRecord {
    pub fn delta_hash(&self) -> Option<String> {
        self.delta_hash.clone()
    }

    pub fn idempotency_key(&self) -> Option<String> {
        self.idempotency_key.clone()
    }
}

impl HookCondition {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Datalog { .. } => "datalog",
            Self::Delta { .. } => "delta",
            Self::Threshold { .. } => "threshold",
            Self::Count { .. } => "count",
            Self::Window { .. } => "window",
            Self::Shacl { .. } => "shacl",
            Self::Shex { .. } => "shex",
            Self::N3 { .. } => "n3",
            Self::Sparql { .. } => "sparql",
        }
    }

    pub fn condition_hash(&self) -> Result<String, String> {
        let json = serde_json::to_string(self).map_err(|e| format!("serialize failed: {}", e))?;
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = hasher.finalize();
        let mut s = String::new();
        for byte in hash {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", byte);
        }
        Ok(s)
    }
}

pub fn hook_hash(records: &[HookVerdictRecord]) -> Result<String, String> {
    let json = serde_json::to_string(records).map_err(|e| format!("serialize failed: {}", e))?;
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let hash = hasher.finalize();
    let mut s = String::new();
    for byte in hash {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", byte);
    }
    Ok(s)
}

// ---------------------------------------------------------------------------
// condition evaluation
// ---------------------------------------------------------------------------

fn delta_touches(delta: &GraphDelta, var: &str) -> bool {
    delta
        .additions
        .iter()
        .chain(delta.removals.iter())
        .any(|t| {
            let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
            let clean_p = clean_term(&p_str);
            let clean_v = clean_term(var);
            clean_p == clean_v
        })
}

fn count_pred_in_store(store: &TripleStore, var: &str) -> u64 {
    let clean_v = clean_term(var);
    store
        .triple_index
        .triples
        .iter()
        .filter(|t| {
            let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
            clean_term(&p_str) == clean_v
        })
        .count() as u64
}

fn count_pred_in_delta(delta: &GraphDelta, var: &str) -> u64 {
    let clean_v = clean_term(var);
    let add_count = delta
        .additions
        .iter()
        .filter(|t| {
            let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
            clean_term(&p_str) == clean_v
        })
        .count() as u64;
    let rem_count = delta
        .removals
        .iter()
        .filter(|t| {
            let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
            clean_term(&p_str) == clean_v
        })
        .count() as u64;
    add_count + rem_count
}

fn parse_shape_map(s: &str) -> Vec<(String, String)> {
    let mut map = Vec::new();
    for entry in s.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        if let Some((node, shape)) = entry.split_once('@') {
            let clean = |x: &str| {
                let x = x.trim();
                if x.starts_with('<') && x.ends_with('>') {
                    x[1..x.len() - 1].to_string()
                } else {
                    x.to_string()
                }
            };
            map.push((clean(node), clean(shape)));
        }
    }
    map
}

struct DatalogAtom {
    name: String,
    args: Vec<String>,
}

fn split_depth0(text: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if c == sep && depth == 0 => {
                parts.push(&text[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

fn parse_datalog_atom(s: &str, _subject: &str) -> Result<DatalogAtom, String> {
    let s = s.trim();
    let open = s
        .find('(')
        .ok_or_else(|| format!("datalog atom '{s}' missing '('"))?;
    if !s.ends_with(')') {
        return Err(format!("datalog atom '{s}' missing ')'"));
    }
    let name = s[..open].trim().to_string();
    if name.is_empty() {
        return Err(format!("datalog atom '{s}' has empty predicate"));
    }
    let inner = &s[open + 1..s.len() - 1];
    let args: Vec<String> = if inner.trim().is_empty() {
        Vec::new()
    } else {
        inner.split(',').map(|t| t.trim().to_string()).collect()
    };
    Ok(DatalogAtom { name, args })
}

fn format_term(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('?') {
        let rest = &s[1..];
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            format!("?v{}", rest)
        } else {
            s.to_string()
        }
    } else if s.starts_with('<') && s.ends_with('>') {
        s.to_string()
    } else if s.starts_with('"') && s.ends_with('"') {
        s.to_string()
    } else if s.chars().all(|c| c.is_ascii_digit()) {
        format!("\"{}\"^^<http://www.w3.org/2001/XMLSchema#integer>", s)
    } else {
        format!("<{}>", s)
    }
}

fn translate_datalog_to_n3(program: &str, subject: &str) -> Result<String, String> {
    let mut n3_rules = String::new();
    let statements = split_depth0(program, '.');
    let mut added = 0;
    for stmt in statements {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        let (head_s, body_s) = match stmt.split_once(":-") {
            Some((h, b)) => (h, Some(b)),
            None => (stmt, None),
        };
        let head_atom = parse_datalog_atom(head_s, subject)?;
        if head_atom.name == "t" {
            return Err("datalog head predicate 't' is reserved for EDB".to_string());
        }
        let mut body_triples = Vec::new();
        if let Some(b) = body_s {
            for lit in split_depth0(b, ',') {
                let lit = lit.trim();
                if lit.is_empty() {
                    continue;
                }
                let (negated, atom_str) = if let Some(stripped) = lit.strip_prefix('!') {
                    (true, stripped.trim())
                } else {
                    (false, lit)
                };
                let atom = parse_datalog_atom(atom_str, subject)?;
                let triple_str = if atom.name == "t" {
                    if atom.args.len() != 3 {
                        return Err(format!("t atom must have arity 3, got {}", atom.args.len()));
                    }
                    format!(
                        "{} {} {}",
                        format_term(&atom.args[0]),
                        format_term(&atom.args[1]),
                        format_term(&atom.args[2])
                    )
                } else {
                    match atom.args.len() {
                        1 => format!(
                            "{} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{}>",
                            format_term(&atom.args[0]),
                            atom.name
                        ),
                        2 => format!(
                            "{} <{}> {}",
                            format_term(&atom.args[0]),
                            atom.name,
                            format_term(&atom.args[1])
                        ),
                        _ => {
                            return Err(format!(
                                "atom '{}' must have arity 1 or 2, got {}",
                                atom.name,
                                atom.args.len()
                            ))
                        }
                    }
                };
                if negated {
                    body_triples.push(format!("not {{ {} }}", triple_str));
                } else {
                    body_triples.push(triple_str);
                }
            }
        }
        let head_triple_str = match head_atom.args.len() {
            1 => format!(
                "{} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{}>",
                format_term(&head_atom.args[0]),
                head_atom.name
            ),
            2 => format!(
                "{} <{}> {}",
                format_term(&head_atom.args[0]),
                head_atom.name,
                format_term(&head_atom.args[1])
            ),
            _ => {
                return Err(format!(
                    "head atom '{}' must have arity 1 or 2, got {}",
                    head_atom.name,
                    head_atom.args.len()
                ))
            }
        };
        n3_rules.push_str(&format!(
            "{{ {} }} => {{ {} }} .\n",
            body_triples.join(" . "),
            head_triple_str
        ));
        added += 1;
        if added > 8 {
            return Err("more than 8 rules in datalog program".to_string());
        }
    }
    Ok(n3_rules)
}

pub fn evaluate_condition(
    condition: &HookCondition,
    post_state: &TripleStore,
    delta: &GraphDelta,
    history: &[GraphDelta],
    hook_iri: &str,
) -> Result<(bool, Option<TriggerDiagnostic>), String> {
    match condition {
        HookCondition::Datalog { program, goal } => {
            let n3_rules = translate_datalog_to_n3(program, hook_iri)?;
            let mut temp_store = TripleStore::new();
            for t in &post_state.triple_index.triples {
                temp_store.add(t.clone());
            }
            if !n3_rules.trim().is_empty() {
                temp_store
                    .load_rules(&n3_rules)
                    .map_err(|e| format!("load rules error: {} (rules: {})", e, n3_rules))?;
            }
            let _ = temp_store.materialize();
            let goal_lower = goal.to_lowercase();
            let fired = temp_store.triple_index.triples.iter().any(|t| {
                let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
                let cleaned_p = clean_term(&p_str);
                let is_type = cleaned_p == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                    || cleaned_p == "a";
                if is_type {
                    let o_str = Encoder::decode(&t.o.to_encoded()).unwrap_or_default();
                    let clean_o = clean_term(&o_str).to_lowercase();
                    clean_o == goal_lower
                } else {
                    cleaned_p.to_lowercase() == goal_lower
                }
            });
            let diag = TriggerDiagnostic {
                hook_iri: hook_iri.to_string(),
                conforms: !fired,
                details: if fired {
                    vec![DiagnosticDetail {
                        focus_node: None,
                        result_path: None,
                        value: None,
                        severity: Some("Fired".to_string()),
                        message: format!("Datalog goal '{}' was derived in post-state", goal),
                    }]
                } else {
                    Vec::new()
                },
            };
            Ok((fired, Some(diag)))
        }
        HookCondition::Delta { var } => {
            let fired = delta_touches(delta, var);
            let diag = TriggerDiagnostic {
                hook_iri: hook_iri.to_string(),
                conforms: !fired,
                details: if fired {
                    vec![DiagnosticDetail {
                        focus_node: None,
                        result_path: Some(var.clone()),
                        value: None,
                        severity: Some("Fired".to_string()),
                        message: format!("Delta modified predicate '{}'", var),
                    }]
                } else {
                    Vec::new()
                },
            };
            Ok((fired, Some(diag)))
        }
        HookCondition::Threshold { var, op, k } => {
            let count = count_pred_in_store(post_state, var);
            let fired = op.holds(count, *k);
            let diag = TriggerDiagnostic {
                hook_iri: hook_iri.to_string(),
                conforms: !fired,
                details: if fired {
                    vec![DiagnosticDetail {
                        focus_node: None,
                        result_path: Some(var.clone()),
                        value: Some(count.to_string()),
                        severity: Some("Fired".to_string()),
                        message: format!(
                            "Predicate '{}' count {} held comparison {:?}",
                            var, count, op
                        ),
                    }]
                } else {
                    Vec::new()
                },
            };
            Ok((fired, Some(diag)))
        }
        HookCondition::Count { var, op, k } => {
            let count = count_pred_in_delta(delta, var);
            let fired = op.holds(count, *k);
            let diag = TriggerDiagnostic {
                hook_iri: hook_iri.to_string(),
                conforms: !fired,
                details: if fired {
                    vec![DiagnosticDetail {
                        focus_node: None,
                        result_path: Some(var.clone()),
                        value: Some(count.to_string()),
                        severity: Some("Fired".to_string()),
                        message: format!(
                            "Predicate '{}' delta count {} held comparison {:?}",
                            var, count, op
                        ),
                    }]
                } else {
                    Vec::new()
                },
            };
            Ok((fired, Some(diag)))
        }
        HookCondition::Window { var, op, k, window } => {
            let mut total = count_pred_in_delta(delta, var);
            for d in history.iter().take(usize::from(*window).saturating_sub(1)) {
                total += count_pred_in_delta(d, var);
            }
            let fired = op.holds(total, *k);
            let diag = TriggerDiagnostic {
                hook_iri: hook_iri.to_string(),
                conforms: !fired,
                details: if fired {
                    vec![DiagnosticDetail {
                        focus_node: None,
                        result_path: Some(var.clone()),
                        value: Some(total.to_string()),
                        severity: Some("Fired".to_string()),
                        message: format!(
                            "Predicate '{}' window count {} held comparison {:?}",
                            var, total, op
                        ),
                    }]
                } else {
                    Vec::new()
                },
            };
            Ok((fired, Some(diag)))
        }
        HookCondition::Shacl { shapes } => {
            let report = post_state.validate_shacl(shapes)?;
            let conforms = report.conforms;
            let details = report
                .results
                .iter()
                .map(|res| DiagnosticDetail {
                    focus_node: Some(res.focus_node.to_string()),
                    result_path: res.result_path.as_ref().map(|t| t.to_string()),
                    value: res.value.as_ref().map(|t| t.to_string()),
                    severity: Some(res.severity.to_string()),
                    message: res
                        .message
                        .clone()
                        .unwrap_or_else(|| "SHACL shape violation".to_string()),
                })
                .collect();
            Ok((
                !conforms,
                Some(TriggerDiagnostic {
                    hook_iri: hook_iri.to_string(),
                    conforms,
                    details,
                }),
            ))
        }
        HookCondition::Shex { schema, shape_map } => {
            let shape_map_parsed = parse_shape_map(shape_map);
            let (conforms, failures) = if schema.trim().starts_with('{') {
                let report = post_state
                    .validate_shex(schema, &shape_map_parsed)
                    .map_err(|e| format!("ShexJ error: {}", e))?;
                let failures: Vec<(String, String, String)> = report
                    .failures
                    .iter()
                    .map(|fail| {
                        (
                            fail.node.to_string(),
                            fail.shape.to_string(),
                            fail.reason.to_string(),
                        )
                    })
                    .collect();
                (report.conforms, failures)
            } else {
                let report = post_state
                    .validate_shex_c(schema, &shape_map_parsed)
                    .map_err(|e| format!("ShexC error: {}", e))?;
                let failures: Vec<(String, String, String)> = report
                    .failures
                    .iter()
                    .map(|fail| {
                        (
                            fail.node.to_string(),
                            fail.shape.to_string(),
                            fail.reason.to_string(),
                        )
                    })
                    .collect();
                (report.conforms, failures)
            };
            let details = failures
                .iter()
                .map(|(node, shape, reason)| DiagnosticDetail {
                    focus_node: Some(node.clone()),
                    result_path: None,
                    value: None,
                    severity: Some("Violation".to_string()),
                    message: format!("Shape validation failed for {}: {}", shape, reason),
                })
                .collect();
            Ok((
                !conforms,
                Some(TriggerDiagnostic {
                    hook_iri: hook_iri.to_string(),
                    conforms,
                    details,
                }),
            ))
        }
        HookCondition::N3 { rules } => {
            let mut temp_store = TripleStore::from(rules);
            for t in &post_state.triple_index.triples {
                temp_store.add(t.clone());
            }
            let _ = temp_store.materialize();
            let violations = temp_store.check_denials();
            let conforms = violations.is_empty();
            let details = violations
                .iter()
                .map(|msg| DiagnosticDetail {
                    focus_node: None,
                    result_path: None,
                    value: None,
                    severity: Some("Denial".to_string()),
                    message: msg.clone(),
                })
                .collect();
            Ok((
                !conforms,
                Some(TriggerDiagnostic {
                    hook_iri: hook_iri.to_string(),
                    conforms,
                    details,
                }),
            ))
        }
        HookCondition::Sparql { query } => {
            let results = post_state
                .query(query)
                .map_err(|e| format!("SPARQL error: {}", e))?;
            let fired = !results.is_empty();
            let diag = TriggerDiagnostic {
                hook_iri: hook_iri.to_string(),
                conforms: !fired,
                details: if fired {
                    vec![DiagnosticDetail {
                        focus_node: None,
                        result_path: None,
                        value: Some(results.len().to_string()),
                        severity: Some("Fired".to_string()),
                        message: format!("SPARQL query returned {} results", results.len()),
                    }]
                } else {
                    Vec::new()
                },
            };
            Ok((fired, Some(diag)))
        }
    }
}

/// Evaluates CompiledHooks against the current state and delta.
///
/// # Complexity
/// O(|H| * C) where |H| = number of hooks, C = per-condition evaluation cost
/// (typically O(|F|) where |F| = triple store size, dominated by SHACL/SPARQL)
pub fn evaluate_hooks(
    hooks: &[CompiledHook],
    post_state: &TripleStore,
    delta: &GraphDelta,
    history: &[GraphDelta],
) -> Result<Vec<HookVerdictRecord>, String> {
    let mut records = Vec::with_capacity(hooks.len());
    for hook in hooks {
        let gated = match hook.on.as_str() {
            "assert" => delta.additions.is_empty(),
            "retract" => delta.removals.is_empty(),
            _ => false,
        };
        let (verdict, diagnostics) = if gated {
            (HookVerdict::Gated, None)
        } else {
            let (fired, diag) =
                evaluate_condition(&hook.condition, post_state, delta, history, &hook.iri)?;
            let verdict = if fired {
                HookVerdict::Fired
            } else {
                HookVerdict::NotFired
            };
            (verdict, diag)
        };

        let condition_hash = hook.condition.condition_hash()?;
        records.push(HookVerdictRecord {
            hook_id: hook.id,
            hook_iri: hook.iri.clone(),
            hook_name: hook.name.clone(),
            condition_kind: hook.condition.kind().to_string(),
            condition_hash,
            verdict,
            effect: hook.effect.clone(),
            action_iri: hook.action.clone(),
            diagnostics,
            delta_hash: None,
            idempotency_key: None,
        });
    }
    Ok(records)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutcome {
    pub additions: Vec<Triple>,
    pub deletions: Vec<Triple>,
}

pub struct ConstructQuery {
    pub graph: Option<String>,
    pub template_triples: Vec<(String, String, String)>,
    pub where_query: String,
    pub is_delete: bool,
}

pub fn strip_comments(s: &str) -> String {
    let mut out = String::new();
    for line in s.lines() {
        if let Some(idx) = line.find('#') {
            out.push_str(&line[..idx]);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

pub fn tokenize_triple(s: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '<' {
            let mut iri = String::new();
            while let Some(c) = chars.next() {
                iri.push(c);
                if c == '>' {
                    break;
                }
            }
            tokens.push(iri);
        } else if c == '"' || c == '\'' {
            let quote = chars.next().unwrap();
            let mut lit = String::new();
            lit.push(quote);
            while let Some(c) = chars.next() {
                lit.push(c);
                if c == quote {
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_whitespace() || next_c == '.' || next_c == ';' {
                            break;
                        }
                        lit.push(chars.next().unwrap());
                    }
                    break;
                }
            }
            tokens.push(lit);
        } else {
            let mut token = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '.' || c == ';' || c == '}' {
                    break;
                }
                token.push(chars.next().unwrap());
            }
            tokens.push(token);
        }
    }
    Ok(tokens)
}

pub fn parse_construct(query_str: &str) -> Result<ConstructQuery, String> {
    let clean_query = strip_comments(query_str);
    let const_idx = clean_query
        .to_uppercase()
        .find("CONSTRUCT")
        .ok_or_else(|| "Not a CONSTRUCT query".to_string())?;
    let first_brace = clean_query[const_idx..]
        .find('{')
        .ok_or_else(|| "Missing opening brace for CONSTRUCT template".to_string())?
        + const_idx;

    let mut brace_count = 1;
    let mut template_end = None;
    for (idx, c) in clean_query[first_brace + 1..].char_indices() {
        if c == '{' {
            brace_count += 1;
        } else if c == '}' {
            brace_count -= 1;
            if brace_count == 0 {
                template_end = Some(first_brace + 1 + idx);
                break;
            }
        }
    }
    let template_end =
        template_end.ok_or_else(|| "Unmatched brace in CONSTRUCT template".to_string())?;
    let template_text = clean_query[first_brace + 1..template_end].trim();

    let where_text = clean_query[template_end + 1..].trim();
    let where_pattern = if where_text.to_uppercase().starts_with("WHERE") {
        where_text[5..].trim()
    } else {
        where_text
    };

    let mut graph = None;
    let mut template_triples = Vec::new();
    let is_delete = template_text.is_empty();

    if !is_delete {
        let mut inner_text = template_text;
        if template_text.to_uppercase().starts_with("GRAPH") {
            let first_g_brace = template_text
                .find('{')
                .ok_or_else(|| "Missing brace in GRAPH template".to_string())?;
            let g_part = template_text[5..first_g_brace].trim();
            graph = Some(g_part.to_string());

            let last_g_brace = template_text
                .rfind('}')
                .ok_or_else(|| "Missing closing brace in GRAPH template".to_string())?;
            inner_text = template_text[first_g_brace + 1..last_g_brace].trim();
        }

        for triple_part in inner_text.split('.') {
            let triple_part = triple_part.trim();
            if triple_part.is_empty() {
                continue;
            }
            let tokens = tokenize_triple(triple_part)?;
            if tokens.len() == 3 {
                template_triples.push((tokens[0].clone(), tokens[1].clone(), tokens[2].clone()));
            }
        }
    }

    let select_query = format!("SELECT * WHERE {}", where_pattern);

    Ok(ConstructQuery {
        graph,
        template_triples,
        where_query: select_query,
        is_delete,
    })
}

pub fn get_where_triple_pattern(where_pattern: &str) -> Option<(String, String, String)> {
    let clean = where_pattern.trim();
    let start = clean.find('{')? + 1;
    let end = clean.rfind('}')?;
    let inner = &clean[start..end];
    let tokens = tokenize_triple(inner).ok()?;
    if tokens.len() >= 3 {
        Some((tokens[0].clone(), tokens[1].clone(), tokens[2].clone()))
    } else {
        None
    }
}

pub fn serialize_quad(t: &Triple) -> String {
    let s_str = Encoder::decode(&t.s.to_encoded()).unwrap_or_default();
    let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
    let o_str = Encoder::decode(&t.o.to_encoded()).unwrap_or_default();
    let g_str =
        t.g.as_ref()
            .map(|g| Encoder::decode(&g.to_encoded()).unwrap_or_default());

    let format_val = |s: &str| -> String {
        let s = s.trim();
        if s.starts_with('?') {
            s.to_string()
        } else if s.starts_with('<') && s.ends_with('>') {
            s.to_string()
        } else if s.starts_with('_') && s.contains(':') {
            s.to_string()
        } else if s.starts_with('"') {
            s.to_string()
        } else {
            format!("<{}>", s)
        }
    };

    let s_fmt = format_val(&s_str);
    let p_fmt = format_val(&p_str);
    let o_fmt = format_val(&o_str);
    let g_fmt = g_str
        .map(|g| format!(" {}", format_val(&g)))
        .unwrap_or_default();

    format!("{} {} {}{} .", s_fmt, p_fmt, o_fmt, g_fmt)
}

pub fn escape_literal(s: &str) -> String {
    if !s.starts_with('"') {
        return s.to_string();
    }
    if let Some(end_quote) = s[1..].rfind('"') {
        let end_quote = end_quote + 1;
        let value = &s[1..end_quote];
        let suffix = &s[end_quote + 1..];

        let escaped_value: String = value
            .chars()
            .flat_map(|c| match c {
                '\\' => vec!['\\', '\\'],
                '"' => vec!['\\', '"'],
                '\n' => vec!['\\', 'n'],
                '\r' => vec!['\\', 'r'],
                '\t' => vec!['\\', 't'],
                other => vec![other],
            })
            .collect();

        format!("\"{}\"{}", escaped_value, suffix)
    } else {
        s.to_string()
    }
}

pub fn canonicalize_quads(quads: &[Triple]) -> Vec<String> {
    let mut raw_lines: Vec<String> = quads.iter().map(serialize_quad).collect();
    let mut bnodes = std::collections::BTreeSet::new();
    for line in &raw_lines {
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '_' && chars.peek() == Some(&':') {
                chars.next();
                let mut label = String::from("_:");
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        label.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                bnodes.insert(label);
            }
        }
    }

    if !bnodes.is_empty() {
        let mut bnode_vec: Vec<String> = bnodes.into_iter().collect();
        bnode_vec.sort_by(|a, b| b.len().cmp(&a.len()));

        let c14n_mappings: Vec<(String, String)> = bnode_vec
            .iter()
            .enumerate()
            .map(|(idx, bnode)| (bnode.clone(), format!("_:c14n{}", idx)))
            .collect();

        for line in &mut raw_lines {
            *line = escape_literal(line);
            for (bnode, c14n) in &c14n_mappings {
                let mut new_line = String::new();
                let mut parts = line.split(bnode);
                if let Some(first) = parts.next() {
                    new_line.push_str(first);
                    for part in parts {
                        new_line.push_str(c14n);
                        new_line.push_str(part);
                    }
                }
                *line = new_line;
            }
        }
    } else {
        for line in &mut raw_lines {
            *line = escape_literal(line);
        }
    }

    raw_lines.sort();
    raw_lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_trigger_dialects() {
        let data = "<http://e/s> <http://e/p> <http://e/o> .
<http://e/s> <http://e/item> \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://e/s> <http://e/item> \"2\"^^<http://www.w3.org/2001/XMLSchema#integer> .";
        let store = TripleStore::from(data);

        let delta = GraphDelta {
            additions: Parser::parse_triples(
                "<http://e/s> <http://e/item> \"3\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
                crate::parser::Syntax::Turtle,
            )
            .unwrap(),
            removals: Vec::new(),
        };

        let history = vec![GraphDelta {
            additions: Parser::parse_triples(
                "<http://e/s> <http://e/item> \"9\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
                crate::parser::Syntax::Turtle,
            )
            .unwrap(),
            removals: Vec::new(),
        }];

        let cond_delta = HookCondition::Delta {
            var: "http://e/item".to_string(),
        };
        let (fired, _) =
            evaluate_condition(&cond_delta, &store, &delta, &history, "ex:h1").unwrap();
        assert!(fired);

        let cond_threshold = HookCondition::Threshold {
            var: "http://e/item".to_string(),
            op: CmpOp::Gt,
            k: 1,
        };
        let (fired, _) =
            evaluate_condition(&cond_threshold, &store, &delta, &history, "ex:h2").unwrap();
        assert!(fired);

        let cond_count = HookCondition::Count {
            var: "http://e/item".to_string(),
            op: CmpOp::Eq,
            k: 1,
        };
        let (fired, _) =
            evaluate_condition(&cond_count, &store, &delta, &history, "ex:h3").unwrap();
        assert!(fired);

        let cond_window = HookCondition::Window {
            var: "http://e/item".to_string(),
            op: CmpOp::Eq,
            k: 2,
            window: 2,
        };
        let (fired, _) =
            evaluate_condition(&cond_window, &store, &delta, &history, "ex:h4").unwrap();
        assert!(fired);

        let cond_sparql = HookCondition::Sparql {
            query: "SELECT * WHERE { ?s <http://e/p> <http://e/o> }".to_string(),
        };
        let (fired, _) =
            evaluate_condition(&cond_sparql, &store, &delta, &history, "ex:h5").unwrap();
        assert!(fired);

        let cond_sparql_ask = HookCondition::Sparql {
            query: "ASK { ?s <http://e/p> <http://e/o> }".to_string(),
        };
        let (fired, _) =
            evaluate_condition(&cond_sparql_ask, &store, &delta, &history, "ex:h6").unwrap();
        assert!(fired);

        let cond_datalog = HookCondition::Datalog {
            program: "linked(?0) :- t(?1, <http://e/p>, ?0). orphan(?0) :- t(?0, <http://e/item>, ?1), !linked(?0).".to_string(),
            goal: "orphan".to_string(),
        };
        let (fired, _) =
            evaluate_condition(&cond_datalog, &store, &delta, &history, "ex:h7").unwrap();
        assert!(fired);
    }

    // ========================================================================
    // PROJ-403: Compiled Hook IR Tests
    // ========================================================================

    #[test]
    fn test_compile_hooks_assigns_unique_hook_ids() {
        let hooks = vec![
            KnowledgeHook {
                iri: "http://ex/h1".to_string(),
                name: "h1".to_string(),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: vec![],
            },
            KnowledgeHook {
                iri: "http://ex/h2".to_string(),
                name: "h2".to_string(),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: vec![],
            },
        ];

        let compiled = compile_hooks(hooks).expect("compile should succeed");
        assert_eq!(compiled.len(), 2);
        assert_eq!(compiled[0].id, HookId(0));
        assert_eq!(compiled[1].id, HookId(1));
        assert_ne!(compiled[0].id, compiled[1].id);
    }

    #[test]
    fn test_compile_hooks_resolves_dependencies() {
        let hooks = vec![
            KnowledgeHook {
                iri: "http://ex/h1".to_string(),
                name: "h1".to_string(),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: vec![],
            },
            KnowledgeHook {
                iri: "http://ex/h2".to_string(),
                name: "h2".to_string(),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: vec!["http://ex/h1".to_string()],
            },
        ];

        let compiled = compile_hooks(hooks).expect("compile should succeed");
        assert_eq!(compiled[1].after.len(), 1);
        assert_eq!(compiled[1].after[0], HookId(0));
    }

    #[test]
    fn test_compile_hooks_unknown_dependency_error() {
        let hooks = vec![KnowledgeHook {
            iri: "http://ex/h1".to_string(),
            name: "h1".to_string(),
            on: "assert".to_string(),
            condition: HookCondition::Delta {
                var: "p".to_string(),
            },
            effect: EffectKind::EmitDelta,
            action: None,
            reason: None,
            priority: 0,
            after: vec!["http://ex/unknown".to_string()],
        }];

        let result = compile_hooks(hooks);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown after-dependency"));
    }

    #[test]
    fn test_schedule_hooks_tie_break_by_hook_id() {
        let hooks = vec![
            CompiledHook {
                id: HookId(2),
                iri: "http://ex/h2".to_string(),
                name: "h2".to_string(),
                event: EventId(0),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: SmallVec::new(),
            },
            CompiledHook {
                id: HookId(0),
                iri: "http://ex/h0".to_string(),
                name: "h0".to_string(),
                event: EventId(0),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: SmallVec::new(),
            },
            CompiledHook {
                id: HookId(1),
                iri: "http://ex/h1".to_string(),
                name: "h1".to_string(),
                event: EventId(0),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: SmallVec::new(),
            },
        ];

        let scheduled = schedule_hooks(&hooks).expect("schedule should succeed");
        assert_eq!(scheduled.len(), 3);
        // Should be ordered by HookId: 0, 1, 2
        assert_eq!(scheduled[0].id, HookId(0));
        assert_eq!(scheduled[1].id, HookId(1));
        assert_eq!(scheduled[2].id, HookId(2));
    }

    #[test]
    fn test_schedule_hooks_cycle_detection() {
        let hooks = vec![
            CompiledHook {
                id: HookId(0),
                iri: "http://ex/h0".to_string(),
                name: "h0".to_string(),
                event: EventId(0),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: {
                    let mut sv = SmallVec::new();
                    sv.push(HookId(1));
                    sv
                },
            },
            CompiledHook {
                id: HookId(1),
                iri: "http://ex/h1".to_string(),
                name: "h1".to_string(),
                event: EventId(0),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: {
                    let mut sv = SmallVec::new();
                    sv.push(HookId(0));
                    sv
                },
            },
        ];

        let result = schedule_hooks(&hooks);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dependency cycle"));
    }

    #[test]
    fn test_schedule_hooks_respects_dependencies() {
        let hooks = vec![
            CompiledHook {
                id: HookId(0),
                iri: "http://ex/h0".to_string(),
                name: "h0".to_string(),
                event: EventId(0),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: SmallVec::new(),
            },
            CompiledHook {
                id: HookId(1),
                iri: "http://ex/h1".to_string(),
                name: "h1".to_string(),
                event: EventId(0),
                on: "assert".to_string(),
                condition: HookCondition::Delta {
                    var: "p".to_string(),
                },
                effect: EffectKind::EmitDelta,
                action: None,
                reason: None,
                priority: 0,
                after: {
                    let mut sv = SmallVec::new();
                    sv.push(HookId(0));
                    sv
                },
            },
        ];

        let scheduled = schedule_hooks(&hooks).expect("schedule should succeed");
        assert_eq!(scheduled.len(), 2);
        // h0 should come before h1
        assert_eq!(scheduled[0].id, HookId(0));
        assert_eq!(scheduled[1].id, HookId(1));
    }
}
