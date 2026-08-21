//! Real role-inference layer (Mycin production rules + praxis-graphlaw
//! Datalog materialization), gated behind the `role-inference` feature
//! (widened from its original bench-only scope — see `Cargo.toml`). Moved
//! here, verbatim in logic, from `bench::roles` so the SAME engine is
//! reachable both from bench fixture call sites (`bench/roles.rs`
//! re-exports everything below) and from the live, non-bench plan-admit
//! path (`plan_approval.rs::derive_roster_roles`).
//!
//! ## Disclosed generalization boundary
//!
//! `derive_roles_datalog` needs `RosterWorker` facts (worker id, declared
//! role, department) — a concept that does not exist anywhere in the live
//! plan-admit path's native data model (`pipeline::ImportedArtifact` /
//! `Pddl8Tape`, which carries PDDL domain/problem text and ground plan ops,
//! never a personnel roster). Arbitrary imported PDDL planning artifacts
//! that carry no roster facts genuinely have nothing for this layer to
//! derive roles over; `plan_approval::derive_roster_roles` returns `Ok(None)`
//! for such artifacts, honestly, rather than fabricating role facts. What
//! IS generalized here relative to bench: the roster fact SOURCE. Bench
//! sources roster facts only from its own `ObsWriter`-emitted
//! `roster_admitted` observations, built exclusively by bench fixture
//! generators (`workday.rs`/`soc2.rs`). The live path
//! (`pipeline::import_roster`) sources the identical `RosterWorker` shape
//! from a plain, independent Turtle vocabulary
//! (`ceng:rosterDeclaredRole`/`ceng:rosterDepartment`) any imported `.ttl`
//! artifact may carry — not bench-fixture-shaped, real Datalog derivation
//! over it, proven in `plan_approval_test.rs` against a real non-bench
//! artifact.

use std::collections::BTreeMap;

use crate::powl::CngRefusal;
use wasm4pm_cognition::breeds::production_rules::Mycin;
use wasm4pm_cognition::breeds::{BreedInput, CognitionBreed, Fact, Rule};

/// Knowledge base for the Mycin production-rule breed: category → standing
/// role → lawful next action. The facts come from the admitted graph; the
/// derivation is real forward chaining with certainty factors
/// (wasm4pm-cognition, Shortliffe-Buchanan), not a Rust match table.
pub fn role_rules() -> Vec<Rule> {
    let role_of = [
        ("email-routing", "coordinator"),
        ("calendar-change", "coordinator"),
        ("invoice-matching", "reviewer"),
        ("purchase-order-approval", "approver"),
        ("expense-review", "reviewer"),
        ("hr-notice", "operator"),
        ("customer-request", "operator"),
        ("logistics-event", "operator"),
        ("compliance-check", "auditor"),
        ("document-request", "coordinator"),
        ("software-delivery", "operator"),
        ("admission-request", "approver"),
        ("interruption", "coordinator"),
        ("planning", "coordinator"),
        ("api-orchestration", "operator"),
        ("soc2-audit", "auditor"),
    ];
    let action_of = [
        ("coordinator", "route-and-schedule"),
        ("reviewer", "review-then-escalate-to-approver"),
        ("approver", "authorize-transition"),
        ("operator", "execute-standard-procedure"),
        ("auditor", "verify-evidence-chain"),
    ];
    let mut rules = Vec::new();
    for (cat, role) in role_of {
        rules.push(Rule {
            id: format!("r-role-{cat}"),
            premise: vec![format!("category={cat}")],
            conclusion: format!("role={role}"),
            certainty: 0.95,
        });
    }
    for (role, action) in action_of {
        rules.push(Rule {
            id: format!("r-act-{role}"),
            premise: vec![format!("role={role}")],
            conclusion: format!("next={action}"),
            certainty: 0.9,
        });
    }
    rules
}

/// Old-AI role inference: derive the standing role and lawful next action
/// for a classified artifact via the Mycin forward-chaining breed. Returns
/// the terminal conclusion (`next=<action>`), or None when no lawful action
/// is derivable — callers must refuse, never fall back silently.
pub fn infer_lawful_next_action(category: &str) -> Option<String> {
    let input = BreedInput {
        intent: "derive standing role and lawful next action".to_string(),
        facts: vec![Fact {
            key: "category".to_string(),
            value: category.to_string(),
        }],
        rules: role_rules(),
        ..Default::default()
    };
    Mycin.run(&input).ok().and_then(|out| out.selected)
}

/// SOC2 audit-engagement standing roles: responsibility → standing role →
/// lawful next action. See `bench::roles`'s prior doc (unchanged logic,
/// moved verbatim).
pub fn soc2_role_rules() -> Vec<Rule> {
    let role_of = [
        (
            "control-design-and-evidence",
            "control-owner",
            "document-control-design-and-attach-evidence",
        ),
        (
            "readiness-and-oe-testing",
            "internal-audit-lead",
            "execute-readiness-assessment-and-oe-testing",
        ),
        (
            "scoping-and-bundle-coordination",
            "compliance-program-manager",
            "coordinate-scope-and-assemble-evidence-bundle",
        ),
        (
            "exception-remediation",
            "remediation-engineer",
            "implement-remediation-for-identified-exception",
        ),
        (
            "evidence-chain-of-custody",
            "evidence-custodian",
            "maintain-evidence-chain-of-custody",
        ),
    ];
    let mut rules = Vec::new();
    for (responsibility, role, _action) in role_of {
        rules.push(Rule {
            id: format!("r-soc2-role-{role}"),
            premise: vec![format!("responsibility={responsibility}")],
            conclusion: format!("role={role}"),
            certainty: 0.95,
        });
    }
    for (_responsibility, role, action) in role_of {
        rules.push(Rule {
            id: format!("r-soc2-act-{role}"),
            premise: vec![format!("role={role}")],
            conclusion: format!("next={action}"),
            certainty: 0.9,
        });
    }
    rules
}

