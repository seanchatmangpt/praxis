# Global Forensics Index: Broken & Fake Code

## Executive Summary
This document indexes the results of the phase 3 and phase 4 security and validation sweep across internal and external repositories, updated during Phase 6 adjudication. A total of 40 files have been flagged for containing fake validation mocks, unsafe credentials, structural syntax failures, or logical mismatches (with 2 of these files now repaired, reducing blockers).

## Classification Counts
- **FAKE**: 7
- **UNSAFE**: 7
- **BROKEN**: 19
- **CLAIM-MISMATCH**: 0
- **THEATRE**: 1
- **SUSPECT**: 1
- **ORPHAN**: 0
- **FAKE/UNSAFE**: 2
- **PLACEHOLDER**: 1

## Severity Table
| Severity | Count |
|----------|-------|
| CRITICAL | 30    |
| HIGH     | 5     |
| MEDIUM   | 2     |
| LOW      | 1     |

## v26.7.3 Blocker Table
| File | Repo | Classification | Severity | Impact |
|------|------|----------------|----------|--------|
| `src/migration/security/pqc_mcp_transport.py` | bytestar | FAKE | CRITICAL | Blocker |
| `src/migration/security/pqc_certificate_manager.py` | bytestar | FAKE | CRITICAL | Blocker |
| `src/migration/security/pqc_security_hooks.py` | bytestar | BROKEN | MEDIUM | Warning |
| `byteactor/src/bytecore/content_addressing.py` | bytestar | FAKE | CRITICAL | Blocker |
| `crates/praxis-synthesis/ontology/lord_prayer.ttl` | praxis | CLAIM-MISMATCH | INFO | NONE (Repaired) |
| `crates/praxis-synthesis/src/kernel.rs` | praxis | CLAIM-MISMATCH | INFO | NONE (Repaired) |
| `rust/genesis-construct8/src/replay.rs` | knhk | THEATRE | HIGH | Blocker |
| `crates/wasm4pm-cli/src/commands/receipt.rs` | wasm4pm | BROKEN | HIGH | Blocker |
| `packages/kgc-probe/src/receipt.mjs` | unrdf | BROKEN | CRITICAL | Blocker |
| `packages/kgc-swarm/src/compression.mjs` | unrdf | BROKEN | CRITICAL | Blocker |
| `packages/kgc-probe/src/probes/filesystem.mjs` | unrdf | BROKEN | CRITICAL | Blocker |
| `packages/fusion/src/policy-engine.mjs` | unrdf | BROKEN | CRITICAL | Blocker |
| `src/core/receipt.rs` | chicago-tdd-tools | UNSAFE | HIGH | Blocker |
| `integrations/bitjob-chrome-ext/src/lib/ahi/autonomous-legal-filer.ts` | cns | UNSAFE | CRITICAL | Blocker |
| `ahi/ahi_legal_filer.c` | cns | UNSAFE | CRITICAL | Blocker |
| `integrations/bitjob-chrome-ext/src/lib/ahi/8-space-contract-enforcer.ts` | cns | UNSAFE | HIGH | Blocker |
| `enterprise-postgresql-cluster/ssl-private/server.key` | cns | UNSAFE | CRITICAL | Blocker |
| `src/revenue_engine/.env.production` | cns | UNSAFE | CRITICAL | Blocker |
| `aegis-nuxt/utils/rightsValidation.ts` | cns | UNSAFE | CRITICAL | Blocker |
| `test/cli-stubs-smoke.test.mjs` | unrdf | BROKEN | MEDIUM | Blocker |
| `bitactor_core_ontology.ttl` | bitactor | SUSPECT | HIGH | Blocker |
| `wasm4pm/ontology/standards/croissant.ttl` | wasm4pm | BROKEN | CRITICAL | Blocker |
| `wasm4pm/ontology/standards/dmop.ttl` | wasm4pm | BROKEN | CRITICAL | Blocker |
| `wasm4pm/ontology/standards/mex-algo.ttl` | wasm4pm | BROKEN | CRITICAL | Blocker |
| `wasm4pm/ontology/standards/mex-perf.ttl` | wasm4pm | BROKEN | CRITICAL | Blocker |
| `wasm4pm/ontology/standards/ontodm.ttl` | wasm4pm | BROKEN | CRITICAL | Blocker |
| `wasm4pm/ontology/standards/skos.ttl` | wasm4pm | BROKEN | CRITICAL | Blocker |
| `wasm4pm/ontology/standards/time.ttl` | wasm4pm | BROKEN | CRITICAL | Blocker |
| `wasm4pm-compat/ggen/shapes/loss-accounting.shacl.ttl` | ggen | BROKEN | CRITICAL | Blocker |
| `wasm4pm-compat/ggen/shapes/process-tree.shacl.ttl` | ggen | BROKEN | CRITICAL | Blocker |
| `unrdf/unproj-ontology.ttl` | unrdf | BROKEN | CRITICAL | Blocker |
| `unrdf/packages-discovered.ttl` | unrdf | BROKEN | CRITICAL | Blocker |
| `unrdf/unrdf-packages.ttl` | unrdf | BROKEN | CRITICAL | Blocker |
| `federal-rights-platform/server/api/auth/login.post.ts` | cns | FAKE/UNSAFE | CRITICAL | Blocker |
| `federal-rights-platform/server/api/auth/logout.post.ts` | cns | FAKE | CRITICAL | Blocker |
| `federal-rights-platform/server/api/auth/mfa/setup.post.ts` | cns | FAKE | CRITICAL | Blocker |
| `federal-rights-platform/server/api/auth/mfa/verify.post.ts` | cns | FAKE/UNSAFE | CRITICAL | Blocker |
| `federal-rights-platform/server/api/auth/refresh.post.ts` | cns | FAKE | CRITICAL | Blocker |
| `federal-rights-platform/server/api/auth/validate.get.ts` | cns | FAKE | CRITICAL | Blocker |

