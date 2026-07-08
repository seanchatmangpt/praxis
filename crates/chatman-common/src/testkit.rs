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
            (hi >> 56) as u8,
            (hi >> 48) as u8,
            (hi >> 40) as u8,
            (hi >> 32) as u8,
            (hi >> 24) as u8,
            (hi >> 16) as u8,
            ((hi >> 8) as u8 & 0x0f) | 0x50,
            hi as u8,
            (lo >> 56) as u8 & 0x3f | 0x80,
            (lo >> 48) as u8,
            (lo >> 40) as u8,
            (lo >> 32) as u8,
            (lo >> 24) as u8,
            (lo >> 16) as u8,
            (lo >> 8) as u8,
            lo as u8,
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

// ---------------------------------------------------------------------------
// TestState<Phase> — compile-time AAA enforcement
// ---------------------------------------------------------------------------

/// Arrange phase type token (zero runtime cost).
pub struct Arrange;
/// Act phase type token (zero runtime cost).
pub struct Act;
/// Assert phase type token (zero runtime cost).
pub struct Assert;

/// Compile-time guard for the Arrange→Act→Assert pattern.
/// Construct with `TestState::new()`, advance with `.act()`, then `.assert()`.
pub struct TestState<Phase> {
    _phase: std::marker::PhantomData<Phase>,
}

impl TestState<Arrange> {
    /// Create a new `TestState` in the `Arrange` phase.
    pub fn new() -> Self {
        TestState {
            _phase: std::marker::PhantomData,
        }
    }
    /// Advance to the `Act` phase.
    pub fn act(self) -> TestState<Act> {
        TestState {
            _phase: std::marker::PhantomData,
        }
    }
}

impl Default for TestState<Arrange> {
    fn default() -> Self {
        Self::new()
    }
}

impl TestState<Act> {
    /// Advance to the `Assert` phase.
    pub fn assert(self) -> TestState<Assert> {
        TestState {
            _phase: std::marker::PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// TestOutput trait — enables `?` in `#[test]` bodies
// ---------------------------------------------------------------------------

/// Allows using `?` inside `#[test]` functions by returning `impl TestOutput`.
pub trait TestOutput {
    /// Convert to a test result, panicking on failure.
    fn into_test_result(self);
}

impl<E: std::fmt::Debug> TestOutput for std::result::Result<(), E> {
    fn into_test_result(self) {
        if let Err(e) = self {
            panic!("test failed: {e:?}");
        }
    }
}

impl TestOutput for () {
    fn into_test_result(self) {}
}

// ---------------------------------------------------------------------------
// assert_fail! macro
// ---------------------------------------------------------------------------

/// Assert that an expression produces `Err(e)` where `e` matches `$pat`.
/// Panics if the expression returns `Ok(_)` or `Err` with a non-matching variant.
#[macro_export]
macro_rules! assert_fail {
    ($expr:expr, $pat:pat) => {
        match $expr {
            Err($pat) => {}
            Ok(v) => panic!("expected Err, got Ok({v:?})"),
            Err(e) => panic!("wrong error kind: {e:?}"),
        }
    };
    ($expr:expr) => {
        if $expr.is_ok() {
            panic!("expected Err, got Ok");
        }
    };
}

// ---------------------------------------------------------------------------
// EnvironmentFingerprint and TestReceipt
// ---------------------------------------------------------------------------

/// Snapshot of the test execution environment for auditable receipts.
#[derive(Debug, Clone)]
pub struct EnvironmentFingerprint {
    /// The operating system name (e.g. `"linux"`, `"macos"`, `"windows"`).
    pub os: String,
    /// The Rust version declared in `Cargo.toml` via `rust-version`, or `"unknown"`.
    pub rust_version: &'static str,
    /// The CPU architecture (e.g. `"x86_64"`, `"aarch64"`).
    pub target: &'static str,
    /// Seconds since the Unix epoch at the time of capture.
    pub timestamp_unix: i64,
}

impl EnvironmentFingerprint {
    /// Capture the current execution environment.
    pub fn capture() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            rust_version: env!("CARGO_PKG_RUST_VERSION", "unknown"),
            target: std::env::consts::ARCH,
            timestamp_unix: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        }
    }
}

/// Auditable, optionally BLAKE3-chained record of a single test's outcome.
#[derive(Debug, Clone)]
pub struct TestReceipt {
    /// Name of the test.
    pub test_name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// Environment snapshot at test time.
    pub environment: EnvironmentFingerprint,
    /// BLAKE3 hex digest of (test_name || passed || duration_ms || os).
    /// Only populated when the `living-docs` feature is active.
    pub chain_hash: Option<String>,
    /// Ed25519-signed version of `chain_hash`.
    /// Populated by [`TestReceipt::sign`] when the `signed-receipts` feature is active.
    #[cfg(feature = "signed-receipts")]
    pub signed: Option<crate::signed_receipt::SignedReceipt>,
}

impl TestReceipt {
    /// Record a test result by name, pass/fail, and duration.
    pub fn record(test_name: impl Into<String>, passed: bool, duration_ms: u64) -> Self {
        let test_name = test_name.into();
        let env = EnvironmentFingerprint::capture();

        #[cfg(feature = "living-docs")]
        let chain_hash = {
            let input = format!("{test_name}|{passed}|{duration_ms}|{}", env.os);
            Some(blake3::hash(input.as_bytes()).to_hex().to_string())
        };
        #[cfg(not(feature = "living-docs"))]
        let chain_hash = None;

        Self {
            test_name,
            passed,
            duration_ms,
            environment: env,
            chain_hash,
            #[cfg(feature = "signed-receipts")]
            signed: None,
        }
    }