/// Old-AI standing-role inference for the SOC2 audit engagement (moved
/// verbatim from `bench::roles`).
pub fn infer_soc2_standing_role(responsibility: &str) -> Option<String> {
    let input = BreedInput {
        intent: "derive SOC2 standing role and lawful next action".to_string(),
        facts: vec![Fact {
            key: "responsibility".to_string(),
            value: responsibility.to_string(),
        }],
        rules: soc2_role_rules(),
        ..Default::default()
    };
    Mycin.run(&input).ok().and_then(|out| out.selected)
}

/// One roster worker: id, declared role, department — the fact shape
/// `derive_roles_datalog` requires. Populated from bench observation
/// scans (`bench::roles::roster_workers`) or, on the live path, from
/// plain Turtle roster triples (`pipeline::import_roster`).
pub struct RosterWorker {
    pub worker_id: String,
    pub role: String,
    pub department: String,
}

/// Result of the Datalog role derivation layer.
pub struct DatalogRoles {
    /// worker id → Datalog-derived role.
    pub derived: BTreeMap<String, String>,
    /// worker id → Datalog-derived `:obligation` atom.
    pub obligations: BTreeMap<String, String>,
    /// Total derived facts (roles + obligations + custody + closure).
    pub derived_facts: usize,
}

/// Runs the praxis-graphlaw Datalog engine over the roster fact base with
/// the rules in `rules_text` (`crates/cng/rules/bench-roles.dl` — a
/// generic identity + per-role-obligation rule set, not bench-specific
/// despite the filename: it needs only `:declaredRole`/`:department`
/// facts, which either bench observations or live-path roster triples can
/// supply), deriving role/obligation/custody/closure facts. A worker whose
/// Datalog-derived role differs from the roster-declared role is a typed
/// refusal.
///
/// # Complexity
/// O(workers) facts; semi-naive materialization over the rule set.
pub fn derive_roles_datalog(
    workers: &[RosterWorker],
    rules_text: &str,
) -> Result<DatalogRoles, CngRefusal> {
    use praxis_graphlaw::parser::Parser;
    use praxis_graphlaw::TripleStore;

    let mut doc = String::with_capacity(workers.len() * 64 + rules_text.len());
    for w in workers {
        doc.push_str(&format!(":{} :declaredRole :{}.\n", w.worker_id, w.role));
        doc.push_str(&format!(
            ":{} :department :{}.\n",
            w.worker_id, w.department
        ));
    }
    for line in rules_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        doc.push_str(trimmed);
        doc.push('\n');
    }

    let (facts, rules) = Parser::parse(doc);
    if rules.is_empty() {
        return Err(CngRefusal::UnsupportedConstruct(
            "bench-roles.dl yielded zero parsed Datalog rules".to_string(),
        ));
    }
    let mut store = TripleStore::new();
    for fact in facts {
        store.add(fact);
    }
    store.add_rules(rules).map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!("Datalog rule validation refused: {e}"))
    })?;
    let inferred = store.materialize().map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!("Datalog materialization refused: {e}"))
    })?;

    let decode = |encoded: usize| -> Result<String, CngRefusal> {
        praxis_graphlaw::encoding::Encoder::decode(&encoded)
            .ok_or_else(|| CngRefusal::MalformedTtl("Datalog term failed to decode".to_string()))
    };
    let mut derived: BTreeMap<String, String> = BTreeMap::new();
    let mut obligations: BTreeMap<String, String> = BTreeMap::new();
    for triple in &inferred {
        let predicate = decode(triple.p.to_encoded())?;
        if predicate == ":derivedRole" {
            let worker = decode(triple.s.to_encoded())?;
            let role = decode(triple.o.to_encoded())?;
            derived.insert(
                worker.trim_start_matches(':').to_string(),
                role.trim_start_matches(':').to_string(),
            );
        } else if predicate == ":obligation" {
            let worker = decode(triple.s.to_encoded())?;
            let obligation = decode(triple.o.to_encoded())?;
            obligations.insert(
                worker.trim_start_matches(':').to_string(),
                obligation.trim_start_matches(':').to_string(),
            );
        }
    }
    for w in workers {
        match derived.get(&w.worker_id) {
            Some(role) if role == &w.role => {}
            Some(role) => {
                return Err(CngRefusal::HardcodingSuspicion(format!(
                    "Datalog-derived role {role} for worker {} contradicts the \
                     roster-declared role {}; the roster graph is the admitted input",
                    w.worker_id, w.role
                )));
            }
            None => {
                return Err(CngRefusal::HardcodingSuspicion(format!(
                    "Datalog derived no role for roster worker {}; derivation must \
                     cover every admitted worker",
                    w.worker_id
                )));
            }
        }
    }
    Ok(DatalogRoles {
        derived,
        obligations,
        derived_facts: inferred.len(),
    })
}
