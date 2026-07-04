# Orchestration Workflows (`workflows/`)

The [`workflows/`](file:///Users/sac/praxis/workflows/) directory contains build scripts, orchestration workflows, and release cycle definitions.

## Key Subdirectories

- **`genesis/`**: Multi-day release plans and scripts executing the Genesis Day cycles (Day 1 through Day 7). These contain JS templates directing compiler builds, repository syncs, and publication verifications.
- **`ci/`**: Pipelines configurations ensuring standard checks pass on remote repository branches.

## Execution and Control

Workflows are executed by orchestration tools or script wrappers. The execution steps are designed to be deterministic and repeatable:
- Output actions are committed as data frames.
- Completed jobs emit signed receipt hashes logged to manifest logs (e.g. `MANIFEST_DAY_1.json`).
- Violations in testing coverage or compilation issues refuse the release cycle.
