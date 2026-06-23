//! Git-as-runtime patterns from gitvan: distributed locks and audit ledgers.
//!
//! This module provides two primitives that use git's internal mechanics as
//! a lightweight coordination layer — no external services required:
//!
//! - [`GitLock`] — RAII distributed lock via `git update-ref`. Atomic across
//!   concurrent processes sharing the same git repository.
//!
//! - [`GitAuditLedger`] — Append-only NDJSON audit log via `git notes append`.
//!   Each entry carries a timestamp, event type, and BLAKE3 payload hash so the
//!   ledger is independently verifiable.
//!
//! # Git-as-Runtime Design
//!
//! From the gitvan survey finding (INN-5), git's object store and ref namespace
//! already provide the primitives needed for distributed coordination:
//!
//! - **Atomic CAS:** `git update-ref --create-reflog` with an expected-value check
//!   is atomic on POSIX filesystems because git uses `rename(2)` for the final
//!   write. Two processes racing for the same ref will see one succeed and one fail.
//!
//! - **Immutable append-only log:** `git notes append --ref=<ns>` appends bytes to
//!   the note attached to a commit. Because git objects are content-addressed and
//!   immutable, old entries can never be overwritten — only the ref pointer moves.
//!
//! # Example
//!
//! ```no_run
//! use chatman_common::git_runtime::{GitLock, GitAuditLedger};
//! use std::path::Path;
//!
//! let repo = Path::new(".");
//!
//! // Acquire a distributed lock for CI mutual exclusion.
//! let lock = GitLock::acquire(repo, "ci/integration-db").unwrap();
//! // ... do exclusive work ...
//! drop(lock); // lock released on drop
//!
//! // Append an audit entry after the work succeeds.
//! let ledger = GitAuditLedger::new(repo);
//! let payload = serde_json::json!({ "suite": "integration", "passed": true });
//! ledger.append("test.completed", &payload).unwrap();
//! ```

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Return the current epoch seconds (UTC).
fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// BLAKE3 hex digest of arbitrary bytes (32 bytes → 64 hex chars).
fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

// ---------------------------------------------------------------------------
// GitLock
// ---------------------------------------------------------------------------

/// A RAII distributed lock backed by a git ref.
///
/// Acquiring the lock writes a git ref `refs/locks/<name>` pointing at the
/// current `HEAD` commit.  Releasing it (via `Drop`) deletes that ref.
///
/// Two processes in the same repository racing for the same lock name will see
/// at most one succeed: git's atomic ref-update semantics (POSIX `rename(2)`)
/// ensure that the loser gets a non-zero exit status.
///
/// # Caveats
///
/// - This is a **best-effort** cooperative lock.  It is not a kernel-level
///   exclusive lock.  Processes that crash without running `Drop` will leave a
///   stale ref behind.  Clean up with `git update-ref -d refs/locks/<name>`.
/// - The lock is scoped to one git repository.  Cross-host locking requires a
///   shared filesystem or a remote-tracking ref.
pub struct GitLock {
    ref_name: String,
    repo_path: PathBuf,
}

impl GitLock {
    /// Attempt to acquire the distributed lock named `lock_name` in `repo`.
    ///
    /// Internally writes `refs/locks/<lock_name>` pointing at `HEAD`.
    /// Returns `Ok(GitLock)` on success, or `Err` if the ref already exists
    /// (lock is held) or if git is unavailable.
    ///
    /// # Errors
    ///
    /// - `Error::Other` if the lock ref already exists (another holder).
    /// - `Error::Other` if `git update-ref` fails for any other reason.
    pub fn acquire(repo: &Path, lock_name: &str) -> Result<Self> {
        let ref_name = format!("refs/locks/{lock_name}");

        // Resolve HEAD to a commit SHA so we have a value to store.
        let head_sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_owned())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                // Bare / empty repo: use a placeholder SHA.
                "0000000000000000000000000000000000000000".to_owned()
            });

        // `git update-ref <ref> <new-value> <expected-value>` is an atomic CAS.
        // The all-zeros OID is git's sentinel meaning "ref must not exist".
        let result = Command::new("git")
            .args([
                "update-ref",
                "--create-reflog",
                &ref_name,
                &head_sha,
                "0000000000000000000000000000000000000000",
            ])
            .current_dir(repo)
            .output()
            .map_err(|e| Error::msg(format!("git subprocess error: {e}")))?;

        if result.status.success() {
            Ok(Self {
                ref_name,
                repo_path: repo.to_path_buf(),
            })
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
            Err(Error::msg(format!(
                "lock '{lock_name}' already held or git error: {stderr}"
            )))
        }
    }

    /// The git ref name that backs this lock (e.g. `refs/locks/ci/integration-db`).
    pub fn ref_name(&self) -> &str {
        &self.ref_name
    }
}

