# PROJ-502: rudof v0.1.12 (MIT/Apache-2.0) — Quick Audit

**Source Location**: `/Users/sac/praxis/vendors/vendor/rudof/src/`  
**License**: MIT OR Apache-2.0 (verified in Cargo.toml)  
**Version**: 0.1.12 (stable)  
**Authors**: Jose Emilio Labra Gayo, Ángel Iglesias Préstamo, Marc-Antoine Arnaud  
**Repository**: https://github.com/rudof-project/rudof  
**Note**: v0.1.x is legacy; v0.3.6 is current (see vendor for shacl, shacl_validation, shacl_ast, shacl_ir, shacl_rdf which are v0.3.6)

---

## A. License: MIT OR Apache-2.0 (Dual Permissive)

**File**: `vendor/rudof/Cargo.toml:16`
```
license = "MIT OR Apache-2.0"
```

**Adaptation Class**: ✅ **ADAPT_CODE** (direct code reuse with attribution)

Dual permissive licensing allows direct code adaptation without copyleft implications.

---

## B. Source Structure

| File | Size | Purpose |
|------|------|---------|
| `lib.rs` | 498 B | Module re-exports |
| `shacl.rs` | 3.1 KB | SHACL validator wrapper/facade |

**Key Finding**: rudof v0.1.12 is a **thin facade** that re-exports functionality from `rudof_iri`, `rudof_rdf`, `shacl`, etc.

The actual implementation is in separate crates (now found in vendors/):
- `shacl` v0.3.6 (SHACL Core validator) — ✅ Already audited in PROJ-503
- `shacl_ast` v0.3.6 (AST definitions)
- `shacl_ir` v0.3.6 (Internal representation)
- `shacl_rdf` v0.3.6 (RDF conversions)
- `shacl_validation` v0.2.12 (Validation traits)
- `rudof_iri` v0.3.6 (IRI handling)
- `rudof_rdf` v0.3.6 (RDF core)

---

## C. ShEx/DCTAP Scope

**Finding**: rudof v0.1.12 does **not** appear to contain ShEx validation code.

The actual ShEx support is likely in:
- Newer rudof versions (v0.3.6+)
- Separate `purrdf-shex` crate (mentioned in cargo search: "ShEx 2.1 engine")
- Or `shex_cli` crate

---

## D. Verdict

**Status**: PROJ-502 requires clarification.

rudof v0.1.12 is a **compatibility/facade crate** that aggregates functionality from the actual implementations. For ShEx audit:

1. Check if `purrdf-shex` or newer `rudof` versions (v0.3.6+) contain ShEx validator
2. Or use `shacl` + `rudof_rdf` as the ShEx/SHACL base (which is already audited in PROJ-503)

**Current Finding**: SHACL ecosystem is already audited (PROJ-503 complete). ShEx support location is unclear and requires user clarification.

---

## E. Recommendation

Since SHACL (MIT/Apache-2.0) is fully audited in PROJ-503:

- If ShEx is needed separately: audit `purrdf-shex` crate
- If ShEx scope is "SHACL is sufficient": ShEx audit can be deferred (PROJ-502 marked DEFERRED)
- If ShEx/SHACL are bundled in rudof v0.3.6+: add those versions to Cargo.toml and re-vendor for full audit

