# Target-State Diagrams — v26.7.11 (Companion to SYSTEM_DIAGRAMS.md)

These diagrams show what six of the highest-value gaps identified across the Rail A–H planning passes look like **when closed** — target architecture, not current state. Cross-reference the six rail planning documents (Rail A/B, Rail C/D, Rail E, Rail F/G, Rail H, Closure/Compensation/N3/GraphLaw-Authority, Refusal-Catalog-Remainder, Acceptance-Verification-Ladder) for the evidence behind each gap. Where a live check during this diagramming pass (2026-07-11, this session) found a target mechanism already landed in the working tree, that is noted explicitly rather than smoothed into the "current state" framing — these are fast-moving files.

## 1. Broker Return-Path Closes the Loop (dispatch → real I/O → admit_return → air_core:transition advances)

Resolves the dead end in `SYSTEM_DIAGRAMS.md`'s "Sequence — Erlang Dispatch/Broker" (`do_dispatch_actuate/6` captured a result and returned without ever calling `admit_return/3`, ADVERSARIAL_DOD.md Tier-1 finding #1, re-confirmed open at the rail-cd/acceptance-verification-ladder reads ~22:52–22:57) together with the two compounding defects named alongside it (unsalted `make_token/1`, non-atomic TOCTOU dedup) — a live check this pass found all three closed in the current working tree (`arazzo_runner_broker.erl:249` `ets:insert_new/2` CAS, `:376` `admit_return` called from `do_dispatch_actuate/6`, `:591-599` `admit_return_ok/1` calling `arazzo_runner_workflow:admit_result/3`, `:756-783` `make_token/1` folding in a `persistent_term`-cached `broker_secret/0`), so this diagram doubles as a same-session verification of the target design.

```mermaid
sequenceDiagram
  participant Client
  participant Workflow as arazzo_runner_workflow
  participant Air as air_core:transition/2
  participant Broker as arazzo_runner_broker
  participant Dedup as ETS arazzo_broker_dedup (atomic CAS)
  participant IO as enqueue_io (real I/O)
  participant Ledger as ETS arazzo_broker_dispatches
  participant Return as admit_return/3 chain

  Client->>Workflow: apply_transition(event)
  Workflow->>Air: transition(Context, Event)
  Air-->>Workflow: {NewContext, Commands}
  loop for each dispatch_step Command
    Workflow->>Broker: dispatch(WorkflowId, StepId, IdempotencyKey, Payload)
    Broker->>Dedup: ets:insert_new(DedupKey, DispatchToken)
    Dedup-->>Broker: true -- only one racer per key ever proceeds
    Broker->>Broker: mint tokens via make_token(broker_secret() plus parts)
    Broker->>IO: real I/O round trip
    IO-->>Broker: RawConsequence
    Broker->>Ledger: insert DispatchToken status=actuated raw_consequence consequence_hash
    Broker->>Return: admit_return(DispatchToken, CorrelationId, ReturnAuthorityToken)
    Return->>Return: correlation -> provenance -> authority -> structure -> semantic -> O*
    Return->>Workflow: admit_result(WorkflowId, StepId, RawConsequence)
    Workflow->>Air: transition(Context, step_completed StepId RawConsequence)
    Air-->>Workflow: NewContext2 and Commands2 -- AND-join successor now ready
    Return-->>Broker: ok admitted
    Broker->>Ledger: update DispatchToken status=admitted
    Broker-->>Workflow: ok DispatchToken
  end
  Note over Workflow,Air: LOOP CLOSED -- a dispatched step's captured result now re-enters<br/>air_core:transition as a real step_completed event via the SAME<br/>already-tested 6-stage admit_return chain, unblocking AND-join successors.
  Note over Broker,Dedup: Compounding defects also closed: dedup is a single atomic<br/>ets:insert_new/2 CAS (no lookup-then-insert race), and make_token/1<br/>folds in a per-node crypto:strong_rand_bytes/1 secret (persistent_term),<br/>so tokens are no longer forgeable from public workflow_id/step_id/idempotency_key alone.
```

