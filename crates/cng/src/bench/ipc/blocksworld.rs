//! Clean-room blocksworld generator (PROJ-711): the classic four-operator
//! table/tower domain (pickup / putdown / stack / unstack over `on`,
//! `on-table`, `clear`, `arm-empty`, `holding`), modeled from first
//! principles — no IPC file consulted.
//!
//! Size law: `size` = number of blocks (2..=[`MAX_SIZE`]). All blocks start
//! on the table, clear, arm empty. The goal is a tower of
//! `k = min(3, size)` blocks chosen by a seeded permutation, so the plan is
//! `2·(k−1) ≤ 4` steps — comfortably inside the blind-BFS bound.
//!
//! `SwappedGoalIdentities` swaps the identities of the top two tower blocks
//! (`k ≥ 2` always): the goal atoms change textually by construction, and
//! solvability is preserved because all blocks are symmetric in the initial
//! state.

use crate::powl::CngRefusal;

use super::{atom, permutation, ActionSpec, DomainSpec, IpcProblem, IpcVariant, ProblemSpec};

/// Maximum block count: grounding is O(2n + 2n²) ≤ 84 actions at n = 6.
pub const MAX_SIZE: u8 = 6;

/// Domain-namespacing salt for the seeded permutation (arbitrary fixed
/// constant; "blocks" in ASCII).
const SALT: u64 = 0x0062_6c6f_636b_73;

/// The four blocksworld schemas as typed specs.
///
/// # Complexity
/// O(1).
fn domain_spec() -> DomainSpec {
    DomainSpec {
        name: "cng-blocksworld".to_string(),
        actions: vec![
            ActionSpec {
                name: "pickup".to_string(),
                params: vec!["?x".to_string()],
                pre: vec![
                    atom("clear", &["?x"]),
                    atom("on-table", &["?x"]),
                    atom("arm-empty", &[]),
                ],
                add: vec![atom("holding", &["?x"])],
                del: vec![
                    atom("clear", &["?x"]),
                    atom("on-table", &["?x"]),
                    atom("arm-empty", &[]),
                ],
            },
            ActionSpec {
                name: "putdown".to_string(),
                params: vec!["?x".to_string()],
                pre: vec![atom("holding", &["?x"])],
                add: vec![
                    atom("clear", &["?x"]),
                    atom("on-table", &["?x"]),
                    atom("arm-empty", &[]),
                ],
                del: vec![atom("holding", &["?x"])],
            },
            ActionSpec {
                name: "stack".to_string(),
                params: vec!["?x".to_string(), "?y".to_string()],
                pre: vec![atom("holding", &["?x"]), atom("clear", &["?y"])],
                add: vec![
                    atom("on", &["?x", "?y"]),
                    atom("clear", &["?x"]),
                    atom("arm-empty", &[]),
                ],
                del: vec![atom("holding", &["?x"]), atom("clear", &["?y"])],
            },
            ActionSpec {
                name: "unstack".to_string(),
                params: vec!["?x".to_string(), "?y".to_string()],
                pre: vec![
                    atom("on", &["?x", "?y"]),
                    atom("clear", &["?x"]),
                    atom("arm-empty", &[]),
                ],
                add: vec![atom("holding", &["?x"]), atom("clear", &["?y"])],
                del: vec![
                    atom("on", &["?x", "?y"]),
                    atom("clear", &["?x"]),
                    atom("arm-empty", &[]),
                ],
            },
        ],
    }
}

/// Generates the `(seed, size)` blocksworld problem.
///
/// # Errors
/// `CNG_R05` for `size < 2` or `size > MAX_SIZE`; template IO refusals.
///
/// # Complexity
/// O(size) spec construction + render cost.
pub fn generate(seed: u64, size: u8, variant: IpcVariant) -> Result<IpcProblem, CngRefusal> {
    if !(2..=MAX_SIZE).contains(&size) {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "blocksworld size {size} outside 2..={MAX_SIZE}"
        )));
    }
    let n = size as usize;
    let blocks: Vec<String> = (1..=n).map(|i| format!("b{i}")).collect();

    // Tower = first k blocks of the seeded permutation, bottom-to-top.
    let mut perm = permutation(seed, SALT, n);
    let k = usize::min(3, n);
    if variant == IpcVariant::SwappedGoalIdentities {
        // k ≥ 2 always (n ≥ 2): swapping the two topmost tower identities
        // is guaranteed to change the goal text.
        perm.swap(k - 1, k - 2);
    }

    let mut init = vec![atom("arm-empty", &[])];
    for block in &blocks {
        init.push(atom("on-table", &[block.as_str()]));
        init.push(atom("clear", &[block.as_str()]));
    }
    // Goal: on(perm[i], perm[i-1]) for the tower spine. O(k).
    let mut goal = Vec::with_capacity(k - 1);
    for i in 1..k {
        goal.push(atom(
            "on",
            &[blocks[perm[i]].as_str(), blocks[perm[i - 1]].as_str()],
        ));
    }

    let problem = ProblemSpec {
        name: format!("cng-blocksworld-s{seed}-n{size}"),
        domain: "cng-blocksworld".to_string(),
        objects: blocks.clone(),
        init,
        goal,
    };
    super::render_ipc(&domain_spec(), &problem, seed, size, variant)
}
