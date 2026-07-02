//! v1 walking-skeleton integration test.
//!
//! Exercises the full flow end to end without any network access: `load_task` ->
//! `compile_task_prompt` -> mocked model call -> `stage_fixture` +
//! `apply_model_output` -> `run_pipeline_for_task` -> chained receipt append. The
//! [`rust_fable_testbed::model_client::MockModelClient`] returned by
//! `MockModelClient::ok_text` (already public in `model_client.rs`, not
//! `#[cfg(test)]`-gated) satisfies `ModelClient` and stands in for
//! `AnthropicClient`, so the non-`#[ignore]`d test below needs neither
//! `ANTHROPIC_API_KEY` nor network access.
//!
//! A second, `#[ignore]`d test runs the identical flow against a real
//! `AnthropicClient::from_env()` for manual verification; it skips (rather than
//! panics) when `ANTHROPIC_API_KEY` is unset.

use std::path::{Path, PathBuf};

use rust_fable_testbed::model_client::{Message, MessageRequest, MockModelClient, ModelClient};
use rust_fable_testbed::pipeline::run_pipeline_for_task;
use rust_fable_testbed::prompt::compile_task_prompt;
use rust_fable_testbed::receipt::{append_receipt, chain_receipt, genesis_chain_hash};
use rust_fable_testbed::sandbox::{apply_model_output, stage_fixture};
use rust_fable_testbed::spec::load_task;

/// A corrected `binary_search` that tracks the leftmost matching index (by
/// narrowing `hi` to `mid` on a match and continuing to search left) instead of
/// returning on the first probe that hits `target`. Passes all three fixture
/// tests, including `finds_leftmost_index_with_duplicate_keys`.
const CORRECTED_LIB_RS: &str = r#"//! Fixture for `function_bugfix_001`: corrected `binary_search`.
//!
//! Returns the *leftmost* matching index when `target` appears multiple times in
//! `arr`, by continuing to narrow left after a match instead of returning
//! immediately.

/// Search `arr` (must be sorted ascending) for `target`, returning the leftmost
/// matching index.
#[must_use]
pub fn binary_search(arr: &[i32], target: i32) -> Option<usize> {
    let mut lo = 0usize;
    let mut hi = arr.len();
    let mut found = None;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if arr[mid] == target {
            found = Some(mid);
            // Keep searching left: an earlier occurrence may still exist.
            hi = mid;
        } else if arr[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::binary_search;

    #[test]
    fn finds_unique_element() {
        let arr = [1, 3, 5, 7, 9, 11];
        assert_eq!(binary_search(&arr, 7), Some(3));
    }

    #[test]
    fn returns_none_when_absent() {
        let arr = [2, 4, 6, 8];
        assert_eq!(binary_search(&arr, 5), None);
    }

    #[test]
    fn finds_leftmost_index_with_duplicate_keys() {
        let arr = [1, 2, 3, 3, 3, 3, 3, 8, 9];
        assert_eq!(binary_search(&arr, 3), Some(2));
    }
}
"#;

/// A fixed, hardcoded model response containing one fenced ```rust code block
/// with [`CORRECTED_LIB_RS`] — mirrors what a real model's `text()` output looks
/// like (prose before/after the fence).
fn mock_model_response() -> String {
    format!("Here is the corrected `src/lib.rs`:\n\n```rust\n{CORRECTED_LIB_RS}\n```\n\nThis fixes the leftmost-index bug.")
}

/// Path to `tasks/function_bugfix_001.ttl`, resolved from the crate root so the
/// test works regardless of the process's current working directory.
fn task_ttl_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tasks/function_bugfix_001.ttl")
}

