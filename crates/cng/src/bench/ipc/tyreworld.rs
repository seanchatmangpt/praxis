//! Clean-room tyreworld generator (PROJ-711): swap a flat wheel for the
//! spare — open the boot, fetch tools and spare, loosen the nuts, jack up,
//! remove the nuts and the flat wheel, mount the spare, tighten — modeled
//! from first principles on a single hub; no IPC file consulted.
//!
//! Size law (`1..=`[`MAX_SIZE`]) scales the plan length by how much of the
//! setup is already done:
//! - size 1: boot open, wrench + jack in hand, spare in boot → 7-step plan
//! - size 2: boot open, wrench + jack + spare all in boot → 9-step plan
//! - size 3: boot closed, everything in boot → 10-step plan
//!
//! The corridor-shaped state space (few applicable actions per state) keeps
//! blind BFS cheap despite the 10-step horizon.
//!
//! The seed selects which of the two symmetric wheels is flat (on the hub)
//! versus spare (to be mounted); `SwappedGoalIdentities` flips that
//! assignment — the goal names the other wheel by construction, and
//! solvability is preserved by wheel symmetry.

use crate::powl::CngRefusal;

use super::{atom, draw, ActionSpec, DomainSpec, IpcProblem, IpcVariant, ProblemSpec};

/// Maximum setup-depth size (see module doc size law).
pub const MAX_SIZE: u8 = 3;

/// Domain-namespacing salt ("tyre" in ASCII).
const SALT: u64 = 0x0000_0074_7972_65;

/// The nine tyreworld schemas as typed specs.
///
/// # Complexity
/// O(1).
fn domain_spec() -> DomainSpec {
    DomainSpec {
        name: "cng-tyreworld".to_string(),
        actions: vec![
            ActionSpec {
                name: "open-boot".to_string(),
                params: vec![],
                pre: vec![atom("boot-closed", &[])],
                add: vec![atom("boot-open", &[])],
                del: vec![atom("boot-closed", &[])],
            },
            ActionSpec {
                name: "fetch".to_string(),
                params: vec!["?x".to_string()],
                pre: vec![atom("boot-open", &[]), atom("in-boot", &["?x"])],
                add: vec![atom("have", &["?x"])],
                del: vec![atom("in-boot", &["?x"])],
            },
            ActionSpec {
                name: "loosen".to_string(),
                params: vec!["?t".to_string()],
                pre: vec![
                    atom("is-wrench", &["?t"]),
                    atom("have", &["?t"]),
                    atom("nuts-tight", &[]),
                    atom("on-ground", &[]),
                ],
                add: vec![atom("nuts-loose", &[])],
                del: vec![atom("nuts-tight", &[])],
            },
            ActionSpec {
                name: "jack-up".to_string(),
                params: vec!["?j".to_string()],
                pre: vec![
                    atom("is-jack", &["?j"]),
                    atom("have", &["?j"]),
                    atom("on-ground", &[]),
                    atom("nuts-loose", &[]),
                ],
                add: vec![atom("jacked-up", &[])],
                del: vec![atom("on-ground", &[])],
            },
            ActionSpec {
                name: "remove-nuts".to_string(),
                params: vec!["?t".to_string()],
                pre: vec![
                    atom("is-wrench", &["?t"]),
                    atom("have", &["?t"]),
                    atom("nuts-loose", &[]),
                    atom("jacked-up", &[]),
                ],
                add: vec![atom("nuts-off", &[])],
                del: vec![atom("nuts-loose", &[])],
            },
            ActionSpec {
                name: "remove-wheel".to_string(),
                params: vec!["?w".to_string()],
                pre: vec![
                    atom("is-wheel", &["?w"]),
                    atom("on-hub", &["?w"]),
                    atom("nuts-off", &[]),
                    atom("jacked-up", &[]),
                ],
                add: vec![atom("have", &["?w"]), atom("hub-free", &[])],
                del: vec![atom("on-hub", &["?w"])],
            },
            ActionSpec {
                name: "put-on-wheel".to_string(),
                params: vec!["?w".to_string()],
                pre: vec![
                    atom("is-wheel", &["?w"]),
                    atom("have", &["?w"]),
                    atom("hub-free", &[]),
                    atom("nuts-off", &[]),
                    atom("jacked-up", &[]),
                ],
                add: vec![atom("on-hub", &["?w"])],
                del: vec![atom("have", &["?w"]), atom("hub-free", &[])],
            },
            ActionSpec {
                name: "tighten".to_string(),
                params: vec!["?t".to_string(), "?w".to_string()],
                pre: vec![
                    atom("is-wrench", &["?t"]),
                    atom("have", &["?t"]),
                    atom("is-wheel", &["?w"]),
                    atom("on-hub", &["?w"]),
                    atom("nuts-off", &[]),
                    atom("jacked-up", &[]),
                ],
                add: vec![atom("nuts-tight", &[])],
                del: vec![atom("nuts-off", &[])],
            },
            ActionSpec {
                name: "jack-down".to_string(),
                params: vec!["?j".to_string()],
                pre: vec![
                    atom("is-jack", &["?j"]),
                    atom("have", &["?j"]),
                    atom("jacked-up", &[]),
                    atom("nuts-tight", &[]),
                ],
                add: vec![atom("on-ground", &[])],
                del: vec![atom("jacked-up", &[])],
            },
        ],
    }
}

