use crate::error::{LeanRefusal, Result};
use crate::index::LeanDeclarationIndex;
use crate::lean::LeanRunner;
use crate::no_sorry::{AuditPolicy, NoSorryAudit};
use crate::receipt::{ReceiptLedger, VerificationReceipt};
use crate::report::VerificationReport;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

#[cfg(feature = "standalone-cli")]
use clap::{Args, Parser, Subcommand};

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Parser)]
#[command(
    name = "praxis-l4",
    version,
    about = "Praxis Lean 4 manufacturing wrapper"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Subcommand)]
pub enum Command {
    Init(InitArgs),
    Verify(VerifyArgs),
    NoSorry(NoSorryArgs),
    IndexBuild(IndexBuildArgs),
    Reconcile(ReconcileArgs),
    Report(ReportArgs),
}

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long, default_value = ".")]
    pub root: Utf8PathBuf,
    #[arg(long, default_value = "leanprover/lean4:stable")]
    pub toolchain: String,
}

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Args)]
pub struct VerifyArgs {
    #[arg(long, default_value = ".")]
    pub root: Utf8PathBuf,
    #[arg(long, default_value = "formalization_receipts_v2.jsonl")]
    pub receipts: Utf8PathBuf,
    #[arg(long, default_value_t = crate::lean::default_lean_command())]
    pub lean: String,
    #[arg(long, default_value_t = crate::lean::default_lake_command())]
    pub lake: String,
    #[arg(long, default_value_t = false)]
    pub lake_env: bool,
}

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Args)]
pub struct NoSorryArgs {
    #[arg(long, default_value = ".")]
    pub root: Utf8PathBuf,
}

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Args)]
pub struct IndexBuildArgs {
    #[arg(long, default_value = ".")]
    pub repo_root: Utf8PathBuf,
    #[arg(long)]
    pub corpus: Utf8PathBuf,
    #[arg(long)]
    pub lean_pilot_dir: Utf8PathBuf,
    #[arg(long)]
    pub out: Utf8PathBuf,
}

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Args)]
pub struct ReconcileArgs {
    #[arg(long)]
    pub index: Utf8PathBuf,
    #[arg(long)]
    pub receipts: Utf8PathBuf,
}

#[cfg(feature = "standalone-cli")]
#[derive(Debug, Args)]
pub struct ReportArgs {
    #[arg(long)]
    pub index: Utf8PathBuf,
    #[arg(long)]
    pub receipts: Utf8PathBuf,
    #[arg(long)]
    pub out: Utf8PathBuf,
}

