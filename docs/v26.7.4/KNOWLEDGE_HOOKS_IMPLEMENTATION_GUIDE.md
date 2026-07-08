# Knowledge Hooks Implementation Guide — Praxis v26.7.4

## Overview

This guide provides implementation notes for the Knowledge Hooks Specification. It maps normative requirements to concrete code locations and test evidence.

## Core API

### Loading Hook Packs

```rust
pub fn load_hook_pack<P: AsRef<std::path::Path>>(&mut self, pack_ref: P) -> Result<(), String>
```

**Behavior:**
- Accepts either an inline Turtle string or a directory path.
- If directory: reads `pack.toml` (metadata) and `ontology.ttl` (hook definitions).
- If string: treats as inline Turtle content.
- Validates all hooks through admission gates.
- On success: adds hooks to `self.hooks` and triples to `self.triple_index`.
- On failure: returns error; no state changes.

**Example (inline):**
```rust
let mut store = TripleStore::new();
let pack = r#"
  @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
  ex:my_hook a kh:Hook ;
      kh:name "my_hook" ;
      kh:kind "delta" ;
      kh:var "<http://example.org/status>" ;
      kh:effect "emit-delta" .
"#;
store.load_hook_pack(pack)?;
```

**Example (directory):**
```rust
store.load_hook_pack("./my_hook_pack/")?;
// Reads ./my_hook_pack/pack.toml and ./my_hook_pack/ontology.ttl
```

### Retrieving Receipts

```rust
pub fn get_hook_receipts(&self) -> Vec<hooks::HookReceipt>
```

**Behavior:**
- Returns all hook receipts generated during the most recent `materialize()` call.
- Cleared on each new `materialize()` call.
- Contains: hook_name, delta_hash (BLAKE3), idempotency_key, delta_quads (N-Quads).

**Example:**
```rust
store.materialize();
let receipts = store.get_hook_receipts();
for receipt in receipts {
    println!("Hook: {}", receipt.hook_name);
    println!("Delta hash: {}", receipt.delta_hash);
    println!("Idempotency key: {}", receipt.idempotency_key);
}
```

### Accessing Verdicts

Verdicts are stored in `store.verdicts` (a public field):

```rust
pub verdicts: Vec<hooks::HookVerdictRecord>
```

**Example:**
```rust
store.materialize();
for verdict in &store.verdicts {
    println!("{}: {:?}", verdict.hook_name, verdict.verdict);
    if let Some(diag) = &verdict.diagnostics {
        for detail in &diag.details {
            println!("  - {}", detail.message);
        }
    }
}
```

## Admission Gates (Implementation Order)

Hooks are validated in this order. First failure stops the pack:

### 1. SHACL Shape Validation

**Code:** `hooks.rs:330`, `validate_shacl(SHACL_LAW_PACK)`

**Schema enforced:**
- All hooks must conform to `HookShape` (target: `kh:Hook`).
- All actions must conform to `ActionShape` (target: `kh:Action`).
- Closed vocabulary: `sh:closed true`.
- Required properties: `kh:name`, `kh:kind`, `kh:effect`.
- Optional properties: all others, maxCount 1 each.
- `kh:after` is multi-valued (no maxCount).

**Failure:** Returns SHACL validation report with focus node, path, and violation details.

### 2. Closed Vocabulary Gating

**Code:** `hooks.rs:393-399`, `ALLOWED_KH_PREDICATES`

**Check:**
- For each predicate in `kh:` namespace, verify local name is in whitelist.
- Allowed: `name, on, kind, var, op, k, window, program, goal, query, effect, action, reason, priority, after, handler, adds_ttl`.

**Failure:** "forbidden predicate in kh: namespace: [pred]"

### 3. Forbidden Keyword Scanning

**Code:** `hooks.rs:260-270`, `contains_forbidden_keyword()`

**Check:**
- Scan all subject, predicate, object, graph IRIs for case-insensitive: `shell, exec, curl, socket, fetch`.
- Allow matches that start with:
  - `http://seanchatmangpt.github.io/praxis/`
  - `http://www.w3.org/`

**Failure:** "forbidden keyword in [subject|predicate|object|graph]: [term]"

### 4. Condition Kind Validation

**Code:** `hooks.rs:435-487`, per-kind validation

