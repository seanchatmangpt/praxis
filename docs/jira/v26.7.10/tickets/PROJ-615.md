# PROJ-615 — Optional: ed25519 signatures on workday evidence manifests

Status: CUT (optional cut line exercised at PROJ-617 closure — ed25519 signatures deferred
out of v26.7.10; `EvidenceManifest.signatures` stays a deliberately empty `signatures: []`,
PARTIAL by design; see `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8.1)

## Summary

Wire ed25519 signing (`praxis-core/src/signing.rs`) into `EvidenceManifest.signatures`
(currently the unwired `signatures: []`) for workday bundles, using a seed-derived key so
determinism and same-seed byte-identity are preserved. If key management balloons in scope,
deliver the seed-derived signing only and mark the ticket PARTIAL naming the gap.

## Acceptance criteria

1. Workday `EvidenceManifest.signatures` carries at least one ed25519 signature over the
   canonical manifest bytes; verification API refuses a tampered manifest.
2. Key derivation is seed-based; two same-seed runs remain byte-identical including
   signatures.
3. Negative test: mutated manifest byte ⇒ signature verification refuses with a typed
   refusal.

## Verification

`just cng-test-bench` once implemented: signature round-trip and tamper tests green;
byte-identity gate (PROJ-616) still passes.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 13
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8
