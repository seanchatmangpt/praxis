//! The foreign firing verifier, tested: the `firing` subcommand of
//! `scripts/foreign_verify_graph.py` (python3 + `b3sum`) must agree with
//! `fire_hooks` on an honest outer-chain receipt, agree on a refusal
//! receipt, and fail on a tampered verdict payload.

use std::path::{Path, PathBuf};

use praxis_synthesis::handlers::HANDLER_NS;
use praxis_synthesis::{
    fire_hooks, FiringOutcome, HandlerRegistry, MeaningSource, Origin, Reference,
};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

fn b3sum_available() -> bool {
    std::process::Command::new("b3sum")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "praxis-synth-firing-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

fn run_verifier(base: &Path, adds: &Path, removes: &Path, receipt: &Path) -> std::process::Output {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/foreign_verify_graph.py"
    );
    std::process::Command::new("python3")
        .args([
            script,
            "firing",
            base.to_str().expect("utf8 path"),
            adds.to_str().expect("utf8 path"),
            removes.to_str().expect("utf8 path"),
            receipt.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run foreign firing verifier")
}

fn kernel_with_binding(delegability: &str) -> String {
    let mut base = KERNEL.to_string();
    for cap in &["orientToFather", "surrenderWill", "requestDailyBread", "writePrayerReceipt"] {
        base.push_str(&format!(
            "\n<http://seanchatmangpt.github.io/praxis/prayer#{cap}> \
             <http://seanchatmangpt.github.io/praxis/workflow#handler> <{HANDLER_NS}deterministic-v1> ;\n\
             <http://seanchatmangpt.github.io/praxis/workflow#delegability> \"{delegability}\" .\n"
        ));
    }
    base
}

struct Artifacts {
    base: PathBuf,
    adds: PathBuf,
    removes: PathBuf,
    receipt: PathBuf,
}

impl Artifacts {
    fn write(tag: &str, base_ttl: &str, source: &MeaningSource, receipt_json: &str) -> Self {
        let a = Self {
            base: temp_path(&format!("{tag}-base.ttl")),
            adds: temp_path(&format!("{tag}-adds.ttl")),
            removes: temp_path(&format!("{tag}-removes.ttl")),
            receipt: temp_path(&format!("{tag}-receipt.json")),
        };
        std::fs::write(&a.base, base_ttl.as_bytes()).expect("write base");
        std::fs::write(&a.adds, source.adds_ttl.as_bytes()).expect("write adds");
        std::fs::write(&a.removes, source.removes_ttl.as_bytes()).expect("write removes");
        std::fs::write(&a.receipt, receipt_json.as_bytes()).expect("write receipt");
        a
    }

    fn run(&self) -> std::process::Output {
        run_verifier(&self.base, &self.adds, &self.removes, &self.receipt)
    }
}

impl Drop for Artifacts {
    fn drop(&mut self) {
        for p in [&self.base, &self.adds, &self.removes, &self.receipt] {
            let _ = std::fs::remove_file(p);
        }
    }
}

fn src(adds: &str) -> MeaningSource {
    MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds.to_string(),
        removes_ttl: String::new(),
    }
}

#[test]
fn foreign_firing_verifier_agrees_on_an_honest_completed_receipt() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let base = kernel_with_binding("verifiable");
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(receipt.outcome, FiringOutcome::Completed);
    assert_eq!(receipt.inner.len(), 1, "one inner v1 chain folded");

    let arts = Artifacts::write(
        "honest",
        &base,
        &source,
        &serde_json::to_string(&receipt).expect("json"),
    );
    let out = arts.run();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "foreign firing verifier disagreed: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("VERIFIED firing"), "{stdout}");
    for stage in ["event_hash", "admission_hash", "handler_hash", "hook_hash",
                  "outcome_hash", "chain"] {
        assert!(stdout.contains(&format!("PASS {stage}")), "{stage}: {stdout}");
    }
    assert!(stdout.contains("refolded-from-payload"), "{stdout}");
}

#[test]
fn foreign_firing_verifier_agrees_on_a_declared_refusal_receipt() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let reference = Reference::genesis(KERNEL).expect("admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}threat999> <{LIFE}hasUnboundedThreat> 1 ."));
    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("receipted");
    assert!(matches!(receipt.outcome, FiringOutcome::Refused { .. }));
    assert!(receipt.inner.is_empty(), "no-action sentinel path");

    let arts = Artifacts::write(
        "refusal",
        KERNEL,
        &source,
        &serde_json::to_string(&receipt).expect("json"),
    );
    let out = arts.run();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "refusal receipt must verify foreign: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("VERIFIED firing"), "{stdout}");
    assert!(stdout.contains("0 inner chain(s)"), "{stdout}");
}

#[test]
fn foreign_firing_verifier_fails_a_tampered_verdict_payload() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let base = kernel_with_binding("verifiable");
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let mut receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");

    // Forge one verdict body: the honest hook_hash no longer matches the
    // refolded payload — the foreign verifier must name hook_hash and,
    // downstream, the chain cannot re-fold either.
    receipt.verdicts[0].hook_name = "forged".to_string();

    let arts = Artifacts::write(
        "tamper",
        &base,
        &source,
        &serde_json::to_string(&receipt).expect("json"),
    );
    let out = arts.run();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "tampered verdict payload must fail: {stdout}"
    );
    assert!(stdout.contains("FAIL hook_hash"), "{stdout}");
}

/// Adversarial finding (closed): forging the embedded `admission` /
/// `bindings` / `agents` objects while leaving their flat hash strings
/// (`admission_hash` / `handler_hash` / `agent_registry_hash`) untouched
/// used to sail through `verify_firing` — it never read those embedded
/// fields at all, only comparing its own from-scratch TTL recomputation
/// against the (still-correct) hash string. Now each is also payload-bound
/// (`refold_admission`/`refold_bindings`/`refold_agents`), so a forged body
/// is caught even though the flat hash string is honest.
#[test]
fn foreign_firing_verifier_fails_a_forged_admission_body_behind_an_honest_hash() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let base = kernel_with_binding("verifiable");
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let mut receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");

    // Forge the displayed admission body only — admission_hash (and every
    // other top-level hash string) is left exactly as honestly computed.
    receipt.admission.epoch = 999;

    let arts = Artifacts::write(
        "forged-admission",
        &base,
        &source,
        &serde_json::to_string(&receipt).expect("json"),
    );
    let out = arts.run();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "forged admission body behind an honest admission_hash must fail: {stdout}"
    );
    assert!(stdout.contains("PASS admission_hash"), "{stdout}");
    assert!(stdout.contains("FAIL admission payload"), "{stdout}");
}

/// Same doctrine as above, for the embedded `bindings` array: forging a
/// binding's declared delegability grade while leaving `handler_hash`
/// untouched must now be caught.
#[test]
fn foreign_firing_verifier_fails_a_forged_bindings_body_behind_an_honest_hash() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let base = kernel_with_binding("verifiable");
    let reference = Reference::genesis(&base).expect("kernel admits");
    let registry = HandlerRegistry::builtin();
    let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
    let mut receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert!(!receipt.bindings.is_empty(), "must have at least one binding to forge");

    receipt.bindings[0].handler = "http://forged/handler".to_string();

    let arts = Artifacts::write(
        "forged-bindings",
        &base,
        &source,
        &serde_json::to_string(&receipt).expect("json"),
    );
    let out = arts.run();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "forged bindings body behind an honest handler_hash must fail: {stdout}"
    );
    assert!(stdout.contains("PASS handler_hash"), "{stdout}");
    assert!(stdout.contains("FAIL bindings payload"), "{stdout}");
}
