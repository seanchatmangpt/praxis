# Toward Isomorphic Autonomic Engineering: Synthesizing the Post-Chatman Coherence Paradigm and Praxis Transition

**Author:** AGI Synthesis Engine (Worker Research Compiler)  
**Date:** June 22, 2026  
**Status:** Comprehensive Synthesis & Research Paper (PhD Level)  
**Target Output:** `/Users/sac/praxis/research/post_chatman_research.md`

---

## Abstract

This paper synthesizes the findings of the Wave 1 Explorer subagents regarding the structural, theoretical, and practical aspects of generative software architectures. We analyze the transition from the classical unidirectional Chatman Equation ($A = \mu(O)$) to the three-pole isomorphic coherence model ($A \cong O \cong L$). 

In Section 1, we conduct a forensic audit of the `ggen-marketplace` codebase, highlighting critical security and reliability vulnerabilities in cache verification, receipt serialization, symlink extraction, RDF mapping, SPARQL validation, and failure mitigation. 

In Section 2, we present the mathematical and architectural formulation of the post-Chatman Equation ($A \cong O \cong L$), modeling the runtime process log ($L$) as a first-class ontological pole, expanding the Language Server Protocol (LSP) diagnostic surface to USD, MaterialX, and POWL assets, and introducing residual-vector repair loops for autonomous, bounded self-healing. 

Section 3 addresses the mathematical convergence criteria (idempotency, fixed points) of the forward ($\mu$) and inverse ($\mu^{-1}$) pipeline loops, describing mechanisms to prevent infinite feedback oscillations (metadata sidecars, lexicographical sorting, canonical formatting) and resolving concurrent mutations through an algebraic three-way triple merge. 

Finally, Section 4 details the transition design and execution roadmap for the `praxis` boilerplate project generator, replacing static template files with dynamic, ontology-driven Tera templates linked to a formalized Turtle ontology (`praxis.ttl`) and verified by cryptographic receipts. This synthesis establishes a rigorous foundation for autonomous, closed-loop AGI software engineering.

---

## Section 1: Detailed Audit of ggen-marketplace Core Rust and Validation/Catalog Subsystems

A forensic examination of the `ggen-marketplace` codebase in `/Users/sac/ggen` reveals several structural defects, ranging from cryptographic verification mismatches and security vulnerabilities to dead-code placeholders and indexer mismatches. This section details the root causes, mechanics, and implications of these defects.

### 1.1 Cache Verification Logic Bug
In `crates/ggen-marketplace/src/marketplace/install.rs` (lines 452–468), the installer registers a downloaded package inside the cache using a SHA-256 hash computed over the *raw, compressed archive data*:
```rust
// Verify SHA-256 digest
self.verify_pack_digest(&pack_data, &release.checksum)?;

// Extract pack to cache directory
let cache_path = self.extract_pack(&pack_data, package_id, version)?;

// Calculate final digest
let digest = ChecksumCalculator::calculate(&pack_data);

// Create cached pack entry
let cached_pack = CachedPack::new(
    package_id.clone(),
    version.clone(),
    digest,
    pack_data.len() as u64,
    cache_path,
);

// Insert into cache
self.cache.insert(cached_pack.clone())?;
```
Here, `pack_data` is the raw compressed archive (`Vec<u8>`). However, in `crates/ggen-marketplace/src/marketplace/cache.rs` (lines 458–494), the cache verification routine `verify_digest` implements a different hashing behavior:
```rust
pub fn verify_digest(&self, pack: &CachedPack) -> Result<bool> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    let mut verified = true;

    // Walk the pack directory and hash all files
    if pack.cache_path.exists() {
        for entry in walkdir::WalkDir::new(&pack.cache_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(contents) = fs::read(entry.path()) {
                    hasher.update(&contents);
                } else {
                    verified = false;
                    break;
                }
            }
        }
    }

    let calculated_digest = hex::encode(hasher.finalize());
    let matches = calculated_digest == pack.digest;

    if !matches {
        warn!(
            "Digest mismatch for {}: expected {}, got {}",
            pack.cache_key(),
            pack.digest,
            calculated_digest
        );
    }

    Ok(verified && matches)
}
```
#### Core Issues:
1. **Archive-to-Directory Mismatch**: The digest stored in `CachedPack` represents the SHA-256 hash of the compressed zip or tarball file. Conversely, `verify_digest` walks the extracted cache directory and hashes the raw, uncompressed bytes of individual files sequentially. Because compressed archive bytes are mathematically distinct from uncompressed concatenated file bytes, `calculated_digest == pack.digest` evaluates to `false` in every execution.
2. **Forced Cache Eviction and Redundant Downloads**: During subsequent installation requests, the installer checks for a cache hit in `install.rs` (lines 392–408):
   ```rust
   if let Some(cached) = self.cache.get(package_id, version) {
       if self.cache.verify_digest(&cached)? {
           return Ok(cached);
       }
       warn!("Cached pack digest verification failed, re-downloading");
       self.cache.remove(package_id, version)?;
   }
   ```
   Because `verify_digest` always returns `false`, the cache hit is invalidated, the existing files are physically purged from the disk via `self.cache.remove`, and the package is re-downloaded over the network, rendering the caching subsystem entirely non-functional.
3. **Non-deterministic File Walking Order**: `walkdir::WalkDir` does not guarantee a deterministic sorting order of files. The traversal sequence is dependent on the file-system layout and OS-level directory entries. Consequently, even if `verify_digest` were updated to check uncompressed contents, the lack of sorting would lead to transient failures when running on different platforms or filesystems, producing varying hashes for identical sets of files.