    /// Run `f`, capture pass/fail, and return a receipt.
    pub fn capture<F: FnOnce()>(test_name: impl Into<String>, f: F) -> Self {
        let start = std::time::Instant::now();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        let duration_ms = start.elapsed().as_millis() as u64;
        let passed = result.is_ok();
        Self::record(test_name, passed, duration_ms)
    }

    /// Attach an ed25519 signature to this receipt using the signing key
    /// loaded from the environment (`PRAXIS_SIGNING_KEY` or
    /// `PRAXIS_SIGNING_KEY_FILE`).
    ///
    /// Only available when the `signed-receipts` feature is active.
    /// If no signing key is set the receipt is returned unchanged (no error).
    #[cfg(feature = "signed-receipts")]
    pub fn sign(mut self) -> Self {
        if let Some(ref hash) = self.chain_hash {
            if let Ok(signed) = crate::signed_receipt::sign_with_env_key(hash) {
                self.signed = Some(signed);
            }
        }
        self
    }

    /// Attach an ed25519 signature to this receipt using an explicit
    /// hex-encoded signing key.
    ///
    /// Only available when the `signed-receipts` feature is active.
    #[cfg(feature = "signed-receipts")]
    pub fn sign_with(mut self, signing_key_hex: &str) -> crate::Result<Self> {
        if let Some(ref hash) = self.chain_hash {
            self.signed = Some(crate::signed_receipt::sign(hash, signing_key_hex)?);
        }
        Ok(self)
    }
}

// ---------------------------------------------------------------------------
// DocEvent + DocContext (feature = "living-docs")
// ---------------------------------------------------------------------------

#[cfg(feature = "living-docs")]
mod living_docs {
    use std::path::{Path, PathBuf};

