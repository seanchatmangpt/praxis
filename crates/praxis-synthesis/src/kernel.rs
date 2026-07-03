//! The Lord's Prayer kernel — all 11 clauses as typed, closed-world nodes.
//!
//! The `prayer-kernel:` namespace declares the prayer's clause structure as
//! DATA in the admitted graph: each clause names its problem class and its
//! delegation boundary. God is never modeled as an agent, handler, or
//! capability — the boundary is a STRING property (`"human-only"` |
//! `"god-receives-unbounded"` | `"automatable-support"`), not an executable
//! node. Clauses whose boundary is `"god-receives-unbounded"` mark the
//! unbounded as surrendered, never computed.
//!
//! Extraction mirrors [`crate::hooks`]: closed vocabulary, shape rules, and
//! EXACT clause coverage — the kernel must declare all 11 canonical clauses,
//! no more, no fewer. Every violation is a typed
//! [`Refusal::KernelIllFormed`], never a silent skip.

use serde::{Deserialize, Serialize};

use chatman_common::provenance::content_address;

use crate::graph::{Object, Triple};
use crate::Refusal;

/// The prayer-kernel vocabulary namespace. Closed world, same law as
/// `hook:` — distinct from the `.../prayer#` hook-instance namespace.
pub const KERNEL_NS: &str = "http://seanchatmangpt.github.io/praxis/prayer-kernel#";

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

const KERNEL_CLASSES: [&str; 2] = ["Kernel", "Clause"];
const KERNEL_PREDICATES: [&str; 5] = ["clause", "name", "problemClass", "boundary", "action"];

/// The three lawful boundary strings. `"god-receives-unbounded"` is a data
/// marker of surrender — it never resolves to a handler.
pub const BOUNDARIES: [&str; 3] =
    ["human-only", "god-receives-unbounded", "automatable-support"];

/// The 11 canonical clause names, in scriptural order. Extraction requires
/// this set EXACTLY (order is not required in the graph; coverage is).
pub const CANONICAL_CLAUSES: [&str; 11] = [
    "our-father",
    "hallowed-name",
    "kingdom-come",
    "will-be-done",
    "on-earth-as-heaven",
    "daily-bread",
    "forgive-debts",
    "forgive-debtors",
    "temptation-guard",
    "deliverance",
    "doxology",
];

fn ill(subject: &str, detail: impl Into<String>) -> Refusal {
    Refusal::KernelIllFormed { subject: subject.to_string(), detail: detail.into() }
}

/// One extracted prayer clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrayerClause {
    /// The clause node's IRI.
    pub iri: String,
    /// Canonical clause name (one of [`CANONICAL_CLAUSES`]).
    pub name: String,
    /// The problem class this clause addresses (free string, declared).
    pub problem_class: String,
    /// Delegation boundary (one of [`BOUNDARIES`]).
    pub boundary: String,
    /// Optional IRI of a hook or workflow that SUPPORTS the clause.
    pub action: Option<String>,
}

struct Props<'a> {
    props: Vec<(&'a str, &'a Object)>,
}

impl<'a> Props<'a> {
    fn objects(&self, local: &str) -> Vec<&'a Object> {
        let pred = format!("{KERNEL_NS}{local}");
        self.props.iter().filter(|(p, _)| *p == pred).map(|(_, o)| *o).collect()
    }

    fn one_str(&self, subject: &str, local: &str) -> Result<String, Refusal> {
        match self.objects(local).as_slice() {
            [Object::Str(s)] => Ok(s.clone()),
            [] => Err(ill(subject, format!("missing prayer-kernel:{local}"))),
            [_] => Err(ill(subject, format!("prayer-kernel:{local} must be a string literal"))),
            _ => Err(ill(subject, format!("multiple prayer-kernel:{local}"))),
        }
    }

    fn opt_iri(&self, subject: &str, local: &str) -> Result<Option<String>, Refusal> {
        match self.objects(local).as_slice() {
            [] => Ok(None),
            [Object::Iri(iri)] => Ok(Some(iri.clone())),
            [_] => Err(ill(subject, format!("prayer-kernel:{local} must be an IRI"))),
            _ => Err(ill(subject, format!("multiple prayer-kernel:{local}"))),
        }
    }
}

