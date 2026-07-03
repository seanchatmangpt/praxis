//! Graph-declared handler binding + the delegability lattice.
//!
//! The archaeology's root blocker: the workflow runner was hardcoded and
//! the graph had no way to say who executes what. Here the binding is
//! graph-declared (`wf:handler` on a `wf:Capability` node, exact-IRI) and
//! judged against a CLOSED [`HandlerRegistry`] — an unknown IRI is a typed
//! [`Refusal::UnknownHandler`] BEFORE any solving; string-convention
//! binding is forbidden by construction (exact-key lookup only).
//!
//! The delegability lattice is authored fresh (it existed nowhere in the
//! constellation): `human-only < assistive < automatable < verifiable`.
//! An automated runner may execute a capability only at `automatable` or
//! above; below that the action parks for the human — a typed
//! [`Refusal::DelegabilityViolation`], receipted, never silent.
//!
//! Template lineage: five-layer-agents `schema/agents.ttl`
//! (ag:agentTools / ag:canSpawnType — graph-declared assignment consumed
//! mechanically); a2a-rs `a2a:hasCapability`; open-ontologies
//! `port:cellAgent`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use chatman_common::provenance::content_address;

use crate::graph::{Object, Triple, WF_NS};
use crate::Refusal;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// The handler IRI namespace praxis registers built-ins under.
pub const HANDLER_NS: &str = "http://seanchatmangpt.github.io/praxis/handler#";

/// The delegability lattice, ordered: who may act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Delegability {
    /// Only the human acts (forgiveness, surrender). Agents may support,
    /// never execute.
    HumanOnly,
    /// The human acts with agent assistance; execution still parks for the
    /// human.
    Assistive,
    /// An automated handler may execute.
    Automatable,
    /// An automated handler may execute AND the result is independently
    /// verifiable by replay.
    Verifiable,
}

impl Delegability {
    fn parse(s: &str, subject: &str) -> Result<Self, Refusal> {
        Ok(match s {
            "human-only" => Self::HumanOnly,
            "assistive" => Self::Assistive,
            "automatable" => Self::Automatable,
            "verifiable" => Self::Verifiable,
            other => {
                return Err(Refusal::WorkflowIllFormed {
                    subject: subject.to_string(),
                    detail: format!(
                        "wf:delegability '{other}' not in \
                         human-only|assistive|automatable|verifiable"
                    ),
                })
            }
        })
    }

    /// Render for canonical binding lines.
    #[must_use]
    pub fn render(self) -> &'static str {
        match self {
            Self::HumanOnly => "human-only",
            Self::Assistive => "assistive",
            Self::Automatable => "automatable",
            Self::Verifiable => "verifiable",
        }
    }
}

/// One graph-declared binding: capability name -> handler IRI + lattice grade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandlerBinding {
    /// The capability's `wf:name`.
    pub capability: String,
    /// The declared handler IRI (exact key into the registry).
    pub handler: String,
    /// The declared delegability grade.
    pub delegability: Delegability,
}

