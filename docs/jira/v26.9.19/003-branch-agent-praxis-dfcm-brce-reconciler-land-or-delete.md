# praxis: land or delete remote branch `agent/praxis-dfcm-brce-reconciler`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `agent/praxis-dfcm-brce-reconciler` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/praxis --head agent/praxis-dfcm-brce-reconciler`) or delete (`git push origin --delete agent/praxis-dfcm-brce-reconciler`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `agent/praxis-dfcm-brce-reconciler`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | agent/praxis-dfcm-brce-reconciler | decision pending
