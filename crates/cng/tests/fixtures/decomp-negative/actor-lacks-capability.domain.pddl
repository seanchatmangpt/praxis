; Doctrine sec. 13 negative fixture: ACTOR-LACKS-CAPABILITY.
; The only achiever of approved(?d) requires has-capability(?w), and no
; action ever grants a capability -- an actor without the admitted
; capability fact can never lawfully approve. Surfaces as CNG_R05
; UnsupportedConstruct: the relaxed-reachability grounder (PROJ-733)
; proves has-capability(?w) unreachable and prunes the only schema to zero
; ground actions before any search runs (see tests/cng_ipc_corpus.rs).
(define (domain actor-lacks-capability)
  (:requirements :strips)
  (:predicates (has-capability ?w) (pending ?d) (approved ?d))
  (:action approve
    :parameters (?w ?d)
    :precondition (and (has-capability ?w) (pending ?d))
    :effect (and (approved ?d) (not (pending ?d)))))
