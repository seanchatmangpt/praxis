# Fable Genesis Day 2 — PRD/ARD (Recorded Copy)

This PRD/ARD was supplied by the operator on 2026-07-02 and is recorded verbatim as the release's requirements document.

## Product Requirements (condensed)

- **PR-1** — Receipts-grounded: every claim in the fable must map to a receipt, refusal, or deferral.
- **PR-2** — Sean is the protagonist; the story follows his actual work, not an invented hero.
- **PR-3** — Teach the law: the narrative must convey the Law of Work as its spine.
- **PR-4** — Connect to prototype: the fable links to the running Praxis prototype.
- **PR-5** — Projection principle: the fable is a projection of the machine, never a substitute for it.
- **PR-6** — Include the spilled-milk and pen scenes as concrete grounding moments.
- **PR-7** — Refuse false biography: no invented life events or achievements.
- **PR-8** — First-contact path: a new reader can onboard from the fable to the receipts.
- **PR-9** — Machine-readable: claims and manifest ship as JSON/JSONL metadata.
- **PR-10** — Phase-change-eligible-only language: adoption outcomes are stated as eligible, never achieved.

## Architecture Decisions (condensed)

- **AD-1** — The fable is a projection: narrative derives from receipts, not the reverse.
- **AD-2** — Sean is the protagonist by decision, fixing the point of view.
- **AD-3** — Mythic tone over a technical spine: style may soar, facts may not.
- **AD-4** — Unsupported claims are recorded as refusals, not softened into the text.
- **AD-5** — Machine-readable metadata (manifest + claims JSONL) is a first-class artifact.
- **AD-6** — Phase-change language is bounded: "eligible" is the ceiling.
- **AD-7** — No public release is made from the fable itself.
- **AD-8** — The fable links to the projection principle documentation.