/// Generates the `(seed, size)` tyreworld problem.
///
/// # Errors
/// `CNG_R05` for `size` outside `1..=MAX_SIZE`; template IO refusals.
///
/// # Complexity
/// O(1) spec construction + render cost.
pub fn generate(seed: u64, size: u8, variant: IpcVariant) -> Result<IpcProblem, CngRefusal> {
    if !(1..=MAX_SIZE).contains(&size) {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "tyreworld size {size} outside 1..={MAX_SIZE}"
        )));
    }
    // Seeded flat/spare assignment over the two symmetric wheels; the
    // variant flips it (guaranteed goal-text change).
    let seed_bit = draw(seed, SALT) & 1 == 1;
    let flipped = match variant {
        IpcVariant::Canonical => seed_bit,
        IpcVariant::SwappedGoalIdentities => !seed_bit,
    };
    let (flat, spare) = if flipped { ("w2", "w1") } else { ("w1", "w2") };

    let objects = vec![
        "w1".to_string(),
        "w2".to_string(),
        "wrench1".to_string(),
        "jack1".to_string(),
    ];
    let mut init = vec![
        atom("is-wheel", &["w1"]),
        atom("is-wheel", &["w2"]),
        atom("is-wrench", &["wrench1"]),
        atom("is-jack", &["jack1"]),
        atom("on-hub", &[flat]),
        atom("nuts-tight", &[]),
        atom("on-ground", &[]),
        atom("in-boot", &[spare]),
    ];
    match size {
        1 => {
            init.push(atom("boot-open", &[]));
            init.push(atom("have", &["wrench1"]));
            init.push(atom("have", &["jack1"]));
        }
        2 => {
            init.push(atom("boot-open", &[]));
            init.push(atom("in-boot", &["wrench1"]));
            init.push(atom("in-boot", &["jack1"]));
        }
        _ => {
            init.push(atom("boot-closed", &[]));
            init.push(atom("in-boot", &["wrench1"]));
            init.push(atom("in-boot", &["jack1"]));
        }
    }
    let goal = vec![atom("on-hub", &[spare]), atom("nuts-tight", &[])];

    let problem = ProblemSpec {
        name: format!("cng-tyreworld-s{seed}-n{size}"),
        domain: "cng-tyreworld".to_string(),
        objects,
        init,
        goal,
    };
    super::render_ipc(&domain_spec(), &problem, seed, size, variant)
}