## Security Flags
*Security notice: Raw secrets and private keys are not printed below.*
- **src/core/receipt.rs** (chicago-tdd-tools): Incomplete serialization signature input.
- **integrations/bitjob-chrome-ext/src/lib/ahi/autonomous-legal-filer.ts** (cns): Auto-approves low risk filings without human approval.
- **ahi/ahi_legal_filer.c** (cns): Theatrical usleep simulation of human approval.
- **integrations/bitjob-chrome-ext/src/lib/ahi/8-space-contract-enforcer.ts** (cns): Only reduces compliance score on unauthorized actions.
- **enterprise-postgresql-cluster/ssl-private/server.key** (cns): RSA Private Key PEM file checked into repo.
- **src/revenue_engine/.env.production** (cns): Checked-in production dotenv containing Vault, Salesforce, SAP, Oracle, and Workday secrets.
- **aegis-nuxt/utils/rightsValidation.ts** (cns): Hardcoded plain-text PROOF_SECRET.
- **federal-rights-platform/server/api/auth/login.post.ts** (cns): Hardcoded administrator credentials (`admin@federal.gov`) with database lookup bypass.
- **federal-rights-platform/server/api/auth/logout.post.ts** (cns): Mock session invalidation and token blacklisting.
- **federal-rights-platform/server/api/auth/mfa/setup.post.ts** (cns): Mock user lookup and console-only logging for TOTP setups.
- **federal-rights-platform/server/api/auth/mfa/verify.post.ts** (cns): Mock MFA verification against static secret (`MOCK_SECRET_BASE32`) and predefined backup codes.
- **federal-rights-platform/server/api/auth/refresh.post.ts** (cns): Hardcoded session refreshes returning mocked admin details.
- **federal-rights-platform/server/api/auth/validate.get.ts** (cns): Stubbed active session validation returning a hardcoded admin session.

## Refuse / Quarantine List
- **src/migration/security/pqc_mcp_transport.py** (bytestar) - MockDilithium.verify returns True unconditionally.
- **src/migration/security/pqc_certificate_manager.py** (bytestar) - CA validate_certificate uses MockDilithium.verify.
- **byteactor/src/bytecore/content_addressing.py** (bytestar) - MockPQCValidator.verify_signature only checks key/signature length.
- **src/core/receipt.rs** (chicago-tdd-tools) - Incomplete serialization signature input.
- **integrations/bitjob-chrome-ext/src/lib/ahi/autonomous-legal-filer.ts** (cns) - Auto-approves low risk filings without human approval.
- **ahi/ahi_legal_filer.c** (cns) - Theatrical usleep simulation of human approval.
- **integrations/bitjob-chrome-ext/src/lib/ahi/8-space-contract-enforcer.ts** (cns) - Only reduces compliance score on unauthorized actions.
- **enterprise-postgresql-cluster/ssl-private/server.key** (cns) - RSA Private Key PEM file checked into repo.
- **src/revenue_engine/.env.production** (cns) - Checked-in production dotenv containing Vault, Salesforce, SAP, Oracle, and Workday secrets.
- **aegis-nuxt/utils/rightsValidation.ts** (cns) - Hardcoded plain-text PROOF_SECRET.
- **federal-rights-platform/server/api/auth/login.post.ts** (cns) - Hardcoded administrator credentials (`admin@federal.gov`) with database lookup bypass.
- **federal-rights-platform/server/api/auth/logout.post.ts** (cns) - Mock session invalidation and token blacklisting.
- **federal-rights-platform/server/api/auth/mfa/setup.post.ts** (cns) - Mock user lookup and console-only logging for TOTP setups.
- **federal-rights-platform/server/api/auth/mfa/verify.post.ts** (cns) - Mock MFA verification against static secret (`MOCK_SECRET_BASE32`) and predefined backup codes.
- **federal-rights-platform/server/api/auth/refresh.post.ts** (cns) - Hardcoded session refreshes returning mocked admin details.
- **federal-rights-platform/server/api/auth/validate.get.ts** (cns) - Stubbed active session validation returning a hardcoded admin session.

