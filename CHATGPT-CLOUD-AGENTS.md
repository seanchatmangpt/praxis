# Hosted GitHub Agent Transport Contract

Read `AGENTS.md` first. This file applies when the agent runs through the
GitHub connector, an ephemeral cloud shell, or another environment where the
normal local checkout or GitHub CLI may be unavailable.

## 1. Orientation receipt

Before work, record:

- repository full name;
- authenticated permission level;
- default branch;
- exact base SHA;
- available connector actions;
- local `git` availability;
- `gh` availability and authentication;
- outbound network availability;
- mounted files and existing checkout state.

Do not classify the task blocked because one transport edge fails.

## 2. Transport ladder

Use the strongest available edge:

1. verified existing checkout;
2. exact-SHA archive;
3. clone/fetch;
4. Git bundle;
5. workflow artifact;
6. connector-backed Git tree/blob reconstruction;
7. dependency-closed sparse reconstruction.

A connector-provided exact file and blob graph can be sufficient to implement
and publish a bounded change. State which edges failed and which edge was used.

## 3. Exact base and branch law

Resolve the exact base SHA before any write. Create a dedicated
`agent/<description>` branch from that SHA. Never write directly to the default
branch unless the user explicitly requires it.

For sequential connector writes, carry the returned content SHA into the next
update of the same path. Never run parallel writes against one path.

## 4. Verification classification

Connector metadata and file reads are `Observed`, not `Executed`. GitHub Actions
results are execution evidence only for their exact commit and commands.

If no exact local tree can be materialized:

- policy/documentation checks that can be recomputed from fetched bytes may be
  verified locally;
- application build/test standing remains `UNKNOWN` or `BLOCKED`, never
  `ALIVE`;
- publish the change only with explicit disclosure of commands not executed.

If an exact tree is reconstructed without `.git`, local builds and tests can
still establish `ALIVE` for those boundaries. Publication may proceed through
blob → tree → commit → ref → draft PR.

## 5. Publication

Default to a draft pull request. The PR body must identify:

```text
State
Base SHA
Head SHA
Transport used
Files changed
Commands executed
Observed validation
Commands not executed
Known exclusions
Falsifier or tamper result
```

After publication, inspect the exact PR diff and current CI state. Do not claim
CI results that have not completed.

## 6. No unreceipted actuation

Every connector mutation must be followed by its returned object identity:
branch, commit SHA, content SHA, PR number, or comment ID. A successful tool
response without preserving the returned identity is an unreceipted actuation.
