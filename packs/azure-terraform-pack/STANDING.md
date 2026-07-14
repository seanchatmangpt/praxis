# Azure/Terraform Pack — Standing Reconciliation (Iteration 1 of 4)

Mirrors `packs/ma-case-study-pack/STANDING.md`'s claim-table shape, scoped to this pack. Task
#143 caps this Azure/Terraform + M&A workstream at 4 hourly, session-only iterations; this is
iteration 1. Per `docs/releases/v26.7.14/THESIS.md` Section 33.12, the M&A case study overall
remains **PLANNED**, and this document does not round that up — it records what this iteration
built and verified, and states what it explicitly did not do.

## What is real this iteration

- **A real external Terraform module was fetched, read, and adapted with disclosed
  attribution.** `Azure/terraform-azurerm-avm-res-containerinstance-containergroup` (MIT,
  Microsoft-co-maintained) was fetched directly (`main.tf`/`variables.tf`/`outputs.tf`/
  `LICENSE` via `raw.githubusercontent.com`) and its `azurerm_container_group` resource shape
  (argument names, nested `container` block) was adapted verbatim into
  `templates/main.tf.tmpl`/`variables.tf.tmpl`/`outputs.tf.tmpl`. Its `tests/` directory ships
  zero `.tftest.hcl` files (confirmed via GitHub tree API — a placeholder README only), so the
  test-harness *shape* (`mock_provider "azurerm" {}` + `command = plan` + `assert` blocks) was
  instead adapted from `Azure/terraform-azurerm-avm-res-keyvault-vault`'s
  `tests/unit/*.tftest.hcl` (also MIT, also fetched directly), whose native-`azurerm`-resource
  mock pattern transfers because `azurerm_container_group`, like `azurerm_key_vault`, is a
  native `azurerm` resource, not `azapi`. Full ledger, including what was deliberately **not**
  ported and why (RBAC/`azurerm_role_assignment`, `dynamic "container"`, DNS/telemetry/
  Key-Vault-key/probe/private-endpoint arguments — none grounded in the crown-bribery-case
  chain's actual code shape): `packs/azure-terraform-pack/SOURCE_ATTRIBUTION.md`.
- **The pack's ontology/shapes/templates render deterministically and validate.**
  `ggen graph validate --files packs/azure-terraform-pack/ontology.ttl --shapes
  packs/azure-terraform-pack/shapes.ttl` reports `shapes_conform: true` (re-run this iteration:
  96 quads, hash `96af818a02...`). Two independent `ggen sync run` invocations from fresh
  scratch consumer directories (fresh symlink, fresh `ggen.toml`, fresh `ggen.lock` each time)
  produced byte-identical SHA-256 hashes across all generated `.tf`/`.tftest.hcl` files and an
  identical `graph_hash_hex` — regeneration is deterministic, not a one-off.
- **Generated Terraform passes `terraform validate` and `terraform test` with mocked
  providers, re-confirmed this iteration.** In `deploy/azure/ma-case-study/` (real HashiCorp
  Terraform CLI, provider `hashicorp/azurerm` v3.117.1, no Azure credentials involved):
  `terraform validate` → `Success! The configuration is valid.` `terraform test` → `run
  "container_group_topology"... pass` / `Success! 1 passed, 0 failed.`, covering 8 real
  assertions (resource-group name, `os_type == "Linux"`, `restart_policy == "Never"`,
  `ip_address_type == "None"`, container name/cpu/memory/command) against a
  `mock_provider "azurerm" {}` plan. A sibling crown-bribery-case deployment generated from the
  same pack/graph in the same scratch `ggen sync run` (not committed to `deploy/` — only
  `ma-case-study` is tracked in this repo, per `git ls-files deploy/`) was independently
  re-confirmed the same way, showing the fix is general to the pack's templates, not specific
  to one row.
- **The pack was generalized from one embedded instance to a multi-deployment vocabulary.**
  `aztf:deploymentSlug`/`aztf:workloadLabel` were added to `ontology.ttl` so more than one
  `aztf:ContainerGroup` individual can coexist in one graph, each routed by its own `to:`
  frontmatter to its own `deploy/azure/<slug>/` directory. `instances/ma-case-deployment.ttl`
  is the second individual, grounded in `packs/ma-case-study-pack/STANDING.md`'s real M&A-C2
  claim (a `ggen graph validate` SHACL check that already runs and passes) rather than in the
  still-PLANNED M&A-C4/M&A-C5 PDDL8/POWL/Erlang chain — it containerizes the compliance-check
  command that exists today, not a speculative future pipeline. Resource sizing (`aztf:cpuCores
  1.0` / `aztf:memoryGb 1.0`) is Azure Container Instances' documented platform floor (verified
  against Microsoft's own resource-and-quota-limits page this session), not a load-tested
  requirement — the measured validate-command workload (`/usr/bin/time -l`: 0.01s, ~14.9 MiB
  peak RSS) sits roughly 68x under that floor.
- **Three real template bugs were found and fixed during generalization**, not hidden after
  the fact: (1) `main.tf.tmpl` referenced a Tera variable one line before it was `{%- set -%}`,
  breaking generation for any graph with ≥1 `aztf:ContainerGroup`; (2) a trailing `-%}` trim
  marker on that same `set` tag ate the newline after it, merging a header comment onto the
  first line of real HCL and producing invalid syntax (`terraform init`/`validate` failed with
  "Argument or block definition required"); (3) the test template looped over the
  graph-wide (unfiltered) container array, so a second deployment's rendered assertion
  referenced `container[1]` when its own `main.tf` only ever creates `container[0]` — this
  would have failed `terraform test` at plan time had it not been caught first. All three are
  fixed in the tracked templates and re-verified via the `terraform validate`/`terraform test`
  runs cited above.
- **A real MFW PDDL8 model orders the provisioning sequence.** `pddl-domain.ttl` +
  `pddl-problem-plannable.ttl`/`pddl-problem-blocked.ttl` manufacture real PDDL8 text
  (`my_conforming_project::mfg::manufacture`) and ground+solve
  (`bcinr_pddl::GroundProblem::find_plan`) via
  `crates/multifractal-workflow/tests/azure_terraform_pddl.rs`, re-run this iteration:
  `cargo test -p multifractal-workflow --test azure_terraform_pddl` → 3 passed, 0 failed. The
  plannable scenario solves to a real 12-step ordered plan (init-provider →
  bind-container-image-variable → ... → run-plan-test); the blocked scenario (missing container
  image) solves to a 2-step plan reaching `(deployment-blocked ...)`, not
  `plan-verified` — the domain models a **provisioning-order sequence**, not deal-progression
  or M&A-specific planning, and does not invoke `terraform apply` anywhere.

## What is explicitly NOT real this iteration

- **`terraform apply` was never run.** No real Azure resource — container group, resource
  group, or otherwise — exists anywhere as a result of this iteration's work. Every
  `terraform`/`ggen` command run this iteration was `validate`, `test` (mocked provider), `fmt
  -check`, or `graph validate` — none touch a live subscription.
- **No Azure credentials, service principal, or subscription were configured.**
  `terraform init -backend=false` downloaded the `azurerm` provider *binary* only (build-time
  plugin fetch, not a live-account connection); nothing in this iteration authenticated to
  Azure.
- **The PDDL8 provisioning-order model is not the M&A deal-progression model.** It sequences
  *how Terraform resources get created* (provider init → variable binding → resource creation →
  output binding → test), grounded in the crown-bribery-case/generalized-pack topology — it is
  not `packs/ma-case-study-pack/STANDING.md`'s still-PLANNED M&A-C4 (PDDL8 deal-progression
  planning over the 6 concurrent M&A processes). Naming both "PDDL8" invites conflation; they
  are different domains solving different problems.