| Kind | Checks |
|------|--------|
| `datalog` | Requires `kh:program` and `kh:goal`; program must parse as Datalog; ≤ 8 rules; goal must not be "t" |
| `delta` | Requires `kh:var` |
| `threshold` | Requires `kh:var`, `kh:op` (one of =, !=, <, <=, >, >=), `kh:k` (integer) |
| `count` | Requires `kh:var`, `kh:op`, `kh:k` |
| `window` | Requires `kh:var`, `kh:op`, `kh:k`, `kh:window` (1..255) |
| `shacl` | Requires `kh:program`; must be valid SHACL Turtle |
| `shex` | Requires `kh:program` (ShExJ or ShExC), `kh:goal` (shape map) |
| `n3` | Requires `kh:program`; must be valid N3 rules |
| `sparql` | Requires `kh:query`; must be valid SPARQL (SELECT, ASK, CONSTRUCT) |

**Failure:** Parsing or semantic error from condition validation.

### 5. Effect Validation

**Code:** `hooks.rs:489-507`, effect-to-property mapping

| Effect | Requirement | Failure if Missing |
|--------|-------------|------------------|
| `emit-delta` | None (optional `kh:action` is ignored) | N/A |
| `ground-action` | MUST have `kh:action` | "effect 'ground-action' requires kh:action" |
| `refuse` | MUST have `kh:reason` | "effect 'refuse' requires kh:reason" |
| Other | Not in [emit-delta, ground-action, refuse] | "unknown effect: [value]" |

### 6. Constitutional (Handler) Gating

**Code:** `hooks.rs:365-391`, handler restriction + namespace protection

**Handler Check:**
- If hook has `kh:action`, that action MUST have exactly one `kh:handler`.
- Handler value MUST be: `http://seanchatmangpt.github.io/praxis/handler#sparql-construct`.
- Failure: "forbidden or unrecognized handler: [handler_iri]"

**CONSTRUCT Template Check:**
- If handler is `sparql-construct`, parse the `kh:query` as SPARQL CONSTRUCT.
- Verify template does NOT contain any triple where subject, predicate, or object is in `http://seanchatmangpt.github.io/praxis/kh#`.
- Failure: "CONSTRUCT template attempts to modify hook registry namespace"

### 7. Pack Size Limit

**Code:** `hooks.rs:416-421`

- Count `rdf:type kh:Hook` triples.
- If count > 12: "too many hooks declared: [count]; max 12"

### 8. Atomic Rollback

**Code:** `hooks.rs` + `lib.rs:429-487` `load_hook_pack`

- If any gate fails, the entire pack is refused.
- No triples from the pack are added to `self.triple_index`.
- No hooks are added to `self.hooks`.
- Return error immediately.

## Condition Evaluation (Integration Points)

### Delta Condition

**Code:** `hooks.rs:1008-1019`, `reasoner/mod.rs:332-334`

**Logic:**
```rust
fn delta_touches(delta: &GraphDelta, var: &str) -> bool {
    delta.additions.iter().chain(delta.removals.iter())
        .any(|t| {
            let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
            clean_term(&p_str) == clean_term(var)
        })
}
```

**Integration:** Called in `reasoner.materialize()` during per-hook evaluation, after rule fixpoint.

### Threshold Condition

**Code:** `hooks.rs:1021-1032`, `reasoner/mod.rs:335-337`

**Logic:**
```rust
fn count_pred_in_store(store: &TripleStore, var: &str) -> u64 {
    store.triple_index.triples.iter()
        .filter(|t| {
            let p_str = Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
            clean_term(&p_str) == clean_term(var)
        })
        .count() as u64
}
```

### Count Condition

**Code:** `hooks.rs:1034-1053`, `reasoner/mod.rs:339-341`

**Logic:**
```rust
fn count_pred_in_delta(delta: &GraphDelta, var: &str) -> u64 {
    let add_count = delta.additions.iter().filter(...).count();
    let rem_count = delta.removals.iter().filter(...).count();
    add_count + rem_count
}
```

### Window Condition

**Code:** `reasoner/mod.rs:343-351`

**Logic:**
```rust
let mut total = delta_count(&round_additions, var);
for past_adds in stratum_history.iter().rev().take(usize::from(*window) - 1) {
    total += delta_count(past_adds, var);
}
fired = cmp_holds(op, total, *k);
```

**Note:** `stratum_history` is a `Vec<Vec<Triple>>` maintained across iterations within a stratum. Each round's additions are appended.

### Datalog Condition

**Code:** `hooks.rs:1142-1242` (translation), `hooks.rs:1252-1293` (evaluation)

**Process:**
1. Parse `kh:program` as Datalog rules (atoms, arity validation, negation parsing).
2. Translate to N3 via `translate_datalog_to_n3`.
3. Create temporary store with post-state triples.
4. Load N3 rules into temporary store.
5. Materialize.
6. Search for goal predicate or rdf:type with goal as object.
7. Return true if found.

