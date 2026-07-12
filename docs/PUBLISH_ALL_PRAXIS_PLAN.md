# Publish-All-Praxis Plan (crates.io Readiness)

Version marker: v26.7.12 · Compiled 2026-07-12 · Status: pre-approval assessment, no publish performed

This document is the plan a Rust core-team lead would write before approving any publish work for the
praxis workspace. It synthesizes nine independent read-only research passes over the 20 workspace
members (`Cargo.toml:229-247`) plus the implicit root package. It states facts with evidence and marks
everything it could not verify this session as `UNVERIFIED`. It does not itself run `cargo publish`
(prohibited this session) and does not recommend *whether* praxis should be published — only what it
would take.

## Scope note: there are 21 publishable packages, not 20

The root `Cargo.toml` carries both `[package]` (name `my-conforming-project`, version `26.7.2`,
`Cargo.toml:2-3`) and `[workspace]` (`Cargo.toml:229`), so under Cargo semantics the root is itself a
publishable workspace member. Confirmed: `cargo metadata --no-deps` reports 21 workspace members. Two
consequences a core team must not miss:

1. The default-publishable crate name would be `my-conforming-project` (boilerplate scaffold name), not
   `praxis`.
2. The root package pulls in most of the workspace plus the `BUSL-1.1` `wasm4pm` subtree (§4), so it is
   the *most* exposed package, and it is not in the 20-member list anyone would think to review.

No member (nor the root) sets `publish = false` — verified this session:
`grep -rn "^publish" crates/*/Cargo.toml audit-tools/Cargo.toml apps/air_core/native/air_core_nif/Cargo.toml tmp_sparql2/Cargo.toml`
returns nothing (exit 1). The idiom *is* used elsewhere in this repo
(`crates/rust-fable-testbed/fixtures/*/Cargo.toml:5`, `tools/hollow-gate/Cargo.toml:5`), so its absence
on the real members is an omission, not unfamiliarity.

---

## 1. Go/No-Go blockers (must clear before ANY related crate can publish)

These are ordered by how much of the workspace they gate.

### B1 — External path deps with no crates.io home (gates the whole `praxis-graphlaw` subtree)

`praxis-graphlaw` is the single most load-bearing crate (6 in-workspace dependents: cng, ggen,
praxis-core, praxis-synthesis, praxis-graphlaw-wasm, multifractal-workflow). Its own manifest hard-depends
on crates that live in *sibling repos outside this workspace* and are not on crates.io:

- `bcinr-pddl`, `bcinr-powl`, `bcinr-powl-receipt` → `/Users/sac/bcinr/crates/*`
  (`crates/praxis-graphlaw/Cargo.toml:39-41`, all three **unversioned** path deps).
- `wasm4pm-compat` → `/Users/sac/wasm4pm-compat` (`:44`, versioned `26.6.29`, still not on crates.io).

Until those external crates are published (or vendored, or the dep removed), no crate transitively rooted
on `praxis-graphlaw` can resolve on public crates.io. crates.io requires every non-dev dependency of a
published crate to already exist on the registry. Effort: **blocked-on-external**.

### B2 — BUSL-1.1 licensed external deps under MIT/Apache-declared crates (licensing, not mechanical)

Three `wasm4pm` crates declare `BUSL-1.1` — a source-available, non-OSI license that converts to
`AGPL-3.0-only` two years after first publication (`/Users/sac/wasm4pm/LICENSE:1-24`;
`license = "BUSL-1.1"` verified this session at `/Users/sac/wasm4pm/crates/prolog8/Cargo.toml:5`,
`/Users/sac/wasm4pm/crates/wasm4pm-cognition/Cargo.toml:5`, and the `wasm4pm` workspace
`/Users/sac/wasm4pm/Cargo.toml:12`).

They enter praxis through **non-optional** paths:

- Root `my-conforming-project` (declares `MIT OR Apache-2.0`): `prolog8` (`Cargo.toml:105`) and
  `wasm4pm-cognition` (`Cargo.toml:114`) as regular deps.
- `multifractal-workflow` (declares `MIT OR Apache-2.0`): `wasm4pm-cognition` and `wasm4pm-planner`
  (`crates/multifractal-workflow/Cargo.toml:261-262`); `wasm4pm-planner` transitively pulls `prolog8`
  (`Cargo.lock:7347-7358`).
- `praxis-graphlaw` (declares `MIT`): `wasm4pm-cognition` **optional**, behind non-default `cognition`
  feature (`crates/praxis-graphlaw/Cargo.toml:45,48`) — only an issue under `--all-features`.

crates.io will not *computationally* block this (it only checks a license field is present). But shipping
MIT/Apache crates that hard-depend on BUSL-1.1 (soon-AGPL) code misrepresents downstream rights. This is a
license-owner decision (relicense `wasm4pm`, drop the dep, or accept copyleft), not an engineering fix.

### B3 — Four crates have no license field at all (hard `cargo publish` error)

Verified this session (`grep -nE "^license|^license-file"`): **no license field** in
`audit-tools/Cargo.toml`, `apps/air_core/native/air_core_nif/Cargo.toml`, `tmp_sparql2/Cargo.toml`,
`crates/mfact-core/Cargo.toml`. `cargo metadata` reports `"license":null` for exactly these four. Each
would hard-error on the missing-license check independent of everything else. All four are scratch/scaffold
(see §3), so the fix is `publish = false`, not adding a license. Effort: **trivial**.

### B4 — `tmp_sparql2` is entirely git-ignored → zero packageable files

`.gitignore:100` (`/tmp_sparql2/`) plus `git ls-files tmp_sparql2 | wc -l` → **0** (verified this session).
It is an active `[workspace] members` entry (`Cargo.toml:246`) added in the same 2026-07-12 "genesis"
commit that gitignored it. Cargo's default git-aware file selection would find no includable files. Fix:
`publish = false` or remove from `members`. Effort: **trivial**.

### B5 — No root LICENSE text file backs the declared SPDX strings

No `LICENSE*`/`COPYING*` at repo root (`find -maxdepth 1 -iname "LICENSE*"` empty). Only
`crates/praxis-graphlaw/LICENSE` exists as a per-crate file. crates.io accepts the SPDX `license` string
without a bundled file, so this is not a hard blocker, but a core team should add root `LICENSE-MIT` /
`LICENSE-APACHE` before any public publish. Effort: **trivial**.

### B6 — Information-disclosure hygiene (not secrets, but ships publicly)

No API keys, tokens, passwords, private keys, or internal IPs were found in any packageable tree (§ security
pass, high-confidence negative). What *would* ship publicly are ~334 hardcoded `/Users/sac/...` local-path
references in tracked doc comments — heaviest in `multifractal-workflow/src/*` (`grep -rn '/Users/sac'
crates/multifractal-workflow/src | wc -l` → 334), also in agent8, praxis-core, praxis-synthesis, cng, and
the root `src/`. They leak the author's username and the names of ~9 private, unpublished sibling repos.
Cosmetic for an internal registry; a real disclosure concern for public crates.io. Effort: **small–medium**
(mechanical scrub, but wide).

### B7 — `praxis-lean` has 3 untracked-but-not-ignored files that would ship

`src/closure.rs`, `src/receipt_gate.rs`, `tests/receipt_closure_gate.rs` are untracked (`??`, not ignored),
so Cargo's default git-aware inclusion would bundle uncommitted, unreviewed code. Manually scanned: no
secrets. Fix: commit or delete before publishing this crate. Effort: **trivial**.

### Known mechanical blocker (given, not re-verified this session)