/// Extract the prayer kernel from admitted triples. Closed-world over the
/// `prayer-kernel:` namespace: unknown predicates/classes, shape violations,
/// and any deviation from EXACT 11-clause canonical coverage are
/// [`Refusal::KernelIllFormed`] naming the culprit (including which
/// canonical clause is missing). Returns the clauses sorted by canonical
/// order.
pub fn extract_kernel(triples: &[Triple]) -> Result<Vec<PrayerClause>, Refusal> {
    // Closed-world vocabulary sweep.
    for t in triples {
        if let Some(local) = t.p.strip_prefix(KERNEL_NS) {
            if !KERNEL_PREDICATES.contains(&local) {
                return Err(ill(&t.s, format!("unknown prayer-kernel: predicate '{local}'")));
            }
        }
        if t.p == RDF_TYPE {
            if let Object::Iri(class) = &t.o {
                if let Some(local) = class.strip_prefix(KERNEL_NS) {
                    if !KERNEL_CLASSES.contains(&local) {
                        return Err(ill(&t.s, format!("unknown prayer-kernel: class '{local}'")));
                    }
                }
            }
        }
    }

    // Exactly one Kernel node.
    let kernel_class = format!("{KERNEL_NS}Kernel");
    let mut kernels: Vec<&str> = triples
        .iter()
        .filter(|t| t.p == RDF_TYPE && matches!(&t.o, Object::Iri(c) if *c == kernel_class))
        .map(|t| t.s.as_str())
        .collect();
    kernels.sort_unstable();
    kernels.dedup();
    let kernel = match kernels.as_slice() {
        [k] => *k,
        [] => return Err(ill("(kernel)", "no prayer-kernel:Kernel node declared")),
        _ => {
            return Err(ill(
                "(kernel)",
                format!("{} prayer-kernel:Kernel nodes declared; exactly 1 required", kernels.len()),
            ))
        }
    };

    // The kernel's clause list (IRIs).
    let clause_pred = format!("{KERNEL_NS}clause");
    let mut listed: Vec<String> = Vec::new();
    for t in triples.iter().filter(|t| t.s == kernel && t.p == clause_pred) {
        match &t.o {
            Object::Iri(iri) => listed.push(iri.clone()),
            _ => return Err(ill(kernel, "prayer-kernel:clause must be an IRI")),
        }
    }
    listed.sort_unstable();
    if listed.windows(2).any(|w| w[0] == w[1]) {
        return Err(ill(kernel, "duplicate prayer-kernel:clause entry"));
    }

    // Typed Clause subjects must equal the listed set.
    let clause_class = format!("{KERNEL_NS}Clause");
    let mut typed: Vec<&str> = triples
        .iter()
        .filter(|t| t.p == RDF_TYPE && matches!(&t.o, Object::Iri(c) if *c == clause_class))
        .map(|t| t.s.as_str())
        .collect();
    typed.sort_unstable();
    typed.dedup();
    for iri in &listed {
        if !typed.contains(&iri.as_str()) {
            return Err(ill(iri, "listed by the kernel but not typed prayer-kernel:Clause"));
        }
    }
    for iri in &typed {
        if !listed.iter().any(|l| l == iri) {
            return Err(ill(iri, "typed prayer-kernel:Clause but not listed by the kernel"));
        }
    }

    // Cross-typing is refused by subject identity, not by name heuristic:
    // no `prayer-kernel:Clause` node may ALSO be typed as an executable
    // class (`wf:Workflow`, `wf:Capability`, `hook:Hook`). The module's own
    // doctrine is that God/the clause boundary is a STRING property, never
    // an executable node — this makes that doctrine a closed-world law
    // instead of a convention that only holds if nobody double-types the
    // node. Renaming the clause, or the IRI it lives on, cannot bypass this:
    // it is keyed on the same subject appearing in `typed` (every
    // canonical clause), not on what the IRI or `pk:name` happen to spell.
    let executable_classes = [
        format!("{}Workflow", crate::graph::WF_NS),
        format!("{}Capability", crate::graph::WF_NS),
        format!("{}Hook", crate::hooks::HOOK_NS),
    ];
    for t in triples {
        if t.p == RDF_TYPE {
            if let Object::Iri(class) = &t.o {
                if executable_classes.contains(class) && typed.contains(&t.s.as_str()) {
                    return Err(ill(
                        &t.s,
                        format!(
                            "prayer-kernel:Clause node is also typed executable '{class}' — \
                             the clause boundary must remain a string property, never an \
                             executable node"
                        ),
                    ));
                }
            }
        }
    }

    // Extract each clause.
    let mut clauses = Vec::with_capacity(listed.len());
    for subject in &listed {
        let props = Props {
            props: triples
                .iter()
                .filter(|t| t.s == *subject)
                .map(|t| (t.p.as_str(), &t.o))
                .collect(),
        };
        let name = props.one_str(subject, "name")?;
        if !CANONICAL_CLAUSES.contains(&name.as_str()) {
            return Err(ill(subject, format!("unknown clause name '{name}'")));
        }
        let problem_class = props.one_str(subject, "problemClass")?;
        let boundary = props.one_str(subject, "boundary")?;
        if !BOUNDARIES.contains(&boundary.as_str()) {
            return Err(ill(
                subject,
                format!("boundary '{boundary}' not in {BOUNDARIES:?}"),
            ));
        }
        let action = props.opt_iri(subject, "action")?;
        clauses.push(PrayerClause {
            iri: subject.clone(),
            name,
            problem_class,
            boundary,
            action,
        });
    }

    // Exact canonical coverage: all 11, no duplicates, nothing extra.
    for canonical in CANONICAL_CLAUSES {
        let hits = clauses.iter().filter(|c| c.name == canonical).count();
        match hits {
            1 => {}
            0 => return Err(ill("(kernel)", format!("missing clause '{canonical}'"))),
            _ => return Err(ill("(kernel)", format!("duplicate clause '{canonical}'"))),
        }
    }
    if clauses.len() != CANONICAL_CLAUSES.len() {
        return Err(ill(
            "(kernel)",
            format!("{} clauses declared; exactly {} required", clauses.len(), CANONICAL_CLAUSES.len()),
        ));
    }

    // Canonical (scriptural) order, independent of graph surface order.
    clauses.sort_by_key(|c| {
        CANONICAL_CLAUSES.iter().position(|n| *n == c.name).unwrap_or(usize::MAX)
    });
    Ok(clauses)
}

