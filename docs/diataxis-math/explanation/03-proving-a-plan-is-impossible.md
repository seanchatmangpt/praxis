# Proving a plan is impossible (not just failing to find one)

**Source:** `docs/thesis/03_planning_geometry.tex`. Three theorems: `thm:bounded-ground`
(line 189), `thm:farkas` (line 410), `thm:lang-correct` (line 678).

## The plain-English version

When Praxis searches for a plan — a sequence of actions that gets from a starting state to
a goal — there are two very different kinds of "no": *"I searched everywhere reasonable
and didn't find one"* (a weak, time-bounded claim), and *"I can show you a short piece of
math that proves no plan can possibly exist"* (a strong, checkable claim, independent of
how the search was run). This paper is about earning the second kind of "no" wherever
possible.

### 1. The search space is provably finite before you even start (`thm:bounded-ground`)

Before any search happens, this theorem bounds how big the space of possible actions can
get: with `N` objects in the world and actions that take at most `k` parameters each, the
number of distinct concrete actions is at most a fixed number (roughly `N` raised to the
power `k`, times the number of action templates) — a hard ceiling that doesn't depend on
how hard the goal turns out to be. That means planning is guaranteed to either find a plan
or run out of things to try within a known, fixed bound — it can never spin forever
without an answer.

### 2. When a goal is unreachable, there's a short certificate proving it — not just an exhausted search (`thm:farkas`)

This is the paper's most striking result. Reachability in this kind of system can be
phrased as: "is there a way to combine the available actions (each used a non-negative
number of times) that adds up to exactly the required change?" Farkas' lemma — a classical
result from linear algebra, roughly 100 years old — says that for exactly this kind of
question, one of two things is always true: either such a combination exists, *or* there's
a short "certificate" (a single vector of numbers) that mathematically proves it can't,
by exhibiting a direction in which every available action moves you the wrong way. When
that certificate exists, it's fast to check (you just do one multiplication) and it proves,
for *every possible order* of actions, not just the orders actually tried, that the goal is
unreachable. This turns "we couldn't find a plan" into "here is a short piece of arithmetic
you can check yourself that proves no plan exists."

### 3. Converting between two plan-representation formats never silently changes what a plan means (`thm:lang-correct`)

Praxis represents workflows in more than one internal format (a "workflow net" style and a
"POWL" style), because different parts of the system want different representations. This
theorem proves that converting between them, and back again, always preserves *exactly*
which sequences of steps are considered valid — for any prefix length you check. In other
words, translating a plan's representation is never itself a source of silent bugs.

## Verification status

- `thm:bounded-ground`: **unformalized** in the Mathlib lane (three attempts, real errors —
  missing lemma names in this Mathlib snapshot); **verified** in the bare-core lane.
- `thm:farkas`: **unformalized** in the Mathlib lane (missing the specific Mathlib lemma
  `PointedCone.FG.isClosed` in this snapshot) and **blocked** in the bare-core lane — this
  is the one headline theorem in this doc set that has **not yet been machine-verified in
  either lane**. The math is proved by hand in the `.tex` source and cites a well-known
  100-year-old classical result, but no Lean kernel has confirmed the corpus's specific
  formalization of it yet. Named here honestly rather than rounded up to "verified."
- `thm:lang-correct`: **verified** in the Mathlib lane, on the first attempt.

See [../reference/00-biggest-theorems-table.md](../reference/00-biggest-theorems-table.md)
for the exact real kernel error text for the two unformalized cases.