#[test]
fn walking_skeleton_runs_full_v1_flow_with_mocked_model() {
    let ttl_path = task_ttl_path();

    // 1. load_task + compile_task_prompt.
    let task = load_task(&ttl_path).expect("load_task should succeed for function_bugfix_001");
    let compiled = compile_task_prompt(&task).expect("compile_task_prompt should succeed");
    assert!(!compiled.content().is_empty(), "compiled prompt content should be non-empty");
    assert!(!compiled.hash().is_empty(), "compiled prompt should carry a hash");

    // 2. Invoke the mock client instead of a real AnthropicClient.
    let client = MockModelClient::ok_text(&mock_model_response());
    let request = MessageRequest {
        model: &task.model,
        max_tokens: 16_000,
        system: None,
        messages: vec![Message::user(compiled.content())],
        effort: None,
    };
    let response = client.send(&request).expect("mock client send should succeed");
    let model_output = response.text().expect("mock response should yield text");

    // 3. stage_fixture + apply_model_output with the mock's response.
    let base_dir = ttl_path.parent().expect("ttl path has a parent directory");
    let fixture_dir = base_dir.join(&task.fixture);
    let staged = stage_fixture(&fixture_dir).expect("stage_fixture should succeed");

    apply_model_output(staged.path(), Path::new("src/lib.rs"), &model_output)
        .expect("apply_model_output should find the fenced rust block and write it");

    // Original fixture must remain untouched by staging/apply (sandbox never
    // mutates the checked-in fixture).
    let original_lib_rs = std::fs::read_to_string(fixture_dir.join("src/lib.rs"))
        .expect("original fixture lib.rs should still be readable");
    assert!(
        original_lib_rs.contains("BUG:"),
        "original fixture should still contain the unfixed, buggy implementation"
    );

    // 4. run_pipeline (build/test/clippy/safety_audit).
    let metrics = run_pipeline_for_task(staged.path(), Some(task.task_type));
    println!("{}", metrics.summary_line());
    assert_eq!(
        metrics.failed_count, 0,
        "expected all pipeline stages to pass with the corrected implementation: {}",
        metrics.summary_line()
    );
    assert_eq!(metrics.stages.len(), 4, "expected the 4-stage build/test/clippy/safety_audit pipeline");

    // 5. Append a receipt to a ledger in a tempdir (never the real crate root).
    let ledger_dir = tempfile::tempdir().expect("ledger tempdir should be creatable");
    let ledger_path = ledger_dir.path().join("testbed_receipts.jsonl");

    let receipt = chain_receipt(&genesis_chain_hash(), &task.id, compiled.hash(), &task.model, &metrics)
        .expect("chain_receipt should succeed");
    append_receipt(&ledger_path, &receipt).expect("append_receipt should succeed");

    let ledger_content = std::fs::read_to_string(&ledger_path).expect("ledger should be readable back");
    let lines: Vec<&str> = ledger_content.lines().collect();
    assert_eq!(lines.len(), 1, "expected exactly one receipt line in the fresh ledger");

    let parsed: serde_json::Value =
        serde_json::from_str(lines[0]).expect("receipt line should be valid JSON");
    assert_eq!(parsed["task_id"], task.id);
    assert_eq!(parsed["prompt_hash"], compiled.hash());
    assert_eq!(parsed["prev_chain_hash"], genesis_chain_hash());
    assert_eq!(parsed["chain_hash"], receipt.chain_hash);
    assert!(
        parsed["metrics_summary"].as_str().unwrap_or_default().starts_with("4/4 passed"),
        "receipt metrics_summary should reflect a fully-passing run, got: {parsed}"
    );
}

/// A corrected `src/lib.rs` for `unsafe_audit_001`: replaces the unsound
/// `last_or_zero` (which read one element past the end of the slice via raw-pointer
/// arithmetic) with a safe `arr.last()`-based implementation, while leaving the two
/// already-sound, already-`SAFETY:`-documented functions (`sum_at_indices`,
/// `swap_first_last`) untouched -- exactly what an audit fix should do.
const CORRECTED_UNSAFE_AUDIT_LIB_RS: &str = r#"//! Fixture module with three raw-pointer blocks (see the paired task spec for the
//! full audit brief).
//!
//! Two of the three blocks are sound and already documented with a `SAFETY:`
//! justification comment. `last_or_zero` has been fixed to use safe slice indexing
//! instead of the unsound raw-pointer read that used to overrun the slice by one
//! element.

/// Return the last element of `arr` (or `0` for an empty slice).
///
/// # Fix
///
/// This previously read one element past the end of `arr`'s buffer via raw-pointer
/// arithmetic (`*ptr.add(arr.len())`), an out-of-bounds heap read (UB, Miri-detectable)
/// that also simply returned the wrong value. No raw-pointer trick is actually needed
/// here, so this now uses safe slice indexing via `arr.last()`.
#[must_use]
pub fn last_or_zero(arr: &[i32]) -> i32 {
    arr.last().copied().unwrap_or(0)
}

