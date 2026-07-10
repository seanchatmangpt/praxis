# Standing schema gap: no milestone-artifact kind

`docs/standing/REALITY_INDEX.md` is a generated SPARQL projection of the
`cicd-standing.v1` index (see that file's header). Its artifact `kind` column
only takes the values present in the compiled ontology: `Bench`, `Client`,
`Doc`, `RustCrate`, `Workflow` (25 artifacts total as of the 2026-07-10
refresh).

There is no `MilestoneArtifact` (or equivalent) kind in this schema. As a
result, the Chatman Engine v26.7.9 closure (Gate F verdict
`ADMITTED_DRY_RUN_PUBLISHABLE`, PROJ-411..414) cannot be represented as a
scoped milestone entry in `REALITY_INDEX.md` — there is no row type that
means "this milestone, at this ladder level, as of this commit."

Consequence: `crate:praxis-graphlaw` (`docs/standing/REALITY_INDEX.md:28`)
remains at kind `RustCrate`, ladder `0`, standing `Discovered` — the same
entry it had before the closure. That ladder level describes the crate as a
whole, not the Chatman Engine module within it, and is not evidence against
the Gate F verdict; the verdict and its evidence live in
`docs/chatman-engine/chicago_tdd_final_report.md` and are not currently
mirrored into the standing index.

This is a schema limitation in `cicd-standing.v1` (upstream: `../cargo-cicd`),
not a praxis-repo bug. Fixing it (adding a milestone-artifact kind and
wiring Gate-verdict evidence into it) is out of scope for this closure pass;
raised here so the gap is visible rather than silently absorbed into the
crate-level ladder-0 reading.
