use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, Verifier};
use std::path::{Path, PathBuf};

const BLOCKING: &[(&str, &str)] = &[
    ("unimplemented!", "HOLLOW-001"),
    ("todo!", "HOLLOW-002"),
    ("// TODO:", "HOLLOW-004"),
    ("// FIXME:", "HOLLOW-005"),
    ("// PLACEHOLDER", "HOLLOW-006"),
];

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct ReceiptData {
    pub project_name: String,
    pub timestamp: String,
    pub source_digest: String,
    pub public_key: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct ComplianceReceipt {
    pub data: ReceiptData,
    pub signature: String,
}

#[derive(Parser, Debug)]
#[command(name = "praxis-guard")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Check {
        #[arg(long)]
        project: PathBuf,

        #[arg(long)]
        output: PathBuf,
    },
    Verify {
        #[arg(long)]
        receipt: PathBuf,

        #[arg(long)]
        project: PathBuf,
    },
}

fn run_cargo_cmd(project_dir: &Path, args: &[&str]) -> anyhow::Result<()> {
    println!("Running: cargo {:?}", args);
    let status = std::process::Command::new("cargo")
        .args(args)
        .current_dir(project_dir)
        .status()?;
    if !status.success() {
        anyhow::bail!("cargo command failed: cargo {:?}", args);
    }
    Ok(())
}

fn compute_source_digest(project_dir: &Path) -> anyhow::Result<String> {
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        anyhow::bail!("src/ directory not found in {:?}", project_dir);
    }

    let mut rs_files = Vec::new();
    for entry in walkdir::WalkDir::new(&src_dir).into_iter().flatten() {
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
        {
            rs_files.push(entry.path().to_path_buf());
        }
    }
    rs_files.sort();

    let mut chain = chatman_common::chain::RollingChain::new("praxis-guard");
    for file in &rs_files {
        let rel_path = file.strip_prefix(project_dir)?;
        let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
        let content = std::fs::read(file)?;

        chain.push(rel_path_str.as_bytes());
        chain.push(&content);
    }
    Ok(chain.finalize())
}

fn load_or_generate_keypair(
    project_dir: &Path,
) -> anyhow::Result<(ed25519_dalek::SigningKey, ed25519_dalek::VerifyingKey)> {
    let keys_dir = project_dir.join(".praxis/keys");
    std::fs::create_dir_all(&keys_dir)?;
    let private_path = keys_dir.join("private.key");
    let public_path = keys_dir.join("public.key");

    if private_path.exists() && public_path.exists() {
        let private_bytes = std::fs::read(&private_path)?;
        let public_bytes = std::fs::read(&public_path)?;

        let private_array: [u8; 32] = private_bytes
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid private key size"))?;
        let public_array: [u8; 32] = public_bytes
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid public key size"))?;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&private_array);
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&public_array)?;
        Ok((signing_key, verifying_key))
    } else {
        use rand_core::OsRng;
        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        std::fs::write(&private_path, signing_key.to_bytes())?;
        std::fs::write(&public_path, verifying_key.to_bytes())?;
        Ok((signing_key, verifying_key))
    }
}