/// Extract handler bindings from admitted triples: every `wf:Capability`
/// node carrying `wf:handler` MUST also carry `wf:delegability` (explicit
/// or nothing — no default grade). Capabilities without `wf:handler` are
/// legacy-lawful (the default deterministic runner applies).
pub fn extract_bindings(triples: &[Triple]) -> Result<Vec<HandlerBinding>, Refusal> {
    let cap_class = format!("{WF_NS}Capability");
    let name_p = format!("{WF_NS}name");
    let handler_p = format!("{WF_NS}handler");
    let deleg_p = format!("{WF_NS}delegability");
    let ill = |subject: &str, detail: String| Refusal::WorkflowIllFormed {
        subject: subject.to_string(),
        detail,
    };

    let mut caps: Vec<&str> = triples
        .iter()
        .filter(|t| t.p == RDF_TYPE && matches!(&t.o, Object::Iri(c) if *c == cap_class))
        .map(|t| t.s.as_str())
        .collect();
    caps.sort_unstable();
    caps.dedup();

    let mut bindings = Vec::new();
    for cap in caps {
        let one = |pred: &str| -> Vec<&Object> {
            triples.iter().filter(|t| t.s == cap && t.p == *pred).map(|t| &t.o).collect()
        };
        let handlers = one(&handler_p);
        if handlers.is_empty() {
            continue;
        }
        let handler = match handlers.as_slice() {
            [Object::Iri(iri)] => iri.clone(),
            [_] => return Err(ill(cap, "wf:handler must be an IRI".to_string())),
            _ => return Err(ill(cap, "multiple wf:handler".to_string())),
        };
        let delegability = match one(&deleg_p).as_slice() {
            [Object::Str(s)] => Delegability::parse(s, cap)?,
            [] => {
                return Err(ill(
                    cap,
                    "wf:handler without wf:delegability — the grade is explicit or \
                     the binding is refused (no default)"
                        .to_string(),
                ))
            }
            [_] => return Err(ill(cap, "wf:delegability must be a string literal".to_string())),
            _ => return Err(ill(cap, "multiple wf:delegability".to_string())),
        };
        let capability = match one(&name_p).as_slice() {
            [Object::Str(s)] => s.clone(),
            _ => return Err(ill(cap, "handled capability missing unique wf:name".to_string())),
        };
        bindings.push(HandlerBinding { capability, handler, delegability });
    }
    bindings.sort_unstable_by(|a, b| a.capability.cmp(&b.capability));
    Ok(bindings)
}

/// Canonical binding form: sorted `capability\thandler\tdelegability`
/// lines, trailing newline. `handler_hash` is its content address.
#[must_use]
pub fn binding_canonical_form(bindings: &[HandlerBinding]) -> String {
    let mut out = String::new();
    for b in bindings {
        out.push_str(&format!("{}\t{}\t{}\n", b.capability, b.handler, b.delegability.render()));
    }
    out
}

/// Content address of the canonical binding form.
#[must_use]
pub fn handler_hash(bindings: &[HandlerBinding]) -> String {
    content_address(binding_canonical_form(bindings).as_bytes())
}

/// The closed handler table. Exact-key lookup ONLY — prefix, suffix, and
/// convention matching are unrepresentable.
#[derive(Debug, Clone, Default)]
pub struct HandlerRegistry {
    known: BTreeMap<String, ()>,
}

impl HandlerRegistry {
    /// The built-in registry: exactly the deterministic v1 handler.
    #[must_use]
    pub fn builtin() -> Self {
        let mut known = BTreeMap::new();
        known.insert(format!("{HANDLER_NS}deterministic-v1"), ());
        Self { known }
    }

    /// Exact-key membership.
    #[must_use]
    pub fn contains(&self, iri: &str) -> bool {
        self.known.contains_key(iri)
    }

    /// Judge every binding BEFORE solving: an unknown handler IRI is a
    /// typed refusal naming the known table; a grade below `automatable`
    /// is a delegability violation (the action parks for the human).
    ///
    /// This is the UNSCOPED judgment: delegability is checked for every
    /// binding regardless of use. The firing pipeline instead splits it
    /// into [`Self::judge_known`] (global, pre-solve) and
    /// [`Self::judge_delegability`] (scoped to the capabilities a fired
    /// action actually uses).
    pub fn judge(&self, bindings: &[HandlerBinding]) -> Result<(), Refusal> {
        self.judge_known(bindings)?;
        let all: BTreeSet<String> = bindings.iter().map(|b| b.capability.clone()).collect();
        self.judge_delegability(bindings, &all)
    }

    /// Judge handler EXISTENCE only, for every binding in the graph — this
    /// check is global and runs BEFORE any solving: an unknown handler IRI
    /// anywhere in the admitted graph refuses the whole firing.
    pub fn judge_known(&self, bindings: &[HandlerBinding]) -> Result<(), Refusal> {
        for b in bindings {
            if !self.contains(&b.handler) {
                return Err(Refusal::UnknownHandler {
                    capability: b.capability.clone(),
                    handler: b.handler.clone(),
                    known: self.known.keys().cloned().collect(),
                });
            }
        }
        Ok(())
    }

