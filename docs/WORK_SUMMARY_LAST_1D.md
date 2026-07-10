# Work Summary: Last 24 Hours (Chatman Engine v26.7.9)

This documentation summary organizes the recent work (PROJ-411..417) using the [Diátaxis](https://diataxis.fr/) framework:

## 1. Tutorials (Learning-oriented)
*Focuses on helping newcomers build confidence through step-by-step projects.*
*   **Getting Started with the S1-S6 Pipeline:** A walkthrough of the new 6-stage Chatman Engine pipeline, tracing a request from the initial snapshot/RDFC-1.0 hash through OWL closure, PDDL plan, POWL v2 trace admission, and knowledge hooks, ending with the 9-hash `ProcessReceiptEnvelope`.
*   **Running PDDL-to-POWL Projections:** A lesson utilizing the newly introduced `cng` crate CLI to project the Joseph famine-cycle plan corpus into POWL v2 traces.

## 2. How-to Guides (Problem-oriented)
*Focuses on directions for accomplishing specific, real-world tasks.*
*   **How to achieve deterministic OCEL evidence:** A guide on using stable ordinals instead of positional event IDs to ensure byte-identical receipt digests when sealing `.cargo-cicd/ocel/chatman/<suite>.{ocel,receipt}.json`.
*   **How to speed up test iteration loops:** Instructions on combining `sccache` with `cargo nextest`, alongside the new lean `dev` profile (which drops debug info to line-tables-only and disables incremental compilation to maximize object reuse across 30+ test binaries).

## 3. Reference (Information-oriented)
*Focuses on dry, accurate, and structured descriptions of the machinery.*
*   **RDF-Native Acceptance Fixtures (`ontology.ttl`):** Reference for the new typed RDF structures in `packs/chatman-engine-pack/ontology.ttl` (covering 43 cases and 646 list-item nodes), which wholly replaced the legacy JSON-blob embedding.
*   **The `Refusal` Enum Catalog:** Documentation of the ~29-variant typed enum utilized across the engine to guarantee zero panics/unwraps and safe error path routing.
*   **RDFTriple8 Hot Path Architecture:** Specifications for the newly implemented 256-entry branchless admission table, gated by the 8-logical-tick Chatman Constant.
*   **Gate A-E Evidence Record:** The formal audit trail, including exact-pinned oxigraph/oxrdf dependencies and the anchored `chatman_s1_receipt_shape` snapshot baseline, stored under `docs/chatman-engine/evidence/`.

## 4. Explanation (Understanding-oriented)
*Focuses on clarifying why the system is built the way it is.*
*   **Why positional event IDs were removed from OCEL:** An exploration of the race conditions that occurred in parallel test threads, explaining why stable ordinals were necessary to satisfy the strict determinism invariant.
*   **Why we anchor the snapshot baseline inside Praxis:** A discussion on moving the `chatman_s1_receipt_shape` baseline from external dependencies directly into `tests/snapshots/` to maintain the EngineProcessReceipt 9-digest doctrine.
*   **Why the dev profile disables incremental compilation:** An explanation of how incremental artifacts defeat `sccache` cacheability, and why disabling them yields a massive net performance win for the project's cross-binary object reuse.
