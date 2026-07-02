//! Rank the 3-account fixture under the default authored objective and
//! print the result. Run from inside the crate:
//!
//! ```sh
//! cargo run --example rank_fixture
//! ```

use praxis_proposer::domain::Account;
use praxis_proposer::{ObjectiveFunction, Proposer, RevenueState, Stage};

fn main() {
    let state = RevenueState {
        accounts: vec![
            Account {
                id: "acct-1".into(),
                stage: Stage::Procurement,
                amount_cents: 2_500_000, // $25,000.00
                security_review_done: true,
                legal_approved: true,
                exec_sponsor: true,
                days_in_stage: 12,
            },
            Account {
                id: "acct-2".into(),
                stage: Stage::Qualified,
                amount_cents: 800_000, // $8,000.00
                security_review_done: true,
                legal_approved: false,
                exec_sponsor: true,
                days_in_stage: 45,
            },
            Account {
                id: "acct-3".into(),
                stage: Stage::Lead,
                amount_cents: 150_000, // $1,500.00
                security_review_done: false,
                legal_approved: false,
                exec_sponsor: false,
                days_in_stage: 120,
            },
        ],
    };

    let objective_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("revenue_objective.json");
    let objective = ObjectiveFunction::from_path(&objective_path).expect("objective loads");
    let ranked = Proposer::new(objective).propose(&state);

    println!("ranked proposals (observations, not authorities):");
    for (i, p) in ranked.iter().enumerate() {
        println!("#{:<2} score={:<12} {}", i + 1, p.score, p.goal_description);
        println!("    pddl_goal: {}", p.pddl_goal());
        println!("    hash:      {}", p.proposal_hash);
        for line in &p.rationale {
            println!("      | {line}");
        }
    }
}
