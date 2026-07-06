# Master Project Inventory — Disk-Verified Reconciliation (2026-07-05)

Method: each claim checked against disk on 2026-07-05/06 via `git -C ~/<name> log -1
--format='%ci %s'`, `ls`, and directory scans. Findings are stated with the command
evidence; no verdicts.

## 1. Verified alive (July 2026 commits on disk)

Each line is the verbatim `git log -1 --format='%ci %s'` output for the repo at `~/<name>`.

| Repo | Last commit |
|---|---|
| praxis | `2026-07-05 18:27:40 -0700 feat(lean-lake): migrate 178 of 202 corpus labels to the Mathlib lane` |
| roxi | `2026-07-05 22:46:50 -0700 Add 80/20 ShExC (compact syntax) support for ShEx` |
| wasm4pm | `2026-07-05 20:55:07 -0700 chore: reseal v26.7.1 release evidence after probe fix` |
| ggen | `2026-07-03 22:43:34 +0000 chore(release): bump version to 26.7.4 (#254)` — deps now frozen |
| lsp-max | `2026-07-03 14:50:28 -0700 docs: record v26.7.3 release, fix stale publish-workflow docs` |
| star-toml | `2026-07-03 14:11:01 -0700 fix: exclude .ggen/ from the published package, bump to 26.7.3` |
| open-ontologies | `2026-07-03 17:34:19 -0700 chore: commit final proof and certificate update` |
| bcinr | `2026-07-03 14:54:55 -0700 fix(deps): resolve path-only dependencies to publishable version requirements` |
| clap-noun-verb | `2026-07-04 20:09:48 -0700 feat: add CommandRegistry::set_app_metadata, bump to 26.7.4` |
| claude-code-config-lsp | `2026-07-03 18:53:28 -0700 feat(cli): add clap-noun-verb CLI with conformance, receipt chain/check, fix commands` |
| osx-clnr | `2026-07-04 19:54:05 -0700 Add Unreal Engine detection to artifact classification` |
| wasm4pm-compat | `2026-07-02 00:25:03 -0700 chore: publication metadata (license/description/repository)` |
| semantic_bit | `2026-07-02 00:25:29 -0700 chore: publication metadata (license/description/repository)` |
| stpnt | `2026-07-02 00:25:29 -0700 chore: publication metadata (license/description/repository)` — implemented repo, not just a concept |
| affidavit | `2026-06-22 20:24:35 -0700 chore: bump version to 26.6.22` |
| chicago-tdd-tools | `2026-06-30 21:58:08 -0700 build: migrate from cargo-make to just` |
| cargo-cicd | `2026-06-29 14:23:52 -0700 fix(cargo-cicd-lsp): correct include_str path depth for bundled schema` |

Note: affidavit (June 22), chicago-tdd-tools (June 30), and cargo-cicd (June 29)
have late-June rather than July last commits; recorded as-is.

## 2. Claimed in conversation, not found on disk

- **tower-lsp-max** — `~/dev/tower-lsp-max` is a symlink (`lrwxr-xr-x ... -> /Users/sac/tower-lsp-max`, created Jun 8) whose target does not exist: `ls /Users/sac/tower-lsp-max` → "No such file or directory". Broken symlink; no repo.
- **oxipraxis / GraphLaw-Oxigraph variant** — `ls /Users/sac/oxipraxis` → "No such file or directory". No `praxis-graphlaw` crate exists in `/Users/sac/praxis/crates/` either. The graph-store role is currently filled by the `~/roxi` fork (alive, see Section 1).
- **Dedicated Lean repos** — no `~/praxis-lean` or other standalone Lean repo. Lean integration lives only inside praxis at `/Users/sac/praxis/crates/praxis-lean`.

## 3. On disk but absent from the conversational inventory

Present under `~/` and git-tracked (or plain directories), not mentioned in the
inventory: roxi, open-ontologies, semantic_bit, osx-clnr, affidavit,
capability-map, mcpp, insa, chicago-tdd-tools, cargo-cicd, claude-code-config-lsp.
Existence verified by `ls -d`; commit dates for the git repos are in Section 1.

## 4. Duplicate families

- **wasm4pm ×8** — copies/worktrees found: `~/wasm4pm` (canonical, July 5 commit), `~/wasm4pm_copy`, `~/wasm4pm-wt-abstractions`, `~/wasm4pm-wt-integration`, `~/wasm4pm-wt-p1` … `-p4`, plus `~/dev/wasm4pm` (stale copy) and `~/chatmangpt/wasm4pm`. `~/wasm4pm-backups/` holds a 2026-05-15 bundle and dot-git tarball.
- **clnrm ×4 backups** — `~/clnrm` plus `~/clnrm.bak`, `~/clnrm-backup-20251015-224552`, `~/clnrm-backup-20251015-233810`, `~/clnrm-backup-2025-10-16-full.tar.gz`, `~/clnrm-dogfood-innovations`.
- **gitvan ×4 backups** — `~/gitvan` plus `~/gitvan-backup-20250918-164242`, `-164245`, `~/gitvan-backup-20250919-084758` (+ `.tar.gz`), `~/gitvan-recent-changes-backup-20250919-091930`, `~/gitvan-work-backup-20250918-164315.zip`, and `~/dev/gitvan`.
- **unibit ×2** — `~/unibit` and `~/chatmangpt/unibit`; the latter contains `unrdf.toml` (verified) and is the bcinr successor.

## 5. Tree characterization

- **`~/`** — live epicenter: all Section 1 repos with July 2026 commits sit directly under the home directory.
- **`~/chatmangpt/`** — Apr-2026 MIOSA/ostar era: contains ostar, BusinessOS, pictl, miniml, canopy, plus large volumes of agent-run report files (AGENT_*, ARMSTRONG_*, A2A_OTEL_* at top level) and the wasm4pm/unibit copies noted above.
- **`~/dev/`** — mostly non-live: 117 non-git directories counted (`for d in ~/dev/*/; do [ -d "$d/.git" ] || ...`), including ~28 zero-commit Nuxt scaffolds per prior census; the live-looking entries are symlinks pointing back to `~/` (`lsp-types-max`, `tower-lsp-max` [broken], `wasm4pm-compat`); `~/dev/wasm4pm` is a stale copy; hidden `~/dev/.ostar-proto.bak` and `~/dev/.qlever` exist on disk and hold star-toml/oxigraph-related material.
