# praxis: land or delete remote branch `feat/dfcm-federated-capabilities-v26.9.1`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `feat/dfcm-federated-capabilities-v26.9.1` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/praxis --head feat/dfcm-federated-capabilities-v26.9.1`) or delete (`git push origin --delete feat/dfcm-federated-capabilities-v26.9.1`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `feat/dfcm-federated-capabilities-v26.9.1`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | feat/dfcm-federated-capabilities-v26.9.1 | decision pending
