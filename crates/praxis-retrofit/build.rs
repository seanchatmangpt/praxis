//! Build script for `praxis-retrofit`: emits an OCEL 2.0 compilation event
//! (`build_event.jsonl` in `OUT_DIR`) recording that this crate's substrate
//! labor ran, for downstream OCEL-based build provenance tooling.

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src/");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Log build compilation event (OCEL 2.0 format)
    let build_log = format!(
        "{{\"event_id\":\"compile-{now}\",\"activity\":\"SubstrateLabor\",\"timestamp\":\"{now}\",\"objects\":[{{\"ocel:objectId\":\"praxis-retrofit\",\"ocel:type\":\"Artifact\"}}]}}"
    );

    let build_log_path = std::path::Path::new(&out_dir).join("build_event.jsonl");
    std::fs::write(build_log_path, build_log).unwrap();
}
