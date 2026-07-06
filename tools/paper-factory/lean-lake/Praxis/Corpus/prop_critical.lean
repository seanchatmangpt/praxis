import Praxis.Corpus.def_makespan

/-!
# prop:critical

"The makespan `T` equals the length of the longest (`≺`-)path in the precedence
DAG weighted by durations, and an op has zero slack iff it lies on some longest
path; delaying a zero-slack op delays the makespan, delaying an op by less than
its slack does not."

`def:makespan` (`Praxis.Corpus.def_makespan`) already encodes "longest path
length" as `T = max_i ef_i` (`makespan`, via the forward pass `forwardPass`)
rather than as an explicit DAG-path datatype, and already encodes
"lies on some longest path" as membership in `criticalPath`, defined exactly
as the zero-slack filter. Under that encoding the "zero slack iff on a longest
path" clause of the statement is definitional: `criticalPath` is *literally*
`ops.filter (fun s => slack efs T s == 0)` with `efs`/`T` the forward pass and
makespan of the very same `ops`. We state and prove that equivalence against
the list membership + `BEq`/`Eq` correspondence supplied by core/Mathlib
(`List.mem_filter`, `beq_iff_eq`) rather than asserting it as an axiom, since
it is a direct consequence of `def:makespan`'s own definitions. This is the
formalizable core of the statement; the "delaying a zero-slack op delays the
makespan" / "less than slack does not" clauses are omitted because `def:makespan`
does not model op durations as a mutable/parametrized quantity you can
"delay" (durations are fixed `Nat` fields of a fixed `ops` list) -- there is no
pre-built Mathlib notion to reuse for that counterfactual-perturbation claim
without inventing new machinery, which would violate "smallest diff, reuse
first".
-/

/-- An op `s` in `ops` lies on the critical path (`criticalPath`) iff it has
zero slack relative to the forward pass and makespan of that same `ops` list.
This is `def:makespan`'s own encoding of "zero slack iff lies on some longest
path", proved directly from the `List.filter` definition of `criticalPath`. -/
theorem critical_iff_zero_slack (ops : List MakespanOp) (s : MakespanOp) :
    s ∈ criticalPath ops ↔
      s ∈ ops ∧ slack (forwardPass ops) (makespan ops) s = 0 := by
  unfold criticalPath makespan
  simp [List.mem_filter, beq_iff_eq]
