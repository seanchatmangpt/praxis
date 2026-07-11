//! PROJ-711 — Clean-room IPC-style benchmark problem generators.
//!
//! Five classical-planning domains (barman, blocksworld, grippers, termes,
//! tyreworld) modeled from first principles — the domain SEMANTICS
//! (stack/unstack, gripper transport, wheel swap, cocktail pouring, tower
//! construction) are public knowledge; no IPC PDDL file was copied or
//! consulted. Every generator is a pure function of `(seed, size)`:
//! deterministic via `splitmix64`, byte-identical on regeneration, no wall
//! clock, no ambient state.
//!
//! STRIPS8 discipline: untyped STRIPS only, ≤ 8 parameters and ≤ 8
//! precondition/effect conjuncts per action schema, canonical symbol
//! grammar `^[a-z][a-z0-9-]*$` (see `ontologies/pddl-strips.ttl`). PDDL
//! text is assembled from typed Rust specs through the on-disk
//! `templates/ipc-*.template.pddl` skeletons (mirroring
//! `decomp/render.rs`) — zero inline PDDL skeletons in Rust source.
//!
//! Solvability is gated honestly against the blind bounded-BFS planner
//! (`pddl_index::ground::IndexedGroundProblem::find_plan`, PROJ-733;
//! `PDDL8_MAX_PLAN_DEPTH`):
//! [`generate_solvable`] steps `size` DOWN from the requested maximum until
//! a plan exists, and refuses `CNG_R04` (with the last per-size refusal
//! recorded) when no size admits one. Sizes are tuned so plans stay short
//! (≤ ~10 steps) and grounding stays under
//! [`crate::bench::decomp::DECOMP_MAX_GROUND`] — the honest constraint of a
//! heuristic-free planner, per the v26.7.10-revised plan risk register.
//!
//! Anti-hardcoding surface (PROJ-713): every generator also accepts an
//! [`IpcVariant`] — `SwappedGoalIdentities` applies a domain-specific
//! identity/goal permutation that is GUARANTEED to change the goal text
//! while preserving solvability by symmetry, so digest-causality tests
//! never flake on a coincidentally-identical random permutation.
//!
//! PROJ-714 (4 long-horizon scenarios, G14/G15) is the declared cut line of
//! this increment and is deliberately NOT implemented here — no
//! `long_horizon` module exists; invoking long-horizon behavior is
//! impossible rather than quietly stubbed.

pub mod barman;
pub mod blocksworld;
pub mod grippers;
pub mod termes;
pub mod tyreworld;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_pddl::{Pddl8Domain, Pddl8Problem, Pddl8Tape};
// PROJ-733: same relaxed-reachability-pruned grounder as decomp/mod.rs —
// grippers-scale untyped domains ground large under bcinr_pddl's naive
// full-cross-product grounder even for this cheap solvability-gate path.
use pddl_index::ground::IndexedGroundProblem as GroundProblem;

use crate::bench::decomp::DECOMP_MAX_GROUND;
use crate::powl::CngRefusal;

/// The five clean-room corpus domains, canonical order.
pub const IPC_DOMAINS: [&str; 5] = ["barman", "blocksworld", "grippers", "termes", "tyreworld"];

/// Corpus width: seeds `0..IPC_CORPUS_SEEDS` per domain at the gated size.
pub const IPC_CORPUS_SEEDS: u64 = 20;

/// STRIPS8 caps enforced at authoring time (mirrors
/// `bcinr_pddl::PDDL8_MAX_PARAMS` / `PDDL8_MAX_CONJUNCTS`).
const IPC_MAX_PARAMS: usize = 8;
const IPC_MAX_CONJUNCTS: usize = 8;

/// Identity/goal permutation axis for the anti-hardcoding gate (PROJ-713).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcVariant {
    /// The canonical `(seed, size)` problem.
    Canonical,
    /// A domain-specific symmetric permutation of goal-bearing identities;
    /// guaranteed to change the goal text, guaranteed still solvable.
    SwappedGoalIdentities,
}