fn run_check(project_dir: &Path, receipt_path: &Path) -> anyhow::Result<()> {
    // 1. Run cargo check, cargo test, cargo fmt --check
    run_cargo_cmd(project_dir, &["check"])?;
    run_cargo_cmd(project_dir, &["test"])?;
    run_cargo_cmd(project_dir, &["fmt", "--", "--check"])?;

    // 2. Scan source files recursively for forbidden stubs
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        anyhow::bail!("src/ directory not found in {:?}", project_dir);
    }

    let mut found_blocking = false;

    // Structural conformance trackers
    let mut has_phantom_data = false;
    let mut has_raw_zst = false;
    let mut has_validated_zst = false;
    let mut has_admitted_zst = false;
    let mut has_evidence_wrapper = false;
    let mut has_admit_trait = false;
    let mut has_rule_pack_server_impl = false;

    let walk_dir = walkdir::WalkDir::new(&src_dir);
    for entry in walk_dir.into_iter().flatten() {
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let path = entry.path();
        let src = std::fs::read_to_string(path)?;

        for (line_no, line) in src.lines().enumerate() {
            // First check the comment patterns on the original line
            for &(pat, code) in BLOCKING {
                if pat.starts_with("//") {
                    if line.contains(pat) {
                        eprintln!(
                            "[{}] {}:{}: {}",
                            code,
                            path.display(),
                            line_no + 1,
                            line.trim()
                        );
                        found_blocking = true;
                    }
                }
            }

            // Strip single-line comments (any text after // on a line)
            let stripped_line = if let Some(idx) = line.find("//") {
                &line[..idx]
            } else {
                line
            };

            // Scan the stripped line for todo! and unimplemented!
            for &(pat, code) in BLOCKING {
                if !pat.starts_with("//") {
                    if stripped_line.contains(pat) {
                        eprintln!(
                            "[{}] {}:{}: {}",
                            code,
                            path.display(),
                            line_no + 1,
                            line.trim()
                        );
                        found_blocking = true;
                    }
                }
            }
        }

        // Structural conformance checks
        if src.contains("PhantomData") {
            has_phantom_data = true;
        }
        if src.contains("struct Raw;") || src.contains("struct Raw") {
            has_raw_zst = true;
        }
        if src.contains("struct Validated;") || src.contains("struct Validated") {
            has_validated_zst = true;
        }
        if src.contains("struct Admitted;") || src.contains("struct Admitted") {
            has_admitted_zst = true;
        }
        if src.contains("struct Evidence") {
            has_evidence_wrapper = true;
        }
        if src.contains("trait Admit") {
            has_admit_trait = true;
        }
        if src.contains("impl RulePackServer for") || src.contains("impl<T> RulePackServer for") {
            has_rule_pack_server_impl = true;
        }
    }

    if found_blocking {
        anyhow::bail!("Forbidden stubs detected.");
    }

    let mut structural_failed = false;
    if !has_phantom_data {
        eprintln!("[ERROR] Missing: PhantomData typestates");
        structural_failed = true;
    }
    if !has_raw_zst {
        eprintln!("[ERROR] Missing: Raw ZST marker");
        structural_failed = true;
    }
    if !has_validated_zst {
        eprintln!("[ERROR] Missing: Validated ZST marker");
        structural_failed = true;
    }
    if !has_admitted_zst {
        eprintln!("[ERROR] Missing: Admitted ZST marker");
        structural_failed = true;
    }
    if !has_evidence_wrapper {
        eprintln!("[ERROR] Missing: Evidence wrapper");
        structural_failed = true;
    }
    if !has_admit_trait {
        eprintln!("[ERROR] Missing: Admit trait");
        structural_failed = true;
    }
    if !has_rule_pack_server_impl {
        eprintln!("[ERROR] Missing: RulePackServer implementation");
        structural_failed = true;
    }

    if structural_failed {
        anyhow::bail!("Structural conformance failed.");
    }

    // 4. Compute deterministic BLAKE3 digest of source tree
    let root_hash = compute_source_digest(project_dir)?;
    println!("Source tree digest: {}", root_hash);

    // 5. Sign the compliance receipt JSON
    let (signing_key, verifying_key) = load_or_generate_keypair(project_dir)?;

    // Get project name from Cargo.toml or directory name
    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    let data = ReceiptData {
        project_name,
        timestamp: chrono::Utc::now().to_rfc3339(),
        source_digest: root_hash,
        public_key: hex::encode(verifying_key.to_bytes()),
    };

    let data_bytes = serde_json::to_vec(&data)?;
    let signature = signing_key.sign(&data_bytes);

    let receipt = ComplianceReceipt {
        data,
        signature: hex::encode(signature.to_bytes()),
    };

    // 6. Write receipt.json
    if let Some(parent) = receipt_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let receipt_json = serde_json::to_string_pretty(&receipt)?;
    std::fs::write(receipt_path, receipt_json)?;
    println!("Signed receipt written to {:?}", receipt_path);

    Ok(())
}

fn run_verify(receipt_path: &Path, project_dir: &Path) -> anyhow::Result<()> {
    // 1. Read the receipt
    let receipt_str = std::fs::read_to_string(receipt_path)?;
    let receipt: ComplianceReceipt = serde_json::from_str(&receipt_str)?;

    // Verify signature against embedded public key
    let sig_bytes = hex::decode(&receipt.signature)?;
    let sig_array: [u8; 64] = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid signature size"))?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

    let pub_key_bytes = hex::decode(&receipt.data.public_key)?;
    let pub_key_array: [u8; 32] = pub_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid public key size"))?;
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pub_key_array)?;

    let data_bytes = serde_json::to_vec(&receipt.data)?;
    verifying_key.verify(&data_bytes, &signature)?;

    // Ensure the public key matches the project's public key in .praxis/keys/
    let project_pub_path = project_dir.join(".praxis/keys/public.key");
    if !project_pub_path.exists() {
        anyhow::bail!("Project public key not found in .praxis/keys/public.key");
    }
    let project_pub_bytes = std::fs::read(&project_pub_path)?;
    if project_pub_bytes != pub_key_bytes {
        anyhow::bail!("Public key in receipt does not match project's public key");
    }

    // 2. Recompute source tree BLAKE3 digest and assert match
    let computed_digest = compute_source_digest(project_dir)?;
    if computed_digest != receipt.data.source_digest {
        anyhow::bail!(
            "Source tree digest mismatch! Receipt: {}, computed: {}",
            receipt.data.source_digest,
            computed_digest
        );
    }

    println!("Verification successful! Project is compliant and matches signature.");
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check { project, output } => run_check(&project, &output),
        Commands::Verify { receipt, project } => run_verify(&receipt, &project),
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}
