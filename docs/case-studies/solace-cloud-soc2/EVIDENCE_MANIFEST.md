# Evidence Manifest — Solace Cloud SOC2 Case Study

Every path below was generated this session by the real `cng` pipeline (`import_artifacts` →
`generate_plan` → `hierarchical_projection` → `powl_to_turtle`, plus `shape_violations` and
`compute_evidence_metrics`) running against an isolated scratchpad copy of the Stage-1-committed
fixtures (see `CASE_STUDY.md`'s concurrent-edit disclosure for why a copy, not the live shared
directory, was used). Hashes are sha256, computed directly from the files as committed in this
bundle.

| path | sha256 | producer | role | deterministic |
|---|---|---|---|---|
| `case-study/pddl/audit-scoping.ttl` .. `audit-report-handoff.ttl` (10 files) | see `sha256sum -c` below | Stage 1 (`packs/soc2-audit-pack` templates, rendered) | PDDL8 domain fragment carrier (literal text inside Turtle — this pack carries no separate `.pddl` file by design) | true |
| `case-study/pddl/soc2-cycle-problem.ttl` | `48ca8d4d…653f3e` | Stage 1 | PDDL8 problem fragment carrier (init + terminal goal atom) | true |
| `case-study/pddl/solace-case-study.ttl` | `7a0e4963…91d39f8` | Stage 1 | case-study instance data (public-vocabulary: skos/prov/dcterms/org) | true |
| `case-study/pddl-out/domain.json` | `357e125e…88b002f2ea` | Stage 2 (`serde_json` dump of the merged `Pddl8Domain`) | structured domain dump (30 actions) | true (re-run this session confirmed byte-identical) |
| `case-study/pddl-out/problem.json` | `714aec3a…3dad605ab316eab4d` | Stage 2 | structured problem dump (init/goal/objects) | true |
| `case-study/pddl-out/plan.json` | `f534693c…6bd89a73c9a` | Stage 2 (`serde_json` dump of the `Pddl8Tape`) | the 30-step plan | true |
| `case-study/powl/solace-soc2-powl.ttl` | `762cb651…4c87d7988a6dab9d5` | Stage 2 (`powl_to_turtle`) | POWL v2 export (10 phase children) | true (blake3 digest below re-confirmed) |
| `case-study/powl/powl-digest.txt` | `7f2869e8…f83d9af86e363666756d` | Stage 2 (`blake3::hash` of the POWL Turtle bytes) | replay digest | true |
| `case-study/powl/phase_sources.json` | `aa1d58ce…6996df7d35e4f3708aa2` | Stage 2 (`hierarchical_projection`'s phase-provenance return) | one source IRI per phase child | true |
| `case-study/shapes/soc2-shapes.ttl` | `42dc623b…9c9f1db461d1b6137fe` | Stage 1 (copy of `crates/cng/shapes/soc2-shapes.ttl`) | SHACL shapes | true (hand-authored, static) |
| `case-study/shapes/shape-violations.json` | `4f53cda1…873c2f11161202b945` | Stage 2 (`shape_violations` over the shipped instance data) | shape-check result (empty = 0 violations) | true |
| `case-study/ocel/evidence-metrics.json` | `abb0df6b…30251983a30bd6c64f` | Stage 2 (`compute_evidence_metrics`) | evidence-bundle metrics (2 measured counts + 1 `DERIVED_ARITHMETIC` ratio) — **not** a full OCEL 2.0 event log, see note below | true |
| `crates/cng/queries/metric-soc2-evidenced-controls.rq` | (see file) | Stage 2 (hand-authored) | on-disk COUNT query | true (hand-authored, static) |
| `crates/cng/queries/metric-soc2-exceptions.rq` | (see file) | Stage 2 (hand-authored) | on-disk COUNT query | true |
| `crates/cng/queries/metric-soc2-remediation-status.rq` | (see file) | Stage 2 (hand-authored) | on-disk COUNT query | true |
| `crates/cng/templates/bench-category-soc2-audit.template.ttl` | (see file) | Stage 2 (hand-authored) | content-bearing category fragment template | true |
| `crates/cng/hooks/workday-pack-2.ttl` (soc2-audit entry) | (see file) | Stage 2 (hand-authored addition, priority 16) | hook-actuation receipt entry | true |
| `crates/cng/rules/bench-roles.dl` (5 soc2 obligation rules) | (see file) | Stage 2 (hand-authored addition) | Datalog obligation rules, Mycin-parity-checked | true |
| `crates/cng/src/bench/roles.rs` (`soc2_role_rules`/`infer_soc2_standing_role`) | (see file) | Stage 2 | Mycin certainty-factor rules for the 5 SOC2 standing roles | true |
| `crates/cng/src/bench/soc2.rs` (`Soc2EvidenceMetrics`/`compute_evidence_metrics`) | (see file) | Stage 2 | evidence-metrics struct + computation | true |

## Note on `case-study/ocel/`

Named `ocel/` to mirror the reference case-study bundle's directory layout, but the file inside
is a SPARQL-measured evidence-metrics snapshot (`Soc2EvidenceMetrics`), not a full OCEL 2.0 event
log (`eventTypes`/`events`/`objectTypes`/`objects`). Building a full OCEL log for this case study
would require standing up the same obs-writer/CONSTRUCT-query evidence pipeline the Fortune-5
`rwai-bench` harness uses (`bench::run`/`bench::manufacture`) for a single case-study cycle — out
of Stage 2's scope as instructed. This is disclosed here and in
`AUDIT_READINESS_ASSESSMENT.md`'s non-goals, not silently substituted.

## Reproducing these numbers

```bash
just cng-test-lib-isolated <name> bench::soc2 -- --nocapture
just cng-test-lib-isolated <name> bench::roles -- --nocapture
```

`sha256sum -c` against the 10 phase-fragment files in `case-study/pddl/` (identical to their
Stage-1-committed content at `756a258470e19bedc3d12b456d9df7b3030ec76b`):

```
f1c5597a6e13015a79d1278dcd1eca684c6ae65d23a7001eba392b51eb0e6ce9  audit-bundle-assembly.ttl
6084d1c63cd5534ff90af8d6e6e4737a4a51468ae421ada492a74d95b3294ef7  audit-collection-init.ttl
3678f386caaacedc698365806c9380d941072e77e31c94ce18240a173995a5a9  audit-control-design-doc.ttl
6ff1a28fec46232c1c82c175d29f3eee1d26efa793ceb62944168d2d626e15bb  audit-design-eval.ttl
3c07a2d29b2c55a03ca5d672f0f479384916d03e3335a308bbb7746e46dd18eb  audit-exception-id.ttl
151a22d5abaa96e809329c22bcaf1de9587997d6f36643c30751e07322caaab4  audit-oe-testing.ttl
2f2b89def1b9578bbe79fb237ba0b10cc47f7c588970733838e7a318aee74649  audit-readiness.ttl
44f16d1c2eb447246876a4573d5ddc5fa22fa541e4f2d096732600ddfd2fd2d7  audit-remediation.ttl
aa3b7cbd857fb1d4e225bf06a43789f6087bcc687dfacb49c42c46571d98e22e  audit-report-handoff.ttl
21ecf99cbce45b9a446ac13ed78b6ce26d09362702f2bfd5a36c4b73adb671ab  audit-scoping.ttl
```