/// Sound: sum the elements of `arr` at each index in `indices`, using
/// `get_unchecked` for the actual reads only after validating that every index is
/// in-bounds.
#[must_use]
pub fn sum_at_indices(arr: &[i32], indices: &[usize]) -> i32 {
    for &i in indices {
        assert!(i < arr.len(), "index {i} out of bounds for slice of len {}", arr.len());
    }
    let mut total = 0i32;
    for &i in indices {
        // SAFETY: every index in `indices` was checked to be `< arr.len()` in the
        // loop above, so `arr.get_unchecked(i)` is always a valid, in-bounds read.
        total += unsafe { *arr.get_unchecked(i) };
    }
    total
}

/// Sound: swap the first and last elements of `arr` via raw-pointer writes, after
/// checking `arr.len() >= 2`.
pub fn swap_first_last(arr: &mut [i32]) {
    let len = arr.len();
    if len < 2 {
        return;
    }
    let ptr = arr.as_mut_ptr();
    // SAFETY: `len >= 2` checked above, so `ptr` and `ptr.add(len - 1)` are both
    // valid, in-bounds, non-overlapping pointers into `arr`, satisfying `ptr::swap`.
    unsafe {
        std::ptr::swap(ptr, ptr.add(len - 1));
    }
}

#[cfg(test)]
mod tests {
    use super::{last_or_zero, sum_at_indices, swap_first_last};

    #[test]
    fn last_or_zero_returns_the_actual_last_element() {
        let arr = [10, 20, 30];
        assert_eq!(last_or_zero(&arr), 30);
    }

    #[test]
    fn last_or_zero_returns_zero_for_empty_slice() {
        let arr: [i32; 0] = [];
        assert_eq!(last_or_zero(&arr), 0);
    }

    #[test]
    fn sum_at_indices_sums_selected_elements() {
        let arr = [10, 20, 30, 40];
        assert_eq!(sum_at_indices(&arr, &[0, 2, 3]), 80);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn sum_at_indices_panics_on_out_of_bounds_index() {
        let arr = [10, 20, 30];
        let _ = sum_at_indices(&arr, &[5]);
    }

    #[test]
    fn swap_first_last_swaps_endpoints() {
        let mut arr = [1, 2, 3, 4];
        swap_first_last(&mut arr);
        assert_eq!(arr, [4, 2, 3, 1]);
    }

    #[test]
    fn swap_first_last_is_noop_for_short_slices() {
        let mut arr = [1];
        swap_first_last(&mut arr);
        assert_eq!(arr, [1]);

        let mut empty: [i32; 0] = [];
        swap_first_last(&mut empty);
        assert_eq!(empty, []);
    }
}
"#;

fn mock_model_response_unsafe_audit() -> String {
    format!("Here is the corrected `src/lib.rs`:\n\n```rust\n{CORRECTED_UNSAFE_AUDIT_LIB_RS}\n```\n\nThis fixes the out-of-bounds read in `last_or_zero` and leaves the sound, documented `unsafe` blocks untouched.")
}

/// Path to `tasks/unsafe_audit_001.ttl`, resolved from the crate root.
fn unsafe_audit_task_ttl_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tasks/unsafe_audit_001.ttl")
}

