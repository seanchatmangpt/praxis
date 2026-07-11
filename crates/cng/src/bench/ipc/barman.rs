//! Clean-room barman generator (PROJ-711): two hands, shot glasses, and
//! ingredient dispensers — grasp a clean shot, fill it from a dispenser,
//! set it down to free the hand — modeled from first principles (a STRIPS
//! reduction of the bartending scenario; no shaker/cocktail chemistry, no
//! IPC file consulted).
//!
//! Size law: `size` = number of shots to fill (1..=[`MAX_SIZE`]). Each shot
//! needs grasp + fill (2 steps); a third shot additionally needs one
//! `leave` to free a hand → plans of 2, 4, 7 steps. Grounding: 3-parameter
//! `fill` over `4 + size ≤ 7` objects → ≤ 343 + 2·49 ground actions.
//!
//! The seed chooses each shot's ingredient (gin/rum);
//! `SwappedGoalIdentities` flips every chosen ingredient — each goal atom
//! changes textually by construction and solvability is preserved by
//! ingredient symmetry.

use crate::powl::CngRefusal;

use super::{atom, draw, ActionSpec, DomainSpec, IpcProblem, IpcVariant, ProblemSpec};

/// Maximum shots-to-fill count (plan length 7 at size 3).
pub const MAX_SIZE: u8 = 3;

/// Domain-namespacing salt ("bar" in ASCII).
const SALT: u64 = 0x0000_0000_6261_72;

/// The three barman schemas as typed specs.
///
/// # Complexity
/// O(1).
fn domain_spec() -> DomainSpec {
    DomainSpec {
        name: "cng-barman".to_string(),
        actions: vec![
            ActionSpec {
                name: "grasp".to_string(),
                params: vec!["?h".to_string(), "?c".to_string()],
                pre: vec![
                    atom("hand-free", &["?h"]),
                    atom("on-table", &["?c"]),
                    atom("is-container", &["?c"]),
                ],
                add: vec![atom("holding", &["?h", "?c"])],
                del: vec![atom("hand-free", &["?h"]), atom("on-table", &["?c"])],
            },
            ActionSpec {
                name: "fill".to_string(),
                params: vec!["?h".to_string(), "?c".to_string(), "?i".to_string()],
                pre: vec![
                    atom("holding", &["?h", "?c"]),
                    atom("clean", &["?c"]),
                    atom("is-ingredient", &["?i"]),
                ],
                add: vec![atom("contains", &["?c", "?i"])],
                del: vec![atom("clean", &["?c"])],
            },
            ActionSpec {
                name: "leave".to_string(),
                params: vec!["?h".to_string(), "?c".to_string()],
                pre: vec![atom("holding", &["?h", "?c"])],
                add: vec![atom("on-table", &["?c"]), atom("hand-free", &["?h"])],
                del: vec![atom("holding", &["?h", "?c"])],
            },
        ],
    }
}

/// Generates the `(seed, size)` barman problem.
///
/// # Errors
/// `CNG_R05` for `size` outside `1..=MAX_SIZE`; template IO refusals.
///
/// # Complexity
/// O(size) spec construction + render cost.
pub fn generate(seed: u64, size: u8, variant: IpcVariant) -> Result<IpcProblem, CngRefusal> {
    if !(1..=MAX_SIZE).contains(&size) {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "barman size {size} outside 1..={MAX_SIZE}"
        )));
    }
    let n = size as usize;
    let shots: Vec<String> = (1..=n).map(|i| format!("shot{i}")).collect();

    let mut objects = vec![
        "hand-left".to_string(),
        "hand-right".to_string(),
        "gin".to_string(),
        "rum".to_string(),
    ];
    objects.extend(shots.iter().cloned());

    let mut init = vec![
        atom("hand-free", &["hand-left"]),
        atom("hand-free", &["hand-right"]),
        atom("is-ingredient", &["gin"]),
        atom("is-ingredient", &["rum"]),
    ];
    for shot in &shots {
        init.push(atom("is-container", &[shot.as_str()]));
        init.push(atom("on-table", &[shot.as_str()]));
        init.push(atom("clean", &[shot.as_str()]));
    }

    // Per-shot ingredient from the seeded stream; the variant flips each
    // choice (guaranteed textual goal change, ingredient symmetry). O(size).
    let bits = draw(seed, SALT);
    let mut goal = Vec::with_capacity(n);
    for (i, shot) in shots.iter().enumerate() {
        let canonical_gin = (bits >> i) & 1 == 0;
        let use_gin = match variant {
            IpcVariant::Canonical => canonical_gin,
            IpcVariant::SwappedGoalIdentities => !canonical_gin,
        };
        goal.push(atom(
            "contains",
            &[shot.as_str(), if use_gin { "gin" } else { "rum" }],
        ));
    }

    let problem = ProblemSpec {
        name: format!("cng-barman-s{seed}-n{size}"),
        domain: "cng-barman".to_string(),
        objects,
        init,
        goal,
    };
    super::render_ipc(&domain_spec(), &problem, seed, size, variant)
}
