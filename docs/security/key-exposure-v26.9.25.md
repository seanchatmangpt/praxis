# Key exposure record (v26.9.25)

Recorded 2026-09-25 from the v26.9.25 topology scan. Base `18a5b3a4f8ca` of `praxis`.
The private key below is public: it is reachable from refs on the public `origin` remote. It was
already revoked on 2026-09-24 (the `.praxis/keys` row of
docs/security/signing-key-rotation-v26.9.24.md). This record adds the exposure surface found on
2026-09-25, including the `backup/v26.9.25/*` refs pushed during the single-checkout migration.

The key stays revoked: any receipt or attestation signed with it carries no signing authority
(standing REFUSED, broken_term R_missing_authority). History is not rewritten and no ref is
force-pushed. Revocation is the remedy; the blob stays reachable from the refs listed below.

## Revoked key

| field | value |
|---|---|
| path | `.praxis/keys/private.key` |
| git blob | `afa504b7002a76eb6d7803697a4d0d0d0fc4309d` |
| private key file sha256 | `9d37225d61e306cc9ed87ad51035d80681ef2daff4586accfde0b80dae6ca9ae` |
| derived Ed25519 public key | `3754cf497bf5505b43fd172bd96264e48ba76060c72d85820326229f8cdcf902` |
| first commit on origin | `055221485b40c7ad020c647d8859049c3b645c1e` (2026-06-23T10:42:47-07:00) |
| commits on origin that add or remove it | 3 |
| origin branches whose history reaches it | 8 (includes `main`: yes) |
| origin branches with the key file at the tip | 4 |

## Replacement

The canonical checkout does not use this key. Its `.praxis/keys` pair is the v26.9.24 rotation
pair: generated locally, never committed (no git object in this repository holds either half),
and the private half derives the public key published in
`docs/security/signing-key-rotation-v26.9.24.md`. Verify receipts signed after 2026-09-24
against that published public key only.

## Exposure surface: origin branches reaching the key

- `agent/praxis-dfcm-brce-reconciler` (key file at tip)
- `agent/v2030-1-1-prd-ard-20260819` (key file at tip)
- `backup/v26.9.25/praxis-hierarchical-projection-wt`
- `backup/v26.9.25/praxis-hierarchical-projection-wt-d3d47661a4`
- `feat/dfcm-federated-capabilities-v26.9.1` (key file at tip)
- `integration/all-relevant-20260819` (key file at tip)
- `main`
- `security/rotate-signing-keys-v26.9.24`

Of these, 2 are `backup/v26.9.25/*` refs. They preserve migrated work and are kept.