#[cfg(feature = "standalone-cli")]
pub fn run_cli(cli: Cli) -> anyhow::Result<()> {
    // NOTE: argument order below matches each function's declaration order,
    // which is alphabetical-by-flag-name -- see the comment on `verify`
    // below for why.
    let value = match cli.command {
        Command::Init(args) => init(&args.root, args.toolchain),
        Command::Verify(args) => verify(
            args.lake,
            args.lake_env,
            args.lean,
            &args.receipts,
            &args.root,
        ),
        Command::NoSorry(args) => no_sorry(&args.root),
        Command::IndexBuild(args) => index_build(
            &args.corpus,
            &args.lean_pilot_dir,
            &args.out,
            &args.repo_root,
        ),
        Command::Reconcile(args) => reconcile(&args.index, &args.receipts),
        Command::Report(args) => report(&args.index, &args.out, &args.receipts),
    }
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

/// Scaffold a fresh Lake package. Deliberately does NOT touch any existing
/// bare-`lean`-verified files -- point `root` at a new sibling directory
/// (e.g. `tools/paper-factory/lean-lake/`), not `lean-pilot/`.
pub fn init(root: &Utf8Path, toolchain: String) -> Result<serde_json::Value> {
    fs::create_dir_all(root.join("Praxis")).map_err(|source| LeanRefusal::Io {
        path: root.to_path_buf(),
        source,
    })?;
    write_if_absent(&root.join("lean-toolchain"), &format!("{toolchain}\n"))?;
    write_if_absent(
        &root.join("lakefile.lean"),
        r#"import Lake
open Lake DSL

package «praxis-lean-pilot» where
  -- Add mathlib in a second lane when needed.

@[default_target]
lean_lib Praxis where
  roots := #[`Praxis]
"#,
    )?;
    write_if_absent(&root.join("Praxis.lean"), "import Praxis.Core\n")?;
    write_if_absent(
        &root.join("Praxis/Core.lean"),
        r#"namespace Praxis

/-- Raw observation. Filled by generated modules. -/
structure Observation where
  label : String
deriving Repr, DecidableEq

/-- Admitted observation. -/
structure AdmittedObservation where
  label : String
deriving Repr, DecidableEq

/-- Receipt marker. -/
structure Receipt where
  label : String
deriving Repr, DecidableEq

end Praxis
"#,
    )?;
    Ok(serde_json::json!({ "initialized_root": root, "toolchain": toolchain }))
}

fn write_if_absent(path: &Utf8Path, text: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LeanRefusal::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, text).map_err(|source| LeanRefusal::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Walk `root` for `.lean` files, kernel-check each one, run the no-sorry
/// audit, and append a genesis-folded receipt for every file -- never
/// trusting a prior report, always a fresh kernel invocation.
///
/// Parameter order is alphabetical by flag name (`lake`, `lake_env`,
/// `lean`, `receipts`, `root`), matching the order the `l4.rs.tmpl`
/// ggen template's `ORDER BY ?flag` SPARQL clause produces for the
/// generated `verbs::l4::l4_verify` wrapper that calls this function
/// positionally -- keep this order in sync with that template rather
/// than a more "natural" reading order, since a mismatch there is a
/// silent argument-swap bug, not a compile error, whenever two
/// same-typed parameters land in the wrong position.
pub fn verify(
    lake: String,
    lake_env: bool,
    lean: String,
    receipts: &Utf8Path,
    root: &Utf8Path,
) -> Result<serde_json::Value> {
    let runner = LeanRunner::new(root, lean, Some(lake), lake_env);
    let toolchain = runner.toolchain();
    let ledger = ReceiptLedger::new(receipts);
    let auditor = NoSorryAudit::new(AuditPolicy::default());
    let mut prev_chain_hash = ledger.tip_chain_hash()?;

    let mut results = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.map_err(|e| LeanRefusal::Io {
            path: root.to_path_buf(),
            source: std::io::Error::other(e.to_string()),
        })?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf())
            .map_err(LeanRefusal::NonUtf8Path)?;
        if path.extension() != Some("lean") {
            continue;
        }
        if path.file_name() == Some("lakefile.lean") {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let check = runner.check_file(rel)?;
        let findings = auditor.audit_file(&path)?;
        let label = rel.file_stem().unwrap_or("unknown").replace('_', ":");
        let receipt = VerificationReceipt::from_check(
            label,
            None,
            vec![],
            1,
            toolchain.clone(),
            check,
            findings,
            &prev_chain_hash,
        );
        prev_chain_hash = receipt.chain_hash.clone();
        results.push(serde_json::json!({
            "file": rel,
            "status": receipt.status,
        }));
        ledger.append(&receipt)?;
    }
    Ok(serde_json::json!({ "checked": results.len(), "results": results }))
}

/// Refuse any `.lean` file using `sorry`, `admit`, or an unauthorized
/// `axiom`, ignoring comments -- an independent check of a claim, not a
/// self-report.
pub fn no_sorry(root: &Utf8Path) -> Result<serde_json::Value> {
    let auditor = NoSorryAudit::new(AuditPolicy::default());
    let findings = auditor.audit_root(root)?;
    Ok(serde_json::json!({
        "passed": findings.is_empty(),
        "finding_count": findings.len(),
        "findings": findings,
    }))
}

/// Build a [`LeanDeclarationIndex`] from the real `corpus.ttl` via SPARQL.
///
/// Parameter order is alphabetical by flag name (`corpus`,
/// `lean_pilot_dir`, `out`, `repo_root`) -- see the comment on `verify`
/// above for why this must stay in sync with `l4.rs.tmpl`.
pub fn index_build(
    corpus: &Utf8Path,
    lean_pilot_dir: &Utf8Path,
    out: &Utf8Path,
    repo_root: &Utf8Path,
) -> Result<serde_json::Value> {
    let index = LeanDeclarationIndex::build_from_corpus(repo_root, corpus, lean_pilot_dir)?;
    index.save(out)?;
    Ok(serde_json::json!({ "record_count": index.records.len(), "out": out }))
}

/// Cross-reference the corpus index against the receipt ledger. Reads
/// either this crate's schema v2 receipts or the pre-existing schema v1
/// `formalization_receipts.jsonl` shape (see
/// `ReceiptLedger::read_all_any_schema`) -- reconciliation only needs a
/// label and a status, so both schemas are accepted.
pub fn reconcile(index_path: &Utf8Path, receipts_path: &Utf8Path) -> Result<serde_json::Value> {
    let index = LeanDeclarationIndex::load(index_path)?;
    let receipts = ReceiptLedger::new(receipts_path).read_all_any_schema()?;
    let report = VerificationReport::build(&index, &receipts);
    Ok(serde_json::to_value(report).unwrap_or(serde_json::Value::Null))
}

/// Write a [`VerificationReport`] as JSON.
///
/// Parameter order is alphabetical by flag name (`index`, `out`,
/// `receipts`) -- see the comment on `verify` above for why this must
/// stay in sync with `l4.rs.tmpl`.
pub fn report(
    index_path: &Utf8Path,
    out: &Utf8Path,
    receipts_path: &Utf8Path,
) -> Result<serde_json::Value> {
    let index = LeanDeclarationIndex::load(index_path)?;
    let receipts = ReceiptLedger::new(receipts_path).read_all_any_schema()?;
    let report = VerificationReport::build(&index, &receipts);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|source| LeanRefusal::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(&report).unwrap_or_default();
    fs::write(out, &text).map_err(|source| LeanRefusal::Io {
        path: out.to_path_buf(),
        source,
    })?;
    Ok(serde_json::json!({ "out": out }))
}
