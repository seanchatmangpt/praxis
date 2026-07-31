# Combinatorial Maximalism Repository Refactor

## Normative Agent Execution Contract

### Invocation

```text
Repo=<owner>/<repo>
Base=<branch|tag|exact SHA>
Task=refactor the repository to implement internal and external combinatorial maximalism
Acceptance=<repository doctrine commands, inferred when not supplied>
Constraints=<user constraints, repository doctrine, generated-surface law>
```

### Required outcome

Transform the repository from an implicit collection of implementations, integrations, workflows, generators, and side effects into an explicit system where:

```text
exact observation
→ admitted semantic model
→ bounded candidate lattice
→ deterministic composition
→ pure plan
→ explicit authorization
→ brokered actuation
→ observed consequence
→ causal receipt
→ independent replay
```

The refactor must implement both:

```text
Internal combinatorial maximalism
=
maximum lawful combination of internal implementations,
runtimes, storage, protocols, projections, and verification rails

External combinatorial maximalism
=
maximum lawful combination of providers, protocols, identities,
consent, trust, jurisdictions, deployment targets, and external capabilities
without collapsing authority boundaries
```

---

# 1. Agent Constitution

Every agent MUST preserve these distinctions:

```text
observation != admitted observation
candidate != verified
verified != authorized
authorized != actuated
command success != consequence success
generated output != semantic authority
plan != execution grant
hook != broker
lifecycle != standing
local standing != external standing
```

The operating law is:

```text
O → O* → I → G → A → R → O′
```

Where:

* `O` is partial or stale observation.
* `O*` is admitted, complete, grounded, bounded observation.
* `I` is deterministic intent.
* `G` is the exact authority grant.
* `A` is the bounded actuation.
* `R` is the causal receipt.
* `O′` is the independently observed consequence.

No agent may collapse or bypass a transition.

## 1.1 Standing vocabulary

Agents may report only:

```text
PARTIAL_ALIVE
ALIVE
BLOCKED
BUILD_BROKEN
UNKNOWN
UNSUPPORTED
```

Rules:

* `ALIVE` requires observed execution at the declared boundary.
* `PARTIAL_ALIVE` means a bounded checkpoint is executable but crown closure is incomplete.
* `BLOCKED` means a required tool, credential, permission, service, or decision is unavailable.
* `BUILD_BROKEN` means admitted source fails its declared build or verifier.
* `UNKNOWN` means evidence is missing, stale, detached, or unexecuted.
* `UNSUPPORTED` means the requested capability lies outside the implemented graph.
* `UNKNOWN` must never be promoted through inference.
* `UNSUPPORTED` must never be described as a refusal.
* `RETIRED`, `ARCHIVED`, and `DEPRECATED` are lifecycle states, not standing states.

## 1.2 Zero unreceipted actuation

No surviving mutation may exist without:

```text
intent receipt
+ exact subject
+ authority grant
+ observed consequence
+ result receipt
```

This applies to:

* repository files;
* lockfiles;
* generated outputs;
* local caches;
* database state;
* Git branches;
* pull requests;
* issues and trackers;
* releases;
* deployments;
* registries;
* cloud resources;
* external APIs.

## 1.3 Generated-surface law

Agents MUST NOT hand-edit generated outputs.

For every generated file:

1. locate its canonical source;
2. locate its generator;
3. modify the canonical source;
4. execute the real generator;
5. verify the generator receipt;
6. execute a second generation;
7. prove byte identity;
8. refuse unexplained drift.

## 1.4 Chesterton-fence law

Agents MUST NOT delete, replace, merge, retire, or bypass a surface until they have identified:

```text
why it exists
who calls it
what it owns
what it mutates
what evidence it emits
how it fails
which compatibility contract depends on it
what executable replacement preserves its function
```

One failed discovery method does not prove that the surface has no purpose.

## 1.5 External actuation law

Architecture kernels, planners, generators, models, hooks, and agents may manufacture typed intents only.

External actuation MUST be routed through the repository’s admitted broker. Use `BRCE` when the Chatman execution boundary is available.

The following are prohibited inside a pure kernel:

* filesystem mutation;
* process execution;
* HTTP calls;
* cloud SDK calls;
* Kubernetes calls;
* Terraform or Pulumi execution;
* package publication;
* deployment;
* tracker mutation;
* Git push or merge;
* credential retrieval.

