;; PDDL Domain: lawobject-capability
;;
;; Defines the planning domain for CPhy LawObject lifecycle transitions.
;; This domain models:
;;   - Obligation types (Precondition, BlockingConstraint, EvidenceRequired)
;;   - Lifecycle stages (Raw → Validated → Admitted → Receipted)
;;   - Andon halt/override state machine
;;   - Chain hash and receipt constraints
;;   - Authority and capacity rules
;;
;; Use with a conforming PDDL planner (e.g., Fast Downward, OPTIC).
;; See PDDL_CAPABILITY_MODEL.md for detailed semantics.

(define (domain lawobject-capability)
  (:requirements :typing :adl)

  ;; =========================================================================
  ;; TYPES
  ;; =========================================================================

  (:types
    ;; Domain instances
    law-object

    ;; Obligation and evidence
    obligation
    evidence-type
    predicate

    ;; State enums
    andon-state
    lifecycle-stage

    ;; Agents and authorities
    validator
    authority

    ;; Chain and receipt
    chain-token
  )

  ;; =========================================================================
  ;; PREDICATES
  ;; =========================================================================

  (:predicates
    ;; =====================================================================
    ;; Lifecycle Stage Predicates
    ;; =====================================================================

    ;; (in-stage ?obj - law-object ?stage - lifecycle-stage)
    ;;   True if ?obj is currently in lifecycle stage ?stage.
    ;;   Stages: raw, validated, admitted, receipted.
    (in-stage ?obj - law-object ?stage - lifecycle-stage)

    ;; =====================================================================
    ;; Obligation Predicates
    ;; =====================================================================

    ;; (has-obligation ?obj - law-object ?ob - obligation)
    ;;   True if law-object ?obj carries obligation ?ob.
    (has-obligation ?obj - law-object ?ob - obligation)

    ;; (is-precondition ?ob - obligation ?pred - predicate)
    ;;   True if ?ob is a Precondition type obligation on predicate ?pred.
    (is-precondition ?ob - obligation ?pred - predicate)

    ;; (is-blocking-constraint ?ob - obligation)
    ;;   True if ?ob is a BlockingConstraint type obligation.
    (is-blocking-constraint ?ob - obligation)

    ;; (requires-evidence ?ob - obligation ?etype - evidence-type)
    ;;   True if ?ob is an EvidenceRequired type obligation for ?etype.
    (requires-evidence ?ob - obligation ?etype - evidence-type)

    ;; (precondition-satisfied ?pred - predicate)
    ;;   True if the predicate ?pred has been evaluated and passed.
    (precondition-satisfied ?pred - predicate)

    ;; (evidence-satisfied ?ob - obligation)
    ;;   True if evidence for obligation ?ob has been supplied and validated.
    (evidence-satisfied ?ob - obligation)

    ;; (blocking-constraint-cleared ?ob - obligation)
    ;;   True if the BlockingConstraint ?ob has been cleared (evidence provided,
    ;;   condition waived, or authority override applied).
    (blocking-constraint-cleared ?ob - obligation)

    ;; (obligation-unmet ?obj - law-object ?ob - obligation)
    ;;   True if obligation ?ob on ?obj is currently unmet.
    ;;   Contributes to Andon::Halted state.
    (obligation-unmet ?obj - law-object ?ob - obligation)

    ;; =====================================================================
    ;; Andon (Halt/Override) Predicates
    ;; =====================================================================

    ;; (andon-status ?obj - law-object ?state - andon-state)
    ;;   Current Andon status of ?obj: green, halted, or overridden.
    (andon-status ?obj - law-object ?state - andon-state)

    ;; (andon-holds ?obj - law-object)
    ;;   Shorthand: Andon is actively halting progress on ?obj.
    ;;   True iff andon-status is halted (not green or overridden).
    (andon-holds ?obj - law-object)

    ;; (andon-override-applied ?obj - law-object ?by - authority)
    ;;   True if an Andon hold on ?obj has been overridden by authority ?by.
    (andon-override-applied ?obj - law-object ?by - authority)

    ;; =====================================================================
    ;; Chain and Receipt Predicates
    ;; =====================================================================

    ;; (chain-hash-computed ?obj - law-object ?token - chain-token)
    ;;   True if chain hash for ?obj has been computed and stored in ?token.
    ;;   Only true for Receipted objects.
    (chain-hash-computed ?obj - law-object ?token - chain-token)

    ;; (prev-chain-valid ?token - chain-token)
    ;;   True if the previous chain hash (parent in receipt chain)
    ;;   is accessible and valid.
    (prev-chain-valid ?token - chain-token)

    ;; (signature-applied ?obj - law-object)
    ;;   True if Ed25519 signature has been applied to ?obj.
    ;;   Only set when receipted (if signed feature enabled).
    (signature-applied ?obj - law-object)

    ;; =====================================================================
    ;; Authority and Capability Predicates
    ;; =====================================================================

    ;; (validated-by ?obj - law-object ?validator - validator)
    ;;   Recorded validator that judged ?obj.
    (validated-by ?obj - law-object ?validator - validator)

    ;; (admitted-by ?obj - law-object ?authority - authority)
    ;;   Recorded authority that admitted ?obj.
    (admitted-by ?obj - law-object ?authority - authority)

    ;; (override-authority ?authority - authority ?ob - obligation)
    ;;   True if ?authority is permitted to override obligation ?ob.
    (override-authority ?authority - authority ?ob - obligation)
  )

  ;; =========================================================================
  ;; ACTIONS
  ;; =========================================================================

  ;; =====================================================================
  ;; Action 1: JUDGE (Raw → Validated)
  ;; =====================================================================
  ;;
  ;; Evaluates all obligations on a law-object in Raw stage.
  ;; Transitions to Validated if all obligations are satisfied.
  ;; Precondition failure leaves object in Raw with Andon::Halted.
  ;;
  ;; Semantics:
  ;;   - Checks that every obligation on ?obj has been satisfied:
  ;;     - Precondition: predicate evaluated to true
  ;;     - BlockingConstraint: evidence provided or waived
  ;;     - EvidenceRequired: evidence supplied
  ;;   - Sets Andon to Green
  ;;   - Records the validating agent

  (:action judge
    :parameters (
      ?obj - law-object
      ?validator - validator
    )
    :precondition (and
      ;; Object must be in Raw stage
      (in-stage ?obj raw)

      ;; All obligations must be satisfied
      ;; (universal quantification over obligations)
      (forall (?ob - obligation)
        (implies
          (has-obligation ?obj ?ob)
          (or
            ;; Precondition satisfied
            (and
              (is-precondition ?ob ?pred)
              (precondition-satisfied ?pred)
            )
            ;; Blocking constraint cleared
            (and
              (is-blocking-constraint ?ob)
              (blocking-constraint-cleared ?ob)
            )
            ;; Evidence provided
            (and
              (requires-evidence ?ob ?etype)
              (evidence-satisfied ?ob)
            )
          )
        )
      )
    )
    :effect (and
      ;; Transition to Validated stage
      (not (in-stage ?obj raw))
      (in-stage ?obj validated)

      ;; Record the validator
      (validated-by ?obj ?validator)

      ;; Clear Andon: all obligations met
      (andon-status ?obj green)
      (not (andon-holds ?obj))

      ;; Clear obligation-unmet flags
      (forall (?ob - obligation)
        (when (has-obligation ?obj ?ob)
          (not (obligation-unmet ?obj ?ob))
        )
      )
    )
  )

  ;; =====================================================================
  ;; Action 2: ADMIT (Validated → Admitted)
  ;; =====================================================================
  ;;
  ;; Transitions a Validated object to Admitted state.
  ;; Requires Andon to be Green (no blocks or overrides outstanding).
  ;;
  ;; Semantics:
  ;;   - Object must be Validated and Andon Green
  ;;   - Authority approves admission
  ;;   - Object becomes Admitted (ready for receipt)

  (:action admit
    :parameters (
      ?obj - law-object
      ?authority - authority
    )
    :precondition (and
      ;; Object must be Validated
      (in-stage ?obj validated)

      ;; Andon must be Green (no holds)
      (not (andon-holds ?obj))
      (andon-status ?obj green)
    )
    :effect (and
      ;; Transition to Admitted stage
      (not (in-stage ?obj validated))
      (in-stage ?obj admitted)

      ;; Record the admitting authority
      (admitted-by ?obj ?authority)
    )
  )

  ;; =====================================================================
  ;; Action 3: RECEIPT (Admitted → Receipted)
  ;; =====================================================================
  ;;
  ;; Computes chain hash and optionally applies signature,
  ;; transitioning Admitted → Receipted.
  ;;
  ;; Semantics:
  ;;   - Object must be Admitted
  ;;   - Previous chain hash must be valid and accessible
  ;;   - New chain token must be unique
  ;;   - Computes blake3(prev_hash || canonical_bytes(payload))
  ;;   - Applies signature if signed feature enabled
  ;;   - Seals the object (immutable, append-only chain)

  (:action receipt
    :parameters (
      ?obj - law-object
      ?prev-token - chain-token
      ?new-token - chain-token
    )
    :precondition (and
      ;; Object must be Admitted
      (in-stage ?obj admitted)

      ;; Previous chain hash must be valid
      (prev-chain-valid ?prev-token)

      ;; New token must not already be used
      (not (chain-hash-computed ?obj ?new-token))
    )
    :effect (and
      ;; Transition to Receipted stage
      (not (in-stage ?obj admitted))
      (in-stage ?obj receipted)

      ;; Compute and store chain hash
      (chain-hash-computed ?obj ?new-token)

      ;; Apply signature
      (signature-applied ?obj)
    )
  )

  ;; =====================================================================
  ;; Action 4: PROMOTE-ANDON (Halted → Overridden)
  ;; =====================================================================
  ;;
  ;; Overrides an Andon hold when authority waives or evidence emerges.
  ;; Transitions Andon::Halted → Andon::Overridden.
  ;;
  ;; Semantics:
  ;;   - Object must be Raw with Andon::Halted
  ;;   - An obligation must be currently unmet
  ;;   - Authority must have override permission for that obligation
  ;;   - Clears the obligation and changes Andon to Overridden
  ;;   - Object can now proceed to judgment (if other obligations met)

  (:action promote-andon
    :parameters (
      ?obj - law-object
      ?authority - authority
      ?ob - obligation
    )
    :precondition (and
      ;; Object must be Raw
      (in-stage ?obj raw)

      ;; Andon must be Halted
      (andon-holds ?obj)
      (andon-status ?obj halted)

      ;; Obligation must be currently unmet
      (obligation-unmet ?obj ?ob)

      ;; Authority must be permitted to override this obligation
      (override-authority ?authority ?ob)
    )
    :effect (and
      ;; Transition Andon: Halted → Overridden
      (not (andon-status ?obj halted))
      (andon-status ?obj overridden)

      ;; Record the override
      (andon-override-applied ?obj ?authority)

      ;; Clear the obligation
      (not (obligation-unmet ?obj ?ob))

      ;; Remove Andon hold (may be lifted if all other obligations met)
      (not (andon-holds ?obj))
    )
  )

  ;; =====================================================================
  ;; Action 5: SUPPLY-EVIDENCE (Satisfy EvidenceRequired)
  ;; =====================================================================
  ;;
  ;; External action: system receives evidence, satisfies an obligation.
  ;; Can occur at any stage (Raw, Validated, etc.).
  ;;
  ;; Semantics:
  ;;   - Object must carry an EvidenceRequired obligation
  ;;   - Evidence has not yet been supplied
  ;;   - Evidence supplier provides evidence of matching type
  ;;   - Obligation becomes satisfied; contributes to clearing Andon

  (:action supply-evidence
    :parameters (
      ?obj - law-object
      ?ob - obligation
      ?etype - evidence-type
    )
    :precondition (and
      ;; Object must have this obligation
      (has-obligation ?obj ?ob)

      ;; Obligation must require evidence
      (requires-evidence ?ob ?etype)

      ;; Evidence has not yet been supplied
      (not (evidence-satisfied ?ob))
    )
    :effect (and
      ;; Mark evidence as satisfied
      (evidence-satisfied ?ob)

      ;; Clear the obligation's unmet flag
      (not (obligation-unmet ?obj ?ob))
    )
  )

  ;; =====================================================================
  ;; Action 6: CLEAR-BLOCKING-CONSTRAINT (Satisfy BlockingConstraint)
  ;; =====================================================================
  ;;
  ;; External action: system clears a blocking constraint when evidence
  ;; arrives or authority waives the constraint.
  ;;
  ;; Semantics:
  ;;   - Object must carry a BlockingConstraint obligation
  ;;   - Constraint has not yet been cleared
  ;;   - Evidence or authority action clears it
  ;;   - Obligation becomes satisfied; contributes to clearing Andon

  (:action clear-blocking-constraint
    :parameters (
      ?obj - law-object
      ?ob - obligation
    )
    :precondition (and
      ;; Object must have this obligation
      (has-obligation ?obj ?ob)

      ;; Obligation must be a BlockingConstraint
      (is-blocking-constraint ?ob)

      ;; Constraint has not yet been cleared
      (not (blocking-constraint-cleared ?ob))
    )
    :effect (and
      ;; Mark constraint as cleared
      (blocking-constraint-cleared ?ob)

      ;; Clear the obligation's unmet flag
      (not (obligation-unmet ?obj ?ob))
    )
  )

  ;; =====================================================================
  ;; Action 7: CONFIRM-PREDICATE (Satisfy Precondition)
  ;; =====================================================================
  ;;
  ;; External action: system evaluates a predicate and confirms it passes.
  ;; Models the outcome of an external predicate check (e.g., signature verification,
  ;; balance check, etc.).
  ;;
  ;; Semantics:
  ;;   - Predicate must not yet be satisfied
  ;;   - External system or oracle confirms it passes
  ;;   - Predicate becomes satisfied globally (applies to all obligations on it)

  (:action confirm-predicate
    :parameters (
      ?pred - predicate
    )
    :precondition (
      (not (precondition-satisfied ?pred))
    )
    :effect (
      (precondition-satisfied ?pred)
    )
  )

)