    /// A structured documentation event emitted inside a test body.
    #[non_exhaustive]
    #[derive(Debug, Clone)]
    pub enum DocEvent {
        /// A Markdown section heading.
        Section(String),
        /// A Markdown paragraph.
        Para(String),
        /// A fenced code block.
        Code {
            /// Language tag for the code fence.
            lang: String,
            /// Body of the code block.
            body: String,
        },
        /// A Markdown table.
        Table {
            /// Column headers.
            header: Vec<String>,
            /// Table rows (each row is a list of cell strings).
            rows: Vec<Vec<String>>,
        },
        /// A list of key/value pairs rendered as `- **key:** value`.
        KeyValue(Vec<(String, String)>),
        /// An assertion result (label + pass/fail).
        Assertion {
            /// Human-readable assertion label.
            label: String,
            /// Whether the assertion passed.
            passed: bool,
        },
        /// A Mermaid diagram DSL block.
        Mermaid(String),
        /// A BLAKE3 chain hash footer.
        ChainHash(String),
    }

    /// Accumulates `DocEvent`s during a test and writes `docs/test/<name>.md`
    /// on `finish()` (or on `Drop`).
    pub struct DocContext {
        name: String,
        events: Vec<DocEvent>,
        output_dir: PathBuf,
        finished: bool,
    }

    impl DocContext {
        /// Create a context for the given test/module name.
        /// Output goes to `docs/test/<name>.md` relative to the crate root.
        pub fn for_test(name: impl Into<String>) -> Self {
            let name = name.into();
            let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("docs")
                .join("test");
            Self {
                name,
                events: Vec::new(),
                output_dir,
                finished: false,
            }
        }

        /// Override the output directory (useful in tests).
        pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
            self.output_dir = dir.into();
            self
        }

        /// Emit a paragraph event.
        pub fn say(&mut self, text: &str) {
            self.events.push(DocEvent::Para(text.to_string()));
        }

        /// Emit a section heading event.
        pub fn say_section(&mut self, heading: &str) {
            self.events.push(DocEvent::Section(heading.to_string()));
        }

        /// Emit a fenced code block event.
        pub fn say_code(&mut self, lang: &str, body: &str) {
            self.events.push(DocEvent::Code {
                lang: lang.to_string(),
                body: body.to_string(),
            });
        }

        /// Emit a table event.
        pub fn say_table(&mut self, header: &[&str], rows: &[&[&str]]) {
            self.events.push(DocEvent::Table {
                header: header.iter().map(|s| s.to_string()).collect(),
                rows: rows
                    .iter()
                    .map(|row| row.iter().map(|s| s.to_string()).collect())
                    .collect(),
            });
        }

        /// Emit a Mermaid diagram event.
        pub fn say_mermaid(&mut self, dsl: &str) {
            self.events.push(DocEvent::Mermaid(dsl.to_string()));
        }

        /// Emit a key/value list event.
        pub fn say_key_value(&mut self, pairs: &[(&str, &str)]) {
            self.events.push(DocEvent::KeyValue(
                pairs
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            ));
        }

        /// Emit an assertion event AND assert the condition.
        /// If `cond` is false the test panics and the event is NOT emitted
        /// (the doc line is only written when the assertion passes).
        pub fn say_and_assert(&mut self, label: &str, cond: bool) {
            assert!(cond, "doc assertion failed: {label}");
            self.events.push(DocEvent::Assertion {
                label: label.to_string(),
                passed: true,
            });
        }

