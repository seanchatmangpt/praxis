# Trust at planetary scale

**Source:** `docs/thesis/04_projection_and_scale.tex`. Three theorems: `thm:kill`
(line 424), `thm:branchless` (line 547), `thm:swar-verify` (line 596).

## The plain-English version

The earlier papers establish that individual checks (admission, receipts, planning) can
be made rigorous for one request at a time. This paper asks: does any of that rigor
survive when you're running not one agent but many, at speed, and how do you know your
*checking* code itself is trustworthy rather than just quiet?

### 1. A test suite either catches an injected bug at exactly the right layer, or the bug was never real (`thm:kill`)

This is about mutation testing: deliberately injecting a small, specific bug ("mutant")
into the code and checking whether the test suite notices. The theorem proves something
sharper than "good test suites catch bugs": if every validation stage in a staged checker
is individually sound and complete, then a mutant gets caught *if and only if* it actually
violates some invariant — and when it does get caught, it's caught at *exactly* the stage
responsible for that invariant, never a different one. A mutation that violates nothing is
provably an "equivalent mutant" (semantically identical to the original despite the code
change) — not a blind spot in the tests. This turns "our test suite feels thorough" into a
mechanical guarantee about exactly which bugs it will and won't catch, and why.

### 2. Checking 64 agents' admission status takes 7 machine instructions total, not 64 (`thm:branchless`)

If each of 64 running agents has 8 separate pass/fail conditions to satisfy, you could
check them one agent at a time, one condition at a time — that's slow and has lots of
"if this fails, branch here" logic, which is exactly what makes modern CPUs slow (branches
are expensive to predict wrong). This theorem proves that if you lay the data out
cleverly (one 64-bit word per condition, one bit per agent, instead of one record per
agent), checking *all 64 agents at once* is just 7 bitwise AND operations — a fixed,
tiny, branch-free computation, regardless of which agents actually pass or fail. That's
about a ninth of one instruction per agent, not one-or-more instructions per agent times 64
agents.

### 3. That same branchless trick is itself independently checkable, not just fast (`thm:swar-verify`)

Making something fast by being clever with bit tricks is exactly the kind of code most
likely to hide a subtle bug. This result shows the fast, "SWAR" (SIMD-within-a-register)
version of the admission sweep computes the *same* answer, bit for bit, as the slow,
obviously-correct one-agent-at-a-time version — so the speed doesn't come at the cost of
correctness confidence; you can verify the fast path against the slow path.

## Verification status

`thm:kill`, `thm:branchless`, and `thm:swar-verify` are all **verified** in the Mathlib
lane — all three of this paper's headline results have real, independent Lean kernel
confirmation.

See [../reference/00-biggest-theorems-table.md](../reference/00-biggest-theorems-table.md)
for citations.
