# 00 — Consolidated Ecosystem Catalog & Abstraction Report

**Owner:** Praxis Project Orchestrator  
**Sources:** Reports from Ecosystem Cataloger 1 (`8ba19fb9`) and Ecosystem Cataloger 2 (`dd3c10f2`)  
**Status:** ALIVE_UNDER_SCOPE  
**Object under test:** Consolidated architectural patterns of Generative Typestates, `RulePackServer`, and `ggen` µ-pipeline.

---

## 1. Executive Summary

This catalog consolidates the key findings from the independent read-only investigations of the `rocket-craft` and `lsp-max` workspaces. These findings extract concrete design patterns and code structures to upgrade the `praxis` boilerplate generator, enabling generated code to natively implement the "Post-Chatman Equation" ($A = \mu(O^*)$).

Three core abstractions have been identified and modeled for integration:
1. **Generative Typestates**: Zero-sized type (ZST) phase markers (`Raw`, `Validated`, `Admitted`) that transition linearly by consuming `self`, combined with Witness-gating and Seal patterns.
2. **`RulePackServer` structures**: Language server traits wrapping `tower-lsp` that implement latency budgets (`EvalBudget`), workspace indexing (`WorkspaceIndex`), online Statistical Process Control (`SpcMonitor`), and rule circuit breakers (`CircuitBreaker`).
3. **`ggen` Micro-Pipeline**: A code generator pipeline composed of loading, SPARQL construct-inference, validation, extraction (with strict `ORDER BY` checks), template lowering, and `FileTransaction` atomic commits with BLAKE3 receipts.

---

## 2. Abstraction Catalog

### A. Generative Typestates
Enforced at compile-time to prevent invalid state operations and bypasses.

*   **Zero-Sized State Markers**:
    ```rust
    pub mod sealed {
        pub trait LifecycleState {}
    }
    pub struct Raw;
    impl sealed::LifecycleState for Raw {}

    pub struct Validated;
    impl sealed::LifecycleState for Validated {}

    pub struct Admitted;
    impl sealed::LifecycleState for Admitted {}
    ```
*   **Evidence Container**:
    ```rust
    use std::marker::PhantomData;

    pub struct Evidence<T, S: sealed::LifecycleState, W> {
        inner: T,
        _state: PhantomData<S>,
        _witness: PhantomData<W>,
    }
    ```
*   **Gated Transitions**:
    ```rust
    impl<T, W> Evidence<T, Raw, W> {
        pub fn new(inner: T) -> Self {
            Self { inner, _state: PhantomData, _witness: PhantomData }
        }
        pub fn inner(&self) -> &T { &self.inner }
    }

    impl<T, W> Evidence<T, Admitted, W> {
        pub fn inner(&self) -> &T { &self.inner }
        // Witness Pattern: restricted to crate/module validator authority
        pub(crate) fn admit_unchecked(inner: T) -> Self {
            Self { inner, _state: PhantomData, _witness: PhantomData }
        }
    }

    pub trait Admit {
        type Input;
        type Witness;
        type Error;

        fn admit(
            input: Evidence<Self::Input, Raw, Self::Witness>,
        ) -> Result<Evidence<Self::Input, Admitted, Self::Witness>, Self::Error>;
    }
    ```
*   **Seal Pattern**:
    ```rust
    pub struct AdmittedReceipt {
        pub chain_hash: [u8; 32],
        pub timestamp: u64,
        _seal: (), // Private field prevents direct construction
    }
    ```

### B. `RulePackServer`
A protocol abstraction wrapping `tower-lsp` to automate document synchronization and diagnostics.

*   **Trait Definition**:
    ```rust
    pub trait RulePackServer: Send + Sync + 'static {
        type Document;
        type Diagnostic;

        fn server_name(&self) -> &'static str;
        fn workspace_index(&self) -> Option<&WorkspaceIndex>;
        fn spc_monitor(&self) -> Option<&Mutex<SpcMonitor>>;
        fn latency_trackers(&self) -> Option<&Arc<DashMap<String, RuleLatencyTracker>>>;
        fn rule_circuit_breaker(&self) -> Option<&Arc<Mutex<CircuitBreaker>>>;
        fn scan_document(&self, uri: &str, content: &str) -> Vec<Self::Diagnostic>;
    }
    ```
*   **EvalBudget (Latency Classification)**:
    Sync rules exceeding 50ms are promoted to Background after 3 consecutive slow runs.
*   **SpcMonitor**:
    Sliding window of 50 samples tracking mean and variance via Welford's algorithm to trigger Western Electric statistical process control alerts.
*   **CircuitBreaker**:
    Trips evaluation to `Open` if failures exceed a designated threshold, protecting the editor process.

### C. `ggen` Micro-Pipeline
1.  **Loading**: Load Turtle files into Oxigraph-backed memory store.
2.  **Construct (Inference)**: Run SPARQL `CONSTRUCT` rules. Strictly enforce `GGEN-INFER-001` (aborts if construct adds zero triples in strict mode to prevent query drift).
3.  **Validation**: Run SHACL and SPARQL `ASK` checks (ASK result = true signals a violation; severity = Error aborts commit).
4.  **Extract**: Execute SPARQL `SELECT` queries. Enforce `ORDER BY` to guarantee compile determinism.
5.  **Lowering**: Render Tera templates (static or dynamic folder fanning). Drop to `TemplateFallback` stubs if LLM is unavailable.
6.  **Atomic Commits & Receipts**: Commit filesystem changes atomically via `FileTransaction` with a `Drop` rollback hook. Generate BLAKE3 receipts.

---

## 3. Consensus & Resolved Issues

*   **Consensus**:
    *   Both catalogers independently verified the exact typestate parameters and generic marker structures from `/Users/sac/rocket-craft/crates/mech_morphology_law/src/machine.rs`.
    *   Both catalogers mapped the Tower-LSP `RulePackServer` trait, `EvalBudget`, `SpcMonitor`, and `CircuitBreaker` abstractions from `lsp-max`.
    *   Both catalogers agreed that compiling/running `oxigraph` and `tree-sitter` inside templates expands compile times and recommended placing them behind optional `lsp` and `ggen` features inside `template/Cargo.toml`.
*   **Resolved Conflicts**:
    *   *Conflict*: Cataloger 2 reported a compile failure in `lsp-max-playground` during base verification.
    *   *Resolution*: Synthesized analysis confirms that the failure occurred in a playground example due to a missing field in `WorkspaceServerCapabilities`. The core `lsp-max` crate, which holds the target abstractions, is compiler-clean and fully admitted.

---

## 4. Residual Risks & Gaps

*   **Risks**:
    *   Performance scaling of Oxigraph's memory store when querying multi-gigabyte ontologies.
    *   Concurrency lockups under highly intense, multi-threaded document edits.
*   **Gaps**:
    *   Live testing of `AppLspServer` in a running editor environment was not performed due to the read-only constraint on source code execution.

---

**Status:** ALIVE_UNDER_SCOPE  
**Verification:** Consensus reached across two independent explorer reports. Synthesized into the final catalog.