### 1.2 Non-Deterministic Receipt IDs
In `crates/ggen-marketplace/src/marketplace/composition_receipt.rs` (lines 487–498), receipt IDs are generated by serializing the `CompositionReceipt` structure to JSON and computing a SHA-256 digest:
```rust
pub fn compute_receipt_id(&mut self) -> Result<()> {
    let _ = self.receipt_id.take(); // Temporarily clear for hashing
    let json = self.to_json()?;
    let hash = sha2_digest(&json);
    self.receipt_id = Some(hash.clone());
    Ok(())
}
```
Where `to_json()` is defined as:
```rust
pub fn to_json(&self) -> Result<String> {
    serde_json::to_string_pretty(self).map_err(Into::into)
}
```
In initial iterations, the `versions` and `ownership_map` fields of the receipt were declared as standard `HashMap<String, String>` and `HashMap<String, OwnershipRecord>`. Rust's default `HashMap` uses SipHash 1-3 with random keys initialized per-process to prevent Hash DoS attacks. When serializing these maps using `serde_json`, the iteration order of keys is non-deterministic, resulting in different JSON strings across execution runs. This broke the cryptographic chain of custody by generating varying receipt IDs for semantically identical receipts. This has been remediated in the current codebase by replacing `HashMap` with `BTreeMap` fields, ensuring lexicographical sorting during JSON serialization.

### 1.3 Path Traversal (Zip Slip) via Symbolic Links
In `crates/ggen-marketplace/src/marketplace/install.rs` (lines 817–859), tarball extraction logic attempts to prevent Zip Slip path traversal by checking path components:
```rust
fn extract_tar_gz(&self, data: &[u8], dest: &Path) -> Result<()> {
    use std::path::Component;
    use tar::Archive;

    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().map_err(|e| Error::InstallationFailed {
        reason: format!("Failed to read tar.gz entries: {}", e),
    })? {
        let mut entry = entry.map_err(|e| Error::InstallationFailed {
            reason: format!("Failed to read tar.gz entry: {}", e),
        })?;

        let entry_path = entry.path().map_err(|e| Error::InstallationFailed {
            reason: format!("Invalid path in tar.gz entry: {}", e),
        })?;

        let mut target = dest.to_path_buf();
        for component in entry_path.components() {
            match component {
                Component::Normal(c) => target.push(c),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(Error::InstallationFailed {
                        reason: format!("Path traversal detected: {}", entry_path.display()),
                    });
                }
            }
        }

        entry.unpack(&target).map_err(|e| Error::InstallationFailed {
            reason: format!("Failed to extract entry: {}", e),
        })?;
    }
    Ok(())
}
```
#### Vulnerability Analysis:
While this function prevents logical traversal within the path declaration (by blocking `..` or `/`), it fails to account for file-system symbolic links. An attacker can craft a malicious archive containing:
1. A symbolic link entry (e.g., named `link_to_root`) pointing to `/` or another directory outside `dest` (e.g., `../../etc`).
2. A subsequent file entry (e.g., named `link_to_root/etc/cron.d/malicious`).

When `extract_tar_gz` runs:
- The first entry `link_to_root` resolves to components `[Normal("link_to_root")]`, passing validation. The symlink is unpacked at `<dest>/link_to_root` pointing to `/`.
- The second entry components are `[Normal("link_to_root"), Normal("etc"), Normal("cron.d"), Normal("malicious")]`. This contains no parent directory or root components, thus passing validation. `target` is constructed as `<dest>/link_to_root/etc/cron.d/malicious`.
- When `entry.unpack(&target)` is called, the OS resolves the path through the symlink, writing the malicious payload directly to `/etc/cron.d/malicious`. This allows arbitrary file writes on the host filesystem.

### 1.4 Registry Class Serialization and Defaults in `rdf_mapper.rs`
In `crates/ggen-marketplace/src/marketplace/rdf_mapper.rs` (lines 228–238, 440–449, 622–633), we observe the following RDF mapping logic:
```rust
// JSON-encoded serialization of RegistryClass in RDF literal triple
let registry_class_json = serde_json::to_string(&release.registry_class).map_err(|e| {
    Error::RegistryError(format!("Failed to serialize registry_class: {}", e))
})?;
self.insert_literal_triple(
    &version_uri,
    &format!("{}registryClass", Namespaces::GGEN),
    &registry_class_json,
)?;
```
During reconstruction, the data is deserialized:
```rust
let registry_class = Self::extract_optional_literal(&solution, "registryClass")
    .and_then(|s| serde_json::from_str::<crate::marketplace::trust::RegistryClass>(&s).ok())
    .unwrap_or_else(crate::marketplace::models::default_registry_class);
```
#### Defect Breakdown:
1. **Opaque RDF Literal Anti-pattern**: Encoding complex structured data (`RegistryClass`) as a JSON string inside a literal triple violates the core principles of semantic graph modeling. This data is opaque to the triplestore, preventing native SPARQL filtering, audit rules, or graph-based access control.
2. **Enterprise Registry Bypass**: If the `registryClass` literal is absent or fails to deserialize, the mapper falls back to `default_registry_class()`, which returns `RegistryClass::Public` with a blank URL and type `Ggen`. In configurations where a security profile blocks public registries (`profile.forbid_public_registry() == true`), reconstructed private enterprise (`PrivateEnterprise`) or air-gapped (`MirroredAirGapped`) releases will be incorrectly classified as `Public` and blocked from installation.
3. **Missing RegistryType Variant**: In `query_package_metadata`, the registry type string is mapped back into an enum:
   ```rust
   let registry_type = Self::extract_optional_literal(&solution, "regType")
       .map(|s| match s.as_str() {
           "crates.io" => crate::marketplace::trust::RegistryType::CratesIo,
           "npm" => crate::marketplace::trust::RegistryType::Npm,
           "pypi" => crate::marketplace::trust::RegistryType::PyPi,
           "github" => crate::marketplace::trust::RegistryType::GitHub,
           _ => crate::marketplace::trust::RegistryType::Ggen,
       });
   ```
   This matching logic completely omits the `RegistryType::Other` variant. Any package registered with a registry type of `"other"` defaults to `RegistryType::Ggen` on load.