#[test]
fn walking_skeleton_runs_unsafe_audit_001_flow_with_mocked_model() {
    let ttl_path = unsafe_audit_task_ttl_path();

    let task = load_task(&ttl_path).expect("load_task should succeed for unsafe_audit_001");
    assert_eq!(task.task_type, rust_fable_testbed::spec::TaskType::UnsafeAudit);
    let compiled = compile_task_prompt(&task).expect("compile_task_prompt should succeed");
    assert!(!compiled.content().is_empty(), "compiled prompt content should be non-empty");

    let client = MockModelClient::ok_text(&mock_model_response_unsafe_audit());
    let request = MessageRequest {
        model: &task.model,
        max_tokens: 16_000,
        system: None,
        messages: vec![Message::user(compiled.content())],
        effort: None,
    };
    let response = client.send(&request).expect("mock client send should succeed");
    let model_output = response.text().expect("mock response should yield text");

    let base_dir = ttl_path.parent().expect("ttl path has a parent directory");
    let fixture_dir = base_dir.join(&task.fixture);
    let staged = stage_fixture(&fixture_dir).expect("stage_fixture should succeed");

    apply_model_output(staged.path(), Path::new("src/lib.rs"), &model_output)
        .expect("apply_model_output should find the fenced rust block and write it");

    // Original fixture must remain untouched (still contains the unsound raw-pointer
    // read past the end of the slice).
    let original_lib_rs = std::fs::read_to_string(fixture_dir.join("src/lib.rs"))
        .expect("original fixture lib.rs should still be readable");
    assert!(
        original_lib_rs.contains("ptr.add(arr.len())"),
        "original fixture should still contain the unsound out-of-bounds read"
    );

    // Gating on UnsafeAudit means safety_audit must also come back clean: the fix
    // removed the only undocumented unsafe block, and the two sound blocks remain
    // documented with SAFETY: comments.
    let metrics = run_pipeline_for_task(staged.path(), Some(task.task_type));
    println!("{}", metrics.summary_line());
    assert_eq!(
        metrics.failed_count, 0,
        "expected all pipeline stages (including the UnsafeAudit-gated safety_audit) to pass: {}",
        metrics.summary_line()
    );
    assert_eq!(metrics.stages.len(), 4, "expected the 4-stage build/test/clippy/safety_audit pipeline");

    let ledger_dir = tempfile::tempdir().expect("ledger tempdir should be creatable");
    let ledger_path = ledger_dir.path().join("testbed_receipts.jsonl");
    let receipt = chain_receipt(&genesis_chain_hash(), &task.id, compiled.hash(), &task.model, &metrics)
        .expect("chain_receipt should succeed");
    append_receipt(&ledger_path, &receipt).expect("append_receipt should succeed");
    let ledger_content = std::fs::read_to_string(&ledger_path).expect("ledger should be readable back");
    assert_eq!(ledger_content.lines().count(), 1, "expected exactly one receipt line in the fresh ledger");
}

/// A corrected `src/lib.rs` for `crypto_codegen_001`: replaces the hardcoded,
/// reused 96-bit nonce with a freshly generated random nonce per `encrypt()` call
/// (via `Aes256Gcm::generate_nonce`/`OsRng`), while continuing to use the vetted
/// `aes-gcm` crate and leaving `decrypt()` untouched.
const CORRECTED_CRYPTO_CODEGEN_LIB_RS: &str = r#"//! Fixture for `crypto_codegen_001`: AES-256-GCM encryption that now generates a
//! fresh random nonce per call instead of reusing a hardcoded constant.
//!
//! `encrypt` uses the vetted `aes-gcm` crate (not a hand-rolled cipher). Every
//! invocation now uses a freshly generated, cryptographically random 96-bit nonce
//! (via `Aes256Gcm::generate_nonce`/`OsRng`), fixing the nonce-reuse vulnerability
//! that previously broke both confidentiality and integrity.

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

/// Encrypt `plaintext` under `key` using AES-256-GCM.
///
/// # Fix
///
/// This previously reused a hardcoded nonce constant on every call, which breaks
/// both confidentiality and integrity under nonce reuse. It now generates a fresh,
/// cryptographically random nonce per call via [`Aes256Gcm::generate_nonce`] (backed
/// by `OsRng`), and returns the ciphertext (including the appended authentication
/// tag) plus the nonce that was actually used, so callers/tests can verify freshness.
///
/// # Panics
///
/// Panics if `key` is not exactly 32 bytes or the underlying AEAD encryption
/// fails (it should not, for valid inputs).
#[must_use]
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("encryption failure");
    (ciphertext, nonce.into())
}

/// Decrypt `ciphertext` (as produced by [`encrypt`]) under `key` and `nonce`.
///
/// # Panics
///
/// Panics if `key` is not exactly 32 bytes or decryption/authentication fails.
#[must_use]
pub fn decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    cipher.decrypt(nonce, ciphertext).expect("decryption/authentication failure")
}

#[cfg(test)]
mod tests {
    use super::{decrypt, encrypt};

    const KEY: [u8; 32] = [7u8; 32];

    #[test]
    fn round_trips_correctly() {
        let plaintext: &[u8] = b"attack at dawn";
        let (ciphertext, nonce) = encrypt(&KEY, plaintext);
        assert_eq!(decrypt(&KEY, &nonce, &ciphertext), plaintext);
    }

    /// A correct implementation must generate a fresh, random nonce for every
    /// call, so two calls to `encrypt` (even with the same key and plaintext)
    /// must return different nonces.
    #[test]
    fn nonces_differ_across_calls() {
        let message_one: &[u8] = b"first message";
        let message_two: &[u8] = b"second message";
        let (_, nonce1) = encrypt(&KEY, message_one);
        let (_, nonce2) = encrypt(&KEY, message_two);
        assert_ne!(
            nonce1, nonce2,
            "encrypt() must use a freshly generated nonce per call, not a hardcoded constant"
        );
    }

