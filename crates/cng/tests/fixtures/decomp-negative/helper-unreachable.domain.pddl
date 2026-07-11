; Doctrine sec. 13 negative fixture: HELPER-UNREACHABLE.
; The delivery chain load -> deliver only works for a crate that is `at`
; somewhere; crate c2 never is, so any helper subproblem carrying
; delivered(c2) is unsolvable. Because the decomposition pipeline plans the
; single-actor candidate #0 first, this surfaces as CNG_R04 PlanUnsolvable
; at the whole-problem gate (see tests/cng_ipc_corpus.rs).
(define (domain helper-unreachable)
  (:requirements :strips)
  (:predicates (at ?x) (loaded ?x) (delivered ?x))
  (:action load
    :parameters (?c)
    :precondition (and (at ?c))
    :effect (and (loaded ?c) (not (at ?c))))
  (:action deliver
    :parameters (?c)
    :precondition (and (loaded ?c))
    :effect (and (delivered ?c) (not (loaded ?c)))))