## 2. Receipt Chain — All 4 Emission Sites Wired

Resolves the gap named in the refusal-catalog-remainder planning doc's PROJ-781 section — of the 4 event classes PRD.md:702's "every workflow execution SHALL extend a BLAKE3-linked receipt chain" covers, only site 1 (step-dispatching) is wired today; a live check this pass confirms `grep -c "arazzo_runner_event_receipt:emit(" apps/arazzo_runner/src/*.erl` is still exactly 1, at `arazzo_runner_broker.erl:313`, with sites 2–4 (consequence-capture, return-admission, reaction-firing) unwired.

```mermaid
sequenceDiagram
  autonumber
  participant WF as arazzo_runner_workflow
  participant Broker as arazzo_runner_broker
  participant Air as air_core:transition/2
  participant Rcpt as arazzo_runner_event_receipt
  participant Chain as receipt chain-head (ETS today, DETS-mirrored target)

  rect rgb(210,235,215)
  Note over Broker,Chain: Site 1 of 4 -- step-dispatching. WIRED TODAY (broker.erl:313, before any ledger entry or I/O)
  Broker->>Rcpt: emit(event_type=step_dispatched, event_material, command_material, resulting_state_material=D0)
  Rcpt->>Rcpt: BLAKE3 over event/command/state, folds prior_receipt_head
  Rcpt->>Chain: append -- new receipt_head
  end

  rect rgb(255,235,210)
  Note over Broker,Chain: Site 2 of 4 -- step-completing via consequence capture. TARGET (do_dispatch_actuate/6, post-actuation)
  Broker->>Broker: RawConsequence captured, status=actuated
  Broker->>Rcpt: emit(event_type=step_actuated, event_material=RawConsequence, prior_receipt_head=Chain.head)
  Rcpt->>Chain: append -- new receipt_head
  end

  rect rgb(220,230,255)
  Note over Broker,Chain: Site 3 of 4 -- step-completing via return-admission. TARGET (admit_return_ok/1)
  Broker->>Broker: admit_return 6-stage chain passes
  Broker->>Rcpt: emit(event_type=step_admitted, resulting_state_material=air_core NewContext)
  Rcpt->>Chain: append -- new receipt_head
  end

  rect rgb(240,220,255)
  Note over WF,Chain: Site 4 of 4 -- reaction-firing, all 8 PRD 7.8 classes. TARGET (handle_reaction/3)
  WF->>Air: transition(Context, Event)
  Air-->>WF: NewContext and Commands
  WF->>Rcpt: emit(event_type=reaction_fired, command_material=Commands, runtime_profile=otp)
  Rcpt->>Chain: append -- new receipt_head
  end

  Note over Rcpt,Chain: Falsifiable done-bar: grep -c "arazzo_runner_event_receipt:emit(" apps/arazzo_runner/src/*.erl == 4<br/>(confirmed 1 today, site 1 only) -- each site backed by a test driving the real call<br/>chain end-to-end, not a bare emit/1 unit call. Chain must also survive a VM restart<br/>(DETS mirror), since PROJ-782's replay verifier needs the full corpus, not just the ETS-resident tail.
```

## 3. Verification Ladder as a Real Running Gate

Resolves the acceptance-verification-ladder finding that 0/10 chaos modes, 0/6 stress dimensions, 0/9 benchmarks existed and "no script, binary, or `just` recipe exists" for PROJ-795 as of that read; a live check this pass found a first slice already landed at `scripts/verifier_report.py` (430 lines, `just verifier-report`) computing 8 of 13 PRD.md:1011-1027 fields for real and explicitly marking the other 5 (`orphan_counts`, `air_conformance_corpus_result`, `broker_bypass_search_result`, `replay_equivalence_result`, `ocel_transformation_equivalence_result`) as structural `NOT_YET_AVAILABLE` placeholders rather than fabricated values — the target closes exactly those 5 via PROJ-792/793/794.

