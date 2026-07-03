//! Graph-declared agent metadata — tool sets and spawn-eligibility edges.
//!
//! `handlers.rs` already binds a capability to an executing handler IRI and
//! judges it against the delegability lattice, but it carries no notion of
//! *what an agent brings* (a declared tool set) or *who it may spawn*
//! (a delegation edge bounded by layer depth). Those two capabilities exist
//! in five-layer-agents' `schema/agents.ttl` as `ag:agentTools` and
//! `ag:canSpawnType`, consumed there by `spawn_tree.sparql` /
//! `tests/spawn_tree_test.rs` to enforce "no agent at depth 5 has
//! `canSpawnType`" — a terminal-by-absence law, not an explicit marker.
//!
//! This module ports that vocabulary CONCEPT-ONLY under a fresh closed
//! namespace (`agent:`), mirroring the `hooks.rs`/`kernel.rs`
//! closed-world-extractor style exactly: unknown `agent:` predicates or
//! classes are refused by name, never silently ignored. [`extract_agents`]
//! and [`spawn_depth_law`] are wired into `firing.rs::fire_hooks` as a
//! pre-solve, global stage (named `agent-spawn-depth`), alongside handler
//! existence judgment — see [`crate::firing::HookFiringReceipt::agents`].
//! `handlers.rs`' own binding/delegability judgment is unaffected: the two
//! vocabularies remain additive.

use serde::{Deserialize, Serialize};

use chatman_common::provenance::content_address;

use crate::graph::{Object, Triple};
use crate::Refusal;

/// The agent vocabulary namespace. Closed world, same law as `wf:`/`hook:`.
pub const AGENT_NS: &str = "http://seanchatmangpt.github.io/praxis/agent#";

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

const AGENT_CLASSES: [&str; 1] = ["Agent"];
const AGENT_PREDICATES: [&str; 3] = ["tool", "canSpawn", "layerDepth"];

/// Max declared tools per agent — the 8-bound.
pub const MAX_TOOLS: usize = 8;
/// Max declared spawn-eligible agent types per agent — the 8-bound.
pub const MAX_CAN_SPAWN: usize = 8;
/// Max agents per registry — the 8-bound.
pub const MAX_AGENTS: usize = 8;

fn ill(subject: &str, detail: impl Into<String>) -> Refusal {
    Refusal::AgentIllFormed { subject: subject.to_string(), detail: detail.into() }
}

/// One graph-declared agent profile: an IRI, its bounded tool set, its
/// bounded spawn-eligibility edges, and its layer depth (1..=5, mirroring
/// five-layer-agents' five-layer hierarchy).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfile {
    /// The agent node's IRI.
    pub iri: String,
    /// Declared tool names (`agent:tool`), sorted, deduplicated, `<= 8`.
    pub tools: Vec<String>,
    /// Declared spawn-eligible agent-type IRIs (`agent:canSpawn`), sorted,
    /// deduplicated, `<= 8`.
    pub can_spawn: Vec<String>,
    /// Declared layer depth (`agent:layerDepth`), `1..=5`.
    pub layer_depth: u8,
}

