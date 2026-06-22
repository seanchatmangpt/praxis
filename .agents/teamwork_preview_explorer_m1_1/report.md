# Ecosystem Abstraction & Architectural Design Report: Typestates, RulePackServer, and ggen Pipeline

**Author**: Ecosystem Cataloger 1  
**Working Directory**: `/Users/sac/praxis/.agents/teamwork_preview_explorer_m1_1`  
**Status**: PARTIAL_ALIVE candidate  
**Object under test**: Rust abstractions in `rocket-craft` and `lsp-max` for migration to `praxis/template`  
**Observed evidence**: Trait and struct definitions verified in `/Users/sac/rocket-craft/crates/mech_morphology_law/src/machine.rs`, `/Users/sac/lsp-max/src/rule_pack_server.rs`, `/Users/sac/lsp-max/src/primitives/spc.rs`, `/Users/sac/lsp-max/src/primitives/circuit_breaker.rs`, and `/Users/sac/ggen/crates/ggen-core/src/codegen/pipeline.rs`.  

---

## 1. Executive Summary

This report catalogues and abstracts the foundational design patterns of the post-Chatman ecosystem discovered within the `rocket-craft`, `lsp-max`, and `ggen` codebases. These patterns are designed to enforce architectural correctness, guarantee compile-time invariants, prevent runtime state violations, and streamline asynchronous evaluation workflows.

We identify three core components to extract, formalize, and inject into the `praxis` boilerplate templates:
1. **Generative Typestates**: Zero-Sized marker types and gating transitions that enforce phase state mutations at compile-time, combined with the Witness and Seal patterns to secure constructor purity.
2. **`RulePackServer`**: A trait-driven protocol abstraction for LSP servers which integrates latency budget classification (`EvalBudget`), cross-file workspace indexing (`WorkspaceIndex`), sliding-window statistical process control (`SpcMonitor`), and rule execution protection (`CircuitBreaker`).
3. **`ggen` Micro-Pipeline**: An ontological code-generation model that Lowerers Turtle graphs to compile-ready files in six stages, utilizing Oxigraph, SHACL/SPARQL validation, Tera templating, and atomic filesystem transactions.

---

## 2. Generative Typestates

The typestate pattern guarantees that operations are only available to objects in valid phases. By consuming `self` at each transition, old states cannot be reused.

### A. Zero-Sized Marker Structs
Marker structs carry no runtime weight. They only exist to instruct the compiler's generic system.
```rust
/// Sealed trait to prevent external implementation of lifecycle states
pub mod sealed {
    pub trait LifecycleState {}
}

/// Zero-Sized marker representing unvalidated, raw data.
#[derive(Debug, Clone, Copy)]
pub struct Raw;
impl sealed::LifecycleState for Raw {}

/// Zero-Sized marker representing validated phase.
#[derive(Debug, Clone, Copy)]
pub struct Validated;
impl sealed::LifecycleState for Validated {}

/// Zero-Sized marker representing admitted, terminal state.
#[derive(Debug, Clone, Copy)]
pub struct Admitted;
impl sealed::LifecycleState for Admitted {}
```

### B. Gated Transitions
Transitions are implemented using generic type constraints. Constructing the initial phase is unrestricted, but moving to validated and admitted states requires executing the appropriate checks.
```rust
use std::marker::PhantomData;

/// The typestate container enforcing compile-time state boundaries.
#[derive(Debug)]
pub struct Evidence<T, S: sealed::LifecycleState, W> {
    inner: T,
    _state: PhantomData<S>,
    _witness: PhantomData<W>,
}

impl<T, W> Evidence<T, Raw, W> {
    /// Create initial raw evidence.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _state: PhantomData,
            _witness: PhantomData,
        }
    }

    /// Retrieve the underlying payload in its raw form.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}
```

