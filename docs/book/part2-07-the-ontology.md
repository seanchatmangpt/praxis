# The Ontology

Praxis's schema is a single Turtle file: `schema/praxis.ttl`. It defines the RDFS
vocabulary used to describe a "conforming project" — its crates, components,
dependencies, config files, CLI commands — and then, in the same file, a
reference instance model that populates that vocabulary for one example project
(`my-conforming-project`, plus a `ggen` crate section). There is no separate
ontology file split by concern; classes, properties, and instances all live in
`schema/praxis.ttl`.

The file declares five standard prefixes at the top: `rdf`, `rdfs`, `owl`, `xsd`,
and the project's own namespace:

```turtle
@prefix praxis:  <http://seanchatmangpt.github.io/praxis/schema#> .
```

(`schema/praxis.ttl:5`)

Everything defined below lives under that `praxis:` prefix.

## Classes

The file declares nine classes, all as `rdfs:Class` with an `rdfs:label` and
`rdfs:comment`, in a section explicitly headed `1. Classes` (`schema/praxis.ttl:7-9`):

| Class | Comment | Citation |
|---|---|---|
| `praxis:Project` | "A top-level repository or system configured under the Praxis house style." | `schema/praxis.ttl:11-13` |
| `praxis:RustCrate` | "A Rust crate within the project, either a single crate or workspace member." | `schema/praxis.ttl:15-17` |
| `praxis:Component` | "An architectural module or design pattern implemented in Rust code." | `schema/praxis.ttl:19-21` |
| `praxis:Feature` | "A Cargo feature flag in a Rust crate." | `schema/praxis.ttl:23-25` |
| `praxis:Dependency` | "An external crate dependency for Cargo.toml." | `schema/praxis.ttl:27-29` |
| `praxis:ConfigurationFile` | "A project hygiene config file (e.g. rustfmt.toml, deny.toml, typos.toml)." | `schema/praxis.ttl:31-33` |
| `praxis:WorkflowFile` | "A GitHub actions workflow definition." | `schema/praxis.ttl:35-37` |
| `praxis:ZstTypestate` | "A Zero-Sized Type used as a compile-time lifecycle state marker." | `schema/praxis.ttl:39-41` |
| `praxis:CliCommand` | "A noun-verb CLI command definition." | `schema/praxis.ttl:43-45` |

`praxis:Project` is the root: a project `hasCrate`, `hasConfig`, and
`hasWorkflow` (see below). `praxis:RustCrate` is the unit that `hasComponent`,
`hasFeature`, and `hasDependency`. `praxis:Component` in turn `hasTypestate`.
`praxis:CliCommand` is not attached to `praxis:Project` or `praxis:RustCrate` via
a `has*` property — instances relate to a crate through `praxis:inCrate`
instead (`schema/praxis.ttl:149-153`), and the reference model's two earliest
`CliCommand` instances (`praxis:CmdDodRun`, `praxis:CmdVerifierVerify`,
`schema/praxis.ttl:241-249`) don't even set `inCrate` — only the later
`ggen`-crate commands do (`schema/praxis.ttl:259-294`).

## Properties

The file's `2. Properties` section (`schema/praxis.ttl:47-49`) groups
properties under four comments: "Project Properties", "Relations", "CLI Command
Properties", and "Hygiene Config Properties". All are declared as `rdf:Property`
with an `rdfs:domain` and `rdfs:range`; several domains are `owl:unionOf` lists
spanning more than one class.

### Project properties

| Property | Domain | Range | Citation |
|---|---|---|---|
| `praxis:name` | union of `Project`, `RustCrate`, `Component`, `Feature`, `Dependency`, `ZstTypestate` | `xsd:string` | `schema/praxis.ttl:52-55` |
| `praxis:version` | union of `Project`, `RustCrate` | `xsd:string` | `schema/praxis.ttl:57-61` |
| `praxis:description` | union of `Project`, `RustCrate` | `xsd:string` | `schema/praxis.ttl:63-66` |
| `praxis:license` | union of `Project`, `RustCrate` | `xsd:string` | `schema/praxis.ttl:68-72` |
| `praxis:rustVersion` | union of `Project`, `RustCrate` | `xsd:string` | `schema/praxis.ttl:74-78` |
| `praxis:edition` | union of `Project`, `RustCrate` | `xsd:string` | `schema/praxis.ttl:80-84` |
| `praxis:dependencySpec` | `Dependency` | `xsd:string` | `schema/praxis.ttl:86-90` |
| `praxis:isWorkspace` | `Project` | `xsd:boolean` | `schema/praxis.ttl:92-95` |

Note that `praxis:name` is the single property shared across six different
classes (via the `owl:unionOf` domain at `schema/praxis.ttl:54`) — it is the one
property every named entity in the graph carries, and it is what SPARQL queries
over this graph would typically `SELECT` to get a human-readable label back for
any resource, regardless of its class.

