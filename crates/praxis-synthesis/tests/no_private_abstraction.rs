//! PROJ-302 — the no-private-abstraction gate.
//!
//! Textual scan (same style as `no_llm_runtime.rs`): reads each closed-world
//! predicate table straight out of `src/` and asserts every predicate,
//! qualified by its namespace prefix, has a corresponding row in
//! `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`. A new private predicate added
//! to any table without a doc entry fails this test — the gate is a
//! cross-check, not aspirational prose.

use std::fs;
use std::path::Path;

/// Extract the quoted string literals of a `const NAME: [&str; N] = [ ... ];`
/// declaration by scanning forward from `const_name` to the first `];`.
fn extract_str_array(src: &str, const_name: &str) -> Vec<String> {
    let start = src
        .find(const_name)
        .unwrap_or_else(|| panic!("'{const_name}' not found in source"));
    let rest = &src[start..];
    let end = rest
        .find("];")
        .unwrap_or_else(|| panic!("no closing '];' found after '{const_name}'"));
    let body = &rest[..end];
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(open) = body[i..].find('"') {
        let start = i + open + 1;
        let Some(close) = body[start..].find('"') else {
            break;
        };
        out.push(body[start..start + close].to_string());
        i = start + close + 1;
    }
    out
}

fn read_src(manifest_dir: &str, file: &str) -> String {
    fs::read_to_string(Path::new(manifest_dir).join("src").join(file))
        .unwrap_or_else(|e| panic!("reading src/{file}: {e}"))
}

/// Parse every backtick-quoted `prefix:local` token out of the mapping doc.
fn mapped_predicates(doc: &str) -> Vec<String> {
    const PREFIXES: [&str; 4] = ["wf:", "hook:", "prayer-kernel:", "agent:"];
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(open) = doc[i..].find('`') {
        let start = i + open + 1;
        let Some(close) = doc[start..].find('`') else {
            break;
        };
        let token = &doc[start..start + close];
        if PREFIXES.iter().any(|p| token.starts_with(p)) && !token.contains(char::is_whitespace) {
            out.push(token.to_string());
        }
        i = start + close + 1;
    }
    out
}

#[test]
fn every_private_predicate_has_a_public_ontology_mapping_entry() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let doc_path = Path::new(manifest_dir).join("../../docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md");
    let doc = fs::read_to_string(&doc_path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", doc_path.display()));
    let mapped = mapped_predicates(&doc);

    let tables: [(&str, &str, &str); 4] = [
        ("wf:", "graph.rs", "WF_PREDICATES"),
        ("hook:", "hooks.rs", "HOOK_PREDICATES"),
        ("prayer-kernel:", "kernel.rs", "KERNEL_PREDICATES"),
        ("agent:", "agent_registry.rs", "AGENT_PREDICATES"),
    ];

    for (prefix, file, const_name) in tables {
        let src = read_src(manifest_dir, file);
        let locals = extract_str_array(&src, const_name);
        assert!(
            !locals.is_empty(),
            "expected at least one predicate in {const_name}"
        );
        for local in locals {
            // `wf:a`, `wf:kind`, etc. are single-letter/bareword locals that
            // also occur as substrings elsewhere in the doc's prose; require
            // an exact qualified token match, not a substring search.
            let qualified = format!("{prefix}{local}");
            assert!(
                mapped.contains(&qualified),
                "predicate '{qualified}' (from {file}::{const_name}) has no row in \
                 docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md — add one before landing this change"
            );
        }
    }
}