/// Extract the agent registry from admitted triples. Closed-world over the
/// `agent:` namespace: unknown predicates/classes and shape violations are
/// [`Refusal::AgentIllFormed`]. At most [`MAX_AGENTS`] agents; at most
/// [`MAX_TOOLS`] tools and [`MAX_CAN_SPAWN`] spawn edges per agent.
pub fn extract_agents(triples: &[Triple]) -> Result<Vec<AgentProfile>, Refusal> {
    // Closed-world vocabulary sweep.
    for t in triples {
        if let Some(local) = t.p.strip_prefix(AGENT_NS) {
            if !AGENT_PREDICATES.contains(&local) {
                return Err(ill(&t.s, format!("unknown agent: predicate '{local}'")));
            }
        }
        if t.p == RDF_TYPE {
            if let Object::Iri(class) = &t.o {
                if let Some(local) = class.strip_prefix(AGENT_NS) {
                    if !AGENT_CLASSES.contains(&local) {
                        return Err(ill(&t.s, format!("unknown agent: class '{local}'")));
                    }
                }
            }
        }
    }

    let agent_class = format!("{AGENT_NS}Agent");
    let mut subjects: Vec<&str> = triples
        .iter()
        .filter(|t| t.p == RDF_TYPE && matches!(&t.o, Object::Iri(c) if *c == agent_class))
        .map(|t| t.s.as_str())
        .collect();
    subjects.sort_unstable();
    subjects.dedup();
    if subjects.len() > MAX_AGENTS {
        return Err(ill(
            "(registry)",
            format!("{} agents declared; max {MAX_AGENTS}", subjects.len()),
        ));
    }

    let tool_p = format!("{AGENT_NS}tool");
    let spawn_p = format!("{AGENT_NS}canSpawn");
    let depth_p = format!("{AGENT_NS}layerDepth");

    let mut agents = Vec::with_capacity(subjects.len());
    for subject in subjects {
        let mut tools: Vec<String> = triples
            .iter()
            .filter(|t| t.s == subject && t.p == tool_p)
            .map(|t| match &t.o {
                Object::Str(s) => Ok(s.clone()),
                _ => Err(ill(subject, "agent:tool must be a string literal")),
            })
            .collect::<Result<_, _>>()?;
        tools.sort_unstable();
        tools.dedup();
        if tools.len() > MAX_TOOLS {
            return Err(ill(subject, format!("{} agent:tool values; max {MAX_TOOLS}", tools.len())));
        }

        let mut can_spawn: Vec<String> = triples
            .iter()
            .filter(|t| t.s == subject && t.p == spawn_p)
            .map(|t| match &t.o {
                Object::Iri(iri) => Ok(iri.clone()),
                _ => Err(ill(subject, "agent:canSpawn must be an IRI")),
            })
            .collect::<Result<_, _>>()?;
        can_spawn.sort_unstable();
        can_spawn.dedup();
        if can_spawn.len() > MAX_CAN_SPAWN {
            return Err(ill(
                subject,
                format!("{} agent:canSpawn values; max {MAX_CAN_SPAWN}", can_spawn.len()),
            ));
        }

        let depths: Vec<&Object> = triples
            .iter()
            .filter(|t| t.s == subject && t.p == depth_p)
            .map(|t| &t.o)
            .collect();
        let layer_depth = match depths.as_slice() {
            [Object::Int(v)] if (1..=5).contains(v) =>
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                (*v as u8)
            }
            [Object::Int(v)] => {
                return Err(ill(subject, format!("agent:layerDepth {v} out of range 1..=5")))
            }
            [] => return Err(ill(subject, "missing agent:layerDepth")),
            [_] => return Err(ill(subject, "agent:layerDepth must be an integer literal")),
            _ => return Err(ill(subject, "multiple agent:layerDepth")),
        };

        agents.push(AgentProfile { iri: subject.to_string(), tools, can_spawn, layer_depth });
    }
    agents.sort_unstable_by(|a, b| a.iri.cmp(&b.iri));
    Ok(agents)
}

/// Canonical registry form: sorted `iri\ttools-csv\tcan_spawn-csv\tdepth`
/// lines, trailing newline. Sub-lists are already sorted+deduped by
/// [`extract_agents`], so this form is stable regardless of source triple
/// order.
#[must_use]
pub fn agent_canonical_form(agents: &[AgentProfile]) -> String {
    let mut out = String::new();
    for a in agents {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            a.iri,
            a.tools.join(","),
            a.can_spawn.join(","),
            a.layer_depth
        ));
    }
    out
}

/// Content address of the canonical agent registry form.
#[must_use]
pub fn agent_registry_hash(agents: &[AgentProfile]) -> String {
    content_address(agent_canonical_form(agents).as_bytes())
}

