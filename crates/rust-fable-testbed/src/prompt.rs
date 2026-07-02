//! Compile a [`crate::spec::TaskSpec`] into a deterministic, hash-addressed prompt.
//!
//! Bridges [`crate::spec::task_to_prompt_ir`] (our own triple-walked IR, since ggen's
//! `PromptIR::from_construct`/`from_store` are stubs — see `spec.rs` module docs) into
//! ggen's real emitter/validator/hash path via
//! `ggen_core::prompt_mfg::PromptCompiler::compile_from_ir`.

use ggen_core::prompt_mfg::{CompiledPrompt, PromptCompiler};

use crate::spec::{task_to_prompt_ir, TaskSpec};
use crate::{Error, Result};

/// Compile `task`'s prompt sections into a [`CompiledPrompt`] (deterministic text +
/// BLAKE3-family hash + the [`ggen_core::prompt_mfg::ir::PromptIR`] it was built from).
///
/// # Errors
///
/// Returns [`Error::Prompt`] if the compiler fails to initialize (malformed embedded
/// Tera templates — a build-time invariant, not expected at runtime) or if the IR
/// fails validation/emission (e.g. an empty `system`/`user` section).
pub fn compile_task_prompt(task: &TaskSpec) -> Result<CompiledPrompt> {
    let ir = task_to_prompt_ir(task);
    let compiler = PromptCompiler::new().map_err(Error::Prompt)?;
    compiler.compile_from_ir(ir).map_err(Error::Prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::load_task;
    use std::path::PathBuf;

    fn write_temp_task(dir: &std::path::Path) -> PathBuf {
        let fixture_rs = dir.join("snippet.rs");
        std::fs::write(&fixture_rs, "fn add(a: i32, b: i32) -> i32 { a + b }\n")
            .expect("write fixture snippet");
        let ttl = format!(
            r#"@prefix tb: <{tb}> .
@prefix ggen: <{ggen}> .

tb:t1 a tb:Task ;
    tb:id "t1" ;
    tb:taskType tb:FunctionLevelBugfix ;
    tb:difficulty "easy" ;
    tb:model "claude-opus-4-8" ;
    tb:description "test task" ;
    tb:promptSection [ a ggen:Section ; ggen:role "system" ;
        ggen:block [ a ggen:Instruction ; ggen:text "You are a careful Rust engineer." ] ] ;
    tb:promptSection [ a ggen:Section ; ggen:role "user" ;
        ggen:block [ a ggen:Code ; ggen:lang "rust" ; ggen:path "snippet.rs" ] ;
        ggen:block [ a ggen:Instruction ; ggen:text "Fix it." ] ] ;
    tb:fixture "fixtures/t1/" ;
    tb:expectedSteps ( tb:Build tb:Test ) ;
    tb:passCriteria [ tb:cargoTest "cargo test" ; tb:clippyDenyWarnings true ] .
"#,
            tb = crate::spec::TB_NS,
            ggen = crate::spec::GGEN_NS,
        );
        let ttl_path = dir.join("t1.ttl");
        std::fs::write(&ttl_path, ttl).expect("write ttl");
        ttl_path
    }

    #[test]
    fn compiles_task_prompt_deterministically() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ttl_path = write_temp_task(dir.path());
        let task = load_task(&ttl_path).expect("load_task");

        let compiled_a = compile_task_prompt(&task).expect("compile a");
        let compiled_b = compile_task_prompt(&task).expect("compile b");

        assert_eq!(compiled_a.hash(), compiled_b.hash());
        assert_eq!(compiled_a.content(), compiled_b.content());
        assert!(!compiled_a.content().is_empty());
    }
}