`cargo publish -p multifractal-workflow --dry-run` fails with "all dependencies must have a version
requirement specified when publishing" — 7 in-workspace path deps with no version (powl2-decompose,
praxis-core, praxis-graphlaw, wasm4pm-arazzo, pddl-index, cng, ggen). Taken as given: `cargo publish`
(incl. dry-run) is prohibited this session, so this specific failure was **not re-verified now** and is
carried forward from the session's prior confirmation. The same missing-`version` pattern recurs across the
graph (§3); adding versions is the bulk of the mechanical work but does **not** clear B1–B2.

---

## 2. Publish order (topological)

The dependency DAG is a strict 5-level DAG (0→4), **no cycles** (cross-checked by grep and
`cargo metadata --no-deps`; the in-repo back-edge comments at `crates/praxis-core/Cargo.toml:30-32` and
`crates/multifractal-workflow/Cargo.toml:25-26` hold). In-degree leader: `praxis-graphlaw` (6). Out-degree
leader: `multifractal-workflow` (7).

Publish order **if** the external gate (B1) were cleared:

```
Level 0 (leaves): chatman-common, powl2-decompose, praxis-lean, praxis-retrofit,
                  pddl-index, wasm4pm-arazzo, mfact-core*, audit-tools*,
                  air_core_nif*, tmp_sparql2*        (* scaffold → publish=false, skip)
Level 1:          praxis-graphlaw            (needs powl2-decompose + bcinr-* + wasm4pm-compat)
Level 2:          praxis-core, cng, praxis-synthesis, praxis-graphlaw-wasm
Level 3:          agent8, praxis-proposer, rust-fable-testbed, ggen
Level 4:          multifractal-workflow
(Level 5:         root my-conforming-project — depends on L2/L3 crates)
```

### The crates that could actually go FIRST with the least new work

Only four members have **both** a license **and** zero path dependencies of any kind (no in-workspace, no
external), so they need only metadata polish + a version bump — no B1/B2 exposure:

1. **`chatman-common`** — MIT/Apache, zero path deps. Historical `cargo publish --dry-run` exit 0 (16
   files) recorded at `docs/releases/v26.7.6/TEST_REPORT.md:159-163` (stale, 2026-07-06). Only gap: the
   `repository` field points at a *different* repo (`github.com/…/chatman-common`,
   `crates/chatman-common/Cargo.toml:8`) and its on-disk `README.md` is not declared via `readme =`.
2. **`powl2-decompose`** — MIT/Apache, zero path deps. Same stale exit-0 precedent (11 files). Gap: no
   `repository` field at all.
3. **`praxis-lean`** — Apache/MIT, zero path deps. Gap: B7 (3 untracked files) + undeclared README.
4. **`praxis-retrofit`** — MIT/Apache, zero path deps. Gap: it is internal house-style tooling carrying a
   recorded "615 findings" lint-debt allow-list (`src/lib.rs:20-26`); publishable mechanically, low
   external value.

These four are the entire genuinely-first-movable set. Everything else is blocked on either external deps
(B1/B2) or on one of these/`praxis-graphlaw` publishing first.

Note `pddl-index` and `wasm4pm-arazzo` are graph-level-0 but each carry the single external dep
`wasm4pm-compat` (not on crates.io) → they are **blocked-on-external**, not first-movable.

---

## 3. Per-crate readiness table

`publish=` column is the recommendation. Effort classes: trivial / small / medium / large /
blocked-on-external / blocked-on-internal.

