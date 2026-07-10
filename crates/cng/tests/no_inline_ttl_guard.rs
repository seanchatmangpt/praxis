//! Static artifact-boundary guard: no inline Turtle or PDDL payloads may
//! exist in this crate's Rust source or tests. Turtle/PDDL enters only from
//! `.ttl` files and leaves only as serializer output. This test fails if a
//! forbidden payload marker appears in any `.rs` file of the crate.

use std::fs;
use std::path::{Path, PathBuf};

/// Collect every `.rs` file under `dir`, recursively.
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_inline_turtle_or_pddl_in_rust_sources() {
    // Needles are assembled from parts so this guard file itself can never
    // match its own patterns.
    let turtle_needle = format!("@{}", "prefix");
    let pddl_domain_needle = format!("({} ({}", "define", "domain");
    let pddl_problem_needle = format!("({} ({}", "define", "problem");
    let literal_needle = format!("ceng:{} \"\"\"", "pddlDomain");

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_files(&crate_root.join("src"), &mut files);
    rs_files(&crate_root.join("tests"), &mut files);
    assert!(
        files.len() >= 5,
        "guard must see the crate sources; found only {} .rs files",
        files.len()
    );

    let mut violations = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", file.display()));
        // The lawful Turtle-prefix emitters are the serializer and the
        // benchmark corpus generator ("normal serializer output" exemption):
        // both WRITE prefixes into generated artifacts that are only ever
        // consumed back through oxigraph. PDDL payload markers stay
        // forbidden even there.
        let is_serializer = file.ends_with(Path::new("src").join("powl.rs"))
            || file.ends_with(Path::new("src").join("bench.rs"));
        for (needle, kind) in [
            (&turtle_needle, "inline Turtle prefix"),
            (&pddl_domain_needle, "inline PDDL domain"),
            (&pddl_problem_needle, "inline PDDL problem"),
            (&literal_needle, "inline PDDL-in-Turtle literal"),
        ] {
            if is_serializer && needle == &turtle_needle {
                continue;
            }
            if content.contains(needle.as_str()) {
                violations.push(format!("{}: {kind}", file.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "artifact boundary violated — Turtle/PDDL payloads must live in .ttl \
         files, not Rust sources:\n{}",
        violations.join("\n")
    );
}