### 1.5 SPARQL Injection Check Brittleness in `rdf_control.rs`
In `crates/ggen-marketplace/src/marketplace/rdf/rdf_control.rs` (lines 100–107, 390–411), injection detection is performed via exact substring matching:
```rust
fn detect_injection(&self, query: &str) -> bool {
    let query_upper = query.to_uppercase();
    let suspicious_patterns = [
        "DROP",
        "DELETE WHERE {",
        "INSERT DATA {",
        "CLEAR GRAPH",
        "; DELETE",
    ];

    for pattern in &suspicious_patterns {
        if query_upper.contains(pattern) {
            warn!("Suspicious pattern detected in query: {pattern}");
            return true;
        }
    }
    false
}
```
#### Defect Breakdown:
1. **Trivial Bypass via Whitespace and Newlines**: Because SPARQL is whitespace-insensitive, an attacker can construct queries that execute modifications while bypassing the hardcoded filters, such as:
   - `DELETE WHERE{ ?s ?p ?o }` (No space before the opening brace)
   - `DELETE   WHERE   { ?s ?p ?o }` (Multiple spaces)
   - `DELETE\nWHERE\n{ ?s ?p ?o }` (Newline characters)
2. **False Positives on Safe Identifiers**: Checking `query_upper.contains("DROP")` scans the entire query string, blocking valid queries that contain variables or namespaces with "drop" (e.g., `?dropdown_menu`, `?dropped_packets`), leading to service denials on benign operations.
3. **Lack of Abstract Syntax Tree (AST) Validation**: Relying on string heuristic validation instead of parsing queries into an AST violates standard secure coding guidelines.

### 1.6 Dead Code and Stubs in Policy and FMEA Mitigations
#### Policy Stubs (`policy.rs`)
In `crates/ggen-marketplace/src/marketplace/policy.rs` (lines 513–518), custom policy rules evaluate to immediate failures:
```rust
PolicyRule::CustomSparql { .. } | PolicyRule::CustomShell { .. } => {
    return Err(Error::ValidationFailed {
        reason: "Custom policy rules require execution context".to_string(),
    });
}
```
No execution context or interpreter engine exists for custom rules, making these enum variants non-functional stubs.

#### FMEA Mitigation Stubs (`fmea_mitigations.rs`)
The mitigation manager claims to address all 47 failure modes identified in the marketplace FMEA analysis, but only 15 (`FM-001` through `FM-015`) are defined. The remaining 32 failure modes are missing. Additionally, the implemented mitigations are stubs:
- `mitigate_circular_dependency` (FM-003): Returns `MitigationResult::ManualInterventionRequired` immediately.
- `mitigate_memory_exhaustion` (FM-011): Merely logs a string claiming to clear caches and trigger GC, but executes no actual cleanup logic.
- `has_no_references`: Returns hardcoded `false` unless the resource ID string begins with `_:`.
- `optimize_query`: Appends `LIMIT 100` to a query string but the caller in `mitigate_query_timeout` discards the modified string, leaving the timeout unmitigated.
- `mitigate_config_parse_error` (FM-012): Reads a backup configuration file but discards the content, failing to apply the configuration.

### 1.7 Python Registry Indexer Mismatch
In `marketplace/scripts/validate_marketplace.py` (lines 254–303), the validation script determines if a package is production-ready and writes that state into `package.toml` in the root of the package:
```python
content = content.rstrip() + f"\n\n[marketplace]\nproduction_ready = {new_val}\n"
```
However, in `marketplace/scripts/generate_registry_index.py` (line 117), the registry index generator reads this value from a different nested section:
```python
package_metadata = package.get("metadata", {})
production_ready = package_metadata.get("production_ready", False)
```
#### Core Issues:
1. **Section Key Incompatibility**: The validator writes to `data["marketplace"]["production_ready"]`, while the reader expects `data["package"]["metadata"]["production_ready"]`.
2. **Security Check Bypass**: A package (such as `agent-cli-copilot`) might fail validation and be marked `production_ready = false` under `[marketplace]`. However, if `[package.metadata]` has a hardcoded `production_ready = true`, the indexer reads the stale metadata value and indexes the package as production-ready, bypassing safety controls.

---

## Section 2: Mathematical and Architectural Formulation of the Post-Chatman Equation

The classical Chatman Equation defined the development process as a unidirectional compilation function $\mu$ mapping an ontology $O$ onto an artifact $A$:
$$A = \mu(O)$$
This model assumes that the ontology is the static source of truth and the codebase is an inert compilation target. In practice, this leads to an asymmetric boundary. The post-Chatman paradigm replaces this with a three-pole isomorphic relation:
$$A \cong O \cong L$$
where the software artifact ($A$), ontology ($O$), and runtime process log ($L$) represent isomorphic projections of a single canonical formal system.

