use praxis_synthesis::{load_hook_pack, Refusal};
use std::fs;

fn create_temp_pack(
    name: &str,
    version: &str,
    description: &str,
    dialects: &[&str],
    ttl: &str,
) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let toml = format!(
        r#"[pack]
name = "{name}"
version = "{version}"
description = "{description}"
required_dialects = [{dialects}]
"#,
        dialects = dialects
            .iter()
            .map(|d| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(", ")
    );
    fs::write(dir.path().join("pack.toml"), toml).unwrap();
    fs::write(dir.path().join("ontology.ttl"), ttl).unwrap();
    dir
}

#[test]
fn test_valid_pack_admission() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:on "assert" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "some refusal reason" ;
    hook:priority 3 .
"#;
    let pack_dir = create_temp_pack("valid-pack", "1.0.0", "A valid pack", &["delta"], ttl);
    let pack = load_hook_pack(pack_dir.path()).expect("should load valid pack");
    assert_eq!(pack.name, "valid-pack");
    assert_eq!(pack.version, "1.0.0");
    assert_eq!(pack.hooks.len(), 1);
    let hook = &pack.hooks[0];
    assert_eq!(hook.name, "h1");
    assert_eq!(hook.priority, 3);
}

#[test]
fn test_pack_missing_toml() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ontology.ttl"), "").unwrap();
    let res = load_hook_pack(dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::InvalidInput { detail } => assert!(detail.contains("pack.toml missing")),
        other => panic!("expected InvalidInput, got {:?}", other),
    }
}

#[test]
fn test_pack_missing_ontology() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("pack.toml"),
        r#"[pack]
name = "test"
version = "1.0"
description = "test"
"#,
    )
    .unwrap();
    let res = load_hook_pack(dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::InvalidInput { detail } => assert!(detail.contains("ontology.ttl missing")),
        other => panic!("expected InvalidInput, got {:?}", other),
    }
}

#[test]
fn test_pack_unsupported_dialect() {
    let pack_dir = create_temp_pack("bad-dialect", "1.0.0", "desc", &["sparql-select"], "");
    let res = load_hook_pack(pack_dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::ConditionUnsupported { kind, .. } => assert_eq!(kind, "sparql-select"),
        other => panic!("expected ConditionUnsupported, got {:?}", other),
    }
}

#[test]
fn test_pack_duplicate_hook_names() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "dup" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" .

ex:h2 a hook:Hook ;
    hook:name "dup" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" .
"#;
    let pack_dir = create_temp_pack("dup-names", "1.0.0", "desc", &["delta"], ttl);
    let res = load_hook_pack(pack_dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::HookIllFormed { detail, .. } => assert!(detail.contains("duplicate hook name")),
        other => panic!("expected HookIllFormed, got {:?}", other),
    }
}

#[test]
fn test_pack_duplicate_hook_iris() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:dup a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" .

ex:dup a hook:Hook ;
    hook:name "h2" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" .
"#;
    let pack_dir = create_temp_pack("dup-iris", "1.0.0", "desc", &["delta"], ttl);
    let res = load_hook_pack(pack_dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::HookIllFormed { detail, .. } => assert!(detail.contains("multiple hook:name")),
        other => panic!("expected HookIllFormed, got {:?}", other),
    }
}

#[test]
fn test_deterministic_scheduling_with_priority() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:priority 5 .

ex:h2 a hook:Hook ;
    hook:name "h2" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:priority 2 .
"#;
    let pack_dir = create_temp_pack("priority-sort", "1.0.0", "desc", &["delta"], ttl);
    let pack = load_hook_pack(pack_dir.path()).unwrap();
    assert_eq!(pack.hooks[0].name, "h2");
    assert_eq!(pack.hooks[1].name, "h1");
}

#[test]
fn test_deterministic_scheduling_with_dependencies() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:priority 2 ;
    hook:after ex:h2 .

ex:h2 a hook:Hook ;
    hook:name "h2" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:priority 5 .
"#;
    let pack_dir = create_temp_pack("dependency-sort", "1.0.0", "desc", &["delta"], ttl);
    let pack = load_hook_pack(pack_dir.path()).unwrap();
    assert_eq!(pack.hooks[0].name, "h2");
    assert_eq!(pack.hooks[1].name, "h1");
}

#[test]
fn test_scheduling_unknown_dependency() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:after ex:unknown .
"#;
    let pack_dir = create_temp_pack("unknown-dep", "1.0.0", "desc", &["delta"], ttl);
    let res = load_hook_pack(pack_dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::HookIllFormed { detail, .. } => {
            assert!(detail.contains("unknown after-dependency"))
        }
        other => panic!("expected HookIllFormed, got {:?}", other),
    }
}

#[test]
fn test_scheduling_cycle_detection() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:after ex:h2 .

ex:h2 a hook:Hook ;
    hook:name "h2" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:after ex:h1 .
"#;
    let pack_dir = create_temp_pack("cycle-dep", "1.0.0", "desc", &["delta"], ttl);
    let res = load_hook_pack(pack_dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::HookIllFormed { detail, .. } => assert!(detail.contains("dependency cycle")),
        other => panic!("expected HookIllFormed, got {:?}", other),
    }
}

#[test]
fn test_forbidden_handler_refused() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "action" ;
    hook:action ex:wf1 .

ex:wf1 a wf:Capability ;
    wf:handler <http://seanchatmangpt.github.io/praxis/handler#shell-exec> .
"#;
    let pack_dir = create_temp_pack("forbidden-handler", "1.0.0", "desc", &["delta"], ttl);
    let res = load_hook_pack(pack_dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::HookIllFormed { detail, .. } => assert!(
            detail.contains("forbidden handler IRI") || detail.contains("forbidden keyword")
        ),
        other => panic!("expected HookIllFormed, got {:?}", other),
    }
}

#[test]
fn test_forbidden_keyword_in_string_literal_refused() {
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "this reason tries to run curl http://malicious.com" .
"#;
    let pack_dir = create_temp_pack("forbidden-keyword", "1.0.0", "desc", &["delta"], ttl);
    let res = load_hook_pack(pack_dir.path());
    assert!(res.is_err());
    match res.unwrap_err() {
        Refusal::HookIllFormed { detail, .. } => assert!(detail.contains("forbidden keyword")),
        other => panic!("expected HookIllFormed, got {:?}", other),
    }
}
