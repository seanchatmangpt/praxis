//! Clean-room termes generator (PROJ-711): a construction robot on a line
//! of grid positions picks blocks at the depot and places them at target
//! positions (move / pick-block / place-block), modeled from first
//! principles as a STRIPS reduction of the termite-robot construction
//! scenario (no numeric heights, single block in hand; no IPC file
//! consulted).
//!
//! Size law: `size` (1..=[`MAX_SIZE`]) sets the line length
//! `m = size + 2` positions (`p1..pm`), depot and robot at `p1`. The goal
//! places one block at a seeded target position `p_t`, `t ∈ 2..=m`, so the
//! plan is `pick + (t−1) moves + place ≤ m + 1 ≤ 7` steps with tiny
//! branching (≤ 2 move directions). Grounding: 2-parameter `move` over
//! `m ≤ 6` objects → ≤ 36 + 2·6 ground actions.
//!
//! `SwappedGoalIdentities` shifts the target to the next position on the
//! line (wrapping `m → 2`), guaranteed distinct from the canonical target
//! and always reachable.

use crate::powl::CngRefusal;

use super::{atom, draw, ActionSpec, DomainSpec, IpcProblem, IpcVariant, ProblemSpec};

/// Maximum size: line length `size + 2 ≤ 6` positions.
pub const MAX_SIZE: u8 = 4;

/// Domain-namespacing salt ("term" in ASCII).
const SALT: u64 = 0x0000_0074_6572_6d;

/// The three termes schemas as typed specs.
///
/// # Complexity
/// O(1).
fn domain_spec() -> DomainSpec {
    DomainSpec {
        name: "cng-termes".to_string(),
        actions: vec![
            ActionSpec {
                name: "move".to_string(),
                params: vec!["?a".to_string(), "?b".to_string()],
                pre: vec![atom("robot-at", &["?a"]), atom("neighbor", &["?a", "?b"])],
                add: vec![atom("robot-at", &["?b"])],
                del: vec![atom("robot-at", &["?a"])],
            },
            ActionSpec {
                name: "pick-block".to_string(),
                params: vec!["?p".to_string()],
                pre: vec![
                    atom("robot-at", &["?p"]),
                    atom("depot", &["?p"]),
                    atom("hand-empty", &[]),
                ],
                add: vec![atom("has-block", &[])],
                del: vec![atom("hand-empty", &[])],
            },
            ActionSpec {
                name: "place-block".to_string(),
                params: vec!["?p".to_string()],
                pre: vec![atom("robot-at", &["?p"]), atom("has-block", &[])],
                add: vec![atom("block-at", &["?p"]), atom("hand-empty", &[])],
                del: vec![atom("has-block", &[])],
            },
        ],
    }
}

/// Generates the `(seed, size)` termes problem.
///
/// # Errors
/// `CNG_R05` for `size` outside `1..=MAX_SIZE`; template IO refusals.
///
/// # Complexity
/// O(size) spec construction + render cost.
pub fn generate(seed: u64, size: u8, variant: IpcVariant) -> Result<IpcProblem, CngRefusal> {
    if !(1..=MAX_SIZE).contains(&size) {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "termes size {size} outside 1..={MAX_SIZE}"
        )));
    }
    let m = size as usize + 2;
    let positions: Vec<String> = (1..=m).map(|i| format!("p{i}")).collect();

    let mut init = vec![
        atom("depot", &["p1"]),
        atom("robot-at", &["p1"]),
        atom("hand-empty", &[]),
    ];
    // Line adjacency, both directions. O(m).
    for i in 0..(m - 1) {
        init.push(atom(
            "neighbor",
            &[positions[i].as_str(), positions[i + 1].as_str()],
        ));
        init.push(atom(
            "neighbor",
            &[positions[i + 1].as_str(), positions[i].as_str()],
        ));
    }

    // Seeded target t ∈ 2..=m; the variant shifts it to the next position
    // on the line (wrapping m → 2), guaranteed distinct because m ≥ 3.
    let canonical_t = 2 + (draw(seed, SALT) % (m as u64 - 1)) as usize;
    let t = match variant {
        IpcVariant::Canonical => canonical_t,
        IpcVariant::SwappedGoalIdentities => {
            if canonical_t == m {
                2
            } else {
                canonical_t + 1
            }
        }
    };
    // t ∈ 2..=m by construction; positions is 1-indexed by name.
    let goal = vec![atom("block-at", &[positions[t - 1].as_str()])];

    let problem = ProblemSpec {
        name: format!("cng-termes-s{seed}-n{size}"),
        domain: "cng-termes".to_string(),
        objects: positions.clone(),
        init,
        goal,
    };
    super::render_ipc(&domain_spec(), &problem, seed, size, variant)
}