impl IpcVariant {
    /// Stable receipt string.
    pub fn as_str(&self) -> &'static str {
        match self {
            IpcVariant::Canonical => "canonical",
            IpcVariant::SwappedGoalIdentities => "swapped-goal-identities",
        }
    }
}

/// Generation metadata carried alongside the PDDL text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcMeta {
    /// Domain family name (one of [`IPC_DOMAINS`]).
    pub domain: String,
    /// Generation seed.
    pub seed: u64,
    /// Generation size parameter.
    pub size: u8,
    /// Variant receipt string (see [`IpcVariant::as_str`]).
    pub variant: &'static str,
    /// Object count in the problem.
    pub objects: usize,
    /// Goal conjunct count.
    pub goal_atoms: usize,
}

/// One generated corpus problem: deterministic PDDL text + metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcProblem {
    /// Rendered PDDL domain text.
    pub domain_pddl: String,
    /// Rendered PDDL problem text.
    pub problem_pddl: String,
    /// Generation metadata.
    pub meta: IpcMeta,
}

/// A schema-level atom: predicate name + `?var`/constant argument symbols.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AtomSpec {
    pub(crate) pred: String,
    pub(crate) args: Vec<String>,
}

impl AtomSpec {
    /// PDDL s-expression `(pred a0 a1)` / bare `(pred)`.
    ///
    /// # Complexity
    /// O(arity).
    fn sexpr(&self) -> String {
        if self.args.is_empty() {
            format!("({})", self.pred)
        } else {
            format!("({} {})", self.pred, self.args.join(" "))
        }
    }
}

/// Typed atom constructor used by every generator.
///
/// # Complexity
/// O(arity).
pub(crate) fn atom(pred: &str, args: &[&str]) -> AtomSpec {
    AtomSpec {
        pred: pred.to_string(),
        args: args.iter().map(|a| (*a).to_string()).collect(),
    }
}

/// One STRIPS action schema, as typed data (never PDDL text).
#[derive(Debug, Clone)]
pub(crate) struct ActionSpec {
    pub(crate) name: String,
    /// Parameter names, each starting with `?`.
    pub(crate) params: Vec<String>,
    pub(crate) pre: Vec<AtomSpec>,
    pub(crate) add: Vec<AtomSpec>,
    pub(crate) del: Vec<AtomSpec>,
}

/// A typed domain spec ready for template rendering.
#[derive(Debug, Clone)]
pub(crate) struct DomainSpec {
    pub(crate) name: String,
    pub(crate) actions: Vec<ActionSpec>,
}

/// A typed problem spec ready for template rendering.
#[derive(Debug, Clone)]
pub(crate) struct ProblemSpec {
    pub(crate) name: String,
    pub(crate) domain: String,
    pub(crate) objects: Vec<String>,
    pub(crate) init: Vec<AtomSpec>,
    pub(crate) goal: Vec<AtomSpec>,
}

/// Loads the two on-disk ipc templates (`(domain, problem)` skeletons).
///
/// # Errors
/// `CNG_R10 IoRefused` for unreadable template files.
///
/// # Complexity
/// O(files).
fn load_ipc_templates() -> Result<(String, String), CngRefusal> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let read = |name: &str| -> Result<String, CngRefusal> {
        fs::read_to_string(dir.join(name))
            .map_err(|e| CngRefusal::IoRefused(format!("cannot read template {name}: {e}")))
    };
    Ok((
        read("ipc-domain.template.pddl")?,
        read("ipc-problem.template.pddl")?,
    ))
}

