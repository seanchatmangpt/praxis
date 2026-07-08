//! Knowledge hooks — declared graph trigger/effect units, h = (trigger,
//! check, act, receipt).
//!
//! Hooks live IN the admitted graph as `hook:` nodes: the registry is
//! extracted from the same triples the law-hash covers, so hook declarations
//! are content-addressed for free. Every registered hook produces a verdict
//! record on every event — `NotFired` is recorded, so silence is provable
//! (knhk Covenant-2: a firing without a receipt is a violation).
//!
//! Condition kinds are the bounded subset praxis can evaluate with controlled
//! in-crate or graphlaw-backed engines: `datalog`, `delta`, `threshold`,
//! `count`, `window`, `shacl`, `shex`, and `n3`.
//! Unsupported external/query kinds such as `sparql-ask`, `sparql-select`,
//! and `semantic-inference` are refused by name with a supported analog.
//!
//! Lineage: knhk 03-knowledge-hooks.tex hook tuple -> unrdf
//! `define-hook.mjs`/`condition-evaluator.mjs` condition-kind contract ->
//! this Rust port (the `src/frontier.rs` reimplementation note, made real);
//! knowd `hooks_v33.rs` trigger vocabulary (OnDelta analog of OnCommit).

use serde::{Deserialize, Serialize};

use chatman_common::provenance::content_address;

use crate::datalog::{Atom as DlAtom, DlRule, Program, Term};
use crate::delta::GraphDelta;
use crate::graph::{render_object, Object, Triple};
use crate::quarantine::AdmittedEvent;
use crate::Refusal;

/// The hook vocabulary namespace. Closed world, same law as `wf:`.
pub const HOOK_NS: &str = "http://seanchatmangpt.github.io/praxis/hook#";

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

const HOOK_CLASSES: [&str; 1] = ["Hook"];
const HOOK_PREDICATES: [&str; 14] = [
    "name", "on", "kind", "var", "op", "k", "window", "program", "goal", "action", "effect",
    "reason", "priority", "after",
];

/// Max hooks per registry — the 8-bound.
pub const MAX_HOOKS: usize = 12;
/// Max datalog program text bytes per hook condition.
pub const MAX_PROGRAM_BYTES: usize = 4_096;

/// Refused condition kinds mapped to their honest supported analog.
const REFUSED_KINDS: [(&str, &str); 3] = [
    ("sparql-ask", "datalog (goal reachability)"),
    ("sparql-select", "datalog (goal reachability)"),
    (
        "semantic-inference",
        "(none — refused everywhere; unrdf itself throws unimplemented)",
    ),
];

fn ill(subject: &str, detail: impl Into<String>) -> Refusal {
    Refusal::HookIllFormed {
        subject: subject.to_string(),
        detail: detail.into(),
    }
}

/// Comparison operator for counting conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpOp {
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
}

impl CmpOp {
    fn parse(s: &str, subject: &str) -> Result<Self, Refusal> {
        Ok(match s {
            "=" => Self::Eq,
            "!=" => Self::Ne,
            "<" => Self::Lt,
            "<=" => Self::Le,
            ">" => Self::Gt,
            ">=" => Self::Ge,
            other => return Err(ill(subject, format!("unknown hook:op '{other}'"))),
        })
    }

    fn holds(self, lhs: u64, rhs: u64) -> bool {
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

/// The bounded condition kinds. Every variant is deterministically
/// evaluable in-crate; anything else is refused at registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookCondition {
    /// Fires iff `goal` is derivable: EDB = post-state triples projected as
    /// `t(s, p, o)` facts, rules from the bounded micro-syntax in
    /// `hook:program`, semi-naive stratified saturation, then
    /// `count_for(goal) > 0`. The honest sparql-ask/shacl analog.
    Datalog {
        /// Rule text (micro-syntax: `head(?0) :- t(?0, p, o), !bad(?0).`).
        program: String,
        /// Goal predicate name.
        goal: String,
    },
    /// Fires iff this delta touches predicate `var` (addition or removal).
    Delta {
        /// Watched predicate IRI.
        var: String,
    },
    /// count(triples in POST-state with predicate `var`) OP k.
    Threshold {
        /// Watched predicate IRI.
        var: String,
        /// Comparator.
        op: CmpOp,
        /// Bound.
        k: u64,
    },
    /// count(occurrences in THIS delta with predicate `var`) OP k.
    Count {
        /// Watched predicate IRI.
        var: String,
        /// Comparator.
        op: CmpOp,
        /// Bound.
        k: u64,
    },
    /// Count over this delta plus the last `window - 1` deltas; window 1..=8.
    Window {
        /// Watched predicate IRI.
        var: String,
        /// Comparator.
        op: CmpOp,
        /// Bound.
        k: u64,
        /// Number of deltas covered including the current one (1..=8).
        window: u8,
    },
    /// SHACL policy validation trigger (FR3, FR4)
    Shacl {
        /// Turtle serialization of shapes.
        shapes: String,
    },
    /// ShEx policy validation trigger (FR3, FR4)
    Shex {
        /// JSON (ShExJ) or compact (ShExC) serialization of the schema.
        schema: String,
        /// Shape map string (e.g. "node@shape").
        shape_map: String,
    },
    /// N3 policy validation trigger (FR3, FR4)
    N3 {
        /// N3 rules text.
        rules: String,
    },
}

impl HookCondition {
    /// Kind string as declared in the graph.
    #[must_use]
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
        }
    }

    /// Content address of the condition's serde rendering — conditions are
    /// content-addressed (the unrdf pattern), never referenced by mutable name.
    pub fn condition_hash(&self) -> Result<String, Refusal> {
        let json = serde_json::to_string(self).map_err(|e| Refusal::InvalidInput {
            detail: format!("condition failed to serialize: {e}"),
        })?;
        Ok(content_address(json.as_bytes()))
    }
}

/// Declared effect kind. The act leg of the tuple: no arbitrary code —
/// unrdf's sandboxed function effects are deliberately absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectKind {
    /// Emit a candidate delta (re-enters the quarantine door).
    EmitDelta,
    /// Ground a declared action (the `hook:action` workflow fragment).
    GroundAction,
    /// Refuse: firing yields a refusal receipt with the declared reason.
    Refuse,
}