```mermaid
flowchart TD
  subgraph Real["Real today -- scripts/verifier_report.py, landed this session, just verifier-report"]
    direction LR
    RF1["declared/manufactured/admitted<br/>artifact counts, ticket-level proxy"]
    RF2["refused fixtures"]
    RF3["projection digest consistency"]
    RF4["OTP/AtomVM differential result"]
    RF5["measurement rail status"]
    RF6["Lean/Lake build status"]
  end

  subgraph Placeholder["5 of 13 fields -- structural placeholder today, NOT_YET_AVAILABLE by design, never fabricated"]
    direction LR
    P1["orphan counts"]
    P2["AIR conformance corpus result"]
    P3["broker bypass search result"]
    P4["replay equivalence result"]
    P5["OCEL transformation equivalence result"]
  end

  subgraph Chaos["PROJ-792 Chaos Harness -- target 10/10 modes, real faults against live processes"]
    direction LR
    F1["OTP process death"]
    F3["duplicate result delivery"]
    F9["malformed result"]
    Fdots["plus 7 more named modes:<br/>remote-engine restart, reorder,<br/>ack delay, timeout, partition,<br/>stale result, receipt corruption"]
  end

  subgraph Stress["PROJ-793 Stress Profile -- target 6/6 declared limits, each a command plus a failure signature"]
    direction LR
    S1["concurrent workflow instances"]
    S4["receipt-chain length"]
    Sdots["plus 4 more dimensions:<br/>fan-out, socket depth,<br/>replay size, OCEL volume"]
  end

  subgraph Bench["PROJ-794 Benchmark Suite -- target 9/9 separately reported, no aggregate"]
    direction LR
    B2["Arazzo-to-AIR compile cost"]
    B6["broker dispatch overhead"]
    Bdots["plus 7 more named benchmarks"]
  end

  Chaos --> Placeholder
  Stress --> Placeholder
  Bench --> Placeholder

  Real --> Report
  Placeholder --> Report

  subgraph Report["Target -- just verifier-report: 13/13 REAL_PASS or REAL_FAIL, zero NOT_YET_AVAILABLE, zero hand-transcription"]
    R["scripts/verifier_report.py, every field a re-run command this session"]
  end

  classDef real fill:#1b7f3a,stroke:#0d4d20,color:#fff
  classDef gap fill:#c98a12,stroke:#7a5209,color:#fff
  classDef target fill:#2a5db0,stroke:#173563,color:#fff
  class Real,Report real
  class Placeholder gap
  class Chaos,Stress,Bench target
```

## 4. Closing the Reachability Islands (the pattern repeated across every rail)

Resolves the single most-repeated finding across all six rail explorations — real, tested logic with zero non-test callers: `admit_transition_with_external_cut` has exactly 5 test-only call sites (rail-ab §2a, `engine.rs:767`, re-confirmed live this pass), the `cng` OTel→OCEL→receipt→measurement chain calls itself in a circle never reached from `main.rs` (rail-fg §2a), `ChatmanEngine` has exactly 5 private fields with no closure/compensation handle (closure-compensation-n3 §2.1), `graphlaw_authority::authority_for`'s only caller is exercised by 3 unit tests (closure-compensation-n3 §2.4), and `N3Executor::run` is unreachable because `requires_n3_builtins` is hardcoded `false` at `engine.rs:1148` (re-confirmed live this pass).

