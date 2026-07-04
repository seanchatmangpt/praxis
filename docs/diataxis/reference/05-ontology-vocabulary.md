# Ontology Vocabulary Reference

Source file: `/Users/sac/praxis/schema/praxis.ttl` (294 lines, read in full for this reference).

Namespace prefix: `praxis:` = `http://seanchatmangpt.github.io/praxis/schema#` (schema/praxis.ttl:5).

Standard prefixes used in the file: `rdf:` = `http://www.w3.org/1999/02/22-rdf-syntax-ns#` (schema/praxis.ttl:1), `rdfs:` = `http://www.w3.org/2000/01/rdf-schema#` (schema/praxis.ttl:2), `owl:` = `http://www.w3.org/2002/07/owl#` (schema/praxis.ttl:3), `xsd:` = `http://www.w3.org/2001/XMLSchema#` (schema/praxis.ttl:4).

## 1. Classes (`rdfs:Class`)

Every class declared in the file, in declaration order.

| Class (local name) | `rdfs:label` | `rdfs:comment` | Line |
|---|---|---|---|
| `praxis:Project` | "Project" | "A top-level repository or system configured under the Praxis house style." | schema/praxis.ttl:11-13 |
| `praxis:RustCrate` | "RustCrate" | "A Rust crate within the project, either a single crate or workspace member." | schema/praxis.ttl:15-17 |
| `praxis:Component` | "Component" | "An architectural module or design pattern implemented in Rust code." | schema/praxis.ttl:19-21 |
| `praxis:Feature` | "Feature" | "A Cargo feature flag in a Rust crate." | schema/praxis.ttl:23-25 |
| `praxis:Dependency` | "Dependency" | "An external crate dependency for Cargo.toml." | schema/praxis.ttl:27-29 |
| `praxis:ConfigurationFile` | "ConfigurationFile" | "A project hygiene config file (e.g. rustfmt.toml, deny.toml, typos.toml)." | schema/praxis.ttl:31-33 |
| `praxis:WorkflowFile` | "WorkflowFile" | "A GitHub actions workflow definition." | schema/praxis.ttl:35-37 |
| `praxis:ZstTypestate` | "ZstTypestate" | "A Zero-Sized Type used as a compile-time lifecycle state marker." | schema/praxis.ttl:39-41 |
| `praxis:CliCommand` | "CliCommand" | "A noun-verb CLI command definition." | schema/praxis.ttl:43-45 |

Total classes declared: 9.

## 2. Properties (`rdf:Property`)

Every property declared in the file, in declaration order. "Domain" reproduces the `rdfs:domain` value verbatim, including `owl:unionOf` blank-node unions where present.