/// Whether the graph declares any `prayer-kernel:` vocabulary at all — the
/// gate that makes the surrender boundary a conditional law: no kernel, no
/// law to enforce.
#[must_use]
pub fn kernel_declared(triples: &[Triple]) -> bool {
    triples.iter().any(|t| {
        t.p.starts_with(KERNEL_NS)
            || (t.p == RDF_TYPE
                && matches!(&t.o, Object::Iri(c) if c.starts_with(KERNEL_NS)))
    })
}

/// Watched predicate of a hook condition, if it has one.
fn extract_datalog_predicates(program: &str) -> Vec<String> {
    let mut preds = Vec::new();
    let bytes = program.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        if bytes[i] == b't' {
            let prev_ok = if i > 0 {
                let prev = bytes[i - 1];
                !prev.is_ascii_alphanumeric() && prev != b'_' && prev != b':' && prev != b'?'
            } else {
                true
            };
            if prev_ok {
                let mut j = i + 1;
                while j < n && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < n && bytes[j] == b'(' {
                    let mut depth = 1;
                    let mut k = j + 1;
                    let mut content = String::new();
                    while k < n && depth > 0 {
                        let c = bytes[k] as char;
                        if c == '(' {
                            depth += 1;
                        } else if c == ')' {
                            depth -= 1;
                        }
                        if depth > 0 {
                            content.push(c);
                        }
                        k += 1;
                    }
                    let mut args = Vec::new();
                    let mut current_arg = String::new();
                    let mut arg_depth = 0usize;
                    for ac in content.chars() {
                        match ac {
                            '(' => {
                                arg_depth += 1;
                                current_arg.push(ac);
                            }
                            ')' => {
                                arg_depth = arg_depth.saturating_sub(1);
                                current_arg.push(ac);
                            }
                            ',' if arg_depth == 0 => {
                                args.push(current_arg.trim().to_string());
                                current_arg.clear();
                            }
                            _ => {
                                current_arg.push(ac);
                            }
                        }
                    }
                    args.push(current_arg.trim().to_string());
                    if args.len() >= 2 {
                        let pred_arg = &args[1];
                        if pred_arg.starts_with('<') && pred_arg.ends_with('>') {
                            let pred = &pred_arg[1..pred_arg.len() - 1];
                            preds.push(pred.to_string());
                        }
                    }
                    i = k;
                    continue;
                }
            }
        }
        i += 1;
    }
    preds
}