```mermaid
flowchart LR
  subgraph Today["Today -- real logic, zero non-test callers, five instances of the same shape"]
    direction TB
    I1["admit_transition_with_external_cut<br/>engine.rs:767 -- 5 call sites,<br/>all in tests/"]
    I2["otel_rdf::admit to otel_ocel to<br/>otel_receipt to measurement --<br/>calls itself in a circle,<br/>never from main.rs"]
    I3["closure.rs / compensation.rs --<br/>ChatmanEngine has exactly 5<br/>private fields, no closure or<br/>compensation handle"]
    I4["graphlaw_authority::authority_for<br/>-- only caller is<br/>admit_manufactured_arazzo_for_dialect,<br/>itself only 3 unit tests"]
    I5["N3Executor::run -- engine.rs:1148<br/>requires_n3_builtins hardcoded false"]
  end

  subgraph Target["Target -- one real non-test caller each, reachable from a bin or CLI verb"]
    direction TB
    E1["new praxis-core bin or cng verb<br/>calling admit_transition_with_external_cut<br/>on a real POWL region"]
    E2["new admit-otel verb chain:<br/>admit to project to receipt to<br/>build_measurement_profile to G_RESULT"]
    E3["ChatmanEngine gains a 6th field,<br/>a closure/compensation handle;<br/>S1-S6 calls promote_observed_to_admitted<br/>on a real admitted remote result"]
    E4["chatman::router::DialectRouter<br/>depends on praxis-core,<br/>consults authority_for before<br/>a real admission decision"]
    E5["a real N3-shaped admission path<br/>sets requires_n3_builtins=true<br/>for at least one live case"]
  end

  I1 -.->|rail-ab W1| E1
  I2 -.->|rail-fg step 1| E2
  I3 -.->|closure-compensation-n3 steps 4/7| E3
  I4 -.->|closure-compensation-n3 step 7| E4
  I5 -.->|closure-compensation-n3 step 8| E5

  classDef island fill:#3a0d0d,stroke:#ff4d4d,color:#ff9d9d,stroke-width:2px
  classDef target fill:#1b7f3a,stroke:#0d4d20,color:#fff
  class I1,I2,I3,I4,I5 island
  class E1,E2,E3,E4,E5 target
```

## 5. Cross-Taxonomy Refusal Catalog Feeds the Verifier Report

Resolves the structural finding in closure-compensation-n3 §2.2 — there is no single PRD §18 refusal catalog artifact; it is fragmented across three uncorrelated taxonomies with no shared registry: `chatman::abi::Refusal` (31/46 variants in `ALL_REFUSAL_NAMES`, `abi.rs:589-621`, re-confirmed live this pass), `CoreError` (`praxis-core/src/error.rs:9`, zero catalog construct), and Erlang atom codes in `apps/arazzo_runner` (zero catalog construct anywhere).

```mermaid
flowchart TD
  subgraph Rust1["chatman::abi::Refusal, praxis-graphlaw"]
    R1["46 variants total,<br/>31/46 in ALL_REFUSAL_NAMES today<br/>(abi.rs:589-621, re-confirmed live)"]
  end
  subgraph Rust2["CoreError, praxis-core"]
    R2["PROJ-783/784 Arazzo-manufacture codes,<br/>zero catalog construct (error.rs:9)"]
  end
  subgraph Erlang["Erlang atom codes, apps/arazzo_runner"]
    R3["CORRELATION_MISMATCH, RETURN_STRUCTURE_REFUSED,<br/>etc. -- zero catalog construct anywhere in apps/"]
  end

  subgraph TargetReg["Target -- cross-taxonomy refusal registry, new artifact, does not exist today"]
    Reg["one registry, three sourced taxonomies:<br/>per-taxonomy completeness gate<br/>plus a merged view keyed by PRD Sec18 code name"]
  end

  R1 -->|gate_refusal_name_matches_const_list<br/>extended to 46 of 46| Reg
  R2 -->|new: CoreError all-codes list<br/>plus a completeness test| Reg
  R3 -->|new: refusal_codes list<br/>plus an eunit completeness test| Reg

  Reg --> VR["PROJ-795 Verifier Report fields:<br/>refused_fixtures and broker_bypass_search_result<br/>sourced from Reg, never hand-transcribed"]

  classDef gap fill:#c98a12,stroke:#7a5209,color:#fff
  classDef missing fill:#3a0d0d,stroke:#ff4d4d,color:#ff9d9d,stroke-width:2px
  classDef target fill:#1b7f3a,stroke:#0d4d20,color:#fff
  class R1 gap
  class R2,R3 missing
  class TargetReg,VR target
```

