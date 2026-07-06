(define (problem autonomic-standing-factory-case-001)
  (:domain autonomic-standing-factory-pddl8)
  (:objects
    art-pddl-plan - evidence-artifact
    autonomic-standing-factory-local-first - claim
    bench-case-study - benchmark-report
    chain-case-study - receipt-chain
    client-autonomic-platform - client-surface
    eff-cargo-cicd-receipts - external-side-effect
    env-case-study - standing-envelope
    judg-case-study - graphlaw-judgment
    log-case-study - ocel-log
    repo-praxis - repo
    val-wasm4pm - process-validation
  )
  (:init
    (claim-promoted autonomic-standing-factory-local-first)
    (external-side-effect-open eff-cargo-cicd-receipts)
    (has-evidence repo-praxis)
  )
  (:goal (ready-for-scope autonomic-standing-factory-local-first))
)