---

# 2. Agent Orchestration

## 2.1 Coordinator agent

One coordinator owns the refactor program.

The coordinator MUST:

1. resolve the exact base SHA;
2. create the checkpoint dependency graph;
3. assign non-overlapping path scopes;
4. create one work order per checkpoint;
5. prevent concurrent writers to the same semantic object or path;
6. inspect every agent receipt;
7. integrate only dependency-closed checkpoints;
8. run the aggregate verifier;
9. publish a draft PR;
10. classify the final state honestly.

The coordinator MUST NOT allow agents to branch from an unresolved moving reference.

Resolve:

```bash
git fetch --all --tags --prune
git rev-parse <base>
git show --no-patch --format='%H %T %cI' <resolved-sha>
```

Record:

```text
base_ref
base_commit_sha
base_tree_sha
observation_time
repository_remote
toolchain identity
```

## 2.2 Agent work order

Every implementation agent receives:

```text
repository
exact base SHA
checkpoint identity
dependency checkpoints
allowed paths
forbidden paths
required observations
required changes
required commands
required falsifiers
required evidence outputs
standing ceiling
```

Example:

```yaml
checkpoint: G3-INTERNAL-LATTICE
base_sha: "<exact SHA>"
allowed_paths:
  - ontology/cmd/**
  - packs/combinatorial-maximalism/**
  - tests/cmd/**
forbidden_paths:
  - generated/**
  - .github/workflows/release*
  - production/**
required_commands:
  - "<ontology parse command>"
  - "<gate verifier command>"
  - "<candidate coverage command>"
standing_ceiling: PARTIAL_ALIVE
```

## 2.3 Isolated work

Each mutating agent MUST use:

* a dedicated branch;
* a dedicated worktree;
* an exact parent SHA;
* an allowed-path boundary;
* a forbidden-path boundary.

Recommended branch names:

```text
agent/cmd-g0-observation
agent/cmd-g1-fences
agent/cmd-g2-authority
agent/cmd-g3-internal-lattice
agent/cmd-g4-external-lattice
agent/cmd-g5-pack-bblock
agent/cmd-g6-kernel
agent/cmd-g7-materializer
agent/cmd-g8-broker
agent/cmd-g9-crown
```

Agents MUST report any changed path outside their work order as a typed refusal.

## 2.4 Parallelism rules

Agents may work concurrently only when:

* their checkpoint dependencies are satisfied;
* their path scopes do not overlap;
* their semantic ownership does not overlap;
* their generated outputs do not overlap;
* they do not edit a shared lockfile concurrently.

Serialize changes involving:

* root manifests;
* dependency lockfiles;
* canonical ontology indexes;
* workflow ownership;
* generated catalogs;
* shared receipt schemas;
* shared CLI routing.

---

# 3. Required Repository Machinery

Agents must reuse existing repository machinery when equivalent machinery already exists.

When absent, introduce the smallest bounded implementation of:

```text
exact-tree observer
semantic authority
admission gates
candidate lattice
coverage verifier
pure composition kernel
ownership solver
planner
transactional materializer
broker intent boundary
receipt chain
replay verifier
external verifier report
Gall checkpoint controller
```

## 3.1 ggen integration

Agents MUST first determine whether the project already uses ggen.

Inspect:

```bash
find . -name 'ggen.toml' -o -name 'pack.toml' -o -name 'ontology.ttl'
find . -path '*/packs/*' -maxdepth 5
grep -R "ggen sync" .github scripts justfile Makefile.toml 2>/dev/null
```

When ggen is already present:

* preserve its current authority;
* extend the existing root manifest;
* reuse its receipt storage;
* mount the combinatorial-maximalism and Gall packs through the existing pack mechanism;
* do not create a competing root generator.

When ggen is absent:

1. resolve an exact ggen version or commit;
2. record its identity and digest;
3. create the minimal root `ggen.toml`;
4. create an admitted RDF source graph;
5. mount the required packs;
6. run the real ggen binary;
7. verify the receipt;
8. rerun generation;
9. prove byte identity.

Do not use an unpinned floating “latest” version.

## 3.2 Reference packs

Use these capability roles when available:

