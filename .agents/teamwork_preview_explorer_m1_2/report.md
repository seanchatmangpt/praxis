# Ecosystem Architectural Patterns & Abstraction Report

**Author:** Ecosystem Cataloger 2 (`teamwork_preview_explorer_m1_2`)  
**Date:** 2026-06-21  
**Status:** ALIVE_UNDER_SCOPE  
**Object under test:** Architectural Patterns in `rocket-craft`, `lsp-max`, and `ggen` for `praxis` Scaffolding Integration

---

## 1. Generative Typestates

The typestate pattern in the post-Chatman ecosystem guarantees software lifecycle state safety at compile time. By mapping phase states onto zero-sized marker structs (ZSTs) and utilizing generic wrappers, the compiler physically rejects invalid transitions or un-admitted types.

### ZSTs and generic wrappers
Zero-sized marker structs are used exclusively for type parameters and carry no runtime overhead.

```rust
use core::marker::PhantomData;

/// Represents baseline unvalidated raw evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Raw;

/// Represents data that has undergone metric validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validated;

/// Represents terminal admitted proof phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Admitted;

/// Container holding the inner domain object under a specific lifecycle state.
#[derive(Debug)]
pub struct Evidence<T, State, Witness> {
    inner: T,
    _state: PhantomData<State>,
    _witness: PhantomData<Witness>,
}

impl<T, Witness> Evidence<T, Raw, Witness> {
    /// Construct raw evidence containing the input data.
    pub fn raw(inner: T) -> Self {
        Self {
            inner,
            _state: PhantomData,
            _witness: PhantomData,
        }
    }
}

impl<T, State, Witness> Evidence<T, State, Witness> {
    /// Unsafely consume and extract the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Read a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}
```

### Witness and Seal Patterns
* **Witness Pattern:** The `Witness` type parameter acts as a cryptographic or logical signature. Standard transitions require implementing a specific `Admit` trait, preventing arbitrary state casting.
* **Seal Pattern:** Structs are sealed by including a private field (`_seal: ()`), preventing callers from bypass-constructing the admitted struct directly using struct literals.

```rust
/// The standard gatekeeper trait.
pub trait Admit {
    type Input;
    type Witness;
    type Reason;

    /// The only logical path to transition from Raw to Admitted state.
    fn admit(
        input: Evidence<Self::Input, Raw, Self::Witness>,
    ) -> Result<Evidence<Self::Input, Admitted, Self::Witness>, Self::Reason>;
}

/// A terminal cryptographically addressable record.
#[derive(Debug, Clone)]
pub struct AdmittedReceipt {
    pub content_hash: String,
    pub timestamp: String,
    /// Private field enforces constructor purity (Seal Pattern)
    _seal: (),
}

impl AdmittedReceipt {
    /// Public constructor is unavailable outside of its defining module.
    /// Allowed inside this module to produce the proof receipt.
    pub(crate) fn new(content_hash: String, timestamp: String) -> Self {
        Self {
            content_hash,
            timestamp,
            _seal: (),
        }
    }
}
```

### Gated Transitions (Consumption and Gating)
Methods that advance typestates consume the caller (`self`), preventing double-state mutations or reuse of raw transitions.

```rust
#[derive(Debug, Clone)]
pub struct MeasuredMech {
    pub serial_id: String,
    pub bipedal_height: f32,
    pub payload_mass: f32,
}

/// Gated morphology admission machine.
#[derive(Debug)]
pub struct AdmissionMachine<L, State> {
    law: L,
    mech: MeasuredMech,
    outcome: Option<String>,
    _state: PhantomData<State>,
}

impl<L> AdmissionMachine<L, Raw> {
    pub fn new(law: L, mech: MeasuredMech) -> Self {
        Self {
            law,
            mech,
            outcome: None,
            _state: PhantomData,
        }
    }

    /// Transitions Raw -> Validated. Consumes self.
    pub fn validate(self) -> AdmissionMachine<L, Validated> {
        // Enforce validations ...
        AdmissionMachine {
            law: self.law,
            mech: self.mech,
            outcome: Some("Pass".to_string()),
            _state: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct ClaimHold {
    pub reason: String,
}

impl<L> AdmissionMachine<L, Validated> {
    /// Gated transition Validated -> Admitted.
    /// Returns Ok(AdmissionMachine<L, Admitted>) only if outcome is valid.
    pub fn admit(self) -> Result<AdmissionMachine<L, Admitted>, ClaimHold> {
        match self.outcome.as_deref() {
            Some("Pass") => Ok(AdmissionMachine {
                law: self.law,
                mech: self.mech,
                outcome: self.outcome,
                _state: PhantomData,
            }),
            _ => Err(ClaimHold {
                reason: "Validation check did not yield Pass state".to_string(),
            }),
        }
    }
}
```

