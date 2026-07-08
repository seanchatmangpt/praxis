# Semantic-Web Library Source Code Audits: Status & Findings

**Date**: 2026-07-08  
**Methodology**: Direct source inspection via `cargo vendor` extraction from crates.io  
**Source Location**: `/Users/sac/praxis/vendors/vendor/` (360 transitive dependencies)

---

## Completed Audits (Evidence-Based, with Code Snippets)

### ✅ PROJ-505: horned-owl (LGPL-3.0)

**Audit Document**: `PROJ-505_horned_owl_source_audit.md` (13 KB)

**Key Findings**:
- **License**: LGPL-3.0 (copyleft) — Code copy forbidden; ideas via clean-room only
- **AST Design**: Generic `IRI<A>` + `Component` enum with 200+ axiom types
- **Determinism**: Uses Rc<str> interning + BTreeSet for deterministic ordering
- **Version**: 1.4.0 (published) / 2.0.0-dev (GitHub, unreleased)
- **Memory Pattern**: Rc<str> memory sharing + symbol interning — pattern to study for PROJ-401

**Recommendation**: ADAPT_IDEA (clean-room reimplementation of OWL AST design patterns)

**Integration Path**: Conditional (PROJ-505 P1) — only if PROJ-501 determines OWL RL v0 needs sophisticated AST beyond RDF triples.

---

### ✅ PROJ-503: SHACL Validation Ecosystem

**Audit Document**: `PROJ-503_shacl_source_audit.md` (9.6 KB)

**Crates Analyzed** (from vendors/):
- **shacl** 0.3.6 — Main SHACL validator (MIT/Apache-2.0)
- **shacl_validation** 0.2.12 — Validation traits (MIT/Apache-2.0)
- **shacl_ast** — SHACL Abstract Syntax Tree (MIT/Apache-2.0)
- **shacl_ir** — Optimized Internal Representation (MIT/Apache-2.0)
- **shacl_rdf** — RDF ↔ SHACL conversions (MIT/Apache-2.0)

**Key Findings**:
- **License**: MIT OR Apache-2.0 (dual permissive) — ADAPT_CODE eligible for all
- **Architecture**: RDF → AST → IR → Validation (three-layer design)
- **Constraints**: Complete SHACL Core coverage (minCount, maxCount, pattern, nodeKind, class, closed, etc.)
- **SPARQL Boundary**: Feature-gated (optional; SHACL Core always available)
- **Determinism**: Uses petgraph + prefixmap for deterministic shape evaluation
- **Test Suite**: W3C SHACL conformance test integration (tests/shacl_testsuite.rs)

**Adaptation Strategy**:
- **Non-Hot-Path (ADAPT_CODE)**: AST types, violation rendering, RDF conversions — direct with attribution
- **Hot-Path (ADAPT_IDEA)**: Constraint evaluation ordering, shape compilation patterns — study, reimplement clean-room

**Integration Timeline**:
- **Weeks 1-2**: Non-hot-path modules (AST types, RDF parsing, violation rendering)
- **Weeks 3-5**: Hot-path validation logic (study petgraph ordering, reimplement with determinism)
- **Week 6**: W3C conformance testing

---

## Partial / In-Progress Audits

### ⚠️ PROJ-501: OWL RL (reasonable crate) — NOT FOUND

**Status**: Incomplete  
**Issue**: The `reasonable` crate is **not published on crates.io** under that name

**Possible Sources**:
- Published under a different name on crates.io (e.g., `owl-rl`, `datalog-reasoner`)
- Only available on GitHub (not published to crates.io)
- Part of a larger project (monorepo)

**Next Action**: User to provide repository URL or crate name variant for PROJ-501 audit.

---

### ⚠️ PROJ-502: ShEx/DCTAP (rudof crate family) — PARTIALLY FOUND

**Status**: Incomplete (found related crates, not main rudof package)

