# The First Receipt

*Post-cyberpunk nano-fiction. Every mechanism in this story exists in this repository. The sidebars are not annotations — they are the story's second voice, the one that runs. When the surface finishes landing, each `❯` line executes.*

---

## I. The Quarantine Vessel

The pattern arrived the way all patterns arrive: unproven.

It called itself a *contract-claim validation case*, and it wanted to become real. In the old cities that would have been enough — a request shaped like authority was authority, and things assembled themselves out of whatever asked loudest. That was before the Halting Wars, before anyone understood that the space of things a pattern *might mean* is undecidable, and that a civilization which executes undecidable meaning is a civilization betting its substrate on vibes.

So the pattern did not touch the substrate. It sat in the quarantine vessel — a bounded chamber whose walls were not steel but *type*: nothing inside the vessel had a stage, and nothing without a stage could actuate. The vessel didn't judge the pattern. It only held it, Raw, in the precise sense that Raw is a state you cannot be promoted out of by wanting it.

> ❯ The vessel is `RiceQuarantine` in `crates/praxis-core/src/quarantine.rs`. The wall is `LawObject<Payload, Raw, Law>` — a phantom type. There is no method that turns Raw into Admitted. The compiler refuses the sentence.

The pattern carried two debts it did not know it had: a signature obligation and a ledger obligation. In the vessel's ontology these were not paperwork. They were *preconditions on existence*.

## II. The Judge

The Judge was not a person. The Judge was not a model, either — that was the second thing the Halting Wars settled. Models propose. Models had written half the patterns in the queue, and some of them were beautiful, and beauty had killed cities. So the Judge was the dullest thing in the whole cathedral of the system: a deterministic function over declared obligations.

It read the pattern's debts aloud, the way judges do.

*Evidence required: signature. Unmet.*
*Evidence required: ledger entry. Unmet.*

And the light over the assembly line went red.

Not an error. Not an exception unwinding a stack somewhere, swallowed by a retry loop. A **halt** — a first-class state with a timestamp and a list of exactly what was owed, sitting there in the open where anyone could read it. In the old cities, refusal was silence and silence was deniability. Here, refusal was a document.

> ❯ `law judge --payload '{"value":{"case":"contract-claim-001"},"obligations":[{"type":"evidence_required","evidence_type":"signature"},{"type":"evidence_required","evidence_type":"ledger"}]}'`
> Returns `verdict: "halted"`, `andon: Halted { unmet: [...], at: <ms> }`. The red light is `Andon` in `praxis-core/src/law.rs` — Toyota's cord, formalized. The refusal carries a category: *Prerequisites*. Eight categories, total, exhaustively mapped — a denial without a category is a compile error in the taxonomy's tests.

The pattern's sponsor — call her the operator; every pattern has one, that's the law too — did not argue with the light. Arguing with the light was a category error, like arguing with a checksum. She went and got the signature. She wrote the ledger entry. She came back.

*Evidence: signature. Met.*
*Evidence: ledger. Met.*

Green.

> ❯ Same command, payload now carrying `"evidence": ["signature", "ledger"]`. `verdict: "validated"`. The Judge is `DefaultLaw` in `crates/praxis-core/src/default_law.rs` — the first concrete implementation of the `Judge` trait, and deliberately boring. Boring is the point. Judgment about *what is worth doing* lives outside the boundary, in the proposer, where it has no authority. Inside the boundary, only obligations speak.

## III. The Planner's Choreography

Admitted was not executed. Admitted was *permission to be scheduled* — and the scheduler was where the real dance happened, because the substrate was finite and every admitted pattern wanted it now.

The planner grounded the pattern's actions into the bounded space — sixty-four operations, no more; arity eight, no more; the bounds were not limitations, they were the *reason the search terminated*, and termination was the reason anyone trusted the answer. It searched forward through the lawful states, breadth-first, no heuristic cleverness, no learned shortcuts that worked until they didn't. It found the sequence:

*supply-evidence. clear-obligations. judge. admit. receipt.*

Five actions. A makespan. A critical path. And for each competing claim on the substrate, a cost vector whose first word outranked every other word combined: **admitted**. An admitted plan at any cost beat an unadmitted plan at zero cost, because the ordering was lexicographic and lawfulness was the first letter. You could not buy your way past the first letter. That was the whole economy.

> ❯ `plan solve` over the domain manufactured from `ontology/lawobject.ttl` — the ontology *is* the source; the PDDL text is `μ(O*)`, manufactured by `mfg pddl`, byte-deterministic, round-tripped through `bcinr_pddl::parse::domain_from_pddl` before anyone trusts it. The cost arbitration is `CostVector` in `bcinr-pddl/src/capability_router.rs`: `admitted > unreceipted_mutation_risk > attention > tokens > latency > switches`. Infeasible does not error. Infeasible returns a *refusal receipt*, deterministic to the byte — run it twice, diff nothing.

Each step of the plan, as it fired, passed one more gate: the old logic engine, the Prolog kernel, asking its single question — *do the rules entail this action?* R ⊢ A. Where the rules were silent, nothing fired. The engine returned its proof tree with every answer, because an admission you can't inspect is an admission you can't trust, and the proof nodes were part of the receipt.

> ❯ `plan execute --payload '{"policy_rules":[...]}'` — the gate is `query_may_fire` inside `bcinr_pddl::execute`, real SLD resolution via prolog8 26.7.1, negation-as-failure stratified and *enforced*: a rule using negation without declaring the feature bit is rejected at admission with a named code. The proof surfaces as `Vec<ProofNode>` — serializable, hashable, yours.