    /// Judge delegability ONLY for bindings whose capability is in `used`
    /// — the capability names a fired action's derived plan actually
    /// executes. A `human-only` binding on a capability no fired action
    /// touches must not refuse the firing; one the plan would execute is a
    /// typed [`Refusal::DelegabilityViolation`] (the action parks for the
    /// human).
    pub fn judge_delegability(
        &self,
        bindings: &[HandlerBinding],
        used: &BTreeSet<String>,
    ) -> Result<(), Refusal> {
        for b in bindings {
            if !used.contains(&b.capability) {
                continue;
            }
            if b.delegability < Delegability::Automatable {
                return Err(Refusal::DelegabilityViolation {
                    capability: b.capability.clone(),
                    required: "automatable".to_string(),
                    declared: b.delegability.render().to_string(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::parse_ttl;

    fn doc(cap_extra: &str) -> String {
        format!(
            "@prefix wf: <{WF_NS}> .\n@prefix ex: <http://e/> .\n\
             ex:c a wf:Capability ; wf:name \"c\" ; wf:params 0 ; wf:cost 1 {cap_extra}.\n"
        )
    }

    #[test]
    fn binding_extracts_and_hashes_canonically() {
        let ttl = doc(&format!(
            "; wf:handler <{HANDLER_NS}deterministic-v1> ; wf:delegability \"verifiable\" "
        ));
        let bindings = extract_bindings(&parse_ttl(&ttl).unwrap()).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].delegability, Delegability::Verifiable);
        assert_eq!(
            binding_canonical_form(&bindings),
            format!("c\t{HANDLER_NS}deterministic-v1\tverifiable\n")
        );
        HandlerRegistry::builtin().judge(&bindings).expect("known + automatable-or-above");
    }

    #[test]
    fn unknown_handler_refused_exact_key_only() {
        // A SUFFIX of a registered IRI is unknown: exact-key proof.
        let ttl = doc(&format!(
            "; wf:handler <{HANDLER_NS}deterministic> ; wf:delegability \"verifiable\" "
        ));
        let bindings = extract_bindings(&parse_ttl(&ttl).unwrap()).unwrap();
        match HandlerRegistry::builtin().judge(&bindings) {
            Err(Refusal::UnknownHandler { handler, known, .. }) => {
                assert!(handler.ends_with("deterministic"));
                assert_eq!(known, vec![format!("{HANDLER_NS}deterministic-v1")]);
            }
            other => panic!("expected UnknownHandler, got {other:?}"),
        }
    }

    #[test]
    fn human_only_is_a_delegability_violation_for_automated_runners() {
        let ttl = doc(&format!(
            "; wf:handler <{HANDLER_NS}deterministic-v1> ; wf:delegability \"human-only\" "
        ));
        let bindings = extract_bindings(&parse_ttl(&ttl).unwrap()).unwrap();
        match HandlerRegistry::builtin().judge(&bindings) {
            Err(Refusal::DelegabilityViolation { declared, .. }) => {
                assert_eq!(declared, "human-only");
            }
            other => panic!("expected DelegabilityViolation, got {other:?}"),
        }
    }

    #[test]
    fn handler_without_delegability_is_ill_formed() {
        let ttl = doc(&format!("; wf:handler <{HANDLER_NS}deterministic-v1> "));
        match extract_bindings(&parse_ttl(&ttl).unwrap()) {
            Err(Refusal::WorkflowIllFormed { detail, .. }) => {
                assert!(detail.contains("no default"));
            }
            other => panic!("expected WorkflowIllFormed, got {other:?}"),
        }
    }

    #[test]
    fn two_declarations_two_binding_hashes() {
        let a = doc(&format!(
            "; wf:handler <{HANDLER_NS}deterministic-v1> ; wf:delegability \"verifiable\" "
        ));
        let b = doc(&format!(
            "; wf:handler <{HANDLER_NS}deterministic-v1> ; wf:delegability \"automatable\" "
        ));
        let ha = handler_hash(&extract_bindings(&parse_ttl(&a).unwrap()).unwrap());
        let hb = handler_hash(&extract_bindings(&parse_ttl(&b).unwrap()).unwrap());
        assert_ne!(ha, hb, "the graph decides the binding; the hash proves which graph");
    }
}
