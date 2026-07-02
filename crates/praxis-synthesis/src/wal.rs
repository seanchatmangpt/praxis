//! Step 2 of the cell roadmap — durable replay: receipts must survive
//! machine death.
//!
//! An append-only binary write-ahead log journals the memo cache: each frame
//! is one `(memo_key, output)` pair, length-prefixed and BLAKE3-guarded, so
//! a torn tail (the process died mid-write) is detected and dropped rather
//! than trusted. Recovery is not a special code path: rehydrate the memo
//! cache from the surviving frames and re-run the same deterministic
//! execution — journaled nodes replay, missing nodes recompute, and the
//! final receipt is byte-identical to an uninterrupted run
//! (Theorem 4.2 of the synthesis thesis does the work; the WAL only has to
//! not lie about what completed).
//!
//! Frame layout: `[len: u32 LE][hash: 32 bytes = BLAKE3(payload)][payload]`
//! where payload = `[key_len: u32 LE][key bytes][output bytes]`.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use crate::dag::MemoCache;
use crate::Refusal;

/// Append-only journal of memo-cache entries.
#[derive(Debug)]
pub struct Wal {
    file: File,
}

impl Wal {
    /// Open (creating if absent) the journal at `path` for appending.
    pub fn open(path: &Path) -> Result<Self, Refusal> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| Refusal::InvalidInput { detail: format!("wal open: {e}") })?;
        Ok(Self { file })
    }

    /// Append one `(key, output)` frame and fsync it — the frame is durable
    /// (or absent) after this returns; never torn-and-trusted.
    pub fn append(&mut self, key: &str, output: &[u8]) -> Result<(), Refusal> {
        let mut payload = Vec::with_capacity(4 + key.len() + output.len());
        #[allow(clippy::cast_possible_truncation)]
        payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
        payload.extend_from_slice(key.as_bytes());
        payload.extend_from_slice(output);
        let hash = blake3::hash(&payload);
        let mut frame = Vec::with_capacity(4 + 32 + payload.len());
        #[allow(clippy::cast_possible_truncation)]
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(hash.as_bytes());
        frame.extend_from_slice(&payload);
        self.file
            .write_all(&frame)
            .and_then(|()| self.file.sync_data())
            .map_err(|e| Refusal::InvalidInput { detail: format!("wal append: {e}") })
    }

    /// Recover all intact frames from `path` into a fresh [`MemoCache`].
    /// A torn or corrupt tail frame (short read, length overrun, or hash
    /// mismatch — the kill-9 signature) ends recovery at the last good
    /// frame. Returns `(cache, intact_frames, tail_torn)`.
    pub fn recover(path: &Path) -> Result<(MemoCache, usize, bool), Refusal> {
        let mut bytes = Vec::new();
        match File::open(path) {
            Ok(mut f) => {
                f.read_to_end(&mut bytes)
                    .map_err(|e| Refusal::InvalidInput { detail: format!("wal read: {e}") })?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok((MemoCache::new(), 0, false));
            }
            Err(e) => return Err(Refusal::InvalidInput { detail: format!("wal open: {e}") }),
        }
        let mut cache = MemoCache::new();
        let mut at = 0usize;
        let mut frames = 0usize;
        let mut torn = false;
        while at < bytes.len() {
            if at + 36 > bytes.len() {
                torn = true;
                break;
            }
            let len =
                u32::from_le_bytes(bytes[at..at + 4].try_into().expect("4 bytes")) as usize;
            let hash: [u8; 32] = bytes[at + 4..at + 36].try_into().expect("32 bytes");
            let start = at + 36;
            if start + len > bytes.len() {
                torn = true;
                break;
            }
            let payload = &bytes[start..start + len];
            if blake3::hash(payload).as_bytes() != &hash {
                torn = true;
                break;
            }
            // Decode payload.
            if payload.len() < 4 {
                torn = true;
                break;
            }
            let klen =
                u32::from_le_bytes(payload[..4].try_into().expect("4 bytes")) as usize;
            if 4 + klen > payload.len() {
                torn = true;
                break;
            }
            let Ok(key) = std::str::from_utf8(&payload[4..4 + klen]) else {
                torn = true;
                break;
            };
            cache.insert_raw(key.to_string(), payload[4 + klen..].to_vec());
            frames += 1;
            at = start + len;
        }
        Ok((cache, frames, torn))
    }
}
