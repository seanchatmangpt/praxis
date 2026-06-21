//! Hollow gate: blocks unimplemented!/todo!/placeholder stubs from reaching CI.
//!
//! Mirrors bcinr's contract-gate and anti-llm-cheat hollow.rs pattern.
//! Exit 0 = clean. Exit 1 = hollow stubs detected.

const BLOCKING: &[(&str, &str)] = &[
    ("unimplemented!()", "HOLLOW-001"),
    ("todo!()",          "HOLLOW-002"),
    ("// TODO:",         "HOLLOW-004"),
    ("// FIXME:",        "HOLLOW-005"),
    ("// PLACEHOLDER",   "HOLLOW-006"),
];

fn main() {
    let mut found = false;
    for entry in walkdir::WalkDir::new("src").into_iter().flatten() {
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let src = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for (line_no, line) in src.lines().enumerate() {
            for (pat, code) in BLOCKING {
                if line.contains(pat) {
                    eprintln!(
                        "[{}] {}:{}: {}",
                        code,
                        entry.path().display(),
                        line_no + 1,
                        line.trim()
                    );
                    found = true;
                }
            }
        }
    }
    if found {
        std::process::exit(1);
    }
}
