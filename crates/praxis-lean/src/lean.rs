use crate::error::{LeanRefusal, Result};
use crate::hash::{blake3_hex, file_blake3_hex};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Default location of the `elan`-managed Lean/Lake binaries in this
/// environment: they are NOT on the default `$PATH` (confirmed this
/// session), only under `~/.elan/bin`. CLI flags can still override these
/// with a bare command name (e.g. `"lean"`) if the caller's own `$PATH`
/// already includes them.
fn default_elan_bin(name: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.elan/bin/{name}")
}

/// Default `lean` command: absolute `~/.elan/bin/lean`.
pub fn default_lean_command() -> String {
    default_elan_bin("lean")
}

/// Default `lake` command: absolute `~/.elan/bin/lake`.
pub fn default_lake_command() -> String {
    default_elan_bin("lake")
}

/// Auto-detect whether `root` is a Lake-managed corpus, i.e. it has its own
/// `lakefile.lean` / `lakefile.toml` (as scaffolded by [`crate::cli::init`]
/// at `root.join("lakefile.lean")`) versus a bare-`lean`-verified corpus
/// with no Lake package of its own.
///
/// Used to default `--lake-env` when the caller doesn't force it: a
/// Lake-managed corpus (e.g. `tools/paper-factory/lean-lake/`, whose files
/// `import Mathlib...`) must be checked via `lake env lean`, which
/// resolves Mathlib and other Lake dependencies onto the search path.
/// Bare `lean <file>` on such a file fails with "unknown module prefix
/// 'Mathlib'" (exit 1) even though the file's proof is correct --
/// confirmed this session against
/// `tools/paper-factory/lean-lake/Praxis/Corpus/con_agent8.lean`, which
/// exits 1 under bare `lean` and exits 0 under `lake env lean` on the
/// identical file. A bare-`lean`-verified corpus (e.g.
/// `tools/paper-factory/lean-pilot/`, no `lakefile.lean`) has no Lake
/// package to resolve against, so bare `lean` is still correct there.
pub fn detect_lake_env(root: &Utf8Path) -> bool {
    root.join("lakefile.lean").is_file() || root.join("lakefile.toml").is_file()
}

/// Lean/Lake toolchain data recorded into receipts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanToolchain {
    pub lean_command: String,
    pub lake_command: Option<String>,
    pub lean_version: Option<String>,
    pub lake_version: Option<String>,
    pub toolchain_file_hash: Option<String>,
}

impl LeanToolchain {
    pub fn detect(root: &Utf8Path, lean_command: String, lake_command: Option<String>) -> Self {
        let lean_version = version_of(&lean_command, "--version").ok();
        let lake_version = lake_command
            .as_ref()
            .and_then(|cmd| version_of(cmd, "--version").ok());
        let toolchain_file_hash = file_blake3_hex(&root.join("lean-toolchain")).ok();
        Self {
            lean_command,
            lake_command,
            lean_version,
            lake_version,
            toolchain_file_hash,
        }
    }
}

fn version_of(command: &str, arg: &str) -> std::result::Result<String, ()> {
    let out = Command::new(command).arg(arg).output().map_err(|_| ())?;
    let text = if out.stdout.is_empty() {
        String::from_utf8_lossy(&out.stderr).to_string()
    } else {
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    Ok(text.trim().to_string())
}

/// Result of invoking Lean on one file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanCheck {
    pub file_path: Utf8PathBuf,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub stdout_hash: String,
    pub stderr_hash: String,
    pub stdout_preview: String,
    pub stderr_preview: String,
}

#[derive(Debug, Clone)]
pub struct LeanRunner {
    root: Utf8PathBuf,
    lean_command: String,
    lake_command: Option<String>,
    use_lake_env: bool,
}

impl LeanRunner {
    pub fn new(
        root: impl Into<Utf8PathBuf>,
        lean_command: impl Into<String>,
        lake_command: Option<String>,
        use_lake_env: bool,
    ) -> Self {
        Self {
            root: root.into(),
            lean_command: lean_command.into(),
            lake_command,
            use_lake_env,
        }
    }

    pub fn toolchain(&self) -> LeanToolchain {
        LeanToolchain::detect(
            &self.root,
            self.lean_command.clone(),
            self.lake_command.clone(),
        )
    }

    pub fn check_file(&self, file: &Utf8Path) -> Result<LeanCheck> {
        let output = if self.use_lake_env {
            let lake = self.lake_command.as_deref().unwrap_or("lake");
            Command::new(lake)
                .current_dir(&self.root)
                .args(["env", self.lean_command.as_str(), file.as_str()])
                .output()
                .map_err(|source| LeanRefusal::Io {
                    path: file.to_path_buf(),
                    source,
                })?
        } else {
            Command::new(&self.lean_command)
                .current_dir(&self.root)
                .arg(file.as_str())
                .output()
                .map_err(|source| LeanRefusal::Io {
                    path: file.to_path_buf(),
                    source,
                })?
        };

        let stdout = output.stdout;
        let stderr = output.stderr;
        Ok(LeanCheck {
            file_path: file.to_path_buf(),
            exit_code: output.status.code(),
            success: output.status.success(),
            stdout_hash: blake3_hex(&stdout),
            stderr_hash: blake3_hex(&stderr),
            stdout_preview: preview(&stdout),
            stderr_preview: preview(&stderr),
        })
    }
}

fn preview(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    let s = s.trim();
    if s.len() <= 2048 {
        s.to_string()
    } else {
        format!("{}…", &s[..2048])
    }
}
