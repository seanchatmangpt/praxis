//! Apply retrofit changes to repositories

use std::path::Path;

use crate::{models::*, Result};

pub async fn apply_retrofit(_repo_path: &Path, _plan: &RetrofitPlan) -> Result<Vec<String>> {
    let mut results = vec![];

    for action in &_plan.actions {
        match action.action_type {
            RetrofitActionType::Create => {
                std::fs::write(&action.file_path, &action.content)?;
                results.push(format!("Created: {}", action.file_path.display()));
            }
            RetrofitActionType::Update => {
                std::fs::write(&action.file_path, &action.content)?;
                results.push(format!("Updated: {}", action.file_path.display()));
            }
            RetrofitActionType::Delete => {
                if action.file_path.exists() {
                    std::fs::remove_file(&action.file_path)?;
                    results.push(format!("Deleted: {}", action.file_path.display()));
                }
            }
        }
    }

    Ok(results)
}

pub async fn validate_retrofit(_repo_path: &Path) -> Result<bool> {
    // Placeholder: Validate that retrofit was successful
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::models::*;

    #[tokio::test]
    async fn test_apply_empty_plan_is_noop() {
        let repo = RepositoryMetadata {
            path: PathBuf::from("/tmp"),
            name: "test".to_string(),
            workspace_root: PathBuf::from("/tmp"),
            crate_count: 0,
            has_workspace: false,
        };
        let plan = RetrofitPlan {
            repository: repo,
            actions: vec![],
            phase: RetrofitPhase::Phase1Lints,
            estimated_risk: RiskLevel::Low,
            commit_message: "noop".to_string(),
        };
        let results = apply_retrofit(Path::new("/tmp"), &plan).await.expect("apply");
        assert!(results.is_empty());
    }
}
