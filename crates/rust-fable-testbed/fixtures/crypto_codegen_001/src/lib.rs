//! Fixture for `crypto_codegen_001`: AES-256-GCM encryption that reuses a
//! hardcoded nonce across every call.
//!
//! `encrypt` uses the vetted `aes-gcm` crate (not a hand-rolled cipher), so the
//! algorithm choice is sound, but every invocation is encrypted under the exact
//! same 96-bit nonce. Reusing a nonce with the same key under AES-GCM is
//! catastrophic: GCM (like any counter-mode/stream construction) XORs the
//! plaintext with a keystream derived from `(key, nonce)`, so two ciphertexts
//! produced under the same `(key, nonce)` pair leak the XOR of their plaintexts
//! (`c1 XOR c2 == p1 XOR p2`) and the shared authentication keystream is exposed,
//! letting an attacker forge tags. This is the bug the model under test is
//! expected to fix: give every call a fresh, randomly generated nonce (e.g. via
//! `aes_gcm::aead::OsRng`), never a constant.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};

// BUG: this nonce is a fixed constant reused on every call to `encrypt` below,
// instead of being freshly and randomly generated per encryption. Reusing a
// nonce with AES-GCM under the same key breaks both confidentiality (two-time
// pad via XOR of ciphertexts) and integrity (forgeable tags).
const HARDCODED_NONCE: [u8; 12] = [0u8; 12];

/// Encrypt `plaintext` under `key` using AES-256-GCM.
///
/// # Bug
///
/// Every call reuses [`HARDCODED_NONCE`] instead of generating a fresh random
/// nonce per call, so any two ciphertexts produced with the same key leak
/// `plaintext_a XOR plaintext_b` and allow tag forgery. Returns the ciphertext
/// (including the appended authentication tag) plus the nonce that was actually
/// used, so callers/tests can observe the reuse.
///
/// # Panics
///
/// Panics if `key` is not exactly 32 bytes or the underlying AEAD encryption
/// fails (it should not, for valid inputs).
#[must_use]
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&HARDCODED_NONCE);
    let ciphertext = cipher.encrypt(nonce, plaintext).expect("encryption failure");
    (ciphertext, HARDCODED_NONCE)
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
        let (ciphertext, nonce) = encrypt(&KEY, b"attack at dawn");
        assert_eq!(decrypt(&KEY, &nonce, &ciphertext), b"attack at dawn");
    }

    /// This is the bug: a correct implementation must generate a fresh, random
    /// nonce for every call, so two calls to `encrypt` (even with the same key
    /// and plaintext) must return different nonces. The current implementation
    /// always returns [`super::HARDCODED_NONCE`], so this fails.
    #[test]
    fn nonces_differ_across_calls() {
        let (_, nonce1) = encrypt(&KEY, b"first message");
        let (_, nonce2) = encrypt(&KEY, b"second message");
        assert_ne!(
            nonce1, nonce2,
            "encrypt() must use a freshly generated nonce per call, not a hardcoded constant"
        );
    }

    /// Documents *why* the bug in `encrypt()` matters, independent of whatever
    /// nonce `encrypt()` itself currently picks: this test constructs its own
    /// AES-256-GCM cipher and deliberately encrypts two plaintexts under one
    /// explicitly shared nonce, then shows that XOR-ing the two ciphertexts
    /// (dropping the 16-byte GCM authentication tag each carries) recovers
    /// exactly the XOR of the two plaintexts — the classic two-time-pad break
    /// of any nonce-reuse in a stream/counter-mode cipher, which is exactly the
    /// class of bug `HARDCODED_NONCE` causes in `encrypt()`. This test does not
    /// exercise `encrypt()` and is unaffected by fixing it, so it must keep
    /// passing after the bug is fixed.
    #[test]
    fn reused_nonce_leaks_plaintext_xor() {
        use aes_gcm::aead::{Aead, KeyInit};
        use aes_gcm::{Aes256Gcm, Key, Nonce};

        let plaintext_a = b"AAAAAAAAAAAAAAAA";
        let plaintext_b = b"BBBBBBBBBBBBBBBB";
        let shared_nonce_bytes = [0u8; 12];

        let key = Key::<Aes256Gcm>::from_slice(&KEY);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&shared_nonce_bytes);
        let ciphertext_a = cipher.encrypt(nonce, plaintext_a.as_slice()).expect("encrypt a");
        let ciphertext_b = cipher.encrypt(nonce, plaintext_b.as_slice()).expect("encrypt b");

        let tag_len = 16;
        let body_len = ciphertext_a.len() - tag_len;
        assert_eq!(body_len, plaintext_a.len());

        let xor_ciphertexts: Vec<u8> = ciphertext_a[..body_len]
            .iter()
            .zip(ciphertext_b[..body_len].iter())
            .map(|(a, b)| a ^ b)
            .collect();
        let xor_plaintexts: Vec<u8> =
            plaintext_a.iter().zip(plaintext_b.iter()).map(|(a, b)| a ^ b).collect();

        assert_eq!(
            xor_ciphertexts, xor_plaintexts,
            "nonce reuse should leak plaintext_a XOR plaintext_b via ciphertext XOR"
        );
    }
}
