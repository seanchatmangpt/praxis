; Doctrine sec. 13 negative fixture: HELPER-UNREACHABLE (problem half).
; Goal names delivered(c2) but init only places c1; no action ever adds
; at(c2), so the delivered(c2) subgoal is unreachable for any actor.
(define (problem helper-unreachable-1)
  (:domain helper-unreachable)
  (:objects c1 c2)
  (:init (at c1))
  (:goal (and (delivered c1) (delivered c2)))
)
