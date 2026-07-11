; Doctrine sec. 13 negative fixture: ACTOR-LACKS-CAPABILITY (problem half).
; worker1 has no has-capability fact in init, so approved(doc1) is
; unreachable.
(define (problem actor-lacks-capability-1)
  (:domain actor-lacks-capability)
  (:objects worker1 doc1)
  (:init (pending doc1))
  (:goal (and (approved doc1)))
)
