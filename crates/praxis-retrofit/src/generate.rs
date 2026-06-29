//! Generate retrofit artifacts and plans

use std::path::Path;

use crate::{models::*, templates, PraxisSpec, Result};

pub fn generate_retrofit_plan(
    repo_path: &Path,
    phase: RetrofitPhase,
    spec: &PraxisSpec,
) -> Result<RetrofitPlan> {
    let metadata = RepositoryMetadata {
        path: repo_path.to_path_buf(),
        name: repo_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        workspace_root: repo_path.to_path_buf(),
        crate_count: 1,
        has_workspace: true,
    };

    let actions = match phase {
        RetrofitPhase::Phase1Lints => generate_lints_actions(repo_path, spec)?,
        RetrofitPhase::Phase2Deps => vec![],
        RetrofitPhase::Phase3Justfile => vec![generate_justfile_action(repo_path)?],
        RetrofitPhase::Phase4Typos => vec![generate_typos_action(repo_path)?],
        RetrofitPhase::Phase5Docs => vec![],
    };

    let commit_msg =
        templates::commit_message(&format!("{:?}", phase), &metadata.name, actions.len());

    Ok(RetrofitPlan {
        repository: metadata,
        actions,
        phase,
        estimated_risk: RiskLevel::Low,
        commit_message: commit_msg,
    })
}

fn generate_lints_actions(repo_path: &Path, spec: &PraxisSpec) -> Result<Vec<RetrofitAction>> {
    let cargo_toml_path = repo_path.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Ok(vec![]);
    }

    let mut actions = vec![];

    let lints_block = templates::cargo_lints_template(spec);
    actions.push(RetrofitAction {
        action_type: RetrofitActionType::Update,
        file_path: cargo_toml_path,
        content: lints_block,
        description: "Add workspace [lints] configuration".to_string(),
    });

    Ok(actions)
}

fn generate_typos_action(repo_path: &Path) -> Result<RetrofitAction> {
    Ok(RetrofitAction {
        action_type: RetrofitActionType::Create,
        file_path: repo_path.join("typos.toml"),
        content: templates::typos_toml_template(),
        description: "Add typos.toml spell-check configuration".to_string(),
    })
}

fn generate_justfile_action(repo_path: &Path) -> Result<RetrofitAction> {
    let crate_name = repo_path.file_name().unwrap_or_default().to_string_lossy().to_string();

    Ok(RetrofitAction {
        action_type: RetrofitActionType::Create,
        file_path: repo_path.join("justfile"),
        content: templates::justfile_template(&crate_name),
        description: "Add Justfile with standard praxis tasks".to_string(),
    })
}