## Repair-now List
- **crates/praxis-synthesis/ontology/lord_prayer.ttl** (praxis) - REPAIRED: Missing pk:action bindings for three clauses.
- **crates/praxis-synthesis/src/kernel.rs** (praxis) - REPAIRED: enforce_surrender_boundary skips validation if action is None.
- **crates/wasm4pm-cli/src/commands/receipt.rs** (wasm4pm) - verify_challenge only checks PlaceholderEvidenceDetected, ignores nonces.
- **packages/kgc-probe/src/receipt.mjs** (unrdf) - Flawed recursive JSON.stringify replacer.
- **packages/kgc-swarm/src/compression.mjs** (unrdf) - Flawed recursive JSON.stringify replacer.
- **packages/kgc-probe/src/probes/filesystem.mjs** (unrdf) - Flawed recursive JSON.stringify replacer.
- **packages/fusion/src/policy-engine.mjs** (unrdf) - Flawed recursive JSON.stringify replacer.
- **test/cli-stubs-smoke.test.mjs** (unrdf) - References non-existent CLI files, breaking suite execution.
- **wasm4pm/ontology/standards/croissant.ttl** (wasm4pm) - Syntax or namespace prefix failures in OWL/Turtle file croissant.ttl.
- **wasm4pm/ontology/standards/dmop.ttl** (wasm4pm) - Syntax or namespace prefix failures in OWL/Turtle file dmop.ttl.
- **wasm4pm/ontology/standards/mex-algo.ttl** (wasm4pm) - Syntax or namespace prefix failures in OWL/Turtle file mex-algo.ttl.
- **wasm4pm/ontology/standards/mex-perf.ttl** (wasm4pm) - Syntax or namespace prefix failures in OWL/Turtle file mex-perf.ttl.
- **wasm4pm/ontology/standards/ontodm.ttl** (wasm4pm) - Syntax or namespace prefix failures in OWL/Turtle file ontodm.ttl.
- **wasm4pm/ontology/standards/skos.ttl** (wasm4pm) - Syntax or namespace prefix failures in OWL/Turtle file skos.ttl.
- **wasm4pm/ontology/standards/time.ttl** (wasm4pm) - Syntax or namespace prefix failures in OWL/Turtle file time.ttl.
- **wasm4pm-compat/ggen/shapes/loss-accounting.shacl.ttl** (ggen) - Syntax or namespace prefix failures in OWL/Turtle file loss-accounting.shacl.ttl.
- **wasm4pm-compat/ggen/shapes/process-tree.shacl.ttl** (ggen) - Syntax or namespace prefix failures in OWL/Turtle file process-tree.shacl.ttl.
- **unrdf/unproj-ontology.ttl** (unrdf) - Syntax or namespace prefix failures in OWL/Turtle file unproj-ontology.ttl.
- **unrdf/packages-discovered.ttl** (unrdf) - Syntax or namespace prefix failures in OWL/Turtle file packages-discovered.ttl.
- **unrdf/unrdf-packages.ttl** (unrdf) - Syntax or namespace prefix failures in OWL/Turtle file unrdf-packages.ttl.
- **src/migration/security/pqc_security_hooks.py** (bytestar) - Unsized/unsplit command array execution in subprocess.

## Historical Fossil List
- **test/cli-stubs-smoke.test.mjs** (unrdf) - References non-existent CLI files, breaking suite execution.

## Claim-mismatch List
- **crates/praxis-synthesis/ontology/lord_prayer.ttl** (praxis) - REPAIRED: Missing pk:action bindings for three clauses.
- **crates/praxis-synthesis/src/kernel.rs** (praxis) - REPAIRED: enforce_surrender_boundary skips validation if action is None.

## Orphan List
- **ggen/crates/ggen-cli/src/generated_commands.rs** (ggen) - classification: PLACEHOLDER, severity: LOW, impact: NONE, recommendation: KEEP.

## Next Exact Repair
**Lord's Prayer / God Boundary**