impl Drop for GitLock {
    /// Release the lock by deleting the git ref.
    ///
    /// Errors are swallowed because `Drop` cannot return a `Result`.
    /// If deletion fails (e.g. git is unavailable), the ref remains and must
    /// be cleaned up manually.
    fn drop(&mut self) {
        let _ = Command::new("git")
            .args(["update-ref", "-d", &self.ref_name])
            .current_dir(&self.repo_path)
            .output();
    }
}

// ---------------------------------------------------------------------------
// AuditEntry
// ---------------------------------------------------------------------------

/// A single entry in the [`GitAuditLedger`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unix timestamp (seconds since epoch) when the entry was appended.
    pub timestamp: u64,
    /// Dot-separated event type, e.g. `"test.completed"` or `"build.failed"`.
    pub event_type: String,
    /// BLAKE3 hex digest of the canonical JSON payload bytes.
    pub payload_hash: String,
    /// The original payload, deserialized.
    pub payload: serde_json::Value,
}

// ---------------------------------------------------------------------------
// GitAuditLedger
// ---------------------------------------------------------------------------

/// Append-only NDJSON audit ledger backed by `git notes`.
///
/// Entries are appended to the git note attached to the current `HEAD` commit
/// under the ref namespace `refs/notes/<ref_name>` (default:
/// `refs/notes/praxis/audit`).
///
/// Because git note objects are content-addressed and immutable, previously
/// appended lines cannot be silently altered — any modification would change
/// the note object hash and be detectable.
///
/// # Format
///
/// Each entry is one JSON line (NDJSON):
/// ```json
/// {"timestamp":1700000000,"event_type":"test.completed","payload_hash":"abc…","payload":{…}}
/// ```
///
/// # Reading back
///
/// [`GitAuditLedger::read_all`] returns all entries for `HEAD` in append order.
pub struct GitAuditLedger {
    repo_path: PathBuf,
    /// The notes ref namespace, e.g. `"praxis/audit"`.
    ref_name: String,
}

impl GitAuditLedger {
    /// Create a ledger targeting `repo` with the default ref namespace
    /// `"praxis/audit"`.
    pub fn new(repo: &Path) -> Self {
        Self::with_ref(repo, "praxis/audit")
    }

    /// Create a ledger targeting `repo` with a custom ref namespace.
    ///
    /// The full git notes ref will be `refs/notes/<ref_name>`.
    pub fn with_ref(repo: &Path, ref_name: &str) -> Self {
        Self {
            repo_path: repo.to_path_buf(),
            ref_name: ref_name.to_owned(),
        }
    }

    /// Append an audit entry to the note on `HEAD`.
    ///
    /// The entry is serialized as a single JSON line and appended via
    /// `git notes --ref=<ref_name> append -m <line> HEAD`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `serde_json` serialization fails or if `git notes`
    /// returns a non-zero exit status.
    pub fn append(&self, event_type: &str, payload: &serde_json::Value) -> Result<()> {
        let canonical = serde_json::to_vec(payload)
            .map_err(|e| Error::msg(format!("serialize payload: {e}")))?;

        let entry = AuditEntry {
            timestamp: epoch_secs(),
            event_type: event_type.to_owned(),
            payload_hash: blake3_hex(&canonical),
            payload: payload.clone(),
        };

        let line = serde_json::to_string(&entry)
            .map_err(|e| Error::msg(format!("serialize audit entry: {e}")))?;

        // `git notes append` adds a blank line between successive calls; we use
        // `--allow-empty` and pipe a single NDJSON line via `-m`.
        let result = Command::new("git")
            .args([
                "notes",
                &format!("--ref={}", self.ref_name),
                "append",
                "--allow-empty",
                "-m",
                &line,
                "HEAD",
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| Error::msg(format!("git subprocess error: {e}")))?;

        if result.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
            Err(Error::msg(format!("git notes append failed: {stderr}")))
        }
    }