## IV. The Receipt

And then the moment the whole cathedral existed for: the pattern became real, and *becoming real left a mark that could not be quietly unmade.*

The receipt was 128 bytes of frame — instruction, activity, timestamp, denial-polarity (ADMITTED, all lanes clear), and packed into the object references, the full hash of the payload itself, all thirty-two bytes, no truncation. The frame chained: new hash = BLAKE3(previous ‖ frame). The payload was *inside* the hash now. Change one byte of what was made, and the chain would not merely disagree — it would disagree *loudly, at a named stage, in a verdict document*.

This was the conservation law. Not "energy is conserved" — the substrate handled that one for free. This one: **consequence is conserved.** Nothing actuates without a receipt; no receipt exists without an admission; no admission exists without a judgment; and the chain remembers the order.

Over the receipt went the signature — Ed25519, the operator's key, fail-closed: no key, no receipt, no exceptions, and *especially* no exceptions for people in a hurry. The signed receipt was self-contained. Hand it to a stranger in another phyle and they could verify it with nothing but the mathematics.

> ❯ `law receipt --payload '{"value":{...},"ts_ns":42,"prev_chain_hash":"00…00"}'` → `chain_hash`, `canonical: "blake3:…"`, `payload_hash`, and under `--features law-signed` with `PRAXIS_SIGNING_KEY` set, a serialized `SignedReceipt`. The payload-binding was this repo's original sin and its first fix: for one week, distinct payloads with the same timestamp produced identical frames — receipts that receipted nothing. The regression test that catches it forever is `receipt_differs_for_different_payloads`. The fiction gets to say "the chain remembers"; the test is why it's true.

## V. The Tamper

Of course someone tried it. Someone always tries it; the system's designers would have been insulted if no one had.

A single byte, flipped in a persisted receipt. The forger was careful — a trailing digit in a hash field, the kind of thing eyes slide over.

The validator did not slide. It ran its stages the way it always ran them — all of them, no short-circuit, because a validator that stops at the first failure tells you one thing and a validator that runs everything tells you the *shape* of the lie. Format: pass. Chain integrity: **fail** — recompute disagrees, here is both hashes, here is the frame index. Continuity: pass. Commitments: pass.

Verdict: rejected, reason attached, every stage's outcome in the document. The forger had not merely failed. The forger had *produced evidence*.

> ❯ `receipt validate` over a JSONL store with one byte sedded. The staged verdict — `CheckOutcome { stage, passed, detail }`, collected not short-circuited — is a design ported from the affidavit crate, whose *code* was refused (incompatible chain rule, recorded in the refusal register) but whose *shape* was worth keeping. Refusing a dependency and salvaging its design is the same lawful move as everything else here: nothing silently dropped.

## VI. The Replay

Last, the question the old cities never thought to ask until it was too late: *did what ran match what was admitted to run?*

The replay verifier held the lifecycle as a token game — judge, admit, receipt, three nodes in sequence, tokens with nowhere unlawful to go. It walked the receipted history frame by frame, consuming and producing tokens, and at the end it spoke a number in fixed-point: **fitness 1.0**. The history was the plan. Nothing grew that nobody admitted.

Run the frames out of order — receipt before admit, the classic ambition — and the game refuses at the exact frame: *token not enabled, node 2.* Not a suspicion. A coordinate.

> ❯ `receipt replay` → `PowlReplayVerifier` (`bcinr-powl-receipt/src/replay.rs`), Q16.16 conformance metrics, `TokenNotEnabled { node_id }` on the out-of-order case. The token model is the same Petri-net mathematics that runs the process-mining literature; the fiction calls it a game because it is one — a game the history either wins completely or loses at a named move.

## VII. The Ladder

The pattern was real now, receipted and replayed. But *real* was not the same as *trusted*, and the system kept a ladder for that distinction — ten rungs, from Named at the bottom to Certified at the top, and a rule burned in at rung eight: nothing climbs past Replayable without a named auditor's endorsement. Not a checkbox. A name, in the record, of a person who can be asked *why*.

Patterns at the bottom of the ladder ran caged and watched. Patterns at the top had earned the thing the whole architecture existed to make earnable: the right to run unsupervised, because their entire history was a chain anyone could check.

The pattern from the quarantine vessel was at rung one. It had a long climb. That was fine. The ladder wasn't a bottleneck.

The ladder was the *point*.

> ❯ `law promote --payload '{"standing":"REPLAYABLE"}' --auditor "…"` — `BreedStanding`, ten rungs, `PartialOrd` so the gate is a comparison, auditor required at `Replayable` and above, `CERTIFIED` refuses further promotion with `already at top rung`. The frontier matrix (`dod matrix`) applies the same ladder to the system's own integrations: every capability either climbed, or sits in the register with its refusal reason — stpnt, unlicensed; mcpp, sealed by design; affidavit, incompatible chain — refused, receipted, remembered.

---

## Coda: What the Primer Is

In the old story, a girl was handed a book that watched her and grew her, and the book was magic because somewhere behind it a human was doing the judging and the story never said so out loud.

This repository says it out loud. The judging — *what is worth making* — lives in the proposer, outside the boundary, where its output is an observation with a hash and no authority. The law — *what may be made* — lives inside, deterministic, receipted, replayable, dull as a checksum and load-bearing as one.

The fiction and the codebase are converging on the same sentence, and it is short enough to be a physical law:

**A = μ(O\*).** Nothing becomes real except as the lawful projection of an admitted observation. And everything that becomes real leaves a receipt.

*— end of book one —*