| Property (local name) | `rdfs:label` | `rdfs:domain` | `rdfs:range` | `rdfs:comment` | Line |
|---|---|---|---|---|---|
| `praxis:name` | "name" | union of `praxis:Project`, `praxis:RustCrate`, `praxis:Component`, `praxis:Feature`, `praxis:Dependency`, `praxis:ZstTypestate` | `xsd:string` | (none) | schema/praxis.ttl:52-55 |
| `praxis:version` | "version" | union of `praxis:Project`, `praxis:RustCrate` | `xsd:string` | "Project version adhering to CalVer (e.g. YY.M.patch)." | schema/praxis.ttl:57-61 |
| `praxis:description` | "description" | union of `praxis:Project`, `praxis:RustCrate` | `xsd:string` | (none) | schema/praxis.ttl:63-66 |
| `praxis:license` | "license" | union of `praxis:Project`, `praxis:RustCrate` | `xsd:string` | "License type (e.g., 'MIT OR Apache-2.0')." | schema/praxis.ttl:68-72 |
| `praxis:rustVersion` | "rustVersion" | union of `praxis:Project`, `praxis:RustCrate` | `xsd:string` | "Minimum Supported Rust Version (e.g., '1.82')." | schema/praxis.ttl:74-78 |
| `praxis:edition` | "edition" | union of `praxis:Project`, `praxis:RustCrate` | `xsd:string` | "Rust edition (e.g., '2021')." | schema/praxis.ttl:80-84 |
| `praxis:dependencySpec` | "dependencySpec" | `praxis:Dependency` | `xsd:string` | "Cargo dependency specification string." | schema/praxis.ttl:86-90 |
| `praxis:isWorkspace` | "isWorkspace" | `praxis:Project` | `xsd:boolean` | (none) | schema/praxis.ttl:92-95 |
| `praxis:hasCrate` | "hasCrate" | `praxis:Project` | `praxis:RustCrate` | (none) | schema/praxis.ttl:98-101 |
| `praxis:hasComponent` | "hasComponent" | `praxis:RustCrate` | `praxis:Component` | (none) | schema/praxis.ttl:103-106 |
| `praxis:hasFeature` | "hasFeature" | `praxis:RustCrate` | `praxis:Feature` | (none) | schema/praxis.ttl:108-111 |
| `praxis:hasDependency` | "hasDependency" | `praxis:RustCrate` | `praxis:Dependency` | (none) | schema/praxis.ttl:113-116 |
| `praxis:hasConfig` | "hasConfig" | `praxis:Project` | `praxis:ConfigurationFile` | (none) | schema/praxis.ttl:118-121 |
| `praxis:hasWorkflow` | "hasWorkflow" | `praxis:Project` | `praxis:WorkflowFile` | (none) | schema/praxis.ttl:123-126 |
| `praxis:hasTypestate` | "hasTypestate" | `praxis:Component` | `praxis:ZstTypestate` | (none) | schema/praxis.ttl:128-131 |
| `praxis:noun` | "noun" | `praxis:CliCommand` | `xsd:string` | (none) | schema/praxis.ttl:134-137 |
| `praxis:verb` | "verb" | `praxis:CliCommand` | `xsd:string` | (none) | schema/praxis.ttl:139-142 |
| `praxis:handler` | "handler" | `praxis:CliCommand` | `xsd:string` | (none) | schema/praxis.ttl:144-147 |
| `praxis:inCrate` | "inCrate" | `praxis:CliCommand` | `praxis:RustCrate` | "Which workspace crate a CLI command's noun-verb route is generated into." | schema/praxis.ttl:149-153 |
| `praxis:flag` | "flag" | `praxis:CliCommand` | `xsd:string` | "An optional boolean CLI flag (snake_case name) exposed by the command and passed to its handler." | schema/praxis.ttl:155-159 |
| `praxis:targetPath` | "targetPath" | union of `praxis:ConfigurationFile`, `praxis:WorkflowFile` | `xsd:string` | (none) | schema/praxis.ttl:162-165 |

Total properties declared: 20.

## 3. Named Instances

Every named individual asserted in section 3 of the file ("Reference Instance Model: my-conforming-project", schema/praxis.ttl:167-169), in declaration order, with its `rdf:type` and every property-value pair asserted on it in the source.

### 3.1 Project instance

| Instance | `rdf:type` | Property | Value(s) | Line |
|---|---|---|---|---|
| `praxis:MyConformingProject` | `praxis:Project` | `praxis:name` | `"my-conforming-project"` | schema/praxis.ttl:171-172 |
| | | `praxis:version` | `"26.6.0"` | schema/praxis.ttl:173 |
| | | `praxis:description` | `"A standardized, structurally conforming boilerplate project."` | schema/praxis.ttl:174 |
| | | `praxis:license` | `"MIT OR Apache-2.0"` | schema/praxis.ttl:175 |
| | | `praxis:rustVersion` | `"1.82"` | schema/praxis.ttl:176 |
| | | `praxis:edition` | `"2021"` | schema/praxis.ttl:177 |
| | | `praxis:isWorkspace` | `true` | schema/praxis.ttl:178 |
| | | `praxis:hasCrate` | `praxis:CrateCore`, `praxis:CrateGgen` | schema/praxis.ttl:179 |
| | | `praxis:hasConfig` | `praxis:ConfigRustfmt`, `praxis:ConfigDeny`, `praxis:ConfigTypos` | schema/praxis.ttl:180 |
| | | `praxis:hasWorkflow` | `praxis:WorkflowCI`, `praxis:WorkflowRelease` | schema/praxis.ttl:181 |

Note: `praxis:WorkflowCI` and `praxis:WorkflowRelease` are referenced as objects of `praxis:hasWorkflow` at schema/praxis.ttl:181 but have no separate declaration block of their own anywhere else in the 294-line file — no `praxis:name`/`praxis:targetPath` assertions exist for them in this source.

### 3.2 RustCrate instances