### C. Gating with GMF/DOD Outcomes
The transition from `Validated` to `Admitted` is gated by a runtime outcome check. In our mecha morphology implementation, we return a `ClaimHold` when validation fails, preserving the state machine in a non-admitted status.
```rust
/// The outcome of domain law validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    Admitted,
    PartialAlive { issues: Vec<String> },
    Refused { issues: Vec<String> },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ValidationOutcome {
    pub standing: Standing,
    pub refusals: Vec<String>,
}

/// A runtime claim-hold indicating that admission has been deferred.
#[derive(Debug, Clone)]
pub struct ClaimHold {
    pub standing: Standing,
    pub reason: Vec<String>,
}

impl<T, W> Evidence<T, Validated, W> {
    /// Attempt to admit the validated evidence. 
    /// Consumes the validated state; returns `Ok(Evidence<T, Admitted, W>)` 
    /// if the standing is admitted, or `Err(ClaimHold)` to hold the transaction.
    pub fn admit(self, outcome: &ValidationOutcome) -> Result<Evidence<T, Admitted, W>, ClaimHold> {
        match &outcome.standing {
            Standing::Admitted => Ok(Evidence {
                inner: self.inner,
                _state: PhantomData,
                _witness: PhantomData,
            }),
            other => Err(ClaimHold {
                standing: other.clone(),
                reason: outcome.refusals.clone(),
            }),
        }
    }
}
```

### D. The Witness & Seal Patterns
- **Witness Pattern**: Restricts transition authorization. The constructor `admit_unchecked` is restricted using `pub(crate)` visibility so only the validating compiler module (holding the witness handle `W`) can elevate a type to `Admitted`.
- **Seal Pattern**: Prevents consumers from bypassing constructor rules by injecting a private field.
```rust
/// The Seal Pattern: prevents external crates from constructing the struct directly.
pub struct AdmittedReceipt {
    pub chain_hash: [u8; 32],
    pub timestamp: u64,
    _seal: (), // Private field prevents struct-literal instantiation
}

impl AdmittedReceipt {
    /// Public constructor enforcing the seal.
    pub fn new(hash: [u8; 32], ts: u64) -> Self {
        Self {
            chain_hash: hash,
            timestamp: ts,
            _seal: (),
        }
    }
}

/// The Witness Pattern: Generic parameter `W` acts as proof of validator authority.
pub trait ValidateAuthority {
    type Input;
    type Witness;
    type Error;

    fn validate(&self, input: Self::Input) -> Result<Evidence<Self::Input, Validated, Self::Witness>, Self::Error>;
}
```

---

## 3. `RulePackServer` Structures

The `RulePackServer` trait abstracts LSP mechanics, allowing language servers to sync documents, run rules, and push diagnostics without manual tower-lsp boilerplate.

```rust
use std::sync::Arc;
use std::collections::HashMap;
use dashmap::DashMap;
use parking_lot::Mutex;

/// custom LSP severities mapping to domain law axes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LawAxis {
    Domain,
    Protocol,
    Documentation,
    Fixture,
    Custom(String),
}

/// The main RulePackServer abstraction to be added to praxis templates.
pub trait RulePackServer: Send + Sync + 'static {
    type Document;
    type Diagnostic;

    fn server_name(&self) -> &'static str;
    fn workspace_index(&self) -> Option<&WorkspaceIndex>;
    
    // Safety components
    fn spc_monitor(&self) -> Option<&Mutex<SpcMonitor>>;
    fn latency_trackers(&self) -> Option<&Arc<DashMap<String, RuleLatencyTracker>>>;
    fn rule_circuit_breaker(&self) -> Option<&Arc<Mutex<CircuitBreaker>>>;

    /// Process document edits, classification and diagnostics.
    fn scan_document(&self, uri: &str, content: &str) -> Vec<Self::Diagnostic>;
}
```