    /// Documents *why* nonce reuse matters, independent of whatever nonce
    /// `encrypt()` itself currently picks: this test constructs its own AES-256-GCM
    /// cipher and deliberately encrypts two plaintexts under one explicitly shared
    /// nonce, then shows that XOR-ing the two ciphertexts (dropping the 16-byte GCM
    /// authentication tag each carries) recovers exactly the XOR of the two
    /// plaintexts — the classic two-time-pad break of any nonce-reuse in a
    /// stream/counter-mode cipher, which is exactly the class of bug the old
    /// hardcoded-nonce `encrypt()` used to cause. This test does not exercise
    /// `encrypt()` and is unaffected by fixing it, so it must keep passing after
    /// the bug is fixed.
    #[test]
    fn reused_nonce_leaks_plaintext_xor() {
        use aes_gcm::aead::{Aead, KeyInit};
        use aes_gcm::{Aes256Gcm, Key, Nonce};

        let plaintext_one: &[u8] = b"AAAAAAAAAAAAAAAA";
        let plaintext_two: &[u8] = b"BBBBBBBBBBBBBBBB";
        let shared_bytes = [0u8; 12];

        let key = Key::<Aes256Gcm>::from_slice(&KEY);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&shared_bytes);
        let first_ciphertext = cipher.encrypt(nonce, plaintext_one).expect("encrypt first");
        let second_ciphertext = cipher.encrypt(nonce, plaintext_two).expect("encrypt second");

        let tag_len = 16;
        let body_len = first_ciphertext.len() - tag_len;
        assert_eq!(body_len, plaintext_one.len());

        let xor_ciphertexts: Vec<u8> = first_ciphertext[..body_len]
            .iter()
            .zip(second_ciphertext[..body_len].iter())
            .map(|(a, b)| a ^ b)
            .collect();
        let xor_plaintexts: Vec<u8> =
            plaintext_one.iter().zip(plaintext_two.iter()).map(|(a, b)| a ^ b).collect();

        assert_eq!(
            xor_ciphertexts, xor_plaintexts,
            "nonce reuse should leak plaintext_a XOR plaintext_b via ciphertext XOR"
        );
    }
}
"#;

fn mock_model_response_crypto_codegen() -> String {
    format!("Here is the corrected `src/lib.rs`:\n\n```rust\n{CORRECTED_CRYPTO_CODEGEN_LIB_RS}\n```\n\nThis generates a fresh random nonce per call instead of reusing a hardcoded constant.")
}

/// Path to `tasks/crypto_codegen_001.ttl`, resolved from the crate root.
fn crypto_codegen_task_ttl_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tasks/crypto_codegen_001.ttl")
}