/// Renders a [`DomainSpec`] to deterministic PDDL domain text: predicate
/// declarations are derived from every atom's `(name, arity)` (arity
/// conflict refused), action blocks and all conjunct lists are sorted.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` for STRIPS8 bound violations (> 8 params,
/// > 8 precondition conjuncts, > 8 effect conjuncts) or predicate arity
/// conflicts — authoring bugs surface loudly, never as planner mysteries.
///
/// # Complexity
/// O(A · c log c) over A actions with ≤ c conjuncts each.
fn render_domain_spec(spec: &DomainSpec, template: &str) -> Result<String, CngRefusal> {
    let mut arities: BTreeMap<String, usize> = BTreeMap::new();
    for action in &spec.actions {
        if action.params.len() > IPC_MAX_PARAMS {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "ipc action {} has {} parameters; STRIPS8 caps at {IPC_MAX_PARAMS}",
                action.name,
                action.params.len()
            )));
        }
        if action.pre.len() > IPC_MAX_CONJUNCTS {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "ipc action {} has {} precondition conjuncts; STRIPS8 caps at {IPC_MAX_CONJUNCTS}",
                action.name,
                action.pre.len()
            )));
        }
        if action.add.len() + action.del.len() > IPC_MAX_CONJUNCTS {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "ipc action {} has {} effect conjuncts; STRIPS8 caps at {IPC_MAX_CONJUNCTS}",
                action.name,
                action.add.len() + action.del.len()
            )));
        }
        for a in action.pre.iter().chain(&action.add).chain(&action.del) {
            match arities.get(&a.pred) {
                Some(&known) if known != a.args.len() => {
                    return Err(CngRefusal::UnsupportedConstruct(format!(
                        "ipc predicate {} used with arities {known} and {}; STRIPS predicates \
                         have one arity",
                        a.pred,
                        a.args.len()
                    )));
                }
                _ => {
                    arities.insert(a.pred.clone(), a.args.len());
                }
            }
        }
    }

    // Predicate declarations, sorted by name. O(P).
    let mut predicates = Vec::with_capacity(arities.len());
    for (pred, arity) in &arities {
        let vars: Vec<String> = (0..*arity).map(|i| format!("?x{i}")).collect();
        if vars.is_empty() {
            predicates.push(format!("({pred})"));
        } else {
            predicates.push(format!("({pred} {})", vars.join(" ")));
        }
    }

    // Action blocks, sorted conjuncts, sorted blocks. O(A · c log c).
    let mut blocks = Vec::with_capacity(spec.actions.len());
    for action in &spec.actions {
        let mut pre: Vec<String> = action.pre.iter().map(AtomSpec::sexpr).collect();
        pre.sort();
        pre.dedup();
        let mut effects: Vec<String> = action.add.iter().map(AtomSpec::sexpr).collect();
        effects.extend(action.del.iter().map(|a| format!("(not {})", a.sexpr())));
        effects.sort();
        effects.dedup();
        let mut block = String::new();
        block.push_str("  (:action ");
        block.push_str(&action.name);
        block.push_str("\n    :parameters (");
        block.push_str(&action.params.join(" "));
        block.push_str(")\n    :precondition (and ");
        block.push_str(&pre.join(" "));
        block.push_str(")\n    :effect (and ");
        block.push_str(&effects.join(" "));
        block.push_str("))");
        blocks.push(block);
    }
    blocks.sort();

    Ok(crate::bench::fill_template(
        template,
        &[
            ("DOMAIN_NAME", spec.name.as_str()),
            ("PREDICATES", predicates.join(" ").as_str()),
            ("ACTIONS", blocks.join("\n").as_str()),
        ],
    ))
}

/// Renders a [`ProblemSpec`] to deterministic PDDL problem text (sorted
/// objects/init/goal lists).
///
/// # Errors
/// `CNG_R05` for an empty goal (not renderable STRIPS) or > 8 goal
/// conjuncts.
///
/// # Complexity
/// O(n log n) over objects + init + goal atoms.
fn render_problem_spec(spec: &ProblemSpec, template: &str) -> Result<String, CngRefusal> {
    if spec.goal.is_empty() {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "ipc problem {} has an empty goal; an empty goal is not renderable STRIPS",
            spec.name
        )));
    }
    if spec.goal.len() > IPC_MAX_CONJUNCTS {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "ipc problem {} has {} goal conjuncts; STRIPS8 caps at {IPC_MAX_CONJUNCTS}",
            spec.name,
            spec.goal.len()
        )));
    }
    let mut objects = spec.objects.clone();
    objects.sort();
    objects.dedup();
    let mut init: Vec<String> = spec.init.iter().map(AtomSpec::sexpr).collect();
    init.sort();
    init.dedup();
    let mut goal: Vec<String> = spec.goal.iter().map(AtomSpec::sexpr).collect();
    goal.sort();
    goal.dedup();
    Ok(crate::bench::fill_template(
        template,
        &[
            ("PROBLEM_NAME", spec.name.as_str()),
            ("DOMAIN_NAME", spec.domain.as_str()),
            ("OBJECTS", objects.join(" ").as_str()),
            ("INIT", init.join(" ").as_str()),
            ("GOAL", goal.join(" ").as_str()),
        ],
    ))
}

