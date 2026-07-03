# Trustless replay

Re-verify praxis-synthesis receipts with **no cargo, no crate source**, in a
bare directory whose PATH contains nothing but `python3` and `b3sum`.

## The two commands

```sh
# 1. (Re)generate the artifacts — requires cargo:
scripts/trustless_replay.sh package
#    (equivalently: cargo test -p praxis-synthesis --test trustless_artifacts -- --ignored)

# 2. Verify them without cargo or source:
scripts/trustless_replay.sh verify          # dir defaults to receipts/trustless
```

`verify` copies exactly six files into a fresh `mktemp -d` directory —

| File | What |
|---|---|
| `foreign_verify.py` | cell verifier (python3 stdlib + `b3sum`) |
| `foreign_verify_graph.py` | workflow-graph verifier (python3 stdlib + `b3sum`) |
| `cell.json` | `CellReceipt` from `run_cell(400, 4, 8)` |
| `groups.json` | the four `GroupReceipt` roll-ups |
| `workflow.ttl` | raw bytes of `ontology/workflow_demo.ttl` |
| `workflow_receipt.json` | `WorkflowReceipt` from `execute_workflow` |

— builds a bare `bin/` holding symlinks to the resolved `python3` and `b3sum`,
and runs both verifiers under `env -i PATH="$tmp/bin" HOME="$tmp"`. Any
mismatch fails the script with the verifier's own `MISMATCH: <stage>` line
(exit 1); a missing prerequisite or artifact exits 2 naming it and printing
the `package` command.

## What a passing run proves

The cell receipt and the workflow receipt re-verify from their JSON alone, by
a **second implementation in a second language** (python3) using a **second
BLAKE3 binary** (`b3sum`), inside a directory containing no crate source,
with a PATH containing nothing but `python3` and `b3sum`. The receipts are
therefore not self-attested: two independent codebases agree on every hash
they can both compute.

## What it does NOT prove

- The `ir_hash` / `plan_hash` / `topology_hash` / `geometry_hash` stage hashes
  in the workflow chain are **refolded as claimed, not re-derived** — the
  python verifier re-derives only `graph_hash` (from the TTL bytes) and
  `exec_hash` (from the supervised payload), and checks the plan-body binding.
  Re-derivation of the middle stages requires `replay_workflow` in the Rust
  crate.
- Nothing binds the artifacts to a git commit.
- **No container or namespace isolation is claimed** — with or without docker
  installed, this recipe does not use it. The guarantee is directory + PATH
  hygiene only, and the `python3`/`b3sum` used are the host's own binaries.
