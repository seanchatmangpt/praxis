//! Sandbox fixture staging and model-output application.
//!
//! v1 scope (per the plan): fixtures are copied into a fresh tempdir and never
//! mutated in place; the model's output is expected to contain exactly one fenced
//! ```rust ... ``` code block, which is written verbatim over the fixture's target
//! file. Real diff/patch application and multi-file repo-level translation are an
//! explicit v2 scope cut.

use std::path::{Path, PathBuf};

/// Errors from staging a fixture or applying model output.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    /// The fixture directory doesn't exist or couldn't be read.
    #[error("fixture directory {path} is not readable: {source}")]
    FixtureUnreadable {
        /// Path that failed to read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A tempdir could not be created.
    #[error("failed to create sandbox tempdir: {0}")]
    TempDir(#[source] std::io::Error),

    /// A file copy failed while staging the fixture.
    #[error("failed to copy {from} to {to}: {source}")]
    Copy {
        /// Source path.
        from: PathBuf,
        /// Destination path.
        to: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The model output contained no fenced `` ```rust `` code block.
    #[error("model output contains no fenced ```rust code block")]
    NoRustCodeBlock,

    /// Writing the extracted code to the target path failed.
    #[error("failed to write extracted code to {path}: {source}")]
    Write {
        /// Path that failed to write.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Result alias scoped to sandbox operations.
pub type SandboxResult<T> = std::result::Result<T, SandboxError>;

/// Recursively copy `fixture_dir` into a fresh [`tempfile::TempDir`] and return it.
///
/// The original fixture directory is never mutated.
///
/// # Errors
///
/// Returns [`SandboxError::TempDir`] if the tempdir can't be created, or
/// [`SandboxError::FixtureUnreadable`] / [`SandboxError::Copy`] if walking or copying
/// the fixture tree fails.
pub fn stage_fixture(fixture_dir: &Path) -> SandboxResult<tempfile::TempDir> {
    let dir = tempfile::tempdir().map_err(SandboxError::TempDir)?;
    copy_dir_recursive(fixture_dir, dir.path())?;
    Ok(dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> SandboxResult<()> {
    let entries = std::fs::read_dir(src).map_err(|source| SandboxError::FixtureUnreadable {
        path: src.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| SandboxError::FixtureUnreadable {
            path: src.to_path_buf(),
            source,
        })?;
        let file_type = entry
            .file_type()
            .map_err(|source| SandboxError::FixtureUnreadable {
                path: entry.path(),
                source,
            })?;
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|source| SandboxError::Copy {
                from: entry.path(),
                to: dst_path.clone(),
                source,
            })?;
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &dst_path).map_err(|source| SandboxError::Copy {
                from: entry.path(),
                to: dst_path.clone(),
                source,
            })?;
        }
        // Symlinks are deliberately skipped in v1 (no fixtures are expected to use them).
    }
    Ok(())
}

/// Extract the first fenced ```rust ... ``` code block from `model_output` and write
/// it to `dir.join(target_rel_path)`, overwriting any existing content.
///
/// Uses a simple `str::find`/`split` scan — no `regex` dependency needed for this
/// bounded, well-known fence shape.
///
/// # Errors
///
/// Returns [`SandboxError::NoRustCodeBlock`] if no fenced block is found, or
/// [`SandboxError::Write`] if writing the target file fails.
pub fn apply_model_output(
    dir: &Path,
    target_rel_path: &Path,
    model_output: &str,
) -> SandboxResult<()> {
    let code = extract_first_rust_block(model_output).ok_or(SandboxError::NoRustCodeBlock)?;
    let target = dir.join(target_rel_path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|source| SandboxError::Write {
            path: target.clone(),
            source,
        })?;
    }
    std::fs::write(&target, code).map_err(|source| SandboxError::Write {
        path: target,
        source,
    })
}

/// Find the first `` ```rust `` fenced code block and return its inner content.
fn extract_first_rust_block(text: &str) -> Option<String> {
    // Accept the exact "```rust" fence (with optional trailing content on the same
    // line, e.g. "```rust\n") as well as a bare "```" fence immediately followed by
    // "rust" on its own -- both are common model output shapes.
    let fence_markers = ["```rust", "``` rust"];
    let start_idx = fence_markers
        .iter()
        .find_map(|marker| text.find(marker).map(|i| (i, marker.len())));
    let (marker_pos, marker_len) = start_idx?;

    let after_marker = &text[marker_pos + marker_len..];
    // Skip to the end of the fence's opening line.
    let content_start = after_marker.find('\n').map_or(0, |i| i + 1);
    let body = &after_marker[content_start..];

    let end_pos = body.find("```")?;
    Some(body[..end_pos].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_fixture_into_fresh_tempdir_without_mutating_original() {
        let src = tempfile::tempdir().expect("src tempdir");
        std::fs::write(src.path().join("Cargo.toml"), "[package]\nname = \"f\"\n").expect("write");
        std::fs::create_dir_all(src.path().join("src")).expect("mkdir src");
        std::fs::write(src.path().join("src/lib.rs"), "pub fn f() {}\n").expect("write lib.rs");

        let staged = stage_fixture(src.path()).expect("stage_fixture should succeed");
        assert!(staged.path().join("Cargo.toml").exists());
        assert!(staged.path().join("src/lib.rs").exists());
        // Original untouched.
        assert!(src.path().join("Cargo.toml").exists());
    }

    #[test]
    fn extracts_and_writes_first_rust_block() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output =
            "Here is the fix:\n\n```rust\nfn add(a: i32, b: i32) -> i32 { a + b }\n```\n\nDone.";
        apply_model_output(dir.path(), Path::new("src/lib.rs"), output)
            .expect("apply should succeed");

        let written = std::fs::read_to_string(dir.path().join("src/lib.rs")).expect("read back");
        assert!(written.contains("fn add"));
    }

    #[test]
    fn errors_when_no_rust_block_present() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = apply_model_output(dir.path(), Path::new("src/lib.rs"), "no code here")
            .expect_err("should fail without a rust block");
        assert!(matches!(err, SandboxError::NoRustCodeBlock));
    }
}
