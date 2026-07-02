; church.pddl — hand-authored PDDL8-safe church-operations planning domain.
;
; The exact parallel of ontology/revenue.pddl. Companion to ontology/church.ttl
; (the operator vocabulary) and to the praxis-proposer church pack (Genesis
; Day 6): `propose church` emits goal atoms of the form
;   (stage <person-id> <stage-name>)
; which splice directly into this file's problem `(:goal ...)` block and are
; then solvable by `plan solve` (classical mode, STRIPS grounder).
;
; Design constraints honored (AR-5 / PR-7 bounded planning), identical to
; revenue.pddl:
;   * :strips + :typing only — no ADL (no forall/implies/negative
;     preconditions), so the grounder can ground every action.
;   * Evidence gates are separate action schemas keyed off static per-tier
;     stage predicates (gate-free / needs-followup / needs-group / needs-care)
;     instead of conditional preconditions. Positive preconditions only.
;   * Stage progression is data ((next ?from ?to) facts), mirroring
;     praxis-proposer's church::Stage::ALL order.
;   * Effects are add-only: reaching a later stage never deletes the record
;     that an earlier stage was passed (receipt-friendly monotone state).
;
; The problem block mirrors the 3-person fixture in
; crates/praxis-proposer/tests/church_proposer_tests.rs: visitor-1 (connected,
; full evidence), visitor-2 (first-time, welcomed but never followed up),
; visitor-3 (first-time, no touch). Its goal is the fixture's top-ranked
; proposal under the authored ZOE objective: walk visitor-1 to leading.

(define (domain church-operations)
  (:requirements :strips :typing)
  (:types person cstage)
  (:predicates
    (stage ?p - person ?s - cstage)
    (next ?s1 - cstage ?s2 - cstage)
    (gate-free ?s - cstage)
    (needs-followup ?s - cstage)
    (needs-group ?s - cstage)
    (needs-care ?s - cstage)
    (welcomed ?p - person)
    (followed-up ?p - person)
    (in-small-group ?p - person)
    (care-assigned ?p - person))
  (:action invite-back
    :parameters (?p - person ?from - cstage ?to - cstage)
    :precondition (and (stage ?p ?from) (next ?from ?to) (gate-free ?to))
    :effect (stage ?p ?to))
  (:action advance-to-connected
    :parameters (?p - person ?from - cstage ?to - cstage)
    :precondition (and (stage ?p ?from) (next ?from ?to) (needs-followup ?to)
                       (welcomed ?p) (followed-up ?p))
    :effect (stage ?p ?to))
  (:action advance-to-serving
    :parameters (?p - person ?from - cstage ?to - cstage)
    :precondition (and (stage ?p ?from) (next ?from ?to) (needs-group ?to)
                       (welcomed ?p) (followed-up ?p) (in-small-group ?p))
    :effect (stage ?p ?to))
  (:action advance-to-leading
    :parameters (?p - person ?from - cstage ?to - cstage)
    :precondition (and (stage ?p ?from) (next ?from ?to) (needs-care ?to)
                       (welcomed ?p) (followed-up ?p) (in-small-group ?p) (care-assigned ?p))
    :effect (stage ?p ?to)))

(define (problem church-fixture)
  (:domain church-operations)
  (:objects
    visitor-1 visitor-2 visitor-3 - person
    first-time returning connected serving leading - cstage)
  (:init
    (stage visitor-1 connected)
    (stage visitor-2 first-time)
    (stage visitor-3 first-time)
    (next first-time returning)
    (next returning connected)
    (next connected serving)
    (next serving leading)
    (gate-free returning)
    (needs-followup connected)
    (needs-group serving)
    (needs-care leading)
    (welcomed visitor-1)
    (followed-up visitor-1)
    (in-small-group visitor-1)
    (care-assigned visitor-1)
    (welcomed visitor-2)
    (in-small-group visitor-2)
    (care-assigned visitor-2))
  (:goal (stage visitor-1 leading)))