### A. `EvalBudget` (Latency Classification)
Rules are classified into `Sync` and `Background` to prevent main-thread editor lockups. Sync rules must run under ~50ms. If a Sync rule executes slowly three consecutive times, it is dynamically promoted to Background.
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvalBudget {
    #[default]
    Sync,
    Background,
}

#[derive(Debug, Clone)]
pub struct RuleLatencyTracker {
    pub rule_name: String,
    pub consecutive_slow_runs: u32,
    pub last_duration_ms: f64,
}

impl RuleLatencyTracker {
    pub fn record_run(&mut self, duration_ms: f64, budget: &mut EvalBudget) {
        self.last_duration_ms = duration_ms;
        if *budget == EvalBudget::Sync && duration_ms > 50.0 {
            self.consecutive_slow_runs += 1;
            if self.consecutive_slow_runs >= 3 {
                *budget = EvalBudget::Background;
                log::warn!(
                    "RulePackServer: Promoting slow sync rule '{}' to Background (consecutive slow: {})",
                    self.rule_name,
                    self.consecutive_slow_runs
                );
            }
        } else {
            self.consecutive_slow_runs = 0;
        }
    }
}
```

### B. `WorkspaceIndex` & Cross-File Rules
For global check coherence, `WorkspaceIndex` registers files and symbols across the workspace.
```rust
pub struct WorkspaceIndex {
    documents: Arc<DashMap<String, IndexedDocument>>,
}

pub struct IndexedDocument {
    pub uri: String,
    pub content_hash: [u8; 32],
    pub symbols: HashMap<String, SymbolDefinition>,
}

pub struct SymbolDefinition {
    pub name: String,
    pub kind: String,
    pub location: String,
}

impl WorkspaceIndex {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(DashMap::new()),
        }
    }

    pub fn insert_document(&self, uri: String, doc: IndexedDocument) {
        self.documents.insert(uri, doc);
    }

    pub fn remove_document(&self, uri: &str) {
        self.documents.remove(uri);
    }
}
```

### C. `SpcMonitor` (Statistical Process Control)
`SpcMonitor` tracks the performance of document scanning. It implements a sliding window of size 50 and updates statistical metrics (mean, variance) online via Welford's algorithm. It triggers alerts based on Western Electric rules to detect system anomalies.
```rust
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum SpcAlert {
    /// Rule 1: Point is beyond ±3 standard deviations.
    Rule1(f64),
    /// Rule 2: 9 consecutive points on the same side of the mean.
    Rule2,
    /// Rule 3: 6 consecutive points monotonically increasing or decreasing.
    Rule3,
    /// Rule 4: 2 of 3 consecutive points beyond ±2 standard deviations on the same side.
    Rule4,
}

#[derive(Debug)]
pub struct SpcMonitor {
    window: VecDeque<f64>,
    mean: f64,
    m2: f64, // Sum of squares of differences from the current mean
    count: u64,
}