/// One declared knowledge hook, extracted from the admitted graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeHook {
    /// The hook node's IRI.
    pub iri: String,
    /// Declared unique name.
    /// The name of the hook pack.
    pub name: String,
    /// Trigger gate: `assert` (fires only on additions), `retract`
    /// (only on removals), or `any`.
    pub on: String,
    /// The check.
    pub condition: HookCondition,
    /// The act.
    pub effect: EffectKind,
    /// Action fragment IRI (required iff effect = GroundAction).
    pub action: Option<String>,
    /// Declared refusal reason (required iff effect = Refuse).
    pub reason: Option<String>,
    /// Evaluation priority 0..=7 (ties broken by IRI byte order).
    pub priority: u8,
    /// Dependency IRIs (this hook evaluates after all named dependencies).
    pub after: Vec<String>,
}

/// Verdict of one hook against one event. All verdicts are recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookVerdict {
    /// Condition held; the effect is due.
    Fired,
    /// Condition did not hold. Recorded — silence is provable.
    NotFired,
    /// The trigger gate excluded this event (e.g. `on = assert`, empty additions).
    Gated,
}

/// Typed structured diagnostic details for trigger failures or conformance checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDiagnostic {
    /// The hook name or IRI.
    pub hook_iri: String,
    /// Conforms: true if validation passed, false if failed.
    pub conforms: bool,
    /// The list of specific diagnostic details.
    pub details: Vec<DiagnosticDetail>,
}

/// A single diagnostic violation or violation detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticDetail {
    /// Focus node (e.g. subject node that failed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_node: Option<String>,
    /// Result path (e.g. predicate that failed validation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_path: Option<String>,
    /// Value that failed validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Severity: violation, warning, info, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    /// Message explaining the failure.
    pub message: String,
}

/// One row of the hook verdict record — the bytes behind `hook_hash`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookVerdictRecord {
    /// Hook IRI.
    pub hook_iri: String,
    /// Hook name.
    pub hook_name: String,
    /// Condition kind.
    pub condition_kind: String,
    /// Computed condition hash.
    pub condition_hash: String,
    /// Verdict.
    pub verdict: HookVerdict,
    /// Effect kind declared.
    pub effect: EffectKind,
    /// Action IRI (when declared).
    pub action_iri: Option<String>,
    /// Optional structured diagnostics associated with the verdict.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<TriggerDiagnostic>,
}

// ---------------------------------------------------------------------------
// extraction
// ---------------------------------------------------------------------------

struct HookProps<'a> {
    props: Vec<(&'a str, &'a Object)>,
}

impl<'a> HookProps<'a> {
    fn objects(&self, local: &str) -> Vec<&'a Object> {
        let pred = format!("{HOOK_NS}{local}");
        self.props
            .iter()
            .filter(|(p, _)| *p == pred)
            .map(|(_, o)| *o)
            .collect()
    }

    fn one_str(&self, subject: &str, local: &str) -> Result<String, Refusal> {
        match self.objects(local).as_slice() {
            [Object::Str(s)] => Ok(s.clone()),
            [] => Err(ill(subject, format!("missing hook:{local}"))),
            [_] => Err(ill(
                subject,
                format!("hook:{local} must be a string literal"),
            )),
            _ => Err(ill(subject, format!("multiple hook:{local}"))),
        }
    }

    fn opt_str(&self, subject: &str, local: &str) -> Result<Option<String>, Refusal> {
        match self.objects(local).as_slice() {
            [] => Ok(None),
            [Object::Str(s)] => Ok(Some(s.clone())),
            [_] => Err(ill(
                subject,
                format!("hook:{local} must be a string literal"),
            )),
            _ => Err(ill(subject, format!("multiple hook:{local}"))),
        }
    }

    fn opt_int(&self, subject: &str, local: &str) -> Result<Option<i64>, Refusal> {
        match self.objects(local).as_slice() {
            [] => Ok(None),
            [Object::Int(v)] => Ok(Some(*v)),
            [_] => Err(ill(
                subject,
                format!("hook:{local} must be an integer literal"),
            )),
            _ => Err(ill(subject, format!("multiple hook:{local}"))),
        }
    }

    fn opt_iri(&self, subject: &str, local: &str) -> Result<Option<String>, Refusal> {
        match self.objects(local).as_slice() {
            [] => Ok(None),
            [Object::Iri(iri)] => Ok(Some(iri.clone())),
            [_] => Err(ill(subject, format!("hook:{local} must be an IRI"))),
            _ => Err(ill(subject, format!("multiple hook:{local}"))),
        }
    }

    fn all_iri(&self, subject: &str, local: &str) -> Result<Vec<String>, Refusal> {
        let mut out = Vec::new();
        for obj in self.objects(local) {
            if let Object::Iri(iri) = obj {
                out.push(iri.clone());
            } else {
                return Err(ill(subject, format!("hook:{local} must be an IRI")));
            }
        }
        out.sort_unstable();
        Ok(out)
    }

    #[allow(dead_code)]
    fn iris(&self, subject: &str, local: &str) -> Result<Vec<String>, Refusal> {
        let mut out = Vec::new();
        for obj in self.objects(local) {
            match obj {
                Object::Iri(iri) => out.push(iri.clone()),
                _ => return Err(ill(subject, format!("hook:{} must be an IRI", local))),
            }
        }
        Ok(out)
    }
}

fn cmp_fields(props: &HookProps<'_>, subject: &str) -> Result<(String, CmpOp, u64), Refusal> {
    let var = props.one_str(subject, "var")?;
    let op = CmpOp::parse(&props.one_str(subject, "op")?, subject)?;
    let k = match props.opt_int(subject, "k")? {
        Some(v) if v >= 0 =>
        {
            #[allow(clippy::cast_sign_loss)]
            (v as u64)
        }
        Some(v) => return Err(ill(subject, format!("hook:k {v} must be non-negative"))),
        None => return Err(ill(subject, "missing hook:k")),
    };
    Ok((var, op, k))
}