| Crate | publish= | Primary blockers | Effort |
|---|---|---|---|
| `chatman-common` | true | `repository` points to a different repo; `readme` undeclared | small |
| `powl2-decompose` | true | no `repository` field | small |
| `praxis-lean` | true | B7 (3 untracked files would ship); `readme` undeclared | small |
| `praxis-retrofit` | true | none mechanical; internal tooling, low reuse value | small |
| `pddl-index` | true | external `wasm4pm-compat` not on crates.io; no `repository`; cng's own manifest calls it "not published on crates.io" (`crates/cng/Cargo.toml:42-44`) | blocked-on-external |
| `wasm4pm-arazzo` | true | external `wasm4pm-compat` **unversioned** (`Cargo.toml:19`) + not on crates.io; no `repository` | blocked-on-external |
| `praxis-graphlaw` | true | B1 (bcinr-* + wasm4pm-compat, 4 unversioned path deps `Cargo.toml:38-41`); B2 via optional `cognition`; live dry-run status **UNVERIFIED** (§ verification) | blocked-on-external |
| `praxis-graphlaw-wasm` | true | depends `praxis-graphlaw` (versioned) → inherits B1 | blocked-on-internal |
| `praxis-core` | true | unversioned path deps (chatman-common/powl2-decompose/wasm4pm-arazzo `Cargo.toml:19,39,40`); external `bcinr-powl-receipt`+`wasm4pm-compat` unversioned (`:20-21`) → B1 | blocked-on-external |
| `praxis-synthesis` | true | all 3 in-workspace deps versioned, but siblings not yet on crates.io; no `repository`; README self-labels "Prototype crate" | blocked-on-internal |
| `cng` | true | 3 unversioned optional path deps (praxis-graphlaw/pddl-index/chicago-tdd-tools `Cargo.toml:41,50,59`); version `26.9.10` violates repo CalVer (§ versioning); B1 via those deps | medium + blocked-on-internal |
| `ggen` | true | unversioned `praxis-core`+`praxis-graphlaw` (`Cargo.toml:63-64`) → B1 chain | blocked-on-internal |
| `agent8` | true | unversioned `praxis-core` (`Cargo.toml:13`) → B1 chain | blocked-on-internal |
| `praxis-proposer` | true | unversioned `praxis-core` (`Cargo.toml:13`); MIT-only (inconsistent) | blocked-on-internal |
| `rust-fable-testbed` | maybe | unversioned `praxis-core` + external `ggen-core` (`/Users/sac/ggen`, `Cargo.toml:11`); internal Claude-eval tooling | blocked-on-external |
| `multifractal-workflow` | **false (defer)** | self-declared "scaffolding skeleton only" (`Cargo.toml:7`, `src/lib.rs:6-9`); 7 unversioned path deps (known blocker); B2; 334 path leaks | large / blocked |
| `audit-tools` | **false** | no license (B3); unmodified `cargo new` template (14 LOC) | trivial |
| `air_core_nif` | **false** | no license (B3); `cdylib`-only rustler NIF, not a crates.io artifact | trivial |
| `tmp_sparql2` | **false** | no license (B3); git-ignored → unpackageable (B4); 5-line spike | trivial |
| `mfact-core` | **false** | no license (B3); `0.1.0` off-scheme scaffold, one undocumented module | trivial |
| root `my-conforming-project` | **false / rename** | B2 (non-optional BUSL deps); boilerplate name; unversioned `ggen-graph` (`Cargo.toml:84`); path leaks in root `src/` | large / blocked |

---

## 4. The external-dependency problem — is public crates.io even the right target?

Praxis depends on **7 distinct sibling repos outside this workspace**, all real, git-tracked, non-scratch:

| External repo | Crates pulled in | License | On crates.io? |
|---|---|---|---|
| `/Users/sac/bcinr` | bcinr-pddl, bcinr-powl, bcinr-powl-receipt, bcinr-logic | MIT/Apache | No |
| `/Users/sac/wasm4pm` | prolog8, wasm4pm-cognition, wasm4pm-planner | **BUSL-1.1 → AGPL** | No |
| `/Users/sac/wasm4pm-compat` | wasm4pm-compat | MIT/Apache | No |
| `/Users/sac/ggen` | ggen-core, ggen-graph (distinct from in-repo `crates/ggen`) | MIT | No |
| `/Users/sac/chicago-tdd-tools` | chicago-tdd-tools (dev + one optional) | MIT | No |
| `/Users/sac/lsp-max` | lsp-max (root, optional) | MIT/Apache | No |
| `/Users/sac/affidavit` | none — orphaned `[patch]` only (`Cargo.toml:218`, no real dep) | MIT/Apache | n/a |