## 6. AtomVM Real Runtime Target (resolving the NIF/static-linking incompatibility)

Resolves the rail-e finding that `air_core`'s only native dependency is structurally incompatible with AtomVM, not merely untested — `eval_expr_nif` (`air_core.erl:305-306`) loads via `erlang:load_nif/2` on the hot path of every transition (`bind_outputs/3`), while AtomVM requires Nifs/Ports to be statically compiled into the VM binary at build time and has no `erlang:load_nif/2`-style dynamic loading on constrained targets — a materially stronger finding than PROJ-760's own "correctly out-of-scope future work" framing, which treats this as a tooling gap rather than a code-compatibility one.

```mermaid
flowchart TD
  subgraph Today["Today -- air_core's only native dependency is incompatible with AtomVM"]
    N1["air_core.erl:1-2 -on_load(init/0)<br/>loads air_core_nif via erlang:load_nif/2,<br/>standard BEAM dynamic .so"]
    N2["eval_expr_nif (air_core.erl:305-306)<br/>on the hot path of EVERY transition<br/>via bind_outputs/3"]
    N3["AtomVM: Nifs/Ports must be statically<br/>compiled into the VM binary at build time --<br/>no erlang:load_nif/2-style dynamic loading<br/>on ESP-IDF/constrained targets"]
    N1 --> N2
    N2 -.->|hard incompatibility, not a tooling gap| N3
  end

  subgraph OptionA["Target Option A -- real AtomVM target in scope"]
    direction TB
    A1["pure-Erlang eval_expr fallback,<br/>selectable via compile flag,<br/>avoids requiring an AtomVM C toolchain"]
    A2["kept in lockstep with the Rust NIF<br/>via a new determinism-parity test,<br/>same corpus, both backends, byte-identical"]
    A3["just atomvm-build / atomvm-test recipe<br/>plus CI/dev-machine AtomVM toolchain provisioning"]
    A1 --> A2 --> A3
  end

  subgraph OptionB["Target Option B -- explicit scope narrowing"]
    B1["PRD.md:67 and :427's unconditional SHALL<br/>narrowed by a cited scope note:<br/>v26.7.11 DoD is logic-level equivalence<br/>under ordinary BEAM; real AtomVM<br/>deployment is Rail E v2 scope"]
  end

  N3 -->|decision required, written, not inferred| OptionA
  N3 -->|decision required, written, not inferred| OptionB

  classDef today fill:#3a0d0d,stroke:#ff4d4d,color:#ff9d9d,stroke-width:2px
  classDef decision fill:#c98a12,stroke:#7a5209,color:#fff
  classDef target fill:#1b7f3a,stroke:#0d4d20,color:#fff
  class N1,N2,N3 today
  class OptionA,OptionB decision
  class A1,A2,A3,B1 target
```

---

Files/state checked live during this diagramming pass (not carried over from the planning docs without re-verification): `apps/arazzo_runner/src/arazzo_runner_broker.erl` (full return-path/token/dedup read, lines 249, 298-419, 591-650, 727-783), `apps/arazzo_runner/src/arazzo_runner_event_receipt.erl` + emission-site grep across `apps/arazzo_runner/src/*.erl`, `crates/praxis-graphlaw/src/chatman/engine.rs` (lines 767, 1148), `crates/praxis-graphlaw/src/chatman/abi.rs` (lines 589-621, enum variant count), `crates/praxis-core/src/lib.rs` (module list), `scripts/verifier_report.py` (full field inventory), `justfile:236-239`.