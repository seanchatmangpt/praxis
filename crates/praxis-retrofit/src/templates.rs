//! Praxis standardized templates for retrofit

use crate::PraxisSpec;

/// Template for Cargo.toml [lints] block
pub fn cargo_lints_template(_spec: &PraxisSpec) -> String {
    format!(
        r#"[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"

[lints.rustdoc]
missing_crate_level_docs = "warn"
"#
    )
}

/// Template for workspace [lints] inheritance
pub fn cargo_lints_workspace_inherit() -> String {
    "[lints]\nworkspace = true\n".to_string()
}

/// Template for typos.toml
pub fn typos_toml_template() -> String {
    r#"[default]
check-filenames = true
check-files = true

[default.extend-exclude]
# Common non-source directories
"bench_data/" = "benchmark data"
"results/" = "benchmark results"

[default.extend-words]
# Domain-specific terms that aren't typos
"nd" = "nd"
"od" = "od"
"te" = "te"
"te" = "te"
"OCELObject" = "OCELObject"
"OCEL" = "OCEL"
"BPMN" = "BPMN"
"DFG" = "DFG"
"OC" = "OC"
"WASM" = "WASM"
"WASI" = "WASI"
"wasm" = "wasm"
"pm4py" = "pm4py"
"Petri" = "Petri"
"conformance" = "conformance"
"behaviours" = "behaviours"
"behaviours" = "behaviours"
"Cognition" = "Cognition"
"cognition" = "cognition"
"breed" = "breed"
"Prolog" = "Prolog"
"SPARQL" = "SPARQL"
"BLAKE" = "BLAKE"
"blake" = "blake"
"NFT" = "NFT"
"CallId" = "CallId"
"triadic" = "triadic"
"BUSL" = "BUSL"
"AGPL" = "AGPL"
"Diátaxis" = "Diátaxis"
"qualifiers" = "qualifiers"
"unsealed" = "unsealed"
"Unsealed" = "Unsealed"
"serde" = "serde"
"serde" = "serde"
"natively" = "natively"
"Natively" = "Natively"
"Monorepo" = "Monorepo"
"monorepo" = "monorepo"
"Monorepos" = "Monorepos"
"monorepos" = "monorepos"
"orchestration" = "orchestration"
"Orchestration" = "Orchestration"
"SPC" = "SPC"
"OTEL" = "OTEL"
"otel" = "otel"
"OTel" = "OTel"
"otels" = "otels"
"observability" = "observability"
"Observability" = "Observability"
"Metrics" = "Metrics"
"metrics" = "metrics"
"Telemetry" = "Telemetry"
"telemetry" = "telemetry"
"Telemetries" = "Telemetries"
"telemetries" = "telemetries"
"provenance" = "provenance"
"Provenance" = "Provenance"
"provenances" = "provenances"
"Provenances" = "Provenances"
"verifiable" = "verifiable"
"Verifiable" = "Verifiable"
"auditable" = "auditable"
"Auditable" = "Auditable"
"Audit" = "Audit"
"audit" = "audit"
"Audits" = "Audits"
"audits" = "audits"
"Retrofitting" = "Retrofitting"
"retrofitting" = "retrofitting"
"Retrofit" = "Retrofit"
"retrofit" = "retrofit"
"Retrofits" = "Retrofits"
"retrofits" = "retrofits"
"CONTRIBUTING" = "CONTRIBUTING"
"Contributing" = "Contributing"
"contributing" = "contributing"
"CODE_OF_CONDUCT" = "CODE_OF_CONDUCT"
"CODE" = "CODE"
"Conduct" = "Conduct"
"conduct" = "conduct"
"Conducts" = "Conducts"
"conducts" = "conducts"
"Typestates" = "Typestates"
"typestates" = "typestates"
"Typestate" = "Typestate"
"typestate" = "typestate"
"Typestated" = "Typestated"
"typestated" = "typestated"
"Witness" = "Witness"
"witness" = "witness"
"Witnesses" = "Witnesses"
"witnesses" = "witnesses"
"Witnessed" = "Witnessed"
"witnessed" = "witnessed"
"Witnessing" = "Witnessing"
"witnessing" = "witnessing"
"FlexDuck" = "FlexDuck"
"linkme" = "linkme"
"Linkme" = "Linkme"
"Linkmes" = "Linkmes"
"linkmes" = "linkmes"
"sealing" = "sealing"
"Sealing" = "Sealing"
"sealed" = "sealed"
"Sealed" = "Sealed"
"seals" = "seals"
"Seals" = "Seals"
"Clauses" = "Clauses"
"clauses" = "clauses"
"Clause" = "Clause"
"clause" = "clause"
"Clauess" = "Clauses"
"clauess" = "clauses"
"Imports" = "Imports"
"imports" = "imports"
"Import" = "Import"
"import" = "import"
"Imported" = "Imported"
"imported" = "imported"
"Importing" = "Importing"
"importing" = "importing"
"Exports" = "Exports"
"exports" = "exports"
"Export" = "Export"
"export" = "export"
"Exported" = "Exported"
"exported" = "exported"
"Exporting" = "Exporting"
"exporting" = "exporting"
"typedefs" = "typedefs"
"Typedefs" = "Typedefs"
"Typedef" = "Typedef"
"typedef" = "typedef"
"Typediff" = "Typediff"
"Typediffer" = "Typediffer"
"Typediffs" = "Typediffs"
"typediff" = "typediff"
"typediffer" = "typediffer"
"typediffs" = "typediffs"
"Runtypes" = "Runtypes"
"runtypes" = "runtypes"
"Runtype" = "Runtype"
"runtype" = "runtype"
"Typeguard" = "Typeguard"
"typeguard" = "typeguard"
"Typeguards" = "Typeguards"
"typeguards" = "typeguards"
"Conformant" = "Conformant"
"conformant" = "conformant"
"Conformants" = "Conformants"
"conformants" = "conformants"
"Conforms" = "Conforms"
"conforms" = "conforms"
"Conformed" = "Conformed"
"conformed" = "conformed"
"Conforming" = "Conforming"
"conforming" = "conforming"
"Conformance" = "Conformance"
"conformance" = "conformance"
"Conformances" = "Conformances"
"conformances" = "conformances"
"Refusals" = "Refusals"
"refusals" = "refusals"
"Refusal" = "Refusal"
"refusal" = "refusal"
"Refused" = "Refused"
"refused" = "refused"
"Refusing" = "Refusing"
"refusing" = "refusing"
"Admit" = "Admit"
"admit" = "admit"
"Admits" = "Admits"
"admits" = "admits"
"Admitted" = "Admitted"
"admitted" = "admitted"
"Admitting" = "Admitting"
"admitting" = "admitting"
"Admission" = "Admission"
"admission" = "admission"
"Admissions" = "Admissions"
"admissions" = "admissions"
"Bounded" = "Bounded"
"bounded" = "bounded"
"Bounds" = "Bounds"
"bounds" = "bounds"
"Bound" = "Bound"
"bound" = "bound"
"Bounding" = "Bounding"
"bounding" = "bounding"
"Receipts" = "Receipts"
"receipts" = "receipts"
"Receipt" = "Receipt"
"receipt" = "receipt"
"Receipting" = "Receipting"
"receipting" = "receipting"
"Receipt" = "Receipt"
"receipt" = "receipt"
"Rceeipts" = "Receipts"
"rceeipts" = "receipts"
"Archetypal" = "Archetypal"
"archetypal" = "archetypal"
"Archetypes" = "Archetypes"
"archetypes" = "archetypes"
"Archetype" = "Archetype"
"archetype" = "archetype"
"Archetyped" = "Archetyped"
"archetyped" = "archetyped"
"Archetyping" = "Archetyping"
"archetyping" = "archetyping"
"Axioms" = "Axioms"
"axioms" = "axioms"
"Axiom" = "Axiom"
"axiom" = "axiom"
"Axiomatic" = "Axiomatic"
"axiomatic" = "axiomatic"
"Axiomatically" = "Axiomatically"
"axiomatically" = "axiomatically"
"Axiomatized" = "Axiomatized"
"axiomatized" = "axiomatized"
"Axiomatizing" = "Axiomatizing"
"axiomatizing" = "axiomatizing"
"Sequent" = "Sequent"
"sequent" = "sequent"
"Sequents" = "Sequents"
"sequents" = "sequents"
"Sequentially" = "Sequentially"
"sequentially" = "sequentially"
"Sequentiality" = "Sequentiality"
"sequentiality" = "sequentiality"
"Sequencing" = "Sequencing"
"sequencing" = "sequencing"
"Sequenced" = "Sequenced"
"sequenced" = "sequenced"
"Sequencer" = "Sequencer"
"sequencer" = "sequencer"
"Sequencers" = "Sequencers"
"sequencers" = "sequencers"
"Seance" = "Seance"
"seance" = "seance"
"Seances" = "Seances"
"seances" = "seances"
"Seanced" = "Seanced"
"seanced" = "seanced"
"Seancing" = "Seancing"
"seancing" = "seancing"
"Seancer" = "Seancer"
"seancer" = "seancer"
"Seancers" = "Seancers"
"seancers" = "seancers"
"Refracted" = "Refracted"
"refracted" = "refracted"
"Refractions" = "Refractions"
"refractions" = "refractions"
"Refraction" = "Refraction"
"refraction" = "refraction"
"Refractive" = "Refractive"
"refractive" = "refractive"
"Refractor" = "Refractor"
"refractor" = "refractor"
"Refractors" = "Refractors"
"refractors" = "refractors"
"Refract" = "Refract"
"refract" = "refract"
"Refracts" = "Refracts"
"refracts" = "refracts"
"Refracting" = "Refracting"
"refracting" = "refracting"
"Refracts" = "Refracts"
"refracts" = "refracts"
"Refractive" = "Refractive"
"refractive" = "refractive"
"Refracted" = "Refracted"
"refracted" = "refracted"
"Refractor" = "Refractor"
"refractor" = "refractor"
"Refractors" = "Refractors"
"refractors" = "refractors"
"Refraction" = "Refraction"
"refraction" = "refraction"
"Refractions" = "Refractions"
"refractions" = "refractions"
"#
    .to_string()
}

