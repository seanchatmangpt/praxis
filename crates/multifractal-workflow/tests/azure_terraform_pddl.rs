//! Stage-1 live verification that the Azure infrastructure PROVISIONING
//! SEQUENCE PDDL8 domain (`packs/azure-terraform-pack/pddl-domain.ttl`) is
//! not a hand-authored resemblance of a generated artifact -- it IS one.
//! Mirrors `/Users/sac/praxis/tests/bribery_case_pddl.rs`'s own real,
//! already-proven pipeline exactly:
//!   `my_conforming_project::mfg::manufacture` (SPARQL extraction +
//!   `enforce_pddl8` bound checking + PDDL8 text emission) ->
//!   `my_conforming_project::mfg::validate` (real
//!   `bcinr_pddl::domain_from_pddl` / `problem_from_pddl` /
//!   `GroundProblem::build` / `GroundProblem::find_plan`).
//!
//! `my_conforming_project` is already a real dependency of this crate
//! (`Cargo.toml`'s `my-conforming-project = { path = "../..", features =
//! ["ggen"] }`, added for the `crown-bribery-case` bin's own
//! `mfg::manufacture` call) -- unlike the root crate's own
//! `tests/bribery_case_pddl.rs`, which gates on `#![cfg(feature = "ggen")]`
//! because `ggen` is an *optional* feature of the root package itself, this
//! test needs no such gate: the dependency edge above always requests
//! `features = ["ggen"]`, unconditionally.
//!
//! Two scenarios, both genuinely ground+solve:
//!   1. `pddl-problem-plannable.ttl`: all 4 required Terraform input
//!      variables (`location`/`environment`/`resource_group_name`/
//!      `container_image` -- `deploy/azure/ma-case-study/variables.tf`)
//!      are bound -> the deployment reaches `(plan-verified ...)`, the
//!      mocked-provider `terraform test` plan-check state.
//!   2. `pddl-problem-blocked.ttl`: the `container_image` variable's value
//!      is positively known unavailable (no Dockerfile exists yet in this
//!      repo -- `deploy/azure/ma-case-study/variables.tf:50`'s own
//!      disclosure) -> the deployment reaches `(deployment-blocked ...)`,
//!      a typed non-closure terminal state distinct from a verified plan.
//!
//! # Scope (explicit, not implied)
//! This test proves the PDDL8 domain manufactures and solves to a real,
//! ordered plan -- it does **not** invoke `terraform apply` (or `plan`/
//! `init`/`test`) for real, and makes no claim that it does. See
//! `packs/azure-terraform-pack/pddl-domain.ttl`'s own header for the exact
//! generated-Terraform-file fact each action represents.

const DOMAIN_TTL: &str = include_str!("../../../packs/azure-terraform-pack/pddl-domain.ttl");
const PROBLEM_PLANNABLE_TTL: &str =
    include_str!("../../../packs/azure-terraform-pack/pddl-problem-plannable.ttl");
const PROBLEM_BLOCKED_TTL: &str =
    include_str!("../../../packs/azure-terraform-pack/pddl-problem-blocked.ttl");

