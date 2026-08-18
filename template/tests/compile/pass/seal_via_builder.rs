// PASS: A sealed type built via its canonical constructor compiles fine.
//
// The `ChainAssembler` is the only code path that can produce a `Blake3Hash`
// via the chain fold — no struct literal is needed on the calling side.

fn main() {
    use {{project-name}}::chain::ChainAssembler;

    let mut asm = ChainAssembler::new();
    asm.append(b"event-payload");
    let _chain_hash: String = asm.finalize();
}
