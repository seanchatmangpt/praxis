# SOC 2 Trust Services Criteria & Post-AGI Compliance Report
## For Praxis & MFACT Workspaces

**Audit Date**: 2026-07-13  
**Auditor**: Project Orchestrator Hierarchy  
**Scope**: 
* **Praxis Workspace** (`/Users/sac/praxis`): Rust-based triple store and reasoner.
* **MFACT Workspace** (`/Users/sac/mfact`): Lean 4 based proof-engineering and continuous validation environment.

---

## 1. Executive Summary

This report documents the security, determinism, data integrity, and compliance posture of the Praxis and MFACT workspaces under the AICPA Trust Services Criteria (TSC). 

* **Security**: Both repositories leverage strict dependency management (`deny.toml`, lockfiles) and automated CI/CD gating. Praxis successfully maintains low unsafe-code surfaces by auditing and removing `unsafe` blocks, though a minor configuration gap exists in workspace lint inheritance. A division-by-zero panic hazard was identified in the air_core NIF boundary.
* **Determinism**: The core logical paths (reasoner, hashing, telemetry) are completely free from system clock non-determinism, utilizing logical tick counters or injected clock interfaces. Non-determinism in SHACL validation reports (due to Rust's randomized `HashSet` hashing) has been successfully resolved through sorting.
* **Data Integrity**: Cryptographic integrity is maintained via BLAKE3 hash-chaining and atomic Git transaction ledgers. Lean proof files are continuously audited for completeness (`sorry` checks) while stripping comments to prevent false negatives.
* **Structural Verification**: MFACT formally checks key security properties (tenancy isolation and residue purity) inside the Lean 4 kernel, mapping them to CC6 (Access Controls) and C1.1/C1.2 (Confidentiality). However, these proofs operate on abstract Lean carriers; in production, all mappings remain at the **ANALOGY** level. There are no compiled production Rust bindings that carry formal proof correspondence today.
* **Continuous Auditing**: A dual-loop system—consisting of a 30-minute self-audit loop and an autonomous self-improvement loop—monitors repository drift, ledger consistency, and gap-closure metrics using a mathematical frontier-edge priority selection law.
* **Compilation Defect**: A pre-existing compilation error was identified in the `praxis-lean` crate of the Praxis workspace, where several untracked files fail to compile due to missing exports in `lib.rs` and missing enum variants in `error.rs`.

---

## 2. Security & Compliance Posture

### 2.1 Dependency Management & Gates
* **Automated License & Advisory Auditing**: Dependency access controls are enforced in Praxis using `deny.toml`. It strictly denies copyleft (GPL, AGPL, LGPL) licenses, blocks wildcard dependency versions, and limits allowed registries to `crates.io` and the `seanchatmangpt` GitHub organization.
* **CI/CD Compliance Gates**: Praxis implements an automated gate in `.github/workflows/praxis-validate.yml` that runs `praxis-retrofit validate compliance`. If the codebase score drops below 85%, pull requests are blocked.
* **Lint Inheritance Gap**: While the root package in Praxis Cargo.toml specifies `unsafe_code = "forbid"`, sub-crates in the workspace do not automatically inherit this check unless they explicitly include `[lints.rust] workspace = true`.

### 2.2 Unsafe Code & FFI Boundaries
* **Unsafe Code Elimination**: In `crates/wasm4pm-arazzo/src/parse.rs`, undocumented `unsafe` blocks previously used for zero-copy memory mapping (`MmapOptions`) were refactored into safe Rust alternatives (`fs::read`) to satisfy the workspace's zero-unsafe-code discipline.
* **Verification Testbed**: `crates/rust-fable-testbed/src/pipeline.rs` runs automated static checks that flag `unsafe` blocks, unapproved cipher modes (e.g. ECB), or hardcoded IVs. It requires a `// SAFETY:` or `// AUDITED:` comment to bypass the build warning. The fixture `fixtures/unsafe_audit_001/src/lib.rs` verifies this scanner by including a deliberate heap-buffer-overflow bug (`get_unchecked` past array bounds).
* **NIF FFI Safety & Panic Hazard**: The Erlang/Rust boundary (`apps/air_core/native/air_core_nif/src/lib.rs`) uses safe `rustler` bindings, avoiding raw pointers. However, line 96 contains a panic hazard where the division operator (`/`) executes division without checking if the divisor is zero. While `rustler` catches panic unwinds, a zero divisor causes a thread panic and should be handled gracefully by returning a `BadArg` error.

### 2.3 Praxis Workspace Compilation Failure
* **Compilation Blockers**: During workspace verification (`cargo check --tests --workspace`), a compilation failure occurs in the `praxis-lean` crate. The error originates from the untracked test file `crates/praxis-lean/tests/receipt_closure_gate.rs`, which references nonexistent functions (`praxis_lean::receipt_closure_gate`) and nonexistent enum variants (`LeanRefusal::UnclosedVerifiedReceipt`).
* **Underlying Causes**: The compilation failure is caused by an incomplete integration of the following untracked files:
  - `crates/praxis-lean/src/closure.rs` (untracked)
  - `crates/praxis-lean/src/receipt_gate.rs` (untracked)
  - `crates/praxis-lean/tests/receipt_closure_gate.rs` (untracked)
  These files drift from `lib.rs` (which does not declare the `closure` or `receipt_gate` modules) and `error.rs` (which is missing the `UnclosedVerifiedReceipt` and `ReceiptLineMissingField` enum variants).

---

## 3. Determinism Controls

### 3.1 Elimination of Clock Non-Determinism
* **No Wall Clocks in Hashing/Telemetry**: To guarantee byte-identical receipt logs under fixed seeds, system clocks are strictly banned in core reasoning and serialization paths:
  * `crates/cng/src/otel_rdf.rs` sorts telemetry quads lexicographically and parses timestamps strictly as static payload data.
  * `crates/multifractal-workflow/src/f02_observation_admission.rs` enforces sorted N-Triples parsing before BLAKE3 hashing.
  * `crates/multifractal-workflow/src/f11_bcinr_runtime.rs` drives progress using a logical tick counter (`self.run_state.tick`) instead of wall-clock time.
  * `crates/multifractal-workflow/src/trajectory_failure_process.rs` parses static git author timestamps instead of querying system time.
* **Clock Mocking & Dependency Injection**: In `crates/praxis-core/src/receipt_validator.rs`, checking future-timestamp boundaries is done via a mockable `Clock` trait. Tests inject a `FixedClock` while production uses `SystemClock::now()`, keeping proof environments deterministic.

### 3.2 Randomness Controls
* **Cryptographic Keys**: `crates/chatman-common/src/signed_receipt.rs` utilizes secure system entropy via `rand_core::OsRng` for generating Ed25519 signing keys.
* **Deterministic Seeding**: Benchmarks (`crates/cng/src/bench/engine.rs`) and property tests use a SplitMix64 pseudo-random generator seeded via configuration (`cfg.seed`), ensuring test vectors are reproducible.

---

## 4. Data Integrity Controls

### 4.1 Cryptographic Receipt Logging & Hash Chaining
* **BLAKE3 Hash Chaining**: `crates/chatman-common/src/provenance.rs` implements incremental execution tracing:
  $$\text{chain\_hash}_n = \text{blake3}(\text{chain\_hash}_{n-1} \mathbin{\Vert} \text{payload}_n)$$
  This enforces sequential execution ordering and makes the transaction log tamper-proof.
* **Git Transaction Ledger**: `crates/chatman-common/src/git_runtime.rs` serializes execution metadata to canonical JSON before computing the hash, ensuring that metadata fields like local system time do not pollute the content-addressed hash. It uses atomic locks (`GitLock`) to enforce commit serialization.

### 4.2 SHACL Validation & HashSet Non-Determinism
* **Validation Engine**: `crates/praxis-graphlaw/src/shacl/validate.rs` recursively validates RDF graphs against structural shapes (defined in files like `soc2-shapes.ttl`).
* **HashSet Randomization Mitigations**: Rust's standard `HashSet` (which uses a randomized hash builder) previously caused non-deterministic execution orders when iterating over shape nodes and focus nodes in `crates/praxis-graphlaw/src/shacl/report.rs`. This broke receipt hashing. The audit verified that unstable sorting (`sort_unstable()`) is now applied to these nodes prior to execution.
* **SPARQL Boundaries**: `crates/praxis-graphlaw/src/shacl/sparql.rs` rewrite queries to bind variables securely. It enforces the `CORE_ONLY` dialet boundary by rejecting `sh:sparql` constraints if the engine is restricted.

### 4.3 Proof Completeness (`no_sorry` Audit)
* **Sorry & Axiom Gating**: `crates/praxis-lean/src/no_sorry.rs` scans Lean 4 files to detect incomplete proofs (`sorry`) or unauthorized axiom declarations.
* **Comment Stripping**: The parser strips single-line (`--`) and multi-line (`/- ... -/`) comments prior to parsing, preventing false negatives where `sorry` occurs inside a comment string, while preserving line numbers for accurate reporting.

---

## 5. Structural Verification & Formal Mappings

MFACT's `ROADMAP_SOC2_MATH.md` maps specific Trust Services Criteria to Lean 4 proofs compiled via Lake. The four core formal mappings are detailed below:

| TSC Category | Criterion | Lean Proof File & Theorem | Target Invariant | Production Correspondence |
|---|---|---|---|---|
| **Security** | CC6 (Access Control) | `Tenancy.lean` <br> `minimalSupport_tenant_pure` <br> `crossTenant_residue_disjoint` | Tenant isolation: Residue sets of two distinct tenants are disjoint. | **ANALOGY** (No tenant-isolated Rust store exists) |
| **Processing Integrity** | PI1.3 (Processing Control) | `Runtime.lean` <br> `zero_unreceipted_completion` | Receipt completeness: No completed state exists without a receipt. | **ANALOGY** (Trivial by construction; FFI bypasses exist) |
| **Availability** | A1.2 (Disaster Recovery) | `Replay.lean` <br> <br> `replay_eq_of_traceEq` | Replay confluence: Event logs with commuting swaps resolve to identical states. | **MISSING** (No Rust recovery replay exists) |
| **Confidentiality** | C1.1 (Data Lifecycle) | `Antichain.lean` <br> `residue_purity` <br> `residue_isAntichain` | Residue minimality: Minimal support set never contains obligations in the context closure. | **ANALOGY** (Residue minimality does not equal data privacy) |

### 5.1 Verification of MFACT Lean Proofs
* **Tenancy Isolation (`Tenancy.lean`)**: Implements the tenant tag mapping function. Includes `TenancyCountermodel` which verifies that if the `Separated` hypothesis fails (i.e. access control is not structurally separated), the tenant purity theorem fails.
* **Confidentiality (`Antichain.lean`)**: Formally proves that the residue set is an antichain under $\subseteq$, meaning no minimal support is a strict subset of another, guaranteeing information minimality.

### 5.2 Limits & Boundaries of Formal Attestation
* **Organizational Criteria Excluded**: CC1-CC5, CC8-CC9, and all Privacy criteria are marked as `none`. Governance, personnel competence, change management policies, and incident response procedures are organizational processes and are out of scope for mathematical compiler checks.
* **The Analogy Gap**: Abstract Lean proofs do not directly govern production systems. There are no Rust FFI bindings or runtime carriers that verify these invariants in the live execution environment.
* **FFI Bypass Hazards (PA23/PA24)**: The self-audit ledger highlights that mathematical models are vulnerable to FFI bypasses. In finding PA23/PA24, the Rust code for `thermo_helmholtz` was documented as corresponding to a Lean proof, but the implementation bypassed it entirely using an FFI wrapper that returned a mocked constant, while a C wrapper returned mocked proof lemmas.

---

## 6. Continuous Compliance Processes

Both repositories operate under a dual-loop continuous auditing mechanism to prevent code-to-theory drift.

```
                  +---------------------------------------+
                  |        30-Minute Self-Audit Loop      |
                  |     (Writes to PRAXIS_SELF_AUDIT.md)  |
                  +-------------------+-------------------+
                                      |
                           Detects code-to-theory drift
                                      |
                                      v
+------------------+      Updates     +-------------------+
|    Gap Ledger    +----------------->|  Improvement Loop |
| (GAP_LEDGER.md)  |  (Successes close| (MFACT_SELF_      |
|                  |   ledger gaps)   |  IMPROVEMENT.md)  |
+--------+---------+                  +---------+---------+
         ^                                      |
         | Picks high-priority frontier gaps    |
         +--------------------------------------+
```

1. **30-Minute Self-Audit Loop (`PRAXIS_SELF_AUDIT.md`)**: A recurring cron job audits the repository's commit claims, standing files, and documentation against the live files. Findings are categorized as `CONFIRMED`, `REFUTED`, `DRIFTED`, or `UNVERIFIABLE`.
2. **Self-Improvement Loop (`MFACT_SELF_IMPROVEMENT_LOOP.md`)**: An autonomous loop that checks out fresh worktrees to resolve open compliance gaps, writing cryptographically signed receipts to `.mfact/receipts/` and appending logs to `metrics-history.jsonl`.
3. **Safety Guardrails**:
   * **Stuck-Item Guard** (`scripts/stuck_item_guard.py`): Skips a gap if it fails $>7$ times in the last 10 attempts to avoid infinite resource loops.
   * **Collision Guard (v3)**: Evaluates live git changes against `known-persistent-drift.txt`. If uncommitted files outside the baseline exist, the loop halts to avoid colliding with active developers.
4. **Frontier-Edge Priority Law**: The gap ledger (`GAP_LEDGER_v26.7.12.md`) prioritizes gap resolution using the following mathematical optimization:
   $$e^* = \arg\max_{e \in \{\text{OPEN, frontier-closed}\}} \frac{\text{UnlockMass}(e) \cdot \text{StandingCriticality}(e) \cdot \text{ScenarioCoverage}(e)}{\text{ClosureMass}(e)}$$

---

## 7. Recommendations & Remediation Plan

1. **Workspace Lint Inheritance**: Configure sub-crates in the Praxis Cargo workspace to inherit the root `unsafe_code = "forbid"` configuration. Add `[lints] workspace = true` to each crate's manifest.
2. **NIF Robustness Fix**: Resolve the division-by-zero panic in `apps/air_core/native/air_core_nif/src/lib.rs` by adding a check:
   ```rust
   "/" => {
       let i1: i64 = v1.decode()?;
       let i2: i64 = v2.decode()?;
       if i2 == 0 {
           return Err(rustler::Error::BadArg);
       }
       Ok((i1 / i2).encode(env))
   }
   ```
3. **Resolve `praxis-lean` Compilation Errors**: Add the module declarations for `closure` and `receipt_gate` to `lib.rs` and define the required error variants `UnclosedVerifiedReceipt` and `ReceiptLineMissingField` in `error.rs` to allow the workspace test suite to build and pass cleanly.
4. **Close Long-standing Ledger Gaps**: Prioritize closing the remaining gaps in `GAP_LEDGER_v26.7.12.md` using the frontier-edge formula, starting with unblocked low-mass items.
5. **Establish Real Production Correspondence**: To move from `ANALOGY` toward `CORRESPONDENCE`, develop Rust carriers that serialize state and feed it to the Lean checker to discharge the formal properties at runtime.