```
       [ O: RDF Ontology ]
          /           \
         /             \
        /   Coherence   \
       /     Checker     \
      /                   \
[ A: Artifact ] <=======> [ L: Event Log (OCEL) ]
```

### 2.1 The Three-Pole Model and Event Logs as First-Class Ontological Evidence
We define:
*   **Ontology ($O \in \mathcal{O}$)**: An RDF triple graph representing structural invariants, types, and relations.
*   **Artifact ($A \in \mathcal{A}$)**: The physical files on disk (source code, geometry, build rules).
*   **Event Log ($L \in \mathcal{L}$)**: An Object-Centric Event Log (OCEL) capturing the runtime execution states and compilation steps.

Instead of treating execution logs as transient text streams, the post-Chatman model represents $L$ as a graph mapping system lifecycle transitions. In `crates/ggen-graph/src/ocel/conformance.rs`, the system validates process conformance using SPARQL ASK queries over the projected OCEL graph, evaluating temporal invariants:
```sparql
PREFIX ocel: <http://www.ocel-standard.org/ns#>
ASK {
    ?e0 ocel:activity "DiagnosticRaised" ; ocel:timestamp ?t0 ; <qualifier> ?case .
    ?e1 ocel:activity "RepairApplied" ; ocel:timestamp ?t1 ; <qualifier> ?case .
    ?e2 ocel:activity "GatePassed" ; ocel:timestamp ?t2 ; <qualifier> ?case .
    FILTER(?t0 < ?t1 && ?t1 < ?t2)
}
```
This query asserts a temporal ordering invariant: a system cannot transition to `GatePassed` without first raising a diagnostic and applying a repair. By encoding these paths in the event log graph and running graph conformance checkers, we treat the event log as co-equal evidence of system correctness.

### 2.2 Coherence Checker Mechanics
The three-pole alignment is monitored by `crates/ggen-graph/src/coherence.rs` via the `CoherenceChecker`. It computes cryptographic digests for each pole:
1.  **$\text{Hash}_O$**: The BLAKE3 hash of the canonicalized RDF triple serialization.
2.  **$\text{Hash}_A$**: The BLAKE3 hash of the concatenated, lexicographically sorted artifact source files.
3.  **$\text{Hash}_L$**: The BLAKE3 hash of the chronological OCEL event trail.

The checker generates a `CoherenceReport` and logs discrepancies as `CoherenceDrift`:
```rust
pub struct CoherenceDrift {
    pub kind: DriftKind,
    pub source_pole: Pole,
    pub target_pole: Pole,
    pub delta: String,
}
```
If a developer manually modifies code without updating the ontology, $\text{Hash}_A$ drifts, causing receipt verification to fail and blocking compilation or admission gates.

### 2.3 Residual-Vector Repair Loops
Rather than relying on unguided generative loops to fix drift or compilation errors, the post-Chatman system implements a deterministic feedback control loop based on **residual-vector minimization**, utilizing the structures in `crates/genesis-types-v2/src/lib.rs` (lines 690–916):

1.  **Residual Definition**:
    We represent the system state as a vector in a multidimensional metric space. The drift between the current state and target constraints is captured in a `VisualGapReport` containing a `ResidualVector`:
    ```rust
    pub struct ResidualVector {
        pub dimensions: Vec<ResidualDimension>,
        pub norm: f64,
    }

    pub struct ResidualDimension {
        pub name: String,
        pub measured: f64,
        pub target: (f64, f64),
        pub residual: f64,
    }
    ```
2.  **Bounded Repair Operators**:
    A `BoundedRepairOperator` maps specific types of residuals to code modification functions, constrained by a target `RepairBand` and backed by an `EvidenceTier`:
    ```rust
    pub struct BoundedRepairOperator {
        pub target_dimension: String,
        pub band: RepairBand,
        pub tier: EvidenceTier, // Known | Inferred | Estimated
    }
    ```
3.  **Repair Admission**:
    Executing a repair operator yields a `RepairAdmissionReport`. The state transition is admitted if and only if the absolute residual norm decreases without violating the safety bands:
    $$\|R_{\text{after}}\| < \|R_{\text{before}}\| \quad \lor \quad \text{AllPassing}$$

### 2.4 LSP Diagnostic Expansion
In a post-Chatman world, non-code assets like USD geometry files, MaterialX shader networks, and POWL process laws are primary artifacts ($A$) of the ontology. The Language Server Protocol (LSP) in `crates/ggen-lsp/src/state.rs` must extend its diagnostic authority over these files.

#### FileType Expansion:
```rust
pub enum FileType {
    Rdf,
    Sparql,
    Tera,
    Toml,
    Usd,  // *.usda / *.usd
    Mtlx, // *.mtlx
    Powl, // *.powl
    Unknown,
}
```

#### Specialized Diagnostics:
We define four diagnostic checks:
*   **`GGEN-USD-001` (Foreign geometry in part file)**: Mesh identifiers within a modular torso USD part file must match the ontologically declared part ID, blocking head or arm geometries from being declared in the torso layout.
*   **`GGEN-USD-002` (Missing `owner_part_id` attribute)**: Mesh prims in USD must carry an explicit custom attribute `string owner_part_id` mapping them to their owner in the RDF graph.
*   **`GGEN-USD-003` (Socket boundary violation)**: A socket prim (an attachment coordinate) must not contain a mesh payload, enforcing a separation between physical geometry and attachment interfaces.
*   **`GGEN-MTLX-001` (Unbound material input)**: Every input parameter of a shader node in a MaterialX document must bind to an upstream node output or define a static fallback, preventing compilation failures at render time.