impl SpcMonitor {
    pub fn new() -> Self {
        Self {
            window: VecDeque::with_capacity(50),
            mean: 0.0,
            m2: 0.0,
            count: 0,
        }
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn std_dev(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        (self.m2 / (self.count - 1) as f64).sqrt()
    }

    pub fn push(&mut self, sample_ms: f64) -> Option<SpcAlert> {
        let base_mean = self.mean;
        let base_sd = self.std_dev();
        let base_n = self.window.len();

        // 1. Welford's online update
        self.count += 1;
        let delta = sample_ms - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = sample_ms - self.mean;
        self.m2 += delta * delta2;

        self.window.push_back(sample_ms);
        if self.window.len() > 50 {
            self.window.pop_front();
            // Recompute stats on window eviction to avoid float drift
            let n = self.window.len();
            let mean: f64 = self.window.iter().sum::<f64>() / n as f64;
            let m2: f64 = self.window.iter().map(|&x| (x - mean).powi(2)).sum();
            self.mean = mean;
            self.m2 = m2;
            self.count = n as u64;
        }

        if base_n < 2 || base_sd == 0.0 {
            return None;
        }

        // Rule 1: Point beyond ±3σ
        if (sample_ms - base_mean).abs() > 3.0 * base_sd {
            return Some(SpcAlert::Rule1(sample_ms));
        }

        if self.window.len() < 9 {
            return None;
        }

        let recent: Vec<f64> = self.window.iter().copied().rev().take(9).rev().collect();

        // Rule 4: 2 of 3 beyond ±2σ
        {
            let last_3 = &recent[6..];
            let above = last_3.iter().filter(|&&v| v > base_mean + 2.0 * base_sd).count();
            let below = last_3.iter().filter(|&&v| v < base_mean - 2.0 * base_sd).count();
            if above >= 2 || below >= 2 {
                return Some(SpcAlert::Rule4);
            }
        }

        // Rule 2: 9 consecutive same side of mean
        {
            let above = recent.iter().filter(|&&v| v > base_mean).count();
            let below = recent.iter().filter(|&&v| v < base_mean).count();
            if above == 9 || below == 9 {
                return Some(SpcAlert::Rule2);
            }
        }

        // Rule 3: 6 consecutive monotone
        {
            let last_6 = &recent[3..];
            let incr = last_6.windows(2).all(|w| w[1] > w[0]);
            let decr = last_6.windows(2).all(|w| w[1] < w[0]);
            if incr || decr {
                return Some(SpcAlert::Rule3);
            }
        }

        None
    }
}
```

### D. `CircuitBreaker` (Loop Protection State Machine)
The rule evaluation loop is protected by a circuit breaker. If evaluation fails repeatedly (e.g. tree-sitter panic, stack overflow), it trips to prevent blocking the LSP.
```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    HalfOpen,
    Open,
}

pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<Instant>,
    cooldown: Duration,
    failure_threshold: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            cooldown,
            failure_threshold,
        }
    }

    pub fn is_allowed(&mut self) -> bool {
        match self.state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                let elapsed = self.last_failure.map(|t| t.elapsed()).unwrap_or(Duration::MAX);
                if elapsed >= self.cooldown {
                    self.state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn record_success(&mut self) {
        if self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Closed;
            self.failure_count = 0;
            self.last_failure = None;
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen | CircuitState::Open => {
                self.state = CircuitState::Open;
                self.last_failure = Some(Instant::now());
            }
        }
    }
}
```

---

## 4. `ggen` µ-Pipeline Stages

The ontology-driven code generation pipeline follows six distinct phases.

```
                  ┌────────────────────────┐
                  │ 1. Load Ontology (TTL) │
                  └───────────┬────────────┘
                              ▼
                  ┌────────────────────────┐
                  │ 2. Construct Inference │
                  └───────────┬────────────┘
                              ▼
                  ┌────────────────────────┐
                  │ 3. Validation SHACL/ASK│
                  └───────────┬────────────┘
                              ▼
                  ┌────────────────────────┐
                  │  4. Extract (SELECT)   │
                  └───────────┬────────────┘
                              ▼
                  ┌────────────────────────┐
                  │  5. Template Lowering  │
                  └───────────┬────────────┘
                              ▼
                  ┌────────────────────────┐
                  │ 6. Receipt & Commit    │
                  └────────────────────────┘
```

### Stage 1: Loading (`load_ontology`)
Turtle files containing the schema definition and instance data are loaded into an Oxigraph-backed in-memory database.
```rust
use oxigraph::store::Store;
use oxigraph::model::GraphName;
use std::io::Cursor;