/// Watched predicates of a hook condition.
fn watched_vars(hook: &crate::hooks::KnowledgeHook) -> Vec<String> {
    use crate::hooks::HookCondition as C;
    match &hook.condition {
        C::Delta { var }
        | C::Threshold { var, .. }
        | C::Count { var, .. }
        | C::Window { var, .. } => vec![var.clone()],
        C::Datalog { program, .. } => extract_datalog_predicates(program),
    }
}

/// Enforce the surrender invariant as a RUNTIME law over an admitted graph:
/// if the graph declares a prayer kernel, then (a) every clause whose
/// boundary is `god-receives-unbounded` and whose `pk:action` names a hook
/// must name a hook whose effect is `refuse` — surrender is never re-routed
/// to computation; and (b) every hook watching a predicate that a
/// surrendered clause's refuse-hook watches must itself refuse — the
/// surrendered predicate cannot be siphoned into a ground-action by a
/// second hook. Graphs without a kernel are untouched. Every violation is
/// a typed [`Refusal::BoundaryViolation`] naming the culprit.
pub fn enforce_surrender_boundary(
    triples: &[Triple],
    hooks: &[crate::hooks::KnowledgeHook],
) -> Result<(), Refusal> {
    if !kernel_declared(triples) {
        return Ok(());
    }
    let clauses = extract_kernel(triples)?;
    let mut surrendered_vars: Vec<(String, String)> = Vec::new(); // (var, clause iri)
    for clause in clauses.iter().filter(|c| c.boundary == "god-receives-unbounded") {
        let Some(action) = clause.action.as_deref() else {
            return Err(Refusal::BoundaryViolation {
                subject: clause.iri.clone(),
                detail: format!(
                    "god-receives-unbounded clause <{}> is missing pk:action",
                    clause.iri
                ),
            });
        };
        let Some(hook) = hooks.iter().find(|h| h.iri == action) else {
            return Err(Refusal::BoundaryViolation {
                subject: clause.iri.clone(),
                detail: format!(
                    "god-receives-unbounded clause action <{action}> does not resolve \
                     to a declared hook"
                ),
            });
        };
        if hook.effect != crate::hooks::EffectKind::Refuse {
            return Err(Refusal::BoundaryViolation {
                subject: clause.iri.clone(),
                detail: format!(
                    "god-receives-unbounded clause action hook <{action}> must have \
                     effect 'refuse'; the unbounded is surrendered, never computed"
                ),
            });
        }
        if matches!(hook.condition, crate::hooks::HookCondition::Datalog { .. }) {
            return Err(Refusal::BoundaryViolation {
                subject: hook.iri.clone(),
                detail: "god-receives-unbounded hook cannot use a Datalog condition".to_string(),
            });
        }
        for var in watched_vars(hook) {
            surrendered_vars.push((var, clause.iri.clone()));
        }
    }
    for hook in hooks.iter().filter(|h| h.effect != crate::hooks::EffectKind::Refuse) {
        for var in watched_vars(hook) {
            if let Some((v, clause)) = surrendered_vars.iter().find(|(v, _)| *v == var) {
                return Err(Refusal::BoundaryViolation {
                    subject: hook.iri.clone(),
                    detail: format!(
                        "non-refusing hook watches surrendered predicate <{v}> of \
                         god-receives-unbounded clause <{clause}>"
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Content address of the kernel: byte-sorted canonical lines
/// `name\tproblem_class\tboundary\taction` — COMPUTED, stable under any
/// surface reordering of the source document.
#[must_use]
pub fn kernel_hash(clauses: &[PrayerClause]) -> String {
    let mut lines: Vec<String> = clauses
        .iter()
        .map(|c| {
            format!(
                "{}\t{}\t{}\t{}",
                c.name,
                c.problem_class,
                c.boundary,
                c.action.as_deref().unwrap_or("")
            )
        })
        .collect();
    lines.sort_unstable();
    content_address(lines.join("\n").as_bytes())
}
