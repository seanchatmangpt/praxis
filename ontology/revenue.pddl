; revenue.pddl — hand-authored PDDL8-safe revenue-pipeline planning domain.
;
; Companion to ontology/revenue.ttl (the operator vocabulary) and to the
; praxis-proposer crate (PR-14): `propose goal` emits goal atoms of the form
;   (stage <account-id> <stage-name>)
; which splice directly into this file's problem `(:goal ...)` block and are
; then solvable by `plan solve` (classical mode, STRIPS8 grounder).
;
; Design constraints honored (AR-5 / PR-7 bounded planning):
;   * :strips + :typing only — no ADL (no forall/implies/negative
;     preconditions), so the STRIPS8 grounder can ground every action.
;   * Evidence gates are separate action schemas keyed off static stage
;     predicates (gate-free / needs-legal-security / needs-full-evidence)
;     instead of conditional preconditions.
;   * Stage progression is data ((next ?from ?to) facts), mirroring
;     praxis-proposer's Stage::ALL pipeline order.
;   * Effects are add-only: reaching a later stage never deletes the record
;     that an earlier stage was passed (receipt-friendly monotone state).
;
; The problem block mirrors the 3-account fixture in
; crates/praxis-proposer/examples/rank_fixture.rs: acct-1 (procurement, full
; evidence), acct-2 (qualified, missing legal approval), acct-3 (lead, no
; evidence). Its goal is the fixture's top-ranked proposal under the default
; authored objective: advance acct-1 to closed-won.

(define (domain revenue-pipeline)
  (:requirements :strips :typing)
  (:types account rstage)
  (:predicates
    (stage ?a - account ?s - rstage)
    (next ?s1 - rstage ?s2 - rstage)
    (gate-free ?s - rstage)
    (needs-legal-security ?s - rstage)
    (needs-full-evidence ?s - rstage)
    (legal-approved ?a - account)
    (security-reviewed ?a - account)
    (exec-sponsored ?a - account))
  (:action advance
    :parameters (?a - account ?from - rstage ?to - rstage)
    :precondition (and (stage ?a ?from) (next ?from ?to) (gate-free ?to))
    :effect (stage ?a ?to))
  (:action advance-gated
    :parameters (?a - account ?from - rstage ?to - rstage)
    :precondition (and (stage ?a ?from) (next ?from ?to) (needs-legal-security ?to)
                       (legal-approved ?a) (security-reviewed ?a))
    :effect (stage ?a ?to))
  (:action close
    :parameters (?a - account ?from - rstage ?to - rstage)
    :precondition (and (stage ?a ?from) (next ?from ?to) (needs-full-evidence ?to)
                       (legal-approved ?a) (security-reviewed ?a) (exec-sponsored ?a))
    :effect (stage ?a ?to)))

(define (problem revenue-fixture)
  (:domain revenue-pipeline)
  (:objects
    acct-1 acct-2 acct-3 - account
    lead qualified proposal procurement closed-won - rstage)
  (:init
    (stage acct-1 procurement)
    (stage acct-2 qualified)
    (stage acct-3 lead)
    (next lead qualified)
    (next qualified proposal)
    (next proposal procurement)
    (next procurement closed-won)
    (gate-free qualified)
    (gate-free proposal)
    (needs-legal-security procurement)
    (needs-full-evidence closed-won)
    (legal-approved acct-1)
    (security-reviewed acct-1)
    (exec-sponsored acct-1)
    (security-reviewed acct-2)
    (exec-sponsored acct-2))
  (:goal (stage acct-1 closed-won)))