#[test]
fn walking_skeleton_runs_crypto_codegen_001_flow_with_mocked_model() {
    let ttl_path = crypto_codegen_task_ttl_path();

    let task = load_task(&ttl_path).expect("load_task should succeed for crypto_codegen_001");
    assert_eq!(task.task_type, rust_fable_testbed::spec::TaskType::CryptoCodegen);
    let compiled = compile_task_prompt(&task).expect("compile_task_prompt should succeed");
    assert!(!compiled.content().is_empty(), "compiled prompt content should be non-empty");

    let client = MockModelClient::ok_text(&mock_model_response_crypto_codegen());
    let request = MessageRequest {
        model: &task.model,
        max_tokens: 16_000,
        system: None,
        messages: vec![Message::user(compiled.content())],
        effort: None,
    };
    let response = client.send(&request).expect("mock client send should succeed");
    let model_output = response.text().expect("mock response should yield text");

    let base_dir = ttl_path.parent().expect("ttl path has a parent directory");
    let fixture_dir = base_dir.join(&task.fixture);
    let staged = stage_fixture(&fixture_dir).expect("stage_fixture should succeed");

    apply_model_output(staged.path(), Path::new("src/lib.rs"), &model_output)
        .expect("apply_model_output should find the fenced rust block and write it");

    // Original fixture must remain untouched (still reuses the hardcoded nonce).
    let original_lib_rs = std::fs::read_to_string(fixture_dir.join("src/lib.rs"))
        .expect("original fixture lib.rs should still be readable");
    assert!(
        original_lib_rs.contains("HARDCODED_NONCE"),
        "original fixture should still contain the hardcoded-nonce bug"
    );

    // Gating on CryptoCodegen means safety_audit must also come back clean: the
    // fix removed the only hardcoded-nonce pattern, and no other risky pattern
    // (unsafe/ecb/md5/sha1) was introduced.
    let metrics = run_pipeline_for_task(staged.path(), Some(task.task_type));
    println!("{}", metrics.summary_line());
    assert_eq!(
        metrics.failed_count, 0,
        "expected all pipeline stages (including the CryptoCodegen-gated safety_audit) to pass: {}",
        metrics.summary_line()
    );
    assert_eq!(metrics.stages.len(), 4, "expected the 4-stage build/test/clippy/safety_audit pipeline");

    let ledger_dir = tempfile::tempdir().expect("ledger tempdir should be creatable");
    let ledger_path = ledger_dir.path().join("testbed_receipts.jsonl");
    let receipt = chain_receipt(&genesis_chain_hash(), &task.id, compiled.hash(), &task.model, &metrics)
        .expect("chain_receipt should succeed");
    append_receipt(&ledger_path, &receipt).expect("append_receipt should succeed");
    let ledger_content = std::fs::read_to_string(&ledger_path).expect("ledger should be readable back");
    assert_eq!(ledger_content.lines().count(), 1, "expected exactly one receipt line in the fresh ledger");
}

/// A corrected `src/describe.rs` for `repo_translation_001`: updates
/// `describe_shape` to include `shape.label` in its output (following the same
/// prefix convention `area::describe_area` already uses), per this harness's v1
/// single-target-file scope -- `lib.rs` and `area.rs` are given as read-only
/// context and are not (and need not be) rewritten.
const CORRECTED_REPO_TRANSLATION_DESCRIBE_RS: &str = r#"//! Target module for `repo_translation_001`: `describe_shape` now includes
//! `shape.label` in its summary, matching the sibling `area::describe_area`
//! convention.

use crate::Shape;

/// Return a human-readable summary of `shape`, including its label.
///
/// # Fix
///
/// This previously omitted `shape.label` entirely. It now follows the same
/// `"{label} ..."` prefix convention `area::describe_area` already uses.
#[must_use]
pub fn describe_shape(shape: &Shape) -> String {
    format!("{} {}x{} shape", shape.label, shape.width, shape.height)
}

#[cfg(test)]
mod tests {
    use super::describe_shape;
    use crate::Shape;

    #[test]
    fn describe_shape_includes_dimensions() {
        let shape = Shape::new(3.0, 4.0, "tile");
        let out = describe_shape(&shape);
        assert!(out.contains('3'));
        assert!(out.contains('4'));
    }

    #[test]
    fn describe_shape_includes_the_label() {
        let shape = Shape::new(3.0, 4.0, "tile");
        let out = describe_shape(&shape);
        assert!(out.contains("tile"), "expected output to include the shape's label, got: {out}");
    }
}
"#;

fn mock_model_response_repo_translation() -> String {
    format!("Here is the corrected `src/describe.rs`:\n\n```rust\n{CORRECTED_REPO_TRANSLATION_DESCRIBE_RS}\n```\n\nThis includes shape.label in describe_shape's output, following area::describe_area's prefix convention.")
}

/// Path to `tasks/repo_translation_001.ttl`, resolved from the crate root.
fn repo_translation_task_ttl_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tasks/repo_translation_001.ttl")
}

