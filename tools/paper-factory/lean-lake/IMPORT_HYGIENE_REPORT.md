# Import Hygiene Report — praxis-lean-pilot

Report-only; no imports in `Praxis/Corpus/`, `Praxis/Mathlib/`, `Praxis/Milestone/`, or
`Praxis/MFW/` were edited. Generated 2026-07-12 while another agent had a concurrent
`lake build` in progress against this same `.lake` build directory — that build was not
interrupted or raced against.

## Tooling Availability

- **`lake shake`**: built into this toolchain (`lake shake --help` resolves; ships with
  Lake itself, not a separate package). Confirmed flags: `--explain`, `--fix`,
  `--keep-implied`, `--keep-prefix`, `--keep-public`, `--gh-style`.
- **`importGraph`**: already present as a **transitive dependency of Mathlib**
  (`.lake/packages/importGraph/`, listed in `lake-manifest.json`). No `require` line was
  added to `lakefile.lean` — it was not needed. `ImportGraph.Tools` (home of
  `#redundant_imports`, `#min_imports`, `#find_home`) was already built
  (`.lake/packages/importGraph/.lake/build/lib/lean/ImportGraph/Tools.olean` exists);
  `ImportGraph.Imports` was not built and was not exercised.

## `lake shake --explain` Result: Blocked, Not Failed

`lake shake --explain` (and `--explain --force`) could not produce a report this session:

```
error: object file '.../lean-lake/.lake/build/lib/lean/Praxis.olean' of module Praxis does not exist
```

Root cause: a concurrent `lake build Praxis.CorpusExtra` (another agent, PID 83739/83928
at capture time) was mid-flight, and `Praxis.olean` plus most of `Mathlib`'s ~3100 oleans
were not yet built. `lake shake` requires up-to-date oleans project-wide (or `--force` to
skip the freshness *check*, which does not conjure missing oleans). Per the task's
no-race constraint, this agent did not run `lake build` to force the missing oleans.

**Action for follow-up**: once the in-flight `Praxis.CorpusExtra` build (and full project
build) completes, re-run `lake shake --explain > shake_report.txt` from this same
directory — no setup is required, the tool is ready.

## importGraph Scratch Test: Confirmed Working

A throwaway `Scratch.lean` (deleted after use) was used to exercise the `ImportGraph.Tools`
commands against `Praxis.Corpus.def_makespan` (one of the ~29 Corpus modules whose oleans
already existed on disk from a prior/partial build):

```lean
import ImportGraph.Tools
import Praxis.Corpus.def_makespan

#redundant_imports
```

Output:
```
Found the following transitively redundant imports:
ImportGraph.Tools
```

(Expected — `ImportGraph.Tools` itself is redundant once its declarations aren't used;
this confirms the command runs and reports correctly, not a hygiene finding about
`def_makespan`.) `#min_imports` on the same file produced no output, i.e. no reduction
suggested for that one-declaration file — consistent with a leaf module.

`#find_home` was not exercised against a specific declaration (no target picked this
session); the command is confirmed available (`ImportGraph.Tools` source, line 22) and
ready for targeted use once the full build is current.

## Findings

No `lake shake --explain` findings were produced this session — the tool never got past
the freshness check because oleans for `Praxis` and most of `Mathlib` were absent during
the concurrent build. There is nothing to categorize into (a) namespace-collision overlap
vs. (b) pure import hygiene yet.

### (a) Overlaps with the known namespace-collision work — defer

Not yet determined. Once `lake shake --explain` runs, cross-reference any
`Praxis/Corpus/*.lean` entries against the files the concurrent collision-resolution agent
is touching before acting on them.

### (b) Pure import hygiene — safe to fix later

Not yet determined, same reason.

## Final `lake build` Status

Not re-run to completion by this agent, to avoid contending with the concurrent build
already in progress on the same `.lake` directory. `lake build --no-build` (read-only,
does not trigger compilation) confirmed the target list is still out-of-date, ending with:

```
- Praxis.Corpus.def_gitlock
- Praxis.Corpus.def_saturation
- Praxis.Corpus.def_earned
- Praxis.Corpus.thm_trichotomy
- Praxis.MFW.State
```

No files under `Praxis/Corpus/`, `Praxis/Mathlib/`, `Praxis/Milestone/`, or `Praxis/MFW/`
were modified. `lakefile.lean` was not modified (no `require` line added — unneeded).
`Scratch.lean` was created and deleted; no stray files remain.

## See Also

- `lakefile.lean` — package definition, Mathlib pin at `v4.31.0`
- `.lake/packages/importGraph/ImportGraph/Tools.lean` — source of `#redundant_imports`,
  `#min_imports`, `#find_home`