```text
ggen-combinatorial-maximalism-pack
  bounded candidates, coverage, state/authority separation,
  brokered local actuation, receipts, replay

gall-core-pack
  checkpoints, obligations, work orders, evidence, exclusions,
  execution scheduling, crown verification

ggen-self-host-pack
  exact-tree repository observation and authority mapping

part-passport capability
  lawful internal and external substitution

automatic-autonomic operations capability
  bounded trigger-to-consequence and repair loops
```

If the exact packs cannot be resolved, classify the integration as `BLOCKED`; do not silently recreate weaker lookalikes and call them equivalent.

---

# 4. Mandatory Evidence Layout

Use the repository’s existing evidence layout when one exists.

Otherwise create:

```text
.ggen/cmd/
├── observation/
│   ├── repository.json
│   ├── surfaces.json
│   ├── load-paths.json
│   └── unknowns.json
├── authority/
│   ├── ownership.json
│   ├── fences.json
│   └── collisions.json
├── candidates/
│   ├── dimensions.json
│   ├── options.json
│   ├── internal-candidates.json
│   ├── external-candidates.json
│   └── coverage.json
├── plans/
├── receipts/
│   ├── intents/
│   ├── results/
│   └── chain.json
├── replay/
├── ocel/
├── verifier/
│   └── report.json
└── gall/
    ├── checkpoints.json
    ├── work-orders/
    └── crown.json
```

Generated evidence should normally be ignored from Git unless repository doctrine explicitly commits verifier fixtures or canonical reports.

---

# 5. G0 — Orient and Admit the Exact Repository Tree

## Objective

Produce a mechanically verified observation of the exact repository before changing architecture.

## Mandatory actions

1. Read repository doctrine in this order when present:

```text
AGENTS.md
CLAUDE.md
CONTRIBUTING.md
README.md
architecture documentation
workspace manifests
task-runner files
CI workflows
generation instructions
```

2. Detect:

* languages;
* package managers;
* workspaces;
* generated surfaces;
* test suites;
* build commands;
* release commands;
* deployment commands;
* external integrations;
* data stores;
* protocol surfaces;
* credentials and permission boundaries.

3. Enumerate every tracked Git object using Git semantics:

```bash
git ls-files -s
git ls-tree -r -t HEAD
```

4. Distinguish:

```text
regular file
executable file
symlink
gitlink/submodule
```

5. Classify every path:

```text
authored constitution
domain source
implementation
generated consequence
template
fixture
evidence
workflow
configuration
documentation
archive
asset
unknown
```

6. Record all untracked files separately. Do not admit them as repository authority.

7. Produce an exact-set verifier that fails if:

* one tracked path is omitted;
* one path has the wrong digest;
* one symlink is hashed as target bytes instead of link bytes;
* one gitlink is treated as a normal directory;
* the base SHA changes.

## Required artifacts

```text
observation/repository.json
observation/surfaces.json
observation/load-paths.json
observation/unknowns.json
```

## Mandatory falsifier

Remove one expected tracked path from a copied observation fixture.

Expected result:

```text
REFUSED: CMD-G0-EXACT-SET
```

## Prohibited actions

* deleting files;
* renaming packages;
* consolidating workflows;
* moving generated authority;
* changing runtime behavior.

## Exit criteria

```text
exact tree observed
all paths classified or explicitly UNKNOWN
independent exact-set verifier passes
omission falsifier refuses
no production behavior changed
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 6. G1 — Build the Chesterton-Fence and Ownership Graph

## Objective

Explain every live surface and assign one semantic owner per object and consequence.

## Mandatory actions

For every surface, record:

```text
surface identity
path or external locator
semantic owner
operational owner
generator
consumers
load path
inputs
outputs
mutation scope
evidence produced
failure behavior
retirement dependency
standing
lifecycle
```

Identify:

* duplicate implementations;
* duplicate generators;
* duplicate lockfile writers;
* duplicate receipt issuers;
* duplicate workflow ownership;
* shared production outputs;
* hidden provider-specific branches;
* direct external actuators;
* undocumented generated outputs.

For every apparent duplicate, classify:

```text
equivalent
semantically distinct
compatibility adapter
historical fence
unknown
```

Do not delete any duplicate in G1.

## Required artifacts

```text
authority/ownership.json
authority/fences.json
authority/collisions.json
authority/retirement-candidates.json
```

## Mandatory refusals

### Missing owner

Delete the owner from one live production surface.

Expected:

```text
REFUSED: CMD-G1-OWNER-MISSING
```

### Duplicate exclusive owner

Assign two exclusive owners to one output.

Expected:

```text
REFUSED: CMD-G1-DUPLICATE-AUTHORITY
```

## Exit criteria

```text
every live surface has an owner or UNKNOWN
every generated output has one writer or named merge law
every retirement candidate has an executable fence
no deletion performed
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 7. G2 — Establish Canonical Semantic Authority