---

## 2. `RulePackServer` Structures

`RulePackServer` represents a standardized abstraction that bridges custom programming language ASTs, Tree-Sitter incremental diagnostics, and the Tower-LSP framework. It manages file synchronization, indexing, latency routing, and Statistical Process Control (SPC) diagnostics.

### Config and Schema Structs
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalBudget {
    /// Synchronous execution. Must evaluate under 50ms (typing thread).
    Sync,
    /// Background execution. Evaluated asynchronously in the background.
    Background,
}

impl Default for EvalBudget {
    fn default() -> Self {
        EvalBudget::Sync
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub severity: String,
    pub pattern: String,
    pub path_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub message: String,
    pub rationale: String,
    #[serde(default)]
    pub eval_budget: EvalBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePack {
    pub id: String,
    pub version: String,
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}
```

### Validated Rule Set & Conflict Resolution
Enforces topological sorting of dependent rule packs and flags identical rule ID conflicts.

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct PackConflict {
    pub rule_id: String,
    pub pack_a: String,
    pub pack_b: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidatedRulePackSet {
    ordered: Vec<RulePack>,
}

impl ValidatedRulePackSet {
    pub fn new(packs: &[RulePack]) -> Result<Self, Vec<PackConflict>> {
        let mut ordered = Vec::new();
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        // Topological sorting DFS
        fn visit(
            id: &str,
            by_id: &HashMap<&str, &RulePack>,
            visited: &mut HashSet<String>,
            in_stack: &mut HashSet<String>,
            ordered: &mut Vec<RulePack>,
        ) {
            if visited.contains(id) {
                return;
            }
            if in_stack.contains(id) {
                return; // Cycle detected, degraded behavior
            }
            if let Some(pack) = by_id.get(id) {
                in_stack.insert(id.to_string());
                for dep in &pack.depends_on {
                    visit(dep, by_id, visited, in_stack, ordered);
                }
                in_stack.remove(id);
                visited.insert(id.to_string());
                ordered.push((*pack).clone());
            }
        }

        let by_id: HashMap<&str, &RulePack> = packs.iter().map(|p| (p.id.as_str(), p)).collect();
        for pack in packs {
            visit(&pack.id, &by_id, &mut visited, &mut in_stack, &mut ordered);
        }

        // Conflict check
        let mut seen = HashMap::new();
        let mut conflicts = Vec::new();
        for pack in &ordered {
            for rule in &pack.rules {
                if let Some(owner) = seen.insert(rule.id.as_str(), pack.id.as_str()) {
                    conflicts.push(PackConflict {
                        rule_id: rule.id.clone(),
                        pack_a: owner.to_string(),
                        pack_b: pack.id.clone(),
                    });
                }
            }
        }

        if conflicts.is_empty() {
            Ok(Self { ordered })
        } else {
            Err(conflicts)
        }
    }

    pub fn packs(&self) -> &[RulePack] {
        &self.ordered
    }
}
```

### Workspace Index & Cross-File Evaluation
```rust
use std::sync::Arc;
use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct IndexedDoc {
    pub content: String,
    pub version: i32,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceIndex {
    docs: Arc<DashMap<String, IndexedDoc>>,
}

impl WorkspaceIndex {
    pub fn upsert(&self, uri: String, content: String, version: i32) {
        self.docs.insert(uri, IndexedDoc { content, version, score: None });
    }

    pub fn remove(&self, uri: &str) {
        self.docs.remove(uri);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFileRule {
    pub id: String,
    pub name: String,
    pub severity: String,
    pub source_glob: String,
    pub source_pattern: String,
    pub target_glob: String,
    pub target_pattern: String,
    pub message: String,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct CrossFileViolation {
    pub source_uri: String,
    pub line: u32,
    pub matched_text: String,
    pub rule: CrossFileRule,
}
```

### SPC Monitor & Circuit Breaker
* **SpcMonitor:** Uses moving averages and standard deviation boundaries to trigger alerts when evaluation latencies drift.
* **CircuitBreaker:** Temporarily disables rule matching (returning empty results) if regex operations timeout or continuously fail.

```rust
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpcAlert {
    Rule1(f64), // Latency exceeds 3 standard deviations
    Rule2,      // Structural latency drift
}

#[derive(Debug, Default)]
pub struct SpcMonitor {
    history: Vec<f64>,
}

impl SpcMonitor {
    pub fn push(&mut self, latency_ms: f64) -> Option<SpcAlert> {
        self.history.push(latency_ms);
        if self.history.len() < 10 {
            return None;
        }
        let mean = self.history.iter().sum::<f64>() / self.history.len() as f64;
        let variance = self.history.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.history.len() as f64;
        let std_dev = variance.sqrt();

        if latency_ms > mean + 3.0 * std_dev {
            Some(SpcAlert::Rule1(latency_ms))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct CircuitBreaker {
    failures: usize,
    open: bool,
    last_failure: std::time::Instant,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            failures: 0,
            open: false,
            last_failure: std::time::Instant::now(),
        }
    }
}

impl CircuitBreaker {
    pub fn is_allowed(&mut self) -> bool {
        if self.open {
            if self.last_failure.elapsed() > Duration::from_secs(30) {
                self.open = false; // Half-open
                self.failures = 0;
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn record_failure(&mut self) {
        self.failures += 1;
        if self.failures >= 3 {
            self.open = true;
            self.last_failure = std::time::Instant::now();
        }
    }

    pub fn record_success(&mut self) {
        if self.failures > 0 {
            self.failures -= 1;
        }
    }
}
```

---

## 3. `ggen` µ-Pipeline Stages

The `ggen` pipeline translates declarative semantic graphs (RDF/TTL) into code artifacts ($A = \mu(O^*)$). It enforces exact ordering, compile-time validation, and commit isolation.

```text
Load Ontology (TTL) 
  → Execute Inference Rules (CONSTRUCT) 
  → SHACL & SPARQL ASK Validation Invariants
  → Execute Generation Rules (SELECT -> Tera template rendering)
  → Poka-Yoke file validation & Unsafe audit check
  → Atomic Commit (FileTransaction) & Receipt (BLAKE3) Generation
```

### Pipeline Structural Engine
```rust
use std::path::PathBuf;
use std::time::Instant;

pub struct PipelineState {
    pub executed_rules: Vec<ExecutedRule>,
    pub generated_files: Vec<GeneratedFile>,
    pub validation_results: Vec<ValidationResult>,
    pub started_at: Instant,
}

pub struct GenerationPipeline {
    manifest_path: PathBuf,
    executed_rules: Vec<ExecutedRule>,
    generated_files: Vec<GeneratedFile>,
    validation_results: Vec<ValidationResult>,
    started_at: Instant,
}

pub struct ExecutedRule {
    pub name: String,
    pub triples_added: usize,
    pub duration_ms: u64,
    pub query_hash: String,
}

pub struct GeneratedFile {
    pub path: PathBuf,
    pub content_hash: String,
    pub size_bytes: usize,
    pub source_rule: String,
}

pub struct ValidationResult {
    pub rule_name: String,
    pub passed: bool,
    pub message: Option<String>,
    pub severity: String,
}
```

### Stage 1: Loading
Loads the Turtle graph files (`.ttl`) dynamically into an oxigraph-backed memory `Graph`.

```rust
impl GenerationPipeline {
    pub fn load_ontology(&self, paths: &[PathBuf]) -> Result<oxigraph::store::Store, String> {
        let store = oxigraph::store::Store::new().map_err(|e| e.to_string())?;
        for path in paths {
            let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            store.load(
                content.as_bytes(),
                oxigraph::model::Syntax::Turtle,
                oxigraph::model::GraphName::DefaultGraph,
                None
            ).map_err(|e| e.to_string())?;
        }
        Ok(store)
    }
}
```

### Stage 2: Construct (Inference)
Materializes implicit triples using SPARQL `CONSTRUCT` queries. Crucially enforces the `GGEN-INFER-001` law: in strict mode, any rule returning zero new triples aborts compilation immediately.

```rust
impl GenerationPipeline {
    pub fn execute_inference_rules(
        &mut self,
        store: &oxigraph::store::Store,
        construct_queries: &[(String, String)], // (Rule Name, Query Text)
        strict_mode: bool,
    ) -> Result<(), String> {
        for (name, query) in construct_queries {
            let start = Instant::now();
            // In a real construct execution, we execute query and write outputs back
            let added_triples = self.run_construct_and_materialize(store, query)?;
            
            if added_triples == 0 && strict_mode {
                return Err(format!(
                    "error[GGEN-INFER-001]: Inference rule '{}' materialized 0 triples. Aborting under strict_mode.",
                    name
                ));
            }
            
            self.executed_rules.push(ExecutedRule {
                name: name.clone(),
                triples_added: added_triples,
                duration_ms: start.elapsed().as_millis() as u64,
                query_hash: blake3::hash(query.as_bytes()).to_string(),
            });
        }
        Ok(())
    }

    fn run_construct_and_materialize(&self, _store: &oxigraph::store::Store, _query: &str) -> Result<usize, String> {
        // Oxigraph construct evaluation and inserts ...
        Ok(0) // Mock representation
    }
}
```

### Stage 3: Validation (SHACL & SPARQL ASK)
* **SHACL Validation:** Evaluates SHACL constraints over the materialized store.
* **SPARQL ASK Polarity:** Assesses custom invariants. Unlike typical test logic, the query pattern expresses a valid state assertion. Polarity: `ASK = true` denotes passing validation; `ASK = false` raises a violation.

```rust
impl GenerationPipeline {
    pub fn validate_invariants(
        &mut self,
        store: &oxigraph::store::Store,
        ask_rules: &[(String, String, String)], // (Name, ASK Query, Description)
    ) -> Result<(), String> {
        let mut errors = Vec::new();
        for (name, query, desc) in ask_rules {
            let results = store.query(query).map_err(|e| e.to_string())?;
            let conforms = match results {
                oxigraph::sparql::QueryResults::Boolean(b) => b,
                _ => return Err("Validation ASK query must return a boolean".to_string()),
            };

            self.validation_results.push(ValidationResult {
                rule_name: name.clone(),
                passed: conforms,
                message: if conforms { None } else { Some(desc.clone()) },
                severity: "Error".to_string(),
            });

            if !conforms {
                errors.push(format!("Validation rule '{}' failed: {}", name, desc));
            }
        }

        if !errors.is_empty() {
            return Err(format!(
                "error[GGEN-VALIDATION]: Invariant check failed:\n  - {}\n  = generation aborted before writing files",
                errors.join("\n  - ")
            ));
        }
        Ok(())
    }
}
```

### Stage 4: Extract (SELECT)
Queries variables for lowering. The pipeline enforces that all SELECT queries must carry an `ORDER BY` clause to prevent Cartesian randomness and ensure compiler determinism.

```rust
impl GenerationPipeline {
    pub fn extract_bindings(
        &self,
        store: &oxigraph::store::Store,
        select_query: &str,
    ) -> Result<Vec<BTreeMap<String, String>>, String> {
        // Enforce strict determinism rule
        if !select_query.to_lowercase().contains("order by") {
            return Err("error[E0003]: SELECT query must contain an 'ORDER BY' clause for strict determinism".to_string());
        }
        
        // Execute query and capture rows ...
        Ok(Vec::new())
    }
}
```

### Stage 5: Template Lowering
Iterates over SPARQL bindings and renders templates via Tera.
* **Static Fold Mode:** Consolidates all rows into a single `results` context to output one aggregate file.
* **Dynamic Fan-out Mode:** Triggered if `output_file` contains placeholder variables (e.g. `{{ name }}`). Renders and writes one file per row.
* **LLM Skill Integration:** Checks for `?system_prompt` and `?skill_name`. Calls LLM services dynamically to populate `{{ generated_impl }}`; falls back to a deterministic comment stub if the LLM fails.

```rust
pub trait LlmService: Send + Sync {
    fn generate_impl(&self, name: &str, prompt: &str) -> Result<String, String>;
}

pub struct TemplateLowerer;

impl TemplateLowerer {
    pub fn lower_static(
        tera: &mut tera::Tera,
        template_name: &str,
        rows: &[BTreeMap<String, String>],
    ) -> Result<String, String> {
        let mut ctx = tera::Context::new();
        ctx.insert("results", rows);
        tera.render(template_name, &ctx).map_err(|e| e.to_string())
    }

    pub fn lower_dynamic(
        tera: &mut tera::Tera,
        template_name: &str,
        row: &BTreeMap<String, String>,
        llm: Option<&dyn LlmService>,
    ) -> Result<String, String> {
        let mut ctx = tera::Context::new();
        for (k, v) in row {
            ctx.insert(k, v);
        }

        // LLM Skill injection fallback
        if let (Some(name), Some(prompt)) = (row.get("skill_name"), row.get("system_prompt")) {
            let code = match llm {
                Some(svc) => svc.generate_impl(name, prompt).unwrap_or_else(|_| {
                    format!("// [TemplateFallback] LLM generation failed for {}", name)
                }),
                None => format!("// [TemplateFallback] LLM not configured for {}", name),
            };
            ctx.insert("generated_impl", &code);
        }

        tera.render(template_name, &ctx).map_err(|e| e.to_string())
    }
}
```

### Stage 6: Receipts & Poka-Yoke
Enforces rigorous validation on rendered outputs:
1. **NotEmpty:** Aborts if output length is 0.
2. **Size Bounds:** Rejects files exceeding 10MB.
3. **Poka-Yoke (Anti-Traversal):** Rejects paths referencing relative parent directories (`../`).
4. **Safety Check:** If strict validation is set, scans output for `unsafe` keywords.
5. **BLAKE3 Verification:** Commits outputs atomically using a file transaction containing BLAKE3 content-addressing hashes.

```rust
impl GenerationPipeline {
    pub fn validate_file_safety(content: &str, path: &std::path::Path, check_unsafe: bool) -> Result<(), String> {
        if content.is_empty() {
            return Err("error[E0004]: Render output is empty".to_string());
        }
        if content.len() > 10 * 1024 * 1024 {
            return Err("error[E0005]: Render output exceeds 10MB limit".to_string());
        }
        if path.to_string_lossy().contains("../") || path.to_string_lossy().contains("..\\") {
            return Err("error[E0006]: Directory traversal sequence detected".to_string());
        }
        if check_unsafe && content.contains("unsafe ") {
            return Err("error[E0007]: Unsafe Rust code detected under strict validation rules".to_string());
        }
        Ok(())
    }
}
```

---

## 4. Praxis Integration Plan

To implement these patterns inside `praxis/template` (scaffolded via `cargo generate`), we specify the following modular package design:

### Cargo.toml Dependencies and Feature Flags
Provide optimal, zero-overhead settings using optional features.

```toml
[package]
name = "praxis-template"
version = "0.2.0"
edition = "2021"

[features]
default = []
typestate = []
lsp = ["dep:lsp-max", "dep:tree-sitter", "dep:tokio", "dep:dashmap", "dep:parking_lot"]
ggen = ["dep:oxigraph", "dep:tera", "dep:blake3", "dep:serde", "dep:serde_json"]

[dependencies]
# Shared primitives
tokio = { version = "1.35", features = ["full"], optional = true }
dashmap = { version = "5.5", optional = true }
parking_lot = { version = "0.12", optional = true }

# LSP integration
lsp-max = { git = "https://github.com/seanchatmangpt/lsp-max", optional = true }
tree-sitter = { version = "0.20", optional = true }

# ggen integration
oxigraph = { version = "0.3", optional = true }
tera = { version = "1.19", optional = true }
blake3 = { version = "1.5", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
```

### Module File Layout
Every generated crate includes standard hooks in `src/`:

```text
praxis-template/
├── Cargo.toml
├── ggen.toml               <-- Boilerplate generator configuration
├── ontology/
│   └── domain.ttl          <-- Baseline domain ontology definitions
├── queries/
│   ├── inference.rq        <-- CONSTRUCT rules
│   └── extraction.rq       <-- SELECT code extraction queries
└── src/
    ├── lib.rs
    ├── types.rs            <-- Evidence wrappers, Witness traits, ZSTs
    ├── lsp.rs              <-- AppLspServer : RulePackServer scaffolding
    └── generator.rs        <-- Local ggen runner (oxigraph wrapper)
```

### Scaffold Boilerplate for Developer Use
The code skeletons are generated directly into the target crate to facilitate instant compliance.

#### `src/types.rs`
```rust
use std::marker::PhantomData;

pub struct Raw;
pub struct Admitted;

pub struct Evidence<T, State, Witness> {
    inner: T,
    _state: PhantomData<State>,
    _witness: PhantomData<Witness>,
}

impl<T, W> Evidence<T, Raw, W> {
    pub fn raw(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}

impl<T, W> Evidence<T, Admitted, W> {
    pub fn inner(&self) -> &T { &self.inner }
    pub(crate) fn admit_unchecked(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}

pub trait Admit {
    type Input;
    type Witness;
    type Reason;

    fn admit(
        input: Evidence<Self::Input, Raw, Self::Witness>,
    ) -> Result<Evidence<Self::Input, Admitted, Self::Witness>, Self::Reason>;
}
```

#### `src/lsp.rs`
```rust
#[cfg(feature = "lsp")]
pub mod server {
    use lsp_max::rule_pack_server::{RulePackServer, ValidatedRulePackSet, WorkspaceIndex, PrimitivesBundle};
    use lsp_max::service::Client;
    use lsp_max_ast::AutoLspAdapter;
    use tree_sitter::Language;

    pub struct AppLspServer {
        packs: ValidatedRulePackSet,
        grammar: Language,
        adapter: AutoLspAdapter,
        client: Client,
        index: WorkspaceIndex,
        primitives: PrimitivesBundle,
    }

    impl RulePackServer for AppLspServer {
        fn rule_packs(&self) -> &ValidatedRulePackSet { &self.packs }
        fn grammar(&self) -> Language { self.grammar.clone() }
        fn server_name(&self) -> &'static str { "app-lsp-server" }
        fn client(&self) -> &Client { &self.client }
        fn adapter(&self) -> &AutoLspAdapter { &self.adapter }
        fn workspace_index(&self) -> Option<&WorkspaceIndex> { Some(&self.index) }
        
        fn spc_monitor(&self) -> Option<&std::sync::Mutex<lsp_max::primitives::SpcMonitor>> {
            Some(self.primitives.spc_monitor_ref())
        }
        fn latency_trackers(&self) -> Option<&std::sync::Arc<dashmap::DashMap<String, lsp_max::primitives::RuleLatencyTracker>>> {
            Some(self.primitives.latency_trackers_ref())
        }
        fn rule_circuit_breaker(&self) -> Option<&std::sync::Arc<parking_lot::Mutex<lsp_max::primitives::CircuitBreaker>>> {
            Some(self.primitives.circuit_breaker_ref())
        }
    }
}
```