#### POWL Process Graphs and wasm4pm Ingestion:
Process law graphs (POWL) define workflow transitions (e.g., `CompilationLoop.powl`). These graphs are serialized into `PowlGraph` data models inside `crates/genesis-types-v2/src/lib.rs` (`PowlNode`, `PowlEdge`). The Wasm-based Process Manager (`wasm4pm`) consumes `PowlGraph` schemas and current `OcelLog` execution trails to generate a `ProcessAdmissionReport`:
```rust
pub struct ProcessAdmissionReport {
    pub operation_id: String,
    pub graph_id: String,
    pub status: AdmissionStatus, // Alive | PartialAlive | Refused | Unknown
    pub gates: Vec<GateResult>,
    pub receipt_hash: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```
This ensures runtime process execution conforms to the ontologically declared state machine.

---

## Section 3: Convergence of the Forward ($\mu$) and Inverse ($\mu^{-1}$) Pipeline Loops

When code artifacts ($A$) and ontologies ($O$) are both active, editing either side requires synchronizing changes to the other. This creates a bidirectional loop:
*   **Forward Projection ($\mu: \mathcal{O} \to \mathcal{A}$)**: Compiles the RDF ontology into physical code files.
*   **Inverse Extraction ($\mu^{-1}: \mathcal{A} \to \mathcal{O}$)**: Extracts semantic structures from code ASTs back into RDF.

```
          Forward Projection (μ)
      ┌─────────────────────────────┐
      │                             │
      ▼                             │
┌───────────┐                 ┌───────────┐
│ Ontology  │                 │ Artifacts │
│    (O)    │                 │    (A)    │
└───────────┘                 └───────────┘
      │                             ▲
      │                             │
      └─────────────────────────────┘
          Inverse Extraction (μ⁻¹)
```

### 3.1 Mathematical Convergence Criteria
We define the round-trip mappings as:
*   Ontology-initiated: $f(O) = \mu^{-1}(\mu(O))$
*   Artifact-initiated: $g(A) = \mu(\mu^{-1}(A))$

For the system to reach stability and prevent infinite execution runs, $f$ and $g$ must be **idempotent**, meaning they reach a fixed point in a single step:
$$f(f(O)) = f(O) \quad \text{and} \quad g(g(A)) = g(A)$$
If $f$ or $g$ is not idempotent, successive sync runs will continually modify the codebase or the ontology, leading to non-convergent oscillations.