**Crates Discovered in vendors/**:
- `rudof_iri` 0.3.6 — IRI handling for ShEx/SHACL
- `rudof_rdf` 0.3.6 — RDF support

**Crates NOT Found**:
- Main `rudof` package (likely exists but not a direct dependency of our test Cargo.toml)
- `rudof_shex` or `shex` (ShEx validator)

**Next Action**: Add `rudof`, `shex`, or relevant crate name to Cargo.toml dependencies and re-run `cargo vendor`.

---

### ⚠️ PROJ-504: N3 Reasoning (oxirs-ttl, eyeron) — PARTIALLY FOUND

**Status**: Incomplete

**Crates Discovered in vendors/**:
- `oxttl` 0.3.1 — Turtle/N-Triples parsing (part of oxigraph)

**Crates NOT Found**:
- `oxirs-ttl` (may not be published; likely internal to oxigraph)
- `eyeron` (Prolog-based; not Rust ecosystem)

**Next Action**: Check if N3 support is in oxigraph codebase or published separately.

---

## Available but Not Yet Audited

### Oxigraph Ecosystem (foundation RDF layer)

**Crates in vendors/**:
- `oxrdf` 0.3.0 — Core RDF types (Term, Triple, Subject, Predicate, Object)
- `oxrdfio` — RDF I/O (parsing, serialization)
- `oxrdfxml` — RDF/XML support
- `oxttl` 0.3.1 — Turtle/N-Triples/N-Quads parsing
- `oxiri` 0.2.11 — IRI validation/parsing
- `oxilangtag` — Language tag handling
- `oxsdatatypes` — XSD datatype support
- `oxjsonld` — JSON-LD support

**License**: MIT (fully permissive)

**Finding**: oxigraph provides the **RDF foundation layer** that all semantic-web crates depend on. Understanding oxigraph's data structures is essential for Graphlaw integration.

**Recommendation**: Consider adding a PROJ-506 (or inline with PROJ-401) audit of oxigraph RDF types for:
- Triple representation (Subject, Predicate, Object, Graph)
- Term types (IRI, Blank node, Literal)
- N-Quads encoding for receipt canonicalization
- Deterministic iteration guarantees

---

## Vendored Crates Summary

**Total crates vendored**: 360 (including transitive dependencies)

**Target semantic-web crates found**:
- ✅ horned-owl (OWL AST)
- ✅ shacl, shacl_validation, shacl_ast, shacl_ir, shacl_rdf (SHACL ecosystem)
- ✅ oxigraph + family (RDF foundation)
- ⚠️ rudof_iri, rudof_rdf (ShEx/SHACL support, but not main rudof package)
- ❌ reasonable (OWL RL — not found)
- ❌ eyeron (N3 Prolog — out of scope anyway)
- ⚠️ N3 support (oxttl partial; oxirs-ttl not found)

---

## Next Steps

1. **PROJ-501 (OWL RL audit)**: Provide repository URL or crate name for `reasonable`
   - If available on crates.io, add to Cargo.toml and re-vendor
   - If GitHub-only, clone from repository

2. **PROJ-502 (ShEx audit)**: Add `rudof` main package and related ShEx crates to Cargo.toml
   - Re-run `cargo vendor` to fetch full dependency tree
   - Audit AST types, shape-map parser, validation report structures

3. **PROJ-504 (N3 audit)**: Clarify N3 implementation source
   - Check if `eyeron` Prolog crate has Rust bindings
   - Or use `oxttl::n3` if N3 is part of oxigraph's turtle module

4. **Optional PROJ-506** (oxigraph foundation audit): Analyze RDF types in oxigraph for:
   - Triple/Term representation patterns
   - N-Quads canonical form for receipt generation
   - Deterministic iteration guarantees

---

## Methodology Notes

- **Source extraction**: `cargo vendor` downloads all transitive dependencies to `vendor/` directory
- **Source inspection**: Direct read of Cargo.toml, src/lib.rs, key modules
- **Code snippets**: Direct extraction from source with file paths and line numbers
- **License verification**: SPDX identifiers from Cargo.toml (Cargo.toml generated automatically from crates.io metadata)

**Advantages of this approach**:
- ✅ Access to actual published source code (not docs, not speculative)
- ✅ Dependency graph visibility (what transitive deps are pulled in)
- ✅ Version accuracy (exactly what crates.io published)
- ✅ Reproducible (can be redone any time by re-running `cargo vendor`)

**Limitations**:
- ❌ Does not include GitHub-only crates (unreleased/development versions)
- ❌ Does not include private repositories