/// Extract the hook registry from admitted triples. Closed-world over the
/// `hook:` namespace: unknown predicates/classes and shape violations are
/// [`Refusal::HookIllFormed`]; unsupported condition kinds are
/// [`Refusal::ConditionUnsupported`] naming the honest analog. At most
/// [`MAX_HOOKS`] hooks; datalog programs are parse-validated HERE so an
/// ill-formed rule is refused at registration, never at firing time.
pub fn extract_hooks(triples: &[Triple]) -> Result<Vec<KnowledgeHook>, Refusal> {
    // Closed-world vocabulary sweep.
    for t in triples {
        if let Some(local) = t.p.strip_prefix(HOOK_NS) {
            if !HOOK_PREDICATES.contains(&local) {
                return Err(ill(&t.s, format!("unknown hook: predicate '{local}'")));
            }
        }
        if t.p == RDF_TYPE {
            if let Object::Iri(class) = &t.o {
                if let Some(local) = class.strip_prefix(HOOK_NS) {
                    if !HOOK_CLASSES.contains(&local) {
                        return Err(ill(&t.s, format!("unknown hook: class '{local}'")));
                    }
                }
            }
        }
    }

    // Collect hook subjects.
    let hook_class = format!("{HOOK_NS}Hook");
    let mut subjects: Vec<&str> = triples
        .iter()
        .filter(|t| t.p == RDF_TYPE && matches!(&t.o, Object::Iri(c) if *c == hook_class))
        .map(|t| t.s.as_str())
        .collect();
    subjects.sort_unstable();
    subjects.dedup();
    if subjects.len() > MAX_HOOKS {
        return Err(ill(
            "(registry)",
            format!("{} hooks declared; max {MAX_HOOKS}", subjects.len()),
        ));
    }

    let mut hooks = Vec::with_capacity(subjects.len());
    let mut names = std::collections::BTreeSet::new();
    for subject in subjects {
        let props = HookProps {
            props: triples
                .iter()
                .filter(|t| t.s == subject)
                .map(|t| (t.p.as_str(), &t.o))
                .collect(),
        };
        let name = props.one_str(subject, "name")?;
        if !names.insert(name.clone()) {
            return Err(ill(subject, format!("duplicate hook name '{name}'")));
        }
        let on = props
            .opt_str(subject, "on")?
            .unwrap_or_else(|| "any".to_string());
        if !matches!(on.as_str(), "assert" | "retract" | "any") {
            return Err(ill(
                subject,
                format!("hook:on '{on}' not in assert|retract|any"),
            ));
        }
        let kind = props.one_str(subject, "kind")?;
        let condition = match kind.as_str() {
            "datalog" => {
                let program = props.one_str(subject, "program")?;
                if program.len() > MAX_PROGRAM_BYTES {
                    return Err(Refusal::GraphCapExceeded {
                        what: "hook_program_bytes".to_string(),
                        cap: MAX_PROGRAM_BYTES as u64,
                        actual: program.len() as u64,
                    });
                }
                let goal = props.one_str(subject, "goal")?;
                // Registration-time validation: rules must parse and pass
                // the engine's safety checks against a scratch program,
                // AND stratify — stratification is EDB-independent, so a
                // trial saturation over the empty EDB proves it here. An
                // unstratifiable hook is refused at registration, never at
                // firing time.
                let mut scratch = Program::new();
                add_rules(&mut scratch, &program, subject)?;
                scratch.saturate().map_err(|e| {
                    ill(
                        subject,
                        format!("datalog program rejected at registration: {e}"),
                    )
                })?;
                HookCondition::Datalog { program, goal }
            }
            "delta" => HookCondition::Delta {
                var: props.one_str(subject, "var")?,
            },
            "threshold" => {
                let (var, op, k) = cmp_fields(&props, subject)?;
                HookCondition::Threshold { var, op, k }
            }
            "count" => {
                let (var, op, k) = cmp_fields(&props, subject)?;
                HookCondition::Count { var, op, k }
            }
            "window" => {
                let (var, op, k) = cmp_fields(&props, subject)?;
                let window = match props.opt_int(subject, "window")? {
                    Some(v) if (1..=8).contains(&v) =>
                    {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        (v as u8)
                    }
                    Some(v) => {
                        return Err(ill(subject, format!("hook:window {v} out of range 1..=8")))
                    }
                    None => return Err(ill(subject, "missing hook:window")),
                };
                HookCondition::Window { var, op, k, window }
            }
            "shacl" => {
                let program = props.one_str(subject, "program")?;
                HookCondition::Shacl { shapes: program }
            }
            "shex" => {
                let program = props.one_str(subject, "program")?;
                let goal = props.one_str(subject, "goal")?;
                HookCondition::Shex {
                    schema: program,
                    shape_map: goal,
                }
            }
            "n3" => {
                let program = props.one_str(subject, "program")?;
                HookCondition::N3 { rules: program }
            }
            other => {
                let analog = if let Some((_, a)) = REFUSED_KINDS.iter().find(|(k, _)| *k == other) {
                    (*a).to_string()
                } else {
                    "datalog".to_string()
                };
                return Err(Refusal::ConditionUnsupported {
                    kind: other.to_string(),
                    subject: subject.to_string(),
                    supported_analog: analog,
                });
            }
        };
        let effect = match props.one_str(subject, "effect")?.as_str() {
            "emit-delta" => EffectKind::EmitDelta,
            "ground-action" => EffectKind::GroundAction,
            "refuse" => EffectKind::Refuse,
            other => {
                return Err(ill(
                    subject,
                    format!("hook:effect '{other}' not in emit-delta|ground-action|refuse"),
                ))
            }
        };
        let action = props.opt_iri(subject, "action")?;
        let reason = props.opt_str(subject, "reason")?;
        match effect {
            EffectKind::GroundAction if action.is_none() => {
                return Err(ill(subject, "effect 'ground-action' requires hook:action"));
            }
            EffectKind::Refuse if reason.is_none() => {
                return Err(ill(subject, "effect 'refuse' requires hook:reason"));
            }
            _ => {}
        }
        let priority = match props.opt_int(subject, "priority")? {
            None => 0,
            Some(v) if (0..=7).contains(&v) =>
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                (v as u8)
            }
            Some(v) => {
                return Err(ill(
                    subject,
                    format!("hook:priority {v} out of range 0..=7"),
                ))
            }
        };
        let after = props.all_iri(subject, "after")?;
        hooks.push(KnowledgeHook {
            iri: subject.to_string(),
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
    hooks.sort_unstable_by(|a, b| (a.priority, a.iri.as_str()).cmp(&(b.priority, b.iri.as_str())));
    Ok(hooks)
}

// ---------------------------------------------------------------------------
// datalog micro-syntax
// ---------------------------------------------------------------------------

/// Parse one term: `?0..?7` is a variable; anything else is an interned
/// constant.
fn parse_term(program: &mut Program, tok: &str) -> Term {
    if let Some(rest) = tok.strip_prefix('?') {
        if rest.len() == 1 && rest.as_bytes()[0].is_ascii_digit() {
            let d = rest.as_bytes()[0] - b'0';
            if d < 8 {
                return Term::Var(d);
            }
        }
    }
    Term::Const(program.intern(tok))
}

fn parse_atom(program: &mut Program, s: &str, subject: &str) -> Result<DlAtom, Refusal> {
    let s = s.trim();
    let open = s
        .find('(')
        .ok_or_else(|| ill(subject, format!("datalog atom '{s}' missing '('")))?;
    if !s.ends_with(')') {
        return Err(ill(subject, format!("datalog atom '{s}' missing ')'")));
    }
    let name = s[..open].trim();
    if name.is_empty() {
        return Err(ill(
            subject,
            format!("datalog atom '{s}' has empty predicate"),
        ));
    }
    let inner = &s[open + 1..s.len() - 1];
    let args: Vec<Term> = if inner.trim().is_empty() {
        Vec::new()
    } else {
        inner
            .split(',')
            .map(|t| parse_term(program, t.trim()))
            .collect()
    };
    let pred = program.intern(name);
    Ok(DlAtom::new(pred, args))
}

/// Split on `sep` only at paren depth 0, so atom argument lists (and dotted
/// IRIs inside them) survive statement/literal splitting.
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

/// Parse the bounded rule micro-syntax and add every rule to `program`.
///
/// Grammar (one statement per '.'): `head :- lit, lit, ...` where a literal
/// is `atom` or `!atom`; atoms are `name(arg, ...)`; args `?0..?7` are
/// variables, all else constants. Every rule requires at least one POSITIVE
/// body atom (a bodiless rule would assert a fact through program text) and
/// the head predicate `t` is reserved for the EDB projection. Engine safety
/// (bound head/negation vars, arity, MAX_VARS) is enforced by
/// [`Program::add_rule`].
fn add_rules(program: &mut Program, text: &str, subject: &str) -> Result<usize, Refusal> {
    let mut added = 0usize;
    for stmt in split_depth0(text, '.') {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        let (head_s, body_s) = match stmt.split_once(":-") {
            Some((h, b)) => (h, Some(b)),
            None => (stmt, None),
        };
        let head = parse_atom(program, head_s, subject)?;
        if head.pred == program.intern("t") {
            return Err(ill(
                subject,
                "datalog head predicate 't' is reserved for the EDB projection",
            ));
        }
        let mut body = Vec::new();
        let mut negative = Vec::new();
        if let Some(b) = body_s {
            for lit in split_depth0(b, ',') {
                let lit = lit.trim();
                if let Some(pos) = lit.strip_prefix('!') {
                    negative.push(parse_atom(program, pos, subject)?);
                } else {
                    body.push(parse_atom(program, lit, subject)?);
                }
            }
        }
        if body.is_empty() {
            return Err(ill(
                subject,
                "datalog rule must have at least one positive body atom; \
                 facts cannot be asserted via program text",
            ));
        }
        program
            .add_rule(DlRule {
                head,
                body,
                negative,
            })
            .map_err(|e| ill(subject, format!("datalog rule rejected by engine: {e}")))?;
        added += 1;
        if added > 8 {
            return Err(ill(subject, "more than 8 datalog rules in one hook"));
        }
    }
    Ok(added)
}

// ---------------------------------------------------------------------------
// evaluation
// ---------------------------------------------------------------------------

fn count_pred(triples: &[Triple], var: &str) -> u64 {
    triples.iter().filter(|t| t.p == var).count() as u64
}

fn delta_touches(delta: &GraphDelta, var: &str) -> bool {
    delta
        .additions()
        .iter()
        .chain(delta.removals().iter())
        .any(|t| t.p == var)
}

fn delta_count(delta: &GraphDelta, var: &str) -> u64 {
    count_pred(delta.additions(), var) + count_pred(delta.removals(), var)
}

/// Evaluate one bounded datalog condition over post-state triples: EDB is
/// `t(s, p, o)` in canonical renderings; returns whether `goal` is derivable.
/// Shared with `livelock.rs` so livelock detection reuses THIS evaluator
/// instead of duplicating it.
pub(crate) fn eval_datalog(
    program_text: &str,
    goal: &str,
    post: &[Triple],
    subject: &str,
) -> Result<bool, Refusal> {
    let mut program = Program::new();
    // EDB: every post-state triple as t(s, p, o), every position in its
    // CANONICAL rendering (`<iri>`, `"str"`, decimal) — uniform across
    // positions, so a value bound at an object position joins against the
    // same value at a subject position.
    let t_pred = program.intern("t");
    for triple in post {
        let s = program.intern(&format!("<{}>", triple.s));
        let p = program.intern(&format!("<{}>", triple.p));
        let o = program.intern(&render_object(&triple.o));
        program.add_fact(t_pred, &[s, p, o])?;
    }
    add_rules(&mut program, program_text, subject)?;
    program.saturate()?;
    let goal_pred = program.intern(goal);
    Ok(program.count_for(goal_pred) > 0)
}

/// Evaluate every registered hook against one admitted event. `history` is
/// the most-recent-first list of previously admitted deltas (used only by
/// `window` conditions). Returns one verdict record PER REGISTERED HOOK —
/// `NotFired` and `Gated` rows included, so absence of firing is in the hash.
fn escape_str(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn to_graphlaw_triple(t: &crate::graph::Triple) -> praxis_graphlaw::term::Triple {
    let s_str = if t.s.starts_with('<') || t.s.starts_with('_') {
        t.s.clone()
    } else {
        format!("<{}>", t.s)
    };
    let p_str = if t.p.starts_with('<') {
        t.p.clone()
    } else {
        format!("<{}>", t.p)
    };
    let o_str = match &t.o {
        crate::graph::Object::Iri(iri) => format!("<{}>", iri),
        crate::graph::Object::Str(s) => format!("\"{}\"", escape_str(s)),
        crate::graph::Object::Int(v) => {
            format!("\"{}\"^^<http://www.w3.org/2001/XMLSchema#integer>", v)
        }
    };
    praxis_graphlaw::term::Triple::from(s_str, p_str, o_str)
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

/// Evaluates every registered hook condition against an event.
pub fn evaluate_hooks(
    hooks: &[KnowledgeHook],
    event: &AdmittedEvent,
    history: &[GraphDelta],
) -> Result<Vec<HookVerdictRecord>, Refusal> {
    let mut records = Vec::with_capacity(hooks.len());
    for hook in hooks {
        let gated = match hook.on.as_str() {
            "assert" => event.delta().additions().is_empty(),
            "retract" => event.delta().removals().is_empty(),
            _ => false,
        };
        let (verdict, diagnostics) = if gated {
            (HookVerdict::Gated, None)
        } else {
            let (fired, diagnostics) = match &hook.condition {
                HookCondition::Datalog { program, goal } => {
                    let fired = eval_datalog(program, goal, event.post(), &hook.iri)?;
                    let diag = TriggerDiagnostic {
                        hook_iri: hook.iri.clone(),
                        conforms: !fired,
                        details: if fired {
                            vec![DiagnosticDetail {
                                focus_node: None,
                                result_path: None,
                                value: None,
                                severity: Some("Fired".to_string()),
                                message: format!(
                                    "Datalog goal '{}' was derived in post-state",
                                    goal
                                ),
                            }]
                        } else {
                            Vec::new()
                        },
                    };
                    (fired, Some(diag))
                }
                HookCondition::Delta { var } => {
                    let fired = delta_touches(event.delta(), var);
                    let diag = TriggerDiagnostic {
                        hook_iri: hook.iri.clone(),
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
                    (fired, Some(diag))
                }
                HookCondition::Threshold { var, op, k } => {
                    let count = count_pred(event.post(), var);
                    let fired = op.holds(count, *k);
                    let op_str = match op {
                        CmpOp::Eq => "=",
                        CmpOp::Ne => "!=",
                        CmpOp::Lt => "<",
                        CmpOp::Le => "<=",
                        CmpOp::Gt => ">",
                        CmpOp::Ge => ">=",
                    };
                    let diag = TriggerDiagnostic {
                        hook_iri: hook.iri.clone(),
                        conforms: !fired,
                        details: if fired {
                            vec![DiagnosticDetail {
                                focus_node: None,
                                result_path: Some(var.clone()),
                                value: Some(count.to_string()),
                                severity: Some("Fired".to_string()),
                                message: format!(
                                    "Predicate '{}' count {} held comparison {} {}",
                                    var, count, op_str, k
                                ),
                            }]
                        } else {
                            Vec::new()
                        },
                    };
                    (fired, Some(diag))
                }
                HookCondition::Count { var, op, k } => {
                    let count = delta_count(event.delta(), var);
                    let fired = op.holds(count, *k);
                    let op_str = match op {
                        CmpOp::Eq => "=",
                        CmpOp::Ne => "!=",
                        CmpOp::Lt => "<",
                        CmpOp::Le => "<=",
                        CmpOp::Gt => ">",
                        CmpOp::Ge => ">=",
                    };
                    let diag = TriggerDiagnostic {
                        hook_iri: hook.iri.clone(),
                        conforms: !fired,
                        details: if fired {
                            vec![DiagnosticDetail {
                                focus_node: None,
                                result_path: Some(var.clone()),
                                value: Some(count.to_string()),
                                severity: Some("Fired".to_string()),
                                message: format!(
                                    "Predicate '{}' delta count {} held comparison {} {}",
                                    var, count, op_str, k
                                ),
                            }]
                        } else {
                            Vec::new()
                        },
                    };
                    (fired, Some(diag))
                }
                HookCondition::Window { var, op, k, window } => {
                    let mut total = delta_count(event.delta(), var);
                    for d in history.iter().take(usize::from(*window) - 1) {
                        total += delta_count(d, var);
                    }
                    let fired = op.holds(total, *k);
                    let op_str = match op {
                        CmpOp::Eq => "=",
                        CmpOp::Ne => "!=",
                        CmpOp::Lt => "<",
                        CmpOp::Le => "<=",
                        CmpOp::Gt => ">",
                        CmpOp::Ge => ">=",
                    };
                    let diag = TriggerDiagnostic {
                        hook_iri: hook.iri.clone(),
                        conforms: !fired,
                        details: if fired {
                            vec![DiagnosticDetail {
                                focus_node: None,
                                result_path: Some(var.clone()),
                                value: Some(total.to_string()),
                                severity: Some("Fired".to_string()),
                                message: format!(
                                    "Predicate '{}' window count {} held comparison {} {}",
                                    var, total, op_str, k
                                ),
                            }]
                        } else {
                            Vec::new()
                        },
                    };
                    (fired, Some(diag))
                }
                HookCondition::Shacl { shapes } => {
                    let mut store = praxis_graphlaw::TripleStore::new();
                    for t in event.post() {
                        store.add(to_graphlaw_triple(t));
                    }
                    let report = store
                        .validate_shacl(shapes)
                        .map_err(|e| Refusal::InvalidInput { detail: e })?;
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
                    (
                        !conforms,
                        Some(TriggerDiagnostic {
                            hook_iri: hook.iri.clone(),
                            conforms,
                            details,
                        }),
                    )
                }
                HookCondition::Shex { schema, shape_map } => {
                    let mut store = praxis_graphlaw::TripleStore::new();
                    for t in event.post() {
                        store.add(to_graphlaw_triple(t));
                    }
                    let shape_map_parsed = parse_shape_map(shape_map);
                    let report = if schema.trim().starts_with('{') {
                        praxis_graphlaw::shex::validate_shex(
                            &store.triple_index,
                            schema,
                            &shape_map_parsed,
                        )
                        .map_err(|e| Refusal::InvalidInput {
                            detail: e.to_string(),
                        })?
                    } else {
                        store
                            .validate_shex_c(schema, &shape_map_parsed)
                            .map_err(|e| Refusal::InvalidInput { detail: e })?
                    };
                    let conforms = report.conforms;
                    let details = report
                        .failures
                        .iter()
                        .map(|fail| DiagnosticDetail {
                            focus_node: Some(fail.node.to_string()),
                            result_path: None,
                            value: None,
                            severity: Some("Violation".to_string()),
                            message: format!(
                                "Shape validation failed for {}: {}",
                                fail.shape, fail.reason
                            ),
                        })
                        .collect();
                    (
                        !conforms,
                        Some(TriggerDiagnostic {
                            hook_iri: hook.iri.clone(),
                            conforms,
                            details,
                        }),
                    )
                }
                HookCondition::N3 { rules } => {
                    let mut store = praxis_graphlaw::TripleStore::from(rules);
                    for t in event.post() {
                        store.add(to_graphlaw_triple(t));
                    }
                    store.materialize();
                    let violations = store.check_denials();
                    let conforms = violations.is_empty();
                    let details = violations
                        .iter()
                        .map(|message| DiagnosticDetail {
                            focus_node: None,
                            result_path: None,
                            value: None,
                            severity: Some("Denial".to_string()),
                            message: message.clone(),
                        })
                        .collect();
                    (
                        !conforms,
                        Some(TriggerDiagnostic {
                            hook_iri: hook.iri.clone(),
                            conforms,
                            details,
                        }),
                    )
                }
            };
            if fired {
                (HookVerdict::Fired, diagnostics)
            } else {
                (HookVerdict::NotFired, diagnostics)
            }
        };
        records.push(HookVerdictRecord {
            hook_iri: hook.iri.clone(),
            hook_name: hook.name.clone(),
            condition_kind: hook.condition.kind().to_string(),
            condition_hash: hook.condition.condition_hash()?,
            verdict,
            effect: hook.effect.clone(),
            action_iri: hook.action.clone(),
            diagnostics,
        });
    }
    Ok(records)
}

/// Content address of the full verdict record list (already in registry
/// order: priority then IRI). The `hook_hash` stage of the firing chain.
pub fn hook_hash(records: &[HookVerdictRecord]) -> Result<String, Refusal> {
    let json = serde_json::to_string(records).map_err(|e| Refusal::InvalidInput {
        detail: format!("hook verdict records failed to serialize: {e}"),
    })?;
    Ok(content_address(json.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quarantine::{Admission, Reference};

    const EX: &str = "http://e/";

    fn hook_doc(body: &str) -> String {
        format!("@prefix hook: <{HOOK_NS}> .\n@prefix ex: <{EX}> .\n{body}")
    }

    fn admitted(base_extra: &str, adds: &str, removes: &str) -> AdmittedEvent {
        let base = hook_doc(base_extra);
        let reference = Reference::genesis(&base).expect("base admits");
        let delta = GraphDelta::parse(adds, removes).expect("delta parses");
        Admission::admit(&reference, &delta).expect("admits")
    }

    const DELTA_HOOK: &str = "ex:h1 a hook:Hook ; hook:name \"watch-p\" ; \
        hook:kind \"delta\" ; hook:var \"http://e/p\" ; hook:effect \"refuse\" ; \
        hook:reason \"p is frozen\" .\n";

    #[test]
    fn delta_hook_fires_and_notfired_is_recorded() {
        let event = admitted(DELTA_HOOK, "<http://e/x> <http://e/p> 1 .", "");
        let hooks = extract_hooks(event.post()).expect("registry extracts");
        assert_eq!(hooks.len(), 1);
        let records = evaluate_hooks(&hooks, &event, &[]).expect("evaluates");
        assert_eq!(records[0].verdict, HookVerdict::Fired);

        let quiet = admitted(DELTA_HOOK, "<http://e/x> <http://e/q> 1 .", "");
        let hooks = extract_hooks(quiet.post()).expect("registry extracts");
        let records = evaluate_hooks(&hooks, &quiet, &[]).expect("evaluates");
        assert_eq!(
            records[0].verdict,
            HookVerdict::NotFired,
            "silence is recorded"
        );
        assert!(hook_hash(&records).unwrap().len() > 16);
    }

    #[test]
    fn unsupported_kinds_refused_by_name_with_analog() {
        for kind in ["sparql-ask", "sparql-select", "semantic-inference"] {
            let doc = hook_doc(&format!(
                "ex:h a hook:Hook ; hook:name \"x\" ; hook:kind \"{kind}\" ; \
                 hook:effect \"refuse\" ; hook:reason \"r\" ."
            ));
            let triples = crate::graph::parse_ttl(&doc).expect("parses");
            match extract_hooks(&triples) {
                Err(Refusal::ConditionUnsupported {
                    kind: k,
                    supported_analog,
                    ..
                }) => {
                    assert_eq!(k, kind);
                    assert!(!supported_analog.is_empty());
                }
                other => panic!("expected ConditionUnsupported({kind}), got {other:?}"),
            }
        }
    }

    #[test]
    fn hook_shape_refusals_each_named() {
        let cases: &[(&str, &str)] = &[
            ("ex:h a hook:Hook ; hook:kind \"delta\" ; hook:var \"v\" ; hook:effect \"refuse\" ; hook:reason \"r\" .", "missing hook:name"),
            ("ex:h a hook:Hook ; hook:name \"x\" ; hook:kind \"delta\" ; hook:var \"v\" ; hook:effect \"ground-action\" .", "requires hook:action"),
            ("ex:h a hook:Hook ; hook:name \"x\" ; hook:kind \"delta\" ; hook:var \"v\" ; hook:effect \"refuse\" .", "requires hook:reason"),
            ("ex:h a hook:Hook ; hook:name \"x\" ; hook:kind \"threshold\" ; hook:var \"v\" ; hook:op \"~\" ; hook:k 1 ; hook:effect \"refuse\" ; hook:reason \"r\" .", "unknown hook:op"),
            ("ex:h a hook:Hook ; hook:name \"x\" ; hook:kind \"window\" ; hook:var \"v\" ; hook:op \">\" ; hook:k 1 ; hook:window 9 ; hook:effect \"refuse\" ; hook:reason \"r\" .", "out of range 1..=8"),
            ("ex:h a hook:Hook ; hook:name \"x\" ; hook:kind \"delta\" ; hook:var \"v\" ; hook:effect \"refuse\" ; hook:reason \"r\" ; hook:frobnicate 1 .", "unknown hook: predicate"),
            ("ex:h a hook:Gizmo .", "unknown hook: class"),
        ];
        for (body, needle) in cases {
            let triples = crate::graph::parse_ttl(&hook_doc(body)).expect("parses");
            match extract_hooks(&triples) {
                Err(Refusal::HookIllFormed { detail, .. }) => assert!(
                    detail.contains(needle),
                    "detail '{detail}' missing '{needle}'"
                ),
                other => panic!("expected HookIllFormed({needle}), got {other:?}"),
            }
        }
    }

    #[test]
    fn datalog_condition_fires_via_rules() {
        // orphan(X) :- t(X, rdf:type, <thing>), !linked(X); linked comes from
        // a t(_, links, X) projection rule.
        let body = "ex:h a hook:Hook ; hook:name \"orphans\" ; hook:kind \"datalog\" ; \
            hook:program \"linked(?0) :- t(?1, <http://e/links>, ?0). \
            orphan(?0) :- t(?0, <http://e/is>, <http://e/thing>), !linked(?0).\" ; \
            hook:goal \"orphan\" ; hook:effect \"refuse\" ; hook:reason \"orphan admitted\" .\n";
        // Base declares one linked thing; the delta admits an unlinked one.
        let base_extra = format!(
            "{body}ex:a <http://e/is> <http://e/thing> .\nex:root <http://e/links> ex:a .\n"
        );
        let event = admitted(
            &base_extra,
            "<http://e/b> <http://e/is> <http://e/thing> .",
            "",
        );
        let hooks = extract_hooks(event.post()).expect("extracts");
        let records = evaluate_hooks(&hooks, &event, &[]).expect("evaluates");
        assert_eq!(
            records[0].verdict,
            HookVerdict::Fired,
            "unlinked b is an orphan"
        );

        // Linking b in the same delta keeps the goal underivable.
        let event2 = admitted(
            &base_extra,
            "<http://e/b> <http://e/is> <http://e/thing> .\n\
             <http://e/root> <http://e/links> <http://e/b> .",
            "",
        );
        let hooks2 = extract_hooks(event2.post()).expect("extracts");
        let records2 = evaluate_hooks(&hooks2, &event2, &[]).expect("evaluates");
        assert_eq!(records2[0].verdict, HookVerdict::NotFired);
    }

    #[test]
    fn threshold_count_window_and_gate() {
        let body = "ex:h1 a hook:Hook ; hook:name \"cap\" ; hook:kind \"threshold\" ; \
            hook:var \"http://e/item\" ; hook:op \">\" ; hook:k 2 ; \
            hook:effect \"refuse\" ; hook:reason \"over\" .\n\
            ex:h2 a hook:Hook ; hook:name \"burst\" ; hook:kind \"window\" ; \
            hook:var \"http://e/item\" ; hook:op \">=\" ; hook:k 3 ; hook:window 2 ; \
            hook:effect \"refuse\" ; hook:reason \"burst\" .\n\
            ex:h3 a hook:Hook ; hook:name \"retracts\" ; hook:on \"retract\" ; \
            hook:kind \"count\" ; hook:var \"http://e/item\" ; hook:op \">\" ; hook:k 0 ; \
            hook:effect \"refuse\" ; hook:reason \"loss\" .\n";
        let base_extra = format!("{body}ex:w <http://e/item> 1 .\nex:w <http://e/item> 2 .\n");
        let prior = GraphDelta::parse("<http://e/w> <http://e/item> 9 .", "").unwrap();
        let event = admitted(&base_extra, "<http://e/w> <http://e/item> 3 .", "");
        let hooks = extract_hooks(event.post()).expect("extracts");
        assert_eq!(hooks.len(), 3);
        let records = evaluate_hooks(&hooks, &event, std::slice::from_ref(&prior)).unwrap();
        let by_name = |n: &str| records.iter().find(|r| r.hook_name == n).unwrap();
        // post has 3 item triples > 2 → fired
        assert_eq!(by_name("cap").verdict, HookVerdict::Fired);
        // window 2: this delta (1) + prior (1) = 2 < 3 → not fired
        assert_eq!(by_name("burst").verdict, HookVerdict::NotFired);
        // on=retract with pure-addition delta → gated
        assert_eq!(by_name("retracts").verdict, HookVerdict::Gated);
    }

    #[test]
    fn registry_bound_and_duplicate_names_refused() {
        let mut body = String::new();
        for i in 0..13 {
            body.push_str(&format!(
                "ex:h{i} a hook:Hook ; hook:name \"h{i}\" ; hook:kind \"delta\" ; \
                 hook:var \"v\" ; hook:effect \"refuse\" ; hook:reason \"r\" .\n"
            ));
        }
        let triples = crate::graph::parse_ttl(&hook_doc(&body)).expect("parses");
        match extract_hooks(&triples) {
            Err(Refusal::HookIllFormed { detail, .. }) => assert!(detail.contains("max 12")),
            other => panic!("expected registry bound refusal, got {other:?}"),
        }
    }
}

/// A parsed, validated hook pack representing a group of hooks and their metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents a hook pack structure.
pub struct HookPack {
    /// The name of the hook pack.
    pub name: String,
    /// The version of the hook pack.
    pub version: String,
    /// The description of the hook pack.
    pub description: String,
    /// The list of required dialects.
    pub required_dialects: Vec<String>,
    /// The list of hooks inside the pack.
    pub hooks: Vec<KnowledgeHook>,
}

/// Schedules the hooks in topological order of after-dependencies.
pub fn schedule_hooks(hooks: Vec<KnowledgeHook>) -> Result<Vec<KnowledgeHook>, Refusal> {
    let mut hook_map = std::collections::BTreeMap::new();
    let mut in_degree = std::collections::BTreeMap::new();
    let mut adj = std::collections::BTreeMap::new();

    for h in &hooks {
        hook_map.insert(h.iri.clone(), h.clone());
        in_degree.insert(h.iri.clone(), 0);
        adj.insert(h.iri.clone(), Vec::new());
    }

    for h in &hooks {
        for dep in &h.after {
            if !hook_map.contains_key(dep) {
                return Err(Refusal::HookIllFormed {
                    subject: h.iri.clone(),
                    detail: format!("unknown after-dependency '{}'", dep),
                });
            }
            adj.get_mut(dep).unwrap().push(h.iri.clone());
            *in_degree.get_mut(&h.iri).unwrap() += 1;
        }
    }

    let mut zero_in_degree = Vec::new();
    for (iri, &deg) in &in_degree {
        if deg == 0 {
            zero_in_degree.push(hook_map.get(iri).unwrap().clone());
        }
    }

    let mut scheduled = Vec::new();
    while !zero_in_degree.is_empty() {
        zero_in_degree.sort_unstable_by(|a, b| {
            (a.priority, a.iri.as_str()).cmp(&(b.priority, b.iri.as_str()))
        });
        let next = zero_in_degree.remove(0);
        scheduled.push(next.clone());

        for neighbor_iri in adj.get(&next.iri).unwrap() {
            let deg = in_degree.get_mut(neighbor_iri).unwrap();
            *deg -= 1;
            if *deg == 0 {
                zero_in_degree.push(hook_map.get(neighbor_iri).unwrap().clone());
            }
        }
    }

    if scheduled.len() < hooks.len() {
        return Err(Refusal::HookIllFormed {
            subject: "(registry)".to_string(),
            detail: "dependency cycle detected in hooks".to_string(),
        });
    }

    Ok(scheduled)
}

struct SimplePackMeta {
    name: String,
    version: String,
    description: String,
    required_dialects: Vec<String>,
}

fn parse_simple_toml(content: &str) -> Result<SimplePackMeta, Refusal> {
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
            let section = &line[1..line.len() - 1].trim();
            if *section == "pack" {
                in_pack_section = true;
            } else {
                return Err(Refusal::InvalidInput {
                    detail: format!("unknown TOML section: {}", section),
                });
            }
            continue;
        }
        if !in_pack_section {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            return Err(Refusal::InvalidInput {
                detail: format!("invalid TOML line {}: {}", line_idx + 1, line),
            });
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
                return Err(Refusal::InvalidInput {
                    detail: format!("unknown TOML key: {}", other),
                });
            }
        }
    }

    let name = name.ok_or_else(|| Refusal::InvalidInput {
        detail: "missing hook pack name".to_string(),
    })?;
    let version = version.ok_or_else(|| Refusal::InvalidInput {
        detail: "missing hook pack version".to_string(),
    })?;
    let description = description.ok_or_else(|| Refusal::InvalidInput {
        detail: "missing hook pack description".to_string(),
    })?;

    Ok(SimplePackMeta {
        name,
        version,
        description,
        required_dialects,
    })
}

fn strip_quotes(s: &str) -> Result<String, Refusal> {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        Ok(s[1..s.len() - 1].to_string())
    } else {
        Err(Refusal::InvalidInput {
            detail: format!("expected quoted string literal, got: {}", s),
        })
    }
}

fn parse_toml_array(s: &str) -> Result<Vec<String>, Refusal> {
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(Refusal::InvalidInput {
            detail: format!("expected TOML array, got: {}", s),
        });
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

/// Loads and validates a hook pack from the specified directory, checking metadata,
/// dialects, forbidden actions, duplicate hooks, and scheduling constraints.
pub fn load_hook_pack(pack_dir: &std::path::Path) -> Result<HookPack, Refusal> {
    let toml_path = pack_dir.join("pack.toml");
    let toml_content = std::fs::read_to_string(&toml_path).map_err(|e| Refusal::InvalidInput {
        detail: format!("pack.toml missing or unreadable: {e}"),
    })?;
    let meta = parse_simple_toml(&toml_content)?;

    for dialect in &meta.required_dialects {
        if ![
            "datalog",
            "delta",
            "threshold",
            "count",
            "window",
            "shacl",
            "shex",
            "n3",
        ]
        .contains(&dialect.as_str())
        {
            return Err(Refusal::ConditionUnsupported {
                kind: dialect.clone(),
                subject: "(pack)".to_string(),
                supported_analog: "datalog, delta, threshold, count, window, shacl, shex, n3"
                    .to_string(),
            });
        }
    }

    let ttl_path = pack_dir.join("ontology.ttl");
    let ttl_content = std::fs::read_to_string(&ttl_path).map_err(|e| Refusal::InvalidInput {
        detail: format!("ontology.ttl missing or unreadable: {e}"),
    })?;
    let triples = crate::graph::parse_ttl(&ttl_content)?;

    let wf_handler_iri = format!("{}handler", crate::graph::WF_NS);
    for t in &triples {
        if t.p == wf_handler_iri {
            if let Object::Iri(handler_iri) = &t.o {
                let allowed = format!("{}deterministic-v1", crate::handlers::HANDLER_NS);
                if handler_iri != &allowed {
                    return Err(Refusal::HookIllFormed {
                        subject: t.s.clone(),
                        detail: format!("forbidden handler IRI: {}", handler_iri),
                    });
                }
            }
        }
        let check_for_forbidden = |text: &str| -> bool {
            let text = text.to_lowercase();
            let suspicious = ["shell", "exec", "network", "curl", "socket", "fetch"];
            suspicious.iter().any(|&keyword| {
                text.contains(keyword)
                    && !text.starts_with("http://seanchatmangpt.github.io/praxis/")
                    && !text.starts_with("http://www.w3.org/")
            })
        };
        if check_for_forbidden(&t.s) || check_for_forbidden(&t.p) {
            return Err(Refusal::HookIllFormed {
                subject: t.s.clone(),
                detail: format!("forbidden keyword in triple: {} {}", t.s, t.p),
            });
        }
        match &t.o {
            Object::Iri(iri) if check_for_forbidden(iri) => {
                return Err(Refusal::HookIllFormed {
                    subject: t.s.clone(),
                    detail: format!("forbidden keyword in object IRI: {}", iri),
                });
            }
            Object::Str(s) if check_for_forbidden(s) => {
                return Err(Refusal::HookIllFormed {
                    subject: t.s.clone(),
                    detail: format!("forbidden keyword in string literal: {}", s),
                });
            }
            _ => {}
        }
    }

    let hooks = extract_hooks(&triples)?;

    let mut iris = std::collections::BTreeSet::new();
    for hook in &hooks {
        if !iris.insert(hook.iri.clone()) {
            return Err(Refusal::HookIllFormed {
                subject: hook.iri.clone(),
                detail: format!("duplicate hook IRI '{}'", hook.iri),
            });
        }
    }

    let scheduled_hooks = schedule_hooks(hooks)?;

    Ok(HookPack {
        name: meta.name,
        version: meta.version,
        description: meta.description,
        required_dialects: meta.required_dialects,
        hooks: scheduled_hooks,
    })
}
