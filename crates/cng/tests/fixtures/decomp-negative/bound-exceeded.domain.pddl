; Doctrine sec. 13 negative fixture: DEPTH/COST-BOUND-EXCEEDED.
; A 5-parameter schema over the 8 objects of the problem half grounds to
; 8^5 = 32768 actions, above DECOMP_MAX_GROUND = 16384. Every parameter is
; independently reachability-constrained by (marked ?x), and every object
; is marked in init (PROJ-733: the relaxed-reachability grounder prunes
; groundings whose precondition atoms are unreachable, so a bound-exceeded
; fixture must make the REACHABLE set itself exceed the cap, not merely the
; naive raw cross-product -- a single-param precondition here would let
; pruning legitimately shrink the reachable set to 8^4=4096, under the
; cap, and the bound would never fire). The pipeline must refuse the bound
; loudly (CNG_R05, grounding failed) rather than attempt an unbounded
; search (see tests/cng_ipc_corpus.rs).
(define (domain bound-exceeded)
  (:requirements :strips)
  (:predicates (marked ?x) (done ?a ?b ?c ?d ?e))
  (:action combine
    :parameters (?a ?b ?c ?d ?e)
    :precondition (and (marked ?a) (marked ?b) (marked ?c) (marked ?d) (marked ?e))
    :effect (and (done ?a ?b ?c ?d ?e))))