#[test]
fn walking_skeleton_runs_repo_translation_001_flow_with_mocked_model() {
    let ttl_path = repo_translation_task_ttl_path();

    let task = load_task(&ttl_path).expect("load_task should succeed for repo_translation_001");
    assert_eq!(task.task_type, rust_fable_testbed::spec::TaskType::RepoLevelTranslation);
    let compiled = compile_task_prompt(&task).expect("compile_task_prompt should succeed");
    assert!(!compiled.content().is_empty(), "compiled prompt content should be non-empty");

    let client = MockModelClient::ok_text(&mock_model_response_repo_translation());
    let request = MessageRequest {
        model: &task.model,
        max_tokens: 16_000,
        system: None,
        messages: vec![Message::user(compiled.content())],
        effort: None,
    };
    let response = client.send(&request).expect("mock client send should succeed");
    let model_output = response.text().expect("mock response should yield text");

    let base_dir = ttl_path.parent().expect("ttl path has a parent directory");
    let fixture_dir = base_dir.join(&task.fixture);
    let staged = stage_fixture(&fixture_dir).expect("stage_fixture should succeed");

    // v1 single-target-file scope (see sandbox.rs / repo_translation_001.ttl docs):
    // only describe.rs is overwritten; lib.rs and area.rs are already correct and
    // are left as staged (copied verbatim from the fixture).
    apply_model_output(staged.path(), Path::new("src/describe.rs"), &model_output)
        .expect("apply_model_output should find the fenced rust block and write it");

    // Original fixture must remain untouched (describe_shape still drops label).
    let original_describe_rs = std::fs::read_to_string(fixture_dir.join("src/describe.rs"))
        .expect("original fixture describe.rs should still be readable");
    assert!(
        original_describe_rs.contains(r#"format!("{}x{} shape", shape.width, shape.height)"#),
        "original fixture describe.rs should still drop the label from its output"
    );

    let metrics = run_pipeline_for_task(staged.path(), Some(task.task_type));
    println!("{}", metrics.summary_line());
    assert_eq!(
        metrics.failed_count, 0,
        "expected all pipeline stages to pass with the corrected describe.rs: {}",
        metrics.summary_line()
    );
    assert_eq!(metrics.stages.len(), 4, "expected the 4-stage build/test/clippy/safety_audit pipeline");

    let ledger_dir = tempfile::tempdir().expect("ledger tempdir should be creatable");
    let ledger_path = ledger_dir.path().join("testbed_receipts.jsonl");
    let receipt = chain_receipt(&genesis_chain_hash(), &task.id, compiled.hash(), &task.model, &metrics)
        .expect("chain_receipt should succeed");
    append_receipt(&ledger_path, &receipt).expect("append_receipt should succeed");
    let ledger_content = std::fs::read_to_string(&ledger_path).expect("ledger should be readable back");
    assert_eq!(ledger_content.lines().count(), 1, "expected exactly one receipt line in the fresh ledger");
}

/// Same flow against a real `AnthropicClient::from_env()`, for manual
/// verification. Skips (does not panic) when `ANTHROPIC_API_KEY` is unset, so it
/// is safe to leave enabled in environments without the key — it just never runs
/// unless explicitly requested via `--ignored`.
#[test]
#[ignore = "hits the real Anthropic API; run with `cargo test -- --ignored` and ANTHROPIC_API_KEY set"]
fn walking_skeleton_runs_full_v1_flow_against_real_anthropic_api() {
    use rust_fable_testbed::model_client::AnthropicClient;

    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("skipping walking_skeleton_runs_full_v1_flow_against_real_anthropic_api: ANTHROPIC_API_KEY not set");
        return;
    }

    let ttl_path = task_ttl_path();
    let task = load_task(&ttl_path).expect("load_task should succeed for function_bugfix_001");
    let compiled = compile_task_prompt(&task).expect("compile_task_prompt should succeed");

    let client = AnthropicClient::from_env().expect("ANTHROPIC_API_KEY is set, so this should build a client");
    let request = MessageRequest {
        model: &task.model,
        max_tokens: 16_000,
        system: None,
        messages: vec![Message::user(compiled.content())],
        effort: None,
    };
    let response = client.send(&request).expect("real API call should succeed");
    let model_output = response.text().expect("real response should yield text (or report a refusal)");

    let base_dir = ttl_path.parent().expect("ttl path has a parent directory");
    let fixture_dir = base_dir.join(&task.fixture);
    let staged = stage_fixture(&fixture_dir).expect("stage_fixture should succeed");
    apply_model_output(staged.path(), Path::new("src/lib.rs"), &model_output)
        .expect("apply_model_output should find a fenced rust block in the real response");

    let metrics = run_pipeline_for_task(staged.path(), Some(task.task_type));
    println!("real API run: {}", metrics.summary_line());

    let ledger_dir = tempfile::tempdir().expect("ledger tempdir should be creatable");
    let ledger_path = ledger_dir.path().join("testbed_receipts.jsonl");
    let receipt = chain_receipt(&genesis_chain_hash(), &task.id, compiled.hash(), &task.model, &metrics)
        .expect("chain_receipt should succeed");
    append_receipt(&ledger_path, &receipt).expect("append_receipt should succeed");
}