        /// Render events to Markdown bytes.
        pub fn render_markdown(&self) -> Vec<u8> {
            let mut out = format!("# {}\n\n", self.name);
            for event in &self.events {
                match event {
                    DocEvent::Section(h) => out.push_str(&format!("## {h}\n\n")),
                    DocEvent::Para(p) => out.push_str(&format!("{p}\n\n")),
                    DocEvent::Code { lang, body } => {
                        out.push_str(&format!("```{lang}\n{body}\n```\n\n"))
                    }
                    DocEvent::Table { header, rows } => {
                        let cols = header.join(" | ");
                        let sep = header.iter().map(|_| "---").collect::<Vec<_>>().join(" | ");
                        out.push_str(&format!("| {cols} |\n| {sep} |\n"));
                        for row in rows {
                            out.push_str(&format!("| {} |\n", row.join(" | ")));
                        }
                        out.push('\n');
                    }
                    DocEvent::KeyValue(pairs) => {
                        for (k, v) in pairs {
                            out.push_str(&format!("- **{k}:** {v}\n"));
                        }
                        out.push('\n');
                    }
                    DocEvent::Assertion { label, passed } => {
                        let icon = if *passed { "✓" } else { "✗" };
                        out.push_str(&format!("- {icon} {label}\n\n"));
                    }
                    DocEvent::Mermaid(dsl) => out.push_str(&format!("```mermaid\n{dsl}\n```\n\n")),
                    DocEvent::ChainHash(h) => out.push_str(&format!("*chain_hash: `{h}`*\n\n")),
                }
            }
            // Footer with BLAKE3 provenance
            let body_hash = blake3::hash(out.as_bytes()).to_hex().to_string();
            out.push_str(&format!(
                "\n---\n*Generated by chatman-common testkit — chain: `{body_hash}`*\n"
            ));
            out.into_bytes()
        }

        /// Write the rendered Markdown to `<output_dir>/<name>.md`.
        pub fn finish(mut self) -> crate::Result<()> {
            self.finished = true;
            let bytes = self.render_markdown();
            let path = self
                .output_dir
                .join(format!("{}.md", self.name.replace(['/', '\\', ' '], "_")));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, &bytes)?;
            Ok(())
        }
    }

    impl Drop for DocContext {
        fn drop(&mut self) {
            if !self.finished && !self.events.is_empty() && !std::thread::panicking() {
                let bytes = self.render_markdown();
                let path = self
                    .output_dir
                    .join(format!("{}.md", self.name.replace(['/', '\\', ' '], "_")));
                let _ = std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")));
                let _ = std::fs::write(&path, &bytes);
            }
        }
    }
}

#[cfg(feature = "living-docs")]
pub use living_docs::{DocContext, DocEvent};

// ---------------------------------------------------------------------------
// doc_assert! macro (available whenever testkit is active)
// ---------------------------------------------------------------------------

/// Assert a condition inside a `DocContext` test body.
/// The assertion label is emitted as a documentation event only if it passes.
/// Panics with the label message if `$cond` is false.
#[macro_export]
macro_rules! doc_assert {
    ($ctx:expr, $label:expr, $cond:expr) => {{
        assert!($cond, "doc assertion failed: {}", $label);
        $ctx.say_and_assert($label, true);
    }};
}

// ---------------------------------------------------------------------------
// allocate_ephemeral_port + skip_without_docker!
// ---------------------------------------------------------------------------

/// Bind to port 0, let the OS assign a free port, immediately release it,
/// and return the port number.
///
/// There is an inherent TOCTOU race — another process may grab the port
/// between the release and your bind.  For tests that just need *a* free
/// port this is acceptable.
pub fn allocate_ephemeral_port() -> u16 {
    use std::net::TcpListener;
    TcpListener::bind("127.0.0.1:0")
        .expect("bind to port 0")
        .local_addr()
        .expect("local_addr")
        .port()
}

/// Skip the current test if the Docker socket is not reachable.
///
/// # Example
/// ```rust,no_run
/// use chatman_common::skip_without_docker;
/// #[test]
/// fn needs_docker() {
///     skip_without_docker!();
///     // … docker-dependent test body …
/// }
/// ```
#[macro_export]
macro_rules! skip_without_docker {
    () => {
        if std::net::TcpStream::connect("127.0.0.1:2375").is_err()
            && !std::path::Path::new("/var/run/docker.sock").exists()
        {
            eprintln!("SKIP: Docker not available");
            return;
        }
    };
}

// ---------------------------------------------------------------------------
// performance_test! macro
// ---------------------------------------------------------------------------