`praxis:version`'s comment records the versioning convention directly in the
schema: "Project version adhering to CalVer (e.g. YY.M.patch)" (`schema/praxis.ttl:57-61`).
`praxis:rustVersion`'s comment gives an example MSRV of `'1.82'`
(`schema/praxis.ttl:74-78`), and the reference instance `praxis:MyConformingProject`
does in fact set `praxis:rustVersion "1.82"` (`schema/praxis.ttl:176`).

### Relations

| Property | Domain | Range | Citation |
|---|---|---|---|
| `praxis:hasCrate` | `Project` | `RustCrate` | `schema/praxis.ttl:98-101` |
| `praxis:hasComponent` | `RustCrate` | `Component` | `schema/praxis.ttl:103-106` |
| `praxis:hasFeature` | `RustCrate` | `Feature` | `schema/praxis.ttl:108-111` |
| `praxis:hasDependency` | `RustCrate` | `Dependency` | `schema/praxis.ttl:113-116` |
| `praxis:hasConfig` | `Project` | `ConfigurationFile` | `schema/praxis.ttl:118-121` |
| `praxis:hasWorkflow` | `Project` | `WorkflowFile` | `schema/praxis.ttl:123-126` |
| `praxis:hasTypestate` | `Component` | `ZstTypestate` | `schema/praxis.ttl:128-131` |

These seven `has*` properties form the object-property backbone of the graph:
`Project → RustCrate → Component → ZstTypestate`, with `Feature` and
`Dependency` hanging directly off `RustCrate`, and `ConfigurationFile` /
`WorkflowFile` hanging directly off `Project`.

### CLI command properties

| Property | Domain | Range | Citation |
|---|---|---|---|
| `praxis:noun` | `CliCommand` | `xsd:string` | `schema/praxis.ttl:134-137` |
| `praxis:verb` | `CliCommand` | `xsd:string` | `schema/praxis.ttl:139-142` |
| `praxis:handler` | `CliCommand` | `xsd:string` | `schema/praxis.ttl:144-147` |
| `praxis:inCrate` | `CliCommand` | `RustCrate` | `schema/praxis.ttl:149-153` |
| `praxis:flag` | `CliCommand` | `xsd:string` | `schema/praxis.ttl:155-159` |

`praxis:noun` and `praxis:verb` are what encode the "noun-verb CLI" pattern
that `praxis:CliCommand`'s own class comment names (`schema/praxis.ttl:43-45`):
every command instance is a `(noun, verb)` pair plus a `handler` string naming
the Rust function that implements it. `praxis:inCrate`'s comment states its
purpose precisely: "Which workspace crate a CLI command's noun-verb route is
generated into" (`schema/praxis.ttl:149-152`). `praxis:flag`'s comment likewise
is explicit about shape and wiring: "An optional boolean CLI flag (snake_case
name) exposed by the command and passed to its handler" (`schema/praxis.ttl:155-159`).

### Hygiene config properties

| Property | Domain | Range | Citation |
|---|---|---|---|
| `praxis:targetPath` | union of `ConfigurationFile`, `WorkflowFile` | `xsd:string` | `schema/praxis.ttl:162-165` |

A single property serves both hygiene-config and workflow-file instances,
pointing at the on-disk path the file is generated to.

## The reference instance model

The remainder of the file (`3. Reference Instance Model: my-conforming-project`,
starting `schema/praxis.ttl:167-169`) is not more schema — it's a worked
example: a fully populated instance graph showing what a project satisfying
this vocabulary looks like end to end.

`praxis:MyConformingProject` (`schema/praxis.ttl:171-181`) is a `praxis:Project`
with `name "my-conforming-project"`, `version "26.6.0"`, `license "MIT OR
Apache-2.0"`, `rustVersion "1.82"`, `edition "2021"`, `isWorkspace true`, two
crates (`praxis:CrateCore`, `praxis:CrateGgen`), three configs
(`praxis:ConfigRustfmt`, `praxis:ConfigDeny`, `praxis:ConfigTypos`), and two
workflows (`praxis:WorkflowCI`, `praxis:WorkflowRelease`) — all set via the
properties and relations tabulated above.

`praxis:CrateCore` (`schema/praxis.ttl:184-188`) is named `"my-conforming-project"`
(matching the project name — the crate and its containing project share a
name in this reference model) and carries three components
(`praxis:CompTypestates`, `praxis:CompLsp`, `praxis:CompCli`), two features
(`praxis:FeatLsp`, `praxis:FeatTypestate`), and two dependencies
(`praxis:DepLspMax`, `praxis:DepSerde`).

`praxis:CompTypestates` (`schema/praxis.ttl:191-193`) is named
`"GenerativeTypestates"` and has three typestates: `praxis:StateRaw`,
`praxis:StateValidated`, `praxis:StateAdmitted` (`schema/praxis.ttl:201-209`,
each named `"Raw"`, `"Validated"`, `"Admitted"` respectively) — this is the
`ZstTypestate` class in use, modeling a compile-time state-machine lifecycle
with three named states. `praxis:CompLsp` is named `"RulePackServer"`
(`schema/praxis.ttl:195-196`) and `praxis:CompCli` is named `"NounVerbCli"`
(`schema/praxis.ttl:198-199`).

