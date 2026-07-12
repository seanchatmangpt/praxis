//! Regression test for the `--lake-env` auto-detection gap found by the
//! 80/20 sweep: `praxis-l4 verify` used to default `lake_env` to `false`
//! unconditionally, so any Lake-managed corpus (e.g.
//! `tools/paper-factory/lean-lake/`) was kernel-checked via bare `lean`,
//! which fails with "unknown module prefix 'Mathlib'" on any file that
//! `import`s a Lake dependency -- a false `KernelRejected`, not a real
//! proof defect. `detect_lake_env` now drives the default from whether
//! `root` itself owns a `lakefile.lean`/`lakefile.toml`, matching where
//! `crate::cli::init` scaffolds one.

use camino::Utf8PathBuf;
use praxis_lean::lean::detect_lake_env;
use tempfile::tempdir;

#[test]
fn detects_lakefile_lean() {
    let dir = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    std::fs::write(root.join("lakefile.lean"), "-- lake package\n").unwrap();
    assert!(detect_lake_env(&root));
}

#[test]
fn detects_lakefile_toml() {
    let dir = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    std::fs::write(root.join("lakefile.toml"), "name = \"praxis\"\n").unwrap();
    assert!(detect_lake_env(&root));
}

#[test]
fn bare_root_without_lakefile_stays_bare_lean() {
    let dir = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    // No lakefile written -- mirrors a bare-`lean`-verified corpus such as
    // `tools/paper-factory/lean-pilot/`.
    assert!(!detect_lake_env(&root));
}

#[test]
fn subdirectory_of_a_lake_root_is_not_itself_detected() {
    // `detect_lake_env` only looks at `root` directly, not any ancestor --
    // matches `praxis-l4 verify`'s contract that `root` must be the Lake
    // package directory itself (where `lakefile.lean` lives), not an
    // arbitrary subdirectory under it.
    let dir = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    std::fs::write(root.join("lakefile.lean"), "-- lake package\n").unwrap();
    let sub = root.join("Praxis").join("Corpus");
    std::fs::create_dir_all(&sub).unwrap();
    assert!(!detect_lake_env(&sub));
}
