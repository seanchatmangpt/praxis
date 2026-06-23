use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

fn get_bin_path(name: &str) -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dir = manifest_dir.parent().unwrap().join("target/debug");
    let bin_path = target_dir.join(name);
    assert!(bin_path.exists(), "Binary {} not found at {:?}", name, bin_path);
    bin_path
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn setup_test_project(src: &Path, dst: &Path) {
    if dst.exists() {
        let _ = std::fs::remove_dir_all(dst);
    }
    std::fs::create_dir_all(dst).unwrap();

    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "tests" || name_str == "target" || name_str == ".praxis" {
            continue;
        }
        let ty = entry.file_type().unwrap();
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.join(name)).unwrap();
        } else {
            std::fs::copy(entry.path(), dst.join(name)).unwrap();
        }
    }

    // Rewrite Cargo.toml to use absolute path for chatman-common and append [workspace]
    let cargo_toml_path = dst.join("Cargo.toml");
    let mut cargo_toml = std::fs::read_to_string(&cargo_toml_path).unwrap();

    cargo_toml = cargo_toml.replace(
        "path = \"../../crates/chatman-common\"",
        "path = \"/Users/sac/praxis/crates/chatman-common\"",
    );

    cargo_toml.push_str("\n[workspace]\n");
    std::fs::write(&cargo_toml_path, cargo_toml).unwrap();
}

#[test]
fn test_reconciler_restores_drift() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    let template_dir = Path::new("/Users/sac/praxis/template");

    // Copy reference files initially
    let files = vec!["rustfmt.toml", "deny.toml", ".editorconfig"];
    for f in &files {
        let src = template_dir.join(f);
        if src.exists() {
            let dst = project_dir.join(f);
            std::fs::copy(&src, &dst).unwrap();
        }
    }

    // Spawn reconciler
    let reconciler_bin = get_bin_path("praxis-reconciler");
    let mut child = Command::new(reconciler_bin)
        .arg("--project")
        .arg(&project_dir)
        .arg("--template")
        .arg(template_dir)
        .spawn()
        .unwrap();

    // Let it initialize
    std::thread::sleep(Duration::from_millis(500));

    // 1. Modify rustfmt.toml
    let rustfmt_path = project_dir.join("rustfmt.toml");
    std::fs::write(&rustfmt_path, "invalid content indeed").unwrap();

    // Wait for restoration
    std::thread::sleep(Duration::from_secs(2));

    let restored_content = std::fs::read_to_string(&rustfmt_path).unwrap();
    let original_content = std::fs::read_to_string(template_dir.join("rustfmt.toml")).unwrap();
    assert_eq!(restored_content, original_content, "rustfmt.toml was not restored");

    // 2. Delete deny.toml
    let deny_path = project_dir.join("deny.toml");
    std::fs::remove_file(&deny_path).unwrap();

    // Wait for restoration
    std::thread::sleep(Duration::from_secs(2));

    assert!(deny_path.exists(), "deny.toml was not restored");
    let restored_deny = std::fs::read_to_string(&deny_path).unwrap();
    let original_deny = std::fs::read_to_string(template_dir.join("deny.toml")).unwrap();
    assert_eq!(restored_deny, original_deny, "deny.toml content mismatch after restore");

    // Cleanup process
    let _ = child.kill();
}

#[test]
fn test_guard_check_and_verify() {
    let playground_dir = Path::new("/Users/sac/praxis/playground");
    let temp_dir = tempfile::tempdir().unwrap();
    let test_project_dir = temp_dir.path().join("project");

    setup_test_project(&playground_dir.join("my-sample-project"), &test_project_dir);

    let guard_bin = get_bin_path("praxis-guard");
    let receipt_path = test_project_dir.join("receipt.json");

    // 1. Run praxis-guard check (should succeed)
    let output = Command::new(&guard_bin)
        .arg("check")
        .arg("--project")
        .arg(&test_project_dir)
        .arg("--output")
        .arg(&receipt_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "check failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(receipt_path.exists(), "receipt.json was not created");

    // 2. Run praxis-guard verify (should succeed)
    let output_verify = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();

    assert!(
        output_verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&output_verify.stderr)
    );

    // 3. Tamper with a source file in test_project_dir/src
    let lib_path = test_project_dir.join("src/lib.rs");
    let mut lib_content = std::fs::read_to_string(&lib_path).unwrap();
    lib_content.push_str("\n// Tamper");
    std::fs::write(&lib_path, lib_content).unwrap();

    // 4. Verify should now FAIL
    let output_verify_tampered = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();

    assert!(!output_verify_tampered.status.success(), "verify succeeded on tampered files");
}

#[test]
fn test_guard_fails_on_todo_stub() {
    let playground_dir = Path::new("/Users/sac/praxis/playground");
    let temp_dir = tempfile::tempdir().unwrap();
    let test_project_dir = temp_dir.path().join("project");

    setup_test_project(&playground_dir.join("my-sample-project"), &test_project_dir);

    // Add a todo! stub inside a src file
    let lib_path = test_project_dir.join("src/lib.rs");
    let mut lib_content = std::fs::read_to_string(&lib_path).unwrap();
    let stub = format!("{}!", "todo");
    lib_content.push_str(&format!("\nfn dummy() {{ {}; }}\n", stub));
    std::fs::write(&lib_path, lib_content).unwrap();

    let guard_bin = get_bin_path("praxis-guard");
    let receipt_path = test_project_dir.join("receipt.json");

    // Run check - should fail
    let output = Command::new(&guard_bin)
        .arg("check")
        .arg("--project")
        .arg(&test_project_dir)
        .arg("--output")
        .arg(&receipt_path)
        .output()
        .unwrap();

    // If it failed because of the stub, stderr should mention HOLLOW or todo!
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "check succeeded despite todo! stub");
    assert!(
        stderr.contains("HOLLOW") || stderr.contains("todo!"),
        "failed for wrong reason: {}",
        stderr
    );
}

#[test]
fn test_guard_fails_on_structural_non_conformance() {
    let playground_dir = Path::new("/Users/sac/praxis/playground");
    let temp_dir = tempfile::tempdir().unwrap();
    let test_project_dir = temp_dir.path().join("project");

    setup_test_project(&playground_dir.join("my-sample-project"), &test_project_dir);

    // Modify src/types.rs to remove "PhantomData"
    let types_path = test_project_dir.join("src/types.rs");
    let types_content = std::fs::read_to_string(&types_path).unwrap();
    let tampered_content = types_content.replace("PhantomData", "Phantom_Data");
    std::fs::write(&types_path, tampered_content).unwrap();

    let guard_bin = get_bin_path("praxis-guard");
    let receipt_path = test_project_dir.join("receipt.json");

    // Run check - should fail
    let output = Command::new(&guard_bin)
        .arg("check")
        .arg("--project")
        .arg(&test_project_dir)
        .arg("--output")
        .arg(&receipt_path)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "check succeeded despite missing PhantomData");
    assert!(
        stderr.contains("Structural conformance failed") || stderr.contains("PhantomData"),
        "failed for wrong reason: {}",
        stderr
    );
}
