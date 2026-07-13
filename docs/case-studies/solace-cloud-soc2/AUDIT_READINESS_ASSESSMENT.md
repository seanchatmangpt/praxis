# Audit-Evidence-Pipeline Readiness Assessment — Declared Scope

Named deliberately NOT "production readiness" or "audit readiness" in the compliance sense: this
document assesses whether the SOC2 audit-engagement EVIDENCE PIPELINE is verified to do what it
claims — evidence gathering, control-point modeling, exception/remediation tracking, evidence
metrics — never whether Solace Cloud (or any audited entity) IS SOC2 compliant. That
determination belongs exclusively to a licensed CPA firm exercising professional judgment; this
pipeline cannot make it and does not attempt to.

## Declared scope

`A structural evidence-bundle pipeline for the SOC2 Type II audit engagement PROCESS (scoping
through auditor report handoff), modeled as PDDL8/POWL over public-vocabulary (skos/prov/
dcterms/org) instance data, with a Mycin/Datalog standing-role layer and SPARQL-measured
evidence metrics — for one fictional case-study engagement (Solace Cloud).`

## The 10 acceptance criteria

1. **Ontology is public-vocabulary-first.** No private RDF/OWL namespace is minted for the AICPA
   TSC taxonomy itself (confirmed via live web search in Stage 1: no such public ontology
   exists); SKOS concept schemes carry it. *Evidence: `packs/soc2-audit-pack/ontology.ttl`.*
2. **PDDL domain has no compliance/opinion effect atom.** Mechanically checked, not just
   declared. *Evidence: `soc2_test.rs::no_action_effect_ever_asserts_compliance_or_opinion`
   (passes on the real 30-action domain; 2 adversarial mutants both refuse typed).*
3. **The only terminal goal atom is `evidence-bundle-complete`.** *Evidence: the same test;
   `tape.ops[29].label == "confirm-evidence-bundle-complete(solace)"` in
   `case-study/pddl-out/plan.json`.*
4. **The 8-constraint split law holds and is enforced, not assumed.**
   *Evidence: `soc2_test.rs::eight_constraint_split_violations_refuse_typed`.*
5. **The 30-step cycle plans, projects hierarchically into 10 phase children, validates against
   the POWL structural shape, and replays byte-identically.**
   *Evidence: `soc2_test.rs::full_audit_cycle_plans_projects_validates_and_replays_byte_identically`;
   `case-study/powl/powl-digest.txt`.*
6. **Case-study instance data satisfies the SHACL shapes, and the shapes actually bite.**
   *Evidence: `soc2_test.rs::case_study_instance_data_passes_the_soc2_shapes_and_a_mutant_fails`;
   `case-study/shapes/shape-violations.json` (0 violations on the shipped data).*
7. **A `soc2-audit` bench category exists with a content-bearing marker, a hook actuation
   receipt, and a generic role classification** (mirroring every other bench category's
   pattern). *Evidence: `CATEGORIES[15] == "soc2-audit"`; `hooks/workday-pack-2.ttl`'s
   `ex:hook-soc2-audit`; `roles.rs`'s `role_of` entry `("soc2-audit", "auditor")`.*
8. **5 distinct SOC2 standing roles are derivable with real certainty factors AND agree with an
   independent Datalog engine on both role identity and lawful next action.**
   *Evidence: `roles_test.rs::soc2_standing_roles_mycin_and_datalog_agree`.*
9. **3 on-disk SPARQL queries measure the evidence bundle (COUNT-shaped, no inline SPARQL, no
   unsupported aggregates), and exactly one Rust-computed ratio is explicitly tagged
   `DERIVED_ARITHMETIC`.** *Evidence: `queries/metric-soc2-*.rq`;
   `soc2_test.rs::soc2_evidence_metrics_measure_the_shipped_case_study_instance_data`.*
10. **No file in this bundle, this pack, or this bench module claims a compliance verdict.**
    *Evidence: this document's own self-audit, and every other doc in this bundle; see
    `FINAL_VERDICT.md`.*

## The non-goals

- Issuing, simulating, or approximating a SOC2 Type II opinion.
- Determining whether Solace Cloud (fictional) or any real entity is or is not compliant.
- Counting or classifying individual control exceptions within an exception register (the
  pipeline records that a register was produced and evidenced, never a deviation count within
  it — see `queries/metric-soc2-exceptions.rq`'s own header comment).
- A full OCEL 2.0 event log for this case study (`case-study/ocel/evidence-metrics.json` is a
  SPARQL-measured evidence-metrics snapshot, explicitly not an OCEL 2.0 log — disclosed in that
  directory and in `EVIDENCE_MANIFEST.md`).
- Production deployment of this pipeline as a real audit tool for a real CPA firm's engagement.
- Any claim about the TOGAF `ea-adm` bench category or the `arclight-case-study.ttl` rename
  observed mid-session in the concurrent-edit disclosure (`CASE_STUDY.md`) — out of this
  document's scope entirely.

## Explicit forbidden claims

This bundle, and any document that cites it, must never say: "Solace Cloud is SOC2 compliant,"
"the audit passed," "SOC2-ready," "the engagement is complete" (in the attestation sense — Phase
10's evidence-bundle handoff is a PROCESS completion, not an attestation), or any paraphrase
implying a licensed auditor's opinion has been rendered by this software. The lawful vocabulary
is: evidenced, exception identified, remediation applied, evidence bundle assembled.

## Verdict source of truth

There is no computed "verdict" object in this pipeline (unlike the autonomic-standing-factory
case study's GraphLaw judgment) — by design, per the fence above: a verdict is exactly the thing
this pipeline must never produce. `FINAL_VERDICT.md` states pipeline-verification status only
(tests passed, artifacts produced) and is worded to make that distinction explicit in its own
title and body.
