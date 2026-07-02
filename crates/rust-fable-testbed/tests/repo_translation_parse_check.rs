//! Scratch verification that `repo_translation_001.ttl` parses via `load_task`.
use std::path::PathBuf;

use rust_fable_testbed::spec::{load_task, TaskType};

#[test]
fn repo_translation_001_loads() {
    let ttl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tasks/repo_translation_001.ttl");
    let task = load_task(&ttl_path).expect("load_task should succeed for repo_translation_001");
    assert_eq!(task.id, "repo_translation_001");
    assert_eq!(task.task_type, TaskType::RepoLevelTranslation);
    assert_eq!(task.difficulty, "hard");
    assert_eq!(task.prompt_sections.len(), 2);
    let user = &task.prompt_sections[1];
    assert_eq!(user.blocks.len(), 6);
    let code_blocks: Vec<_> =
        user.blocks.iter().filter(|b| matches!(b.kind, rust_fable_testbed::spec::PromptBlockKind::Code { .. })).collect();
    assert_eq!(code_blocks.len(), 3);
    assert!(code_blocks[2].content.contains("describe_shape"));
}

#[test]
fn repo_translation_001_target_path_is_describe_rs_not_lib_rs() {
    let ttl_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tasks/repo_translation_001.ttl");
    let task = rust_fable_testbed::spec::load_task(&ttl_path).expect("load_task should succeed");
    assert_eq!(task.target_path, Some(std::path::PathBuf::from("src/describe.rs")));
}