/// STRIPS8 bound check by inspection (cross-checked against the same
/// `PDDL8_MAX_*` constants `ontology/lawobject.ttl` and bribery-case's own
/// `pddl-domain.ttl` document): every action in this domain stays at <= 4
/// params / <= 4 precondition conjuncts; every atom used stays at <= 2
/// args. All well under `PDDL8_MAX_ARITY` / `PDDL8_MAX_CONJUNCTS` /
/// `PDDL8_MAX_PARAMS` = 8. `mfg::enforce_pddl8` (called inside
/// `mfg::manufacture` below) is the LIVE re-check of this claim -- if it
/// were wrong, `manufacture()` would return `Err(MfgError::BoundExceeded)`
/// and this test would fail at the `.expect(...)` below, not silently
/// pass.
#[test]
fn domain_is_strips8_safe_and_manufactures_real_pddl8_text() {
    // `mfg::manufacture` extracts a domain AND a problem from one graph
    // (`extract_problem` requires exactly one `pdl:Problem` instance), so
    // this bound-checking test concatenates the domain with one problem
    // file (arbitrarily, the plannable one) purely to give it a Problem to
    // extract -- the assertions below are about the DOMAIN text only.
    let combined = format!("{DOMAIN_TTL}\n{PROBLEM_PLANNABLE_TTL}");
    let manufactured = my_conforming_project::mfg::manufacture(
        &combined,
        "packs/azure-terraform-pack/pddl-domain.ttl",
    )
    .expect("pddl-domain.ttl must satisfy PDDL8 bounds (arity/conjuncts/params <= 8)");
    for action_name in [
        "init-provider",
        "bind-location-variable",
        "bind-environment-variable",
        "bind-resource-group-name-variable",
        "bind-container-image-variable",
        "confirm-variables-ready",
        "create-resource-group",
        "define-container-spec",
        "create-container-group",
        "bind-outputs",
        "prepare-mock-provider",
        "run-plan-test",
        "block-for-missing-image",
    ] {
        assert!(
            manufactured.domain_text.contains(action_name),
            "manufactured domain text must declare action {action_name:?}: {}",
            manufactured.domain_text
        );
    }
    eprintln!("manufactured domain text:\n{}", manufactured.domain_text);
}

/// Scenario 1: lawful provisioning sequence. Concatenates pddl-domain.ttl
/// with pddl-problem-plannable.ttl, manufactures real PDDL8 text, and
/// calls the REAL `bcinr_pddl` grounder+solver -- not a mock.
#[test]
fn plannable_deployment_grounds_and_solves_to_plan_verified() {
    let combined = format!("{DOMAIN_TTL}\n{PROBLEM_PLANNABLE_TTL}");
    let manufactured = my_conforming_project::mfg::manufacture(
        &combined,
        "azure-terraform-pack (plannable scenario)",
    )
    .expect("domain+plannable-problem must manufacture");
    eprintln!("manufactured problem text:\n{}", manufactured.problem_text);

    let report =
        my_conforming_project::mfg::validate(&manufactured.domain_text, &manufactured.problem_text);
    assert!(
        report.parsed,
        "must round-trip through bcinr-pddl's parser: {:?}",
        report.error
    );
    assert!(
        report.solvable,
        "GroundProblem::find_plan must find a real plan reaching plan-verified: {:?}",
        report.error
    );
    eprintln!(
        "REAL bcinr_pddl plan ({} steps): {:?}",
        report.plan_len, report.plan_steps
    );

    // The plan must: initialize the provider, bind all 4 required
    // variables (one distinct bind-*-variable action per variable), confirm
    // them ready, create the resource group and define the container spec,
    // create the container group, bind outputs, prepare the mocked
    // provider, then verify the plan -- 12 actions total, ending in exactly
    // `run-plan-test`.
    assert_eq!(report.plan_len, 12, "plan: {:?}", report.plan_steps);
    assert_eq!(
        report.plan_steps.last().map(String::as_str),
        Some("run-plan-test")
    );
    for action_name in [
        "init-provider",
        "bind-location-variable",
        "bind-environment-variable",
        "bind-resource-group-name-variable",
        "bind-container-image-variable",
        "confirm-variables-ready",
        "create-resource-group",
        "define-container-spec",
        "create-container-group",
        "bind-outputs",
        "prepare-mock-provider",
        "run-plan-test",
    ] {
        assert_eq!(
            report
                .plan_steps
                .iter()
                .filter(|s| s.as_str() == action_name)
                .count(),
            1,
            "expected exactly one {action_name} in the plan: {:?}",
            report.plan_steps
        );
    }
    // block-for-missing-image must NEVER appear in a lawful plannable plan.
    assert!(!report
        .plan_steps
        .contains(&"block-for-missing-image".to_string()));

    // Real Terraform dependency-order invariants (packs/azure-terraform-pack/
    // pddl-domain.ttl's own header cites the exact main.tf/outputs.tf/
    // tests/container_group.tftest.hcl lines each ordering constraint
    // represents):
    let pos = |name: &str| -> usize {
        report
            .plan_steps
            .iter()
            .position(|s| s == name)
            .unwrap_or_else(|| panic!("{name} must appear in the plan: {:?}", report.plan_steps))
    };
    let init_provider = pos("init-provider");
    let confirm_ready = pos("confirm-variables-ready");
    let create_rg = pos("create-resource-group");
    let define_container = pos("define-container-spec");
    let create_cg = pos("create-container-group");
    let bind_outputs = pos("bind-outputs");
    let prepare_mock = pos("prepare-mock-provider");
    let run_test = pos("run-plan-test");

    // terraform init (init-provider) precedes every variable bind.
    for var_action in [
        "bind-location-variable",
        "bind-environment-variable",
        "bind-resource-group-name-variable",
        "bind-container-image-variable",
    ] {
        assert!(
            init_provider < pos(var_action),
            "init-provider must precede {var_action}: {:?}",
            report.plan_steps
        );
        assert!(
            pos(var_action) < confirm_ready,
            "{var_action} must precede confirm-variables-ready: {:?}",
            report.plan_steps
        );
    }
    // main.tf:39-42 (resource group) and main.tf:45-58's container block
    // both require variables-ready first.
    assert!(confirm_ready < create_rg, "{:?}", report.plan_steps);
    assert!(confirm_ready < define_container, "{:?}", report.plan_steps);
    // main.tf:47-48's real attribute references force resource-group
    // before container-group.
    assert!(create_rg < create_cg, "{:?}", report.plan_steps);
    assert!(define_container < create_cg, "{:?}", report.plan_steps);
    // outputs.tf reads azurerm_container_group.this.*/azurerm_resource_group.this.*.
    assert!(create_cg < bind_outputs, "{:?}", report.plan_steps);
    // tests/container_group.tftest.hcl needs the mocked provider and the
    // planned container group (transitively, its bound outputs) before it
    // can run its plan-only assertions.
    assert!(init_provider < prepare_mock, "{:?}", report.plan_steps);
    assert!(prepare_mock < run_test, "{:?}", report.plan_steps);
    assert!(bind_outputs < run_test, "{:?}", report.plan_steps);
}