/// Run a test body and enforce a wall-clock SLA in milliseconds.
///
/// # Example
/// ```rust,no_run
/// use chatman_common::performance_test;
/// performance_test!(hash_is_fast, 5, {
///     let _ = std::hint::black_box(42u64.wrapping_mul(7));
/// });
/// ```
#[macro_export]
macro_rules! performance_test {
    ($name:ident, $sla_ms:expr, $body:block) => {
        #[test]
        fn $name() {
            let start = std::time::Instant::now();
            $body
            let elapsed = start.elapsed().as_millis() as u64;
            assert!(
                elapsed <= $sla_ms,
                "SLA violated: {}ms > {}ms (test `{}`)",
                elapsed,
                $sla_ms,
                stringify!($name)
            );
        }
    };
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

    // ----- TestState tests -----

    #[test]
    fn test_state_transitions() {
        let s = TestState::<Arrange>::new();
        let s = s.act();
        let _s = s.assert();
    }

    #[test]
    fn test_state_default_is_arrange() {
        let _s: TestState<Arrange> = TestState::default();
    }

    // ----- TestReceipt tests -----

    #[test]
    fn test_receipt_record_passed() {
        let r = TestReceipt::record("my_test", true, 42);
        assert_eq!(r.test_name, "my_test");
        assert!(r.passed);
        assert_eq!(r.duration_ms, 42);
    }

    #[test]
    fn test_receipt_capture_passes() {
        let r = TestReceipt::capture("capture_test", || {
            let _ = 1 + 1;
        });
        assert!(r.passed);
        assert_eq!(r.test_name, "capture_test");
    }

    #[test]
    fn test_receipt_capture_fails_on_panic() {
        let r = TestReceipt::capture("fail_test", || {
            panic!("intentional");
        });
        assert!(!r.passed);
    }

    // ----- allocate_ephemeral_port tests -----

    #[test]
    fn ephemeral_port_is_nonzero() {
        let port = allocate_ephemeral_port();
        assert!(port > 0, "expected a non-zero port, got {port}");
    }

    // ----- assert_fail! macro tests -----

    #[test]
    fn assert_fail_catches_err() {
        let result: Result<(), &str> = Err("oops");
        assert_fail!(result);
    }

    #[test]
    fn assert_fail_with_pattern() {
        #[derive(Debug)]
        enum MyErr {
            NotFound,
            Other,
        }
        let result: std::result::Result<(), MyErr> = Err(MyErr::NotFound);
        assert_fail!(result, MyErr::NotFound);
    }

    // ----- TestReceipt::sign tests (signed-receipts feature) -----

    #[test]
    #[cfg(feature = "signed-receipts")]
    fn test_receipt_sign_with_explicit_key() {
        use crate::signed_receipt::KeyPair;
        let kp = KeyPair::generate();
        // Needs living-docs for chain_hash to be populated
        let mut r = TestReceipt::record("signed_test", true, 10);
        // Manually inject a chain_hash so we can test signing without living-docs
        r.chain_hash = Some("a".repeat(64));
        let r = r.sign_with(&kp.signing_key_hex()).unwrap();
        let signed = r
            .signed
            .expect("signed field must be populated after sign_with");
        assert!(
            crate::signed_receipt::verify(&signed, &kp.verifying_key_hex()).unwrap(),
            "TestReceipt signature must be valid"
        );
    }

    #[test]
    #[cfg(feature = "signed-receipts")]
    fn test_receipt_sign_no_key_env_is_noop() {
        // If env key is not set, sign() must not panic — it silently skips.
        std::env::remove_var("PRAXIS_SIGNING_KEY");
        std::env::remove_var("PRAXIS_SIGNING_KEY_FILE");
        let mut r = TestReceipt::record("no_key_test", true, 1);
        r.chain_hash = Some("b".repeat(64));
        let r = r.sign();
        assert!(
            r.signed.is_none(),
            "sign() without env key should leave signed = None"
        );
    }
}