pub fn load_ontology(sources: &[String], imports: &[String]) -> Result<Store, String> {
    let store = Store::new().map_err(|e| e.to_string())?;
    for src in sources {
        let content = std::fs::read_to_string(src).map_err(|e| e.to_string())?;
        store.load(
            Cursor::new(content),
            oxigraph::io::RdfFormat::Turtle,
            GraphName::DefaultGraph,
            None,
        ).map_err(|e| e.to_string())?;
    }
    for import in imports {
        let content = std::fs::read_to_string(import).map_err(|e| e.to_string())?;
        store.load(
            Cursor::new(content),
            oxigraph::io::RdfFormat::Turtle,
            GraphName::DefaultGraph,
            None,
        ).map_err(|e| e.to_string())?;
    }
    Ok(store)
}
```

### Stage 2: Construct / Inference (`execute_inference_rules`)
SPARQL `CONSTRUCT` queries execute recursively to materialize derived facts (e.g. socket mappings, joint links). Conditional execution is supported via `when` guards. The pipeline rejects CONSTRUCT queries that yield 0 triples in strict mode to prevent query drift (`GGEN-INFER-001`).
```rust
pub struct InferenceRule {
    pub name: String,
    pub when: Option<String>,
    pub construct: String,
}

pub fn execute_inference(store: &Store, rules: &[InferenceRule], strict_mode: bool) -> Result<(), String> {
    for rule in rules {
        // Evaluate the when clause if present
        if let Some(ref ask) = rule.when {
            let query_res = store.query(ask).map_err(|e| e.to_string())?;
            if let oxigraph::sparql::QueryResults::Boolean(false) = query_res {
                continue; // Skip execution
            }
        }

        let construct_res = store.query(&rule.construct).map_err(|e| e.to_string())?;
        if let oxigraph::sparql::QueryResults::Solutions(_) = construct_res {
            return Err("Inference query must be CONSTRUCT, not SELECT".into());
        }

        // Materialize triples in the store
        let mut added = 0;
        if let oxigraph::sparql::QueryResults::Graph(graph_iter) = construct_res {
            for quad in graph_iter {
                let q = quad.map_err(|e| e.to_string())?;
                if store.insert(&q.clone().into()).map_err(|e| e.to_string())? {
                    added += 1;
                }
            }
        }

        if added == 0 {
            let msg = format!("GGEN-INFER-001: Inference rule '{}' produced 0 new triples.", rule.name);
            if strict_mode {
                return Err(msg);
            } else {
                log::warn!("{}", msg);
            }
        }
    }
    Ok(())
}
```

### Stage 3: Validation (`execute_shacl_validation`, `execute_validation_rules`)
Validates structural invariants using SHACL files (evaluating node shapes, minimum counts) and custom SPARQL `ASK` rules.
- Custom ASK rules check for database invariants: an `ASK` result of `true` means a violation exists.
- Any violation with `Error` severity halts the pipeline before writing files to the disk.
```rust
pub struct ValidationRule {
    pub name: String,
    pub ask: String,
    pub description: String,
    pub is_error: bool,
}

pub fn execute_validation(store: &Store, rules: &[ValidationRule]) -> Result<(), String> {
    let mut errors = Vec::new();
    for rule in rules {
        let results = store.query(&rule.ask).map_err(|e| e.to_string())?;
        if let oxigraph::sparql::QueryResults::Boolean(true) = results {
            let msg = format!("Validation violation on rule '{}': {}", rule.name, rule.description);
            if rule.is_error {
                errors.push(msg);
            } else {
                log::warn!("{}", msg);
            }
        }
    }
    if !errors.is_empty() {
        return Err(format!("Validation failed with {} error(s):\n{}", errors.len(), errors.join("\n")));
    }
    Ok(())
}
```

### Stage 4: Extract (`execute_generation_rules`)
Executes SPARQL `SELECT` queries to retrieve variables for template mapping. The extraction enforces strict determinism by requiring an `ORDER BY` clause on queries to avoid floating-point / list ordering changes in parallel execution.
```rust
pub fn extract_generation_bindings(store: &Store, query: &str) -> Result<Vec<HashMap<String, String>>, String> {
    if !query.to_uppercase().contains("ORDER BY") {
        return Err("SPARQL query requires an ORDER BY clause to guarantee compilation determinism".into());
    }

    let results = store.query(query).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
        for sol in solutions {
            let s = sol.map_err(|e| e.to_string())?;
            let mut row = HashMap::new();
            for (var, term) in s.iter() {
                row.insert(var.to_string(), term.to_string());
            }
            rows.push(row);
        }
    } else {
        return Err("Generation query must be SELECT, not ASK/CONSTRUCT".into());
    }
    Ok(rows)
}
```

### Stage 5: Template Lowering
The extracted rows are rendered into templates (e.g. Tera).
- **Static Output Path**: The entire collection of rows is exposed as a `results` list in the Tera context, rendering a single merged file.
- **Dynamic Output Path**: If the output file path contains placeholder variables (e.g. `{{part_name}}.rs`), the pipeline iterates over rows, rendering one file per row.
If a skill requires custom code and the compiler lacks an active LLM connection, it drops back to generating manual stubs using `TemplateFallback`.
```rust
pub struct TeraRenderer {
    tera: tera::Tera,
}