    /// Read all audit entries from the note on `HEAD`.
    ///
    /// Lines that cannot be parsed as [`AuditEntry`] JSON are silently skipped
    /// (git adds blank separator lines between `notes append` calls; those are
    /// filtered out here).
    ///
    /// Returns an empty `Vec` if no note exists for `HEAD` yet.
    pub fn read_all(&self) -> Result<Vec<AuditEntry>> {
        // `git notes show` exits with code 1 when no note exists — treat that
        // as an empty ledger rather than an error.
        let output = Command::new("git")
            .args([
                "notes",
                &format!("--ref={}", self.ref_name),
                "show",
                "HEAD",
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| Error::msg(format!("git subprocess error: {e}")))?;

        if !output.status.success() {
            // No note on HEAD — ledger is empty.
            return Ok(Vec::new());
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let entries = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<AuditEntry>(l).ok())
            .collect();

        Ok(entries)
    }

    /// The ref namespace this ledger writes to (e.g. `"praxis/audit"`).
    pub fn ref_name(&self) -> &str {
        &self.ref_name
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    /// Create a minimal git repository with one commit so that `HEAD` resolves.
    fn init_test_repo() -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        let p = dir.path();

        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(p)
            .output()
            .expect("git init");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(p)
            .output()
            .expect("git config email");

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(p)
            .output()
            .expect("git config name");

        // Create an initial commit so HEAD exists.
        std::fs::write(p.join("README"), "init").expect("write README");
        Command::new("git")
            .args(["add", "README"])
            .current_dir(p)
            .output()
            .expect("git add");

        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(p)
            .output()
            .expect("git commit");

        dir
    }

    #[test]
    fn git_lock_acquire_and_release() {
        let dir = init_test_repo();
        let repo = dir.path();

        let lock = GitLock::acquire(repo, "test/my-lock").expect("acquire lock");
        assert_eq!(lock.ref_name(), "refs/locks/test/my-lock");

        // Verify the ref exists while the lock is held.
        let output = Command::new("git")
            .args(["rev-parse", "--verify", "refs/locks/test/my-lock"])
            .current_dir(repo)
            .output()
            .expect("git rev-parse");
        assert!(output.status.success(), "lock ref should exist while held");

        // Release.
        drop(lock);

        // Verify the ref is gone.
        let output = Command::new("git")
            .args(["rev-parse", "--verify", "refs/locks/test/my-lock"])
            .current_dir(repo)
            .output()
            .expect("git rev-parse");
        assert!(
            !output.status.success(),
            "lock ref should be deleted after drop"
        );
    }

    #[test]
    fn git_lock_double_acquire_fails() {
        let dir = init_test_repo();
        let repo = dir.path();

        let _lock = GitLock::acquire(repo, "test/exclusive").expect("first acquire");
        let second = GitLock::acquire(repo, "test/exclusive");
        assert!(
            second.is_err(),
            "second acquire of the same lock must fail"
        );
    }

    #[test]
    fn audit_ledger_append_and_read() {
        let dir = init_test_repo();
        let repo = dir.path();

        let ledger = GitAuditLedger::new(repo);

        // Empty ledger returns no entries.
        let entries = ledger.read_all().expect("read empty ledger");
        assert!(entries.is_empty());

        // Append one entry.
        let payload = serde_json::json!({ "suite": "unit", "passed": true });
        ledger.append("test.completed", &payload).expect("append");

        // Read it back.
        let entries = ledger.read_all().expect("read after append");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].event_type, "test.completed");
        assert_eq!(entries[0].payload["passed"], true);
        assert_eq!(entries[0].payload_hash.len(), 64, "BLAKE3 hex is 64 chars");
    }

    #[test]
    fn audit_ledger_append_multiple_entries() {
        let dir = init_test_repo();
        let repo = dir.path();

        let ledger = GitAuditLedger::new(repo);

        let p1 = serde_json::json!({ "step": 1 });
        let p2 = serde_json::json!({ "step": 2 });
        ledger.append("step.started", &p1).expect("append 1");
        ledger.append("step.finished", &p2).expect("append 2");

        let entries = ledger.read_all().expect("read all");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].event_type, "step.started");
        assert_eq!(entries[1].event_type, "step.finished");
    }

    #[test]
    fn audit_entry_payload_hash_is_deterministic() {
        let dir = init_test_repo();
        let repo = dir.path();

        let ledger = GitAuditLedger::new(repo);
        let payload = serde_json::json!({ "key": "value" });
        ledger.append("hash.check", &payload).expect("append");

        let entries = ledger.read_all().expect("read");
        let stored_hash = &entries[0].payload_hash;

        // Recompute manually.
        let canonical = serde_json::to_vec(&payload).expect("serialize");
        let expected = blake3_hex(&canonical);

        assert_eq!(stored_hash, &expected, "stored hash must match manual BLAKE3");
    }

    #[test]
    fn audit_ledger_custom_ref() {
        let dir = init_test_repo();
        let repo = dir.path();

        let ledger = GitAuditLedger::with_ref(repo, "myproject/events");
        assert_eq!(ledger.ref_name(), "myproject/events");

        let payload = serde_json::json!({ "x": 42 });
        ledger.append("custom.event", &payload).expect("append");

        let entries = ledger.read_all().expect("read");
        assert_eq!(entries.len(), 1);
    }
}
