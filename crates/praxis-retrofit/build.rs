fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src/");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let now =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();

    // Log build compilation event (OCEL 2.0 format)
    let build_log = format!(
        "{{\"event_id\":\"compile-{}\",\"activity\":\"SubstrateLabor\",\"timestamp\":\"{}\",\"objects\":[{{\"ocel:objectId\":\"praxis-retrofit\",\"ocel:type\":\"Artifact\"}}]}}",
        now, now
    );

    let build_log_path = std::path::Path::new(&out_dir).join("build_event.jsonl");
    std::fs::write(build_log_path, build_log).unwrap();
}
