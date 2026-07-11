; Doctrine sec. 13 negative fixture: DEPTH/COST-BOUND-EXCEEDED (problem
; half). Exactly 8 objects, all marked: 8^5 = 32768 ground `combine`
; actions > 16384, and none of it is prunable (every object satisfies
; every parameter's precondition, so relaxed-reachability grounding
; reaches the same full cross-product as naive grounding).
(define (problem bound-exceeded-1)
  (:domain bound-exceeded)
  (:objects o1 o2 o3 o4 o5 o6 o7 o8)
  (:init (marked o1) (marked o2) (marked o3) (marked o4) (marked o5) (marked o6) (marked o7) (marked o8))
  (:goal (and (done o1 o1 o1 o1 o1)))
)