## Objective

Move domain, architecture, capability, ownership, policy, and standing meaning into one admitted semantic model.

## Mandatory actions

1. Reuse existing ontology authority where present.
2. Prefer public vocabularies for public concepts.
3. Introduce project-local terms only for project-specific constitutional law.
4. Define SHACL or equivalent machine constraints.
5. Model:

```text
repository
component
capability
dimension
option
candidate
pack
building block
owner
output
policy
authority grant
broker intent
receipt
verifier
standing
lifecycle
exclusion
falsifier
```

6. Generate or mechanically project:

* documentation;
* indexes;
* catalogs;
* JSON schemas;
* diagrams;
* candidate reports.

7. Prove round-trip equivalence between canonical facts and retained projections.

## Required gates

```text
required facts
single-valued fields
unique identity
reference closure
controlled standing values
standing/lifecycle separation
one owner per exclusive output
no generated projection as authority
```

## Mandatory falsifier

Hand-modify a generated projection fixture without changing its canonical graph.

Expected:

```text
REFUSED: CMD-G2-PROJECTION-DRIFT
```

## Exit criteria

```text
canonical meaning has one authority
projections are reproducible
drift is refused
existing semantics are preserved
unknown fields are preserved or refused, never dropped silently
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 8. G3 — Model Internal Combinatorial Maximalism

## Objective

Represent internal implementation variation as a bounded candidate lattice rather than scattered conditionals.

## Mandatory actions

Identify dimensions such as:

```text
runtime
storage
invocation protocol
serialization
consistency model
scheduling model
error model
deployment projection
generation cardinality
ownership mode
recovery mode
verification mode
```

For every dimension, record:

```text
dimension identity
owner
allowed options
selection cardinality
dependencies
constraints
risk class
coverage mode
resource ceiling
```

For every option, record:

```text
option identity
implementation
required capabilities
provided capabilities
incompatibilities
resource envelope
authority requirement
reversibility
verifier
```

## Candidate construction

Calculate:

```text
P = Options(D1) × Options(D2) × ... × Options(Dn)
```

Then admit:

```text
Cvalid = candidates satisfying all constraints
```

The agent MUST NOT materialize the entire unconstrained product when it exceeds admitted limits.

## Coverage selection

Use:

* exhaustive coverage for small bounded lattices;
* pairwise coverage for primarily binary interactions;
* t-wise coverage for known higher-order interactions;
* risk-weighted exhaustive coverage for authority, consent, data, and recovery dimensions.

The coverage verifier must independently recompute expected coverage. It must not trust a declared expected count.

## Mandatory internal constraints

```text
identity closure
dependency closure
candidate totality
option compatibility
resource closure
ownership closure
reversal classification
proof closure
```

## Mandatory falsifiers

1. candidate missing one required dimension;
2. duplicate candidate signature;
3. incompatible options combined;
4. candidate count metadata altered;
5. resource ceiling exceeded;
6. unauthorized option marked actuated.

Each must return a typed refusal.

## Required outputs

```text
candidates/dimensions.json
candidates/options.json
candidates/internal-candidates.json
candidates/coverage.json
```

## Exit criteria

```text
internal variation is explicit data
valid candidate identities are deterministic
coverage is independently verified
invalid combinations are refused
no candidate has actuation authority
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 9. G4 — Model External Combinatorial Maximalism

## Objective

Represent external interoperability without allowing protocols, providers, models, hooks, or adapters to become execution authority.

## Mandatory dimensions

At minimum model:

```text
provider
protocol
identity
authentication
authority scope
consent
data classification
jurisdiction
runtime target
availability model
consequence class
evidence source
rollback or compensation mode
```

## External candidate contract

Every external candidate must declare:

```text
required identity
required authentication
required consent
required trust tier
allowed data classes
allowed jurisdictions
required broker
resource budget
idempotency law
expected postcondition
evidence obligations
reversal classification
```

## Consent model

Consent must be an evidence object containing:

```text
subject
action
resource scope
purpose
issuer
issued time
expiry
revocation status
evidence digest
```

Do not use a generic `consent=true` field as universal authorization.

## Trust model

Represent trust as an explicit state, for example:

```text
UNTRUSTED
LOCALLY_ADMITTED
SIGNED
VERIFIED_PUBLISHER
INDEPENDENTLY_VERIFIED
PRODUCTION_APPROVED
REVOKED
```

## Jurisdiction model

Declare:

```text
processing location
storage location
residency
operator jurisdiction
subprocessor constraints
retention
deletion
legal hold
```

## Part Passport

Every substitutable external realization must carry a passport containing:

```text
conditioned inputs
guaranteed outputs
causal polarity
authority ceiling
resource ceiling
isolation model
host profile
jurisdiction profile
conformity evidence
independent verifier
receipt format
replacement law
retirement law
```

Syntax, API, or ABI compatibility alone is insufficient for substitution.

## Mandatory falsifiers

1. action without consent;
2. action with consent for the wrong resource;
3. jurisdiction mismatch;
4. trust below required floor;
5. revoked identity;
6. direct provider call from pure code;
7. successful API response without postcondition evidence;
8. external action without a broker receipt.

## Exit criteria

```text
external variation is graph-owned
all external actions remain inert intents
identity, consent, trust, and jurisdiction are conjunctive
no provider SDK enters the pure kernel
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 10. G5 — Refactor into Atomic Packs and Building Blocks

## Objective

Make capabilities composable without copying implementations or introducing hidden provider branches.

## Atomic pack rules

An atomic pack is the smallest independently:

```text
versioned
content-addressed
policy-checkable
ownership-declaring
verifiable
replaceable
```

Classify packs as:

```text
surface-*
contract-*
projection-*
runtime-*
policy-*
validator-*
receipt-*
consequence-*
core-*
```

Each pack MUST declare:

```text
stable identity
version
content digest
immutable source
dependencies
provided capabilities
required capabilities
owned outputs
parameters
verifier commands
evidence obligations
migration law
rollback law
license
trust policy
```

## Building Block rules

A Building Block is a composition graph over atomic packs.

It MUST declare:

```text
identity
version
owner
member packs
dependent bblocks
required capabilities
exclusive capabilities
parameter schema
variant rules
output ownership
policy profile
trust floor
verifier profile
migration law
removal law
allowed downstream intents
exclusions
```

It MUST NOT:

* contain copied pack implementations;
* contain provider-specific imperative branches;
* directly deploy infrastructure;
* issue aggregate standing;
* maintain a separate resolver;
* maintain a separate receipt schema.

## Lockfile

Introduce or extend a lockfile binding:

```text
compiler identity
root requests
resolved atomic-pack closure
dependency edges
selected variants
parameters
source identities
immutable revisions
content digests
signatures
ownership claims
policy digest
ontology digest
plan digest
receipt-chain head
```

## Compatibility

Preserve existing commands as thin adapters until equivalence is proven.

Agents must produce a compatibility matrix:

```text
legacy command
legacy semantic mode
new kernel operation
observed equivalent output
known difference
retirement checkpoint
```

## Mandatory falsifier

Resolve the same admitted request through two command surfaces.

If the closures differ without a declared mode distinction:

```text
REFUSED: CMD-G5-SEMANTIC-DIVERGENCE
```

## Exit criteria

```text
capabilities are atomic
compositions are graph-owned
one shared identity and resolution law exists
legacy surfaces remain fenced
no implementation was removed without equivalence
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 11. G6 — Implement the Pure Shared Kernel

## Objective

Centralize combinatorial law in one deterministic, IO-free kernel.

## Kernel responsibilities

The kernel owns:

```text
identity parsing
manifest normalization
dependency closure
capability closure
candidate constraints
ownership analysis
profile conflict detection
resource calculations
substitution assessment
deterministic ordering
plan identity
typed refusals
```

## Kernel prohibitions

The kernel MUST NOT import or invoke:

```text
filesystem mutation
process execution
HTTP
cloud SDKs
Kubernetes
Terraform
Pulumi
Git mutation
tracker APIs
package registries
deployment systems
credential stores
```

## Adapter responsibilities

Adapters may:

* serialize inputs;
* deserialize outputs;
* map protocol representations;
* translate errors into typed refusals;
* manufacture inert intents.

Adapters may not grant authority.

## CLI responsibilities

CLI modules must be thin noun/verb adapters over the shared kernel.

No CLI module may contain a second:

* dependency resolver;
* ownership solver;
* lockfile writer;
* receipt format;
* standing calculator.

## Deterministic plan identity

Calculate:

```text
plan_digest = BLAKE3(
  canonical semantic inputs
  + exact source revisions
  + resolved closure
  + parameters
  + policy
  + observed project tree
  + ownership graph
  + consequence graph
  + compiler identity
)
```

A changed project tree must produce a changed plan identity.

## Mandatory tests

* repeated plan equality;
* changed input changes plan digest;
* cycle refusal;
* ownership collision refusal;
* unknown capability refusal;
* profile conflict refusal;
* no-actuator import scan;
* property-based closure tests.

## Exit criteria

```text
one pure kernel exists
all read-only command surfaces use it
same inputs produce same plan
actuation imports are structurally absent
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 12. G7 — Implement Transactional Local Materialization

## Objective

Apply verified internal candidates without leaving unclassified partial state.

## Required sequence

```text
admit exact plan
→ observe current tree
→ issue intent receipt
→ create bounded staging area
→ materialize candidate
→ run declared validators
→ calculate observed diff
→ commit atomically or restore
→ observe final tree
→ issue result receipt
→ invoke external verifier
```

## Ownership enforcement

Before staging:

* canonicalize all target paths;
* detect aliases such as `x` and `./x`;
* detect traversal;
* detect symlink escape;
* assign one owner or merge law;
* classify deletion authority;
* calculate complete prospective bytes;
* enforce file and aggregate byte ceilings.

## Transaction law

Do not:

```text
write output
→ later attempt to write receipt
```

Use:

* atomic directory rename;
* transactional database commit;
* immutable object plus atomic manifest pointer;
* compare-and-swap;
* append-only commit with atomic receipt head.

## Rollback classification

Every operation must be classified:

```text
REVERSIBLE
REVERSIBLE_WITH_SNAPSHOT
COMPENSATABLE
IRREVERSIBLE
UNSUPPORTED
UNKNOWN
```

Rollback must use an admitted prior state, not a guessed deletion list.

## Mandatory chaos tests

Interrupt at:

1. staging creation;
2. first artifact write;
3. validator execution;
4. receipt staging;
5. pre-commit;
6. commit;
7. postcondition observation;
8. result-receipt publication.

Each interruption must produce:

```text
prior state restored
or
complete committed state with receipt
or
typed recovery state
```

No unclassified partial state may survive.

## Exit criteria

```text
materialization is staged
ownership is enforced before mutation
commit is atomic
rollback is receipted
postcondition is observed
```

Maximum standing:

```text
PARTIAL_ALIVE
```

---

# 13. G8 — Implement Brokered External Actuation

## Objective

Allow external capabilities without placing external authority in the repository kernel.

## Intent contract

Every external intent must contain:

```text
intent identity
candidate identity
operation
arguments
subject digest
desired postcondition
required authority
consent evidence
jurisdiction
resource budget
expiry
idempotency key
required broker
expected evidence classes
```

## Grant contract

Every grant must contain:

```text
grant identity
intent identity
approver identity
policy digest
scope
resource ceiling
expiry
precondition digest
```

## Broker requirements

The broker MUST verify:

```text
intent schema
subject identity
current-state precondition
grant scope
grant expiry
consent
trust
jurisdiction
secrets capability
resource budget
retry budget
circuit state
error budget
idempotency
expected postcondition
```

## Postcondition law

A zero exit status or successful API response is not enough.

The agent must implement a separate consequence observer.

Examples:

```text
deployment request accepted
→ expected version observed serving traffic

release API returned success
→ immutable tag and required assets observed

package publish returned success
→ package and expected digest observed in registry