pub fn justfile_template(crate_name: &str) -> String {
    format!(
        r#"# {} Justfile
# Standard praxis task runner

set shell := ["bash", "-uc"]

# Run all formatters
fmt:
    cargo fmt --all

# Run all linters
lint:
    cargo clippy --all --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --all --all-features

# Build release binary
build:
    cargo build --all --release

# Generate documentation
doc:
    cargo doc --all --no-deps --all-features

# Run benchmarks
bench:
    cargo bench --all

# Pre-commit gate: fmt -> lint -> test
pre-commit: fmt lint test

# Clean build artifacts
clean:
    cargo clean

# Run with verbose output
verbose:
    RUST_BACKTRACE=1 cargo test --all -- --nocapture
"#,
        crate_name
    )
}

/// Template for .editorconfig
pub fn editorconfig_template() -> String {
    r#"# EditorConfig is awesome: https://EditorConfig.org

# top-most EditorConfig file
root = true

# Unix-style newlines with a newline ending every file
[*]
end_of_line = lf
insert_final_newline = true
charset = utf-8
trim_trailing_whitespace = true

# Rust files
[*.rs]
indent_style = space
indent_size = 4

# TOML files
[*.toml]
indent_style = space
indent_size = 2

# Markdown files
[*.md]
indent_style = space
indent_size = 2

# JSON files
[*.json]
indent_style = space
indent_size = 2

# YAML files
[*.{yaml,yml}]
indent_style = space
indent_size = 2

# Makefiles (require tabs)
[Makefile*]
indent_style = tab

# Shell scripts
[*.sh]
indent_style = space
indent_size = 2
"#
    .to_string()
}

/// Generate commit message for retrofit
pub fn commit_message(phase: &str, repo_name: &str, actions_count: usize) -> String {
    format!(
        "Retrofit {}: Apply praxis standards ({} files)\n\nPhase: {}\nActions: {}\nStandardizes repository structure and configuration for house-style consistency.\n\nCo-Authored-By: praxis-retrofit <noreply@seanchatmangpt.dev>",
        repo_name, actions_count, phase, actions_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_lints_template() {
        let spec = crate::PraxisSpec::default();
        let template = cargo_lints_template(&spec);
        assert!(template.contains("unsafe_code = \"forbid\""));
        assert!(template.contains("all = \"warn\""));
    }

    #[test]
    fn test_typos_template() {
        let template = typos_toml_template();
        assert!(template.contains("check-filenames"));
        assert!(template.contains("OCEL"));
    }

    #[test]
    fn test_justfile_template() {
        let justfile = justfile_template("my-crate");
        assert!(justfile.contains("fmt:"));
        assert!(justfile.contains("cargo"));
    }
}
