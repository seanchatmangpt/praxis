# SOURCE_ATTRIBUTION.md

Real-source adaptation ledger for `packs/azure-terraform-pack/`, per this repo's
public-source-first discipline (established for RDF ontologies, applied here to Terraform
HCL/module boilerplate). Every URL below was fetched and read directly this session (`curl` to
`raw.githubusercontent.com`, or the GitHub tree/API for repo structure) — nothing here is
summarized from memory or a blog post.

## Sources

### 1. Azure/terraform-azurerm-avm-res-containerinstance-containergroup

- URL: `https://github.com/Azure/terraform-azurerm-avm-res-containerinstance-containergroup`
- Files fetched: `main.tf`, `variables.tf`, `outputs.tf`, `LICENSE` (all from the `main` branch,
  via `raw.githubusercontent.com`)
- License: MIT (Microsoft Corporation) — full text fetched and confirmed this session.
- Coverage: compute only. Wraps native `azurerm_container_group` (`resource
  "azurerm_container_group" "this"`, `main.tf` line 1) plus `azurerm_role_assignment`.
- Why selected: it is the closest real, Microsoft-co-maintained module to this repo's actual
  deployment topology — a single container group, no ingress/listener, no autoscale — the
  topology `crown-bribery-case.rs` and the two `bridge.rs` files (see below) actually require.
- **What was adapted verbatim (argument/attribute names, structural shape):**
  - `main.tf`: the `azurerm_container_group` resource's top-level argument names (`name`,
    `location`, `resource_group_name`, `os_type`, `restart_policy`, `ip_address_type`, `tags`),
    the nested `container` block's argument names (`name`, `image`, `cpu`, `memory`,
    `commands`), and the `timeouts { create = "2h" update = "2h" }` block —
    `templates/main.tf.tmpl`.
  - `variables.tf`: the `location`/`name`/`resource_group_name`/`tags` variable
    `type`/`description`/`nullable` shape — `templates/variables.tf.tmpl`.
  - `outputs.tf`: all 5 output names and their underlying attribute references (`fqdn`,
    `ip_address`, `name`, `resource_group_name`, `resource_id`) — `templates/outputs.tf.tmpl`.
- **What was deliberately NOT ported, and why (disclosed, not silently dropped):**
  - `azurerm_role_assignment "this"` (`main.tf` lines 160–171): no managed-identity/RBAC
    requirement is evidenced anywhere in the crown-bribery-case chain.
  - The `dynamic "container"` HCL construct (`main.tf` lines 17–108): that module takes an
    arbitrary runtime `containers` map(object) **Terraform variable** and renders it with a
    Terraform-native `dynamic` block, because it is a general-purpose reusable module whose
    container topology is a caller-supplied *runtime* value. This pack's container topology is
    instead a *generate-time* fact (one Rust+Erlang runtime container, fixed by
    `ontology.ttl`), so `templates/main.tf.tmpl` loops in **Tera** at generation time
    (`{% for c in b_containers %}`) instead of in HCL `dynamic` at plan/apply time. The rendered
    output still declares one literal `container { ... }` block per ontology-derived container
    — structurally equivalent HCL, produced by a different templating layer.
  - `enable_telemetry`, `dns_name_label`, `dns_name_label_reuse_policy`, `dns_name_servers`,
    `diagnostics_log_analytics`, `exposed_ports`, `image_registry_credential`,
    `key_vault_key_id`, `key_vault_user_assigned_identity_id`, `liveness_probe`,
    `readiness_probe`, `managed_identities`, `priority`, `private_endpoints`, `subnet_ids`,
    `zones`: none of these have a grounded requirement in the crown-bribery-case chain (see
    `ontology.ttl`'s own "SCOPE FENCE" section for the specific evidence — no listener, no
    external secret, no HA/autoscale evidence, no confidential-computing requirement). Pulling
    these in would be modeling infrastructure the code does not need.
  - `one(azurerm_container_group.this[*].attr)` in `outputs.tf`: that wrapper exists because the
    AVM module's resource may be conditionally created (a `count`/`for_each` pattern used
    elsewhere in the shared AVM template scaffold). This configuration's
    `azurerm_container_group.this` carries no `count`/`for_each` — it is always created — so
    `templates/outputs.tf.tmpl` uses the plain `azurerm_container_group.this.attr` form instead.
  - The module itself is not consumed as a Terraform `module "..." { source = ... }` block —
    its resource *shape* is adapted into native resources this pack's own templates render,
    because the actual deployment need (one fixed, ontology-derived topology, not an arbitrary
    reusable multi-caller module) does not call for the indirection of a wrapped child module.

### 2. Azure/terraform-azurerm-avm-res-keyvault-vault

- URL: `https://github.com/Azure/terraform-azurerm-avm-res-keyvault-vault`
- Files fetched: `tests/unit/unit.tftest.hcl`, `tests/unit/access_policies.tftest.hcl` (`main`
  branch, via `raw.githubusercontent.com`)