### Plain assessment

**Public crates.io is not the realistic near-term target for most of the workspace**, for three
independent reasons the findings establish:

1. **Transitive external gate (B1).** The keystone `praxis-graphlaw` — and therefore its 6 dependents and
   everything above them — cannot resolve publicly until `bcinr-*` and `wasm4pm-compat` are themselves
   published. That is a second workspace's worth of publish work (`/Users/sac/bcinr` and
   `/Users/sac/wasm4pm-compat` are multi-crate repos) before praxis's own upper levels can move.

2. **License incompatibility (B2).** The `wasm4pm` subtree is `BUSL-1.1` (converting to AGPL-3.0). Root and
   `multifractal-workflow` pull it **non-optionally** while advertising `MIT OR Apache-2.0`. Publishing
   those to public crates.io as-is would misrepresent downstream rights. This blocks the two most-connected
   packages regardless of any mechanical fix, and is not the engineering team's call to make.

3. **Disclosure surface (B6).** 334+ `/Users/sac/...` references naming ~9 private repos would become
   public.

### Realistic paths, in order of near-term feasibility

- **(A) Internal / private cargo registry** (e.g. a self-hosted `cargo` registry or a Git-source
  vendoring workspace). This sidesteps B1 (siblings can be published to the same private registry, in
  dependency order), tolerates BUSL-1.1 among trusted consumers (B2 becomes an internal-licensing
  question, not a public misrepresentation), and makes B6 a non-issue. **This is the realistic near-term
  answer for anything touching `praxis-graphlaw` or `wasm4pm`.**
- **(B) Public crates.io, narrowly scoped** to the four clean leaves only (chatman-common,
  powl2-decompose, praxis-lean, praxis-retrofit). Real, low-cost, and matches the repo's own
  `PUBLISH_READY` (ladder rung 7, `docs/standing/PRODUCTION_READINESS.md:20`) "requires scope" discipline.
- **(C) Public crates.io, whole workspace** — requires (1) publishing `bcinr` + `wasm4pm-compat` first,
  (2) a license resolution for the `wasm4pm`/BUSL subtree, (3) the B6 scrub, (4) versions on every path
  dep. This is the largest option and gated on a non-engineering decision (B2).

Repo policy already frames the actual `cargo publish` step as an **operator-credentialed external side
effect** an agent must not perform (`CLAUDE.md:31-34`;
`docs/standing/EXTERNAL_OPERATOR_SIDE_EFFECTS.md:14-31`). There is **no live publish automation**: the
`.github/workflows/release.yml` is an unrenamed generic template (builds a binary literally named
`project-name`), its crates.io job is commented out (`release.yml:103-119`), it has **0 runs ever**
(`gh api …/workflows/301004638/runs` → `total_count:0`), and **no `CARGO_REGISTRY_TOKEN` secret exists**
(`…/actions/secrets` → `total_count:0`). The only local automation is `just publish-dry-run <crate>`
(`justfile:516-518`) and `just crates-search` (`:512-514`). A prior cycle already reached this same
conclusion at v26.7.6 (`docs/releases/v26.7.6/TEST_REPORT.md:159-163`,
`FINAL_STATUS.md:132-138` — "Nothing was published").

---

## 5. Honest total-effort estimate + highest-leverage first step

### Effort estimate (by target)

- **Path B (4 clean leaves → public crates.io):** small. Per crate: add/repair `repository`, declare
  `readme`, commit `praxis-lean`'s 3 untracked files, one version bump, one dry-run. Roughly a day of
  focused work total, gated only on an operator running the final publish with credentials.
