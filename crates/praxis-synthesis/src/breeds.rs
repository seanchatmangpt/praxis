//! Compile-checked cognitive breed registry (PROJ-305).
//!
//! Promotes the prose mapping in `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` (PROJ-206)
//! into a const table so it cannot silently drift from the module names it cites. This
//! is documentation, not a new abstraction: no new trait, no new runtime dispatch, no
//! new public type beyond the table itself.
//!
//! Breeds marked "NOT IMPLEMENTED" in the source doc (Translator, Cartographer, Broker,
//! Dachshund, Service, Meta) have no code home and are intentionally absent here rather
//! than forced into a fake mapping.

/// (breed name, module path) — kept in sync with `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md`
/// by the test below, not by convention. `module_path` is the crate-relative module name
/// of an actual `pub mod` declared in `lib.rs`.
pub const BREED_MODULE_MAP: &[(&str, &str)] = &[
    ("guardian", "quarantine"),
    ("detector", "hooks"),
    ("tracker", "firing"),
    ("retriever", "life"),
    ("planner", "ground"),
    ("herding", "dag"),
    ("recorder", "envelope"),
    ("verifier", "firing"),
];

// ---------------------------------------------------------------------------
// wasm4pm fact vocabularies (closed world, PROJ-305 follow-through)
// ---------------------------------------------------------------------------

/// wasm4pm cognition-breed namespace. Closed world: any `compat:` predicate
/// not in the admitted table below is a typed [`crate::Refusal::UnknownPredicate`].
pub const COMPAT_NS: &str = "https://wasm4pm.dev/ns#";

/// wasm4pm process-intelligence namespace. Closed world like [`COMPAT_NS`].
pub const PI_NS: &str = "https://wasm4pm.dev/pi#";

/// Admitted `compat:` predicates (`packs/wasm4pm-facts-pack/ontology.ttl`).
/// `breedStatus` is deliberately ABSENT: status is law-derived (N3 standing
/// rule `ontology/rules/breed_standing.n3`), never asserted in fact files.
pub const COMPAT_PREDICATES: [&str; 5] = [
    "breedId",
    "breedLabel",
    "breedDoc",
    "citation",
    "modulePath",
];

/// Admitted `pi:` predicates. `algorithmStatus`, `measuredFitness`,
/// `piAdmitted`, and `fitnessProvenance` are deliberately ABSENT: those are
/// derived-only in wasm4pm and asserting them here is the refused anti-pattern.
pub const PI_PREDICATES: [&str; 12] = [
    "algorithmId",
    "algorithmLabel",
    "algorithmDoc",
    "citation",
    "outputType",
    "category",
    "speedTier",
    "qualityTier",
    "wasmExport",
    "cliAlias",
    "inputFormat",
    "standing",
];