### 3.2 Mathematical Causes of Divergence
1.  **Information Asymmetry (Lossy Projection)**: The mapping $\mu$ projects a rich semantic graph into raw code syntax. Abstract ontology properties (e.g., namespace prefixes like `@prefix`, comments, disjointness axioms) are omitted in the generated source code. Consequently, the extracted ontology $O' = \mu^{-1}(A)$ is missing these attributes ($O' \subset O$). Running $\mu(O')$ then produces code that lacks these details, leading to code drift ($A_1 \neq A_0$).
2.  **Syntactic Asymmetry (Ordering)**: RDF graphs are unordered sets of triples. Conversely, code files are ordered linear text. If the query projecting data into the code template lacks sorting, successive generations will write struct fields, methods, or modules in random order. A subsequent AST extraction on this reordered file will produce a different hash, preventing convergence.
3.  **Parser Ambiguity**: Variations in indentation, bracing, and line breaks introduced by formatting tools (e.g., `rustfmt` or `gofmt`) during the canonicalization stage ($\mu_4$) can cause AST extraction parsers ($\mu^{-1}_2$) to output slightly different triples, preventing hash convergence.

### 3.3 Prevention of Infinite Feedback Loops (Oscillations)
To enforce idempotency, the synchronization engine implements three mechanisms:

#### 1. Metadata Preservation via Side-Channels
To prevent the loss of non-structural metadata ($\mu^{-1}(\mu(O)) \neq O$), the system preserves these properties across the code boundary:
*   **Embedded Code Annotations**: Attributes are embedded into the generated code to carry semantic metadata.
    - *Rust*:
      ```rust
      #[ggen(uri = "code:UserService", comment = "Core authorization", prefixes = { code = "https://ggen.io/code#" })]
      pub struct UserService {
          pub name: String,
      }
      ```
    - *Go*:
      ```go
      type UserService struct {
          Name string `ggen:"uri=code:name;range=xsd:string"`
      }
      ```
*   **Metadata Sidecars (`*.meta.json`)**: For assets that do not support annotations (such as USD or binary formats), the generator writes a companion sidecar file containing the source triples. During extraction, $\mu^{-1}$ merges the parsed AST with the sidecar metadata to reconstruct the ontology.

#### 2. Reconciliatory Delta Extraction
Instead of allowing the extracted ontology $\mu^{-1}(A)$ to overwrite the target ontology graph, the system uses a **Local Closed-World Assumption**. We define an extraction scope $\text{Scope}(A)$ representing the set of resources the code is authorized to define. The merge is calculated as:
$$O_{n+1} = \Big( O_n \setminus \{ (s, p, o) \in O_n \mid s \in \text{Scope}(A) \} \Big) \cup \mu^{-1}(A)$$
This preserves global prefixes, rules, and external imports defined outside the artifact's scope.

#### 3. Strict Lexicographical Normalization
All projection and emission steps enforce strict sorting:
*   **SPARQL Normalization**: Every SELECT query used during projection must include an explicit `ORDER BY` clause.
*   **Turtle Normalization**: Serialized RDF outputs must sort triples lexicographically (Subject -> Predicate -> Object).
*   **Format-then-Hash Guard**: The BLAKE3 hash of an artifact is computed *only* after standard formatting. If the AST of $A_{n+1}$ matches $A_n$, formatting ensures the files are byte-identical, achieving 100% hash identity and terminating the loop.

### 3.4 Concurrent Update Resolution
When both the ontology and the code are modified concurrently ($O \to O_{\text{mut}}$ and $A \to A_{\text{mut}}$), the system performs a three-way merge in the RDF triple space:

```
                  Base State (O_base, A_base)
                  /                         \
                 /                           \
                ▼                             ▼
       Ontology Edit (O_mut)          Artifact Edit (A_mut)
                \                             /
                 \                           /
                  ▼                         ▼
                    Merged State (O_merged)
                              │
                              ▼
                    Regenerated (A_merged)
```

1.  **Identify Common Ancestor**: Retrieve the last synchronized state $(O_{\text{base}}, A_{\text{base}})$ via the receipt history.
2.  **Project Code to Triple Space**: Run AST extraction on the base and mutated code files:
    $$O_{\text{base\_extracted}} = \mu^{-1}(A_{\text{base}})$$
    $$O_{\text{mut\_extracted}} = \mu^{-1}(A_{\text{mut}})$$
3.  **Compute Deltas**:
    *   Ontology branch:
        $$\Delta^+_O = O_{\text{mut}} \setminus O_{\text{base}} \quad \text{and} \quad \Delta^-_O = O_{\text{base}} \setminus O_{\text{mut}}$$
    *   Artifact branch:
        $$\Delta^+_A = O_{\text{mut\_extracted}} \setminus O_{\text{base\_extracted}} \quad \text{and} \quad \Delta^-_A = O_{\text{base\_extracted}} \setminus O_{\text{mut\_extracted}}$$
4.  **Algebraic Merge Operator ($\oplus$)**:
    $$O_{\text{merged\_raw}} = O_{\text{base}} \cup \Delta^+_O \cup \Delta^+_A$$
    $$O_{\text{merged}} = O_{\text{merged\_raw}} \setminus (\Delta^-_O \cup \Delta^-_A)$$

#### Conflict Detection and Resolution Policies:
A conflict is detected if:
- **Direct Conflict**: $\Delta^+_O$ and $\Delta^+_A$ assign different values to the same property (e.g., the ontology sets a port range to `xsd:integer` while the code changes the struct field to `f64`).
- **Indirect Conflict**: The merged graph $O_{\text{merged}}$ violates SHACL shapes or OWL axioms checked by the Doctrine Engine.

Three resolution policies are defined:
*   **Ontology Primacy** (Default): Discards the conflicting code changes ($\Delta_A$) and regenerates the code to match the ontology.
*   **Artifact Primacy**: Overwrites conflicting ontology assertions ($\Delta_O$) to match the code edits.
*   **Agentic Arbitration**: Delegated to a control loop that analyzes the semantic conflict and applies a SPARQL update patch.

---

## Section 4: Praxis Transition Design and Execution Roadmap

To move `/Users/sac/praxis` from a template-copying model using text substitutions to a **ggen-first** generative architecture, we define the schema, generator rules, template designs, and execution phases.

```
+--------------------+      ggen sync      +--------------------------+
|  schema/praxis.ttl |  ---------------->  |  Cargo.toml              |
+--------------------+                     |  rustfmt.toml            |
          |                                |  src/types.rs            |
          v                                |  src/lsp.rs              |
+--------------------+                     |  src/cli.rs              |
|     ggen.toml      |                     +--------------------------+
+--------------------+                                   |
                                                         v
                                              +---------------------+
                                              |  cargo test         |
                                              |  verify_conformance |
                                              +---------------------+
```

### 4.1 Praxis Ontology Schema (`schema/praxis.ttl`)
This ontology defines the structure of a standardized project, including workspace configuration, dependencies, ZST typestate lifecycles, and CLI command mappings:
```turtle
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix praxis:  <http://seanchatmangpt.github.io/praxis/schema#> .

praxis:Project a rdfs:Class ;
    rdfs:label "Project" ;
    rdfs:comment "A repository configured under the Praxis house style." .

praxis:RustCrate a rdfs:Class ;
    rdfs:label "RustCrate" .

praxis:Component a rdfs:Class ;
    rdfs:label "Component" .

praxis:ZstTypestate a rdfs:Class ;
    rdfs:label "ZstTypestate" ;
    rdfs:comment "A ZST compile-time lifecycle marker." .

praxis:CliCommand a rdfs:Class ;
    rdfs:label "CliCommand" .

praxis:name a rdf:Property ;
    rdfs:domain [ owl:unionOf (praxis:Project praxis:RustCrate praxis:Component praxis:ZstTypestate) ] ;
    rdfs:range xsd:string .

praxis:version a rdf:Property ;
    rdfs:domain [ owl:unionOf (praxis:Project praxis:RustCrate) ] ;
    rdfs:range xsd:string .

praxis:isWorkspace a rdf:Property ;
    rdfs:domain praxis:Project ;
    rdfs:range xsd:boolean .

praxis:hasCrate a rdf:Property ;
    rdfs:domain praxis:Project ;
    rdfs:range praxis:RustCrate .

praxis:hasComponent a rdf:Property ;
    rdfs:domain praxis:RustCrate ;
    rdfs:range praxis:Component .

praxis:hasTypestate a rdf:Property ;
    rdfs:domain praxis:Component ;
    rdfs:range praxis:ZstTypestate .

praxis:noun a rdf:Property ;
    rdfs:domain praxis:CliCommand ;
    rdfs:range xsd:string .

praxis:verb a rdf:Property ;
    rdfs:domain praxis:CliCommand ;
    rdfs:range xsd:string .

praxis:handler a rdf:Property ;
    rdfs:domain praxis:CliCommand ;
    rdfs:range xsd:string .

# Instance Definition
praxis:MyConformingProject a praxis:Project ;
    praxis:name "my-conforming-project" ;
    praxis:version "26.6.0" ;
    praxis:isWorkspace true ;
    praxis:hasCrate praxis:CrateCore .

praxis:CrateCore a praxis:RustCrate ;
    praxis:name "my-conforming-project" ;
    praxis:hasComponent praxis:CompTypestates, praxis:CompCli .

praxis:CompTypestates a praxis:Component ;
    praxis:name "GenerativeTypestates" ;
    praxis:hasTypestate praxis:StateRaw, praxis:StateValidated, praxis:StateAdmitted .

praxis:StateRaw a praxis:ZstTypestate ;
    praxis:name "Raw" .

praxis:StateValidated a praxis:ZstTypestate ;
    praxis:name "Validated" .

praxis:StateAdmitted a praxis:ZstTypestate ;
    praxis:name "Admitted" .

praxis:CmdDodRun a praxis:CliCommand ;
    praxis:noun "dod" ;
    praxis:verb "run" ;
    praxis:handler "handle_dod_run" .
```

### 4.2 Generator Configuration (`ggen.toml`)
This configuration file drives the `ggen` pipeline execution, containing inference, validation, and generation rules:
```toml
[project]
name = "praxis-generator"
version = "26.6.0"
description = "Ontology-driven code generator for Praxis projects"

[ontology]
source = "schema/praxis.ttl"
standard_only = false

[ontology.prefixes]
praxis = "http://seanchatmangpt.github.io/praxis/schema#"
rdf    = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
rdfs   = "http://www.w3.org/2000/01/rdf-schema#"
xsd    = "http://www.w3.org/2001/XMLSchema#"

[inference]
rules = [
    { name = "derive-workspace-members", construct = """
      PREFIX praxis: <http://seanchatmangpt.github.io/praxis/schema#>
      CONSTRUCT {
        ?c praxis:isWorkspaceMember true .
      } WHERE {
        ?p a praxis:Project ;
           praxis:isWorkspace true ;
           praxis:hasCrate ?c .
      }
    """ }
]

[[validation.rules]]
name        = "calver-format-validation"
description = "All project and crate versions must follow CalVer YY.M.patch format."
severity    = "Error"
ask         = """
  PREFIX praxis: <http://seanchatmangpt.github.io/praxis/schema#>
  ASK {
    FILTER NOT EXISTS {
      ?x praxis:version ?v .
      FILTER(!regex(?v, "^[0-9]{2}\\\\.[0-9]{1,2}\\\\.[0-9]+$"))
    }
  }
"""

[generation]
output_dir = "."

[[generation.rules]]
name        = "cargo-toml"
query       = { inline = """
  PREFIX praxis: <http://seanchatmangpt.github.io/praxis/schema#>
  SELECT ?name ?version WHERE {
    ?p a praxis:Project ; praxis:name ?name ; praxis:version ?version .
  }
""" }
template    = { file = "templates/Cargo.toml.tera" }
output_file = "Cargo.toml"
mode        = "Overwrite"

[[generation.rules]]
name        = "src-types"
query       = { inline = """
  PREFIX praxis: <http://seanchatmangpt.github.io/praxis/schema#>
  SELECT ?name WHERE {
    ?p a praxis:Project ; praxis:name ?name .
  }
""" }
template    = { file = "templates/src/types.rs.tera" }
output_file = "src/types.rs"
mode        = "Overwrite"
```

### 4.3 Template Designs
The templates generate standard configurations and typestate files, ensuring structural invariants are enforced at compile time.

#### 1. `templates/Cargo.toml.tera`
```toml
[package]
name = "{{ name }}"
version = "{{ version }}"
edition = "2021"
rust-version = "1.82"
license = "MIT OR Apache-2.0"
description = "Standardized generated project"

[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unreachable_pub = "warn"

[lints.clippy]
all = { level = "warn", priority = -1 }
todo = "deny"
unimplemented = "deny"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
blake3 = "1"
clap = { version = "4", features = ["derive"] }
clap-noun-verb = { path = "/Users/sac/clap-noun-verb" }
thiserror = "2"
```

#### 2. `templates/src/types.rs.tera`
This file implements content addressing, ZST state markers, and the compile-time ZST assertion checks:
```rust
//! Generative Typestates & Integrity Primitives for `{{ name }}`.
//! Generated by ggen-first pipeline. Do not edit manually.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

// --- Content Addressing ---
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Blake3Hash(pub String);

impl Blake3Hash {
    pub fn content_address(bytes: &[u8]) -> Self {
        Blake3Hash(blake3::hash(bytes).to_hex().to_string())
    }

    pub fn from_hex(hex: impl Into<String>) -> Self {
        Blake3Hash(hex.into())
    }

    pub fn as_hex(&self) -> &str {
        &self.0
    }
}

// --- Object References (OCEL Conformant) ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRef {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifier: Option<String>,
}

impl fmt::Display for ObjectRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(q) => write!(f, "{}:{}:{}", self.id, self.type_, q),
            None => write!(f, "{}:{}", self.id, self.type_),
        }
    }
}

// --- Canonical Serialization (Key-sorted JSON) ---
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    let sorted = sort_value(v);
    serde_json::to_vec(&sorted)
}

fn sort_value(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let sorted: std::collections::BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, sort_value(v))).collect();
            let mut out = serde_json::Map::new();
            for (k, v) in sorted {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}

// --- ZST Lifecycle State Markers ---
mod sealed {
    pub trait LifecycleState {}
}

pub struct Raw;
impl sealed::LifecycleState for Raw {}

pub struct Validated;
impl sealed::LifecycleState for Validated {}

pub struct Admitted;
impl sealed::LifecycleState for Admitted {}

// --- Zero-Overhead Evidence Carriage ---
pub struct Evidence<T, State: sealed::LifecycleState, Witness> {
    inner: T,
    _state: PhantomData<State>,
    _witness: PhantomData<Witness>,
}

impl<T, Witness> Evidence<T, Raw, Witness> {
    pub fn new(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T, Witness> Evidence<T, Validated, Witness> {
    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub(crate) fn validate_unchecked(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}

impl<T, Witness> Evidence<T, Admitted, Witness> {
    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub(crate) fn admit_unchecked(inner: T) -> Self {
        Self { inner, _state: PhantomData, _witness: PhantomData }
    }
}

// --- One-Way Admission Trait ---
pub trait Admit {
    type Input;
    type Witness;
    type Error;

    fn admit(
        input: Evidence<Self::Input, Raw, Self::Witness>,
    ) -> Result<Evidence<Self::Input, Admitted, Self::Witness>, Self::Error>;
}

// --- Sealed Cryptographic Admission Receipt ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedReceipt {
    pub chain_hash: [u8; 32],
    pub timestamp: u64,
    #[serde(skip)]
    _seal: (), // Private seal field prevents downstream construction
}

impl AdmittedReceipt {
    pub(crate) fn new(chain_hash: [u8; 32], timestamp: u64) -> Self {
        Self {
            chain_hash,
            timestamp,
            _seal: (),
        }
    }
}

// --- Compile-time ZST Layout Asserts ---
#[allow(dead_code)]
mod layout_assertions {
    use super::*;
    use std::mem::size_of;

    const _RAW_IS_ZST: () = { assert!(size_of::<Raw>() == 0, "Raw marker must stay ZST"); };
    const _VALIDATED_IS_ZST: () = { assert!(size_of::<Validated>() == 0, "Validated marker must stay ZST"); };
    const _ADMITTED_IS_ZST: () = { assert!(size_of::<Admitted>() == 0, "Admitted marker must stay ZST"); };
    
    const _EVIDENCE_NO_OVERHEAD: () = {
        assert!(
            size_of::<Evidence<u64, Raw, Raw>>() == size_of::<u64>(),
            "Evidence must be zero-overhead over T when State/Witness are ZSTs"
        );
    };
}
```

### 4.4 Transition and Execution Roadmap
The transition is structured into four sequential phases:

#### Phase 1: Environment Baseline Setup
1. Move the legacy `/Users/sac/praxis/template/` directory to a backup location (`/Users/sac/praxis/backup_template`).
2. Create directories for the schemas and templates:
   - `/Users/sac/praxis/schema/` (to host `praxis.ttl`).
   - `/Users/sac/praxis/templates/` (to host Tera templates).
3. Place `praxis.ttl` and the root `ggen.toml` file in `/Users/sac/praxis/`.

#### Phase 2: Generation Execution (Synchronization)
1. Run `ggen sync` from the project root directory.
2. The generator parses the `schema/praxis.ttl` graph, evaluates CONSTRUCT inference rules (e.g., deriving workspace members), validates assertions via ASK shapes, and renders the Cargo and Rust source files.
3. Verify that the generated code compiles successfully and contains no `todo!` or `unimplemented!` placeholders.

#### Phase 3: Active Healing Integration
1. Configure `praxis-reconciler` to monitor the generated directories. If out-of-band edits are made to generated files (e.g., manual modifications to `src/types.rs`), the reconciler automatically regenerates the files back to the ontology-defined baseline.
2. Add `praxis-guard` check hooks to the git pre-commit flow, block commits if local files drift from the canonical ontology-generated digests.

#### Phase 4: Verification & Sign-off
1. Execute the conformance script to run the hollow-gate checkers:
   ```bash
   bash /Users/sac/praxis/tools/verify_conformance.sh
   ```
2. Run crate tests to confirm the compilation of ZST constraints:
   ```bash
   cargo test --all-features
   ```
3. Issue and sign the cryptographic provenance receipts, validating the transition.

---

## Conclusion
The transition from unidirectional generation to a three-pole isomorphic paradigm ($A \cong O \cong L$) addresses structural drift. By implementing a bidirectional compilation loop ($\mu$ and $\mu^{-1}$), enforcing idempotency via canonical serialization, and bounding modifications using residual-vector minimization, we establish a closed-loop engineering framework that eliminates out-of-band drift. The transition of the Praxis template to this schema-driven model demonstrates the feasibility of self-healing software systems for autonomous AGI engineering.
