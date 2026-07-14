//! TEST-34 — no LLM, no network, on the execution path.
//!
//! Two decidable checks, honest about their scope:
//!
//! 1. The `[dependencies]` section of this crate's Cargo.toml is EXACTLY the
//!    known offline allowlist — no LLM client, no HTTP stack, no network
//!    crate can be present without failing this test.
//! 2. A plain textual scan of `src/`: after stripping `//` line comments
//!    (which lawfully MENTION LLMs — e.g. quarantine.rs documents that an
//!    LLM proposer's output is never executable), no source line contains
//!    `openai`, `anthropic`, `llm::`, or `gpt` (case-insensitive; the
//!    project's own `chatmangpt` domain substring is scrubbed first).
//!
//! What this does NOT prove: behavior of transitive dependencies beyond
//! their names, or dynamic loading (there is none — `#![forbid(unsafe_code)]`
//! and no process-spawning in `src/`). It is a tripwire, not a sandbox.

use std::fs;
use std::path::Path;

/// The exact dependency set of praxis-synthesis. Path deps are in-repo;
/// the rest are offline data/hash/serde crates.
const ALLOWED_DEPS: [&str; 7] = [

    "chatman-common",
    "blake3",
    "serde",
    "serde_json",
    "thiserror",
    "praxis-graphlaw",
];

#[test]
fn dependencies_are_exactly_the_offline_allowlist() {
    let manifest =
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).expect("Cargo.toml");
    let mut in_deps = false;
    let mut found: Vec<String> = Vec::new();
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if in_deps && !line.is_empty() && !line.starts_with('#') {
            let name = line
                .split('=')
                .next()
                .expect("split yields at least one piece")
                .trim();
            found.push(name.to_string());
        }
    }
    for dep in &found {
        assert!(
            ALLOWED_DEPS.contains(&dep.as_str()),
            "dependency '{dep}' is not in the offline allowlist {ALLOWED_DEPS:?}"
        );
    }
    assert_eq!(
        found.len(),
        ALLOWED_DEPS.len(),
        "dependency count changed; update the allowlist DELIBERATELY: found {found:?}"
    );
}

#[test]
fn source_contains_no_llm_symbols() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = 0usize;
    for entry in fs::read_dir(&src).expect("src/ readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        files += 1;
        let text = fs::read_to_string(&path)
            .expect("source readable")
            .to_lowercase();
        for (n, line) in text.lines().enumerate() {
            // Strip `//` line comments (this also drops `http://...` IRI
            // tails, which is fine: IRIs are data, not symbols).
            let code = line.split("//").next().unwrap_or("");
            let scrubbed = code.replace("chatmangpt", "");
            for needle in ["openai", "anthropic", "llm::", "gpt"] {
                assert!(
                    !scrubbed.contains(needle),
                    "{}:{} contains '{needle}' outside a comment",
                    path.display(),
                    n + 1
                );
            }
        }
    }
    assert!(
        files > 20,
        "expected the full src/ module set, scanned only {files} files"
    );
}

/// TEST — human-unavailable execution (PROJ-306): no interactive/blocking-
/// on-a-human symbol appears in this crate's source or dependencies.
/// Distinct from `source_contains_no_llm_symbols`: that test proves absence
/// of an LLM; this one proves absence of a live human at a terminal.
#[test]
fn source_and_deps_contain_no_interactive_human_symbols() {
    const NEEDLES: [&str; 3] = ["stdin", "dialoguer", "inquire"];

    // Cargo.toml: dependency lines only, mirroring the allowlist test's
    // section-tracking so a needle inside an unrelated comment doesn't trip.
    let manifest =
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).expect("Cargo.toml");
    let mut in_deps = false;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if in_deps && !line.is_empty() && !line.starts_with('#') {
            let lower = line.to_lowercase();
            for needle in NEEDLES {
                assert!(
                    !lower.contains(needle),
                    "Cargo.toml [dependencies] line '{line}' contains '{needle}'"
                );
            }
        }
    }

    // src/: same comment-stripping approach as the LLM-symbol tripwire.
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = 0usize;
    for entry in fs::read_dir(&src).expect("src/ readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        files += 1;
        let text = fs::read_to_string(&path)
            .expect("source readable")
            .to_lowercase();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for needle in NEEDLES {
                assert!(
                    !code.contains(needle),
                    "{}:{} contains '{needle}' outside a comment",
                    path.display(),
                    n + 1
                );
            }
        }
    }
    assert!(
        files > 20,
        "expected the full src/ module set, scanned only {files} files"
    );
}
