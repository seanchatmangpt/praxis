(define (domain kitchen-two-chain)
  (:requirements :strips)
  (:predicates
    (in-pantry ?x)
    (in-drawer ?x)
    (held ?x)
    (cooked ?x)
    (placed ?x))
  (:action fetch-pantry
    :parameters (?x)
    :precondition (in-pantry ?x)
    :effect (and (held ?x) (not (in-pantry ?x))))
  (:action fetch-drawer
    :parameters (?x)
    :precondition (in-drawer ?x)
    :effect (and (held ?x) (not (in-drawer ?x))))
  (:action cook
    :parameters (?x)
    :precondition (held ?x)
    :effect (and (cooked ?x) (not (held ?x))))
  (:action place
    :parameters (?x)
    :precondition (held ?x)
    :effect (and (placed ?x) (not (held ?x))))
)
