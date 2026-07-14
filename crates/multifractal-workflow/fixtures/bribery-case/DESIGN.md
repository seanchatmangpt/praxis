# Bribery-compliance case fixture -- Stage 1 design note

Scope: this is Stage 1 of a 4-stage build. It delivers three independently
real, independently tested artifacts and documents exactly how a Stage 2
should wire them together. Stage 1 does **not** wire them together itself
-- no code in this repo yet reads `hook.ttl`'s CONSTRUCT output and writes
it into a PDDL `:init` block automatically. That composition step is named
below as the concrete Stage 2 scope, not silently assumed done.

## The three artifacts

1. **`case.ttl`** -- an F02-admissible RDF observation (real
   `admit_observation` gates 0-6 all pass; proven by
   `crates/multifractal-workflow/tests/bribery_case_fixture.rs`).
2. **`hook.ttl`** -- a real `kh:Hook` + `kh:Action` pair. Its `kh:kind
   "sparql"` trigger matches "cross-border card payment to a contractor
   with an allegation of improper payment"; its SPARQL-CONSTRUCT action
   derives 3 `sc:hasObligation` triples onto the case. Proven live via
   `praxis_graphlaw::TripleStore::load_hook_pack` + `.materialize()` in the
   same test file.
3. **`pddl-domain.ttl`** + **`pddl-problem-closable.ttl`** /
   **`pddl-problem-blocked.ttl`** -- a STRIPS8-safe `pdl:` vocabulary
   compliance domain (same pattern as `ontology/lawobject.ttl`), proven to
   manufacture (`praxis::mfg::manufacture`) into real PDDL8 text and
   ground+solve (`bcinr_pddl::GroundProblem::find_plan`) via
   `/Users/sac/praxis/tests/bribery_case_pddl.rs`.

## Stage 2 wiring (concrete steps, not yet built)

```text
1. admit case Turtle
   multifractal_workflow::f02_observation_admission::admit_observation(
       &policy, &ledger, RawObservation { payload_turtle: <case turtle>, .. }
   ) -> AdmissionReceipt

2. run the hook's SPARQL CONSTRUCT against the admitted graph
   praxis_graphlaw::TripleStore::load_hook_pack(hook.ttl)
       .load_triples(<the SAME admitted case turtle -- gate 2's identity
                      chain requires re-using the exact admitted bytes, not
                      a re-derived copy>)
       .materialize()
   -> the store now contains `<case-iri> sc:hasObligation <obligation-iri>`
      triples (0 or 3, per the hook's trigger condition)

3. merge those facts into the PDDL problem's :init
   THIS is the un-built Stage 2 step. It needs a small, new Rust
   projector function (NOT yet in this repo) with this exact contract:

     fn project_obligations_to_pddl_init(
         case_local_name: &str,          // e.g. "case-brb-2026-0417"
         derived: &[String],             // sc:hasObligation object IRIs
     ) -> Vec<String>                    // pdl:init-shaped atom literals

   For each derived `sc:hasObligation` object IRI, take its local name
   (the part after `vocab#`, e.g. "verify-transaction-authenticity" --
   this is EXACTLY the same local name pddl-problem-closable.ttl's
   obligation pdl:object entries already use, by design -- see that file's
   header) and emit:
     "(has-obligation <case_local_name> <obligation-local-name>)"
   PLUS the corresponding static `requires-evidence` fact, read from
   hook.ttl's own `sc:requiresEvidenceType` catalog triples (also a simple
   SPARQL SELECT over hook.ttl, joined on the same obligation IRI):
     "(requires-evidence <obligation-local-name> <evidence-type-local-name>)"
   These atom-literal strings are inserted as NEW `pdl:init` values on a
   freshly-minted `pdl:Problem` individual (object name = case_local_name,
   type "law-object"), i.e. this step PRODUCES a `pddl-problem-*.ttl`-shaped
   graph fragment at runtime instead of reading one of the two hand-authored
   problem files checked in here -- those two files are worked EXAMPLES /
   design sketches proving the domain solves in both directions, not the
   live data path.

   Evidence-availability facts (`(evidence-unavailable <ob>)`, powering the
   blocked path -- see pddl-problem-blocked.ttl's own header) are NOT
   produced by step 2's hook in this Stage-1 build. A real
   evidence-collection process (a separate Stage 2/3 concern, likely its
   own hook using a FILTER NOT EXISTS-style CONSTRUCT once a bounded
   evidence-collection deadline has passed) must positively assert them
   before this projector runs; until then, no case should be forced through
   block-for-missing-evidence just because evidence has not YET been
   supplied (that would fabricate a closure/non-closure verdict before the
   evidence-collection window is actually over).

4. solve
   bcinr_pddl::domain_from_pddl(<manufactured domain text from
       praxis::mfg::manufacture over pddl-domain.ttl + the runtime problem
       graph fragment from step 3>)
   -> GroundProblem::build -> .find_plan()
   A `Result::Ok` plan reaching `(in-stage <case> receipted)` is the
   lawful-closure path; a plan reaching `(in-stage <case> blocked)` is the
   typed non-closure path; neither is silently converted into the other.
```

## What Stage 1 proved live (see the two test files for exact commands)

- `case.ttl` passes F02 gates 0-6 for real, and 5 negative-control tests
  prove gates 1/2/3/4/5 each genuinely refuse a tampered variant of the
  same fixture (not merely that the untampered happy path is green).
- `hook.ttl`'s CONSTRUCT fires over the real, admitted `case.ttl` graph and
  derives exactly the 3 obligation triples the pattern implies; a domestic
  (non-cross-border) negative-control case does NOT trigger derivation.
- `pddl-domain.ttl` + `pddl-problem-closable.ttl` manufacture into real
  PDDL8 text (`praxis::mfg::manufacture`) and `bcinr_pddl::GroundProblem`
  finds a real plan reaching `(in-stage case-brb-2026-0417 receipted)`.
- `pddl-domain.ttl` + `pddl-problem-blocked.ttl` manufacture and solve to a
  real plan reaching `(in-stage case-brb-2026-0512 blocked)` -- the typed
  non-closure path, distinct from `receipted`, not fabricated as a closure.

## What Stage 1 explicitly did NOT build (named, not hidden)

- The step-3 projector function above (`sc:hasObligation` RDF triples ->
  `pdl:init` PDDL atom-literal strings). Small, mechanical, not yet
  written.
- A hook that positively derives `evidence-unavailable` facts (would need a
  FILTER NOT EXISTS-style CONSTRUCT and a defined "evidence-collection
  deadline passed" trigger condition -- a real design question about WHEN
  absence becomes actionable, not merely a coding task).
- Any binary/CLI entry point chaining steps 1-4 end to end automatically
  (the closest existing precedent in this repo is
  `crates/multifractal-workflow/src/bin/crown-local-cli.rs`'s
  `drive_local_witness_prefix` chain, which composes F02 -> F08 but over a
  hand-embedded PDDL literal, not a hook-derived one -- a real, reusable
  pattern to extend, not to re-invent).
