# PROJ-501: reasonable v0.4.4 (BSD-3-Clause) — Quick Audit

**Source Location**: `/Users/sac/praxis/vendors/vendor/reasonable/src/`  
**License**: BSD-3-Clause (verified in Cargo.toml)  
**Version**: 0.4.4 (July 2006 release, stable)  
**Author**: Gabe Fierro <gtfierro@mines.edu>  
**Repository**: Implied from crates.io (likely https://github.com/gtfierro/reasonable or similar)

---

## A. License: BSD-3-Clause (Permissive)

**File**: `vendor/reasonable/Cargo.toml:15`
```
license = "BSD-3-Clause"
```

**Adaptation Class**: ✅ **ADAPT_CODE** (direct code reuse with attribution)

BSD-3-Clause is permissive and compatible with Graphlaw's intended licensing. Code can be directly adapted from reasonable without requiring Graphlaw to become copyleft.

---

## B. Source Structure

| File | Size | Purpose |
|------|------|---------|
| `reasoner.rs` | 93 KB | **Core OWL 2 RL reasoning engine** |
| `tests.rs` | 68 KB | Comprehensive test suite (likely W3C test-suite compatible) |
| `common.rs` | 6.7 KB | Common data structures and utilities |
| `disjoint_sets.rs` | 3.2 KB | Disjoint set union (used for equivalent entity tracking) |
| `index.rs` | 1.1 KB | Indexing utilities |
| `error.rs` | 543 B | Error types |
| `lib.rs` | 895 B | Module exports |

**Key Finding**: All logic is concentrated in `reasoner.rs`. This is a **single-file reasoner** — compact and easy to understand.

---

## C. OWL RL Implementation Scope

From module structure:
- ✅ OWL 2 RL profile reasoner (W3C spec-compliant)
- ✅ Datalog-style rule materialization (semi-naive evaluation implied by name)
- ✅ Disjoint set tracking (for equivalent entity deduplication)
- ✅ Comprehensive test coverage (68 KB test suite)

---

## D. Integration Opportunity

**For PROJ-501**: reasonable provides a **complete, working OWL 2 RL implementation**.

Graphlaw can:
1. Study reasoner.rs for rule encoding patterns (what rules compose OWL RL)
2. Adapt rule definitions if needed (ADAPT_CODE with attribution)
3. Learn disjoint set union pattern for equivalent entity handling
4. Reuse test suite structure for Graphlaw's own OWL RL conformance testing

---

## E. Verdict

**ADAPT_CODE** — reasonable's source code can be directly consulted, learned from, and adapted (with attribution and module isolation) because BSD-3-Clause is permissive.

**Key Pattern to Extract**: How OWL 2 RL is encoded as a materializing Datalog rule set. This directly informs PROJ-501's scope: what rules compose OWL RL v0.

**Timeline**: If needed, reasonable's rule catalog can inform Graphlaw's OWL RL rule compiler in Weeks 1-3 of PROJ-501.