| Instance | `rdf:type` | Property | Value(s) | Line |
|---|---|---|---|---|
| `praxis:CrateCore` | `praxis:RustCrate` | `praxis:name` | `"my-conforming-project"` | schema/praxis.ttl:184-185 |
| | | `praxis:hasComponent` | `praxis:CompTypestates`, `praxis:CompLsp`, `praxis:CompCli` | schema/praxis.ttl:186 |
| | | `praxis:hasFeature` | `praxis:FeatLsp`, `praxis:FeatTypestate` | schema/praxis.ttl:187 |
| | | `praxis:hasDependency` | `praxis:DepLspMax`, `praxis:DepSerde` | schema/praxis.ttl:188 |
| `praxis:CrateGgen` | `praxis:RustCrate` | `praxis:name` | `"ggen"` | schema/praxis.ttl:255-256 |
| | | `praxis:description` | `"SPARQL-in-Tera code generation: ggen sync as the single verb."` | schema/praxis.ttl:257 |

### 3.3 Component instances

| Instance | `rdf:type` | Property | Value(s) | Line |
|---|---|---|---|---|
| `praxis:CompTypestates` | `praxis:Component` | `praxis:name` | `"GenerativeTypestates"` | schema/praxis.ttl:191-192 |
| | | `praxis:hasTypestate` | `praxis:StateRaw`, `praxis:StateValidated`, `praxis:StateAdmitted` | schema/praxis.ttl:193 |
| `praxis:CompLsp` | `praxis:Component` | `praxis:name` | `"RulePackServer"` | schema/praxis.ttl:195-196 |
| `praxis:CompCli` | `praxis:Component` | `praxis:name` | `"NounVerbCli"` | schema/praxis.ttl:198-199 |

### 3.4 ZstTypestate instances

| Instance | `rdf:type` | Property | Value(s) | Line |
|---|---|---|---|---|
| `praxis:StateRaw` | `praxis:ZstTypestate` | `praxis:name` | `"Raw"` | schema/praxis.ttl:202-203 |
| `praxis:StateValidated` | `praxis:ZstTypestate` | `praxis:name` | `"Validated"` | schema/praxis.ttl:205-206 |
| `praxis:StateAdmitted` | `praxis:ZstTypestate` | `praxis:name` | `"Admitted"` | schema/praxis.ttl:208-209 |

### 3.5 Feature instances

| Instance | `rdf:type` | Property | Value(s) | Line |
|---|---|---|---|---|
| `praxis:FeatLsp` | `praxis:Feature` | `praxis:name` | `"lsp"` | schema/praxis.ttl:212-213 |
| `praxis:FeatTypestate` | `praxis:Feature` | `praxis:name` | `"typestate"` | schema/praxis.ttl:215-216 |

### 3.6 Dependency instances

| Instance | `rdf:type` | Property | Value(s) | Line |
|---|---|---|---|---|
| `praxis:DepLspMax` | `praxis:Dependency` | `praxis:name` | `"lsp-max"` | schema/praxis.ttl:219-220 |
| | | `praxis:dependencySpec` | `"path = \"/Users/sac/lsp-max\""` | schema/praxis.ttl:221 |
| `praxis:DepSerde` | `praxis:Dependency` | `praxis:name` | `"serde"` | schema/praxis.ttl:223-224 |
| | | `praxis:dependencySpec` | `"1"` | schema/praxis.ttl:225 |

### 3.7 ConfigurationFile instances

| Instance | `rdf:type` | Property | Value(s) | Line |
|---|---|---|---|---|
| `praxis:ConfigRustfmt` | `praxis:ConfigurationFile` | `praxis:name` | `"rustfmt"` | schema/praxis.ttl:228-229 |
| | | `praxis:targetPath` | `"rustfmt.toml"` | schema/praxis.ttl:230 |
| `praxis:ConfigDeny` | `praxis:ConfigurationFile` | `praxis:name` | `"deny"` | schema/praxis.ttl:232-233 |
| | | `praxis:targetPath` | `"deny.toml"` | schema/praxis.ttl:234 |
| `praxis:ConfigTypos` | `praxis:ConfigurationFile` | `praxis:name` | `"typos"` | schema/praxis.ttl:236-237 |
| | | `praxis:targetPath` | `"typos.toml"` | schema/praxis.ttl:238 |

### 3.8 CliCommand instances

