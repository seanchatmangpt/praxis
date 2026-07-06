use crate::types::Blake3Hash;

// Domain binding: chains from different projects/versions must not cross-verify.
const GENESIS_SEED_STR: &str = concat!("{{project-name}}-v", env!("CARGO_PKG_VERSION"), "-genesis");
/// Genesis seed for the audit chain; ties this chain to the specific project and version.
pub const GENESIS_SEED: &[u8] = GENESIS_SEED_STR.as_bytes();

fn genesis_hash() -> Blake3Hash {
    Blake3Hash::from_hex(blake3::hash(GENESIS_SEED).to_hex().to_string())
}

// fold rule: blake3(prev_hex_bytes || event_bytes)
fn fold(prev: &Blake3Hash, event_bytes: &[u8]) -> Blake3Hash {
    let mut buf = Vec::with_capacity(prev.as_hex().len() + event_bytes.len());
    buf.extend_from_slice(prev.as_hex().as_bytes());
    buf.extend_from_slice(event_bytes);
    Blake3Hash::from_hex(blake3::hash(&buf).to_hex().to_string())
}

/// Purely recompute the chain hash over ordered byte slices.
/// Used by the verifier — does not mutate any state.
pub fn recompute_chain(events: &[impl AsRef<[u8]>]) -> String {
    let mut acc = genesis_hash();
    for e in events {
        acc = fold(&acc, e.as_ref());
    }
    acc.into()
}

// ── Type-state markers ────────────────────────────────────────────────────

/// Assembler has not received any events yet. `finalize()` is not available.
pub struct Unstarted;
/// Assembler has received at least one event. `finalize()` is now available.
pub struct NonEmpty;

/// Append-only BLAKE3 chain assembler with compile-time empty-chain prevention.
///
/// `ChainAssembler<Unstarted>` cannot be finalized — calling `finalize()` on an
/// empty assembler is a compile-time error (the method does not exist on that state).
/// Call `append()` at least once to advance to `ChainAssembler<NonEmpty>`, which
/// exposes `finalize()`.
///
/// # Examples
///
/// ```rust
/// use {{project_name}}::chain::ChainAssembler;
///
/// let mut asm = ChainAssembler::new();
/// // asm.finalize();  // ← compile error: method not found on ChainAssembler<Unstarted>
/// let mut asm = asm.append(b"first event");
/// let hash = asm.finalize(); // ✓
/// assert_eq!(hash.len(), 64);
/// ```
pub struct ChainAssembler<State = Unstarted> {
    running: Blake3Hash,
    _state: std::marker::PhantomData<State>,
}

impl ChainAssembler<Unstarted> {
    /// Create a new, empty assembler rooted at the genesis hash.
    pub fn new() -> Self {
        ChainAssembler {
            running: genesis_hash(),
            _state: std::marker::PhantomData,
        }
    }

    /// Append the first event, transitioning to `NonEmpty` state.
    pub fn append(self, event_bytes: &[u8]) -> ChainAssembler<NonEmpty> {
        let running = fold(&self.running, event_bytes);
        ChainAssembler {
            running,
            _state: std::marker::PhantomData,
        }
    }
}

impl Default for ChainAssembler<Unstarted> {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainAssembler<NonEmpty> {
    /// Append another event (remains in `NonEmpty` state).
    pub fn append(self, event_bytes: &[u8]) -> ChainAssembler<NonEmpty> {
        let running = fold(&self.running, event_bytes);
        ChainAssembler {
            running,
            _state: std::marker::PhantomData,
        }
    }

    /// Consume the assembler and return the final chain hash as a hex string.
    ///
    /// Only callable after at least one `append()` — the `Unstarted` state has
    /// no `finalize()` method, so empty chains are rejected at compile time.
    pub fn finalize(self) -> String {
        self.running.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_chain_equals_genesis() {
        let result = recompute_chain(&[] as &[&[u8]]);
        let expected: String = genesis_hash().into();
        assert_eq!(result, expected);
    }

    #[test]
    fn append_matches_recompute() {
        let events: &[&[u8]] = &[b"a", b"b", b"c"];
        let asm = ChainAssembler::new().append(b"a").append(b"b").append(b"c");
        assert_eq!(asm.finalize(), recompute_chain(events));
    }

    #[test]
    fn tamper_breaks_chain() {
        let honest = recompute_chain(&[b"x", b"y"]);
        let tampered = recompute_chain(&[b"x", b"z"]);
        assert_ne!(honest, tampered);
    }

    #[test]
    fn type_state_enforces_non_empty_before_finalize() {
        // This test proves the happy path; the compile-time block is enforced by
        // the type system (no finalize() on ChainAssembler<Unstarted>).
        let hash = ChainAssembler::new().append(b"event").finalize();
        assert_eq!(hash.len(), 64);
    }
}
