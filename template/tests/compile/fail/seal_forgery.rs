// FAIL (E0451): Attempting to construct Blake3Hash with a struct literal that
// reaches into the public tuple field is fine for Blake3Hash, but the Seal
// pattern on domain types prevents struct-literal construction of sealed types.
//
// This fixture proves the pattern: we define a local sealed type and attempt
// to construct it with a struct literal from outside the defining module.

mod sealed_domain {
    pub struct SealedReceipt {
        pub chain_hash: String,
        _seal: (),  // private field — only `SealedReceipt::new()` may set this
    }

    impl SealedReceipt {
        pub fn new(chain_hash: String) -> Self {
            SealedReceipt { chain_hash, _seal: () }
        }
    }
}

fn main() {
    // E0451: field `_seal` of struct `SealedReceipt` is private
    let _r = sealed_domain::SealedReceipt { chain_hash: "abc".into(), _seal: () };
}