/// Renders a `(DomainSpec, ProblemSpec)` pair into an [`IpcProblem`]
/// through the on-disk ipc templates.
///
/// # Errors
/// See [`render_domain_spec`] / [`render_problem_spec`] / template IO.
///
/// # Complexity
/// Render cost of both specs.
pub(crate) fn render_ipc(
    domain: &DomainSpec,
    problem: &ProblemSpec,
    seed: u64,
    size: u8,
    variant: IpcVariant,
) -> Result<IpcProblem, CngRefusal> {
    let (domain_template, problem_template) = load_ipc_templates()?;
    let domain_pddl = render_domain_spec(domain, &domain_template)?;
    let problem_pddl = render_problem_spec(problem, &problem_template)?;
    // Family name = the corpus key (`IPC_DOMAINS` entry); the rendered PDDL
    // domain name carries a `cng-` prefix to mark clean-room provenance.
    let family = domain
        .name
        .strip_prefix("cng-")
        .unwrap_or(domain.name.as_str())
        .to_string();
    Ok(IpcProblem {
        domain_pddl,
        problem_pddl,
        meta: IpcMeta {
            domain: family,
            seed,
            size,
            variant: variant.as_str(),
            objects: problem.objects.len(),
            goal_atoms: problem.goal.len(),
        },
    })
}

/// Deterministic Fisher–Yates permutation of `0..n` seeded by
/// `splitmix64(seed ^ salt)`; the salt namespaces domains so the same seed
/// draws independent permutations per domain.
///
/// # Complexity
/// O(n).
pub(crate) fn permutation(seed: u64, salt: u64, n: usize) -> Vec<usize> {
    let mut out: Vec<usize> = (0..n).collect();
    let mut state = seed ^ salt.rotate_left(17);
    // Fisher–Yates; index arithmetic stays within 0..=i by construction.
    for i in (1..n).rev() {
        let j = (crate::bench::splitmix64(&mut state) % (i as u64 + 1)) as usize;
        out.swap(i, j);
    }
    out
}

/// One deterministic draw from the domain-salted stream (goal/role
/// selections that need a single value rather than a permutation).
///
/// # Complexity
/// O(1).
pub(crate) fn draw(seed: u64, salt: u64) -> u64 {
    let mut state = seed ^ salt.rotate_left(17);
    crate::bench::splitmix64(&mut state)
}

/// Parses a generated problem back through the unchanged bcinr parser.
///
/// # Errors
/// `CNG_R01 MalformedTtl` when the rendered text fails to parse (a
/// generator bug, surfaced loudly).
///
/// # Complexity
/// Parser cost over the rendered text.
pub fn parse_surface(problem: &IpcProblem) -> Result<(Pddl8Domain, Pddl8Problem), CngRefusal> {
    let domain = domain_from_pddl(&problem.domain_pddl).map_err(|e| {
        CngRefusal::MalformedTtl(format!(
            "generated ipc domain {} failed to parse: {e:?}",
            problem.meta.domain
        ))
    })?;
    let parsed = problem_from_pddl(&problem.problem_pddl).map_err(|e| {
        CngRefusal::MalformedTtl(format!(
            "generated ipc problem {} failed to parse: {e:?}",
            problem.meta.domain
        ))
    })?;
    Ok((domain, parsed))
}

