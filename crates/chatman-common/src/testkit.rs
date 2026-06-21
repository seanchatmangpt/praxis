//! Test helpers: snapshot assertions, golden files, TempReceipt builder,
//! deterministic UUIDs, and tempfile re-export.

pub use tempfile;

use std::path::{Path, PathBuf};

use crate::Result;

// ---------------------------------------------------------------------------
// Golden-file assertions
// ---------------------------------------------------------------------------

/// Assert that `actual` matches the bytes stored at `path`.
///
/// If the environment variable `UPDATE_GOLDEN` is set to `"1"` the file is
/// (re-)written with the new bytes and the assertion is skipped.
pub fn assert_golden(actual: &[u8], path: &Path) -> Result<()> {
    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, actual)?;
        return Ok(());
    }

    let expected = std::fs::read(path)?;
    if actual != expected.as_slice() {
        return Err(crate::Error::msg(format!(
            "golden mismatch at {}: actual {} bytes, expected {} bytes\n\
             Re-run with UPDATE_GOLDEN=1 to accept the new output.",
            path.display(),
            actual.len(),
            expected.len(),
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Snapshot assertions (insta-compatible)
// ---------------------------------------------------------------------------

/// Assert `actual` matches a named snapshot stored in `snapshots/<name>.snap`
/// relative to the calling test's source file.
///
/// When `UPDATE_SNAPSHOTS=1` is set the snapshot is created/updated and the
/// assertion passes.  This mirrors the `insta` update workflow without
/// requiring the `insta` crate as a hard dependency.
///
/// # Panics
/// Panics (with a diff-style message) when the snapshot does not match.
pub fn assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) {
    let path = snapshots_dir.join(format!("{name}.snap"));

    if std::env::var("UPDATE_SNAPSHOTS").as_deref() == Ok("1") {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create snapshots dir");
        }
        std::fs::write(&path, actual).expect("write snapshot");
        return;
    }

    if !path.exists() {
        panic!(
            "snapshot `{name}` not found at {}.\n\
             Run with UPDATE_SNAPSHOTS=1 to create it.",
            path.display()
        );
    }

    let expected = std::fs::read_to_string(&path).expect("read snapshot");
    if actual != expected {
        panic!(
            "snapshot `{name}` mismatch.\n\
             --- expected ---\n{expected}\n\
             +++ actual ---\n{actual}\n\
             Run with UPDATE_SNAPSHOTS=1 to accept the new output."
        );
    }
}

// ---------------------------------------------------------------------------
// Deterministic UUID
// ---------------------------------------------------------------------------

/// Generate a deterministic, name-based UUID v5 from `seed` using the DNS
/// namespace OID as the namespace.
///
/// Produces the same output for the same `seed` on every call, every platform,
/// every binary version — suitable for fixtures and snapshot IDs.
///
/// The implementation uses BLAKE3 truncated to 128 bits and formatted per RFC
/// 4122 §4.3 (version=5, variant=0b10xxxxxx).
pub fn deterministic_uuid(seed: &str) -> String {
    // Use blake3 (already a dep via "provenance") to hash the seed.
    // We truncate to 16 bytes and stamp the UUID version/variant bits.
    #[cfg(feature = "provenance")]
    {
        let hash = blake3::hash(seed.as_bytes());
        let bytes = hash.as_bytes();
        let mut b = [0u8; 16];
        b.copy_from_slice(&bytes[..16]);
        // Version 5 (name-based SHA-1 in RFC 4122, but we use blake3 here)
        b[6] = (b[6] & 0x0f) | 0x50;
        // Variant 0b10xxxxxx
        b[8] = (b[8] & 0x3f) | 0x80;
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3],
            b[4], b[5],
            b[6], b[7],
            b[8], b[9],
            b[10], b[11], b[12], b[13], b[14], b[15],
        )
    }
    #[cfg(not(feature = "provenance"))]
    {
        // Fallback: simple FNV-1a over bytes, formatted as UUID-shaped hex.
        let mut h: u64 = 0xcbf29ce484222325;
        for byte in seed.as_bytes() {
            h ^= *byte as u64;
            h = h.wrapping_mul(0x100000000b3);
        }
        let lo = h;
        let hi = !h;
        let b = [
            (hi >> 56) as u8, (hi >> 48) as u8, (hi >> 40) as u8, (hi >> 32) as u8,
            (hi >> 24) as u8, (hi >> 16) as u8,
            ((hi >> 8) as u8 & 0x0f) | 0x50, hi as u8,
            (lo >> 56) as u8 & 0x3f | 0x80, (lo >> 48) as u8,
            (lo >> 40) as u8, (lo >> 32) as u8, (lo >> 24) as u8, (lo >> 16) as u8,
            (lo >> 8) as u8, lo as u8,
        ];
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3],
            b[4], b[5],
            b[6], b[7],
            b[8], b[9],
            b[10], b[11], b[12], b[13], b[14], b[15],
        )
    }
}

