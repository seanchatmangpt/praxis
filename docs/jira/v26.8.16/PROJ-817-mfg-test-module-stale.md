# PROJ-817: root package's `src/mfg.rs` test module is stale (36 compile errors)

**Status**: DONE -- fixed in commit `37fe4405`, plus the further
`PlannerOutcome`/`bcinr_pddl::ground::lazy::*`/`Manufactured` drift that
`just check`/`just test` surfaced once this file's own errors stopped
masking them (see that commit's message for the full per-cause breakdown).
`just check` now reaches exit 0 for the whole workspace. Real remaining gap
found by `just test`, explicitly NOT part of this ticket's scope: `ggen`'s
`dogfood_regression::ggen_regenerates_route_files_byte_identically` fails
inside the external `chicago-tdd-tools` CLI-proof harness's temp-directory
repo copy (reports a real, git-tracked file as missing from the copy) --
unrelated to `src/mfg.rs` or to any file `37fe4405` touches.
**Dependencies**: none (independent of PROJ-811/814/815/816 and of commits
`79cf4912`/`20ce7c56`, which fixed the two other build blockers found this
session)

## Scope

`just check` and the root package's own lib target are clean (see commits
`79cf4912` -- pddl-index restoration + `PlannerOutcome` API-drift fixes --
and `20ce7c56` -- restored `crates/praxis-graphlaw/src/bindings_test.rs`).
But the full workspace's *test* target (`just test`, which builds
`--tests` in addition to the lib) still fails, entirely inside the root
package's own `src/mfg.rs` test module:

```
error[E0432]: unresolved import `ggen_graph::prelude::FactStore`
error[E0425]: cannot find type `PddlDomainIr` in this scope  (x6)
error[E0422]: cannot find struct, variant or union type `PddlDomainIr`/`PddlProblemIr` in this scope (x3)
error[E0119]: conflicting implementations of Debug/Clone/Serialize/Deserialize for `AdmissionReceipt`
error[E0599]: no method named `validate_shacl` found for `&DeterministicGraph`
error[E0277]: `Pddl8ActionSchema`/`Pddl8Domain`/`Pddl8Problem` do not implement `Default`
error[E0609]: no field `domain_text`/`problem_text`/`graph_hash_hex` on `mfg::AdmittedPlanningTask` (x7)
error[E0425]: cannot find function `enforce_pddl8` in this scope
```

36 errors total, spanning at least five independent causes:

1. `PddlDomainIr`/`PddlProblemIr` -- types the test module still names that
   no longer exist under those names (renamed, or moved into `bcinr-pddl`'s
   `Pddl8Domain`/`Pddl8Problem` per the same API evolution that produced the
   `PlannerOutcome` drift fixed in `79cf4912`).
2. `ggen_graph::prelude::FactStore` -- an import path that has moved or been
   removed in `ggen-graph`.
3. `AdmissionReceipt` -- something in the test module (or a module it
   imports `*` from) now derives `Debug`/`Clone`/`Serialize`/`Deserialize`
   for this type a second time, conflicting with an existing impl elsewhere.
4. `mfg::AdmittedPlanningTask` -- the test module references
   `domain_text`/`problem_text`/`graph_hash_hex` fields that the struct's
   current definition does not have (renamed or restructured).
5. `enforce_pddl8` -- a function the test module calls that no longer
   exists in scope.

This has the same shape as the `crates/pddl-index` deletion and the
`bindings_test.rs` gap fixed earlier this session (both traced to commit
`3da96c67`, "PDDL 3.1 consolidation and refactor progress before restart")
-- an incomplete refactor left mid-flight -- but this one is materially
larger: five independent causes rather than one, spread across a type's
own field shape, an external crate's re-export path, and a derive
conflict, not a single missing file or a single mechanical API-migration
call site.

## Why this is a separate ticket, not folded into `79cf4912`/`20ce7c56`

Both of those commits were single-cause, single-fix, verified-in-minutes
restorations (a deleted crate; a never-created test file). This is a
multi-cause investigation: each of the five error classes above needs its
own root-cause read (which commit changed `AdmittedPlanningTask`'s fields
and why; where `FactStore` actually lives now in `ggen-graph`; whether
`AdmissionReceipt`'s duplicate derive is a genuine two-copy problem like
the POWL 3-way fork `PROJ-815` documented, or a stray duplicate `impl`
block) before a real fix can be written, per this repo's own
`.claude/rules/_core/absolute.md` ("Fence" -- identify the exact objects
and boundaries before changing anything). Rushing five unrelated fixes
under one commit risks getting at least one wrong.

## Verification plan (once fixed)

```
just test
```
must reach `test result: ok` for the root package's test target, not just
its lib target -- both `--lib` and `--tests` compiling is the acceptance
bar, matching what `79cf4912`'s and `20ce7c56`'s own verification already
achieved for the lib target and for `praxis-graphlaw`, `pddl-index`, `cng`,
and `my-conforming-project`'s libs individually.

## See Also

- `docs/jira/v26.8.16/tickets/PROJ-811.md` -- the `crates/cng` dependency
  chain blocker this session also traced to a workspace-topology gap
- Commit `79cf4912` -- `crates/pddl-index` restoration + `PlannerOutcome`
  API-drift fixes (same root commit, `3da96c67`, as this ticket's likely
  cause for at least the `PddlDomainIr`/`PddlProblemIr` class)
- Commit `20ce7c56` -- `crates/praxis-graphlaw/src/bindings_test.rs`
  restoration (same "incomplete refactor left mid-flight" pattern)
