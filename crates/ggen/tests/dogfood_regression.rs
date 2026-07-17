//! C3 — Dogfood regression: the OLD installed ggen (26.7.x, the generator
//! that produces `crates/ggen/src/verbs/*.rs` from `schema/praxis.ttl` +
//! root `ggen.toml` + `templates/crates/ggen/*`) must still regenerate the
//! four route files byte-for-byte.
//!
//! The test copies the needed inputs into a TempDir (the real working tree
//! is never mutated), runs the OLD binary there, and diffs the regenerated
//! route files against the current committed ones.
//!
//! Honest caveat on the assertion: the OLD binary's `sync --dry-run true`
//! reports `"would create"` with `size_bytes: 0` for every output even when
//! the files exist and are identical — it does not render in dry-run mode —
//! so a dry-run "no pending changes" assertion is impossible. Instead we run
//! a REAL sync inside the TempDir and assert byte-equality of the four
//! regenerated route files against the repo's current files. This is the
//! stronger check anyway: it fails if anyone breaks
//! `templates/crates/ggen/*.tera` or the CliCommand instances in
//! `schema/praxis.ttl`.

use std::path::{Path, PathBuf};
use std::process::Command;

const ROUTE_FILES: [&str; 4] = [
    "crates/ggen/src/verbs/mod.rs",
    "crates/ggen/src/verbs/sync.rs",
    "crates/ggen/src/verbs/graph.rs",
    "crates/ggen/src/verbs/receipt.rs",
];

/// Walk up from CARGO_MANIFEST_DIR until a directory containing
/// `schema/praxis.ttl` is found.
fn find_praxis_root() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("schema/praxis.ttl").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("mkdir dst");
    for entry in std::fs::read_dir(src).expect("read_dir src") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copy file");
        }
    }
}

#[test]
fn old_ggen_regenerates_route_files_byte_identically() {
    let Some(root) = find_praxis_root() else {
        eprintln!("SKIP: praxis repo root (schema/praxis.ttl) not found above CARGO_MANIFEST_DIR");
        return;
    };

    // Skip gracefully if the OLD installed generator is absent (CI).
    match Command::new("ggen").arg("--version").output() {
        Ok(out) if out.status.success() => {}
        _ => {
            eprintln!("SKIP: old `ggen` binary not on PATH; dogfood regression not run");
            return;
        }
    }

    // Stage a temp copy of exactly the inputs the generator needs. The real
    // working tree is NEVER touched.
    let temp = tempfile::TempDir::new().expect("tempdir");
    let stage = temp.path();
    std::fs::copy(root.join("ggen.toml"), stage.join("ggen.toml")).expect("copy ggen.toml");
    copy_dir_recursive(&root.join("schema"), &stage.join("schema"));
    copy_dir_recursive(&root.join("templates"), &stage.join("templates"));
    let verbs_dst = stage.join("crates/ggen/src/verbs");
    std::fs::create_dir_all(&verbs_dst).expect("mkdir verbs");
    for entry in std::fs::read_dir(root.join("crates/ggen/src/verbs")).expect("read verbs") {
        let entry = entry.expect("verbs entry");
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            std::fs::copy(&path, verbs_dst.join(entry.file_name())).expect("copy verb file");
        }
    }

    // Sanity: dry-run must at least parse the manifest and exit 0 with the
    // OLD binary's Option<bool> flag syntax.
    let dry = Command::new("ggen")
        .args(["sync", "--dry-run", "true"])
        .current_dir(stage)
        .output()
        .expect("run old ggen dry-run");
    assert!(
        dry.status.success(),
        "old ggen dry-run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&dry.stdout),
        String::from_utf8_lossy(&dry.stderr)
    );

    // Real sync inside the temp copy: regenerates every output there.
    let out = Command::new("ggen")
        .arg("sync")
        .current_dir(stage)
        .output()
        .expect("run old ggen sync");
    assert!(
        out.status.success(),
        "old ggen sync failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // The four generated route files must be byte-identical to the repo's
    // current committed files.
    for rel in ROUTE_FILES {
        let regenerated = std::fs::read(stage.join(rel))
            .unwrap_or_else(|e| panic!("regenerated {rel} missing: {e}"));
        let current = std::fs::read(root.join(rel))
            .unwrap_or_else(|e| panic!("current {rel} unreadable: {e}"));
        assert_eq!(
            regenerated, current,
            "dogfood drift: `{rel}` regenerated by the old ggen differs from the \
             committed file — templates/crates/ggen/*.tera or schema/praxis.ttl \
             no longer reproduce the checked-in route files"
        );
    }
}
