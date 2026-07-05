use praxis_lean::{AuditPolicy, NoSorryAudit};
use tempfile::tempdir;

#[test]
fn detects_sorry_and_unauthorized_axiom() {
    let dir = tempdir().unwrap();
    let file = camino::Utf8PathBuf::from_path_buf(dir.path().join("Bad.lean")).unwrap();
    std::fs::write(
        &file,
        r#"
axiom bad_axiom : True
theorem bad : True := by
  sorry
"#,
    )
    .unwrap();

    let audit = NoSorryAudit::new(AuditPolicy::default());
    let findings = audit.audit_file(&file).unwrap();
    assert!(findings.iter().any(|f| f.kind == "axiom"));
    assert!(findings.iter().any(|f| f.kind == "sorry"));
}