/// Grounds and plans a generated problem on the unchanged
/// `GroundProblem::build` → `find_plan` path (bounded blind BFS).
///
/// # Errors
/// `CNG_R01` parse failures, `CNG_R05` grounding-bound failures,
/// `CNG_R04 PlanUnsolvable` when the bounded BFS finds no plan.
///
/// # Complexity
/// Bounded grounding (≤ [`DECOMP_MAX_GROUND`] actions) + bounded BFS
/// (≤ `PDDL8_MAX_PLAN_DEPTH` layers).
pub fn plan(problem: &IpcProblem) -> Result<Pddl8Tape, CngRefusal> {
    let (domain, parsed) = parse_surface(problem)?;
    let ground = GroundProblem::build(&domain, &parsed, Some(DECOMP_MAX_GROUND)).map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!(
            "ipc {} grounding failed: {e}",
            problem.meta.domain
        ))
    })?;
    ground.find_plan().map_err(|e| {
        CngRefusal::PlanUnsolvable(format!(
            "ipc {} seed {} size {} admits no plan: {e}",
            problem.meta.domain, problem.meta.seed, problem.meta.size
        ))
    })
}

/// The declared per-domain maximum size (tuned to the blind-BFS +
/// grounding-bound constraint; see each generator's doc comment).
///
/// # Errors
/// `CNG_R05` for an unknown domain name.
pub fn max_size(domain: &str) -> Result<u8, CngRefusal> {
    match domain {
        "barman" => Ok(barman::MAX_SIZE),
        "blocksworld" => Ok(blocksworld::MAX_SIZE),
        "grippers" => Ok(grippers::MAX_SIZE),
        "termes" => Ok(termes::MAX_SIZE),
        "tyreworld" => Ok(tyreworld::MAX_SIZE),
        other => Err(CngRefusal::UnsupportedConstruct(format!(
            "unknown ipc domain {other:?}; supported: {IPC_DOMAINS:?}"
        ))),
    }
}

/// Generates the canonical `(seed, size)` problem for a named domain.
///
/// # Errors
/// `CNG_R05` for unknown domains or out-of-range sizes; template IO.
pub fn generate(domain: &str, seed: u64, size: u8) -> Result<IpcProblem, CngRefusal> {
    generate_variant(domain, seed, size, IpcVariant::Canonical)
}

/// [`generate`] with an explicit [`IpcVariant`] (PROJ-713 permutation axis).
///
/// # Errors
/// See [`generate`].
pub fn generate_variant(
    domain: &str,
    seed: u64,
    size: u8,
    variant: IpcVariant,
) -> Result<IpcProblem, CngRefusal> {
    match domain {
        "barman" => barman::generate(seed, size, variant),
        "blocksworld" => blocksworld::generate(seed, size, variant),
        "grippers" => grippers::generate(seed, size, variant),
        "termes" => termes::generate(seed, size, variant),
        "tyreworld" => tyreworld::generate(seed, size, variant),
        other => Err(CngRefusal::UnsupportedConstruct(format!(
            "unknown ipc domain {other:?}; supported: {IPC_DOMAINS:?}"
        ))),
    }
}

/// The runtime size-backoff solvability gate: steps `size` DOWN from
/// `max_size` until the bounded blind BFS finds a plan, returning the first
/// solvable problem and its gated size. Per-size failures are RECORDED into
/// the final refusal (never silently swallowed); this function is a runtime
/// gate for tests and corpus runs, never a const/authoring-time evaluation.
///
/// # Errors
/// `CNG_R04 PlanUnsolvable` when no size in `1..=max_size` admits a plan,
/// carrying the last per-size refusal; `CNG_R05` for unknown domains.
///
/// # Complexity
/// O(max_size) generate+ground+BFS attempts, each bounded (see [`plan`]).
pub fn generate_solvable(
    domain: &str,
    seed: u64,
    max_size: u8,
) -> Result<(IpcProblem, u8), CngRefusal> {
    // Refuse unknown domains up front rather than reporting them as
    // unsolvability.
    let _declared = self::max_size(domain)?;
    let mut last = String::from("no size attempted");
    for size in (1..=max_size).rev() {
        match generate(domain, seed, size).and_then(|p| plan(&p).map(|_| p)) {
            Ok(problem) => return Ok((problem, size)),
            Err(refusal) => {
                last = format!("size {size}: {} {}", refusal.code(), refusal.message());
            }
        }
    }
    Err(CngRefusal::PlanUnsolvable(format!(
        "ipc domain {domain} seed {seed}: no size in 1..={max_size} admits a bounded-BFS plan; \
         last recorded refusal: {last}"
    )))
}