// ---------------------------------------------------------------------------
// TempReceipt builder
// ---------------------------------------------------------------------------

/// A temporary directory containing a minimal, well-formed receipt JSON file.
///
/// The directory is deleted when this value is dropped.
///
/// # Example
/// ```rust,no_run
/// use chatman_common::testkit::TempReceipt;
///
/// let tr = TempReceipt::builder()
///     .format_version("core/v1")
///     .chain_hash("aabbcc")
///     .build()
///     .unwrap();
///
/// let path = tr.path();
/// // pass `path` to CLI under test
/// ```
pub struct TempReceipt {
    dir: tempfile::TempDir,
    filename: String,
}

impl TempReceipt {
    /// Return a [`TempReceiptBuilder`] with sane defaults.
    pub fn builder() -> TempReceiptBuilder {
        TempReceiptBuilder::default()
    }

    /// Path to the receipt JSON file.
    pub fn path(&self) -> PathBuf {
        self.dir.path().join(&self.filename)
    }

    /// Path to the temp directory root.
    pub fn dir(&self) -> &Path {
        self.dir.path()
    }
}

/// Builder for [`TempReceipt`].
#[derive(Debug, Clone)]
pub struct TempReceiptBuilder {
    format_version: String,
    chain_hash: String,
    events: Vec<serde_json::Value>,
    profile: String,
    filename: String,
}

impl Default for TempReceiptBuilder {
    fn default() -> Self {
        Self {
            format_version: "core/v1".to_string(),
            chain_hash: "0".repeat(64),
            events: Vec::new(),
            profile: "core/v1".to_string(),
            filename: "receipt.json".to_string(),
        }
    }
}

impl TempReceiptBuilder {
    /// Override the `format_version` field.
    pub fn format_version(mut self, v: impl Into<String>) -> Self {
        self.format_version = v.into();
        self
    }

    /// Override the `chain_hash` field.
    pub fn chain_hash(mut self, h: impl Into<String>) -> Self {
        self.chain_hash = h.into();
        self
    }

    /// Override the `profile` field.
    pub fn profile(mut self, p: impl Into<String>) -> Self {
        self.profile = p.into();
        self
    }

    /// Set the filename inside the temp directory.
    pub fn filename(mut self, f: impl Into<String>) -> Self {
        self.filename = f.into();
        self
    }

    /// Append a raw JSON event object.
    pub fn event(mut self, event: serde_json::Value) -> Self {
        self.events.push(event);
        self
    }

    /// Build the [`TempReceipt`], writing the JSON to a new temp directory.
    pub fn build(self) -> Result<TempReceipt> {
        let dir = tempfile::tempdir()?;
        let receipt = serde_json::json!({
            "format_version": self.format_version,
            "chain_hash": self.chain_hash,
            "profile": self.profile,
            "events": self.events,
        });
        let bytes = serde_json::to_vec_pretty(&receipt)
            .map_err(|e| crate::Error::msg(format!("TempReceipt serialize: {e}")))?;
        let path = dir.path().join(&self.filename);
        std::fs::write(&path, &bytes)?;
        Ok(TempReceipt {
            dir,
            filename: self.filename,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_uuid_stable() {
        let a = deterministic_uuid("hello");
        let b = deterministic_uuid("hello");
        assert_eq!(a, b, "same seed must produce same UUID");
    }

    #[test]
    fn deterministic_uuid_different_seeds() {
        assert_ne!(deterministic_uuid("foo"), deterministic_uuid("bar"));
    }

    #[test]
    fn deterministic_uuid_format() {
        let u = deterministic_uuid("test-seed-42");
        // xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let parts: Vec<&str> = u.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn temp_receipt_builds_and_exists() {
        let tr = TempReceipt::builder()
            .format_version("core/v1")
            .chain_hash("a".repeat(64))
            .build()
            .unwrap();
        assert!(tr.path().exists());
    }

    #[test]
    fn temp_receipt_contains_valid_json() {
        let tr = TempReceipt::builder()
            .event(serde_json::json!({"seq": 0, "event_type": "test"}))
            .build()
            .unwrap();
        let contents = std::fs::read_to_string(tr.path()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(v["format_version"], "core/v1");
        assert_eq!(v["events"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn assert_golden_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.bin");
        let data = b"hello golden";

        // Write via UPDATE_GOLDEN path
        std::env::set_var("UPDATE_GOLDEN", "1");
        assert_golden(data, &path).unwrap();
        std::env::remove_var("UPDATE_GOLDEN");

        // Read back
        assert_golden(data, &path).unwrap();
    }

    #[test]
    fn assert_snapshot_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let snaps = dir.path().join("snapshots");

        std::env::set_var("UPDATE_SNAPSHOTS", "1");
        assert_snapshot("my_snap", "hello\nworld\n", &snaps);
        std::env::remove_var("UPDATE_SNAPSHOTS");

        assert_snapshot("my_snap", "hello\nworld\n", &snaps);
    }
}
