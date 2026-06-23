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

fn wait_for_restore(file_path: &Path, original_content: &str, timeout: Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if file_path.exists() {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if content == original_content {
                    return;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("File {:?} was not restored to original content within {:?}", file_path, timeout);
}

// 1. Stress test the active reconciler
#[test]
fn test_reconciler_rapid_loop() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    let template_dir = Path::new("/Users/sac/praxis/template");
    let original_rustfmt = std::fs::read_to_string(template_dir.join("rustfmt.toml")).unwrap();

    // Copy initial
    std::fs::copy(template_dir.join("rustfmt.toml"), project_dir.join("rustfmt.toml")).unwrap();

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

    // Rapid loop: modify and delete multiple times rapidly
    for i in 0..25 {
        let rustfmt_path = project_dir.join("rustfmt.toml");
        if i % 2 == 0 {
            // Delete file
            let _ = std::fs::remove_file(&rustfmt_path);
        } else {
            // Modify file
            std::fs::write(&rustfmt_path, format!("tampered content {}", i)).unwrap();
        }
        // Verify it is restored within 2 seconds
        wait_for_restore(&rustfmt_path, &original_rustfmt, Duration::from_secs(2));
    }

    let _ = child.kill();
}

// Helper to assert check fails after altering code
fn assert_check_fails(project_dir: &Path, file_rel_path: &str, target: &str, replacement: &str) {
    let file_path = project_dir.join(file_rel_path);
    let original_content = std::fs::read_to_string(&file_path).unwrap();
    let altered_content = original_content.replace(target, replacement);
    assert_ne!(original_content, altered_content, "Target string {:?} not found in {:?}", target, file_rel_path);
    std::fs::write(&file_path, altered_content).unwrap();

    let guard_bin = get_bin_path("praxis-guard");
    let receipt_path = project_dir.join("receipt.json");
    let output = Command::new(&guard_bin)
        .arg("check")
        .arg("--project")
        .arg(project_dir)
        .arg("--output")
        .arg(&receipt_path)
        .output()
        .unwrap();

    // Revert alteration immediately to avoid interfering with subsequent steps
    std::fs::write(&file_path, original_content).unwrap();

    assert!(!output.status.success(), "Check succeeded unexpectedly for replacement {:?} of {:?}", replacement, target);
}

// 2. Stress test the compliance guard on invalid/forbidden patterns
#[test]
fn test_guard_invalid_forbidden_patterns() {
    let playground_dir = Path::new("/Users/sac/praxis/playground");
    let temp_dir = tempfile::tempdir().unwrap();
    let test_project_dir = temp_dir.path().join("project");

    setup_test_project(&playground_dir.join("my-sample-project"), &test_project_dir);

    // Verify adding todo!, unimplemented!, // FIXME:, // TODO:, // PLACEHOLDER fails the check in different files/places
    
    // In dod.rs fn run
    assert_check_fails(&test_project_dir, "src/bin/dod.rs", "fn run", "fn run() { todo!(); }\nfn run_old");
    assert_check_fails(&test_project_dir, "src/bin/dod.rs", "fn run", "fn run() { unimplemented!(); }\nfn run_old");
    assert_check_fails(&test_project_dir, "src/bin/dod.rs", "fn run", "fn run() { // FIXME:\n}\nfn run_old");
    assert_check_fails(&test_project_dir, "src/bin/dod.rs", "fn run", "fn run() { // TODO:\n}\nfn run_old");
    assert_check_fails(&test_project_dir, "src/bin/dod.rs", "fn run", "fn run() { // PLACEHOLDER\n}\nfn run_old");

    // In types.rs
    assert_check_fails(&test_project_dir, "src/types.rs", "pub struct Blake3Hash(pub String);", "pub struct Blake3Hash(pub String); // FIXME: verify this");
    assert_check_fails(&test_project_dir, "src/types.rs", "pub struct Blake3Hash(pub String);", "pub struct Blake3Hash(pub String); // PLACEHOLDER");
}

// 3. Stress test structural conformance for ZST markers
#[test]
fn test_guard_structural_conformance_violations() {
    let playground_dir = Path::new("/Users/sac/praxis/playground");
    let temp_dir = tempfile::tempdir().unwrap();
    let test_project_dir = temp_dir.path().join("project");

    setup_test_project(&playground_dir.join("my-sample-project"), &test_project_dir);

    // Alter / delete structural conformance markers and ensure it fails
    // A. PhantomData typestates
    assert_check_fails(&test_project_dir, "src/types.rs", "PhantomData", "Phantom_Data");
    // B. Raw ZST marker
    assert_check_fails(&test_project_dir, "src/types.rs", "pub struct Raw;", "pub struct RawMarker;");
    // C. Validated ZST marker
    assert_check_fails(&test_project_dir, "src/types.rs", "pub struct Validated;", "pub struct ValidatedMarker;");
    // D. Admitted ZST marker
    assert_check_fails(&test_project_dir, "src/types.rs", "pub struct Admitted;", "pub struct AdmittedMarker;");
    // E. Evidence wrapper
    assert_check_fails(&test_project_dir, "src/types.rs", "pub struct Evidence", "pub struct EvidenceWrapper");
    // F. Admit trait
    assert_check_fails(&test_project_dir, "src/types.rs", "pub trait Admit", "pub trait AdmitTrait");
    // G. RulePackServer implementation
    assert_check_fails(&test_project_dir, "src/lsp.rs", "impl RulePackServer for AppLspServer", "impl Rule_Pack_Server for AppLspServer");
}

// 4. Verify altering even a single byte of source file fails verify
#[test]
fn test_guard_byte_alteration_fails_verification() {
    let playground_dir = Path::new("/Users/sac/praxis/playground");
    let temp_dir = tempfile::tempdir().unwrap();
    let test_project_dir = temp_dir.path().join("project");

    setup_test_project(&playground_dir.join("my-sample-project"), &test_project_dir);

    let guard_bin = get_bin_path("praxis-guard");
    let receipt_path = test_project_dir.join("receipt.json");

    // Check should succeed initially and produce receipt
    let output = Command::new(&guard_bin)
        .arg("check")
        .arg("--project")
        .arg(&test_project_dir)
        .arg("--output")
        .arg(&receipt_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "Initial check failed");

    // Verify should succeed
    let output_verify = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();
    assert!(output_verify.status.success(), "Initial verify failed");

    // Alter a single byte (e.g. change a letter in src/bin/dod.rs comment)
    let dod_path = test_project_dir.join("src/bin/dod.rs");
    let mut dod_content = std::fs::read_to_string(&dod_path).unwrap();
    // Replace "Definition of Done verification" with "Definition of Done VerificatioN"
    let tampered = dod_content.replace("Definition of Done verification", "Definition of Done verificatioN");
    assert_ne!(dod_content, tampered);
    std::fs::write(&dod_path, tampered).unwrap();

    // Verify should now fail
    let output_verify_tampered = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();
    assert!(!output_verify_tampered.status.success(), "Verification succeeded despite byte alteration");

    // Revert change
    std::fs::write(&dod_path, dod_content).unwrap();

    // Verify should succeed again
    let output_verify_restored = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();
    assert!(output_verify_restored.status.success(), "Verification failed after restoring the byte");
}

// 5. Verify altering receipt's signature or public key causes verify to fail
#[test]
fn test_guard_receipt_tampering_fails_verification() {
    let playground_dir = Path::new("/Users/sac/praxis/playground");
    let temp_dir = tempfile::tempdir().unwrap();
    let test_project_dir = temp_dir.path().join("project");

    setup_test_project(&playground_dir.join("my-sample-project"), &test_project_dir);

    let guard_bin = get_bin_path("praxis-guard");
    let receipt_path = test_project_dir.join("receipt.json");

    // Check should succeed initially
    let output = Command::new(&guard_bin)
        .arg("check")
        .arg("--project")
        .arg(&test_project_dir)
        .arg("--output")
        .arg(&receipt_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "Initial check failed");

    // Verify should succeed
    let output_verify = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();
    assert!(output_verify.status.success(), "Initial verify failed");

    // Read receipt
    let receipt_content = std::fs::read_to_string(&receipt_path).unwrap();
    let mut receipt_json: serde_json::Value = serde_json::from_str(&receipt_content).unwrap();

    // Test A: Alter signature
    let orig_sig = receipt_json["signature"].as_str().unwrap().to_string();
    // Swap last character
    let mut tampered_sig = orig_sig.clone();
    let last_char = tampered_sig.pop().unwrap();
    let new_last_char = if last_char == '0' { '1' } else { '0' };
    tampered_sig.push(new_last_char);
    receipt_json["signature"] = serde_json::Value::String(tampered_sig);

    std::fs::write(&receipt_path, serde_json::to_string_pretty(&receipt_json).unwrap()).unwrap();
    let output_verify_bad_sig = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();
    assert!(!output_verify_bad_sig.status.success(), "Verification succeeded with tampered signature");

    // Revert signature
    receipt_json["signature"] = serde_json::Value::String(orig_sig);

    // Test B: Alter public key
    let orig_pubkey = receipt_json["data"]["public_key"].as_str().unwrap().to_string();
    let mut tampered_pubkey = orig_pubkey.clone();
    let last_char = tampered_pubkey.pop().unwrap();
    let new_last_char = if last_char == '0' { '1' } else { '0' };
    tampered_pubkey.push(new_last_char);
    receipt_json["data"]["public_key"] = serde_json::Value::String(tampered_pubkey);

    std::fs::write(&receipt_path, serde_json::to_string_pretty(&receipt_json).unwrap()).unwrap();
    let output_verify_bad_pubkey = Command::new(&guard_bin)
        .arg("verify")
        .arg("--receipt")
        .arg(&receipt_path)
        .arg("--project")
        .arg(&test_project_dir)
        .output()
        .unwrap();
    assert!(!output_verify_bad_pubkey.status.success(), "Verification succeeded with tampered public key");
}