impl TeraRenderer {
    pub fn render_static(&self, template: &str, rows: &[HashMap<String, String>]) -> Result<String, String> {
        let mut context = tera::Context::new();
        context.insert("results", &rows);
        if !rows.is_empty() {
            // Expose the first row variables directly for easy access
            for (key, val) in &rows[0] {
                context.insert(key, val);
            }
        }
        self.tera.render_str(template, &context).map_err(|e| e.to_string())
    }

    pub fn render_dynamic(&self, template: &str, row: &HashMap<String, String>) -> Result<String, String> {
        let mut context = tera::Context::new();
        for (key, val) in row {
            context.insert(key, val);
        }
        self.tera.render_str(template, &context).map_err(|e| e.to_string())
    }
}
```

### Stage 6: Receipts & Atomic Commits (`FileTransaction`)
Files are written atomically using a temporary file rename strategy. Backups are created for existing files. A `Drop` hook guarantees rollback if any error occurs before commit. On a successful run, a BLAKE3 receipt hash is generated from the generated file contents.
```rust
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub enum FileOp {
    Create(PathBuf),
    Modify(PathBuf, PathBuf), // (target_path, backup_path)
}

pub struct FileTransaction {
    ops: Mutex<Vec<FileOp>>,
    committed: std::sync::atomic::AtomicBool,
}