/// Scenario 2: the typed non-closure path. The `container_image` variable
/// is positively unavailable -- the REAL solver must reach `blocked`, not
/// `plan-verified`, and must do so WITHOUT ever invoking
/// create-resource-group/create-container-group/run-plan-test (proving
/// `deployment-blocked` is a genuine sibling terminal state, not a renamed
/// step on the plannable path).
#[test]
fn blocked_deployment_grounds_and_solves_to_blocked_not_plan_verified() {
    let combined = format!("{DOMAIN_TTL}\n{PROBLEM_BLOCKED_TTL}");
    let manufactured = my_conforming_project::mfg::manufacture(
        &combined,
        "azure-terraform-pack (blocked scenario)",
    )
    .expect("domain+blocked-problem must manufacture");
    eprintln!("manufactured problem text:\n{}", manufactured.problem_text);

    let report =
        my_conforming_project::mfg::validate(&manufactured.domain_text, &manufactured.problem_text);
    assert!(
        report.parsed,
        "must round-trip through bcinr-pddl's parser: {:?}",
        report.error
    );
    assert!(
        report.solvable,
        "GroundProblem::find_plan must find a real plan reaching blocked: {:?}",
        report.error
    );
    eprintln!(
        "REAL bcinr_pddl plan ({} steps): {:?}",
        report.plan_len, report.plan_steps
    );

    assert_eq!(
        report.plan_steps,
        vec![
            "init-provider".to_string(),
            "block-for-missing-image".to_string()
        ],
        "the blocked scenario's shortest real plan is exactly: init-provider (providers.tf's \
         required_providers block must still be initialized before any planning action, \
         including the block path), then block-for-missing-image"
    );
    for lawful_action in [
        "create-resource-group",
        "define-container-spec",
        "create-container-group",
        "bind-outputs",
        "prepare-mock-provider",
        "run-plan-test",
    ] {
        assert!(
            !report.plan_steps.contains(&lawful_action.to_string()),
            "the blocked path must never pass through {lawful_action}"
        );
    }
}
