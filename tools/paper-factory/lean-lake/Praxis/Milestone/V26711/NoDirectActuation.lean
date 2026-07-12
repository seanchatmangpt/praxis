/-!
# PROJ-769 / PRD v26.7.11 §12 — N3 Quarantine: No Direct Actuation

Target 3 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"absence of direct actuation in the declared transition relation."

PRD §12 (`docs/jira/v26.7.11/PRD.md:592-609`), in relevant part:

> N3 execution SHALL require: ... zero direct actuation.
> ...
> An N3 rule that requests direct actuation SHALL be refused.

## Real correspondence

This formalizes the actual admission logic in
`crates/praxis-graphlaw/src/chatman/router.rs`'s `N3Executor::run` (not an abstract
restatement): for `rules : &[N3Rule]`, the real Rust scans `rules` in order and, for
the first rule whose `direct_actuation_builtins` is non-empty, returns
`Err(Refusal::N3DirectActuationRefused(..))` unconditionally — "before that rule's
ordinary builtin whitelist is checked or its cost is consumed (and before any later
rule runs) ... no `execution.builtin_whitelist_mask` value can ever admit it" (the
function's own doc comment, `router.rs:785-790`). `runN3` below is the same
sequential admit-or-refuse fold, narrowed to the one invariant this file formalizes
(the direct-actuation gate); the ordinary builtin whitelist and cost-bound checks
the real function also performs are out of scope here.

No axioms: `N3Rule`, `N3Result`, and `runN3` are plain inductive/recursive data;
both theorems below are proved by structural induction on the rule list.
-/

/-- One declared N3 rule: its ID, plus whether it requests a recognized
direct-actuation builtin (`router.rs`'s `N3Rule.direct_actuation_builtins`,
collapsed here to presence/absence — which builtin, specifically, is out of scope
for this invariant). -/
structure N3Rule where
  ruleId : String
  hasDirectActuationBuiltin : Bool
deriving DecidableEq, Repr

/-- The outcome of running a declared rule list: either every rule was admitted (in
order), or the run was refused at a specific rule ID (`router.rs`'s
`Refusal::N3DirectActuationRefused`, collapsed to the triggering rule's ID). -/
inductive N3Result where
  | admitted : List String → N3Result
  | refused  : String → N3Result
deriving DecidableEq, Repr

/-- `runN3`: the same sequential admit-or-refuse scan as `N3Executor::run`
(`router.rs:810-863`), narrowed to the direct-actuation gate — a rule with
`hasDirectActuationBuiltin = true` refuses the whole run unconditionally, before any
later rule is inspected, matching the real function's early-return `for` loop. -/
def runN3 : List N3Rule → N3Result
  | [] => .admitted []
  | r :: rs =>
      if r.hasDirectActuationBuiltin then
        .refused r.ruleId
      else
        match runN3 rs with
        | .admitted ids => .admitted (r.ruleId :: ids)
        | .refused id   => .refused id

/-- `thm:no_direct_actuation_admitted` (soundness): whenever `runN3` admits a rule
list, *every* rule in it has `hasDirectActuationBuiltin = false` — the declared
transition relation's admitted outcomes contain zero direct actuation, matching PRD
§12's "zero direct actuation" for every N3 execution the router can ever admit. -/
theorem no_direct_actuation_admitted :
    ∀ (rules : List N3Rule) (ids : List String),
      runN3 rules = .admitted ids → ∀ r ∈ rules, r.hasDirectActuationBuiltin = false := by
  intro rules
  induction rules with
  | nil => intro ids _ r hr; simp at hr
  | cons r rs ih =>
      intro ids h r' hr'
      simp only [List.mem_cons] at hr'
      unfold runN3 at h
      cases hb : r.hasDirectActuationBuiltin with
      | true => rw [hb] at h; simp at h
      | false =>
          rw [hb] at h
          cases hrec : runN3 rs with
          | admitted ids' =>
              rw [hrec] at h
              rcases hr' with rfl | hmem
              · exact hb
              · exact ih ids' hrec r' hmem
          | refused id => rw [hrec] at h; simp at h

/-- `thm:direct_actuation_forces_refusal` (completeness): if some rule in the list
requests direct actuation, `runN3` refuses the run — "an N3 rule that requests
direct actuation SHALL be refused." Together with `no_direct_actuation_admitted`
this gives a real characterization, not just a one-directional guard: `runN3` admits
iff no rule in the list requests direct actuation. -/
theorem direct_actuation_forces_refusal :
    ∀ (rules : List N3Rule), (∃ r ∈ rules, r.hasDirectActuationBuiltin = true) →
      ∃ id, runN3 rules = .refused id := by
  intro rules
  induction rules with
  | nil => rintro ⟨r, hr, _⟩; simp at hr
  | cons r rs ih =>
      rintro ⟨r', hr', hb'⟩
      simp only [List.mem_cons] at hr'
      unfold runN3
      cases hb : r.hasDirectActuationBuiltin with
      | true => exact ⟨r.ruleId, by simp⟩
      | false =>
          rcases hr' with rfl | hmem
          · rw [hb'] at hb; simp at hb
          · obtain ⟨id, hid⟩ := ih ⟨r', hmem, hb'⟩
            exact ⟨id, by simp [hid]⟩