- **`instances/ma-case-deployment.ttl` deploys a compliance-check batch job, not "the M&A
  case study."** It containerizes one `ggen graph validate` SHACL check (M&A-C2, already real)
  as a run-to-completion job. It is not evidence toward M&A-C4/M&A-C5/M&A-C6 (PDDL8
  deal-progression, POWL v2 + Arazzo + Erlang/OTP dispatch, multi-party Little's Law
  observation) — those remain zero files, per `packs/ma-case-study-pack/STANDING.md`.
- **`just multifractal-workflow-clippy-test-bin azure_terraform_pddl` was not run to a clean
  pass.** It fails to compile because the crate's `lib` target carries 39 pre-existing clippy
  errors in unrelated files (`crown_external.rs`, `crown_local.rs`,
  `f29_capability_roadmap.rs`) confirmed via `git status --porcelain` to predate this iteration
  (zero diff against those files). Not fixed; out of scope for this iteration.
- **No `just verify-all`/`just test-changed`/`just clippy` full-repo gate was run.** The repo
  carries 100+ files changed by concurrent work outside this iteration's scope at the time of
  writing (`git status --porcelain` at iteration start); a full-repo run would not isolate
  signal for this change. Scoped commands only (cited above) were run.

## Relationship to concurrent work in this repo

At the time of this commit, `git status` showed ~100 files modified/untracked outside this
iteration's scope (including a separate, further-along multi-org-merge workstream —
`crates/multifractal-workflow/src/f31_org_merge.rs`,
`crates/multifractal-workflow/tests/ma_org_merge.rs`,
`crates/multifractal-workflow/src/bin/crown-local-cli.rs` — referenced by task #143's redefined
scope but not authored by, or claimed by, this iteration). This iteration's commit stages only
the files it authored or the isolated hunks it is responsible for (notably: a single `justfile`
recipe hunk, `multifractal-workflow-test-azure-terraform-pddl`, staged via a hand-built patch
against the index so the concurrent workstream's own pending `justfile` hunks — `install-cng`,
`cng-clippy-lib-isolated`, `praxis-graphlaw-test-all-isolated`,
`multifractal-workflow-clippy-test-bin`, `multifractal-workflow-run-crown-local-cli`,
`multifractal-workflow-test-org-merge` — remain uncommitted in the working tree, untouched, for
that other work to commit on its own terms). No unrelated file was added, reverted, or
force-overwritten by this iteration.

## See Also

- `packs/azure-terraform-pack/SOURCE_ATTRIBUTION.md` — full URL/license/adapted-vs-custom
  ledger for the Terraform module boilerplate
- `packs/azure-terraform-pack/pack.toml` — topology grounding (why one container, no ingress,
  no volume, no Key Vault) cited against `crates/multifractal-workflow/src/
  f15_air_transition_core/bridge.rs` and `f16_otp_runner/bridge.rs`
- `packs/ma-case-study-pack/STANDING.md` — the M&A-C1..C6 claim table this pack's
  `instances/ma-case-deployment.ttl` is grounded against (M&A-C2 real; M&A-C4..C6 PLANNED)
- `docs/releases/v26.7.14/THESIS.md` Section 33.12 — the source-of-truth PLANNED ruling for
  the M&A case study as a whole
- `docs/releases/v26.7.14/RELEASE_CONTROL.md` Sec. 5, item 5 — the milestone-level open-items
  register entry this iteration's progress updates
- `crates/multifractal-workflow/tests/azure_terraform_pddl.rs` — the PDDL8
  provisioning-order-sequence test evidence cited above