**Example translation:**
```
Datalog:  ancestor(?x, ?z) :- parent(?x, ?y), ancestor(?y, ?z).
N3:       { ?x <http://...#parent> ?y . ?y <http://...#ancestor> ?z } => { ?x <http://...#ancestor> ?z }.
```

### SHACL Condition

**Code:** `hooks.rs:1386-1410`

**Process:**
1. Parse `kh:program` as SHACL shapes.
2. Create temporary store with post-state.
3. Validate shapes against store.
4. Return true iff `!report.conforms`.

### ShEx Condition

**Code:** `hooks.rs:1412-1464`

**Process:**
1. Parse `kh:program` as ShEx schema (JSON or compact syntax).
2. Parse `kh:goal` as shape map.
3. Validate post-state against shapes.
4. Return true iff violations exist.

### N3 Condition

**Code:** `hooks.rs:1466-1491`

**Process:**
1. Parse `kh:program` as N3 rules.
2. Create temporary store with rules + post-state.
3. Materialize.
4. Check for denial violations.
5. Return true iff violations exist.

### SPARQL Condition

**Code:** `hooks.rs:1493-1514`

**Process:**
1. Parse `kh:query` as SPARQL.
2. Create temporary store with post-state.
3. Execute query.
4. Return true iff non-empty result.

## Effect Application

### EmitDelta Effect

**Code:** `reasoner/mod.rs:510-560`

**Process:**
1. If `kh:action` is present, look up the action resource.
2. Retrieve `kh:query` from action (must be SPARQL CONSTRUCT).
3. If hook's condition produced bindings (from SPARQL SELECT), iterate over rows and project CONSTRUCT for each.
4. Collect all constructed triples into `hook_additions`.
5. Apply each triple to `triple_index` via `apply_new_triple`.
6. If any triples added, generate receipt (Section 8 below).

### GroundAction Effect

**Code:** `reasoner/mod.rs:510-560` (same as EmitDelta in this implementation)

**Note:** Both `emit-delta` and `ground-action` are implemented via the same action-projection mechanism. The difference is semantic (outcome classification), not implementation.

### Refuse Effect

**Code:** `reasoner/mod.rs:477-508`

**Process:**
1. On hook firing with `Refuse` effect:
   - Retrieve all triples added since stratum start: `triple_index.triples[stratum_rollback_len..]`.
   - Remove each from `triple_index` via `remove_ref`.
   - Truncate `inferred` to `stratum_rollback_inferred_len`.
   - Create `TriggerDiagnostic` with reason in message.
   - Push `HookVerdictRecord` with `Verdict::Fired`.
   - Return `Err(format!("refused by hook '{}': {}", hook.name, reason))`.
2. On error return, `materialize()` catches it, restores `triple_index` to checkpoint, clears receipts/verdicts, returns empty inferred.

## Receipt Generation

### Canonical N-Quads

**Code:** `hooks.rs:784-838`, `canonicalize_quads()`

**Process:**
1. Convert each triple to N-Quads string via `serialize_quad`.
2. Collect all blank node labels (`_:...`).
3. Sort blank nodes by length (descending) to ensure correct prefix matching.
4. Map each blank node to a canonical label: `_:c14n0`, `_:c14n1`, etc.
5. Replace all blank node labels in quads.
6. Escape literal strings (backslash, quote, newline, carriage return, tab).
7. Sort all quad strings lexicographically.

### BLAKE3 Hashing

**Code:** `reasoner/mod.rs:567-573`

**Process:**
```rust
let lines = canonicalize_quads(&hook_additions);
let delta_quads = lines.join("\n");
let d_hash = blake3::hash(delta_quads.as_bytes()).to_hex().to_string();
let i_key = blake3::hash(
    format!("praxis:idempotency-key:v1{}", d_hash).as_bytes()
).to_hex().to_string();

let receipt = HookReceipt {
    hook_name: hook.name.clone(),
    delta_hash: d_hash.clone(),
    idempotency_key: i_key.clone(),
    delta_quads,
};
receipts.push(receipt);
```

## Event Gating

**Code:** `hooks.rs:1526-1542` (hook evaluation), `reasoner/mod.rs:307-326` (per-stratum gating)

**Logic:**
```rust
let gated = match hook.on.as_str() {
    "assert" => delta.additions.is_empty(),
    "retract" => delta.removals.is_empty(),
    _ => false,  // "any" or default: never gated
};

if gated {
    verdict = HookVerdict::Gated;
    diagnostics = None;
} else {
    // Evaluate condition...
}
```

## Scheduling

**Code:** `hooks.rs:533-586`, `schedule_hooks()`