- **Path A (internal registry, whole workspace):** medium–large. Stand up the registry; publish
  `bcinr-*` and `wasm4pm-compat` there; add `version` to every unversioned path dep across the graph
  (~20+ edges); resolve `cng`'s CalVer anomaly; mark the 4 scaffolds `publish = false`. Days to low weeks.
  Does **not** require the B2 license resolution if the registry is private and all consumers accept it.
- **Path C (whole workspace → public crates.io):** large + externally blocked. Everything in Path A, plus
  publishing two additional external workspaces publicly, plus a **BUSL-1.1/AGPL license resolution
  (non-engineering)**, plus the B6 disclosure scrub, plus deciding what to do with the boilerplate root
  package name and the self-declared-skeleton `multifractal-workflow`. Not schedulable until B2 is
  answered by the license owner. Weeks, and partly out of the team's hands.

### Single highest-leverage first step

**Make the target-registry go/no-go decision — public crates.io vs. internal registry — because it
determines whether ~90% of the mechanical version/metadata work is even worth doing.** That decision turns
entirely on B2 (the BUSL-1.1/AGPL exposure in the `wasm4pm` subtree), which the engineering team cannot
resolve and which gates every package above `praxis-graphlaw`.

Concretely, the first *executable* action under either answer, done in parallel with that decision, is to
**mark the four scaffolds `publish = false`** (audit-tools, air_core_nif, tmp_sparql2, mfact-core) and
**dry-run the four clean leaves** (chatman-common, powl2-decompose, praxis-lean, praxis-retrofit). That
removes the four hard-error crates from every future `--workspace` operation and proves out the only
subset that is genuinely publishable today, with no dependency on the B1/B2 resolution.

---

## Verification status (what this plan did and did NOT confirm this session)

Confirmed this session by read-only grep/git (no `cargo`): no `publish` flag on any member; no
`[workspace.package]`; root package identity (`my-conforming-project` 26.7.2, MIT/Apache); four crates with
no license field; `tmp_sparql2` 0 git-tracked files; `BUSL-1.1` on prolog8/wasm4pm-cognition/wasm4pm root.

**UNVERIFIED this session** (carried from prior findings; `cargo publish`/dry-run prohibited, so not
re-tested now):

1. The multifractal-workflow 7-dep dry-run failure — **taken as given**, not re-run.
2. Whether a **feature-gated / optional** path dep still triggers the "must have a version requirement"
   check (relevant to cng's optional deps). Inferred from Cargo's documented manifest-level check;
   consistent with the multifractal-workflow failure enumerating only non-optional deps, but not
   empirically confirmed for the optional case.
3. The **dev-dependency exemption** (`chicago-tdd-tools` as path-only dev-dep in cng/ggen/praxis-graphlaw
   not blocking publish). Documented Cargo behavior, not dry-run-confirmed here.
4. **`praxis-graphlaw`'s current dry-run status.** The exit-0 result is from v26.7.6 (2026-07-06,
   `docs/releases/v26.7.6/TEST_REPORT.md`); the crate is now 26.7.9 with unversioned path deps
   (`Cargo.toml:38-41`). Current status unknown — do not assume the stale pass still holds.
5. Whether the **four clean leaves** would pass a live dry-run *today* — only chatman-common and
   powl2-decompose have a stale v26.7.6 exit-0 precedent; praxis-lean and praxis-retrofit have none.

## References

- `CLAUDE.md:31-34` — external actions (crates.io publish) are operator side effects, not agent actions
- `docs/standing/EXTERNAL_OPERATOR_SIDE_EFFECTS.md:14-31` — operator publish checklist
- `docs/standing/PRODUCTION_READINESS.md:20` — `PUBLISH_READY` ladder rung 7 ("requires scope")
- `docs/releases/v26.7.6/{TEST_REPORT,FINAL_STATUS,RELEASE_CONTROL}.md` — prior publish investigation
- `justfile:512-518` — `crates-search`, `publish-dry-run` recipes
- `/Users/sac/wasm4pm/LICENSE` — BUSL-1.1 grant, Change License AGPL-3.0-only