;; =========================================================================
;; PROBLEM TEMPLATE: contract-claim-validation-case-001
;; =========================================================================
;;
;; This problem file illustrates a concrete scenario:
;; A smart contract claim arrives in Raw stage with two obligations:
;;   1. Precondition("signature_valid") — signature must be cryptographically valid
;;   2. EvidenceRequired("ledger_entry") — claim must reference a valid ledger entry
;;
;; Goal: Transition the claim from Raw → Receipted with valid chain hash.
;;
;; To use this problem:
;;   1. Save the domain (above) to lawobject-capability.pddl
;;   2. Save the problem (below) to contract-claim-validation-case-001.pddl
;;   3. Run: fast-downward.py lawobject-capability.pddl contract-claim-validation-case-001.pddl
;;
;; Expected plan (or similar):
;;   0: confirm-predicate sig-check
;;   1: supply-evidence claim-001 ob-ledger ledger-type
;;   2: judge claim-001 judge-service
;;   3: admit claim-001 admissions-authority
;;   4: receipt claim-001 chain-genesis chain-claim-001

(define (problem contract-claim-validation-case-001)
  (:domain lawobject-capability)

  (:objects
    ;; Law object (the contract claim under judgment)
    claim-001 - law-object

    ;; Obligations
    ob-signature - obligation
    ob-ledger - obligation

    ;; Predicates and evidence types
    sig-check - predicate
    ledger-type - evidence-type

    ;; Validators and authorities
    judge-service - validator
    admissions-authority - authority

    ;; Chain tokens
    chain-genesis - chain-token
    chain-claim-001 - chain-token
  )

  (:init
    ;; ===================================================================
    ;; Lifecycle Stage: initial Raw state
    ;; ===================================================================
    (in-stage claim-001 raw)

    ;; ===================================================================
    ;; Obligations: two unmet obligations
    ;; ===================================================================
    (has-obligation claim-001 ob-signature)
    (has-obligation claim-001 ob-ledger)

    ;; Obligation 1: Precondition on signature validity
    (is-precondition ob-signature sig-check)
    (not (precondition-satisfied sig-check))
    (obligation-unmet claim-001 ob-signature)

    ;; Obligation 2: Evidence required for ledger entry
    (requires-evidence ob-ledger ledger-type)
    (not (evidence-satisfied ob-ledger))
    (obligation-unmet claim-001 ob-ledger)

    ;; ===================================================================
    ;; Andon: initially Halted (obligations unmet)
    ;; ===================================================================
    (andon-status claim-001 halted)
    (andon-holds claim-001)

    ;; ===================================================================
    ;; Authority relationships
    ;; ===================================================================
    ;; (admissions-authority cannot override these in this scenario,
    ;;  but could if the precondition were a blocking constraint)

    ;; ===================================================================
    ;; Chain: genesis token is valid and accessible
    ;; ===================================================================
    (prev-chain-valid chain-genesis)
  )

  (:goal (and
    ;; Goal 1: claim is Receipted
    (in-stage claim-001 receipted)

    ;; Goal 2: Andon is Green (all obligations satisfied)
    (andon-status claim-001 green)

    ;; Goal 3: chain hash is computed
    (chain-hash-computed claim-001 chain-claim-001)
  ))
)