infrastructure apply returned zero
→ desired resources and policy observed
```

## Automatic operation

Implement:

* bounded trigger admission;
* deterministic route selection;
* exact grants;
* idempotency before actuation;
* bounded retries;
* circuit breaking;
* error-budget gating;
* monotonic Andon;
* lawful inverse or compensation;
* predecessor-linked receipts.

## Autonomic operation

A MAPE-K loop must:

1. observe current state;
2. compare with desired state;
3. manufacture one finite repair intent;
4. execute through the normal broker rail;
5. re-observe;
6. record knowledge;
7. converge or stop at a hard bound.

It MUST NOT:

* use a separate emergency actuator;
* bypass authorization;
* bypass receipts;
* self-promote standing;
* loop indefinitely.

## Mandatory falsifiers

* expired grant;
* wrong subject;
* duplicate idempotency key;
* exhausted retry budget;
* open circuit;
* Andon RED;
* missing consent;
* jurisdiction mismatch;
* postcondition mismatch;
* autonomic cycle limit reached;
* direct provider SDK call outside broker.

## Exit criteria

```text
external capability is intent-driven
authority is explicit
actuation is broker-only
consequence is independently observed
retries and repairs are bounded
```

Maximum standing:

```text
PARTIAL_ALIVE
```

External production standing remains:

```text
UNKNOWN
```

until real external execution evidence exists.

---

# 14. G9 — Execute the Crown Verifier and Self-Host the Repository

## Objective

Prove that the repository can observe, regenerate, verify, sabotage, and replay itself.

## Required verification ladder

Execute in order:

```text
protocol/unit
→ property/fuzz
→ stdio and HTTP integration
→ black-box CLI E2E
→ security
→ chaos
→ stress
→ benchmark
→ replay
→ external verifier report
```

Do not skip a required suite because a lower suite passed.

## Real-boundary requirements

Primary evidence paths must cross real boundaries:

```text
real process
real filesystem
real serialization
real protocol
real database or admitted test service
real receipt verification
real replay
```

Mocks and stubs may support isolated unit tests but may not substitute for crown evidence.

## Self-host observer

The repository must:

1. observe its exact Git tree;
2. classify its own authority;
3. identify every generated output;
4. identify every output writer;
5. identify every workflow and production consequence;
6. refuse missing ownership;
7. refuse duplicate ownership.

## Self-generation

Execute:

```text
real generator
→ first sync
→ receipt verification
→ changed-path ownership check
→ second sync
→ byte-identity comparison
```

No unowned diff may remain.

## Sabotage suite

At minimum mutate:

* one canonical semantic fact;
* one generated output byte;
* one candidate count;
* one ownership assignment;
* one authority grant;
* one receipt digest;
* one replay artifact;
* one direct-actuation prohibition;
* one consent object;
* one exact-head identity.

Each mutation must produce the expected typed refusal.

## Machine-readable report

Emit:

```text
ggen.verifier.report.v1
```

or a repository-equivalent schema containing:

```text
exact subject revision
tree digest
toolchain
policy digest
ontology digest
suite inventory
commands
boundaries crossed
evidence artifacts
passed checks
failed checks
blocked checks
unsupported checks
refusal codes
benchmark measurements
replay result
aggregate standing
verifier identity
```

## External verifier rule

The executor may emit evidence.

Only the independent verifier may set:

```text
aggregate_standing = ALIVE
```

## Crown closure

The crown is `ALIVE` only when:

```text
zero blocking findings
zero unknown live authority
one owner per live generated output
all required evidence surfaces present
exact-head verification completed
first and second manufacture are byte-identical
tampering is refused
clean-tree replay succeeds
```

Otherwise report the strongest honest state.

---

# 15. Required Test Matrix

Every agent must add tests appropriate to its checkpoint.

## 15.1 Positive witnesses

Prove:

* valid observation admission;
* valid candidate construction;
* valid deterministic plan;
* valid local materialization;
* valid external intent;
* valid grant;
* valid receipt;
* valid replay.

## 15.2 Negative falsifiers

Prove refusal of:

```text
missing observation
stale observation
missing owner
duplicate owner
missing dimension
duplicate candidate
constraint violation
resource overflow
path escape
symlink escape
unowned write
premature authorization
direct actuation
missing receipt
receipt tamper
replay divergence
consent mismatch
jurisdiction mismatch
postcondition mismatch
```

## 15.3 Five evidence surfaces

Every promoting obligation requires:

```text
positive witness
negative falsifier
independent verifier
receipt verifier
deterministic replay
```

A checkpoint cannot report `ALIVE` when any required surface is absent.

---

# 16. Language-Specific Validation

Use repository doctrine first. When doctrine is absent, infer the narrowest valid ladder.

## Rust

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

## JavaScript or TypeScript

Prefer the committed package manager.

```bash
corepack enable
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
pnpm test:e2e
```

Do not replace `pnpm` with `npm` or `yarn` without repository authority.

## Python

Prefer the committed environment manager.

```bash
uv sync --frozen
ruff check .
ruff format --check .
mypy .
pytest
```

## Go

```bash
gofmt -w <changed-go-files>
git diff --check
go vet ./...
go test ./...
```

## Java or Kotlin

Use the repository wrapper.

```bash
./gradlew check
```

or:

```bash
./mvnw verify
```

## Mixed repositories

Run narrow package checks first, then expand:

```text
unit
→ package integration
→ workspace
→ repository E2E
```

A pre-existing repository-wide failure must be reproduced on the exact base before being classified as inherited.

---

# 17. Required Pull Request Structure

The coordinator must publish a draft PR containing:

## Preserve

* exact base SHA;
* repository doctrine read;
* preserved surfaces;
* compatibility fences;
* generated-surface boundaries.

## Fence

* scope;
* exclusions;
* allowed paths;
* forbidden paths;
* external capabilities not claimed.

## Calculus

```text
O → O* → candidates → verification → grant → actuation → receipt → replay
```

Explain how this is represented in the repository.

## Internal combinatorial maximalism

Provide:

* dimensions;
* options;
* constraints;
* coverage mode;
* candidate cardinality;
* ownership model;
* resource limits.

## External combinatorial maximalism

Provide:

* providers;
* protocols;
* identities;
* consent;
* trust;
* jurisdictions;
* broker intents;
* consequence observers.

## Verification

Include a table:

| Surface | Command | Result | Evidence | Standing |
| ------- | ------- | ------ | -------- | -------- |

## Falsifiers

List every executed sabotage and typed refusal.

## Receipts

Include:

```text
base SHA
head SHA
tree digest
plan digest
first-sync receipt
second-sync receipt
verifier report digest
replay digest
```

## Standing

Report each checkpoint separately:

```text
G0:
G1:
G2:
G3:
G4:
G5:
G6:
G7:
G8:
G9:
Aggregate:
```

Do not average standings.

---

# 18. Stop Conditions

Agents must stop mutation and report a typed state when:

## BLOCKED

* exact base cannot be resolved;
* required repository is inaccessible;
* required generator cannot be obtained;
* required credentials are unavailable;
* irreversible product decision cannot be inferred;
* required external service cannot be exercised.

## BUILD_BROKEN

* admitted source fails its declared build;
* generated output fails compilation;
* exact-head verifier fails due to the submitted change.

## UNKNOWN

* workflow is queued but not executed;
* external postcondition was not observed;
* replay was not executed;
* verifier identity is unavailable;
* authority or ownership remains unresolved.

## UNSUPPORTED

* the repository has no implementation for the requested external target;
* a consequence cannot be reversed and no compensation contract exists;
* a protocol or provider lies outside the admitted capability graph.

An agent must not convert these states into success through documentation.

---

# 19. Final Agent Completion Rule

The refactor is complete only when the repository demonstrates:

```text
exact-tree observation
+ explicit Chesterton fences
+ one semantic owner per object
+ canonical semantic authority
+ bounded internal candidate lattice
+ bounded external candidate lattice
+ independently verified coverage
+ immutable atomic capability packs
+ graph-owned Building Blocks
+ one pure shared kernel
+ explicit output ownership
+ transactional local materialization
+ broker-only external actuation
+ independent consequence observation
+ causal receipts
+ deterministic replay
+ exact-head external verifier
+ self-hosted regeneration
```

The final invariant is:

```text
Preserve the current system.
Admit the exact state.
Move variation into bounded graph data.
Construct candidates without authority.
Verify before authorization.
Authorize before actuation.
Actuate only through the broker.
Observe the consequence.
Receipt the consequence.
Replay before standing.
Retire old surfaces only after equivalence proof.
```
