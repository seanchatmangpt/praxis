use crate::types::Blake3Hash;

// Domain binding: chains from different projects/versions must not cross-verify.
const GENESIS_SEED_STR: &str = concat!("{{project-name}}-v", env!("CARGO_PKG_VERSION"), "-genesis");
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

/// Append-only assembler; running hash is folded incrementally so finalize is O(1).
pub struct ChainAssembler {
    running: Blake3Hash,
}

impl Default for ChainAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainAssembler {
    pub fn new() -> Self {
        ChainAssembler { running: genesis_hash() }
    }

    pub fn append(&mut self, event_bytes: &[u8]) -> Blake3Hash {
        self.running = fold(&self.running, event_bytes);
        self.running.clone()
    }

    /// Consume the assembler and return the final chain hash as a hex string.
    pub fn finalize(self) -> String {
        // _seal pattern: the caller cannot construct a Receipt{_seal:(),...} directly;
        // they must pass through this method (or an equivalent builder) so the chain
        // hash is always the canonical rolling accumulation — struct-literal construction
        // of sealed types fails at compile time with E0451 (private field).
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
        let mut asm = ChainAssembler::new();
        for e in events {
            asm.append(e);
        }
        assert_eq!(asm.finalize(), recompute_chain(events));
    }

    #[test]
    fn tamper_breaks_chain() {
        let honest = recompute_chain(&[b"x", b"y"]);
        let tampered = recompute_chain(&[b"x", b"z"]);
        assert_ne!(honest, tampered);
    }
}
