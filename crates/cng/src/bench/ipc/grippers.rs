//! Clean-room grippers generator (PROJ-711): one robot with two grippers
//! moves balls between two rooms (move / pick / drop), modeled from first
//! principles — no IPC file consulted.
//!
//! Size law: `size` = ball count (1..=[`MAX_SIZE`]). Guard predicates
//! (`is-room`, `is-gripper`) keep the untyped grounding's junk actions
//! permanently inapplicable. Goal: `min(2, size)` seeded-chosen balls end
//! in the target room, so the plan is ≤ 5 steps (two picks fill both
//! grippers, one move, two drops). Grounding: 4-parameter pick/drop over
//! `5 + size ≤ 9` objects → ≤ 2·9⁴ + 9³ ≈ 13,850 ground actions, under the
//! decomposition bound.
//!
//! `SwappedGoalIdentities` swaps the start/target ROOM identities (balls
//! and robot start in the other room, goal names the other room): the goal
//! text changes by construction and solvability is preserved because the
//! two rooms are symmetric.

use crate::powl::CngRefusal;

use super::{atom, permutation, ActionSpec, DomainSpec, IpcProblem, IpcVariant, ProblemSpec};

/// Maximum ball count: keeps 4-parameter grounding under the decomposition
/// grounding bound (see module doc arithmetic).
pub const MAX_SIZE: u8 = 4;

/// Domain-namespacing salt ("grip" in ASCII).
const SALT: u64 = 0x0000_0067_7269_70;

/// The three grippers schemas as typed specs.
///
/// # Complexity
/// O(1).
fn domain_spec() -> DomainSpec {
    DomainSpec {
        name: "cng-grippers".to_string(),
        actions: vec![
            ActionSpec {
                name: "move".to_string(),
                params: vec!["?r".to_string(), "?from".to_string(), "?to".to_string()],
                pre: vec![
                    atom("at-robot", &["?r", "?from"]),
                    atom("is-room", &["?to"]),
                ],
                add: vec![atom("at-robot", &["?r", "?to"])],
                del: vec![atom("at-robot", &["?r", "?from"])],
            },
            ActionSpec {
                name: "pick".to_string(),
                params: vec![
                    "?r".to_string(),
                    "?b".to_string(),
                    "?room".to_string(),
                    "?g".to_string(),
                ],
                pre: vec![
                    atom("at-robot", &["?r", "?room"]),
                    atom("at-ball", &["?b", "?room"]),
                    atom("free", &["?g"]),
                    atom("is-gripper", &["?g"]),
                ],
                add: vec![atom("carry", &["?b", "?g"])],
                del: vec![atom("at-ball", &["?b", "?room"]), atom("free", &["?g"])],
            },
            ActionSpec {
                name: "drop".to_string(),
                params: vec![
                    "?r".to_string(),
                    "?b".to_string(),
                    "?room".to_string(),
                    "?g".to_string(),
                ],
                pre: vec![
                    atom("at-robot", &["?r", "?room"]),
                    atom("carry", &["?b", "?g"]),
                ],
                add: vec![atom("at-ball", &["?b", "?room"]), atom("free", &["?g"])],
                del: vec![atom("carry", &["?b", "?g"])],
            },
        ],
    }
}

/// Generates the `(seed, size)` grippers problem.
///
/// # Errors
/// `CNG_R05` for `size` outside `1..=MAX_SIZE`; template IO refusals.
///
/// # Complexity
/// O(size) spec construction + render cost.
pub fn generate(seed: u64, size: u8, variant: IpcVariant) -> Result<IpcProblem, CngRefusal> {
    if !(1..=MAX_SIZE).contains(&size) {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "grippers size {size} outside 1..={MAX_SIZE}"
        )));
    }
    let n = size as usize;
    let balls: Vec<String> = (1..=n).map(|i| format!("ball{i}")).collect();
    let (start_room, target_room) = match variant {
        IpcVariant::Canonical => ("room-a", "room-b"),
        IpcVariant::SwappedGoalIdentities => ("room-b", "room-a"),
    };

    let mut objects = vec![
        "robot1".to_string(),
        "g-left".to_string(),
        "g-right".to_string(),
        "room-a".to_string(),
        "room-b".to_string(),
    ];
    objects.extend(balls.iter().cloned());

    let mut init = vec![
        atom("is-room", &["room-a"]),
        atom("is-room", &["room-b"]),
        atom("is-gripper", &["g-left"]),
        atom("is-gripper", &["g-right"]),
        atom("free", &["g-left"]),
        atom("free", &["g-right"]),
        atom("at-robot", &["robot1", start_room]),
    ];
    for ball in &balls {
        init.push(atom("at-ball", &[ball.as_str(), start_room]));
    }

    // Goal: the first min(2, size) balls of the seeded permutation reach
    // the target room. O(size).
    let perm = permutation(seed, SALT, n);
    let k = usize::min(2, n);
    let mut goal = Vec::with_capacity(k);
    for &idx in perm.iter().take(k) {
        goal.push(atom("at-ball", &[balls[idx].as_str(), target_room]));
    }

    let problem = ProblemSpec {
        name: format!("cng-grippers-s{seed}-n{size}"),
        domain: "cng-grippers".to_string(),
        objects,
        init,
        goal,
    };
    super::render_ipc(&domain_spec(), &problem, seed, size, variant)
}