/// The depth-5 spawn law, ported as a Rust invariant from
/// five-layer-agents' `spawn_tree.sparql` / `tests/spawn_tree_test.rs`
/// assertion "no agent at depth 5 has `canSpawnType`": terminal by
/// absence, not by an explicit marker. Refuses the first depth-5 agent
/// (by sorted IRI order) that declares a non-empty `can_spawn`.
pub fn spawn_depth_law(profiles: &[AgentProfile]) -> Result<(), Refusal> {
    for p in profiles {
        if p.layer_depth == 5 && !p.can_spawn.is_empty() {
            return Err(ill(
                &p.iri,
                format!(
                    "agent at layerDepth 5 declares {} agent:canSpawn edge(s); \
                     depth-5 agents are terminal by absence of the spawn predicate \
                     (five-layer-agents spawn_tree law: no agent at depth 5 has canSpawnType)",
                    p.can_spawn.len()
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::parse_ttl;

    fn doc(body: &str) -> String {
        format!("@prefix agent: <{AGENT_NS}> .\n@prefix ex: <http://e/> .\n{body}")
    }

    #[test]
    fn extracts_tools_and_spawn_edges_sorted_deduped() {
        let ttl = doc(&format!(
            "ex:a a agent:Agent ; agent:layerDepth 1 ; \
             agent:tool \"Read\" ; agent:tool \"Agent\" ; agent:tool \"Read\" ; \
             agent:canSpawn <{AGENT_NS}coordinator> ; agent:canSpawn <{AGENT_NS}leaf-read> .\n"
        ));
        let agents = extract_agents(&parse_ttl(&ttl).unwrap()).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].tools, vec!["Agent".to_string(), "Read".to_string()]);
        assert_eq!(agents[0].layer_depth, 1);
        assert_eq!(agents[0].can_spawn.len(), 2);
    }

    #[test]
    fn depth_5_with_can_spawn_is_refused() {
        let ttl = doc(&format!(
            "ex:leaf a agent:Agent ; agent:layerDepth 5 ; \
             agent:canSpawn <{AGENT_NS}anything> .\n"
        ));
        let agents = extract_agents(&parse_ttl(&ttl).unwrap()).unwrap();
        match spawn_depth_law(&agents) {
            Err(Refusal::AgentIllFormed { subject, detail }) => {
                assert_eq!(subject, "http://e/leaf");
                assert!(detail.contains("terminal by absence"));
            }
            other => panic!("expected AgentIllFormed, got {other:?}"),
        }
    }

    #[test]
    fn depth_5_without_can_spawn_is_ok() {
        let ttl = doc("ex:leaf a agent:Agent ; agent:layerDepth 5 ; agent:tool \"Read\" .\n");
        let agents = extract_agents(&parse_ttl(&ttl).unwrap()).unwrap();
        spawn_depth_law(&agents).expect("depth-5 with no can_spawn is lawful");
    }

    #[test]
    fn non_depth_5_with_can_spawn_is_ok() {
        let ttl = doc(&format!(
            "ex:mid a agent:Agent ; agent:layerDepth 3 ; \
             agent:canSpawn <{AGENT_NS}leaf-read> .\n"
        ));
        let agents = extract_agents(&parse_ttl(&ttl).unwrap()).unwrap();
        spawn_depth_law(&agents).expect("non-depth-5 agents may spawn");
    }

    #[test]
    fn unknown_predicate_is_refused_by_name() {
        let ttl = doc("ex:a a agent:Agent ; agent:layerDepth 1 ; agent:bogus \"x\" .\n");
        match extract_agents(&parse_ttl(&ttl).unwrap()) {
            Err(Refusal::AgentIllFormed { detail, .. }) => {
                assert!(detail.contains("unknown agent: predicate 'bogus'"));
            }
            other => panic!("expected AgentIllFormed, got {other:?}"),
        }
    }

    #[test]
    fn unknown_class_is_refused_by_name() {
        let ttl = doc("ex:a a agent:Widget .\n");
        match extract_agents(&parse_ttl(&ttl).unwrap()) {
            Err(Refusal::AgentIllFormed { detail, .. }) => {
                assert!(detail.contains("unknown agent: class 'Widget'"));
            }
            other => panic!("expected AgentIllFormed, got {other:?}"),
        }
    }

    #[test]
    fn missing_layer_depth_is_refused() {
        let ttl = doc("ex:a a agent:Agent ; agent:tool \"Read\" .\n");
        match extract_agents(&parse_ttl(&ttl).unwrap()) {
            Err(Refusal::AgentIllFormed { detail, .. }) => {
                assert!(detail.contains("missing agent:layerDepth"));
            }
            other => panic!("expected AgentIllFormed, got {other:?}"),
        }
    }

    #[test]
    fn out_of_range_layer_depth_is_refused() {
        let ttl = doc("ex:a a agent:Agent ; agent:layerDepth 6 .\n");
        match extract_agents(&parse_ttl(&ttl).unwrap()) {
            Err(Refusal::AgentIllFormed { detail, .. }) => {
                assert!(detail.contains("out of range 1..=5"));
            }
            other => panic!("expected AgentIllFormed, got {other:?}"),
        }
    }

    #[test]
    fn registry_hash_is_stable_and_order_independent() {
        let ttl_ab = doc(
            "ex:a a agent:Agent ; agent:layerDepth 1 ; agent:tool \"Read\" .\n\
             ex:b a agent:Agent ; agent:layerDepth 2 ; agent:tool \"Grep\" .\n",
        );
        let ttl_ba = doc(
            "ex:b a agent:Agent ; agent:layerDepth 2 ; agent:tool \"Grep\" .\n\
             ex:a a agent:Agent ; agent:layerDepth 1 ; agent:tool \"Read\" .\n",
        );
        let ha = agent_registry_hash(&extract_agents(&parse_ttl(&ttl_ab).unwrap()).unwrap());
        let hb = agent_registry_hash(&extract_agents(&parse_ttl(&ttl_ba).unwrap()).unwrap());
        assert_eq!(ha, hb, "canonical form is sorted by IRI; source order must not matter");

        let ttl_changed = doc("ex:a a agent:Agent ; agent:layerDepth 1 ; agent:tool \"Write\" .\n");
        let hc = agent_registry_hash(&extract_agents(&parse_ttl(&ttl_changed).unwrap()).unwrap());
        assert_ne!(ha, hc, "changed content must change the hash");
    }

    #[test]
    fn too_many_tools_is_refused() {
        let tools: String =
            (0..=MAX_TOOLS).map(|i| format!("agent:tool \"t{i}\" ; ")).collect();
        let ttl = doc(&format!("ex:a a agent:Agent ; agent:layerDepth 1 ; {tools}.\n"));
        match extract_agents(&parse_ttl(&ttl).unwrap()) {
            Err(Refusal::AgentIllFormed { .. }) => {}
            other => panic!("expected AgentIllFormed, got {other:?}"),
        }
    }
}