/// Closed-world admission check for a wasm4pm-namespace predicate IRI.
/// Predicates outside `compat:`/`pi:` pass through (`Ok`), mirroring the
/// `wf:` handling in `graph.rs`: foreign namespaces are someone else's law.
///
/// # Errors
/// [`crate::Refusal::UnknownPredicate`] naming the predicate and subject
/// when the IRI is in `compat:`/`pi:` but its local name is not admitted.
pub fn check_wasm4pm_predicate(predicate: &str, subject: &str) -> Result<(), crate::Refusal> {
    let admitted = if let Some(local) = predicate.strip_prefix(COMPAT_NS) {
        COMPAT_PREDICATES.contains(&local)
    } else if let Some(local) = predicate.strip_prefix(PI_NS) {
        PI_PREDICATES.contains(&local)
    } else {
        true
    };
    if admitted {
        Ok(())
    } else {
        Err(crate::Refusal::UnknownPredicate {
            predicate: predicate.to_string(),
            subject: subject.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{check_wasm4pm_predicate, BREED_MODULE_MAP, COMPAT_NS, PI_NS};

    #[test]
    fn admitted_wasm4pm_predicates_pass() {
        for local in super::COMPAT_PREDICATES {
            check_wasm4pm_predicate(&format!("{COMPAT_NS}{local}"), "compat:Breed_x")
                .expect("admitted compat predicate must pass");
        }
        for local in super::PI_PREDICATES {
            check_wasm4pm_predicate(&format!("{PI_NS}{local}"), "pi:Algo_x")
                .expect("admitted pi predicate must pass");
        }
        // Foreign namespace: not this table's law.
        check_wasm4pm_predicate("http://www.w3.org/2000/01/rdf-schema#label", "s")
            .expect("foreign namespace passes through");
    }

    /// Adversarial: asserting a derived-only status predicate is refused BY
    /// NAME — the wasm4pm alive-gate anti-pattern (asserting what must be
    /// CONSTRUCT-derived) cannot enter the graph.
    #[test]
    fn derived_only_status_predicates_are_refused_by_name() {
        for iri in [
            format!("{COMPAT_NS}breedStatus"),
            format!("{PI_NS}algorithmStatus"),
            format!("{PI_NS}measuredFitness"),
            format!("{COMPAT_NS}totallyMadeUp"),
        ] {
            match check_wasm4pm_predicate(&iri, "compat:Breed_allen_temporal") {
                Err(crate::Refusal::UnknownPredicate { predicate, subject }) => {
                    assert_eq!(predicate, iri);
                    assert_eq!(subject, "compat:Breed_allen_temporal");
                }
                other => panic!("expected UnknownPredicate for {iri}, got {other:?}"),
            }
        }
    }

    /// Consistency gate: BREED_MODULE_MAP is exactly the set of IMPLEMENTED
    /// breeds in docs/v26.7.3/COGNITIVE_BREED_MAPPING.md (lowercased), so the
    /// const table and the doc cannot drift apart silently.
    #[test]
    fn breed_module_map_matches_mapping_doc_implemented_rows() {
        let doc = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/v26.7.3/COGNITIVE_BREED_MAPPING.md"
        ))
        .expect("mapping doc must exist (PROJ-206)");
        let mut doc_breeds: Vec<String> = doc
            .lines()
            .filter(|l| l.contains("| IMPLEMENTED |"))
            .filter_map(|l| l.split('|').nth(1))
            .map(|c| c.trim().to_lowercase())
            .collect();
        doc_breeds.sort();
        let mut map_breeds: Vec<String> = BREED_MODULE_MAP
            .iter()
            .map(|(b, _)| b.to_string())
            .collect();
        map_breeds.sort();
        assert_eq!(
            map_breeds, doc_breeds,
            "BREED_MODULE_MAP and COGNITIVE_BREED_MAPPING.md IMPLEMENTED rows diverged"
        );
    }

    /// Consistency gate against the admitted TTL facts: the fact file exists,
    /// carries exactly 55 CognitionBreed and 60 ProcessIntelligenceAlgorithm
    /// individuals, and never asserts a derived-only status predicate.
    #[test]
    fn admitted_fact_file_has_expected_shape() {
        let ttl = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packs/wasm4pm-facts-pack/ontology.ttl"
        ))
        .expect("admitted fact file must exist");
        let breeds = ttl.matches("a compat:CognitionBreed").count();
        let algos = ttl.matches("a pi:ProcessIntelligenceAlgorithm").count();
        assert_eq!(breeds, 55, "expected 55 admitted cognition breeds");
        assert_eq!(algos, 60, "expected 60 admitted PI algorithms");
        for (prefix, line) in ttl.lines().enumerate() {
            let line = line.trim_start();
            if line.starts_with('#') {
                continue;
            }
            for banned in ["compat:breedStatus", "pi:algorithmStatus"] {
                assert!(
                    !line.contains(banned),
                    "line {}: fact file asserts derived-only predicate {banned}",
                    prefix + 1
                );
            }
        }
    }

    #[test]
    fn every_module_path_is_a_declared_pub_mod_in_lib_rs() {
        let lib_src = include_str!("lib.rs");
        for (breed, module_path) in BREED_MODULE_MAP {
            let needle = format!("pub mod {module_path};");
            assert!(
                lib_src.contains(&needle),
                "breed '{breed}' cites module '{module_path}', but `{needle}` was not \
                 found as a pub mod declaration in lib.rs"
            );
        }
    }
}