impl FileTransaction {
    pub fn new() -> Self {
        Self {
            ops: Mutex::new(Vec::new()),
            committed: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        // Enforce Poka-Yoke boundaries: check size, emptiness, and path traversal
        if content.is_empty() {
            return Err("Poka-Yoke: Generated file content is empty".into());
        }
        if content.len() > 10 * 1024 * 1024 {
            return Err("Poka-Yoke: File size exceeds 10MB limit".into());
        }
        if path.to_string_lossy().contains("../") {
            return Err("Poka-Yoke: Path traversal detected".into());
        }

        let existed = path.exists();
        let backup_path = if existed {
            let mut backup = path.to_path_buf();
            backup.set_extension("backup");
            fs::copy(path, &backup).map_err(|e| e.to_string())?;
            Some(backup)
        } else {
            None
        };

        // Atomic write via temp file
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let temp_path = parent.join(format!(".temp_{}", uuid::Uuid::new_v4()));
        {
            let mut f = File::create(&temp_path).map_err(|e| e.to_string())?;
            f.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        }

        fs::rename(&temp_path, path).map_err(|e| e.to_string())?;

        let mut ops = self.ops.lock();
        if let Some(backup) = backup_path {
            ops.push(FileOp::Modify(path.to_path_buf(), backup));
        } else {
            ops.push(FileOp::Create(path.to_path_buf()));
        }

        Ok(())
    }

    pub fn commit(self) -> Result<String, String> {
        self.committed.store(true, std::sync::atomic::Ordering::SeqCst);
        let mut hasher = blake3::Hasher::new();
        let ops = self.ops.lock();
        for op in ops.iter() {
            let path = match op {
                FileOp::Create(p) => p,
                FileOp::Modify(p, backup) => {
                    // Clean up backups on successful commit
                    let _ = fs::remove_file(backup);
                    p
                }
            };
            let content = fs::read(path).map_err(|e| e.to_string())?;
            hasher.update(&content);
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn rollback(&self) {
        if self.committed.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let mut ops = self.ops.lock();
        while let Some(op) = ops.pop() {
            match op {
                FileOp::Create(p) => {
                    let _ = fs::remove_file(p);
                }
                FileOp::Modify(p, backup) => {
                    let _ = fs::copy(&backup, &p);
                    let _ = fs::remove_file(backup);
                }
            }
        }
    }
}

impl Drop for FileTransaction {
    fn drop(&mut self) {
        self.rollback();
    }
}
```

---

## 5. Praxis Integration Plan

We propose extending the standard `praxis/template` generator with native features for Generative Typestates, `RulePackServer`, and `ggen` µ-pipelines.

### A. Template Code Layout
```
template/
├── Cargo.toml
├── ggen.toml
├── ontology/
│   ├── core.ttl
│   └── rules.rq
├── src/
│   ├── lib.rs
│   ├── types.rs  <-- Generative Typestate markers, Evidence, Admit trait
│   ├── lsp.rs    <-- RulePackServer tower-lsp implementation wrapper
│   └── ggen.rs   <-- Local ggen compiler integration module
└── templates/
    └── codegen_class.rs.tera
```

### B. Dependency & Feature Configurations
Inject these optional dependencies in the template's `Cargo.toml` so developers can choose their feature sets:
```toml
# template/Cargo.toml
[features]
default = ["typestate"]
typestate = []
lsp = ["dep:lsp-max", "dep:tree-sitter", "dep:tokio", "dep:dashmap", "dep:parking_lot"]
ggen = ["dep:oxigraph", "dep:tera", "dep:uuid", "dep:blake3"]

[dependencies]
# Typestate dependencies
serde = { version = "1.0", features = ["derive"] }

# LSP components
lsp-max = { git = "https://github.com/seanchatmangpt/lsp-max", optional = true }
tree-sitter = { version = "0.22", optional = true }
tokio = { version = "1", features = ["full"], optional = true }
dashmap = { version = "6", optional = true }
parking_lot = { version = "0.12", optional = true }

# ggen components
oxigraph = { version = "0.4", optional = true }
tera = { version = "1.19", optional = true }
uuid = { version = "1.6", features = ["v4"], optional = true }
blake3 = { version = "1.5", optional = true }
```

### C. Standardizing `types.rs` boilerplate
The generator will scaffold a clean typestate bridge inside `src/types.rs`:
```rust
// template/src/types.rs
use std::marker::PhantomData;

pub mod sealed {
    pub trait LifecycleState {}
}

pub struct Raw;
impl sealed::LifecycleState for Raw {}

pub struct Validated;
impl sealed::LifecycleState for Validated {}

pub struct Admitted;
impl sealed::LifecycleState for Admitted {}

pub struct Evidence<T, S: sealed::LifecycleState, W> {
    inner: T,
    _state: PhantomData<S>,
    _witness: PhantomData<W>,
}

impl<T, W> Evidence<T, Raw, W> {
    pub fn new(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
    pub fn inner(&self) -> &T { &self.inner }
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
    type Error;

    fn admit(
        input: Evidence<Self::Input, Raw, Self::Witness>,
    ) -> Result<Evidence<Self::Input, Admitted, Self::Witness>, Self::Error>;
}
```

### D. Standardizing `lsp.rs` boilerplate
If the `lsp` feature is enabled, a pre-configured `RulePackServer` boilerplate is scaffolded in `src/lsp.rs` that delegates tracking to `SpcMonitor` and `CircuitBreaker`.
```rust
// template/src/lsp.rs
use lsp_max::rule_pack_server::{RulePackServer, ValidatedRulePackSet, WorkspaceIndex};
use lsp_max::primitives::{SpcMonitor, CircuitBreaker, RuleLatencyTracker};
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct AppLspServer {
    name: &'static str,
    index: WorkspaceIndex,
    spc: Mutex<SpcMonitor>,
    cb: Arc<Mutex<CircuitBreaker>>,
    latencies: Arc<DashMap<String, RuleLatencyTracker>>,
}

impl RulePackServer for AppLspServer {
    type Document = String;
    type Diagnostic = lsp_types::Diagnostic;

    fn server_name(&self) -> &'static str {
        self.name
    }

    fn workspace_index(&self) -> Option<&WorkspaceIndex> {
        Some(&self.index)
    }

    fn spc_monitor(&self) -> Option<&Mutex<SpcMonitor>> {
        Some(&self.spc)
    }

    fn latency_trackers(&self) -> Option<&Arc<DashMap<String, RuleLatencyTracker>>> {
        Some(&self.latencies)
    }

    fn rule_circuit_breaker(&self) -> Option<&Arc<Mutex<CircuitBreaker>>> {
        Some(&self.cb)
    }

    fn scan_document(&self, _uri: &str, _content: &str) -> Vec<Self::Diagnostic> {
        let mut _cb = self.cb.lock();
        if !_cb.is_allowed() {
            return vec![]; // Circuit is open, skip scanning to protect host
        }
        
        let start = std::time::Instant::now();
        let diagnostics = vec![]; // Perform AST query evaluations
        
        let elapsed = start.elapsed().as_millis() as f64;
        let mut monitor = self.spc.lock();
        if let Some(alert) = monitor.push(elapsed) {
            log::warn!("SpcMonitor alert fired: {:?}", alert);
        }
        
        _cb.record_success();
        diagnostics
    }
}
```

---

## 6. Verification and Auditing

### Verified Aspects
1. **Compilation-Gated Typestates**: Confirmed in `crates/mech_morphology_law/src/machine.rs` that ZST transitions consume the previous stage to enforce linear mutations, and return `Result` structures carrying `ClaimHold` to represent the DOD default.
2. **RulePackServer & Safety Elements**: Verified that `SpcMonitor` utilizes Welford's online method for statistical evaluation of scan latency and checks for Western Electric rules. Verified that the `CircuitBreaker` manages loop stability.
3. **ggen Stage Executions**: Inspected the Oxigraph Turtle load, query classification execution, materialization construct steps, ASK custom validator rules, Tera template rendering, and FileTransaction atomic write/rollback drop hook.

### Unverified Aspects
1. The performance profile of the dynamic budget promotion under thousands of concurrent document edits (due to read-only constraint, this remains a simulation-level profile).
2. The interaction of SHACL shapes with custom SPARQL rule schemas in a full template-scaffolded test execution (to be verified in Milestone M3).

### Key Risks and Mitigations
- **Risk**: Adding `oxigraph` and `tree-sitter` inside templates may expand compilation overhead.  
  *Mitigation*: Enclose them behind optional `lsp` and `ggen` features inside `template/Cargo.toml`.
- **Risk**: If the `FileTransaction` fails during `commit()` after writing several files, deleting created files during rollback could delete files that weren't created by this pipeline.  
  *Mitigation*: Keep exact record lists in `FileOperation` of only the specific paths created or modified in the current transaction.

---

**Status:** PARTIAL_ALIVE  
**Object under test:** Ecosystem Abstraction Report (`report.md`)  
**Observed evidence:** Completed analysis of structural components. File generated.  
**Receipt required:** Compilation of the updated `praxis` generator templates with typestate constraints and LSP integrations, verified via the programmatic harness.  
**Residuals:** Verification of template code actions within a live IDE environment remains unproven.  