- License: MIT (Microsoft Corporation).
- Coverage: secrets only (Key Vault) — **not used for its resource content**, only for its
  **test harness shape**, because:
  - The Container Instance module (source 1) ships **zero** `.tftest.hcl` files — its `tests/`
    directory contains only `tests/README.md`, whose entire content is the placeholder line
    "Create tests in the provided subdirectories." (confirmed via the GitHub tree API this
    session; zero `.tftest.hcl` matches anywhere in that repo).
  - Key Vault AVM's tests use `mock_provider "azurerm" {}` (plus `modtm`/`random`/`time` mocks
    as needed) with `command = plan`, against **native azurerm resources**
    (`azurerm_key_vault`, `azurerm_key_vault_access_policy`) — the same resource family
    `azurerm_container_group` belongs to (both are plain `hashicorp/azurerm` resources, not
    `azapi`-provider resources). Storage Account AVM's tests were considered and rejected as the
    pattern source for this reason: that module is implemented through the `azapi` provider, so
    its `tests/unit/*.tftest.hcl` mock `azapi`/`modtm`/`random`, not `azurerm` — a pattern that
    does not transfer to a native-`azurerm_container_group` resource.
- **What was adapted:** the `mock_provider "azurerm" {}` + top-level `variables {}` block +
  `run "..." { command = plan ... }` structure from `unit.tftest.hcl`, and the
  `assert { condition = ...; error_message = "..." }` block shape from
  `access_policies.tftest.hcl` (e.g. its
  `azurerm_key_vault_access_policy.this["test"].certificate_permissions` pattern for asserting
  on a resource's computed plan-time attribute) —
  `templates/container_group.tftest.hcl.tmpl`.
- **What was custom-written:** every individual `run` block's assertion *content* (checking
  `azurerm_container_group.this.os_type`/`restart_policy`/`ip_address_type`/
  `container[N].name|cpu|memory|commands[0]` against values bound from this pack's own
  `ontology.ttl` via the same SPARQL queries `templates/main.tf.tmpl` uses) — there is no Key
  Vault equivalent to adapt this content from; only the harness shape (mock provider + plan +
  assert) is shared.

### 3. hashicorp/terraform-provider-azurerm (provider docs + a real GitHub issue)

- URL: `https://raw.githubusercontent.com/hashicorp/terraform-provider-azurerm/main/website/docs/r/container_group.html.markdown`
- License: MPL-2.0 (provider repo license).
- Use: **not adapted as HCL boilerplate** (this file is documentation, not a Terraform module) —
  used to verify, against the real provider's own documented argument reference, the exact
  allowed-value enumerations this pack's `shapes.ttl` SHACL `sh:in` constraints assert:
  `os_type` in `{Linux, Windows}`, `restart_policy` in `{Always, Never, OnFailure}`,
  `ip_address_type` in `{Public, Private, None}` (confirmed via `grep -n -i
  "ip_address_type|restart_policy|os_type"` against the fetched markdown this session, not
  assumed from memory).
- URL: `https://github.com/hashicorp/terraform/issues/34489` (comments fetched via
  `api.github.com/repos/hashicorp/terraform/issues/34489/comments`)
- Use: confirms `mock_provider "azurerm" {}` combined with a root `provider "azurerm" {
  features {} }` block (required by every `azurerm`-provider configuration) works correctly from
  Terraform core `v1.7.0`/`v1.7.1` onward (fixed by PR `hashicorp/terraform#34481`, maintainer
  `@liamcervante`) — grounds `templates/providers.tf.tmpl`'s `required_version = ">= 1.7.1"`
  pin. An earlier, less reliable web-summary pass on this same issue is on record (in the task's
  own research brief) as incorrectly reporting it "closed without resolution"; the primary-source
  issue-comment thread, fetched directly, contradicts that and is the source actually used here.

## What is entirely custom (no real source adapted — standard/boilerplate content, disclosed as
such rather than falsely attributed)

- `templates/providers.tf.tmpl`'s `terraform{}`/`provider{}` blocks: Terraform **child modules**
  (both AVM modules above) declare no provider configuration of their own by design — they
  inherit the caller's. There is no real child-module source to adapt for a root module's own
  provider block; this file is standard Terraform root-module boilerplate.
- `variables.tf.tmpl`'s `environment` and `container_image` variables, and the `locals.tags`
  merge: root-config concerns neither AVM module owns (they never create their own resource
  group or hardcode a deployment-environment tag or a specific image reference).
- `azurerm_resource_group "this"` in `main.tf.tmpl`: standard Terraform root-module content —
  the AVM child module assumes a resource group already exists and is passed in by name; a
  deployable root configuration needs to create one.
- `ontology.ttl`/`shapes.ttl` RDF/SHACL content itself: this pack's own vocabulary (see
  `ontology.ttl`'s own "PUBLIC-ONTOLOGY-FIRST CHECK" section — no public RDF vocabulary for
  Terraform/azurerm resource schemas exists to reuse), though its class/property **local names**
  are taken verbatim from the real azurerm provider argument names wherever a 1:1 mapping
  exists, so the vocabulary stays traceable to the real schema term-for-term.

## Known, disclosed gap (not modeled, not silently assumed away)

No `Dockerfile` exists anywhere in this repository to build the `container_image` this
configuration's `azurerm_container_group.this.container[0].image` argument requires (a single
image containing both `target/release/crown-bribery-case` and `escript` +
`_build/default/lib/{air_core,arazzo_runner,arazzo_atomvm,atomvm_runner}` on `PATH`). Building
that image was out of scope for this task (Terraform generation, not container-image authoring)
and is disclosed here rather than fabricated as a working `container_image` default.
