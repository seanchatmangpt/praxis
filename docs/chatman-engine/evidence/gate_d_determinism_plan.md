# Gate D Determinism — command plan (prepared, not yet run)

Run only after the current release build/test finishes and holds the target-dir lock free.

```bash
# 1. receipt_root byte-identical across 5 runs (reuses chatman_e2e_pipeline.rs, already
#    proves this for the S1-S6 pipeline test; Gate D additionally wants the acceptance-suite
#    receipt paths covered)
for i in 1 2 3 4 5; do
  cargo test -p praxis-graphlaw --release --test chatman_e2e_pipeline -- --nocapture \
    2>&1 | grep -i "receipt_root" >> /tmp/gate_d_roots.txt
done
sort -u /tmp/gate_d_roots.txt | wc -l   # expect 1

# 2. OCEL reseal determinism (after OCEL wiring verified live)
rm -rf .cargo-cicd/ocel/chatman
cargo test -p praxis-graphlaw --release --test chatman_acceptance_receipts -- --nocapture
sha256sum .cargo-cicd/ocel/chatman/receipts.receipt.json > /tmp/ocel_run1.sha
rm -rf .cargo-cicd/ocel/chatman
cargo test -p praxis-graphlaw --release --test chatman_acceptance_receipts -- --nocapture
sha256sum .cargo-cicd/ocel/chatman/receipts.receipt.json > /tmp/ocel_run2.sha
diff /tmp/ocel_run1.sha /tmp/ocel_run2.sha   # expect no diff (path differs, hash must match)

# 3. double ggen sync byte-identical (only if ggen is actually invoked in this repo's chatman
#    flow — confirm `cargo run -p ggen --bin ggen -- sync run` applies to chatman fixtures
#    before treating this as a Gate D requirement; the acceptance harness is hand-written, not
#    ggen-generated in its dispatch logic, only the 8 *_test.rs shells are ggen output)
```

Record raw output under `docs/chatman-engine/evidence/gate_d.txt`, not paraphrased.