Features: `praxis:FeatLsp` named `"lsp"` (`schema/praxis.ttl:212-213`),
`praxis:FeatTypestate` named `"typestate"` (`schema/praxis.ttl:215-216`).

Dependencies: `praxis:DepLspMax`, named `"lsp-max"` with
`dependencySpec "path = \"/Users/sac/lsp-max\""` (`schema/praxis.ttl:219-221`)
— a path dependency, matching the local `lsp-max` crate this file's own
`praxis:DepLspMax` instance names; and `praxis:DepSerde`, named `"serde"` with
`dependencySpec "1"` (`schema/praxis.ttl:223-225`) — a plain version-string
dependency.

Hygiene configs: `praxis:ConfigRustfmt` → `targetPath "rustfmt.toml"`
(`schema/praxis.ttl:228-230`), `praxis:ConfigDeny` → `targetPath "deny.toml"`
(`schema/praxis.ttl:232-234`), `praxis:ConfigTypos` → `targetPath "typos.toml"`
(`schema/praxis.ttl:236-238`).

CLI commands, first pair (no `inCrate` set): `praxis:CmdDodRun` —
noun `"dod"`, verb `"run"`, handler `"handle_dod_run"`
(`schema/praxis.ttl:241-244`); `praxis:CmdVerifierVerify` — noun `"verifier"`,
verb `"verify"`, handler `"handle_verifier_verify"` (`schema/praxis.ttl:246-249`).

Then a second, separately-headed block (`schema/praxis.ttl:251-253`, "ggen
crate (crates/ggen): SPARQL-in-Tera code generation") introduces
`praxis:CrateGgen` — named `"ggen"`, described as `"SPARQL-in-Tera code
generation: ggen sync as the single verb."` (`schema/praxis.ttl:255-257`) — and
four `praxis:CliCommand` instances that *do* set `inCrate praxis:CrateGgen`:

- `praxis:CmdGgenSyncRun` — noun `"sync"`, verb `"run"`, handler
  `"handle_sync_run"`, with two flags, `"dry_run"` and `"watch"`, and a comment
  describing the five-stage pipeline: "resolve, enrich, extract, render,
  write. --watch re-runs the pipeline on filesystem changes."
  (`schema/praxis.ttl:259-266`)
- `praxis:CmdGgenGraphValidate` — noun `"graph"`, verb `"validate"`, handler
  `"handle_graph_validate"`, comment: "Load a Turtle ontology and report parse
  or constraint violations." (`schema/praxis.ttl:268-273`)
- `praxis:CmdGgenReceiptVerify` — noun `"receipt"`, verb `"verify"`, handler
  `"handle_receipt_verify"`, comment: "Recompute and verify the BLAKE3 chain
  hash of a sync receipt." (`schema/praxis.ttl:275-280`)
- `praxis:CmdGgenReceiptHistory` — noun `"receipt"`, verb `"history"`, handler
  `"handle_receipt_history"`, comment: "Verify the full chain of sync receipts
  in .ggen-v2/receipt-log.jsonl." (`schema/praxis.ttl:282-287`)
- `praxis:CmdGgenDoctorRun` — noun `"doctor"`, verb `"run"`, handler
  `"handle_doctor"`, comment: "Check lockfile/pack drift, orphaned generated
  artifacts, and receipt-vs-disk staleness." (`schema/praxis.ttl:289-294`)

`praxis:CmdGgenReceiptVerify` and `praxis:CmdGgenReceiptHistory` share the same
`noun` (`"receipt"`) with different `verb`s (`"verify"` vs. `"history"`) — a
direct illustration, within the ontology's own instance data, of the
noun-verb pattern: one noun can route to multiple verbs, each with its own
handler.

The file ends at `schema/praxis.ttl:295` — 295 lines total, no further
sections.

## Correction of record

An earlier documentation set for this project (deleted at the start of this
book's rewrite) described a `pdl:LawObject` / `pdl:Obligation` vocabulary under
a prefix `pdl: <http://praxis.seanchatmangpt.com/ontology/lawobject#>`. That
vocabulary does not exist anywhere in `schema/praxis.ttl`: there is no `pdl:`
prefix declared in the file, and no `LawObject` or `Obligation` class or
property of any kind. A full-text search of the file for `pdl:` and for
`LawObject`/`lawobject` returns zero matches.

The file's only namespace prefix for project-defined terms is
`praxis:  <http://seanchatmangpt.github.io/praxis/schema#>` (`schema/praxis.ttl:5`),
and its actual vocabulary is the nine classes and properties documented above —
`Project`, `RustCrate`, `Component`, `Feature`, `Dependency`,
`ConfigurationFile`, `WorkflowFile`, `ZstTypestate`, `CliCommand`, and their
associated `name`/`hasCrate`/`noun`/`verb`/`handler`/`inCrate`/`flag`
properties. The `pdl:LawObject`/`pdl:Obligation` description was fictional
relative to this ontology and should not be relied on as a description of
Praxis's schema.