All `praxis:CliCommand` individuals declared in the file, in declaration order.

| Instance | `rdf:type` | `praxis:inCrate` | `praxis:noun` | `praxis:verb` | `praxis:handler` | `praxis:flag` | `rdfs:comment` | Line |
|---|---|---|---|---|---|---|---|---|
| `praxis:CmdDodRun` | `praxis:CliCommand` | (none asserted) | `"dod"` | `"run"` | `"handle_dod_run"` | (none) | (none) | schema/praxis.ttl:241-244 |
| `praxis:CmdVerifierVerify` | `praxis:CliCommand` | (none asserted) | `"verifier"` | `"verify"` | `"handle_verifier_verify"` | (none) | (none) | schema/praxis.ttl:246-249 |
| `praxis:CmdGgenSyncRun` | `praxis:CliCommand` | `praxis:CrateGgen` | `"sync"` | `"run"` | `"handle_sync_run"` | `"dry_run"`, `"watch"` | "Run the five-stage generation pipeline: resolve, enrich, extract, render, write. --watch re-runs the pipeline on filesystem changes." | schema/praxis.ttl:259-266 |
| `praxis:CmdGgenGraphValidate` | `praxis:CliCommand` | `praxis:CrateGgen` | `"graph"` | `"validate"` | `"handle_graph_validate"` | (none) | "Load a Turtle ontology and report parse or constraint violations." | schema/praxis.ttl:268-273 |
| `praxis:CmdGgenReceiptVerify` | `praxis:CliCommand` | `praxis:CrateGgen` | `"receipt"` | `"verify"` | `"handle_receipt_verify"` | (none) | "Recompute and verify the BLAKE3 chain hash of a sync receipt." | schema/praxis.ttl:275-280 |
| `praxis:CmdGgenReceiptHistory` | `praxis:CliCommand` | `praxis:CrateGgen` | `"receipt"` | `"history"` | `"handle_receipt_history"` | (none) | "Verify the full chain of sync receipts in .ggen-v2/receipt-log.jsonl." | schema/praxis.ttl:282-287 |
| `praxis:CmdGgenDoctorRun` | `praxis:CliCommand` | `praxis:CrateGgen` | `"doctor"` | `"run"` | `"handle_doctor"` | (none) | "Check lockfile/pack drift, orphaned generated artifacts, and receipt-vs-disk staleness." | schema/praxis.ttl:289-294 |

Total `praxis:CliCommand` instances: 7 (2 without `praxis:inCrate`, 5 with `praxis:inCrate praxis:CrateGgen`).

## 4. Instance Count Summary

| Class | Named instances in file | Line range of block |
|---|---|---|
| `praxis:Project` | 1 (`praxis:MyConformingProject`) | schema/praxis.ttl:171-181 |
| `praxis:RustCrate` | 2 (`praxis:CrateCore`, `praxis:CrateGgen`) | schema/praxis.ttl:184-188, 255-257 |
| `praxis:Component` | 3 (`praxis:CompTypestates`, `praxis:CompLsp`, `praxis:CompCli`) | schema/praxis.ttl:191-199 |
| `praxis:ZstTypestate` | 3 (`praxis:StateRaw`, `praxis:StateValidated`, `praxis:StateAdmitted`) | schema/praxis.ttl:202-209 |
| `praxis:Feature` | 2 (`praxis:FeatLsp`, `praxis:FeatTypestate`) | schema/praxis.ttl:212-216 |
| `praxis:Dependency` | 2 (`praxis:DepLspMax`, `praxis:DepSerde`) | schema/praxis.ttl:219-225 |
| `praxis:ConfigurationFile` | 3 (`praxis:ConfigRustfmt`, `praxis:ConfigDeny`, `praxis:ConfigTypos`) | schema/praxis.ttl:228-238 |
| `praxis:CliCommand` | 7 (`praxis:CmdDodRun`, `praxis:CmdVerifierVerify`, `praxis:CmdGgenSyncRun`, `praxis:CmdGgenGraphValidate`, `praxis:CmdGgenReceiptVerify`, `praxis:CmdGgenReceiptHistory`, `praxis:CmdGgenDoctorRun`) | schema/praxis.ttl:241-294 |
| `praxis:WorkflowFile` | 0 declared as standalone blocks (only referenced as objects of `praxis:hasWorkflow`, schema/praxis.ttl:181) | — |
