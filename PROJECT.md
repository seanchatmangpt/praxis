# Project: Post-Chatman Research & Praxis Transition Swarm

## Architecture
The project executes two primary tracks in parallel:
1. **PhD Research Track**: Analyzes ggen packs/marketplace and the post-Chatman Equation ($A \cong O \cong L$) paradigm. Synthesizes a PhD-level research paper at `/Users/sac/praxis/research/post_chatman_research.md`.
2. **Praxis Transition Track**: Converts the `~/praxis` directory into a ggen-first architecture. Configures `ggen.toml`, the ontologies, and integration templates.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| M1 | Research Swarm (Wave 1) | Explore Packs, Marketplace, and Post-Chatman theory; compile research paper. | none | DONE |
| M2 | Transition Execution (Wave 2) | Implement ggen-first setup in ~/praxis and run synchronization. | M1 | DONE |
| M3 | Swarm Review & Audit | Independent review and forensic audit verification of the transition and research. | M2 | DONE |
| M4 | Successor Takeover & Final Synthesis | Succession, final verification of results, and user presentation. | M3 | DONE |
| M5 | Repair Core Receipts and Verification Invariants | Bind metadata to chain hash, enforce genesis anchor, update core tests. | M4 | DONE |
| M6 | Repair Theology and Agent Boundaries | Fix surrender boundary bypasses (Datalog/action), default-deny delegability, registry tool check. | M5 | IN_PROGRESS |
| M7 | Repair Quarantine & Firing Verification | Restrict delta-injected hooks/workflows, secure public execute_workflow API. | M6 | PLANNED |
| M8 | Repair PDDL Planning & Solver8 | Fix variable out-of-bounds panics, correct Constraint::After logic, optimize 0-producer goal pre-checks. | M7 | PLANNED |
| M9 | Full Verification & Python Verifier | Update Python verifier (epoch, plan hashing) and run full trustless replay verification. | M8 | PLANNED |
| M10| Disk Cleanup & Hygiene | Clean up caches and build outputs, verify final repository status. | M9 | PLANNED |

## Interface Contracts
- **Research Paper Output**: `/Users/sac/praxis/research/post_chatman_research.md`.
- **Ggen-First Configuration**: Root-level `ggen.toml` and associated schema/templates in `/Users/sac/praxis/`.

## Code Layout
- `research/` - Research documents and papers
- `schema/` - Ontology definition files (`.ttl`)
- `templates/` - Code generation templates
- `crates/` - Source crates for praxis tools