**Algorithm:**
1. Build dependency graph: edge from `dep` to `hook` for each `hook.after = dep`.
2. Compute in-degree for each hook.
3. Initialize queue with all hooks of in-degree 0.
4. While queue not empty:
   - Pop hook with lowest (priority, iri) tuple.
   - Add to scheduled list.
   - For each dependent hook: decrement in-degree; if 0, enqueue.
5. If scheduled.len() < hooks.len(), cycle detected: return error.

**Integration:** Called in `load_hook_pack()` after validation, before storing hooks.

```rust
let extracted_hooks = hooks::validate_and_extract_hooks(&triples)?;
let scheduled_hooks = hooks::schedule_hooks(extracted_hooks)?;
self.hooks.extend(scheduled_hooks);
```

## Materialization Integration

**Code:** `lib.rs:239-267`, `reasoner/mod.rs:33-612`

**Flow:**
1. `store.materialize()` calls `reasoner.materialize()`.
2. Reasoner iterates strata: for each stratum, rules fire until fixpoint.
3. Within each stratum iteration (after rule fixpoint):
   - Build `round_additions` = triples added in this round.
   - For each hook in scheduled order:
     - Check event gate (`kh:on`).
     - Evaluate condition against post-state and delta.
     - Apply effect (if fired).
     - Record verdict.
     - Generate receipt (if non-empty effect).
   - If hooks added triples (`hook_changed`), set `changed = true` (loop iteration).
4. Move to next stratum.
5. On error (Refuse effect), restore from checkpoint, clear receipts/verdicts.

## Testing Patterns

### Admission Tests

```rust
#[test]
fn test_admission_passes() {
    let mut store = TripleStore::new();
    let pack = r#"@prefix kh: ... ex:h a kh:Hook ; kh:name "h" ; ..."#;
    assert!(store.load_hook_pack(pack).is_ok());
}

#[test]
fn test_admission_fails_malformed() {
    let mut store = TripleStore::new();
    let pack = r#"@prefix kh: ... ex:h a kh:Hook ; /* missing kh:kind */ ..."#;
    assert!(store.load_hook_pack(pack).is_err());
}
```

### Condition Tests

```rust
#[test]
fn test_delta_trigger() {
    let mut store = TripleStore::new();
    store.load_hook_pack(r#"
        ex:h a kh:Hook ;
            kh:name "h" ;
            kh:kind "delta" ;
            kh:var "http://example.org/status" ;
            kh:effect "emit-delta" .
    "#).unwrap();
    
    store.load_triples("ex:a <http://example.org/status> 'active' .", Syntax::Turtle).unwrap();
    store.materialize();
    
    assert!(!store.get_hook_receipts().is_empty());
}
```

### Receipt Tests

```rust
#[test]
fn test_receipt_determinism() {
    let mut store_a = TripleStore::new();
    let mut store_b = TripleStore::new();
    
    let pack = r#"..."#;
    store_a.load_hook_pack(pack).unwrap();
    store_b.load_hook_pack(pack).unwrap();
    
    store_a.load_triples("ex:a <http://example.org/x> 'y' . ex:b <http://example.org/x> 'y' .", Syntax::Turtle).unwrap();
    store_b.load_triples("ex:b <http://example.org/x> 'y' . ex:a <http://example.org/x> 'y' .", Syntax::Turtle).unwrap();
    
    store_a.materialize();
    store_b.materialize();
    
    assert_eq!(
        store_a.get_hook_receipts()[0].delta_hash,
        store_b.get_hook_receipts()[0].delta_hash
    );
}
```

## Common Implementation Pitfalls

1. **Non-deterministic ordering:** Always sort quads canonically before hashing. Test with multiple insertion orders.
2. **Event gate confusion:** `kh:on "assert"` means "gate if NO additions", not "fire if additions". Double-check logic.
3. **Keyword false positives:** Keywords are allowed if they start with known prefixes. Check whitelist carefully.
4. **Stratum rollback:** On `Refuse` effect, restore exact checkpoint; don't manually rebuild.
5. **Cycle detection:** Count scheduled hooks vs. input hooks; missing hooks indicate a cycle.
6. **Blank node normalization:** Normalize by first appearance in canonical order, not by label value.
7. **Transitive dependencies:** Topological sort handles arbitrary dependency chains automatically.
8. **Idempotency key derivation:** Include version string ("v1") to allow future changes without collision.

## References

- Full specification: `docs/v26.7.4/KNOWLEDGE_HOOKS_SPECIFICATION.md`
- Hook types and structures: `crates/praxis-graphlaw/src/hooks.rs`
- Materialization and evaluation: `crates/praxis-graphlaw/src/reasoner/mod.rs`
- Store API: `crates/praxis-graphlaw/src/lib.rs`
- End-to-end tests: `crates/praxis-graphlaw/tests/knowledge_hooks_e2e.rs`

