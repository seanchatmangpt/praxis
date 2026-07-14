# Multifractal Workflow

## Recursive Manufacture, Public Semantic State, Process Geometry, and Receipted Consequence

### A dissertation reconstructed from first principles for release v26.7.13

**Primary thesis:** recursive workflow manufacture is the central breakthrough. Every other
component exists to make recursion lawful: RDF gives a child workflow admitted state; bounded
planning gives it a truthful search result; POWL gives it a typed geometric socket; permission gives
it authority; heterogeneous runtimes give it motion; receipts give its motion standing; replay makes
the standing independently reconstructible.

**Release date of this edition:** 13 July 2026  
**Research object:** Multifractal Workflow, mfact, and the Operation Dogfood manufacturing loop  
**Crown experiment:** RDF-governed Rust dry-run publication through the complete Claude Code
lifecycle  
**Current crown standing:** REFUSED for whole-workspace dry-run publication; the truthful refusal is
evidence for the lifecycle doctrine, not evidence that the target publication capability is
complete.

---

## Declaration of epistemic discipline

This dissertation uses the phrase *not debatable* in one precise and limited sense. A reader should
not need to debate what a symbol means, which assumptions a result uses, what was proved, what was
observed, or what would falsify a claim. Mathematics can make an implication compulsory once its
premises and inference rules are admitted. It cannot make an empirical premise true by typography.
Accordingly, this dissertation never uses formal notation to promote an observation into a theorem.

Every material statement belongs to exactly one of the following classes.

| Class | Meaning | Required warrant |
|---|---|---|
| DEFINITION | A term is assigned an exact meaning | The defining clause itself |
| AXIOM | A premise is assumed within a named model | Explicit scope and consistency obligations |
| THEOREM | A proposition follows from definitions, axioms, and earlier results | A proof |
| KERNEL_CHECKED | A proof term was accepted by a named proof-assistant kernel | Source identity, toolchain identity, and checker receipt |
| ALGORITHM | A finite procedure is specified | Inputs, outputs, termination argument, and complexity bound |
| IMPLEMENTED | An algorithm has a concrete implementation | Source identity and build/test evidence |
| OBSERVED | A real execution produced a result | Content-addressed observation and provenance |
| EMPIRICAL | A repeatable regularity is inferred from observations | Protocol, data, uncertainty, and falsifier |
| CONJECTURE | A proposition is proposed but not proved | Explicit counterexample search or future proof obligation |
| PLANNED | A capability is required but not yet evidenced | Promotion test |
| REFUSED | An admitted run reached a typed blocking condition | Refusal reason and residue |
| UNKNOWN | The available evidence does not decide the claim | Missing observation named explicitly |

The classifications are not decorative labels. They are a type discipline. In particular,

\[
\mathsf{Observed} \not\Rightarrow \mathsf{Theorem},
\qquad
\mathsf{Theorem} \not\Rightarrow \mathsf{Observed},
\]

and

\[
\mathsf{Planned} \not\Rightarrow \mathsf{Implemented}
\not\Rightarrow \mathsf{Alive}.
\]

A proof that a transition relation preserves an invariant does not prove that a particular external
command ran. A receipt that a command ran does not prove that the command implements an abstract
theorem. The former is a claim about all objects admitted by a model; the latter is a claim about one
historical occurrence. Multifractal Workflow requires both while permitting neither to impersonate
the other.

---

## Abstract

Most workflow systems assume that the workflow is already known. A human or application author must
translate institutional reality into a diagram, script, directed acyclic graph, prompt, or queue of
tasks before automation begins. This assumption hides the greater part of the work: determining
which observations are authoritative, reconciling contradictions, recovering constraints,
constructing a feasible process, obtaining authority to mutate the system, responding to newly
revealed failure, and proving what occurred. The automation executes the residue of design and then
receives credit for the whole consequence.

This dissertation develops **Multifractal Workflow (MFW)** as a recursive manufacturing system that
begins before a workflow exists. Its governing operation accepts an admitted state and a goal,
searches a bounded design space, manufactures a process geometry, asks for plan-bound permission,
executes through a broker, observes the consequence, and recursively grafts repair workflows into
the parent whenever execution reveals lawful residue. The same operation governs a campaign, a
release, a repair, a test, and a single actuation. The word *multifractal* therefore has two distinct
uses. The first is exact and structural: the same obligation-preserving recursive law appears at
multiple process scales. The second is empirical and conditional: measured workflow mass may obey
non-uniform scale laws describable by a multifractal spectrum. The first does not imply the second.

The authoritative instance state is an RDF dataset rather than a private in-memory ontology. Public
vocabularies represent provenance, policy, architecture, observations, data products, units, and
quality; local vocabulary is restricted to an explicit engine application binary interface. SHACL
and bounded rule closure define admission. PDDL defines finite feasibility problems and truthful
search outcomes. POWL v2 supplies hierarchical process geometry, including partial order,
concurrency, choice, loops, and typed grafting sockets. Arazzo represents manufactured inter-engine
workflows. A normalized transition core projects into Rust, Erlang/OTP, WASM, and AtomVM only where
an explicit correspondence relation has been verified. A broker separates modeled effects from real
effects. Durable pre-actuation and post-actuation receipts make unreceipted completion
unconstructible, while replay tests reconstruction from the admitted inputs and recorded exogenous
events.

The dissertation supplies the necessary mathematics from first principles: sets, relations,
functions, algebraic sums and products, partial orders, finite topologies, trace languages, fixed
points, bounded state-space search, metric spaces, measures, Hausdorff dimension, local dimension,
partition functions, generalized dimensions, Legendre transforms, branching random walks,
multifractal detrended fluctuation analysis, finite-difference calculus, gradients, constrained
optimization, and variational reasoning. Each mathematical object is connected to a runtime object,
a proof obligation, an algorithm, a measurable observable, or a clearly marked conjecture.

The crown experiment for v26.7.13 is Operation Dogfood: any Rust developer states the outcome
*dry-run publish*; MFW discovers the unfamiliar repository, represents research end to end in RDF,
manufactures a bounded plan, requests permission, invokes Claude Code when implementation is
required, executes real Cargo gates through a harness, receipts every effect, and replays the run.
The recorded whole-workspace attempt is currently refused by unversioned path dependencies, license
gaps, a missing root license, and path leakage. This is not rounded into success. The system is
successful only to the level at which it truthfully constructs and receipts the refusal. The release
claim remains below ALIVE until the same admitted object crosses every required gate.

The broader result is a precise form of Buckminster Fuller's comprehensive anticipatory design
science inside a controlled system boundary. MFW does not claim adoption or social response, which
lie outside the controlled calculus. It measures created design capacity: the growth of lawful,
reproducible consequences reachable per unit of matter, energy, time, compute, and interpretation.
Ephemeralization becomes a ratio, synergy becomes a non-additive capability surplus, a trimtab
becomes a minimal-support intervention with maximal constrained leverage, and a World Game becomes
an admitted search over whole-system alternatives whose winning strategies can become permissioned,
receipted processes.

---

## Contents

1. Recursive workflow as the primary breakthrough
2. Primitive mathematics and notation
3. Claims, standing, and truthful outcomes
4. Observation, admission, and RDF authority
5. Semantic contraction and bounded rule closure
6. Manufacture as an algebra of lawful consequence
7. Finite planning from STRIPS and PDDL
8. POWL process geometry
9. Recursive grafting, free structure, and termination
10. Search graphs, manufacturing graphs, and architecture search
11. Permission, brokered actuation, and type-state completion
12. Arazzo, AIR, Erlang/OTP, WASM, AtomVM, and correspondence
13. Receipts, canonicalization, replay, and event evidence
14. Operation Dogfood and the governed Claude Code lifecycle
15. The Rust dry-run publish crown
16. RDFTriple8 and finite semantic admission
17. Metric, topological, and measure-theoretic foundations
18. Multifractal formalism from first principles
19. Workflow measurement, MF-DFA, and branching random walks
20. Discrete calculus and process thermodynamics
21. Design for Combinatorial Maximalism
22. Formal guarantees: residue, isolation, commutation, and receipts
23. mfact, Lean, ggen, and the boundary of proof
24. Comprehensive anticipatory design science and the Fuller canon
25. Evaluation method, exact witnesses, and falsification
26. Current standing of v26.7.13
27. Vision 2030
28. Limitations and open theorems
29. Conclusion
30. Appendices: notation, algorithms, proof obligations, RDF shapes, and bibliography.

---

# Part I. The Recursive Law

# Chapter 1. Recursive Workflow as the Primary Breakthrough

## 1.1 The workflow-before-workflow problem

Let a **goal** be a proposition about a desired future state. Let a **workflow** be a structured set
of activities and relations intended to transform a present state into one satisfying that goal.
Conventional workflow engines receive both. They are asked to execute a process after another actor
has already discovered and encoded the process.

MFW addresses the prior problem:

> Given incomplete observation of a bounded system and a desired consequence, manufacture the
> lawful workflow that can attempt the consequence, and manufacture a lawful child workflow when
> the attempt reveals that the parent workflow is incomplete.

The distinction can be written as a difference in function signatures. A conventional executor is
idealized as

\[
\operatorname{execute}: W \times X \longrightarrow X,
\]

where \(W\) is an already supplied workflow and \(X\) is a state. MFW requires a manufacturing
operator

\[
\mathcal{M}_{B}: O \times G \times P \times \mathbb{N}
\longrightarrow \mathsf{Outcome}(W \times E),
\]

where:

- \(B\) is an explicit system boundary;
- \(O\) is a set of observations, not yet presumed authoritative;
- \(G\) is a goal;
- \(P\) is a permission surface;
- the natural number is a recursion or search budget;
- \(W\) is a manufactured workflow;
- \(E\) is evidence; and
- \(\mathsf{Outcome}\) is a disjoint tagged sum that preserves success, exhaustion, bounds,
  unsupported capability, inconsistency, and refusal.

The manufacturing operator does not merely generate a diagram. It returns a process whose steps
are connected to admitted facts, verifiable preconditions, permission, real actuation mechanisms,
and evidence obligations.

## 1.2 The recursive socket

Suppose a parent workflow \(W\) contains an activity \(a\). Execution reaches \(a\), but an observed
condition \(r\), called **residue**, shows that \(a\) cannot lawfully complete. A conventional system
may fail, improvise outside the model, or require a human to rewrite the workflow. MFW instead treats
the residue as a continuation goal.

Let

\[
g_r = \operatorname{continuationGoal}(r).
\]

If \(g_r\) is admitted and the remaining descent budget is positive, the manufacturing operator
constructs a child workflow

\[
W_r = \mathcal{M}_{B}(O_r^{*}, g_r, P_r, d-1),
\]

where \(O_r^{*}\) is the newly admitted state, \(P_r\) is permission no broader than the parent's
available authority, and \(d-1\) is a strictly smaller natural number. The child is inserted at a
typed socket:

\[
W' = W[a \mapsto W_r].
\]

The notation \(W[a \mapsto W_r]\) means: remove the placeholder activity \(a\), insert all nodes and
relations of \(W_r\), connect every predecessor of \(a\) to each admitted entry of \(W_r\), connect
each admitted exit of \(W_r\) to every successor of \(a\), preserve all unaffected relations, and
record the substitution as provenance. Chapter 9 defines this operation exactly and proves the
termination result produced by the decreasing budget.

## 1.3 The recursive law-state loop

At every scale the same loop appears:

\[
\boxed{
\text{Observe}
\rightarrow \text{Admit}
\rightarrow \text{Plan}
\rightarrow \text{Ask}
\rightarrow \text{Execute}
\rightarrow \text{Repair}
\rightarrow \text{Receipt}
\rightarrow \text{Replay}
\rightarrow \text{Recurse}
}
\]

This is an exact structural claim when the terms are interpreted by the state machine defined in
this dissertation. It is not yet a claim that empirical event distributions are multifractal. The
loop is called **law-state** because two kinds of objects alternate:

1. **law objects** constrain what transitions are admissible; and
2. **state objects** record which admitted conditions currently hold.

Law without state cannot determine what should happen next. State without law cannot determine what
may happen. The loop maintains their separation while allowing state observations to select lawful
continuations.

## 1.4 The core equations

The shortest statement of MFW is

\[
A = \mu(O^{*})
\]

and

\[
R = \operatorname{receipt}(A).
\]

The symbols mean the following.

- \(O\) is raw observation.
- \(O^{*}\) is observation that has crossed a declared admission boundary.
- \(\mu\) is a permissioned manufacturing process, not an arbitrary function.
- \(A\) is an artifact or state-changing consequence.
- \(R\) is an evidence object binding the consequence to its input, plan, permission, actuation, and
  result.

Expanded to expose every required phase, the equation is

\[
\begin{aligned}
O^{*} &= \operatorname{admit}_{B,L}(O),\\
\Pi &= \operatorname{plan}_{B,L,K}(O^{*},g),\\
W &= \operatorname{geometry}(\Pi),\\
p &= \operatorname{authorize}(W,m,c,t),\\
(A,E) &= \operatorname{actuate}_{\mathrm{broker}}(O^{*},W,p),\\
R &= \operatorname{seal}(O^{*},g,\Pi,W,p,A,E),\\
\rho &= \operatorname{replay}(R).
\end{aligned}
\]

Here \(L\) is admitted law, \(K\) is the collection of search bounds, \(m\) is the permitted mutation
set, \(c\) is a resource-cost bound, \(t\) is an expiry time, \(E\) is evidence, and \(\rho\) is a
replay verdict. Every displayed equality is either a definition or a target interface; none implies
that the corresponding implementation is already complete.

## 1.5 Why the breakthrough is not an agent loop

An unrestricted agent loop can repeatedly propose and perform actions. Recursion alone is trivial.
The contribution is **obligation-preserving recursion**. When a parent delegates to a child, the
following must not silently widen:

\[
\begin{aligned}
\operatorname{authority}(W_r) &\subseteq \operatorname{authority}(W),\\
\operatorname{mutation}(W_r) &\subseteq
  \operatorname{mutation}(W) \cup \operatorname{newlyApproved},\\
\operatorname{cost}(W_r) &\leq \operatorname{remainingCost}(W),\\
\operatorname{depth}(W_r) &< \operatorname{depth}(W),\\
\operatorname{evidenceObligations}(W)
  &\subseteq \operatorname{evidenceObligations}(W[a\mapsto W_r]),\\
\operatorname{receiptObligations}(W)
  &\subseteq \operatorname{receiptObligations}(W[a\mapsto W_r]).
\end{aligned}
\]

The last two inclusions say that recursion may add proof obligations but may not delete a parent's
obligations. A child that passes its own tests while erasing the parent's release gate is not a
repair; it is a change of goal.

## 1.6 Research questions

This dissertation answers or sharply bounds the following questions.

1. How can incomplete observations become authoritative planning state without silently becoming
   truth?
2. How can finite search distinguish no solution from insufficient search?
3. How can a plan become process geometry with topology-derived concurrency?
4. Under what interface conditions can a child workflow replace a parent activity without breaking
   causal order, permission, or evidence obligations?
5. How can real side effects be separated from modeled effects and still remain reconstructible?
6. What does a receipt prove, and what does it not prove?
7. What exact correspondence is required before two runtimes may be said to execute the same
   workflow?
8. When does the word *multifractal* denote a measured mathematical property rather than a recursive
   metaphor?
9. How can Fuller's design-science concepts be converted into controlled-system quantities without
   claiming social outcomes outside the system boundary?
10. What evidence is still missing before v26.7.13 may claim a real dual crown witness and a
    whole-workspace Rust dry-run publication?

## 1.7 Contributions

The principal contributions are:

1. a typed standing discipline separating theorem, observation, implementation, and target state;
2. an RDF-first admission model for complete lifecycle instance state;
3. a truthful bounded planning result algebra;
4. a formal core for POWL process geometry and typed recursive grafting;
5. a proof of termination for bounded recursive growth;
6. a two-graph distinction between planning search and real manufacturing history;
7. a broker and receipt model that makes completed-but-unreceipted state unconstructible;
8. a conditional correspondence theory for heterogeneous runtimes;
9. a first-principles multifractal measurement rail separated from generative workflow semantics;
10. a calculus of capability leverage, ephemeralization, synergy, and trimtab interventions;
11. a complete target architecture for Operation Dogfood; and
12. an honest v26.7.13 standing ledger that refuses to promote incomplete crown paths.

---
# Part II. Mathematical Language

# Chapter 2. Primitive Mathematics and Notation

This chapter prevents later symbols from functioning as appeals to authority. It constructs the
minimum mathematical language used by the workflow theory. It does not attempt to reconstruct all
of axiomatic set theory; the background assumption is ordinary classical mathematics with finite
sets, the natural numbers, and the real numbers.

## 2.1 Propositions and logical connectives

A **proposition** is an expression that has one of two truth values: true or false. If \(P\) and
\(Q\) are propositions, then:

- \(\neg P\) means “not \(P\)”;
- \(P \land Q\) means “\(P\) and \(Q\)”;
- \(P \lor Q\) means inclusive “or”;
- \(P \Rightarrow Q\) means “if \(P\), then \(Q\)”;
- \(P \Leftrightarrow Q\) means both \(P \Rightarrow Q\) and \(Q \Rightarrow P\).

The universal quantifier

\[
\forall x\in X,\ P(x)
\]

means that \(P(x)\) holds for every member \(x\) of \(X\). The existential quantifier

\[
\exists x\in X,\ P(x)
\]

means that at least one member of \(X\) satisfies \(P\). The unique-existence notation

\[
\exists!x\in X,\ P(x)
\]

means that an \(x\) exists and any two members satisfying \(P\) are equal.

## 2.2 Sets, membership, and construction

A **set** \(X\) is a collection of distinct objects. The statement \(x\in X\) says that \(x\) is a
member of \(X\); \(x\notin X\) says that it is not. The empty set, containing no members, is
\(\varnothing\).

For sets \(X\) and \(Y\):

\[
X\subseteq Y
\quad\Longleftrightarrow\quad
\forall x\,(x\in X\Rightarrow x\in Y).
\]

Equality is extensional:

\[
X=Y
\quad\Longleftrightarrow\quad
\forall x\,(x\in X\Leftrightarrow x\in Y).
\]

The standard set operations are

\[
\begin{aligned}
X\cup Y &= \{x\mid x\in X\lor x\in Y\},\\
X\cap Y &= \{x\mid x\in X\land x\in Y\},\\
X\setminus Y &= \{x\mid x\in X\land x\notin Y\}.
\end{aligned}
\]

The **power set**

\[
\mathcal{P}(X)=\{A\mid A\subseteq X\}
\]

is the set of every subset of \(X\). If \(X\) is finite and contains \(n\) members, then
\(\mathcal{P}(X)\) contains \(2^n\) members. To see why, associate each subset with an \(n\)-bit
string. Bit \(i\) is one exactly when the \(i\)-th member is present. Every bit has two choices,
independently, producing \(2\cdot2\cdots2=2^n\) strings. This fact will bound the state space of a
finite propositional planner.

An ordered pair is written \((x,y)\). The Cartesian product is

\[
X\times Y=\{(x,y)\mid x\in X\land y\in Y\}.
\]

An \(n\)-tuple \((x_1,\ldots,x_n)\) is an ordered collection of \(n\) entries. Order matters:
\((x,y)\) need not equal \((y,x)\).

The **disjoint union** or tagged sum is

\[
X\sqcup Y=(\{0\}\times X)\cup(\{1\}\times Y).
\]

The tags make members from the two sides distinguishable even when \(X\cap Y\neq\varnothing\).
Truthful planner outcomes use this construction: a bounded result cannot equal an exhausted result
because their constructor tags differ.

## 2.3 Numbers

The natural numbers are

\[
\mathbb{N}=\{0,1,2,3,\ldots\}.
\]

They support addition, multiplication, and the strict order \(<\). The essential induction principle
is:

1. prove \(P(0)\); and
2. prove \(P(n)\Rightarrow P(n+1)\) for arbitrary \(n\in\mathbb{N}\).

Then \(P(n)\) holds for every natural number. Induction proves replay determinism and recursive
termination later in the dissertation.

The integers

\[
\mathbb{Z}=\{\ldots,-2,-1,0,1,2,\ldots\}
\]

add additive inverses. The rational numbers are ratios \(a/b\), where \(a,b\in\mathbb{Z}\) and
\(b\neq0\). The real numbers \(\mathbb{R}\) complete the rationals so that every Cauchy sequence has a
real limit. The nonnegative reals are

\[
\mathbb{R}_{\ge0}=\{x\in\mathbb{R}\mid x\ge0\}.
\]

For \(x>0\), \(\log x\) denotes the natural logarithm, the inverse of the exponential function
\(e^x\). A different logarithm base changes numerator and denominator by the same constant in the
dimension ratios used later, so the limiting dimension is base-independent.

## 2.4 Functions and partial functions

A function

\[
f:X\to Y
\]

assigns exactly one output \(f(x)\in Y\) to each input \(x\in X\). The set \(X\) is the domain and
\(Y\) the codomain. The image of a subset \(A\subseteq X\) is

\[
f[A]=\{f(a)\mid a\in A\}.
\]

The inverse image of \(B\subseteq Y\) is

\[
f^{-1}[B]=\{x\in X\mid f(x)\in B\}.
\]

Composition is

\[
(g\circ f)(x)=g(f(x))
\]

whenever \(f:X\to Y\) and \(g:Y\to Z\). The identity function satisfies
\(\operatorname{id}_X(x)=x\).

A **partial function** \(f:X\rightharpoonup Y\) may be undefined for some members of \(X\). It can be
made total by returning a tagged result:

\[
\widehat f:X\to Y\sqcup E,
\]

where \(E\) contains explicit error or refusal values. MFW prefers this total form because an absent
result must not be confused with success.

A function is **injective** if

\[
f(x_1)=f(x_2)\Rightarrow x_1=x_2.
\]

It is **surjective** if every \(y\in Y\) equals \(f(x)\) for at least one \(x\in X\). It is
**bijective** if both properties hold. A bijection establishes exact structural correspondence
between sets; a hash function is not treated as a mathematical bijection, only as a practical
collision-resistant identifier under a stated cryptographic assumption.

## 2.5 Relations, equivalence, and quotient sets

A binary relation on \(X\) is a subset \(R\subseteq X\times X\). We write \(xRy\) when
\((x,y)\in R\).

An equivalence relation \(\sim\) is:

1. reflexive: \(x\sim x\);
2. symmetric: \(x\sim y\Rightarrow y\sim x\);
3. transitive: \(x\sim y\land y\sim z\Rightarrow x\sim z\).

The equivalence class of \(x\) is

\[
[x]_{\sim}=\{y\in X\mid y\sim x\}.
\]

The quotient set

\[
X/{\sim}=\{[x]_{\sim}\mid x\in X\}
\]

contains one mathematical object for each class of equivalent representations. RDF graph
canonicalization attempts to choose a stable representative for a graph-isomorphism class; semantic
replay may compare quotient classes even when byte serialization differs.

## 2.6 Orders and lattices

A **partial order** \(\leq\) on \(X\) is reflexive, antisymmetric, and transitive:

\[
\begin{aligned}
x&\leq x,\\
(x\leq y\land y\leq x)&\Rightarrow x=y,\\
(x\leq y\land y\leq z)&\Rightarrow x\leq z.
\end{aligned}
\]

The pair \((X,\leq)\) is a partially ordered set, or **poset**. Two members are incomparable when

\[
x\parallel y
\quad\Longleftrightarrow\quad
\neg(x\leq y)\land\neg(y\leq x).
\]

A chain is a subset whose members are pairwise comparable. An antichain is a subset whose distinct
members are pairwise incomparable. POWL uses partial order to represent causal precedence;
compatible antichains expose potential concurrency.

A lattice is a poset in which every pair \(x,y\) has a greatest lower bound \(x\wedge y\) and least
upper bound \(x\vee y\). A complete lattice has these bounds for every subset. Fixed-point semantics
for monotone rule systems use complete lattices of graph facts ordered by inclusion.

## 2.7 Algebraic structures

A **monoid** \((M,\otimes,e)\) consists of a set \(M\), an associative operation, and an identity:

\[
(a\otimes b)\otimes c=a\otimes(b\otimes c),
\qquad
e\otimes a=a=a\otimes e.
\]

Sequences under concatenation form a monoid. Receipt fragments under order-preserving append form a
monoid when their boundary digests agree.

A commutative monoid additionally satisfies \(a\otimes b=b\otimes a\). Independent operations may
be normalized using a commutative operation only after a commutation theorem establishes that their
order does not change the admitted result.

A **semiring** has addition and multiplication-like operations satisfying familiar associativity
and distributivity laws. The Boolean semiring

\[
(\{0,1\},\lor,\land,0,1)
\]

encodes reachability: matrix multiplication over this semiring says whether at least one path
exists, rather than counting paths.

## 2.8 Finite graphs and paths

A directed graph is a pair

\[
G=(V,E),
\qquad E\subseteq V\times V,
\]

where \(V\) is a vertex set and \(E\) is an edge relation. A directed path from \(v_0\) to \(v_n\) is
a sequence

\[
(v_0,v_1,\ldots,v_n)
\]

such that \((v_{i-1},v_i)\in E\) for every \(1\le i\le n\). A cycle is a nonempty path with
\(v_0=v_n\). A directed acyclic graph contains no cycle. Its transitive closure \(E^{+}\) contains
\((x,y)\) whenever a nonempty path runs from \(x\) to \(y\).

A topological ordering of a finite acyclic graph is a sequence of all vertices in which each edge
points from an earlier vertex to a later vertex. Every finite directed acyclic graph has at least one
topological ordering. The different topological orderings are the linear extensions of the induced
partial order and expose executions that differ only by the order of incomparable steps.

## 2.9 Complexity notation

For nonnegative functions \(f,g:\mathbb{N}\to\mathbb{R}_{\ge0}\),

\[
f(n)=O(g(n))
\]

means that constants \(c>0\) and \(n_0\) exist such that \(f(n)\le c g(n)\) for every
\(n\ge n_0\). This is an upper asymptotic bound, not an observed runtime. The notation
\(\Theta(g(n))\) means both an upper and a lower asymptotic bound up to positive constants.

When this dissertation says fixed-table dispatch is \(O(1)\), it means the number of table accesses
does not grow with the number of admitted triples inside the fixed 8-bit profile. It does not mean a
physical processor takes zero time or that cache behavior is constant.

## 2.10 Proof styles

Four proof forms recur.

1. **Direct proof:** assume the premises and derive the conclusion.
2. **Contradiction:** assume the negation of the conclusion and derive both \(P\) and \(\neg P\).
3. **Induction:** establish a base case and a successor step.
4. **Structural induction:** prove a property for each constructor of a recursively defined object,
   assuming it for constructor children.

Machine-checked proofs add a proof term whose type is the proposition. Kernel acceptance establishes
that the term follows the kernel's rules and imported axioms. It does not establish the truth of an
informal English gloss unless the formalization-gloss correspondence has also been reviewed.

## 2.11 Vectors, dot products, and norms

A real \(n\)-vector is an ordered tuple

\[
x=(x_1,\ldots,x_n)\in\mathbb{R}^{n}.
\]

Vector addition and scalar multiplication are coordinatewise:

\[
x+y=(x_1+y_1,\ldots,x_n+y_n),
\]

\[
\lambda x=(\lambda x_1,\ldots,\lambda x_n).
\]

The Euclidean dot product is

\[
x^{\top}y
=
\sum_{i=1}^{n}x_iy_i.
\]

The superscript \(\top\) means transpose: a column vector becomes a row vector so matrix
multiplication produces a scalar. The Euclidean norm is

\[
\|x\|_2
=
\sqrt{x^{\top}x}
=
\sqrt{\sum_{i=1}^{n}x_i^2}.
\]

The \(1\)-norm and maximum norm are

\[
\|x\|_1=\sum_i|x_i|,
\qquad
\|x\|_{\infty}=\max_i|x_i|.
\]

The Cauchy-Schwarz inequality states

\[
|x^{\top}y|
\le
\|x\|_2\|y\|_2.
\]

It is used in Chapter 20 to show the gradient gives the greatest first-order increase among unit
directions.

## 2.12 Matrices

An \(m\times n\) real matrix is an array

\[
A=[a_{ij}],
\qquad
1\le i\le m,\quad1\le j\le n.
\]

For \(x\in\mathbb{R}^{n}\), matrix-vector multiplication is

\[
(Ax)_i
=
\sum_{j=1}^{n}a_{ij}x_j.
\]

For compatible matrices \(A\) and \(B\),

\[
(AB)_{ij}
=
\sum_k a_{ik}b_{kj}.
\]

Matrix multiplication is associative but generally not commutative:

\[
(AB)C=A(BC),
\qquad
AB\neq BA\text{ in general}.
\]

A square matrix \(A\) has eigenvector \(v\neq0\) with eigenvalue \(\lambda\) when

\[
Av=\lambda v.
\]

MFW does not rely on an unspecified spectral argument; any later use of eigenvalues must name the
matrix and property.

## 2.13 Probability and expectation

A probability space is

\[
(\Omega,\mathcal{F},\mathbb{P}),
\]

where \(\Omega\) is the outcome set, \(\mathcal{F}\) a sigma algebra, and
\(\mathbb{P}:\mathcal{F}\to[0,1]\) a measure with \(\mathbb{P}(\Omega)=1\).

A random variable is a measurable function

\[
X:\Omega\to\mathbb{R}.
\]

For a finite discrete random variable taking values \(x_i\) with probabilities \(p_i\),

\[
\mathbb{E}[X]
=
\sum_i x_ip_i.
\]

Its variance is

\[
\operatorname{Var}(X)
=
\mathbb{E}\left[(X-\mathbb{E}[X])^2\right]
=
\mathbb{E}[X^2]-\mathbb{E}[X]^2.
\]

Independence of \(X\) and \(Y\) means joint event probabilities factor for their measurable events.
Branching-process results require explicit independence assumptions; repeated workflow children are
not presumed independent merely because they have separate identities.

## 2.14 Entropy

For finite probability vector \(p=(p_1,\ldots,p_n)\), Shannon entropy is

\[
H(p)
=
-\sum_{i:p_i>0}p_i\log p_i.
\]

The convention \(0\log0=0\) is justified by the limit

\[
\lim_{x\downarrow0}x\log x=0.
\]

Rényi entropy of order \(q\neq1\) is

\[
H_q(p)
=
\frac{1}{1-q}
\log\left(\sum_i p_i^q\right).
\]

The generalized dimensions of Chapter 18 are scale-normalized limits of Rényi entropy.

## 2.15 Frequently used operators

The floor

\[
\lfloor x\rfloor
\]

is the greatest integer not exceeding \(x\). The ceiling \(\lceil x\rceil\) is the least integer not
less than \(x\).

For sets \(A,B\), symmetric difference is

\[
A\triangle B=(A\setminus B)\cup(B\setminus A).
\]

For real function \(f\) over set \(X\),

\[
\operatorname*{arg\,max}_{x\in X}f(x)
\]

is the set of points attaining the maximum:

\[
\{x\in X\mid
\forall y\in X,\ f(x)\ge f(y)\}.
\]

It can contain more than one point. Likewise, \(\inf X\) is the greatest lower bound and
\(\sup X\) the least upper bound when they exist.

The indicator of proposition \(P\) is

\[
\mathbf{1}_{P}
=
\begin{cases}
1,&P\text{ is true},\\
0,&P\text{ is false}.
\end{cases}
\]

---

# Chapter 3. Claims, Standing, and Truthful Outcomes

## 3.1 Why standing is not a Boolean

The ordinary pair true/false is too small for engineering knowledge. Consider four statements:

1. a capability has not been investigated;
2. a capability is intended but not implemented;
3. a genuine slice works but the stated scope is incomplete;
4. an exact capability passed its declared verification ladder.

Collapsing all four into false loses progress; collapsing the latter three into true manufactures
false confidence. Define the claim-standing set

\[
\Sigma=\{
\mathsf{Unknown},
\mathsf{Planned},
\mathsf{PartialAlive},
\mathsf{Alive},
\mathsf{Refused},
\mathsf{Inconsistent}
\}.
\]

This is not a total order. Refused is not “less true” than Alive; it is an evidence-bearing result
that the exact admitted request did not cross a gate. Inconsistent says that authoritative evidence
disagrees and therefore forbids promotion. The only default promotion chain is

\[
\mathsf{Unknown}
\prec
\mathsf{Planned}
\prec
\mathsf{PartialAlive}
\prec
\mathsf{Alive},
\]

and each arrow requires new evidence for the same scoped object.

## 3.2 Claims as typed records

A material claim is represented as

\[
C=(u,p,s,B,E,F,t),
\]

where:

- \(u\) is the subject identity;
- \(p\) is the exact proposition;
- \(s\in\Sigma\) is standing;
- \(B\) is scope and boundary;
- \(E\) is the evidence set;
- \(F\) is a same-object falsifier; and
- \(t\) is the observation time or version.

The phrase **same-object falsifier** matters. A test of a toy planner does not promote a production
planner. A proof about an abstract transition function does not promote a binary unless the binary
is connected to that function by an admitted correspondence. Evidence is monotone only while
subject identity, proposition, boundary, and version remain fixed.

## 3.3 Truthful planner outcomes

For a witness type \(W\), exhaustion certificate type \(X\), frontier type \(F\), unsupported reason
type \(U\), and inconsistency type \(K\), define

\[
\begin{aligned}
\mathsf{Outcome}(W,X,F,U,K)={}&
\mathsf{Found}(W)\\
&\sqcup\mathsf{Exhausted}(X)\\
&\sqcup\mathsf{Bounded}(F)\\
&\sqcup\mathsf{Unsupported}(U)\\
&\sqcup\mathsf{Inconsistent}(K).
\end{aligned}
\]

Each line is a distinct constructor. An admission or permission boundary may additionally return
\(\mathsf{Refused}(r)\), where \(r\) is a structured refusal.

### Theorem 3.1 — Constructor non-collapse

No value built with Bounded equals a value built with Exhausted in the tagged-sum outcome type.

**Proof.** By construction, the disjoint union prefixes members of each variant with a distinct tag.
If \(\mathsf{Bounded}(f)\) equaled \(\mathsf{Exhausted}(x)\), their outer tags would be equal. The
tags are distinct by definition, which is a contradiction. Therefore the values are unequal.
\(\square\)

This elementary theorem is operationally important. A serializer, command-line interface, or
adapter that maps both constructors to the same exit code has implemented a non-injective projection
and destroyed information. The type theorem remains true; the implementation is wrong.

## 3.4 Exhaustion versus a bound

Let \(S\) be the exact finite search space and \(V\subseteq S\) the states visited by an algorithm.
The result Exhausted is lawful only if

\[
V=S
\quad\land\quad
\forall s\in S,\ \neg\operatorname{Goal}(s).
\]

A bound result occurs when a declared resource has been consumed while an unvisited frontier
remains:

\[
\operatorname{budgetUsed}=\operatorname{budgetLimit}
\quad\land\quad
S\setminus V\neq\varnothing.
\]

Thus

\[
\mathsf{Bounded}\not\Rightarrow\mathsf{Exhausted}.
\]

This is not a philosophical preference. The premises of exhaustion are absent when a frontier
remains.

## 3.5 Claim ceilings

Let verification rungs be ordered as

\[
\mathsf{Unit}
<\mathsf{Integration}
<\mathsf{EndToEnd}
<\mathsf{Chaos}
<\mathsf{Stress}
<\mathsf{Benchmark}
<\mathsf{IndependentReplay}.
\]

This order means “requires at least the preceding scope,” not that a benchmark is logically stronger
than a theorem. Formal proof lives on a separate axis. Represent evidence standing as a product:

\[
\mathcal{E}
=
\mathcal{E}_{\mathrm{formal}}
\times
\mathcal{E}_{\mathrm{execution}}
\times
\mathcal{E}_{\mathrm{replay}}.
\]

A claim ceiling is the componentwise maximum actually supported. No narrative may replace a missing
coordinate.

## 3.6 A non-promotion theorem

### Theorem 3.2 — Evidence cannot promote a different subject by identity alone

Let \(C_1=(u_1,p,B,E_1)\) and \(C_2=(u_2,p,B,E_2)\) be claims with the same proposition and boundary.
If \(u_1\neq u_2\), then evidence for \(C_1\) is not, without an admitted correspondence relation,
evidence for \(C_2\).

**Proof.** Evidence is a relation \(\operatorname{supports}(e,u,p,B)\). From
\(\operatorname{supports}(e,u_1,p,B)\) and \(u_1\neq u_2\), no rule of first-order logic derives
\(\operatorname{supports}(e,u_2,p,B)\). Such a derivation requires an additional premise relating
\(u_1\) and \(u_2\), for example verified equivalence or refinement. In its absence the promotion is
invalid. \(\square\)

This theorem is intentionally simple. Many release errors are violations of it: a fixture stands in
for a real repository, a generated model stands in for a runtime, or a local path stands in for an
external path.

## 3.7 Refusal as information

A refusal is a tuple

\[
r=(q,B,\gamma,\Delta,\pi),
\]

where \(q\) is the refused request, \(B\) is its boundary, \(\gamma\) is the failed gate,
\(\Delta\) is the smallest known residue sufficient to explain failure, and \(\pi\) is provenance.
A refusal is *truthful* when the gate was evaluated against the admitted object and the residue is
preserved. It is *actionable* when a continuation-goal function can map the residue into a bounded
repair request.

A refused dry run can therefore be more informative than an unreceipted green command. The former
narrows the design space and carries evidence. The latter has no lawful standing.

## 3.8 Standing-preserving projection

Human-readable documentation is a projection

\[
\operatorname{render}:\mathcal{G}_{\mathrm{claims}}\to\mathcal{D},
\]

from an authoritative claim graph to a document. If a person edits \(\mathcal{D}\) without updating
the graph, the document is no longer an authoritative source. Generated status tables should carry
claim identifiers and graph digests. This establishes one-way authority:

\[
\text{RDF claim graph}
\longrightarrow
\text{release document},
\]

not two competing sources that must be reconciled by memory.

---


# Part III. Semantic State and Lawful Manufacture

# Chapter 4. Observation, Admission, and RDF Authority

## 4.1 Observation is not truth

Let \(\Omega_B\) be the universe of observations that can be made inside boundary \(B\). An
observation is represented as

\[
o=(\operatorname{payload},\operatorname{source},\operatorname{time},
\operatorname{method},\operatorname{digest},\operatorname{confidence}).
\]

The tuple says what bytes or structured data were seen, where they came from, when and how they were
obtained, how they are content-addressed, and what uncertainty the observer reports. It does **not**
say that the payload is true. A README, a test result, a source file, a command exit code, and a
human statement are all observations. They differ in authority only after an admission policy says
how they bear on a particular proposition.

Let

\[
O\subseteq\Omega_B
\]

be the finite collection available to one run. Admission is a total decision function

\[
\operatorname{Admit}_{B,L,S}:
\mathcal{P}_{\mathrm{fin}}(\Omega_B)
\longrightarrow
\mathsf{Accepted}(\mathcal{D})
\sqcup
\mathsf{Refused}(\mathcal{R})
\sqcup
\mathsf{Inconsistent}(\mathcal{K}),
\]

where \(L\) is the selected law and entailment profile, \(S\) is the selected constraint-shape set,
\(\mathcal{D}\) is the type of authoritative RDF datasets, \(\mathcal{R}\) is a refusal report, and
\(\mathcal{K}\) is an inconsistency report. When acceptance occurs, write the admitted dataset as
\(O^{*}\).

The star is not mystical. It records that a transition occurred:

\[
O\neq O^{*}.
\]

Raw bytes remain available as provenance, but only \(O^{*}\) may become planner state.

## 4.2 RDF terms from first principles

Let:

- \(I\) be a set of Internationalized Resource Identifiers;
- \(B_n\) be a set of blank-node identifiers scoped to a graph or dataset;
- \(L_t\) be a set of literals with lexical form, datatype, and optional language metadata; and
- \(T_t\) be the recursively permitted set of RDF 1.2 triple terms.

The set of RDF terms is

\[
\mathcal{T}=I\sqcup B_n\sqcup L_t\sqcup T_t.
\]

An RDF triple is an ordered triple

\[
(s,p,o)\in (I\cup B_n)\times I\times\mathcal{T}.
\]

The first coordinate is the subject, the second the predicate, and the third the object. A graph is
a **set** of triples:

\[
G\subseteq (I\cup B_n)\times I\times\mathcal{T}.
\]

Because \(G\) is a set, duplicate triples do not create a different abstract graph. A serialization
may repeat a statement, but the mathematical graph contains one member.

An RDF dataset is

\[
\mathcal{D}=(G_0,\{(n_i,G_i)\}_{i\in J}),
\]

where \(G_0\) is the default graph and each \(n_i\) names a graph \(G_i\). The index set \(J\) is
finite for a bounded MFW run. Named graphs preserve context that would be lost by unioning all facts
into one set.

For Operation Dogfood, the dataset is partitioned at least into:

\[
\begin{aligned}
G_{\mathrm{intent}} &:& \text{requested outcome and scope},\\
G_{\mathrm{observation}} &:& \text{raw observations and payload references},\\
G_{\mathrm{admission}} &:& \text{accepted facts and conflict sets},\\
G_{\mathrm{plan}} &:& \text{PDDL witness and POWL geometry},\\
G_{\mathrm{permission}} &:& \text{authorized plan digest and mutation surface},\\
G_{\mathrm{execution}} &:& \text{tool and runtime events},\\
G_{\mathrm{evidence}} &:& \text{test, build, proof, and gate outcomes},\\
G_{\mathrm{receipt}} &:& \text{sealed provenance and digests},\\
G_{\mathrm{replay}} &:& \text{reconstruction observations}.
\end{aligned}
\]

Keeping these graphs separate prevents an observation from becoming admitted merely because both
are written in Turtle.

## 4.3 Graphs express claims, not omniscience

An asserted RDF triple denotes a proposition under an interpretation. A graph is the conjunction of
its asserted triples under the chosen entailment regime. Therefore:

\[
G\models t
\]

means that every interpretation satisfying graph \(G\) also satisfies triple or formula \(t\). It
does not mean the physical world agrees with \(G\). That requires an observation-and-admission
argument outside pure model theory.

MFW explicitly records this distinction with provenance:

\[
\operatorname{assertedBy}(t,a),
\qquad
\operatorname{derivedBy}(t,r),
\qquad
\operatorname{observedIn}(t,e).
\]

PROV-O supplies public terms for entities, activities, agents, generation, use, derivation, and
association. RDF gives the graph form; PROV-O gives a shared provenance vocabulary; neither
automatically validates the underlying event.

## 4.4 Public vocabulary before private vocabulary

Let \(V_{\mathrm{pub}}\) be the union of selected public vocabulary terms and
\(V_{\mathrm{abi}}\) the MFW-specific engine interface. The governing constraint is

\[
V_{\mathrm{instance}}\subseteq V_{\mathrm{pub}}\cup V_{\mathrm{abi}},
\]

with every member of \(V_{\mathrm{abi}}\) justified by a namespace ledger. A private term is lawful
only when no selected public term expresses the required runtime distinction without semantic loss.

The intended public surface includes, as applicable:

- TOGAF terms for architecture structure;
- PROV-O for provenance;
- DCAT and Dublin Core terms for data products and metadata;
- ODRL for permission and prohibition;
- SHACL for graph constraints;
- SOSA/SSN for observations and sensors;
- QUDT for quantities and units;
- DQV for data quality;
- SKOS for controlled status and outcome concepts;
- SPDX and DOAP for software-package and project descriptions; and
- OCEL-compatible event and object concepts for process evidence.

This is not a claim that one public ontology already contains every MFW runtime constructor. It is a
least-private-vocabulary rule. Engine ABI terms must remain bounded, versioned, and mapped.

## 4.5 Instance identity and the blank-node rule

Blank nodes express existentially identified resources. They are useful when identity genuinely does
not matter beyond a graph scope. They are hazardous when an institution owns an instance whose
identity must survive receipts, replay, reconciliation, or distributed exchange.

Define an **owned instance** as any entity \(x\) for which one of the following holds:

1. a future event must refer to \(x\);
2. a receipt must bind \(x\);
3. \(x\) crosses a graph, process, runtime, or organizational boundary;
4. \(x\) can be independently updated; or
5. two observations about \(x\) must be reconciled.

The admission rule is

\[
\operatorname{Owned}(x)\Rightarrow \operatorname{Identifier}(x)\in I.
\]

An owned instance therefore receives a stable IRI. This does not prohibit all blank nodes; it
prohibits surrendering durable instance identity to an existential placeholder.

## 4.6 Native payloads remain native

RDF is authoritative lifecycle state, not a demand to encode every byte as triples. Let
\(\mathcal{B}=\{0,1\}^{*}\) be the set of finite byte strings. A payload store is a partial map

\[
\operatorname{store}:H\rightharpoonup\mathcal{B},
\]

where \(H\) is a set of digest identifiers. For payload \(b\), define

\[
h=\operatorname{Hash}(b).
\]

The RDF graph contains an entity \(e_h\) with digest \(h\), media type, byte length, provenance,
classification, and storage locator. Code remains code, archives remain archives, and command output
remains bytes. RDF governs their identity and relationships.

The cryptographic assumption is **collision resistance**, not mathematical injectivity:

\[
\Pr[\exists b_1\neq b_2:
\operatorname{Hash}(b_1)=\operatorname{Hash}(b_2)]
\text{ is computationally negligible for the threat model.}
\]

A security argument must name the hash algorithm and version in the receipt.

## 4.7 Source precedence and conflict sets

Suppose two observations support incompatible values:

\[
o_1\models p,
\qquad
o_2\models \neg p.
\]

MFW does not silently choose the later value. Admission consults a scoped precedence relation
\(\preceq_B\) over source classes. For example, observed runtime output may outrank a stale narrative
claim for an execution proposition, while a signed policy may outrank source comments for an
authorization proposition. Precedence is predicate-specific; no source is universally sovereign.

If the policy cannot lawfully select one observation, admission creates a conflict set

\[
K_p=\{o\in O\mid o\text{ bears materially on }p\}
\]

and returns \(\mathsf{Inconsistent}(K_p)\). Planning from a contradictory authoritative state would
allow explosion in classical logic or arbitrary last-write behavior in an implementation. Typed
inconsistency refuses both.

## 4.8 SHACL as an admission boundary

A SHACL shape is treated here as a constraint predicate

\[
\sigma:G\times x\to\{\mathsf{Conforms},\mathsf{Violates}\}.
\]

For a finite shape set \(S=\{\sigma_1,\ldots,\sigma_m\}\), graph conformance is

\[
\operatorname{Conforms}(G,S)
\Longleftrightarrow
\forall i\in\{1,\ldots,m\},
\forall x\in\operatorname{targets}(\sigma_i,G),
\sigma_i(G,x)=\mathsf{Conforms}.
\]

SHACL checks structural and value constraints. Conformance does not prove physical truth. A perfectly
shaped false observation remains false. SHACL's role is to refuse impossible or incomplete planning
states before they enter the planner.

Example: a permission artifact may require exactly one plan digest, at least one allowed action, a
nonexpired timestamp, a granting agent, and a maximum mutation set. If any required field is absent,
the graph is not admitted as permission.

## 4.9 Admission algorithm

**Algorithm 4.1 — Bounded graph admission**

Input:

- finite observation set \(O\);
- source policy \(P_s\);
- entailment profile \(L\);
- finite rule set \(R\);
- finite SHACL shape set \(S\);
- canonicalization function \(\kappa\).

Output: Accepted, Refused, or Inconsistent.

Procedure:

1. Verify every referenced payload digest. Refuse on missing or mismatched payload.
2. Convert each observation into a provenance-bearing candidate graph.
3. Group materially contradictory candidates into conflict sets.
4. Apply predicate-scoped source policy. Return Inconsistent for every unresolved conflict.
5. Compute the bounded rule closure described in Chapter 5.
6. Evaluate every selected shape over the closed candidate graph.
7. If any mandatory shape fails, return Refused with a complete validation report.
8. Canonicalize the admitted dataset, compute its digest, and emit an admission receipt.
9. Return Accepted with the immutable admitted dataset identity.

The algorithm terminates when the observation set, rule universe, and shape targets are finite and
the rule system satisfies the finite closure conditions proved in Chapter 5.

## 4.10 Admission soundness is relative

### Theorem 4.1 — Structural admission soundness

If Algorithm 4.1 returns \(\mathsf{Accepted}(G)\), then every mandatory selected SHACL shape conforms
to \(G\), every referenced payload passed the selected digest check, and no conflict unresolved by
the source policy remains in the selected graph.

**Proof.** Acceptance occurs only after Steps 1, 4, and 7 complete without returning an alternative
constructor. Step 1 checks payloads; Step 4 returns on unresolved conflict; Step 7 returns on shape
failure. Therefore acceptance implies all three conditions. \(\square\)

The theorem is intentionally relative to the selected checks. It does not prove that the source
policy is morally correct, that the hash cannot collide, that observations are physically true, or
that omitted shapes would have passed.

---

# Chapter 5. Semantic Contraction and Bounded Rule Closure

## 5.1 Why the planner must not see everything

An enterprise graph may contain tens of thousands or millions of triples. A goal usually depends on
a much smaller set. Feeding the entire graph to a planner increases grounding cost, creates
irrelevant symmetries, and makes explanations unreadable.

Let \(G\) be the admitted graph, \(g\) a goal, and \(L\) admitted law. A semantic contraction
operator is

\[
\chi_L(G,g)=G_g,
\qquad G_g\subseteq\operatorname{closure}_L(G),
\]

where \(G_g\) contains the facts and rules judged load-bearing for planning \(g\). The operator must
not remove a fact required by every valid proof or plan. A practical contraction may conservatively
retain extra facts; exact minimality requires a stronger dependency proof.

## 5.2 Positive Datalog from first principles

Let \(\mathcal{A}\) be a finite universe of ground atoms. A positive rule has the form

\[
a_1\land a_2\land\cdots\land a_k\Rightarrow b,
\]

where each \(a_i,b\in\mathcal{A}\). For a fact set \(X\subseteq\mathcal{A}\), define the immediate
consequence operator

\[
T_R(X)
=
X\cup
\left\{
b\in\mathcal{A}
\;\middle|\;
\exists(a_1\land\cdots\land a_k\Rightarrow b)\in R,
\{a_1,\ldots,a_k\}\subseteq X
\right\}.
\]

The sequence

\[
X_0=G,
\qquad
X_{n+1}=T_R(X_n)
\]

adds every consequence whose premises are already present.

### Lemma 5.1 — Monotonicity

If \(X\subseteq Y\), then \(T_R(X)\subseteq T_R(Y)\).

**Proof.** Every member already in \(X\) is in \(Y\). If \(b\) is added to \(T_R(X)\), a rule exists
whose complete premise set is contained in \(X\). Since \(X\subseteq Y\), the same premise set is
contained in \(Y\), so \(b\in T_R(Y)\). \(\square\)

### Lemma 5.2 — Inflation

\[
X\subseteq T_R(X).
\]

**Proof.** The definition forms a union whose first operand is \(X\). \(\square\)

### Theorem 5.3 — Finite closure termination

If \(\mathcal{A}\) is finite, the sequence \(X_0,X_1,\ldots\) reaches a fixed point after at most
\(|\mathcal{A}\setminus X_0|\) strict-growth iterations.

**Proof.** By Lemma 5.2, the sequence is increasing:
\(X_0\subseteq X_1\subseteq\cdots\). Every strict increase adds at least one atom not previously
present. At most \(|\mathcal{A}\setminus X_0|\) such atoms exist. Therefore strict growth cannot
continue longer. At the first non-growth step, \(X_{n+1}=X_n\), which is a fixed point.
\(\square\)

Write this least fixed point as

\[
\operatorname{lfp}(T_R)=\bigcup_{n\ge0}X_n.
\]

Because the sequence stabilizes finitely, the infinite-union notation is a compact description, not
an unbounded runtime requirement.

## 5.3 Stratified negation

Negation requires care. A rule such as “infer \(p\) when \(q\) cannot be proved” is nonmonotone:
adding \(q\) can withdraw \(p\). MFW admits such rules only under an explicit stratification or other
bounded semantics.

Assign every predicate a natural-number stratum. A rule may depend positively on predicates in the
same or lower stratum but may depend negatively only on a strictly lower stratum. Evaluate strata in
increasing order. Because negative dependencies point downward, no predicate depends negatively on
itself through a cycle.

This provides a deterministic finite evaluation for a finite grounded universe. An unstratified
negative cycle is returned as Unsupported or Inconsistent according to the declared rule profile; it
is not resolved by engine-specific accident.

## 5.4 SPARQL CONSTRUCT as semantic capitalization

A deterministic SPARQL CONSTRUCT query maps one graph to another:

\[
Q:G\mapsto G_Q.
\]

When a recurrent derivation is stable, reviewed, and bounded, MFW materializes its result as an
explicit provenance-bearing graph. This is called **semantic capitalization** because future
planning can consume the derived fact rather than repay the reasoning cost.

Capitalization is lawful only if:

1. query identity and version are recorded;
2. input graph digest is recorded;
3. output graph is deterministically canonicalized;
4. provenance connects output triples to query and inputs;
5. invalidation dependencies are recorded; and
6. the result does not silently outrank its premises.

If a source premise changes, every dependent capitalized consequence enters the regeneration
residue.

## 5.5 N3 quarantine doctrine

Notation3 can express contextual implication and built-ins that exceed simple finite rule closure.
That power is useful but expands the semantic and security surface. MFW therefore applies the rule:

> Use the least expressive mechanism that can state the required law. Admit N3 only as a
> permissioned, bounded, receipted last resort.

An admitted N3 execution must declare:

- the exact rule set and graph scope;
- the allowed built-ins;
- time, step, and memory bounds;
- network and filesystem prohibitions or permissions;
- deterministic treatment of external values;
- provenance for every consequence; and
- a refusal outcome for unsupported or exceeded behavior.

N3 never actuates directly. Its output returns as candidate RDF and must cross admission before
planning or execution.

## 5.6 Dependency support

Let \(Y\) be a manufactured artifact and \(X=\{x_1,\ldots,x_n\}\) its admitted inputs. A subset
\(S\subseteq X\) is a **support** of \(Y\) under deterministic manufacturer \(f\) if any two admitted
input assignments agreeing on \(S\) produce the same \(Y\):

\[
\forall u,v\in\operatorname{Dom}(f),
\left(
u|_S=v|_S
\Rightarrow
f(u)=f(v)
\right).
\]

A support \(S\) is inclusion-minimal when no proper subset is also a support:

\[
\operatorname{MinimalSupport}(S,Y)
\Longleftrightarrow
\operatorname{Support}(S,Y)
\land
\forall S'\subsetneq S,\ \neg\operatorname{Support}(S',Y).
\]

Minimal support need not be unique. For example, a redundant Boolean expression may be determined
by either of two independent facts. The set of all minimal supports forms an antichain under subset
order: if one minimal support strictly contained another, the larger one would not be minimal.

## 5.7 Exact and conservative residue

Given a changed input set \(\Delta X\), define the exact semantic residue

\[
\operatorname{Res}_{\mathrm{sem}}(\Delta X)
=
\{Y\mid
\exists S\in\operatorname{MinSupp}(Y),\
S\cap\Delta X\neq\varnothing
\text{ and the change alters }Y
\}.
\]

The last clause matters: intersection with a support makes change possible, not inevitable.

A declared dependency graph yields a conservative residue

\[
\operatorname{Res}_{\mathrm{dep}}(\Delta X)
=
\operatorname{Reach}^{+}_{D}(\Delta X).
\]

If every real semantic dependency is an edge or path in \(D\), then

\[
\operatorname{Res}_{\mathrm{sem}}(\Delta X)
\subseteq
\operatorname{Res}_{\mathrm{dep}}(\Delta X).
\]

This guarantees no under-regeneration but may over-regenerate. Therefore “provably minimal
regeneration” is lawful only for a dependency representation proven complete enough to establish
the semantic support claim. Ordinary graph reachability alone supports a conservative, not exact,
claim.

## 5.8 Backward slicing

Let \(D=(V,E)\) be a dependency graph and \(T\subseteq V\) the target goal nodes. The backward slice is

\[
\operatorname{Slice}(T)
=
\{v\in V\mid
\exists t\in T,\ v=t\text{ or }(v,t)\in(E^{+})\}.
\]

A reverse graph traversal computes this set in

\[
O(|V|+|E|)
\]

time using adjacency lists, because each vertex and edge need be inspected at most a constant number
of times.

The slice is a planning **upper bound**. Semantic rules may further prove some vertices redundant.
Removing a vertex requires a proof that it is not load-bearing for any admitted target path.

## 5.9 Semantic contraction contract

A contraction operator \(\chi\) is sound for goal \(g\) when:

\[
\forall \pi,\
\operatorname{ValidPlan}_{G}(\pi,g)
\Rightarrow
\exists\pi',\
\operatorname{ValidPlan}_{\chi(G,g)}(\pi',g)
\]

for the selected planning semantics. This says contraction does not destroy solvability. A stronger
trace-preservation property would require correspondence between all valid plans, not merely
existence.

At present, exact contraction across every MFW ontology and planner feature is a formalization
program, not a universal finished theorem. Each pack must state which preservation result it
actually establishes.

---

# Chapter 6. Manufacture as an Algebra of Lawful Consequence

## 6.1 Generation is not manufacture

A generator produces candidates:

\[
\operatorname{gen}:O^{*}\to\mathcal{P}(C).
\]

Manufacture selects, verifies, permission-binds, and realizes lawful candidates:

\[
\mu_{B,L,P}:O^{*}\times g
\to
\mathsf{Outcome}(A,E).
\]

The difference is expressed by the inclusion

\[
\operatorname{Manufactured}(O^{*},g)
\subseteq
\operatorname{Generated}(O^{*},g).
\]

The reverse inclusion generally fails. A generated plan may violate a policy, exceed a bound, lack a
runtime mapping, fail its tests, or be denied permission.

## 6.2 Lawful design space

Let \(\mathcal{T}_B(O^{*})\) be all candidate transformations expressible inside boundary \(B\). Let
\(\operatorname{adm}_L(t,O^{*},g)\) be the proposition that transformation \(t\) satisfies admitted
law \(L\) for goal \(g\). Let \(\equiv_L\) identify candidates with the same admitted consequence.
The lawful design space is

\[
\mathcal{D}_{B,L}(O^{*},g)
=
\left\{
t\in\mathcal{T}_B(O^{*})
\mid
\operatorname{adm}_L(t,O^{*},g)
\right\}/{\equiv_L}.
\]

This quotient prevents superficial syntactic variants from inflating the apparent design space. The
space is finite only after explicit bounds on components, parameter domains, recursion, and rule
closure.

## 6.3 Design for Combinatorial Maximalism

Suppose a design has \(k\) independently selectable dimensions, with finite option sets
\(X_1,\ldots,X_k\). The unconstrained Cartesian design space is

\[
\mathcal{X}=\prod_{i=1}^{k}X_i
=
X_1\times\cdots\times X_k,
\]

with cardinality

\[
|\mathcal{X}|=\prod_{i=1}^{k}|X_i|.
\]

Law and compatibility define a predicate \(C:\mathcal{X}\to\{0,1\}\). The admitted space is

\[
\mathcal{X}_{C}=\{x\in\mathcal{X}\mid C(x)=1\}.
\]

**Design for Combinatorial Maximalism (DfCM)** means manufacturing the largest truthfully admitted
capability surface inside the declared bounds, while preserving the distinctions that let the
system refuse unlawful combinations. It does not mean selecting every option or maximizing code
volume.

For a capability set function \(K:\mathcal{P}(\mathcal{X}_C)\to\mathbb{R}_{\ge0}\), a maximal
manufactured family under budget \(b\) is

\[
Y^{*}\in
\operatorname*{arg\,max}_{Y\subseteq\mathcal{X}_C}
K(Y)
\quad\text{subject to}\quad
\operatorname{cost}(Y)\le b.
\]

Because \(\mathcal{X}_C\) is finite, a maximizer exists whenever at least one feasible \(Y\) exists.
Computing it may still be combinatorially expensive; existence is not an efficiency theorem.

## 6.4 Manufacture as a partial algebra

Let \(\mathcal{A}\) be artifact types. Define a partial binary composition

\[
\odot:\mathcal{A}\times\mathcal{A}\rightharpoonup\mathcal{A}.
\]

The expression \(a\odot b\) is defined only when interfaces, versions, identities, policy, and
evidence obligations are compatible. Turning the partial operation into a total result yields

\[
\widehat{\odot}(a,b)
\in
\mathsf{Composed}(\mathcal{A})
\sqcup
\mathsf{Refused}(\mathcal{R})
\sqcup
\mathsf{Inconsistent}(\mathcal{K}).
\]

This is the algebraic form of typed refusal. An interface mismatch is not a null artifact and not an
exception erased by a retry.

## 6.5 Deterministic projection

Let \(G\) be a canonical admitted graph and \(q\) a projection template. A deterministic projector
is

\[
\pi_q:G\to B^{*},
\]

where \(B^{*}\) is a finite byte string. Determinism means

\[
G_1=G_2\land q_1=q_2
\Rightarrow
\pi_{q_1}(G_1)=\pi_{q_2}(G_2).
\]

If environment values, timestamps, map iteration order, or network responses affect projection,
they must either be normalized or included among explicit inputs. Otherwise the function signature
is dishonest.

The artifact receipt records

\[
\left(
\operatorname{digest}(G),
\operatorname{digest}(q),
\operatorname{toolchain},
\operatorname{digest}(\pi_q(G))
\right).
\]

## 6.6 ggen as a manufacturing compiler

Within the architecture, ggen consumes admitted RDF and projects artifacts such as Rust types, PDDL
models, POWL RDF, SPARQL, Tera outputs, Arazzo documents, AIR transition descriptions, and proof
candidates. Its authority is compiler-like:

\[
\text{admitted semantic source}
\xrightarrow{\text{ggen}}
\text{generated candidate artifacts}.
\]

The generated artifacts acquire standing only after their own validators run. “Generated by ggen”
is provenance, not proof. For example:

- generated Rust must compile and pass selected tests;
- generated PDDL must parse and a found plan must verify;
- generated POWL must satisfy structural constraints;
- generated Arazzo must validate against its specification profile;
- generated Lean must elaborate and its proof term must be accepted by the selected kernel.

This preserves one interpretation while retaining format-specific verification.

## 6.7 Shared interpretation and commuting projections

Suppose one semantic graph projects to artifacts \(A\) and \(B\). Let
\(\llbracket-\rrbracket_A\) and \(\llbracket-\rrbracket_B\) map artifacts back to their formal
meanings. A desired commuting property is

\[
\llbracket\pi_A(G)\rrbracket_A
=
\llbracket\pi_B(G)\rrbracket_B
=
\llbracket G\rrbracket.
\]

This equation is a proof obligation, not an automatic consequence of common generation. Sharing an
input reduces drift risk; it does not prove semantic equality. Each projection pair needs a
correspondence theorem or a bounded differential test whose claim is explicitly empirical.

## 6.8 Property-scoped audit

No artifact is simply “verified.” Let \(P_1,\ldots,P_m\) be properties. Verification produces a
vector

\[
v(A)=
(s_1,\ldots,s_m),
\qquad s_i\in\Sigma.
\]

A Rust package might be Alive for compilation, PartialAlive for clean-room portability, Refused for
publication metadata, and Unknown for a target architecture. The vector prevents one green
coordinate from promoting every property.

## 6.9 The edge as the unit of integration

Components often exist while their connections do not. Let an integration edge be

\[
e=(u,v,\tau,\phi,E,F),
\]

where \(u\) and \(v\) are component identities, \(\tau\) is the transferred type, \(\phi\) is the
actual mapping, \(E\) is evidence, and \(F\) is a falsifier. An edge is Real only when a same-object
execution traversed \(\phi\) with a non-vacuous payload and its receiver used the result.

This is why the exact crown prefix

\[
F02\rightarrow F03\rightarrow F08\rightarrow F09\rightarrow F10
\]

cannot be declared real because all five families independently exist. Every arrow is an additional
claim.

---


# Part IV. Planning and Process Geometry

# Chapter 7. Finite Planning from STRIPS and PDDL

## 7.1 A finite planning task

Let \(F=\{f_1,\ldots,f_n\}\) be a finite set of ground Boolean fluents. A **state** is a subset

\[
s\subseteq F.
\]

Membership means truth under the closed finite planning model:

\[
f\in s \Longleftrightarrow f\text{ is true in state }s.
\]

Absence means false **inside this planner model**, not necessarily false in the open world described
by RDF. Admission must perform the open-world-to-closed-world boundary explicitly.

An action is a tuple

\[
a=(\operatorname{pre}^{+}(a),
\operatorname{pre}^{-}(a),
\operatorname{add}(a),
\operatorname{del}(a)),
\]

where each coordinate is a subset of \(F\). Positive preconditions must be present, negative
preconditions absent, add effects become present, and delete effects become absent.

Action \(a\) is applicable in \(s\) when

\[
\operatorname{Applicable}(a,s)
\Longleftrightarrow
\operatorname{pre}^{+}(a)\subseteq s
\land
\operatorname{pre}^{-}(a)\cap s=\varnothing.
\]

The deterministic modeled transition is

\[
\gamma(s,a)
=
\bigl(s\setminus\operatorname{del}(a)\bigr)
\cup
\operatorname{add}(a)
\]

when \(a\) is applicable. Otherwise \(\gamma(s,a)\) is undefined or returns a typed refusal.

A finite planning task is

\[
\Pi=(F,A,s_0,G^{+},G^{-}),
\]

where \(A\) is a finite action set, \(s_0\subseteq F\) is the initial state, and the goal requires

\[
G^{+}\subseteq s
\quad\land\quad
G^{-}\cap s=\varnothing.
\]

PDDL is a concrete language for declaring richer versions of such domains and problem instances.
This dissertation reasons over the finite grounded core after parsing, type checking, and grounding.

## 7.2 Plans and execution

A plan is a finite sequence of actions

\[
\pi=(a_1,\ldots,a_m).
\]

Define its modeled execution recursively:

\[
\gamma^{*}(s,())=s
\]

for the empty plan, and

\[
\gamma^{*}(s,(a_1,\ldots,a_m))
=
\gamma^{*}(\gamma(s,a_1),(a_2,\ldots,a_m))
\]

when each transition is defined.

The plan is valid when:

1. every action is applicable at the state where it occurs; and
2. the final state satisfies the goal.

Formally,

\[
\operatorname{Valid}(\Pi,\pi)
\Longleftrightarrow
\gamma^{*}(s_0,\pi)\downarrow
\land
G^{+}\subseteq\gamma^{*}(s_0,\pi)
\land
G^{-}\cap\gamma^{*}(s_0,\pi)=\varnothing,
\]

where \(\downarrow\) means “is defined.”

## 7.3 The independent plan verifier

**Algorithm 7.1 — Plan witness verification**

Input: finite task \(\Pi\) and proposed plan \(\pi=(a_1,\ldots,a_m)\).

1. Set \(s:=s_0\).
2. For \(i=1,\ldots,m\):
   1. check that \(a_i\in A\);
   2. check \(\operatorname{Applicable}(a_i,s)\);
   3. if not, return Invalid with \(i\), \(s\), and the failed precondition;
   4. set \(s:=\gamma(s,a_i)\).
3. Check the goal in \(s\).
4. Return Valid with the complete state trace, or Invalid with unsatisfied goal atoms.

Using bit vectors for states and action masks, each applicability and transition check costs
\(O(\lceil |F|/w\rceil)\) machine-word operations for word width \(w\). Total verifier cost is

\[
O\left(m\left\lceil\frac{|F|}{w}\right\rceil\right).
\]

### Theorem 7.1 — Verifier soundness

If Algorithm 7.1 returns Valid for \((\Pi,\pi)\), then \(\operatorname{Valid}(\Pi,\pi)\).

**Proof.** The loop checks every action's membership and applicability before applying exactly the
defined transition. By induction on the plan index, the maintained state equals
\(\gamma^{*}(s_0,(a_1,\ldots,a_i))\). The final check establishes both goal clauses. These are exactly
the definition of plan validity. \(\square\)

This theorem is about the modeled transition. It does not prove that an external Cargo command
succeeded.

## 7.4 Finite state-space size

Because a state is a subset of \(F\), the number of possible states is at most

\[
|\mathcal{P}(F)|=2^{|F|}.
\]

Some bit patterns may be unreachable or inconsistent with domain invariants, so the reachable set
can be smaller. It cannot be larger.

Construct the reachability graph

\[
G_{\Pi}=(S_{\Pi},E_{\Pi}),
\]

where \(S_{\Pi}\subseteq\mathcal{P}(F)\) contains reachable states and

\[
(s,s')\in E_{\Pi}
\Longleftrightarrow
\exists a\in A,\
\operatorname{Applicable}(a,s)
\land
s'=\gamma(s,a).
\]

## 7.5 Exhaustive breadth-first search

**Algorithm 7.2 — Truthful finite breadth-first planner**

Input: task \(\Pi\) and optional limits \(K\).

1. Initialize queue \(Q\) with \(s_0\), visited set \(V=\{s_0\}\), and empty predecessor map.
2. While \(Q\) is not empty:
   1. if the front state satisfies the goal, reconstruct a plan and verify it with Algorithm 7.1;
   2. if a declared bound is reached while \(Q\neq\varnothing\), return Bounded with the frontier;
   3. remove the front state \(s\);
   4. for each \(a\in A\), if applicable, compute \(s'=\gamma(s,a)\);
   5. if \(s'\notin V\), record its predecessor, insert it into \(V\), and append it to \(Q\).
3. If the queue becomes empty, return Exhausted with the visited-set certificate.

### Theorem 7.2 — Termination without external interruption

For finite \(F\) and \(A\), Algorithm 7.2 terminates.

**Proof.** There are at most \(2^{|F|}\) distinct states. A state is added to the queue only if it is
not in \(V\), and is inserted into \(V\) simultaneously. Therefore each state is queued at most once.
Each queued state is removed once and examines finitely many actions. The loop performs finitely many
iterations. \(\square\)

### Theorem 7.3 — Found soundness

If Algorithm 7.2 returns Found with plan \(\pi\), then \(\pi\) is valid.

**Proof.** The algorithm returns Found only after Algorithm 7.1 returns Valid. Apply Theorem 7.1.
\(\square\)

### Theorem 7.4 — Exhaustion completeness for the exact finite model

If Algorithm 7.2 returns Exhausted and no pruning rule removes a reachable transition, then no valid
plan exists for the exact grounded task \(\Pi\).

**Proof.** Breadth-first traversal begins at \(s_0\) and adds every successor of every visited state.
By induction on path length, every reachable state is visited. Exhausted is returned only when the
queue is empty and no visited state satisfied the goal. Any valid plan would define a finite path
from \(s_0\) to a goal state, which would have been visited. Contradiction. \(\square\)

The scope phrase “exact grounded task” is load-bearing. A model may omit a real capability. Exhaustion
then proves no plan exists in the model, not that no real-world solution exists.

## 7.6 Bounds

Let bounds be a tuple

\[
K=(k_{\mathrm{states}},k_{\mathrm{depth}},k_{\mathrm{time}},
k_{\mathrm{cost}},k_{\mathrm{memory}}).
\]

A bound is part of the admitted problem. When any coordinate is reached before frontier exhaustion,
the result is

\[
\mathsf{Bounded}(K,V,Q),
\]

including visited states and resumable frontier. Dropping \(Q\) destroys evidence required to
distinguish “search stopped” from “nothing remains.”

## 7.7 Unsupported and inconsistent tasks

Unsupported is returned when syntax or semantics requested by the domain are outside the selected
planner profile: for example, an unimplemented numeric effect or an unbounded external oracle.
Inconsistent is returned when the admitted initial state or invariant set cannot be jointly
satisfied. Neither is a search failure.

## 7.8 From RDF to grounded planning state

Let \(Q_F\) be a finite set of SPARQL or compiled graph queries mapping admitted graph patterns to
ground fluents. The extraction function is

\[
\epsilon_{Q_F}:O^{*}\to\mathcal{P}(F).
\]

For every fluent \(f\), the planning pack must declare its positive evidence query and, when
closed-world absence is used, the closure assumption that makes absence meaningful. The admission
receipt binds \(O^{*}\), \(Q_F\), and the resulting bit vector.

The mapping is reversible only when a decoder and provenance are provided. MFW does not assume that
an arbitrary PDDL state can reconstruct all RDF meaning. Planning is a goal-specific semantic
contraction.

## 7.9 Modeled effects are not real effects

The equation

\[
\gamma(s,a)=s'
\]

is an implication inside the planning model. It does not shell out to a command. In the honest
architecture:

\[
\begin{aligned}
\text{PDDL action} &\longrightarrow \text{planned obligation},\\
\text{POWL activity} &\longrightarrow \text{process position},\\
\text{broker dispatch} &\longrightarrow \text{real command attempt},\\
\text{observer} &\longrightarrow \text{result observation},\\
\text{admission} &\longrightarrow \text{new state fact}.
\end{aligned}
\]

Only the last step permits the next plan to rely on command success. A PDDL effect such as
\(\operatorname{packageVerified}\) is a predicted state transition. The corresponding fact becomes
authoritative only after the harness runs the package verification and its evidence crosses
admission.

## 7.10 Plan explanation

A plan explanation is not merely the action list. For each action \(a_i\), MFW records:

\[
E_i=(a_i,\operatorname{pre}(a_i),s_{i-1},
\operatorname{effect}(a_i),s_i,\operatorname{support}(a_i),\operatorname{falsifier}(a_i)).
\]

The explanation allows a user to see why the action is presently applicable, which admitted facts
support it, what it is predicted to change, and what observation would prove it failed.

---

# Chapter 8. POWL Process Geometry

## 8.1 Why a valid sequence is not enough

A PDDL planner may return one linear sequence even when many actions are causally independent. A
sequence is a witness, not the complete geometry. Process execution needs hierarchy, concurrency,
choice, looping, repair sockets, and evidence boundaries.

POWL is used as the authority for this **process geometry**. The preferred statement is:

> POWL exposes topology-derived concurrency.

The claim is not “branchless search.” Search can branch internally. POWL represents the resulting
causal and choice structure without forcing every independent activity into an arbitrary sequence.

## 8.2 A formal workflow core

Define a finite workflow object

\[
W=(V,\prec,\lambda,\mathcal{C},\mathcal{H},I,O,\mathcal{S},\mathcal{Q}),
\]

where:

- \(V\) is a finite set of activity identities;
- \(\prec\subseteq V\times V\) is a strict causal order;
- \(\lambda:V\to\mathcal{L}\) labels activities;
- \(\mathcal{C}\) is a finite choice-graph structure;
- \(\mathcal{H}\) maps composite activities to child workflows;
- \(I\subseteq V\) is the nonempty entry set;
- \(O\subseteq V\) is the nonempty exit set;
- \(\mathcal{S}\subseteq V\) is the set of typed graft sockets; and
- \(\mathcal{Q}\) is the set of obligations: permission, evidence, cost, and receipt constraints.

The strict order is:

\[
\neg(v\prec v)
\]

and

\[
u\prec v\land v\prec w\Rightarrow u\prec w.
\]

Because it is irreflexive and transitive, it contains no causal cycle. Cyclic behavior is represented
by an explicit loop or choice construct rather than by corrupting the strict precedence relation.

## 8.3 Trace language

A **linear extension** of \((V,\prec)\) is a sequence containing each selected activity exactly once
such that

\[
u\prec v\Rightarrow
\operatorname{position}(u)<\operatorname{position}(v).
\]

Choice semantics selects a lawful substructure; loop semantics can create repeated activity
occurrences, each with a distinct execution-event identity. The trace language

\[
\mathcal{L}(W)\subseteq\mathcal{L}^{*}
\]

contains label sequences admitted by partial order, choices, hierarchy, and loop bounds.

Two workflows are trace-equivalent when

\[
W_1\equiv_{\mathrm{tr}}W_2
\Longleftrightarrow
\mathcal{L}(W_1)=\mathcal{L}(W_2).
\]

Trace equivalence does not preserve timing, resource use, provenance, or internal branching
identity. Stronger equivalences must include the properties they claim.

## 8.4 Primitive composition operators

Assume vertex identities are made disjoint by alpha-renaming before composition.

### Sequence

For workflows \(W_1,W_2\), sequence composition adds precedence from every exit of \(W_1\) to every
entry of \(W_2\):

\[
W_1;W_2.
\]

Its language is concatenation:

\[
\mathcal{L}(W_1;W_2)
=
\{xy\mid x\in\mathcal{L}(W_1),y\in\mathcal{L}(W_2)\}.
\]

### Partial-order parallel composition

Parallel composition takes the disjoint union without adding cross-order edges:

\[
W_1\parallel W_2.
\]

Its traces are interleavings that preserve each component's internal order, subject to shared
resource and choice compatibility.

### Exclusive choice

For a family \(\{W_i\}_{i\in J}\),

\[
\mathsf{Choice}_{i\in J}(W_i)
\]

has trace language

\[
\bigcup_{i\in J}\mathcal{L}(W_i),
\]

with the selected branch and its decision evidence recorded.

### Bounded loop

A loop with body \(W\) and iteration bound \(d\in\mathbb{N}\) has language

\[
\mathcal{L}(\mathsf{Loop}_{\le d}(W))
=
\bigcup_{k=0}^{d}\mathcal{L}(W)^k.
\]

Unbounded semantic looping may be expressed abstractly, but execution under Operation Dogfood must
carry a descent, time, cost, or event bound that makes the run outcome truthful.

## 8.5 The topology of a finite poset

A topology on a set \(X\) is a collection \(\tau\subseteq\mathcal{P}(X)\) such that:

1. \(\varnothing\in\tau\) and \(X\in\tau\);
2. any union of members of \(\tau\) is in \(\tau\);
3. any finite intersection of members of \(\tau\) is in \(\tau\).

A finite poset \((V,\leq)\) induces an Alexandrov topology. Choose upward-closed sets:

\[
U\in\tau_{\uparrow}
\Longleftrightarrow
\forall x\in U,\ \forall y\in V,\
x\leq y\Rightarrow y\in U.
\]

The smallest open neighborhood of \(x\) is

\[
\uparrow x=\{y\in V\mid x\leq y\}.
\]

In a workflow, \(\uparrow x\) is the causal future of \(x\); the downward closure

\[
\downarrow x=\{y\in V\mid y\leq x\}
\]

is its causal history.

This topology gives precise meanings to “local process neighborhood,” “future closure,” and
“affected causal region.” It does not by itself encode resource conflicts or exclusive choices;
those are additional relations.

## 8.6 Topology-derived concurrency

Define causal incomparability:

\[
x\parallel_{\prec}y
\Longleftrightarrow
\neg(x\prec y)\land\neg(y\prec x)\land x\neq y.
\]

Let \(\operatorname{Compat}(x,y,s)\) mean that choice, permission, and resource constraints allow
both activities in state \(s\). Then actual concurrent eligibility is

\[
\operatorname{Concurrent}(x,y,s)
\Longleftrightarrow
x\parallel_{\prec}y
\land
\operatorname{Compat}(x,y,s).
\]

An antichain supplies candidates for concurrency. Compatibility filters them. The **width**

\[
\operatorname{width}(W)
=
\max\{|A|:A\subseteq V\text{ is an antichain}\}
\]

is an upper bound on causal parallelism, not a promise that resources can run all members
simultaneously.

## 8.7 Cuts and frontiers

For completed activity set \(C\subseteq V\), the enabled frontier is

\[
\operatorname{Frontier}(C)
=
\left\{
v\in V\setminus C
\;\middle|\;
\forall u\prec v,\ u\in C
\right\},
\]

further filtered by choice and state preconditions. A frontier is the process analog of a cut:
everything causally required behind it is complete, and its members are candidates for the next
motion.

The external cut separates activities executable within the local engine from those requiring an
external protocol or runtime:

\[
\operatorname{cut}:V\to
\{\mathsf{Local},\mathsf{External}\}.
\]

This classification must come from admitted authority at or above the process geometry. If the
underlying PDDL action contains no external-region information, the POWL projector cannot honestly
invent it.

## 8.8 Choice graphs

POWL v2 extends strictly block-structured choice with choice graphs capable of representing
non-block-structured decisions and cycles. In this dissertation, a choice graph is an explicit
finite structure

\[
C=(N_C,E_C,\eta_C)
\]

whose nodes represent decision states, whose edges represent admitted choice transitions, and whose
labels \(\eta_C\) contain guards and selected workflow regions. Choice-graph execution remains
separate from the causal strict order: a decision can enable or suppress activities without
asserting that all unselected alternatives occurred.

The primary POWL literature establishes language-preserving transformations for selected classes of
safe and sound workflow nets and develops hierarchical decomposition for separable workflow nets.
MFW does not extend those published theorems to arbitrary private models by assertion. Each import
or projection must state the exact supported class.

## 8.9 Structural well-formedness

A workflow is structurally well formed when:

1. \(V\) is finite and identities are unique;
2. \(\prec\) is a strict partial order;
3. every activity lies on a path from an entry to an exit within its selected region;
4. hierarchy is finite or bounded by an explicit descent measure;
5. choice references existing nodes and has deterministic guard evaluation under admitted state;
6. entry and exit interfaces are nonempty and type compatible;
7. every socket declares required precondition, postcondition, authority, and obligations; and
8. every executable activity has a broker mapping or a typed Unsupported result.

SHACL can enforce many structural clauses; semantic clauses such as trace preservation may require
an external verifier or theorem.

---

# Chapter 9. Recursive Grafting, Free Structure, and Termination

## 9.1 Socket interfaces

A socket at activity \(a\in V\) has interface

\[
\mathcal{I}(a)
=
(P_a,Q_a,\mathcal{A}_a,\mathcal{M}_a,\mathcal{E}_a,\mathcal{R}_a),
\]

where:

- \(P_a\) is the required precondition;
- \(Q_a\) is the promised postcondition;
- \(\mathcal{A}_a\) is allowed authority;
- \(\mathcal{M}_a\) is allowed mutation;
- \(\mathcal{E}_a\) is the evidence obligation;
- \(\mathcal{R}_a\) is the receipt obligation.

A child workflow \(U\) is admissible at \(a\) when:

\[
\begin{aligned}
\operatorname{Pre}(U)&\Leftarrow P_a,\\
\operatorname{Post}(U)&\Rightarrow Q_a,\\
\operatorname{Authority}(U)&\subseteq\mathcal{A}_a,\\
\operatorname{Mutation}(U)&\subseteq\mathcal{M}_a,\\
\operatorname{Evidence}(U)&\supseteq\mathcal{E}_a,\\
\operatorname{Receipts}(U)&\supseteq\mathcal{R}_a.
\end{aligned}
\]

The first line says the socket's precondition is sufficient to start the child. The second says child
completion is strong enough to discharge the socket promise. The final four lines forbid silent
authority widening or obligation deletion.

## 9.2 Exact graft construction

Let parent

\[
W=(V,\prec,\ldots)
\]

contain socket \(a\). Let child

\[
U=(V_U,\prec_U,\ldots)
\]

have vertex identities disjoint from \(V\setminus\{a\}\). Define:

\[
\operatorname{Pred}(a)=\{x\in V\mid x\prec a
\text{ and no }z\text{ satisfies }x\prec z\prec a\},
\]

\[
\operatorname{Succ}(a)=\{y\in V\mid a\prec y
\text{ and no }z\text{ satisfies }a\prec z\prec y\}.
\]

The grafted vertex set is

\[
V'=(V\setminus\{a\})\cup V_U.
\]

Start with parent order pairs not incident to \(a\), add the child order, add each predecessor-to-entry
pair, and each exit-to-successor pair:

\[
\begin{aligned}
R'={}&
\{(x,y)\in\prec\mid x\neq a\land y\neq a\}\\
&\cup\prec_U\\
&\cup(\operatorname{Pred}(a)\times I_U)\\
&\cup(O_U\times\operatorname{Succ}(a)).
\end{aligned}
\]

The new strict order is the transitive closure

\[
\prec'=(R')^{+}.
\]

Labels, choices, hierarchy, sockets, and obligations are inherited from the parent and child, with
the socket \(a\) removed and provenance added. Write the result as

\[
W[a\mapsto U].
\]

## 9.3 Graft preserves acyclicity

### Theorem 9.1 — Acyclic graft theorem

Assume:

1. parent and child causal orders are acyclic;
2. parent and child vertex identities are disjoint after removing \(a\);
3. the only cross edges enter the child through \(I_U\) from parent predecessors or leave the child
   through \(O_U\) to parent successors; and
4. no parent path runs from a successor of \(a\) back to a predecessor of \(a\).

Then \(W[a\mapsto U]\) is acyclic.

**Proof.** Suppose a cycle exists in the graft. If it lies entirely in the parent remainder, it
contradicts parent acyclicity. If it lies entirely in the child, it contradicts child acyclicity.
Therefore it crosses the boundary. The only way to enter the child is from a parent predecessor of
\(a\), and the only way to leave is to a parent successor. To return and complete a cycle, a parent
path must lead from a successor back to a predecessor. Assumption 4 forbids this. Therefore no cycle
exists. \(\square\)

Assumption 4 follows automatically from a strict parent order: if \(p\prec a\prec s\), transitivity
gives \(p\prec s\); a path \(s\prec^{+}p\) would produce \(p\prec p\).

## 9.4 Graft preserves unaffected order

### Theorem 9.2 — Context preservation

For parent vertices \(x,y\neq a\), if \(x\prec y\) before grafting, then \(x\prec' y\) afterward.

**Proof.** If the relation does not use \(a\), it is included directly in \(R'\). If its only
witness in a transitive reduction traverses \(a\), then \(x\) lies in the causal past of \(a\) and
\(y\) in its future. The construction creates a path from a predecessor region into the child,
through a child entry-to-exit path, and out to the successor region. Transitive closure restores
\(x\prec' y\). The structural well-formedness assumption that each child entry can reach an exit is
used here. \(\square\)

## 9.5 Obligation monotonicity

Let obligations be ordered by set inclusion: more obligations means a stronger required evidence
surface. Define

\[
\mathcal{Q}(W[a\mapsto U])
=
(\mathcal{Q}(W)\setminus\mathcal{Q}(a))
\cup
\mathcal{Q}(U)
\cup
\mathcal{Q}_{\mathrm{graft}}.
\]

Socket admissibility requires \(\mathcal{Q}(a)\subseteq\mathcal{Q}(U)\). Therefore:

### Theorem 9.3 — No obligation erasure

\[
\mathcal{Q}(W)\setminus\mathcal{Q}(a)
\subseteq
\mathcal{Q}(W[a\mapsto U])
\]

and every obligation of \(a\) is discharged or retained by \(U\).

**Proof.** The first inclusion follows from the union definition. Socket admissibility gives
\(\mathcal{Q}(a)\subseteq\mathcal{Q}(U)\), and \(\mathcal{Q}(U)\) is also a union operand. \(\square\)

## 9.6 Associativity up to identity renaming

Suppose \(a\) is a socket in \(W\) and \(b\) a socket in child \(U\). For fresh disjoint identities,

\[
(W[a\mapsto U])[b\mapsto Z]
\cong
W[a\mapsto(U[b\mapsto Z])],
\]

where \(\cong\) is isomorphism under alpha-renaming.

The equality is not literal because fresh vertex identifiers can differ depending on construction
order. Both sides replace the same nested sockets, preserve the same parent, child, and grandchild
orders, and create the same boundary relations. A bijection that is identity on surviving parent
vertices and maps corresponding fresh child vertices establishes the isomorphism.

This law makes recursive workflow composition predictable: manufacturing a grandchild before or
after inserting its parent does not change the process meaning when interfaces and identities are
fresh.

## 9.7 Descent meter and termination

Define a recursive growth call

\[
\operatorname{Grow}(W,r,d),
\qquad d\in\mathbb{N}.
\]

If \(d=0\) and residue remains, return

\[
\mathsf{Bounded}(\mathsf{DescentExhausted},r).
\]

If \(d>0\), every recursive child call receives \(d-1\).

### Theorem 9.4 — Descent termination

Every chain of recursive Grow calls terminates after at most the initial \(d\) child descents,
assuming each nonrecursive admission, planning, and verification call terminates or returns its own
bound.

**Proof by induction on \(d\).**

Base case \(d=0\): no recursive call is permitted, so the function returns immediately with a result.

Inductive step: assume every call with budget \(d\) terminates. A call with budget \(d+1\) performs a
finite local phase. If no child is needed, it returns. If a child is needed, the child receives
\(d\), which terminates by the induction hypothesis. The parent then performs a finite return phase
and terminates. Therefore the proposition holds for \(d+1\). By induction it holds for every
\(d\in\mathbb{N}\). \(\square\)

This theorem does not prove a child goal will be solved. It proves the bounded recursion will return
a truthful outcome rather than descend forever.

## 9.8 Workflow syntax as a free structure

Let \(A\) be terminal result values and let \(F\) be a workflow-operation signature containing
sequence, partial order, choice, bounded loop, observe, ask, actuate, receipt, and repair requests.
The free recursive syntax is

\[
\mathsf{Free}_F(A)
=
\mathsf{Pure}(A)
\sqcup
\mathsf{Impure}(F(\mathsf{Free}_F(A))).
\]

The constructor Pure contains a completed value. Impure contains one layer of workflow operation
whose continuations are themselves workflows.

Define bind:

\[
\begin{aligned}
\mathsf{Pure}(a)\mathbin{\gg=}k &= k(a),\\
\mathsf{Impure}(u)\mathbin{\gg=}k
&=
\mathsf{Impure}\bigl(F(\lambda x.\ x\mathbin{\gg=}k)(u)\bigr).
\end{aligned}
\]

Under ordinary functor laws for \(F\), structural induction yields the monad laws:

\[
\mathsf{Pure}(a)\mathbin{\gg=}k=k(a),
\]

\[
m\mathbin{\gg=}\mathsf{Pure}=m,
\]

\[
(m\mathbin{\gg=}k)\mathbin{\gg=}h
=
m\mathbin{\gg=}(\lambda x.\ k(x)\mathbin{\gg=}h).
\]

The conceptual connection is that a repair socket is a continuation: the child workflow is bound
into the parent where the residue appears. However, an implementation theorem identifying the full
POWL v2 graph representation with this free syntax requires a formal encoding of choice graphs,
identity, and quotienting. That bridge is a **candidate formalization**, not promoted merely by the
analogy.

## 9.9 Parent-child closure

A child completion is a new observation, not automatic parent success. Let child result be
\((A_c,R_c)\). Parent re-entry performs:

\[
O' = O\cup\{\operatorname{observe}(A_c,R_c)\},
\]

\[
O'^{*}=\operatorname{Admit}(O'),
\]

and replans or resumes only if the child's promised postcondition is now admitted. A forged,
expired, scope-mismatched, or invalid receipt returns Inconsistent or Refused.

## 9.10 Compensation is workflow

An external effect may not be reversible. MFW does not pretend that every action has an algebraic
inverse. A compensation for action \(a\) is a separate workflow \(C_a\) with a target restorative
postcondition:

\[
\operatorname{Post}(C_a)\approx\operatorname{Pre}(a),
\]

where \(\approx\) is a declared recovery equivalence, not necessarily equality. Compensation requires
its own permission, evidence, receipt, and possible recursion.

## 9.11 Implementation standing

The project progress record for v26.7.13 reports that a graft_child primitive and the production
F09-to-F10 edge have been implemented, and that a prior false-success corruption was removed with
tests restored. This is implementation evidence for a real slice. It does not by itself prove all
theorems in this chapter are kernel-connected to that implementation. The remaining crown status is
evaluated in Chapter 26.

---

# Chapter 10. Search Graphs, Manufacturing Graphs, and Architecture Search

## 10.1 Two graphs with different physics

MFW separates:

1. the **POWL Search Graph**, used to explore possible process structures; and
2. the **POWL Manufacturing Graph**, used to represent the process actually authorized and enacted.

Confusing them creates both performance and epistemic errors.

Let

\[
G_{\mu}=(V_{\mu},E_{\mu})
\]

be the search graph. Nodes are candidate states or process fragments. Edges are hypothetical
expansions. Its events may last microseconds. Most nodes are discarded.

Let

\[
G_M=(V_M,E_M)
\]

be the manufacturing graph. Nodes are admitted plan steps, permissions, actuations, observations,
artifacts, receipts, and child workflows. Its events may last minutes, hours, or days. Nothing
material may be silently discarded because it is the historical object.

A selection map

\[
\sigma:G_{\mu}\rightharpoonup W
\]

extracts a verified workflow witness. An enactment map

\[
\eta:W\times P\to G_M
\]

begins manufacturing only after permission.

## 10.2 Why search telemetry is not process evidence

Planner node expansion counts measure algorithm behavior. Manufacturing events measure real work.
If a planner expands one million states to choose six actions, it is false to report one million
business-process activities. Conversely, six PDDL actions do not describe all tool events,
retries, observations, and repair children that occur during execution.

The graphs may be linked by provenance:

\[
\operatorname{wasSelectedFrom}(W,G_{\mu}),
\]

but their event classes, clocks, resource measures, and retention policies remain distinct.

## 10.3 Three search levels

### Level 1 — State and plan search

Given fixed architecture, vocabulary, planner semantics, and workflow operators, search for a plan
that reaches a goal.

### Level 2 — Search-topology manufacture

Given a plan witness and dependency structure, manufacture a POWL topology that exposes hierarchy,
choice, concurrency, loops, and external cuts.

### Level 3 — Architecture search

Generate alternative engine and workflow topologies, project each into runnable artifacts, and
benchmark them under identical admitted workloads.

Let architecture candidates be \(\mathcal{A}=\{A_1,\ldots,A_n\}\). For benchmark vector

\[
b(A_i)=(\operatorname{latency},\operatorname{throughput},
\operatorname{memory},\operatorname{energy},
\operatorname{replayCost},\operatorname{proofCoverage}),
\]

architecture search returns a Pareto frontier rather than collapsing incompatible objectives into
one unexplained score.

## 10.4 Pareto order

For minimization coordinates, \(x\) dominates \(y\) when

\[
\forall j,\ x_j\le y_j
\quad\land\quad
\exists k,\ x_k<y_k.
\]

The Pareto frontier contains candidates not dominated by any other candidate. If some coordinates
are maximized, such as proof coverage, negate or reverse those coordinates consistently.

A single winner requires an admitted utility function. Without one, declaring a winner hides a
value judgment.

## 10.5 Architecture correspondence obligation

Level 3 may manufacture radically different topologies. A benchmark comparison is meaningful only
when workloads and semantics match. For candidates \(A_i,A_j\), require:

\[
\operatorname{InputEq}(A_i,A_j),
\quad
\operatorname{GoalEq}(A_i,A_j),
\quad
\operatorname{TraceCriterionEq}(A_i,A_j),
\quad
\operatorname{EvidenceCriterionEq}(A_i,A_j).
\]

If semantic equivalence is unproved, performance results are labeled for the observed workloads
only. Faster non-equivalent behavior is not an optimization of the same system.

## 10.6 Multiscale recursion across levels

The same manufacturing loop can occur at all three levels:

\[
\begin{aligned}
\mathcal{M}_1 &: \text{state}\to\text{plan},\\
\mathcal{M}_2 &: \text{plan}\to\text{process topology},\\
\mathcal{M}_3 &: \text{requirements}\to\text{architecture topology}.
\end{aligned}
\]

Level 3 can select a new Level 2 engine, which then manufactures Level 1 workflows. This is a
structural recursive hierarchy. Whether measured search mass across the hierarchy has a multifractal
spectrum remains an empirical question addressed in Part VIII.

---


# Part V. Permissioned Motion and Evidence

# Chapter 11. Permission, Brokered Actuation, and Type-State Completion

## 11.1 Planning does not grant authority

A valid plan establishes modeled feasibility. It does not answer who may execute it, what may be
changed, how much resource may be consumed, or when the authority expires. Permission is therefore a
separate state transition:

\[
\mathsf{Planned}(\Pi)
\xrightarrow{\operatorname{authorize}}
\mathsf{Authorized}(\Pi,p).
\]

A permission artifact is

\[
p=(d_{\Pi},d_W,\mathcal{M},\mathcal{A},K,t_0,t_1,g,u,\varsigma),
\]

where:

- \(d_{\Pi}\) is the plan digest;
- \(d_W\) is the POWL geometry digest;
- \(\mathcal{M}\) is the allowed mutation set;
- \(\mathcal{A}\) is the allowed action and tool surface;
- \(K\) is the cost, time, recursion, and concurrency bound;
- \([t_0,t_1]\) is the validity interval;
- \(g\) is the exact goal identity;
- \(u\) is the granting agent; and
- \(\varsigma\) is a signature or other admitted authorization evidence.

Permission is applicable only when all bound identities agree with the proposed execution:

\[
\operatorname{ApplicablePermission}(p,x)
\Longleftrightarrow
\begin{cases}
\operatorname{digest}(\Pi_x)=d_{\Pi},\\
\operatorname{digest}(W_x)=d_W,\\
\operatorname{Mutation}(x)\subseteq\mathcal{M},\\
\operatorname{Actions}(x)\subseteq\mathcal{A},\\
\operatorname{Budget}(x)\le K,\\
t_0\le\operatorname{now}\le t_1,\\
\operatorname{Goal}(x)=g,\\
\operatorname{VerifyAuth}(\varsigma,u)=\mathsf{true}.
\end{cases}
\]

If execution discovers a materially larger mutation, new command class, or new external boundary,
the predicate becomes false and the workflow returns to Ask.

## 11.2 ODRL and executable permission

ODRL provides public concepts for permission, prohibition, constraint, duty, assigner, assignee,
target, and action. MFW can represent the policy in ODRL-compatible RDF, but the broker needs a
deterministic compiled decision:

\[
\operatorname{permit}:p\times x\times s\to
\{\mathsf{Allow},\mathsf{Deny},\mathsf{Indeterminate}\}.
\]

Indeterminate is not Allow. Compilation from RDF policy to broker masks or predicates must be
versioned and receipted. A natural-language approval that is not bound to the plan digest is
insufficient for autonomous mutation.

## 11.3 The broker-only actuation law

Let \(\mathcal{X}\) be all side-effecting operations inside the controlled system. Let
\(\mathcal{B}\subseteq\mathcal{X}\) be operations dispatched by the broker. The architectural axiom
for zero ungoverned actuation is

\[
\mathcal{X}=\mathcal{B}.
\]

Equivalently,

\[
\forall x\in\mathcal{X},\
\operatorname{Actuated}(x)\Rightarrow\operatorname{Brokered}(x).
\]

This is an enforceable architecture property only if bypasses are removed or placed outside the
claimed boundary. A shell, direct SDK, hidden network client, or human console path inside \(B\)
violates the premise.

The broker performs:

1. plan-step binding;
2. permission evaluation;
3. durable pre-actuation receipt;
4. effect dispatch;
5. result observation;
6. durable post-actuation receipt or recovery marker; and
7. admission of the result.

## 11.4 Why a pre-receipt is necessary

Suppose a process performs an external effect and then writes its receipt. A crash can occur between
the two operations. Therefore no ordinary two-step implementation can prove

\[
\operatorname{Actuated}(x)\Rightarrow
\operatorname{OutcomeReceiptExists}(x)
\]

without transactionality extending over the external system.

MFW uses a more precise invariant. Before dispatch, it durably writes an **actuation-intent receipt**

\[
R^{-}_x=(x,d_{\Pi},p,\operatorname{expectedEffect},
\operatorname{idempotencyKey},\operatorname{time}).
\]

After observing the result, it writes

\[
R^{+}_x=(\operatorname{digest}(R^{-}_x),
\operatorname{observedResult},\operatorname{evidence},\operatorname{time}).
\]

Then:

\[
\operatorname{Actuated}(x)\Rightarrow\operatorname{PreReceipted}(x),
\]

and

\[
\operatorname{Completed}(x)\Rightarrow\operatorname{PostReceipted}(x).
\]

If a crash follows dispatch but precedes result capture, the action is **not** represented as
Completed. It remains UnknownAfterDispatch and enters recovery. The system probes the external
idempotency key or reconciles observed state before retrying.

This is the honest meaning of zero unreceipted actuation: no brokered side effect lacks a durable
intent receipt, and no completed state lacks a durable outcome receipt. It is not a claim that
distributed failure becomes impossible.

## 11.5 Type-state encoding

Define execution states as a tagged sum:

\[
\begin{aligned}
\mathsf{ExecState}
={}&
\mathsf{Proposed}(\Pi)\\
&\sqcup\mathsf{Authorized}(\Pi,p)\\
&\sqcup\mathsf{Prepared}(\Pi,p,R^{-})\\
&\sqcup\mathsf{Dispatched}(\Pi,p,R^{-},k)\\
&\sqcup\mathsf{Observed}(\Pi,p,R^{-},e)\\
&\sqcup\mathsf{Completed}(\Pi,p,R^{-},R^{+})\\
&\sqcup\mathsf{UnknownAfterDispatch}(\Pi,p,R^{-},k).
\end{aligned}
\]

There is no constructor

\[
\mathsf{Completed}(\Pi,p,\text{no receipt}).
\]

### Theorem 11.1 — Completed receipt inversion

For every value \(x:\mathsf{ExecState}\), if \(x\) has constructor Completed, then values \(R^{-}\)
and \(R^{+}\) exist.

**Proof.** The Completed constructor's fields include both receipt types. Constructor inversion
returns each field. No alternative Completed constructor exists. \(\square\)

This theorem applies to representable state. The broker-only law is required to connect real effects
to representable state.

## 11.6 Idempotency and at-most-once illusions

Networks can duplicate requests and lose responses. An idempotency key \(k\) makes repeated dispatch
requests refer to one logical actuation:

\[
\operatorname{dispatch}(k,x);\operatorname{dispatch}(k,x)
\approx
\operatorname{dispatch}(k,x).
\]

The equivalence is a contract with the target adapter. If the external system lacks idempotency, the
adapter must expose duplicate risk and compensation. MFW does not claim exactly-once real-world
effects from message delivery alone.

## 11.7 Human execution

A human can be a lawful actuator. Human execution is represented as an explicit task with:

- a plan-step identity;
- requested action;
- allowed scope;
- required evidence;
- performer identity;
- start and completion observations;
- review or countersignature where required; and
- pre- and post-receipts.

A human step is not a gap in automation, and an unrecorded human console action is not a valid human
step. Both machine and human effects cross the same standing boundary.

## 11.8 Machine actuation as the default, not the authority

Machine execution is preferred where it reduces interpretation, variance, and latency. The
preference is quantitative:

\[
\operatorname{machinePreferred}
\Longleftrightarrow
\operatorname{expectedError}_{m}
\le
\operatorname{expectedError}_{h}
\land
\operatorname{authorityAvailable}_{m}
\land
\operatorname{evidenceQuality}_{m}
\ge
\operatorname{requiredEvidence}.
\]

The choice remains a plan decision. A machine does not gain authority merely because an adapter
exists.

---

# Chapter 12. Arazzo, AIR, Erlang/OTP, WASM, AtomVM, and Correspondence

## 12.1 Labeled transition systems

A labeled transition system is

\[
\mathcal{T}=(S,\Lambda,\to,s_0),
\]

where \(S\) is a state set, \(\Lambda\) a label set, \(\to\subseteq S\times\Lambda\times S\) a
transition relation, and \(s_0\in S\) an initial state. Write

\[
s\xrightarrow{\ell}s'
\]

when \((s,\ell,s')\in\to\).

A finite trace is a sequence of labels \((\ell_1,\ldots,\ell_n)\) for which states
\(s_1,\ldots,s_n\) exist satisfying

\[
s_0\xrightarrow{\ell_1}s_1
\xrightarrow{\ell_2}\cdots
\xrightarrow{\ell_n}s_n.
\]

The trace set is \(\operatorname{Traces}(\mathcal{T})\).

## 12.2 Simulation

Let abstract system \(\mathcal{T}_A\) and concrete system \(\mathcal{T}_C\) have a state relation
\(R\subseteq S_A\times S_C\) and label abstraction \(\alpha:\Lambda_C\to\Lambda_A^{*}\). A forward
simulation requires:

1. \((s_{A0},s_{C0})\in R\); and
2. whenever \((a,c)\in R\) and \(c\xrightarrow{\ell_C}c'\), an abstract path
   \(a\xRightarrow{\alpha(\ell_C)}a'\) exists with \((a',c')\in R\).

Under these conditions every concrete trace abstracts to an abstract trace. A backward condition is
needed to show the concrete system can realize every abstract behavior. Both directions give a form
of bisimulation when observations also agree.

## 12.3 No ambient correspondence

Two artifacts do not correspond merely because both were generated from RDF. Let

\[
\operatorname{Corr}(A,C,R,\alpha,P)
\]

mean that concrete artifact \(C\) refines abstract artifact \(A\) under relation \(R\), label mapping
\(\alpha\), and preserved property set \(P\). Correspondence is always parameterized. There is no
global relation

\[
\text{same source}\Rightarrow\text{same semantics}.
\]

If the project claims only that mfact proves a law represented in a certified artifact, downstream
runtime correspondence is outside that claim. If the project claims that a deployed binary
implements that law, a bridge becomes mandatory. Scope, not rhetoric, decides the obligation.

## 12.4 The execution ladder

The canonical manufacturing path is:

\[
\begin{aligned}
\text{admitted RDF graph}
&\rightarrow \text{PDDL}\\
&\rightarrow \text{POWL v2}\\
&\rightarrow \text{external cut}\\
&\rightarrow \text{SPARQL/Tera projections}\\
&\rightarrow \text{Arazzo}\\
&\rightarrow \text{wasm4pm AIR}\\
&\rightarrow \text{Erlang transition core}\\
&\rightarrow \text{OTP or AtomVM}\\
&\rightarrow \text{broker}\\
&\rightarrow \text{admitted consequence}\\
&\rightarrow \text{receipt and replay}.
\end{aligned}
\]

Every arrow has its own input, output, version, verifier, evidence, and falsifier. The ladder is not
one theorem.

## 12.5 Arazzo as an inter-engine artifact

The OpenAPI Initiative's Arazzo specification describes sequences of calls and their dependencies
for achieving outcomes. MFW uses Arazzo after the external cut:

\[
\pi_{\mathrm{Arazzo}}:
W_{\mathrm{external}}\to A_z.
\]

An Arazzo artifact records workflow steps, source descriptions, parameters, success criteria, and
dependencies suitable for protocol-level execution. It is manufactured from POWL geometry; it does
not replace POWL as the authority for process topology.

Validation against the Arazzo schema establishes syntactic and profile conformance. Semantic
correspondence to the originating POWL slice requires a step, dependency, input, success, and
failure mapping.

## 12.6 AIR as normalized transition semantics

AIR is treated as a small intermediate transition representation:

\[
\mathsf{AIR}=(Q,\Sigma,\delta,q_0,\mathcal{O}),
\]

where \(Q\) is a finite state representation, \(\Sigma\) admitted events, \(\delta\) a transition
function or typed result, \(q_0\) initial state, and \(\mathcal{O}\) emitted obligations or commands.

Normalizing runtime behavior through one explicit \(\delta\) reduces semantic duplication. It does
not prove every runtime calls \(\delta\) correctly.

## 12.7 Erlang owns outer transition semantics

Erlang/OTP is suited to durable outer-scale workflows because processes, message passing,
supervision, and restart semantics represent long-lived transition systems. One workflow instance
may be represented by an OTP process, but its PID is not its semantic identity:

\[
\operatorname{workflowIRI}\neq\operatorname{PID}.
\]

PIDs can change under restart. The stable workflow identity lives in admitted state and is restored
into a new process.

The outer transition core determines:

- event admission;
- state transition;
- command emission;
- reaction to broker results;
- persistence;
- child workflow linkage;
- recovery after unknown dispatch; and
- terminal receipt formation.

Supervision recovers a process. It does not by itself prove replay equivalence or prevent duplicate
effects.

## 12.8 AtomVM as a constrained shell

AtomVM executes a constrained Erlang/Elixir surface on smaller environments. The supported MFW
profile must declare:

\[
\mathcal{F}_{\mathrm{AtomVM}}
\subseteq
\mathcal{F}_{\mathrm{OTP}},
\]

where \(\mathcal{F}\) is the set of language, library, timing, persistence, and networking features
used by the workflow shell.

If a workflow needs a feature outside the subset, the outcome is Unsupported or the external cut
moves the activity. Silent fallback would violate correspondence.

## 12.9 WASM and wasm4pm cognitive breeds

WASM supplies portable sandboxed execution. wasm4pm cognitive breeds are typed strategies for
reconnaissance, planning, diagnosis, proof search, implementation, verification, or analysis. A
breed is not an authority role. It is a function contract:

\[
b:X_b\to\mathsf{Proposal}(Y_b)\sqcup\mathsf{Outcome}(E_b).
\]

Proposals return to RDF admission. Capability restrictions, fuel, memory, host calls, and input
digests are part of the breed receipt.

## 12.10 BCINR local microphysics

BCINR is the local or chip-scale path for executing POWL-derived structures where the cost of an
outer protocol is inappropriate. Its claim surface is deliberately local:

\[
\operatorname{Latency}_{\mathrm{local}}
\ll
\operatorname{Latency}_{\mathrm{external}}
\]

is an empirical benchmark proposition, not a universal theorem. Semantics still require a mapping
from POWL activity, state, and event identities to the local runtime.

## 12.11 Differential verification

For input corpus \(X\), runtimes \(r_1,r_2\), and normalization \(\nu\), differential testing checks

\[
\forall x\in X,\
\nu(\operatorname{run}_{r_1}(x))
=
\nu(\operatorname{run}_{r_2}(x)).
\]

Equality over a finite corpus is evidence for those cases. It is not a proof for every input unless
the corpus is exhaustive over a finite domain or a theorem generalizes it.

Adversarial mutation strengthens a test: alter one mapping so the outputs should diverge, confirm
the test fails, then restore byte-identically and confirm it passes. This proves the test is
load-bearing for that mutation.

## 12.12 Conditional trace-preservation theorem

### Theorem 12.1 — Composition of simulations

If concrete runtime \(C\) simulates AIR \(A\), and AIR \(A\) simulates POWL semantics \(P\), then
\(C\) simulates \(P\) under the composed state relation and label abstraction.

**Proof.** Let \(R_{AC}\) relate AIR and concrete states, and \(R_{PA}\) relate POWL and AIR states.
Define

\[
R_{PC}
=
\{(p,c)\mid\exists a,\ (p,a)\in R_{PA}\land(a,c)\in R_{AC}\}.
\]

For any concrete transition, the \(C\)-to-\(A\) simulation supplies a matching AIR path. For each
AIR step in that path, the \(A\)-to-\(P\) simulation supplies a matching POWL path. Concatenating the
paths supplies the required POWL match. Initial states relate by the two initial-state premises.
\(\square\)

This theorem shows how evidence composes. It does not assert that either required simulation has
already been proved for every current runtime.

---

# Chapter 13. Receipts, Canonicalization, Replay, and Event Evidence

## 13.1 A receipt is a structured historical claim

A run receipt is

\[
R=(I,O^{*},L,\Pi,W,p,E,A,\mathcal{H},\sigma,t),
\]

where:

- \(I\) is intent;
- \(O^{*}\) is admitted initial state;
- \(L\) is admitted law and toolchain identity;
- \(\Pi\) is the plan witness or truthful planner result;
- \(W\) is process geometry;
- \(p\) is permission;
- \(E\) is the ordered event/evidence collection;
- \(A\) is terminal artifact or outcome;
- \(\mathcal{H}\) is the digest structure;
- \(\sigma\) is a signature or attestation when used; and
- \(t\) is temporal metadata.

The receipt proves that its signer or controlled broker attests to these linked objects under the
recorded verification. It does not automatically prove every object is semantically correct.

## 13.2 Canonicalization

RDF graphs can have many serializations. Blank-node labels can also vary without changing graph
meaning. A canonicalization function

\[
\kappa:\mathcal{D}\to\mathcal{B}
\]

produces one byte representation for each admitted equivalence class in its supported profile:

\[
D_1\cong D_2
\Rightarrow
\kappa(D_1)=\kappa(D_2).
\]

The converse is expected under correct canonicalization plus collision-resistant hashing, but the
hash itself is not mathematically injective. Every receipt names the canonicalization algorithm,
version, RDF profile, and hash.

## 13.3 Hash chaining

Let event bytes after canonicalization be \(e_1,\ldots,e_n\). Define:

\[
h_0=\operatorname{Hash}(\operatorname{header}),
\]

\[
h_i=\operatorname{Hash}(h_{i-1}\mathbin{\|}e_i)
\quad\text{for }1\le i\le n,
\]

where \(\|\) is unambiguous length-delimited concatenation. The final \(h_n\) commits to event order
and content under the hash assumption.

If an event changes, is deleted, inserted, or reordered, subsequent hashes change except in the
event of a collision. A chain proves tamper evidence, not event truth.

## 13.4 Replay from deterministic transitions

Let transition function

\[
\delta:S\times E\to S
\]

be deterministic. Define replay:

\[
\delta^{*}(s,())=s,
\]

\[
\delta^{*}(s,(e_1,\ldots,e_n))
=
\delta^{*}(\delta(s,e_1),(e_2,\ldots,e_n)).
\]

### Theorem 13.1 — Deterministic replay

For identical initial state \(s_0\) and identical event sequence \(E\),

\[
\delta^{*}(s_0,E)=\delta^{*}(s_0,E).
\]

More substantively, two replay implementations that compute the same deterministic \(\delta\) return
the same state for every finite event sequence.

**Proof by induction on event-sequence length.**

Base case: for the empty sequence both return \(s_0\).

Inductive step: assume equality for sequences of length \(n\). For a sequence
\((e_1,\ldots,e_n,e_{n+1})\), both implementations reach the same state after the first \(n\) events
by the induction hypothesis. Both apply the same deterministic \(\delta\) to that state and
\(e_{n+1}\), yielding the same next state. \(\square\)

The theorem's load-bearing premise is “same deterministic transition.” Different runtimes need the
correspondence evidence of Chapter 12.

## 13.5 Nondeterminism becomes input

Wall-clock time, random numbers, network responses, scheduler decisions, human choices, and external
service results are exogenous. To make replay deterministic, record the value that crossed the
workflow boundary:

\[
e_i=(\operatorname{eventType},\operatorname{payload},
\operatorname{source},\operatorname{time},\operatorname{causalLinks}).
\]

Replay does not ask the network for the old response again; it reuses the admitted observation.
Live re-execution is a different experiment and should receive a new run identity.

## 13.6 Byte replay and semantic replay

**Byte replay** requires identical output bytes:

\[
\operatorname{bytes}(A')=\operatorname{bytes}(A).
\]

**Semantic replay** requires equality under a declared equivalence:

\[
A'\equiv_{\mathrm{sem}}A.
\]

A build containing timestamps may fail byte replay but pass a carefully defined semantic replay.
The release must state which criterion applies. “Replayable” without a criterion is under-specified.

## 13.7 Object-centric event evidence

Traditional event logs often assign each event to one case. MFW events can relate simultaneously to
a workflow, plan, repository, package, artifact, permission, child workflow, agent task, command,
and receipt. An object-centric event is

\[
e=(\operatorname{id},\operatorname{type},t,
\operatorname{attributes},\operatorname{relations}),
\]

where relations connect \(e\) to multiple object identities with qualifiers.

OCEL-compatible projection is constructed from admitted RDF:

\[
\pi_{\mathrm{OCEL}}:G_{\mathrm{execution}}\to\mathsf{OCEL}.
\]

The OCEL file is evidence projection, not a second authority. Its digest and generating query are
linked to the RDF source graph.

## 13.8 No orphan lifecycle object

For every execution event \(e\), define required links:

\[
\begin{aligned}
\operatorname{plannedBy}(e)&=\pi_e,\\
\operatorname{authorizedBy}(e)&=p_e,\\
\operatorname{generatedResult}(e)&=r_e,\\
\operatorname{partOfRun}(e)&=\rho_e.
\end{aligned}
\]

The no-orphan invariant is

\[
\forall e\in G_{\mathrm{execution}},
\exists!\pi_e,p_e,r_e,\rho_e
\text{ satisfying the selected cardinality and scope rules.}
\]

Some read-only observations may be authorized by a standing reconnaissance permission rather than
an individual mutation grant, but they still bind to a task and run.

An unbound agent task, tool event without a plan step, result without an actuation, or actuation
without a receipt causes final verification to refuse the run.

## 13.9 Receipt dominance

Let \(\leadsto\) be a provenance dependency. A terminal receipt dominates a side effect \(x\) when
every accepted completion path for the run includes evidence linked from \(x\) into \(R\):

\[
x\leadsto R.
\]

Graph reachability can check syntactic dominance. Semantic completeness additionally requires that
the observer and broker emit all effect classes. Negative fixtures must bypass or delete a link and
confirm the verifier refuses.

## 13.10 What receipts do and do not prove

A valid receipt can establish, relative to its trust and cryptographic assumptions:

- which admitted input identity was used;
- which plan and permission were bound;
- which broker events were recorded;
- which artifacts and results were content-addressed;
- which verifier results were included;
- whether replay reproduced the declared criterion.

It cannot alone establish:

- that an omitted external effect did not happen;
- that a source observation was truthful;
- that a mathematical specification is correct;
- that a concrete binary refines an abstract model;
- that a human understood an approval; or
- that a hash collision is impossible.

Those require architecture, observation, proof, policy, or threat-model arguments.

## 13.11 Commuting operations and replay convergence

Two state transformers \(f,g:S\to S\) commute when

\[
f\circ g=g\circ f.
\]

If they commute, applying them in either order yields the same state:

\[
f(g(s))=g(f(s)).
\]

For a sequence, adjacent commuting operations may be swapped without changing the final state. By
repeated swaps, any two orderings connected by swaps of independent operations converge.

This theorem is conditional on genuine commutation. Distinct tenant identifiers do not guarantee
disjoint effects if both operations mutate a shared quota, index, or global clock. The support
analysis in Chapter 22 supplies the required independence premise.

---


# Part VI. Operation Dogfood

# Chapter 14. The Governed Claude Code Lifecycle

## 14.1 Customer zero

MFW cannot claim to manufacture other systems' workflows while its own discovery, implementation,
verification, and release process remains outside MFW. Operation Dogfood therefore applies the
complete law-state loop to the production of MFW itself.

The user supplies an intended outcome. The system, not the user, must recover the bounded operating
laws of the repository. For the first crown:

> Any Rust developer may point MFW at an existing repository and ask for a dry-run publish. MFW
> figures out what the system is doing, creates a bounded plan, asks permission, executes the
> approved plan, and launches Claude Code when diagnosis, repair, or implementation is required.

The promise includes unfamiliar repositories. Requiring the developer to hand-author the release
workflow would move the central design labor back to the customer.

## 14.2 Lifecycle state machine

Define states:

\[
\begin{aligned}
\mathcal{L}=\{&
\mathsf{IntentCaptured},
\mathsf{Reconnaissance},
\mathsf{Admitted},
\mathsf{Planned},
\mathsf{AwaitingPermission},
\mathsf{Authorized},
\mathsf{Executing},
\mathsf{Repairing},
\mathsf{Verifying},
\mathsf{Receipting},
\mathsf{Replaying},
\mathsf{Terminal}
\}.
\end{aligned}
\]

The principal transition path is

\[
\begin{aligned}
\mathsf{IntentCaptured}
&\to\mathsf{Reconnaissance}
\to\mathsf{Admitted}
\to\mathsf{Planned}\\
&\to\mathsf{AwaitingPermission}
\to\mathsf{Authorized}
\to\mathsf{Executing}\\
&\to\mathsf{Verifying}
\to\mathsf{Receipting}
\to\mathsf{Replaying}
\to\mathsf{Terminal}.
\end{aligned}
\]

If execution reveals repairable residue:

\[
\mathsf{Executing}
\to\mathsf{Repairing}
\to\mathsf{Verifying}
\to\mathsf{Admitted}
\to\mathsf{Planned},
\]

where the recursive child has a smaller descent or resource budget. A material scope change returns
to AwaitingPermission before mutation continues.

## 14.3 RDF end to end for instance state

Every lifecycle object receives a stable IRI or content identity:

\[
\begin{aligned}
&\operatorname{Intent},\operatorname{Run},\operatorname{RepositorySnapshot},
\operatorname{ResearchTask},\operatorname{AgentInvocation},\\
&\operatorname{ToolIntent},\operatorname{ToolResult},\operatorname{Observation},
\operatorname{Claim},\operatorname{Conflict},\\
&\operatorname{PddlProblem},\operatorname{PlanWitness},\operatorname{PowlWorkflow},
\operatorname{Permission},\\
&\operatorname{Patch},\operatorname{TestResult},\operatorname{Package},
\operatorname{Refusal},\operatorname{Receipt},\operatorname{Replay}.
\end{aligned}
\]

Native prompts, source, patches, logs, and archives remain payloads. The RDF graph carries their
digests, provenance, scope, standing, and relations. Therefore “RDF end to end” means no lifecycle
authority exists only in a chat transcript or process memory; it does not mean source code is
lossily translated into triples.

## 14.4 Reconnaissance is part of dogfooding

Read-only archaeology is operationally different from publication actuation, but it is not exempt
from workflow governance. Each research task is:

\[
q=(\operatorname{question},B,\operatorname{allowedTools},
\operatorname{timeBound},\operatorname{expectedEvidence}).
\]

An Explore agent may use file search, reads, Cargo metadata, Git history, and diagnostics. The
result is a proposal:

\[
\operatorname{Agent}(q)\to
\mathsf{Proposal}(O_q,C_q),
\]

where \(O_q\) is observation and \(C_q\) derived claims. Neither becomes authoritative until
admission. The task, invocation, tool events, results, and claims must bind into the run graph.

This corrects the earlier limited dogfood pattern in which reconnaissance occurred outside MFW and
only the deliverable contained PDDL/POWL.

## 14.5 Tool-event model

For each tool use, create a pre-event:

\[
e^{-}=(\operatorname{tool},\operatorname{operation},
\operatorname{argumentsDigest},\operatorname{task},
\operatorname{planStep},\operatorname{permission},
\operatorname{expectedEffect}).
\]

Create a post-event:

\[
e^{+}=(\operatorname{preEvent},\operatorname{status},
\operatorname{outputDigest},\operatorname{observedEffect},
\operatorname{duration},\operatorname{resourceUse}).
\]

Read operations can use standing reconnaissance permission. Edits, generated writes, installs,
builds with material side effects, commits, tags, pushes, and registry interactions require
plan-bound mutation permission.

The adapter covers:

- searches and reads;
- shell commands;
- file edits and generated output;
- tests and builds;
- agent or subagent launches;
- agent results;
- approval pauses;
- interruptions and bounds;
- patch application;
- verification;
- stop and terminal outcomes.

## 14.6 Claude Code is a governed actuator

Claude Code is invoked when a plan step requires cognitive work not supplied by a deterministic
adapter. Its invocation contract is

\[
\mathcal{C}=(g_c,O_c^{*},B_c,P_c,K_c,E_c,F_c),
\]

where:

- \(g_c\) is the child goal;
- \(O_c^{*}\) is the admitted context slice;
- \(B_c\) is the boundary;
- \(P_c\) is permission;
- \(K_c\) is a bound;
- \(E_c\) is required evidence; and
- \(F_c\) is the falsifier.

Claude Code may investigate, edit, implement, and test within this contract. It cannot:

- expand its own authority;
- admit its own claims;
- promote its own implementation;
- delete the parent goal;
- soften a required test to obtain green output;
- collapse a bound into exhaustion; or
- represent a proposed patch as an observed deployed consequence.

Its output is a candidate child result that the parent re-admits.

## 14.7 Planning when a workflow is unknown

Repository discovery produces facts such as:

\[
\begin{aligned}
&\operatorname{WorkspaceMember}(c),\\
&\operatorname{DependsOn}(c_i,c_j),\\
&\operatorname{GeneratedBy}(f,g),\\
&\operatorname{VerificationRecipe}(r),\\
&\operatorname{PublishRestriction}(c,p),\\
&\operatorname{MissingMetadata}(c,m),\\
&\operatorname{PathLeak}(a,\ell).
\end{aligned}
\]

The dry-run publish pack contains generic action schemas and refusal gates. Grounding them against
the discovered graph produces a repository-specific PDDL problem. The developer need not write a
script enumerating the repository's packages.

Discovery remains bounded. If the repository uses an unrecognized build system, encrypted
configuration, or inaccessible external service, the result is Unsupported or Refused with residue.

## 14.8 Permission presentation

Before mutation, MFW renders from RDF:

1. the requested outcome;
2. repository snapshot digest;
3. selected plan and process geometry;
4. exact files and artifact classes that may change;
5. commands and tools that may execute;
6. external services that may be contacted;
7. maximum time, cost, agent count, and recursion depth;
8. expected evidence;
9. explicit non-goals, including no registry upload; and
10. what events force a new permission request.

Approval creates a machine-evaluable policy bound to these identities. The prose is a projection,
not the sole permission artifact.

## 14.9 Recursive repair

Let a gate return refusal \(r\). The repair sequence is:

\[
\begin{aligned}
r
&\xrightarrow{\operatorname{admit}}
O_r^{*}\\
&\xrightarrow{\operatorname{continuationGoal}}
g_r\\
&\xrightarrow{\operatorname{plan}}
\Pi_r\\
&\xrightarrow{\operatorname{geometry}}
W_r\\
&\xrightarrow{\operatorname{ask}}
p_r\\
&\xrightarrow{\operatorname{ClaudeCode}}
\Delta_r\\
&\xrightarrow{\operatorname{verify}}
E_r\\
&\xrightarrow{\operatorname{graft}}
W[a\mapsto W_r].
\end{aligned}
\]

The parent gate is rerun on the modified repository. A repair is not complete merely because a unit
test for the patch passes; the same failed gate must be re-evaluated.

## 14.10 Lifecycle no-orphan verifier

The final verifier checks:

\[
\begin{aligned}
\forall q\in\operatorname{ResearchTask},&
\ \exists a,o,r:
\operatorname{assignedTo}(q,a)\land
\operatorname{generated}(a,o)\land
\operatorname{receipted}(q,r);\\
\forall e\in\operatorname{ToolEvent},&
\ \exists \pi,p,r:
\operatorname{plannedBy}(e,\pi)\land
\operatorname{authorizedBy}(e,p)\land
\operatorname{receiptedBy}(e,r);\\
\forall \Delta\in\operatorname{Patch},&
\ \exists q,E:
\operatorname{implements}(\Delta,q)\land
\operatorname{verifiedBy}(\Delta,E).
\end{aligned}
\]

The quantified relations are further constrained by run identity and digest. A link to a receipt
from another run does not satisfy the obligation.

## 14.11 Definition of done for Operation Dogfood

Operation Dogfood v26.7.13 is Alive only when one same-object run demonstrates all of the following:

1. user intent is admitted in RDF;
2. unfamiliar-repository reconnaissance is planned, bounded, and receipted;
3. every material observation and claim is admitted or explicitly refused;
4. a finite PDDL problem is generated from the graph;
5. the planner returns a truthful tagged outcome;
6. a found plan passes an independent verifier;
7. POWL geometry is generated with the required recursive sockets;
8. the exact plan and mutation surface are presented for permission;
9. execution cannot cross the mutation boundary without that permission;
10. at least one genuine discovered repair invokes Claude Code;
11. every Claude Code tool event and subtask has RDF lifecycle identity;
12. the patch is re-admitted and passes same-object verification;
13. real Cargo gates execute through the harness;
14. no registry publication occurs;
15. every effect has a pre-receipt and every completion a post-receipt;
16. the terminal result remains Found, Exhausted, Bounded, Unsupported, Inconsistent, or Refused
    without collapse;
17. the final graph contains zero lifecycle orphans;
18. native payload digests verify;
19. replay meets its declared byte or semantic criterion; and
20. independent negative fixtures prove that permission, orphan, digest, and outcome-collapse
    violations are refused.

Success of the product lifecycle does not require the repository's first dry-run publish to be
green. It requires the outcome to be real, typed, receipted, and replayable. A green publication
claim remains separate.

---

# Chapter 15. The Rust Dry-Run Publish Crown

## 15.1 Why dry-run publication is a strong first crown

Cargo's dry-run publication performs publication checks without uploading. It exercises metadata,
dependency resolution, packaging, included files, lock discipline, source portability, and build
verification while keeping external registry mutation outside scope.

The intended command family includes:

\[
\text{cargo package --locked}
\]

and

\[
\text{cargo publish --dry-run --locked}.
\]

The locked flag requires the existing lock file to remain sufficient; Cargo exits when it is absent
or dependency resolution would change it. The dry-run flag performs checks without upload.

## 15.2 Repository model

Let a Rust workspace be

\[
\mathcal{R}=(C,D,M,G,T,L,H),
\]

where:

- \(C\) is the finite crate set;
- \(D\subseteq C\times C\) is the workspace dependency relation;
- \(M\) maps crates to manifest metadata;
- \(G\) is generator configuration and generated-boundary data;
- \(T\) is the test and verification recipe set;
- \(L\) is license and legal metadata;
- \(H\) is repository and VCS identity.

For each crate \(c\), define a gate vector

\[
\Gamma(c)=(g_1(c),\ldots,g_k(c)),
\]

where each coordinate returns a typed result rather than Boolean success.

## 15.3 Gate system

### Gate 1 — Snapshot integrity

Capture commit identity, dirty state, submodules where admitted, toolchain versions, Cargo lock
digest, manifest digests, and generator inputs. The gate refuses if required identity is missing or
changes between plan and actuation without re-admission.

### Gate 2 — Generated-source consistency

Run the admitted synchronization or generation check. Let \(S\) be generated files before and
\(S'\) after deterministic regeneration. Drift is:

\[
\Delta_G=S\triangle S',
\]

the symmetric difference of path-content pairs. The gate passes only if \(\Delta_G=\varnothing\) or
the approved plan explicitly manufactures and verifies the required update.

### Gate 3 — Manifest and metadata admission

Check package name, version, edition, license expression or file, description where required,
repository metadata, include/exclude rules, publish policy, and dependency declarations. A
workspace path dependency intended for registry publication needs a publishable version relation;
an unversioned private path dependency is a typed blocker.

### Gate 4 — Dependency order

For publishable crates, compute a topological order of the dependency graph after collapsing or
refusing cycles. If edge \(c_i\to c_j\) means \(c_i\) depends on \(c_j\), then \(c_j\) precedes
\(c_i\). A cycle in registry-visible dependencies returns Inconsistent or Unsupported according to
the release policy.

### Gate 5 — Package construction

Execute Cargo package with locked dependency resolution for the exact crate. Preserve archive
digest, file list, command output, exit status, and environment identity.

### Gate 6 — Archive path and secret audit

Inspect packaged paths for absolute local paths, user-home leakage, missing required files, excluded
sources, secrets, and generated drift. A path such as a developer-local absolute directory is a
real portability and privacy blocker even if compilation on the source checkout succeeds.

### Gate 7 — Clean-room verification

Unpack the produced crate archive into a fresh admitted directory with no undeclared workspace
files. Build and test according to the package profile. The condition is:

\[
\operatorname{Build}(\operatorname{Unpack}(\operatorname{Archive}(c)))
=
\mathsf{Found}(\operatorname{VerifiedArtifact}),
\]

not merely that the original workspace builds.

### Gate 8 — Cargo publish dry run

Execute the exact no-upload dry-run command with locked resolution. Preserve Cargo's real exit
status and output. A command success is an observation that then crosses admission.

### Gate 9 — Receipt and replay

Seal all prior inputs and events. Replay graph transitions and deterministic artifacts. Refuse if
any package, command, repair, or result is orphaned.

## 15.4 Package-level and workspace-level outcomes

Let \(o_i(c)\) be the outcome of gate \(i\) for crate \(c\). Package success is:

\[
\operatorname{PackageReady}(c)
\Longleftrightarrow
\forall i,\ o_i(c)=\mathsf{Found}(w_i).
\]

Whole-workspace success for selected set \(C_P\) is:

\[
\operatorname{WorkspaceReady}(C_P)
\Longleftrightarrow
\forall c\in C_P,\ \operatorname{PackageReady}(c)
\land
\operatorname{DependencyOrderValid}(C_P).
\]

One publishable subset does not promote the whole workspace. The subject \(C_P\) is part of the
claim identity.

## 15.5 Current observed refusal

The release source record reports that the current whole-workspace dry-run is refused by:

1. seven unversioned in-workspace path dependencies;
2. package-license gaps;
3. a missing root license;
4. developer-local path leakage; and
5. a state in which only a subset of crates is near-term publishable.

This thesis did not rerun the external project workspace; it received these as provenance-bearing
project observations in the v26.7.13 requirements record. Therefore the standing is:

\[
\operatorname{WholeWorkspaceDryRun}=\mathsf{Refused},
\]

not Alive and not Unknown. Promotion requires a real rerun on the same declared crate set after
every blocker is closed or the scope is explicitly and newly admitted.

## 15.6 No registry mutation

The goal is dry-run publication. The prohibited action set includes real upload:

\[
\mathcal{P}_{\mathrm{forbidden}}
\supseteq
\{\operatorname{CargoPublishWithoutDryRun},
\operatorname{RegistryUpload},
\operatorname{TagPushUnlessSeparatelyApproved}\}.
\]

The broker rejects any command whose normalized operation falls in this set. No implied desire to
release authorizes actual publication.

## 15.7 A truthful terminal report

The terminal report is generated from RDF and includes:

- exact repository and crate-set identity;
- gate result per crate;
- dependency order;
- all refusals and residue;
- repairs attempted;
- Claude Code invocations;
- modified files;
- package archives and digests;
- dry-run command receipts;
- prohibited actions confirmed absent from broker events;
- replay verdict; and
- standing ceiling.

The report must be reproducible from the receipt graph. Hand-edited prose cannot upgrade it.

---

# Chapter 16. RDFTriple8 and Finite Semantic Admission

## 16.1 Motivation

Full RDF terms are variable-length objects. At constrained or high-frequency boundaries, a
profile-local compiled representation can turn selected semantic decisions into fixed-width table
operations. RDFTriple8 does this without claiming that all RDF fits into eight bits.

## 16.2 Profile-local symbol universe

Let a compiled profile be

\[
P=(U,\iota,\iota^{-1},h_P),
\]

where:

- \(U\subseteq\mathcal{T}\) is a finite selected RDF-term universe;
- \(|U|\le256\);
- \(\iota:U\to\{0,\ldots,255\}\) is injective;
- \(\iota^{-1}\) decodes assigned values; and
- \(h_P\) is the digest of the canonical profile symbol table and admission law.

Define

\[
\mathsf{Term8}=\{0,\ldots,255\}.
\]

For a triple \((s,p,o)\in U^3\), its Triple8 value is

\[
\operatorname{Triple8}_P(s,p,o)
=
(\iota(s),\iota(p),\iota(o)).
\]

If \(|U|>256\), compilation returns Triple8UniverseOverflow. If a runtime term is absent from \(U\),
encoding returns TermNotInTriple8Universe. Neither condition may wrap, truncate, or invent an index.

## 16.3 Packed key

Three 8-bit terms require 24 bits. A 32-bit cell can be written abstractly as

\[
k=(m\ll24)\lor(s_8\ll16)\lor(p_8\ll8)\lor o_8,
\]

where \(m\) is a profile-local metadata byte and \(\ll\) is left shift. The exact subdivision of
\(m\) between flags and a compact profile tag is an implementation ABI and must be generated and
receipted rather than guessed by consumers. The full profile digest \(h_P\), not the compact tag,
establishes profile identity.

Packing and unpacking are inverses for valid bytes:

\[
\operatorname{unpack}(\operatorname{pack}(m,s,p,o))=(m,s,p,o).
\]

This follows because the four bytes occupy disjoint bit positions and masks recover each position.

## 16.4 Byte equality theorem

### Theorem 16.1 — Profile-local equality

For terms \(x,y\in U\),

\[
\iota(x)=\iota(y)\Longleftrightarrow x=y.
\]

**Proof.** The reverse implication follows because a function maps equal inputs to equal outputs.
The forward implication is the injectivity premise of \(\iota\). \(\square\)

Therefore byte comparison is exact **inside the same profile**. Comparing bytes from different
profiles without checking \(h_P\) is invalid.

## 16.5 Admission8 table

Let

\[
\mathsf{Admission8}[0\ldots255]
\]

be a fixed array indexed by a compiled dispatch symbol, commonly a predicate or transition class.
Each entry contains required and forbidden bit masks plus a target transition or refusal code:

\[
A_i=(R_i,F_i,\delta_i,\rho_i).
\]

Let current state mask be \(M\). Admission succeeds at entry \(i\) when

\[
(M\mathbin{\&}R_i)=R_i
\]

and

\[
(M\mathbin{\&}F_i)=0,
\]

where \(\&\) is bitwise conjunction.

The first equation says every required bit is present. The second says no forbidden bit is present.

## 16.6 Constant dispatch bound

Index calculation, one array read, and a fixed number of bitwise operations do not depend on the
number of triples previously processed. Therefore dispatch cost is

\[
O(1)
\]

with respect to stream length and graph size for a fixed compiled profile and machine word width.
Profile compilation and term lookup have separate costs.

## 16.7 Correctness obligation

Let full admission predicate be

\[
\operatorname{Adm}_{\mathrm{RDF}}(G,t).
\]

Let compiled predicate be

\[
\operatorname{Adm}_{8}(P,M,\operatorname{encode}_P(t)).
\]

The required compiler theorem is

\[
\forall G,t\in\operatorname{ProfileDomain}(P),\
\operatorname{stateMask}_P(G)=M
\Rightarrow
\left(
\operatorname{Adm}_{\mathrm{RDF}}(G,t)
\Longleftrightarrow
\operatorname{Adm}_{8}(P,M,\operatorname{encode}_P(t))
\right).
\]

Without this theorem or exhaustive finite validation, Triple8 is a fast implementation candidate,
not a proved semantic substitute.

## 16.8 Receipted profile identity

Every Triple8 artifact or event records:

\[
(h_P,\operatorname{compilerVersion},
\operatorname{tableDigest},\operatorname{ABI},\operatorname{sourceGraphDigest}).
\]

A replay that lacks the same profile refuses rather than decoding bytes under a new symbol table.
This prevents silent semantic aliasing.

---


# Part VII. Geometry, Measure, and Multifractal Analysis

# Chapter 17. Metric, Topological, and Measure-Theoretic Foundations

## 17.1 Distance

A metric on a set \(X\) is a function

\[
d:X\times X\to\mathbb{R}_{\ge0}
\]

satisfying, for all \(x,y,z\in X\):

\[
d(x,y)=0\Longleftrightarrow x=y,
\]

\[
d(x,y)=d(y,x),
\]

and

\[
d(x,z)\le d(x,y)+d(y,z).
\]

These are identity, symmetry, and the triangle inequality. The pair \((X,d)\) is a metric space.

For a workflow event log, possible metrics include:

- elapsed-time distance \(d_t(e_i,e_j)=|t_i-t_j|\);
- graph distance, the length of a shortest causal path;
- tree distance, the number of edges from events to their lowest common ancestor;
- semantic distance induced by differences in admitted feature vectors; or
- a product metric combining normalized coordinates.

The metric must be declared before dimension is measured. Changing the metric can change the
geometry and spectrum.

## 17.2 Balls and neighborhoods

For center \(x\in X\) and radius \(r>0\), the open ball is

\[
B(x,r)=\{y\in X\mid d(x,y)<r\}.
\]

The closed ball replaces \(<r\) with \(\le r\).

A subset \(U\subseteq X\) is open when every \(x\in U\) has some radius \(r>0\) such that
\(B(x,r)\subseteq U\). The set of open subsets is a topology:

\[
\tau_d=\{U\subseteq X\mid U\text{ is open under }d\}.
\]

The topology captures nearness without retaining exact distances. A POWL poset has its own
Alexandrov topology; an event-time metric has a metric topology. They need not be identical.

## 17.3 Sequences and limits

A sequence in \(X\) is a function

\[
x:\mathbb{N}\to X,
\qquad n\mapsto x_n.
\]

It converges to \(x\in X\), written \(x_n\to x\), if:

\[
\forall\varepsilon>0,\
\exists N\in\mathbb{N},\
\forall n\ge N,\
d(x_n,x)<\varepsilon.
\]

The order of quantifiers is essential. After any requested tolerance \(\varepsilon\) is given, one
must find a point beyond which every sequence term stays within the tolerance.

A sequence is Cauchy if:

\[
\forall\varepsilon>0,\
\exists N,\
\forall m,n\ge N,\
d(x_m,x_n)<\varepsilon.
\]

A metric space is complete when every Cauchy sequence converges to a point in the space. Completeness
matters when iterating a design or analysis operator and claiming a limit exists.

## 17.4 Continuity

A function \(f:(X,d_X)\to(Y,d_Y)\) is continuous at \(x\) when:

\[
\forall\varepsilon>0,\
\exists\delta>0,\
d_X(x,y)<\delta
\Rightarrow
d_Y(f(x),f(y))<\varepsilon.
\]

It is continuous when continuous at every point. Continuity says arbitrarily small input changes
produce controllably small output changes. A package gate with a threshold can be discontinuous:
one-byte metadata change can switch refusal to acceptance. There is no requirement that all workflow
operators be continuous.

## 17.5 Sigma algebras

A sigma algebra \(\mathcal{F}\subseteq\mathcal{P}(X)\) is a collection of measurable sets satisfying:

1. \(X\in\mathcal{F}\);
2. \(A\in\mathcal{F}\Rightarrow X\setminus A\in\mathcal{F}\);
3. if \(A_1,A_2,\ldots\in\mathcal{F}\), then
   \(\bigcup_{i=1}^{\infty}A_i\in\mathcal{F}\).

Closure under complements and countable unions also gives closure under countable intersections by
De Morgan's law.

The Borel sigma algebra \(\mathcal{B}(X)\) is the smallest sigma algebra containing all open sets.
It provides the default measurable sets for a metric space.

## 17.6 Measures

A measure is a function

\[
\mu:\mathcal{F}\to[0,\infty]
\]

such that:

\[
\mu(\varnothing)=0
\]

and for pairwise disjoint \(A_1,A_2,\ldots\),

\[
\mu\left(\bigcup_{i=1}^{\infty}A_i\right)
=
\sum_{i=1}^{\infty}\mu(A_i).
\]

This is countable additivity. A probability measure additionally satisfies

\[
\mu(X)=1.
\]

A workflow mass measure might assign normalized effective work, elapsed time, evidence volume,
resource expenditure, or event count to measurable process regions. These are different measures
and must not be mixed without an explicit vector or scalarization.

## 17.7 Integration from simple functions

For a measurable nonnegative simple function

\[
\phi(x)=\sum_{i=1}^{n}a_i\mathbf{1}_{A_i}(x),
\]

where \(a_i\ge0\) and \(\mathbf{1}_{A_i}\) is one on \(A_i\) and zero outside, define

\[
\int_X\phi\,d\mu
=
\sum_{i=1}^{n}a_i\mu(A_i)
\]

for disjoint \(A_i\). For a general measurable nonnegative function \(f\),

\[
\int_X f\,d\mu
=
\sup\left\{
\int_X\phi\,d\mu
\;\middle|\;
0\le\phi\le f,\ \phi\text{ simple}
\right\}.
\]

An integrable signed function is decomposed into positive and negative parts
\(f=f^{+}-f^{-}\), and the integral is their difference when both are finite.

This construction makes later path-cost integrals precise.

## 17.8 Outer measure

An outer measure

\[
\mu^{*}:\mathcal{P}(X)\to[0,\infty]
\]

satisfies:

1. \(\mu^{*}(\varnothing)=0\);
2. \(A\subseteq B\Rightarrow\mu^{*}(A)\le\mu^{*}(B)\);
3. \(\mu^{*}(\bigcup_iA_i)\le\sum_i\mu^{*}(A_i)\).

Unlike a measure, an outer measure is defined on every subset but is only subadditive. Hausdorff
measure is built by minimizing cover costs and then restricting to measurable sets.

## 17.9 Diameter and covers

The diameter of \(U\subseteq X\) is

\[
\operatorname{diam}(U)
=
\sup\{d(x,y)\mid x,y\in U\}.
\]

A countable collection \(\{U_i\}_{i=1}^{\infty}\) covers \(E\subseteq X\) when

\[
E\subseteq\bigcup_{i=1}^{\infty}U_i.
\]

It is a \(\delta\)-cover when every \(\operatorname{diam}(U_i)\le\delta\).

## 17.10 Hausdorff measure

For exponent \(s\ge0\), define the \(\delta\)-scale Hausdorff content

\[
\mathcal{H}^{s}_{\delta}(E)
=
\inf\left\{
\sum_{i=1}^{\infty}
\bigl(\operatorname{diam}(U_i)\bigr)^s
\;\middle|\;
\{U_i\}\text{ is a }\delta\text{-cover of }E
\right\}.
\]

As \(\delta\) decreases, fewer covers are allowed, so the infimum cannot decrease. Define

\[
\mathcal{H}^{s}(E)
=
\lim_{\delta\downarrow0}\mathcal{H}^{s}_{\delta}(E)
=
\sup_{\delta>0}\mathcal{H}^{s}_{\delta}(E).
\]

Up to a conventional normalization constant, this is \(s\)-dimensional Hausdorff measure.

## 17.11 Hausdorff dimension

For a fixed set \(E\), \(\mathcal{H}^{s}(E)\) typically changes from infinity to zero at a critical
exponent. Define

\[
\dim_H(E)
=
\inf\{s\ge0\mid\mathcal{H}^{s}(E)=0\}
\]

which equals

\[
\sup\{s\ge0\mid\mathcal{H}^{s}(E)=\infty\}
\]

under the standard transition property.

For a finite set \(E\), \(\dim_H(E)=0\). Therefore a finite event log does not literally possess a
nonzero limiting Hausdorff dimension as a bare finite set. Empirical workflow analysis estimates a
scaling regime of an underlying process or growing family, not a theorem that one finite sample is a
fractal continuum.

## 17.12 Box-counting dimensions

Let \(N(E,\varepsilon)\) be the smallest number of sets of diameter at most \(\varepsilon\) required
to cover \(E\). Define lower and upper box dimensions:

\[
\underline{\dim}_{B}(E)
=
\liminf_{\varepsilon\downarrow0}
\frac{\log N(E,\varepsilon)}{-\log\varepsilon},
\]

\[
\overline{\dim}_{B}(E)
=
\limsup_{\varepsilon\downarrow0}
\frac{\log N(E,\varepsilon)}{-\log\varepsilon}.
\]

If the two agree, their common value is box-counting dimension. For finite empirical data,
regression over a declared intermediate scale interval estimates a slope; it is not the mathematical
limit.

## 17.13 Local dimension of a measure

For measure \(\mu\) and point \(x\), define lower and upper local dimensions:

\[
\underline{\alpha}_{\mu}(x)
=
\liminf_{r\downarrow0}
\frac{\log\mu(B(x,r))}{\log r},
\]

\[
\overline{\alpha}_{\mu}(x)
=
\limsup_{r\downarrow0}
\frac{\log\mu(B(x,r))}{\log r}.
\]

When they agree, the local dimension is

\[
\alpha_{\mu}(x)
=
\lim_{r\downarrow0}
\frac{\log\mu(B(x,r))}{\log r}.
\]

The ratio expresses the scaling approximation

\[
\mu(B(x,r))\asymp r^{\alpha_{\mu}(x)},
\]

because taking logarithms yields

\[
\log\mu(B(x,r))
\approx
\alpha_{\mu}(x)\log r.
\]

Different points may have different \(\alpha\). That variation is the foundation of a multifractal
spectrum.

## 17.14 Level sets

For exponent \(\alpha\), define

\[
E_{\alpha}
=
\{x\in X\mid\alpha_{\mu}(x)=\alpha\}.
\]

The geometric multifractal spectrum is

\[
f(\alpha)=\dim_H(E_{\alpha}).
\]

This definition is exact when local dimensions and Hausdorff dimensions exist. Estimating it from
finite data requires the indirect methods in Chapters 18 and 19.

---

# Chapter 18. Multifractal Formalism from First Principles

## 18.1 Partitioning at scale

Let \(\mu\) be a probability measure on bounded \(X\). At resolution \(\varepsilon>0\), choose a
finite measurable partition or cover

\[
\mathcal{P}_{\varepsilon}
=
\{B_1(\varepsilon),\ldots,B_{N(\varepsilon)}(\varepsilon)\}.
\]

Define box masses

\[
p_i(\varepsilon)=\mu(B_i(\varepsilon)).
\]

For a partition of the full support,

\[
p_i(\varepsilon)\ge0,
\qquad
\sum_{i=1}^{N(\varepsilon)}p_i(\varepsilon)=1.
\]

Empty boxes are normally omitted when negative moments are used, because \(0^q\) diverges for
\(q<0\).

## 18.2 Partition function

For real moment order \(q\), define

\[
Z(q,\varepsilon)
=
\sum_{i:p_i(\varepsilon)>0}
p_i(\varepsilon)^q.
\]

Interpretation:

- \(q>1\) amplifies high-mass regions;
- \(q=1\) gives \(Z(1,\varepsilon)=1\);
- \(q=0\) counts nonempty regions because \(p_i^0=1\);
- \(q<0\) amplifies low-mass regions and is statistically fragile.

Suppose a scaling law holds:

\[
Z(q,\varepsilon)\asymp\varepsilon^{\tau(q)}
\quad\text{as }\varepsilon\downarrow0.
\]

Taking logarithms:

\[
\log Z(q,\varepsilon)
\approx
\tau(q)\log\varepsilon.
\]

Therefore the mass exponent is

\[
\tau(q)
=
\lim_{\varepsilon\downarrow0}
\frac{\log Z(q,\varepsilon)}{\log\varepsilon},
\]

when the limit exists.

## 18.3 Generalized dimensions

For \(q\neq1\), define the Rényi generalized dimension

\[
D_q=\frac{\tau(q)}{q-1}.
\]

Equivalently,

\[
D_q
=
\frac{1}{q-1}
\lim_{\varepsilon\downarrow0}
\frac{\log\sum_i p_i(\varepsilon)^q}{\log\varepsilon}.
\]

### The case \(q=0\)

Because \(Z(0,\varepsilon)=N(\varepsilon)\),

\[
D_0
=
\lim_{\varepsilon\downarrow0}
\frac{\log N(\varepsilon)}{-\log\varepsilon},
\]

when the limit exists. This is box-counting dimension of the measure's support.

### The case \(q=1\)

Direct substitution yields \(0/0\), so take a limit. Differentiate

\[
\log Z(q,\varepsilon)
\]

with respect to \(q\):

\[
\frac{\partial}{\partial q}\log Z(q,\varepsilon)
=
\frac{\sum_i p_i(\varepsilon)^q\log p_i(\varepsilon)}
{\sum_i p_i(\varepsilon)^q}.
\]

At \(q=1\), the denominator is one:

\[
\left.
\frac{\partial}{\partial q}\log Z(q,\varepsilon)
\right|_{q=1}
=
\sum_i p_i(\varepsilon)\log p_i(\varepsilon).
\]

Thus the information dimension is

\[
D_1
=
\lim_{\varepsilon\downarrow0}
\frac{\sum_i p_i(\varepsilon)\log p_i(\varepsilon)}
{\log\varepsilon}.
\]

Both numerator and denominator are nonpositive, so the ratio is nonnegative.

### The case \(q=2\)

\[
D_2
=
\lim_{\varepsilon\downarrow0}
\frac{\log\sum_i p_i(\varepsilon)^2}{\log\varepsilon}.
\]

This is the correlation dimension under suitable partitions and regularity.

## 18.4 Uniform measure sanity check

Suppose \(N(\varepsilon)\asymp\varepsilon^{-d}\) boxes have equal mass

\[
p_i(\varepsilon)=\frac{1}{N(\varepsilon)}.
\]

Then

\[
\begin{aligned}
Z(q,\varepsilon)
&=
N(\varepsilon)
\left(\frac{1}{N(\varepsilon)}\right)^q\\
&=
N(\varepsilon)^{1-q}\\
&\asymp
(\varepsilon^{-d})^{1-q}\\
&=
\varepsilon^{d(q-1)}.
\end{aligned}
\]

Therefore

\[
\tau(q)=d(q-1)
\]

and

\[
D_q=d
\]

for every \(q\). A monofractal has constant generalized dimension. A nonconstant \(D_q\) is evidence
of nonuniform scale concentration, subject to estimation validity.

## 18.5 Singularity strength

When \(\tau\) is differentiable, define

\[
\alpha(q)=\frac{d\tau}{dq}(q).
\]

The derivative is the limit

\[
\tau'(q)
=
\lim_{h\to0}
\frac{\tau(q+h)-\tau(q)}{h}.
\]

It measures how the mass-scaling exponent changes as the moment order shifts emphasis between dense
and sparse regions.

## 18.6 Legendre transform

Define the Legendre-Fenchel spectrum estimate

\[
f_L(\alpha)
=
\inf_{q\in\mathbb{R}}
\bigl(q\alpha-\tau(q)\bigr).
\]

When \(\tau\) is differentiable and regularity conditions hold, the minimizing \(q\) satisfies

\[
\alpha=\tau'(q),
\]

and then

\[
f_L(\alpha(q))
=
q\alpha(q)-\tau(q).
\]

Derivation: consider

\[
\phi(q)=q\alpha-\tau(q).
\]

At an interior minimum,

\[
\phi'(q)=\alpha-\tau'(q)=0,
\]

so \(\alpha=\tau'(q)\). Substituting yields the parametric spectrum.

The Legendre spectrum need not equal the geometric spectrum
\(f(\alpha)=\dim_H(E_{\alpha})\) for every measure. Equality is the **multifractal formalism**, a
theorem only under additional hypotheses and otherwise a conjectural or empirical approximation.

## 18.7 Binomial cascade example

Let \(0<p<1\). Begin with interval \([0,1]\) of mass one. At each level, split every interval into
left and right halves, assigning fractions \(p\) and \(1-p\). After \(n\) levels, scale is

\[
\varepsilon=2^{-n}.
\]

A cell with \(k\) left choices and \(n-k\) right choices has mass

\[
p^k(1-p)^{n-k},
\]

and there are

\[
\binom{n}{k}
\]

such cells. The partition function is

\[
\begin{aligned}
Z(q,2^{-n})
&=
\sum_{k=0}^{n}
\binom{n}{k}
\left(p^k(1-p)^{n-k}\right)^q\\
&=
\sum_{k=0}^{n}
\binom{n}{k}
p^{qk}(1-p)^{q(n-k)}\\
&=
\left(p^q+(1-p)^q\right)^n
\end{aligned}
\]

by the binomial theorem.

Because \(\varepsilon=2^{-n}\),

\[
n=-\frac{\log\varepsilon}{\log2}.
\]

Therefore

\[
\begin{aligned}
\log Z(q,\varepsilon)
&=
n\log\left(p^q+(1-p)^q\right)\\
&=
-\frac{\log\varepsilon}{\log2}
\log\left(p^q+(1-p)^q\right).
\end{aligned}
\]

Dividing by \(\log\varepsilon\) gives

\[
\tau(q)
=
-\frac{\log\left(p^q+(1-p)^q\right)}{\log2}.
\]

If \(p=1/2\), this reduces to \(\tau(q)=q-1\), the uniform one-dimensional case. If
\(p\neq1/2\), \(D_q\) varies with \(q\), producing a genuine multifractal measure.

## 18.8 Local exponent for the binomial cascade

For a path whose asymptotic fraction of left choices is \(\theta\), mass after \(n\) levels is
approximately

\[
\mu_n\approx p^{\theta n}(1-p)^{(1-\theta)n}.
\]

Since radius scale is \(2^{-n}\),

\[
\begin{aligned}
\alpha(\theta)
&=
\lim_{n\to\infty}
\frac{\log\mu_n}{\log2^{-n}}\\
&=
\frac{\theta\log p+(1-\theta)\log(1-p)}
-\log2}.
\end{aligned}
\]

The number of paths with fraction \(\theta\) grows according to binomial entropy, yielding a spectrum
whose exact derivation can be expressed through large deviations. The example demonstrates why one
global dimension is insufficient: different paths concentrate mass at different rates.

## 18.9 Vector-valued measures

Workflow mass is not inherently scalar. Define a vector measure

\[
\boldsymbol{\mu}(A)
=
(\mu_{\mathrm{time}}(A),
\mu_{\mathrm{compute}}(A),
\mu_{\mathrm{energy}}(A),
\mu_{\mathrm{evidence}}(A),
\mu_{\mathrm{risk}}(A)).
\]

Each coordinate is measured independently with units. A scalarization uses explicit nonnegative
weights:

\[
\mu_w(A)=w^{\top}\boldsymbol{\mu}(A)
=
\sum_{j=1}^{m}w_j\mu_j(A).
\]

Changing \(w\) changes the measure and can change the spectrum. Therefore results must report
coordinate spectra or the exact scalarization. Unit normalization precedes weighted addition;
seconds cannot be added directly to joules without a declared conversion or utility.

## 18.10 Multifractal claims and their falsifiers

An empirical claim that workflow measure \(\mu\) is multifractal requires at least:

1. a declared metric and measure;
2. enough scale range to fit more than a trivial line;
3. stable \(\tau(q)\) estimates across resamples;
4. nonconstant \(D_q\) beyond uncertainty;
5. comparison with shuffled and surrogate data;
6. robustness to partition choice and \(q\)-range;
7. treatment of finite-size and zero-mass effects; and
8. out-of-sample reproduction.

The claim is falsified or weakened when:

- the apparent spectrum collapses under longer data;
- surrogates with no cross-scale dependence produce the same width;
- estimates depend primarily on one outlier or empty box;
- no stable scaling interval exists;
- confidence intervals include a constant \(D_q\); or
- changing an arbitrary partition choice reverses the result.

---

# Chapter 19. Workflow Measurement, MF-DFA, and Branching Random Walks

## 19.1 Two independent rails

The **generative rail** manufactures workflows:

\[
O^{*}\to\Pi\to W\to A\to R.
\]

The **measurement rail** analyzes observed events:

\[
R\to\mathsf{EventLog}\to\mathsf{ScaleStatistics}\to\mathsf{ModelAssessment}.
\]

The generative rail does not need a multifractal hypothesis to execute. The measurement rail cannot
prove the generative architecture correct. Keeping the rails separate prevents circular evidence.

## 19.2 Workflow observation space

Let an event log be

\[
E_N=(e_1,\ldots,e_N),
\]

ordered by an admitted event index or causal-time rule. Map each event to an observable vector:

\[
x_i=\phi(e_i)\in\mathbb{R}^{m}.
\]

Examples include elapsed duration, CPU time, bytes moved, evidence size, repair depth, fan-out,
rework, or capability gain. The mapping \(\phi\) is versioned.

For scalar analysis, select one coordinate or admitted scalarization:

\[
y_i=w^{\top}x_i.
\]

## 19.3 Scale choices for workflows

Possible scales include:

- wall-clock windows of length \(s\);
- causal depth from a root workflow;
- recursive graft depth;
- number of events per block;
- architecture hierarchy level;
- semantic radius in a feature metric; or
- graph neighborhoods around a goal.

These scales answer different questions. A spectrum over elapsed time cannot be interpreted as a
spectrum over causal depth without an empirical relation between the two.

## 19.4 Multifractal detrended fluctuation analysis

MF-DFA analyzes scale-dependent fluctuations in a possibly nonstationary time series. Begin with
scalar series \(y_1,\ldots,y_N\).

### Step 1 — Mean

\[
\bar y
=
\frac{1}{N}\sum_{i=1}^{N}y_i.
\]

### Step 2 — Integrated profile

\[
Y(k)
=
\sum_{i=1}^{k}(y_i-\bar y),
\qquad 1\le k\le N.
\]

Subtracting the mean prevents a constant offset from dominating the cumulative profile.

### Step 3 — Segmentation

Choose window size \(s\). Let

\[
N_s=\left\lfloor\frac{N}{s}\right\rfloor.
\]

Partition from the beginning into \(N_s\) nonoverlapping windows. Because a remainder may exist,
repeat from the end, producing \(2N_s\) windows.

### Step 4 — Local polynomial trend

For segment \(\nu\), fit a polynomial \(P_{\nu,m}(j)\) of degree \(m\) to the profile values by least
squares. That is, choose coefficients \(a_0,\ldots,a_m\) minimizing

\[
\sum_{j=1}^{s}
\left(
Y(k_{\nu,j})-\sum_{\ell=0}^{m}a_{\ell}j^{\ell}
\right)^2.
\]

The fitted polynomial is

\[
P_{\nu,m}(j)=\sum_{\ell=0}^{m}a_{\ell}j^{\ell}.
\]

### Step 5 — Detrended variance

\[
F^2(\nu,s)
=
\frac{1}{s}
\sum_{j=1}^{s}
\left(
Y(k_{\nu,j})-P_{\nu,m}(j)
\right)^2.
\]

This is nonnegative because it averages squares.

### Step 6 — Moment aggregation for \(q\neq0\)

\[
F_q(s)
=
\left[
\frac{1}{2N_s}
\sum_{\nu=1}^{2N_s}
\left(F^2(\nu,s)\right)^{q/2}
\right]^{1/q}.
\]

Positive \(q\) emphasizes large fluctuations; negative \(q\) emphasizes small fluctuations.

### Step 7 — The \(q=0\) limit

The expression above is singular at \(q=0\). Its continuous limit is the geometric mean:

\[
F_0(s)
=
\exp\left[
\frac{1}{4N_s}
\sum_{\nu=1}^{2N_s}
\log F^2(\nu,s)
\right].
\]

Zero variances require an explicit floor, omission policy, or refusal because the logarithm is
undefined at zero.

### Step 8 — Scaling exponent

If an interval \(s\in[s_{\min},s_{\max}]\) satisfies

\[
F_q(s)\asymp s^{h(q)},
\]

then

\[
\log F_q(s)
\approx
h(q)\log s+c_q.
\]

Estimate \(h(q)\) as the regression slope over a preregistered or cross-validated scale interval.

### Step 9 — Mass exponent

For a one-dimensional time series under the standard MF-DFA relation,

\[
\tau(q)=qh(q)-1.
\]

Then

\[
\alpha(q)=\frac{d\tau}{dq}
=
h(q)+q h'(q),
\]

and

\[
f(\alpha(q))
=
q\alpha(q)-\tau(q)
=
q[\alpha(q)-h(q)]+1.
\]

Numerical differentiation amplifies noise, so uncertainty must propagate through \(h'(q)\).

## 19.5 Regression from first principles

For points

\[
(u_j,v_j)=(\log s_j,\log F_q(s_j)),
\]

the ordinary least-squares slope is

\[
\widehat h(q)
=
\frac{\sum_j(u_j-\bar u)(v_j-\bar v)}
{\sum_j(u_j-\bar u)^2},
\]

where

\[
\bar u=\frac{1}{n}\sum_j u_j,
\qquad
\bar v=\frac{1}{n}\sum_j v_j.
\]

The residuals are

\[
r_j=v_j-\widehat c-\widehat h(q)u_j.
\]

Linearity diagnostics, residual structure, scale-range sensitivity, and bootstrap intervals are
part of the evidence. A high coefficient of determination over three points is not sufficient.

## 19.6 Surrogate tests

Shuffle the original series to destroy temporal correlation while preserving the marginal value
distribution. Compare:

\[
\Delta h_{\mathrm{corr}}(q)
=
h_{\mathrm{original}}(q)-h_{\mathrm{shuffled}}(q).
\]

If broad multifractality remains after shuffling, it may arise from a broad distribution rather than
cross-scale correlation. Phase-randomized or model-based surrogates test other null hypotheses.

The measurement conclusion should therefore distinguish:

- distribution-driven multifractality;
- correlation-driven multifractality;
- finite-size artifact; and
- unresolved origin.

## 19.7 Workflow tree

Recursive manufacture creates a rooted tree or directed acyclic derivation graph. For a pure tree
\(T\), let root be the campaign and children be grafted workflows. Depth of vertex \(v\) is the
number of parent edges from root:

\[
\operatorname{depth}(v)\in\mathbb{N}.
\]

Let \(Z_n\) be the number of workflows at depth \(n\). A Galton-Watson model assumes each node has an
independent identically distributed child count \(\xi\):

\[
Z_{n+1}
=
\sum_{i=1}^{Z_n}\xi_{n,i}.
\]

Its mean reproduction number is

\[
m=\mathbb{E}[\xi].
\]

Classical cases are subcritical \(m<1\), critical \(m=1\), and supercritical \(m>1\). Real MFW
growth is policy-bounded, context-dependent, and not generally independent or identically
distributed. The Galton-Watson process is a null or candidate generative model, not the workflow
semantics.

## 19.8 Branching random walk

Assign each edge \(e\) an increment \(X_e\), such as log cost, log mass ratio, or capability change.
For vertex \(v\), define position

\[
S_v=\sum_{e\in\operatorname{path}(\operatorname{root},v)}X_e.
\]

A branching random walk combines random branching with additive positions. Define a level-\(n\)
partition function

\[
Z_n(q)=\sum_{|v|=n}e^{-qS_v}.
\]

If

\[
\psi(q)=\log\mathbb{E}\left[\sum_{|v|=1}e^{-qS_v}\right]
\]

is finite, a normalized additive process is

\[
W_n(q)=e^{-n\psi(q)}Z_n(q).
\]

Under appropriate independence and integrability hypotheses, \(W_n(q)\) may form a martingale and
converge. Those hypotheses must be tested or justified before importing convergence theorems into
workflow data.

## 19.9 A deterministic workflow cascade

The binomial cascade in Chapter 18 suggests a deterministic model. Suppose every repair splits
remaining work mass into fractions \(p\) and \(1-p\) across two child obligations. Repeating the
split creates a known multifractal measure. MFW does not assume actual work behaves this way. The
model is useful for:

- validating the analysis code against a known spectrum;
- testing sensitivity to finite depth;
- calibrating negative-\(q\) behavior; and
- proving that the measurement pipeline can distinguish uniform and nonuniform cascades.

## 19.10 Empirical workflow hypothesis

The substantive empirical hypothesis is:

> For selected workflow measures and scale definitions, recursive institutional work exhibits
> stable nonuniform scaling not explained solely by finite sample size, marginal distribution, or
> arbitrary hierarchy construction.

This is a CONJECTURE until the complete protocol, data, uncertainty, surrogates, and replication
support it. The name Multifractal Workflow does not count as evidence.

## 19.11 Process mining and feedback

The measurement rail may discover:

- recurring repair sockets;
- scale-specific bottlenecks;
- unusually dense evidence regions;
- sparse but high-cost tail behavior;
- topology whose observed concurrency differs from modeled concurrency; or
- branch distributions inconsistent with planner assumptions.

These findings enter RDF as observations, not automatic law. Admission may turn a stable finding
into a new constraint, cost model, SPARQL CONSTRUCT, or architecture-search goal. Thus measurement
feeds manufacture without being allowed to rewrite authority silently.

---


# Part VIII. Calculus of Capability and Design

# Chapter 20. Discrete Calculus and Process Thermodynamics

## 20.1 Scope of the thermodynamic language

MFW uses “thermodynamics” as a disciplined process model unless a quantity is measured in physical
thermodynamic units. Terms such as work, potential, gradient, dissipation, and free energy can be
mathematically defined over workflow state. They do not thereby become laws of heat, entropy, or
statistical mechanics.

Every equation in this chapter is one of:

1. a mathematical definition;
2. a constrained optimization model;
3. an empirical quantity computed from receipts; or
4. a conjectured relation with an explicit falsifier.

No metaphor is permitted to borrow the authority of physics without a unit-preserving bridge.

## 20.2 Scalar functions and ordinary derivatives

Let

\[
f:\mathbb{R}\to\mathbb{R}.
\]

The derivative of \(f\) at \(x\) is

\[
f'(x)
=
\lim_{h\to0}
\frac{f(x+h)-f(x)}{h},
\]

when the limit exists. The numerator is change in output, the denominator change in input, and the
limit asks for the instantaneous ratio as the input interval shrinks.

For finite workflow observations there is usually no infinitesimal interval. The finite difference
is

\[
\Delta_h f(x)=f(x+h)-f(x),
\]

and the difference quotient is

\[
\frac{\Delta_h f(x)}{h}.
\]

Calling this a derivative is an approximation whose step \(h\) must be reported.

## 20.3 Multivariable state

Let admitted process state be embedded in

\[
x=(x_1,\ldots,x_n)\in\mathbb{R}^{n}.
\]

Coordinates might include verified capabilities, cost, elapsed time, energy, risk, evidence
coverage, open residue, or queue depth after unit normalization.

For

\[
C:\mathbb{R}^{n}\to\mathbb{R},
\]

the partial derivative with respect to coordinate \(i\) is

\[
\frac{\partial C}{\partial x_i}(x)
=
\lim_{h\to0}
\frac{
C(x_1,\ldots,x_i+h,\ldots,x_n)-C(x)
}{h}.
\]

The gradient is the vector

\[
\nabla C(x)
=
\left(
\frac{\partial C}{\partial x_1}(x),
\ldots,
\frac{\partial C}{\partial x_n}(x)
\right).
\]

For a small displacement \(\Delta x\), differentiability gives the first-order approximation

\[
C(x+\Delta x)
\approx
C(x)+\nabla C(x)^{\top}\Delta x.
\]

The approximation error is smaller than first order as \(\|\Delta x\|\to0\).

## 20.4 Directional derivative

For direction \(v\in\mathbb{R}^{n}\), the directional derivative is

\[
D_vC(x)
=
\lim_{h\to0}
\frac{C(x+hv)-C(x)}{h}.
\]

When \(C\) is differentiable,

\[
D_vC(x)=\nabla C(x)^{\top}v.
\]

By the Cauchy-Schwarz inequality,

\[
\nabla C(x)^{\top}v
\le
\|\nabla C(x)\|\,\|v\|.
\]

Among unit directions \(\|v\|=1\), the largest first-order increase occurs in the gradient direction
\(v=\nabla C/\|\nabla C\|\), when the gradient is nonzero. This is a local statement. Constraints,
discrete choices, and model error can make the gradient-inferred move unlawful or globally poor.

## 20.5 Hessian and curvature

If second partial derivatives exist, the Hessian is

\[
H_C(x)
=
\left[
\frac{\partial^2 C}{\partial x_i\partial x_j}(x)
\right]_{i,j=1}^{n}.
\]

The second-order approximation is

\[
C(x+\Delta x)
\approx
C(x)
+\nabla C(x)^{\top}\Delta x
+\frac{1}{2}\Delta x^{\top}H_C(x)\Delta x.
\]

Positive curvature can indicate increasing marginal gain in a direction; negative curvature can
indicate diminishing gain. In process data, estimating a Hessian requires substantially more
evidence than estimating a slope.

## 20.6 Capability, resources, and lawful gain

Define a capability functional

\[
C_B(x)\in\mathbb{R}_{\ge0}
\]

inside boundary \(B\). It must be decomposed into auditable coordinates rather than treated as an
unobservable feeling. For a transition \(x\to x'\), capability gain is

\[
\Delta C=C_B(x')-C_B(x).
\]

Let resource vector be

\[
r=(r_{\mathrm{matter}},r_{\mathrm{energy}},r_{\mathrm{time}},
r_{\mathrm{compute}},r_{\mathrm{interpretation}})
\in\mathbb{R}_{\ge0}^{5}.
\]

Choose admitted positive unit-conversion or utility weights \(w_j>0\). Scalar resource cost is

\[
c_w(r)=w^{\top}r.
\]

The weights are policy, not mathematical constants.

## 20.7 Effective process work

Define **effective process work** for transition \(x\to x'\) as verified positive capability gain:

\[
W_{\mathrm{eff}}(x\to x')
=
\max\{0,C_B(x')-C_B(x)\}
\cdot
\mathbf{1}_{\mathrm{lawful}}
\cdot
\mathbf{1}_{\mathrm{receipted}}
\cdot
\mathbf{1}_{\mathrm{replayAccepted}}.
\]

The indicator \(\mathbf{1}_P\) equals one when proposition \(P\) is true and zero otherwise. This
definition assigns zero standing work to an unreceipted or unlawful apparent improvement. It is a
capability-accounting convention, not mechanical work in joules.

## 20.8 Leverage

For positive scalar resource cost, define leverage

\[
\Lambda(x\to x')
=
\frac{W_{\mathrm{eff}}(x\to x')}{c_w(r)}.
\]

If cost is zero, the ratio is undefined and the measurement system must inspect whether resources
were omitted. “Infinite leverage” is not the default.

A trimtab candidate is a lawful intervention with small support and high leverage:

\[
t^{*}\in
\operatorname*{arg\,max}_{t\in\mathcal{T}_{\mathrm{lawful}}}
\frac{\Delta C(t)}{c_w(t)}
\]

subject to:

\[
\operatorname{Support}(t)\le k,
\qquad
\operatorname{Risk}(t)\le\rho,
\qquad
\operatorname{Evidence}(t)\ge\eta.
\]

This converts “small change, large redirection” into a constrained optimization problem.

## 20.9 Ephemeralization

At state \(x\), define reproducibly available lawful consequence set under resource budget \(b\):

\[
\mathcal{K}_B(x,b)
=
\{a\mid
\exists\text{ admitted, permissionable, receiptable workflow from }x
\text{ to }a
\text{ with cost}\le b
\}.
\]

One capacity measure is weighted cardinality

\[
K_B(x,b)
=
\sum_{a\in\mathcal{K}_B(x,b)}v(a),
\]

with admitted value weights \(v(a)\ge0\). Define ephemeralization efficiency

\[
\mathcal{E}_B(x,b)
=
\frac{K_B(x,b)}{b}
\]

for \(b>0\), or use the full vector frontier rather than scalar budget. Progress through
ephemeralization is:

\[
\Delta\mathcal{E}_B
=
\mathcal{E}_B(x',b)-\mathcal{E}_B(x,b).
\]

This quantity stays inside the controlled system. It does not depend on market adoption.

## 20.10 Synergetic surplus

Let components be \(S_1,\ldots,S_n\). Let \(C(\{S_i\})\) measure each component in an admitted
isolation environment, and \(C(\{S_1,\ldots,S_n\})\) the configured whole. Define synergy:

\[
\operatorname{Syn}(S_1,\ldots,S_n)
=
C(\{S_1,\ldots,S_n\})
-
\sum_{i=1}^{n}C(\{S_i\}).
\]

Positive value means the configured whole provides capability not equal to the sum of isolated
capabilities under this measurement. Negative value identifies interference. Because component
capabilities can overlap, more sophisticated inclusion-exclusion or cooperative-game allocations
may be required; the simple equation is a definition under the stated isolation protocol.

## 20.11 Discrete capability gradient on a graph

Architecture options often form a finite graph rather than \(\mathbb{R}^{n}\). Let

\[
G_A=(V_A,E_A)
\]

and capability \(C:V_A\to\mathbb{R}\). For directed edge \(e=(u,v)\), define

\[
\nabla_e C=C(v)-C(u).
\]

Resource-normalized edge gradient is

\[
g_e=\frac{C(v)-C(u)}{\operatorname{cost}(e)}
\]

for positive cost. Selecting the largest lawful \(g_e\) is discrete steepest ascent. It can be
myopic; a lower immediate gain may unlock a higher future region. PDDL and architecture search
handle the multi-step problem.

## 20.12 Paths and accumulated cost

For a continuous path \(x:[0,T]\to\mathbb{R}^{n}\) with velocity

\[
\dot x(t)=\frac{dx}{dt},
\]

define path cost

\[
J[x]
=
\int_{0}^{T}
L(x(t),\dot x(t),t)\,dt,
\]

where \(L\) is a process cost density. The notation is meaningful only after units and measurable
coordinates are declared.

For a discrete workflow path \(x_0,\ldots,x_N\),

\[
J_{\mathrm{disc}}
=
\sum_{i=0}^{N-1}
L_i(x_i,x_{i+1}).
\]

This is the operational form used by bounded planning.

## 20.13 First variation and Euler-Lagrange equation

To derive a stationary continuous path, perturb \(x\) by a differentiable function \(\eta\) with
\(\eta(0)=\eta(T)=0\):

\[
x_{\varepsilon}(t)=x(t)+\varepsilon\eta(t).
\]

Define

\[
\Phi(\varepsilon)=J[x_{\varepsilon}].
\]

A stationary path satisfies

\[
\Phi'(0)=0
\]

for every allowed \(\eta\). Differentiate under the integral:

\[
\Phi'(0)
=
\int_0^T
\left[
\frac{\partial L}{\partial x}\cdot\eta
+
\frac{\partial L}{\partial\dot x}\cdot\dot\eta
\right]dt.
\]

Integrate the second term by parts:

\[
\int_0^T
\frac{\partial L}{\partial\dot x}\cdot\dot\eta\,dt
=
\left[
\frac{\partial L}{\partial\dot x}\cdot\eta
\right]_0^T
-
\int_0^T
\frac{d}{dt}
\left(
\frac{\partial L}{\partial\dot x}
\right)\cdot\eta\,dt.
\]

The boundary term is zero because \(\eta(0)=\eta(T)=0\). Therefore

\[
\Phi'(0)
=
\int_0^T
\left[
\frac{\partial L}{\partial x}
-
\frac{d}{dt}
\left(
\frac{\partial L}{\partial\dot x}
\right)
\right]\cdot\eta\,dt.
\]

For this integral to be zero for every admissible \(\eta\), the bracket must vanish:

\[
\boxed{
\frac{d}{dt}
\left(
\frac{\partial L}{\partial\dot x}
\right)
-
\frac{\partial L}{\partial x}
=0
}.
\]

This is the Euler-Lagrange equation. MFW does not claim institutional workflows follow smooth
Euler-Lagrange dynamics. The derivation supplies a principled model when continuous roadmap
relaxations are used to generate candidates, which must later be discretized and verified.

## 20.14 Constrained optimization

Let objective \(C(x)\) be maximized subject to equality \(g_i(x)=0\) and inequality
\(h_j(x)\le0\). A Lagrangian is

\[
\mathcal{L}(x,\lambda,\nu)
=
-C(x)
+
\sum_i\lambda_i g_i(x)
+
\sum_j\nu_jh_j(x),
\]

where \(\nu_j\ge0\). Necessary Karush-Kuhn-Tucker conditions under regularity include:

\[
\nabla_x\mathcal{L}=0,
\]

\[
g_i(x)=0,
\qquad
h_j(x)\le0,
\qquad
\nu_j\ge0,
\qquad
\nu_jh_j(x)=0.
\]

These are candidate optimality conditions, not sufficient in every nonconvex problem. MFW can use
continuous solutions to propose regions; exact finite constraints and plan verification remain the
admission boundary.

## 20.15 Capability potential and residue

Define desired capability \(C^{*}\) and current \(C(x)\). Residual potential is

\[
V(x)=\max\{0,C^{*}-C(x)\}.
\]

A repair step is progress when

\[
V(x')<V(x)
\]

while preserving constraints. A descent meter may be independent of \(V\); it guarantees
termination even when an attempted repair fails to reduce potential.

## 20.16 Dissipation as irrecoverable resource

Define measured resource input \(R_{\mathrm{in}}\) and resource recoverable in resulting capability
\(R_{\mathrm{cap}}\) under an admitted conversion. Process dissipation is

\[
D_{\mathrm{proc}}
=
R_{\mathrm{in}}-R_{\mathrm{cap}}.
\]

This quantity is model-dependent. Rework, discarded search, duplicated interpretation, and failed
unreceipted effects can increase it. A negative value indicates inconsistent accounting or a system
receiving unmodeled external resource.

## 20.17 Falsification of the capability calculus

The calculus is useful only if its quantities predict or discriminate real consequences. It should
be revised when:

- estimated high-leverage interventions repeatedly fail under same-object execution;
- capability weights reverse conclusions under small arbitrary perturbations;
- resource accounting omits dominant inputs;
- synergy disappears under controlled isolation;
- continuous gradients point outside the lawful discrete design space; or
- ephemeralization increases only because evidence obligations were weakened.

---

# Chapter 21. Design for Combinatorial Maximalism

## 21.1 The maximalism principle

Design for Combinatorial Maximalism asks:

> What is the largest lawfully generable, verifiable, composable capability surface obtainable from
> the admitted primitives and bounds?

This differs from adding every feature. A combination that cannot be admitted, verified, or
receipted is outside the manufactured capability surface even if source code can express it.

## 21.2 Architectural families and lenses

Let architectural family set be

\[
\mathcal{F}=\{F_1,\ldots,F_n\}.
\]

Let evaluation lenses be

\[
\mathcal{L}=
\{\ell_{\mathrm{semantics}},
\ell_{\mathrm{planning}},
\ell_{\mathrm{geometry}},
\ell_{\mathrm{execution}},
\ell_{\mathrm{evidence}},
\ell_{\mathrm{performance}},
\ell_{\mathrm{security}},
\ell_{\mathrm{formal}}\}.
\]

The atlas is a matrix

\[
A:\mathcal{F}\times\mathcal{L}\to\Sigma\times E.
\]

Every cell has standing and evidence. Empty cells are Unknown, not implicitly irrelevant.

## 21.3 Edges dominate nodes

For families \(F_i,F_j\), define candidate integration edge

\[
e_{ij}:F_i\to F_j.
\]

The combinatorial possibility count can grow as \(n(n-1)\), but only a small subset is semantically
meaningful. Each edge has a contract:

\[
\mathcal{K}_{ij}
=
(\operatorname{sourceType},
\operatorname{targetType},
\operatorname{mapping},
\operatorname{invariants},
\operatorname{evidence},
\operatorname{falsifier}).
\]

A complete node with missing outgoing edges cannot participate in a crown path. Therefore roadmap
priority is often determined by the smallest missing load-bearing edge, not the largest unfinished
component.

## 21.4 Explore and exploit

Let \(U(a)\) be expected capability utility of action \(a\), and \(I(a)\) expected information gain.
A bounded design policy can maximize

\[
J_{\beta}(a)=U(a)+\beta I(a),
\]

where \(\beta\ge0\) is an admitted exploration weight.

Pure exploitation \(\beta=0\) can trap the system in a local architecture. Excessive exploration can
produce no standing capability. DfCM treats the two as structural tension: exploration expands the
design space; exploitation manufactures evidence-bearing paths through it.

Expected values require a probability model. Without calibrated probabilities, MFW can use a
deterministic multiobjective frontier instead of pretending numerical expectation.

## 21.5 Property-scoped 80/20

The phrase “finish the 80/20” is lawful only after a property and measure are declared. Let open
items be \(R=\{r_1,\ldots,r_m\}\) with load-bearing value \(v_i\) and cost \(c_i\). A bounded subset
selection is a knapsack problem:

\[
\max_{x_i\in\{0,1\}}
\sum_i v_ix_i
\quad\text{subject to}\quad
\sum_i c_ix_i\le b.
\]

The highest count of closed files is not necessarily the highest value. Namespace collisions that
block graph identity may dominate many small documentation fixes if they prevent the standing
ledger from distinguishing objects.

## 21.6 Residue-driven roadmapping

Let goal \(g\) have required crown path \(P_g\). Let real edge set be \(E_R\). The immediate residue is

\[
\operatorname{Gap}(g)
=
E(P_g)\setminus E_R.
\]

For each missing edge, compute its backward dependency support and smallest verified repair socket.
Roadmap ranking may use:

\[
\operatorname{Priority}(e)
=
\frac{
\operatorname{GoalCentrality}(e)
\cdot
\operatorname{ExpectedStandingGain}(e)
}{
\operatorname{Cost}(e)
\cdot
\operatorname{Risk}(e)
}.
\]

Again, the weights are policy. The important discipline is that the numerator comes from explicit
goal topology rather than rhetorical urgency.

## 21.7 Architecture generation and benchmarking

At Level 3, templates generate candidates:

\[
\operatorname{ArchGen}(G,\theta_i)=A_i.
\]

Every candidate receives identical workload \(W_0\), environment \(E_0\), and criteria \(Q_0\):

\[
b_i=\operatorname{Benchmark}(A_i,W_0,E_0,Q_0).
\]

The result includes raw traces and receipts, not only summary means. Warm-up, repetitions, variance,
hardware, software versions, and exclusions are part of the evidence.

## 21.8 Anticipatory negative manufacture

DfCM manufactures not only successful artifacts but refusal knowledge. Let

\[
\mathcal{N}
=
\{x\in\mathcal{X}\mid C(x)=0\}
\]

be unlawful or unsupported combinations. Typed refusal templates partition \(\mathcal{N}\) into
explainable classes. A mature architecture increases both:

\[
|\mathcal{X}_C|
\quad\text{and}\quad
\operatorname{Coverage}(\mathcal{N}),
\]

because it can create more lawful consequences while more precisely refusing the rest.

## 21.9 Anti-gaming constraints

A maximalism metric is invalid if capability increases by:

- weakening tests;
- shrinking the subject without renaming the claim;
- converting real edges to documentation links;
- counting generated variants that are semantically equivalent;
- ignoring unsupported runtime features;
- omitting resource or evidence costs;
- rounding PartialAlive to Alive; or
- treating a refused run as an achieved consequence.

The standing ledger, quotient design space, and same-object verification defend against these forms
of metric gaming.

---


# Part IX. Formal Guarantees and Their Boundary

# Chapter 22. Residue, Isolation, Commutation, and Receipted Completion

## 22.1 Formal objects

Let source set be finite:

\[
X=\{x_1,\ldots,x_n\}.
\]

Let artifact or obligation set be finite:

\[
Y=\{y_1,\ldots,y_m\}.
\]

For each \(y\in Y\), let

\[
\operatorname{MinSupp}(y)\subseteq\mathcal{P}(X)
\]

be the family of inclusion-minimal supports defined in Chapter 5.

## 22.2 Minimal supports form an antichain

### Theorem 22.1

For fixed \(y\), \(\operatorname{MinSupp}(y)\) is an antichain under subset inclusion.

**Proof.** Let \(S_1,S_2\in\operatorname{MinSupp}(y)\). Suppose
\(S_1\subsetneq S_2\). Because \(S_1\) is a support, \(S_2\) has a proper subset that is a support.
This contradicts minimality of \(S_2\). The symmetric strict inclusion is equally impossible.
Therefore distinct minimal supports are incomparable. \(\square\)

This theorem is purely order-theoretic and does not depend on MFW implementation.

## 22.3 Load-bearing members of a minimal support

### Theorem 22.2

If \(S\) is a minimal support for \(y\), then for every \(x\in S\), \(S\setminus\{x\}\) is not a
support.

**Proof.** \(S\setminus\{x\}\subsetneq S\). Minimality says no proper subset of \(S\) is a support.
\(\square\)

This gives the exact meaning of “every member load-bearing” for a support. It does not say changing
each member changes \(y\) under every possible surrounding assignment; it says the member cannot be
removed while retaining the universal determination property.

## 22.4 Changed support and possible residue

For changed source set \(\Delta\subseteq X\), define possible residue:

\[
\operatorname{PossRes}(\Delta)
=
\{y\in Y\mid
\exists S\in\operatorname{MinSupp}(y),\
S\cap\Delta\neq\varnothing\}.
\]

This set is exact for possible semantic dependence under the support model. Actual output change may
be smaller:

\[
\operatorname{ActualRes}(\Delta,u,u')
=
\{y\mid f_y(u)\neq f_y(u')\},
\]

where assignments \(u,u'\) differ only on \(\Delta\).

Then, assuming the minimal support family is complete:

### Theorem 22.3 — No actual change outside possible residue

\[
\operatorname{ActualRes}(\Delta,u,u')
\subseteq
\operatorname{PossRes}(\Delta).
\]

**Proof.** Let \(y\notin\operatorname{PossRes}(\Delta)\). Every minimal support of \(y\) is disjoint
from \(\Delta\). Select one minimal support \(S\). Since \(u\) and \(u'\) differ only on \(\Delta\),
they agree on \(S\). Because \(S\) is a support, \(f_y(u)=f_y(u')\). Therefore
\(y\notin\operatorname{ActualRes}\). Taking the contrapositive proves the inclusion. \(\square\)

## 22.5 Minimal regeneration claim

If a manufacturer recomputes exactly \(\operatorname{ActualRes}\), the execution set is
instance-minimal. Computing ActualRes can require evaluating artifacts. If it recomputes
PossRes, it is semantically safe but may over-regenerate for a particular value change.

Therefore four claims must be distinguished:

1. **dependency reachability:** recompute graph descendants;
2. **possible semantic residue:** recompute artifacts with intersecting complete minimal support;
3. **actual instance residue:** recompute artifacts whose values will change for this update;
4. **minimal execution schedule:** recompute the changed artifacts with no redundant execution.

A theorem for one level cannot be narrated as all four.

## 22.6 Tenant support

Let tenants be \(t\in\mathcal{T}\). Let \(Y_t\) be the tenant's obligation artifacts and define total
support:

\[
S_t
=
\bigcup_{y\in Y_t}
\bigcup_{S\in\operatorname{MinSupp}(y)}S.
\]

Structural tenant support isolation is:

\[
t_1\neq t_2
\Rightarrow
S_{t_1}\cap S_{t_2}=\varnothing.
\]

### Theorem 22.4 — Disjoint support noninterference

Assume every tenant output is a deterministic function only of its support. If
\(S_{t_1}\cap S_{t_2}=\varnothing\), then changing only inputs in \(S_{t_1}\) cannot change any
output of tenant \(t_2\).

**Proof.** Let assignments \(u,u'\) differ only in \(S_{t_1}\). Since supports are disjoint, they
agree on every support of every \(y\in Y_{t_2}\). By the support definition,
\(f_y(u)=f_y(u')\) for all \(y\in Y_{t_2}\). \(\square\)

This is stronger than a runtime access-control intention but narrower than complete confidentiality.
Timing, shared caches, global quotas, logs, or external systems outside the support model can create
side channels. A claim of total tenant isolation must include them in \(X\).

## 22.7 Operation support and commutation

Let operation \(a\) read set \(R_a\subseteq X\) and write set \(W_a\subseteq X\). A sufficient
independence condition for deterministic local updates is:

\[
W_a\cap(R_b\cup W_b)=\varnothing
\]

and

\[
W_b\cap(R_a\cup W_a)=\varnothing.
\]

### Theorem 22.5 — Disjoint read/write commutation

If operations \(a\) and \(b\) are deterministic and satisfy the two disjointness conditions, then

\[
a\circ b=b\circ a.
\]

**Proof.** Operation \(b\) does not change any value read or written by \(a\), so \(a\)'s computed
write values are identical whether \(b\) runs first or second. Symmetrically, \(a\) does not change
any value read or written by \(b\). Their write sets are disjoint, so the final assignment contains
the same writes from each operation and the same untouched coordinates in either order.
\(\square\)

The conditions are sufficient, not necessary. Two operations can commute while sharing state, for
example integer additions, but that requires an operation-specific theorem.

## 22.8 Swap relation

Define a sequence rewrite relation \(\rightsquigarrow\) that swaps adjacent independent operations:

\[
u\,a\,b\,v
\rightsquigarrow
u\,b\,a\,v
\quad\text{when }a\mathrel{I}b.
\]

If independence \(I\) is symmetric, the reverse swap is also permitted. Therefore the naive relation
is generally nonterminating:

\[
ab\rightsquigarrow ba\rightsquigarrow ab\rightsquigarrow\cdots.
\]

Nontermination does not imply lack of local confluence. Because each swap preserves denotation under
commutation, all sequences connected by swaps have the same final state.

To obtain a terminating normalizer, choose a total key order \(<_{k}\) and permit only swaps that
remove inversions:

\[
ba\rightsquigarrow ab
\quad\text{when }a<_{k}b\land aIb.
\]

Each rewrite reduces the finite inversion count:

\[
\operatorname{Inv}(w)
=
|\{(i,j)\mid i<j,\ k(w_i)>k(w_j),\ w_iIw_j\}|.
\]

Because \(\operatorname{Inv}\in\mathbb{N}\) strictly decreases, normalization terminates.

## 22.9 Replay convergence

### Theorem 22.6 — Permutation convergence under adjacent commuting swaps

If sequences \(u\) and \(v\) differ only by a finite sequence of swaps of adjacent commuting
operations, then for every initial state \(s\),

\[
\llbracket u\rrbracket(s)=\llbracket v\rrbracket(s).
\]

**Proof by induction on the number of swaps.** Zero swaps gives identical sequences. For the
successor case, one adjacent commuting swap preserves the intermediate state transformation by
definition of commutation. Apply the induction hypothesis to the remaining swaps. \(\square\)

Cross-region convergence is established for the operation classes that satisfy the premises. It is
not inferred from an absence of observed divergence.

## 22.10 Receipted completion invariant

Let an abstract transition system have state constructors from Chapter 11. Define

\[
\operatorname{HasPostReceipt}(x)
\]

by pattern matching: true exactly for Completed with a valid post-receipt.

### Theorem 22.7

\[
\forall x,\
\operatorname{IsCompleted}(x)
\Rightarrow
\operatorname{HasPostReceipt}(x).
\]

**Proof.** Constructor inversion as in Theorem 11.1. \(\square\)

The concrete-system corollary additionally assumes every completion enters through the typed
transition core. If an external script mutates authoritative state directly, the abstract theorem
has not failed; the system boundary has been violated.

## 22.11 Crown path composition

Let family-edge witnesses be relations

\[
R_{02,03},R_{03,08},R_{08,09},R_{09,10}.
\]

A crown witness exists for input \(x_{02}\) only if there are same-run objects
\(x_{03},x_{08},x_{09},x_{10}\) such that:

\[
R_{02,03}(x_{02},x_{03})
\land
R_{03,08}(x_{03},x_{08})
\land
R_{08,09}(x_{08},x_{09})
\land
R_{09,10}(x_{09},x_{10}).
\]

Separate edge tests with unrelated fixtures do not supply the existential chain because the
intermediate witnesses are not shared. This formalizes the literal-prefix requirement.

## 22.12 Local and external crowns

Both local and external observations must traverse:

\[
F02\rightarrow F03\rightarrow F08\rightarrow F09\rightarrow F10.
\]

Only after \(F10\) may their paths branch toward local execution or external cut machinery. Define:

\[
\operatorname{LocalCrown}
\]

and

\[
\operatorname{ExternalCrown}
\]

as separate existential witness records. The marker

\[
\operatorname{ObservationToReplayContiguousPath}=\mathsf{true}
\]

is lawful only when both records exist with real, same-object edges and terminal replay evidence.

---

# Chapter 23. mfact, Lean, ggen, and the Boundary of Proof

## 23.1 Proposition as type

In dependent type theory, a proposition \(P\) can be represented as a type. A proof is a term

\[
p:P.
\]

Checking the proof means verifying that term \(p\) has type \(P\) under the declarations and axioms
in scope. Lean's elaborator and tactics help construct terms; its small kernel checks resulting proof
terms.

A tactic bug that produces an invalid term is rejected by a correct kernel. A kernel bug, imported
inconsistent axiom, or mistranslated proposition remains part of the trusted boundary.

## 23.2 Kernel checking

Let source theorem declaration be \(T\), elaborated proof term \(p_T\), environment \(\Gamma\), and
kernel checker \(K\). Kernel acceptance is:

\[
K(\Gamma,p_T,T)=\mathsf{Accept}.
\]

This establishes:

\[
\Gamma\vdash p_T:T.
\]

It does not establish:

- that the English title of \(T\) accurately describes the formal proposition;
- that \(\Gamma\) contains no unwanted axioms;
- that generated source corresponds to the intended RDF law;
- that a Rust or Erlang runtime implements \(T\); or
- that a real event occurred.

Each is a separate edge.

## 23.3 The mfact manufacturing rail

The intended controlled chain is:

\[
\text{RDF law}
\xrightarrow{\text{ggen}}
\text{Lean source candidate}
\xrightarrow{\text{elaboration}}
\text{proof term}
\xrightarrow{\text{Lean kernel}}
\text{accepted theorem artifact}
\xrightarrow{\text{mfact}}
\text{certificate}.
\]

In the project's preferred responsibility statement:

> mfact proves the law. Praxis manufactures the consequence. Receipts prove occurrence. Replay
> proves reconstruction.

This statement prevents proof and operation from collapsing.

## 23.4 Build identity

A formal build receipt includes:

\[
R_F=(d_{\mathrm{RDF}},d_{\mathrm{generator}},d_{\mathrm{LeanSource}},
d_{\Gamma},v_{\mathrm{Lean}},d_{\mathrm{proofArtifact}},
\operatorname{theoremInventory},\operatorname{axiomInventory}).
\]

If any input changes, the old receipt cannot certify the new object. Rebuilding without a lock or
environment identity weakens reproducibility.

## 23.5 Project-reported evidence

The research record supplied to this dissertation reports an mfact/procint evidence run with:

- 8,611 jobs;
- 145 kernel-checked theorems;
- 318 artifacts; and
- foldHash/runId identity.

These figures are **project-reported observations**. They should be independently recoverable from
the referenced build receipt before use in an external audit. This thesis does not convert counts
into a universal proof-coverage claim.

## 23.6 Theorem families named by the target release

The release narrative names formal work concerning:

1. residue and antichain minimal-support theory;
2. tenancy-isolation disjointness;
3. commuting-swap replay convergence, including the finding that a symmetric naive swap relation is
   nonterminating while local confluence may hold; and
4. a receipted-completion invariant.

The mathematical cores are reconstructed in Chapter 22. The exact kernel standing of each named
implementation theorem must be read from the theorem inventory, not inferred from this prose.

## 23.7 Candidate POWL formalization

The project record also describes a POWL Lean archive that elaborated with zero diagnostics in its
available checker path, while native lake build was blocked by a missing native driver. Additional
multifractal modules remained candidate rather than fully built. Therefore:

- source-level formal definitions and proofs may exist;
- an elaboration observation may be Alive for the observed tool path;
- a complete native build receipt remains Blocked or Unknown for the blocked path; and
- no implementation correspondence is promoted by source presence.

The distinction is exactly why standing is property-scoped.

## 23.8 mathlib capability boundary

The mathematical library surface includes foundations for Hausdorff measure and dimension, outer
measure, ergodic theory including Birkhoff-type results, and topological entropy. A native,
project-specific multifractal namespace is not presumed merely because these ingredients exist.
Definitions and theorems still need to be assembled and checked.

## 23.9 The correspondence decision

There are two legitimate claim scopes.

### Scope A — Certified law artifact

Claim: the controlled RDF-to-Lean-to-kernel-to-certificate chain produced an accepted theorem
artifact. Downstream consumers are outside the claim. No FFI theorem is required to make Scope A
true.

### Scope B — Deployed implementation of certified law

Claim: a particular binary or runtime realizes the proven transition or property. This scope
requires a compiler correctness argument, refinement proof, proof-carrying generated code, verified
interpreter, exhaustive finite correspondence, or other admitted bridge.

The project may intentionally choose Scope A. It may not state Scope B using Scope A evidence.

## 23.10 Trusted computing base

The formal trusted computing base includes at least:

- the Lean kernel binary and its execution environment;
- the declared axioms and imported theorem libraries;
- parsing and elaboration to the extent they affect the checked term presented to the kernel;
- the mechanism identifying source, theorem, and artifact digests;
- the cryptographic assumptions of the receipt; and
- the human correspondence between formal proposition and release claim.

Minimizing this base increases auditability. It never becomes literally zero.

## 23.11 Proof-carrying manufacture

An ideal manufactured artifact carries:

\[
A^{\dagger}=(A,P_A,p_A,R_A),
\]

where \(P_A\) is an exact formal property, \(p_A:P_A(A)\) a proof term, and \(R_A\) the operational
receipt. The proof says what is guaranteed about objects satisfying the formal model. The receipt
says which artifact was manufactured and observed.

The combination is stronger than either alone, but only if the identity in \(P_A(A)\) binds the same
artifact digest as \(R_A\).

---


# Part X. Comprehensive Anticipatory Design Science

# Chapter 24. The Fuller Canon as Controlled-System Engineering

## 24.1 Why the lens matters

Buckminster Fuller's canon asks design to be comprehensive, anticipatory, scientific, resource
accountable, synergetic, and oriented toward doing more with less. MFW adopts these as engineering
constraints rather than inspirational decoration.

The scope excludes adoption, public approval, market response, and every other variable outside the
controlled system. This is not a retreat from civilization-scale design. It is a refusal to claim
causal control where none exists.

Define the controlled boundary:

\[
B=(\operatorname{resources},\operatorname{laws},\operatorname{state},
\operatorname{actuators},\operatorname{observers},\operatorname{receipts}).
\]

The civilization-scale question becomes:

> What new whole-system design capacity exists inside \(B\), whether or not any external actor
> chooses to use it?

## 24.2 Comprehensive

A design is comprehensive relative to \(B\) when every class capable of changing the result is
either:

1. represented inside the model;
2. explicitly parameterized as an exogenous input; or
3. explicitly excluded from the claim.

Let relevant variable set be \(X_B\). Let modeled variables be \(M\), exogenous variables \(E\), and
excluded variables \(Q\). Comprehensiveness requires:

\[
X_B=M\cup E\cup Q,
\]

with pairwise distinctions recorded and no unnamed residue. This does not mean infinite detail. It
means the abstraction boundary is explicit and every omitted class has standing.

## 24.3 Anticipatory

Let possible admitted future states be

\[
\mathcal{F}_B(x,K)
=
\{x'\mid x\rightsquigarrow^{*}x'
\text{ under law and bound }K\}.
\]

Anticipatory design evaluates consequences before actuation:

\[
\operatorname{Evaluate}:
\mathcal{F}_B(x,K)\to
\mathbb{R}^{m}\times\Sigma.
\]

Planning, simulation, model checking, formal proof, negative fixtures, cost bounds, interference
analysis, and architecture benchmarks are distinct anticipatory instruments. A modeled future
remains hypothetical until observed.

## 24.4 Design science

An architecture hypothesis is

\[
H=(D,P,F),
\]

where \(D\) is a manufactured design, \(P\) a predicted property, and \(F\) a falsification
experiment. A valid experiment yields observation \(o\), which updates standing without editing the
original prediction.

The design-science cycle is:

\[
\text{hypothesis}
\to\text{artifact}
\to\text{experiment}
\to\text{observation}
\to\text{admission}
\to\text{standing}.
\]

Receipts supply reproducibility; replay supplies reconstruction; neither eliminates the need for
adversarial tests.

## 24.5 Ephemeralization

Fuller's “doing more with less” becomes the ephemeralization functional of Chapter 20. For resource
budget \(b>0\),

\[
\mathcal{E}_B(x,b)
=
\frac{
\text{weighted lawful reproducible consequence capacity reachable from }x
}{
\text{matter}+\text{energy}+\text{time}+\text{compute}+\text{interpretation}
}.
\]

MFW increases the numerator by manufacturing reusable law, process geometry, adapters, proof
artifacts, and receipts. It decreases the denominator by:

- eliminating duplicate private representations;
- capitalizing recurrent inference;
- regenerating only the lawful residue;
- exposing concurrency;
- preventing unauthorized or irreconstructible work;
- replaying instead of rediscovering;
- moving suitable execution to smaller runtimes; and
- converting agent investigation into reusable graph state.

The ratio must not improve by weakening evidence or moving cost outside accounting.

## 24.6 Synergetics

The capability belongs to the configuration:

\[
\{\text{RDF},\text{GraphLaw},\text{PDDL},\text{POWL},
\text{ggen},\text{agents},\text{runtimes},
\text{proofs},\text{receipts},\text{replay}\}.
\]

RDF without actuation describes. Actuation without permission is dangerous. Planning without
observation is fictional. Proof without occurrence does not establish history. Receipts without law
record arbitrary events. Their lawful composition can have positive synergy:

\[
\operatorname{Syn}(\text{MFW configuration})>0
\]

under the capability protocol of Chapter 20.

This inequality is an empirical target, not granted by architectural beauty. Isolation benchmarks
must estimate component baselines and the configured whole.

## 24.7 Tensegrity as separated authority

Tensegrity maintains integrity through balanced tension and compression. The corresponding MFW
design principle is separation of mutually checking authorities:

\[
\begin{aligned}
\text{observation}&\dashv\text{admission},\\
\text{plan}&\dashv\text{permission},\\
\text{proposal}&\dashv\text{verification},\\
\text{theorem}&\dashv\text{receipt},\\
\text{exploration}&\dashv\text{exploitation},\\
\text{local execution}&\dashv\text{external cut}.
\end{aligned}
\]

The symbol \(\dashv\) here denotes a designed counterpoise, not a category-theoretic adjunction.
Neither side is deleted; the system gains stability from their constrained relation.

## 24.8 Trimtab

A trimtab is operationalized as a smallest support intervention with high whole-system leverage.
Given goal capability gain \(\Delta C_g\), define:

\[
t^{*}\in
\operatorname*{arg\,max}_{t}
\frac{\Delta C_g(t)}
\operatorname{cost}(t)}
\]

subject to lawful permission, risk, evidence, and support-size constraints.

Semantic residue identifies where an intervention is load-bearing. POWL topology identifies where it
can be grafted. Receipts test whether the anticipated redirection occurred. This closes the loop
between trimtab metaphor and engineering evidence.

v26.7.13 is a trimtab release in a specific sense: it aims to move MFW's own lifecycle inside MFW.
Once the manufacturing system manufactures itself, each later improvement can compound through the
same machinery.

## 24.9 World Game

Define a bounded whole-system model:

\[
\mathcal{W}=(R,N,C,L,A,T),
\]

where \(R\) is resource state, \(N\) needs or goals, \(C\) capabilities, \(L\) law, \(A\) actions, and
\(T\) transition semantics.

A World Game scenario is an admitted initial state plus a policy:

\[
\omega=(x_0,g,K,P).
\]

The system searches \(\mathcal{F}_B(x_0,K)\), compares feasible futures on a multiobjective frontier,
and returns a plan witness, exhaustion certificate, bound, unsupported capability, or inconsistency.

The decisive extension is that an admitted winning strategy can become POWL geometry, cross
permission, actuate in the controlled system, and return observations. The World Game becomes an
executable scientific instrument, not only a simulation.

## 24.10 Geoscope

A Geoscope is formalized as a projection of the authoritative state and future set:

\[
\operatorname{GeoView}:
(O^{*},\mathcal{F}_B,E,R)\to\mathcal{V},
\]

where \(\mathcal{V}\) is a human-interpretable spatial, temporal, causal, and evidentiary
visualization.

The visualization must distinguish:

- observed present;
- admitted present;
- simulated future;
- authorized process;
- observed consequence;
- uncertainty;
- missing data; and
- claim standing.

Animation that merges these layers would be epistemically false even if visually compelling.

## 24.11 Spaceship Earth and closed-loop accounting

Inside the controlled boundary, no resource-changing action is allowed outside its account:

\[
\forall a,\
\operatorname{ResourceChanging}(a)
\Rightarrow
\exists R^{-}_a,R^{+}_a.
\]

The resource ledger tracks inputs, transformations, outputs, waste, recovery, and unknowns. A
negative balance or missing mass-energy quantity signals incomplete boundary accounting. Digital
actions also account for compute, storage, network, and interpretation, even though their physical
measurement may initially be PartialAlive.

## 24.12 Civilization-scale capacity

Let the set of civilization-relevant design classes inside the model be \(\mathcal{G}\): energy,
water, food, shelter, mobility, health, education, communication, governance, computation, and
others explicitly admitted. For each class \(g\), let \(\mathcal{K}_g(x,b)\) be lawful reproducible
consequences reachable under budget \(b\).

Define controlled design-capacity impact:

\[
\mathcal{I}_B(x\to x';b)
=
\sum_{g\in\mathcal{G}}
\omega_g
\left(
K_g(x',b)-K_g(x,b)
\right),
\]

where \(\omega_g\) are explicit weights or the vector is reported without scalarization.

This measures created capacity, not adoption. A positive value says more whole-system consequences
can now be lawfully designed, tested, manufactured, and reconstructed inside the controlled system.
It says nothing about whether external institutions choose them.

## 24.13 The comprehensive result

MFW's Fuller-aligned contribution is not that one software release “changes civilization.” It is
that a new controlled capacity may exist:

\[
\text{public meaning}
\to\text{whole-system model}
\to\text{lawful future search}
\to\text{minimal intervention}
\to\text{permissioned manufacture}
\to\text{receipted reality}
\to\text{replayable learning}.
\]

The capacity becomes real only to the standing supported by the release evidence in Chapter 26.

---

# Chapter 25. Evaluation Method, Exact Witnesses, and Falsification

## 25.1 Evaluation philosophy

MFW is evaluated by triangulating:

1. formal proof over an exact model;
2. executable verification over a concrete implementation;
3. same-object end-to-end witnesses;
4. adversarial mutation and chaos;
5. performance and resource measurement; and
6. replay by an independent verifier.

No one axis substitutes for the others.

## 25.2 Verification ladder

For each claim, record the highest completed execution rung:

\[
\mathsf{Unit}
\to
\mathsf{Integration}
\to
\mathsf{EndToEnd}
\to
\mathsf{Chaos}
\to
\mathsf{Stress}
\to
\mathsf{Benchmark}
\to
\mathsf{IndependentReplay}.
\]

Formal proof is recorded in a parallel column:

\[
\mathsf{None},
\mathsf{Defined},
\mathsf{ProvedOnPaper},
\mathsf{Elaborated},
\mathsf{KernelChecked},
\mathsf{ImplementationConnected}.
\]

## 25.3 Non-vacuity

A test for property \(P\) is non-vacuous when a controlled mutation violating \(P\) causes the test
to fail for the predicted reason. Protocol:

1. establish green baseline;
2. apply one minimal mutation \(m\) known to violate \(P\);
3. run the same test and observe failure;
4. verify the failure points to \(P\);
5. restore byte-identically;
6. rerun and observe green; and
7. receipt all three states.

This does not prove the test catches every violation, but it proves the assertion is load-bearing for
\(m\).

## 25.4 Baselines

Compare MFW with:

- hand-authored release scripts;
- conventional CI pipelines;
- LLM execution without admitted RDF lifecycle state;
- a flat DAG without recursive grafting;
- full-graph planning without semantic contraction;
- logging without pre-actuation receipts;
- replay without canonicalized identity; and
- private ontology variants where public mappings exist.

The comparison workload, boundary, and completion criteria must be identical.

## 25.5 Exact local crown

The local crown begins with a real observation admitted at F02 and must traverse the literal prefix:

\[
F02\to F03\to F08\to F09\to F10.
\]

It then continues through the local execution path and returns an observation that is re-admitted,
receipted, and replayed. Every intermediate object shares the run and causal lineage.

## 25.6 Exact external crown

The external crown begins independently with a real observation and traverses the same literal
prefix before taking the external cut. It must then exercise a real Arazzo/AIR/Erlang or other
declared external execution path, broker effect, return observation, receipt, and replay.

An internal fixture that simulates external completion is useful test evidence but not an external
crown.

## 25.7 Dual-crown acceptance

Define witness predicates \(L(w_L)\) and \(X(w_X)\). The release marker is:

\[
\operatorname{Contiguous}
\Longleftrightarrow
\exists w_L,w_X,\
L(w_L)\land X(w_X)
\land\operatorname{PrefixReal}(w_L)
\land\operatorname{PrefixReal}(w_X)
\land\operatorname{ReplayValid}(w_L)
\land\operatorname{ReplayValid}(w_X).
\]

If either witness is absent, the marker is false.

## 25.8 Enterprise architecture case

The TOGAF case evaluates full lifecycle structure: Preliminary, Phases A through H, and Requirements
Management. It tests whether public semantic architecture state can manufacture phase workflows,
roles, evidence, iteration, and continuation growth.

The case is not evidence that every institution's enterprise architecture has been modeled. Its
subject is the admitted case-study ontology and fixtures.

## 25.9 SOC2 evidence case

The SOC2 case evaluates a ten-phase audit engagement:

\[
\begin{aligned}
\text{Scoping}
&\to\text{Readiness}
\to\text{Control Documentation}
\to\text{Design Evaluation}\\
&\to\text{Evidence Period}
\to\text{Operating Effectiveness}
\to\text{Exceptions}\\
&\to\text{Management Response}
\to\text{Evidence Bundle}
\to\text{Auditor Handoff}.
\end{aligned}
\]

Quarterly retesting exercises recursive F09 growth. Evidence remains representative and synthetic
unless a real controlled audit engagement is admitted.

## 25.10 Rust dry-run case

The Rust case is the primary Operation Dogfood witness because it exercises discovery, permission,
Claude Code repair, real commands, packaging, portability, refusal, receipts, and replay on the
software that implements the machinery.

## 25.11 Chaos cases

Required chaos and negative fixtures include:

- crash after pre-receipt but before dispatch;
- crash after dispatch but before result receipt;
- duplicate broker delivery;
- stale permission digest;
- expired permission;
- out-of-scope file edit;
- missing payload;
- hash or canonicalization mismatch;
- orphan tool event;
- false Bounded-to-Exhausted conversion;
- child workflow that attempts authority widening;
- runtime restart with new PID and stable workflow identity;
- reordered commuting operations;
- reordered noncommuting operations;
- corrupted profile hash in Triple8;
- missing result admission; and
- replay under a different toolchain identity.

Each fixture has a predicted typed result.

## 25.12 Stress and benchmark

Stress dimensions include:

\[
\begin{aligned}
n_T&=\text{RDF triple count},\\
n_F&=\text{ground fluent count},\\
n_A&=\text{ground action count},\\
n_V&=\text{POWL activity count},\\
n_E&=\text{event count},\\
d_R&=\text{recursion depth},\\
c_W&=\text{workflow concurrency width},\\
n_C&=\text{crate count}.
\end{aligned}
\]

Report latency distributions, throughput, memory, storage, compute, energy where measurable, and
receipt/replay overhead. Correctness markers are reported separately.

## 25.13 Statistical protocol

For repeated benchmark measurements \(x_1,\ldots,x_n\), report:

\[
\bar x=\frac{1}{n}\sum_i x_i
\]

and sample variance

\[
s^2=\frac{1}{n-1}\sum_i(x_i-\bar x)^2.
\]

Use quantiles for skewed latency, bootstrap intervals where distributional assumptions are weak,
and disclose warm-up and outlier policy. A benchmark result is an observation under a named
environment, not an asymptotic theorem.

## 25.14 Falsification matrix

| Claim | Same-object falsifier |
|---|---|
| RDF is lifecycle authority | A material state transition exists only in chat memory, JSON, or process memory |
| Planner is truthful | A live frontier is reported as Exhausted |
| POWL exposes concurrency | A required causal edge is removed or a compatible antichain is serialized without reason |
| Graft preserves obligations | A child completes after deleting a parent gate |
| Permission precedes mutation | A write occurs without applicable plan-bound permission |
| Zero unreceipted actuation | A brokered effect lacks durable pre-receipt |
| Completed implies outcome receipt | A Completed state can be constructed without post-receipt |
| Replay reconstructs | Same inputs and events fail the declared equivalence |
| Tenant support is disjoint | A modeled source appears in both tenant support sets |
| Cross-region operations commute | A covered pair produces different states under reversal |
| Dry-run publish is ready | Any selected crate fails a required real gate |
| Workflow is empirically multifractal | Spectrum instability or surrogate equivalence explains the result |

---

# Chapter 26. Current Standing of v26.7.13

## 26.1 Date and evidence boundary

This ledger is frozen to the source record available on 13 July 2026. Some entries are drawn from
the v26.7.13 PRD, ARD, manifesto, Vision 2030, prior thesis, and session-provided progress record.
Where this thesis did not rerun the underlying repository, the standing says project-reported rather
than independently reproduced.

## 26.2 Operation Dogfood claim reconciliation

| ID | Exact claim | Standing | Promotion evidence |
|---|---|---|---|
| C1 | MFW models bounded PDDL and projects POWL workflow structure | ALIVE for existing TOGAF/SOC2 slices | Preserve tests and same-object projection evidence |
| C2 | RDF is lifecycle authority from intent through replay | PLANNED | One complete run reconstructible from RDF |
| C3 | Reconnaissance and Explore work are dogfooded | PLANNED | Every task, tool event, observation, claim, and result bound and receipted |
| C4 | MFW discovers an unfamiliar Rust repository | PLANNED | Successful bounded discovery without a hand-authored release workflow |
| C5 | MFW plans and asks permission before mutation | PLANNED | ODRL-backed permission bound to exact plan digest and mutation set |
| C6 | MFW launches and governs Claude Code for repair | PLANNED | Real failed gate, child workflow, invocation, verified patch, receipt, parent re-entry |
| C7 | Every Claude Code tool event is RDF end to end | PLANNED | Complete pre/post lifecycle coverage with zero orphans |
| C8 | Whole-workspace Rust dry-run publication succeeds | REFUSED | Close or explicitly rescope every real blocker and rerun |
| C9 | Receipt and replay cover the complete dogfood lifecycle | PARTIAL_ALIVE | Byte or semantic replay of the entire lifecycle |
| C10 | Public ontology precedes private vocabulary | PARTIAL_ALIVE | Namespace report with bounded private ABI |
| C11 | Outcome algebra is preserved through every layer | PARTIAL_ALIVE | No adapter, CLI, receipt, or replay collapse |
| C12 | Autonomous external publication is permitted | REFUSED by scope | Separate later release and explicit permission surface |

## 26.3 Formalization progress

The progress record reports:

- Rail A formalization \(\Phi\) landed;
- 56 orphaned files were integrated;
- 33 remain, comprising 5 genuine work items and 28 namespace collisions;
- graft_child was implemented;
- the F09-to-F10 production edge was implemented;
- false-success corruption was removed; and
- tests were restored.

These are meaningful implementation gains. Namespace collisions are load-bearing because ambiguous
identity corrupts claim and artifact ledgers. The stated priority is:

1. resolve namespace collisions;
2. upgrade the standing ledger;
3. prove the Crown Theorem obligations;
4. continue Rail A;
5. formalize DescentMeter termination in the proof rail; and
6. connect grafting to the free-monad or equivalent recursive formalization.

## 26.4 Dual crown standing

The required common prefix is:

\[
F02\to F03\to F08\to F09\to F10.
\]

The project record previously identified F08-to-F09 residual-goal extraction as a genuine missing
edge: available F09 machinery parsed externally authored continuation data rather than deriving a
continuation goal algorithmically from the F08 tape. The newly reported F09-to-F10 production edge
does not remove that earlier obligation.

Therefore, absent a new same-object witness proving otherwise:

\[
\operatorname{LocalCrownReal}=\mathsf{false},
\]

\[
\operatorname{ExternalCrownReal}=\mathsf{false},
\]

and

\[
\operatorname{ObservationToReplayContiguousPath}=\mathsf{false}.
\]

This corrects stale v26.7.12 markers that rounded architectural availability into path completion.

## 26.5 Search and manufacturing graph standing

The architecture distinction between microsecond search graph and long-lived manufacturing graph is
specified. Existing PDDL-to-POWL slices demonstrate process structure. Complete lifecycle evidence
connecting selection, permission, execution, receipt, and replay on Operation Dogfood remains
Planned or PartialAlive according to the claim table.

## 26.6 Rust crown standing

The current whole-workspace dry-run remains Refused by recorded path dependencies, licensing,
missing root license, path leakage, and subset-only readiness. The product may still demonstrate a
truthful refused lifecycle, but it may not claim whole-workspace publication readiness.

## 26.7 Formal theorem standing

The mfact theorem suite is project-reported as kernel checked for its recorded theorem inventory.
The theorem count does not establish:

- complete Operation Dogfood lifecycle implementation;
- complete POWL-to-runtime correspondence;
- whole-workspace dry-run success;
- dual crown contiguity; or
- empirical multifractality.

POWL and multifractal Lean modules retain the property-scoped standing described in Chapter 23.

## 26.8 Release decision rule

v26.7.13 may release with refused capability claims if the release honestly states them. It may not
mark Operation Dogfood Alive unless the complete Definition of Done in Chapter 14 is satisfied on a
same-object run.

The release predicate is:

\[
\operatorname{ReleaseHonest}
\Longleftrightarrow
\operatorname{ClaimsMatchEvidence}
\land
\operatorname{NoFalseMarkers}
\land
\operatorname{AllKnownBlockersTyped}
\land
\operatorname{ReceiptsVerify}.
\]

This is distinct from:

\[
\operatorname{AllTargetCapabilitiesAlive}.
\]

A research release can be honest before every target is complete.

---

# Chapter 27. Vision 2030

## 27.1 The operating manual becomes executable

By 2030, the target MFW system can admit a bounded whole system, search lawful futures, manufacture a
minimal sufficient intervention, obtain permission, execute through heterogeneous machinery, and
preserve the consequence as proof-bearing, receipted, replayable standing.

The target chain is:

\[
\begin{aligned}
\text{universe observation}
&\to\text{admitted public semantic model}\\
&\to\text{bounded whole-system design search}\\
&\to\text{minimal-support intervention}\\
&\to\text{permission}\\
&\to\text{recursive manufacture}\\
&\to\text{observed consequence}\\
&\to\text{receipt}\\
&\to\text{replay}\\
&\to\text{new admitted state}.
\end{aligned}
\]

## 27.2 The bootstrap event

Operation Dogfood is the recursive seed. When development goals, reconnaissance, Claude Code
invocations, mutations, verification, and release all become MFW state, every subsequent capability
can be manufactured through the machinery it extends.

Let capability state after release \(n\) be \(x_n\). Let \(\Phi\) be the dogfooded improvement
operator:

\[
x_{n+1}=\Phi(x_n,O_n^{*},g_n,p_n).
\]

The operator is bounded and permissioned. It need not converge. Every iteration returns typed
standing and receipts, allowing the sequence to be studied rather than mythologized.

## 27.3 2030 target capabilities

### Public semantic operating layer

Public ontologies carry architecture, resources, requirements, permission, provenance, observation,
quality, data products, processes, proof standing, and receipts. Private ABI vocabulary is finite,
versioned, and justified.

### Comprehensive system admission

An unfamiliar bounded system can be observed, conflicts identified, authoritative sources selected,
laws recovered, and missing capability refused without requiring a prewritten workflow.

### Truthful design-space engine

Every finite search terminates as Found, Exhausted, Bounded, Unsupported, or Inconsistent. Found
plans carry verifiable witnesses; exhaustion is exact-model scoped; bounds preserve frontiers.

### Recursive process geometry

POWL v2 geometry supports partial order, topology-derived concurrency, choice graphs, bounded loops,
hierarchy, and typed child grafting with obligation preservation.

### Governed machine intelligence

Claude Code and successor cognitive breeds operate as bounded proposers and actuators. They can
manufacture new capability but cannot manufacture new authority.

### Heterogeneous execution

Rust, Erlang/OTP, WASM, AtomVM, Arazzo, AIR, and BCINR participate only on explicit supported
profiles. Semantic equivalence is proved or empirically bounded, never assumed.

### Mathematical manufacture

RDF-declared law projects into kernel-checkable proof artifacts. Theorem identities, axioms,
toolchains, and receipts are independently inspectable.

### Receipted reality

Every controlled side effect has durable intent evidence; every completion has outcome evidence;
every terminal run can be reconstructed to a declared replay criterion.

### Executable process science

Object-centric event evidence supports conformance, causal, statistical, and multifractal analysis.
Findings become candidate law only through admission.

## 27.4 Three nested roadmap scales

### Micro scale

Optimize fixed-table admission, planner verification, local process dispatch, canonicalization, and
receipt overhead.

### Meso scale

Manufacture and repair repository, product, audit, architecture, and release workflows.

### Macro scale

Search whole-system architecture topologies, resource allocations, and cross-domain consequences
under the Fuller calculus.

The same law-state loop governs each scale; their metrics and time constants remain distinct.

## 27.5 Controlled 2030 measures

Vision 2030 is evaluated by:

\[
\operatorname{Coverage}_{\mathrm{RDF}}
=
\frac{\text{authoritative lifecycle classes represented in RDF}}
{\text{declared lifecycle classes}},
\]

\[
\operatorname{ReceiptCoverage}
=
\frac{\text{brokered effects with verified receipt chain}}
{\text{brokered effects observed}},
\]

\[
\operatorname{ReplayRate}
=
\frac{\text{runs meeting declared replay criterion}}
{\text{runs selected for replay}},
\]

\[
\operatorname{ExactResidueRate}
=
\frac{\text{changed artifacts correctly predicted}}
{\text{actual changed artifacts plus false positives}},
\]

\[
\operatorname{Ephemeralization}
=
\frac{\text{lawful reproducible consequence capacity}}
{\text{resource vector or admitted scalarization}},
\]

and a vector of formal and runtime correspondence coverage.

No measure uses adoption as a denominator or completion criterion.

## 27.6 What Vision 2030 refuses

Vision 2030 refuses:

- autonomous authority expansion;
- private-ontology lock-in disguised as semantics;
- planner timeouts reported as impossibility;
- agent confidence reported as admission;
- modeled effects reported as observed effects;
- proof artifacts reported as historical occurrence;
- receipts reported as mathematical correctness;
- one runtime test reported as universal equivalence;
- whole-system language without a declared boundary;
- multifractal language without scale evidence; and
- civilization-impact claims based on variables outside control.

## 27.7 The 2030 completion condition

The target is not one monolithic Alive marker. It is a verified capacity surface whose every cell
has standing, evidence, and falsifier. The governing criterion is:

\[
\forall c\in\operatorname{DeclaredCapability},
\quad
\operatorname{standing}(c)
=
\operatorname{maxStandingSupportedByEvidence}(c).
\]

Honest PartialAlive is preferable to fictional Alive because it preserves the residue from which
the next workflow can be manufactured.

---

# Chapter 28. Limitations and Open Theorems

## 28.1 Open-world completeness

No finite observation guarantees that every relevant fact was observed. Admission can be complete
relative to a declared source set and boundary, not to an unknowable universe. Unobserved external
effects remain a threat to broker-only claims unless the physical and network boundary is enforced.

## 28.2 Planning complexity

Finite classical planning is computationally difficult in general. The \(2^{|F|}\) state bound is
finite but exponential. Semantic contraction, heuristics, hierarchy, and architecture specialization
reduce practical work but need truth-preserving or outcome-preserving evidence.

## 28.3 Residual-goal extraction

Deriving a continuation goal from an F08 planner tape or execution residue remains a central open
edge unless the current repository contains new same-object evidence. An LLM can propose the goal;
the system still needs bounded validation that the proposal corresponds to real residue and does
not widen scope.

## 28.4 POWL v2 completeness

The formal workflow core in this dissertation captures partial order, choice, loop, hierarchy, and
grafting. Exact equivalence to every feature and theorem of the evolving POWL v2 literature requires
a versioned mechanization. Choice-graph semantics and workflow-net transformation classes must be
stated precisely.

## 28.5 Graft implementation correspondence

The mathematical graft theorems assume explicit interface conditions. A proof that the production
graft_child implementation satisfies those definitions remains a separate obligation unless present
in the formal inventory.

## 28.6 Distributed effects

Pre-receipts and idempotency reduce ambiguity but cannot make arbitrary external systems
transactional. UnknownAfterDispatch and compensation remain necessary. Exactly-once claims require
target-specific evidence.

## 28.7 Canonicalization and cryptography

Receipt identity depends on canonicalization and hash assumptions. Algorithm changes, RDF 1.2 triple
terms, blank-node handling, or profile differences can alter canonical bytes. Receipts must name the
exact profile and preserve migration proofs.

## 28.8 Ontology quality

Public vocabulary reduces private lock-in but does not guarantee correct modeling. Terms can be
misused, source ontologies can overlap, and institutional distinctions can exceed existing public
terms. The namespace ledger makes this visible; it does not eliminate semantic review.

## 28.9 N3 expressivity

N3 built-ins can introduce nondeterminism, external access, or nontermination. The quarantine
doctrine bounds risk but limits expressivity. Formal semantics for the selected N3 subset remains a
proof obligation.

## 28.10 Runtime correspondence

Differential tests cover finite corpora. Full Rust/Erlang/WASM/AtomVM correspondence needs stronger
simulation or refinement evidence. The project may scope mfact to certified law artifacts, but
Praxis runtime claims must remain within their receipts.

## 28.11 Empirical multifractality

Finite workflow logs, nonstationarity, hierarchy construction, heavy tails, and short scale ranges
can create false spectra. The multifractal claim remains empirical until robust replicated evidence
exists. The structural recursive law is unaffected if the empirical hypothesis fails.

## 28.12 Capability functional

Capability, value, risk, and interpretation weights are normative model inputs. Sensitivity analysis
must show whether roadmap choices are robust. No scalar functional can silently encode plural human
values.

## 28.13 Fuller calculus boundary

Created design capacity inside MFW is measurable in principle. Civilization-wide consequence
outside the boundary is not controlled and is intentionally excluded. The system can manufacture
options and evidence; it cannot claim external choice.

## 28.14 Current implementation residue

Namespace collisions, remaining orphan files, standing-ledger upgrades, crown theorem proofs, Rail A
completion, DescentMeter mechanization, and recursive formalization remain open according to the
progress record. The exact list must be regenerated from the live repository before release.

## 28.15 Independent audit

This thesis is a formal reconstruction and release artifact. Independent audit should:

1. rebuild source receipts;
2. inspect axioms and theorem statements;
3. rerun crown witnesses;
4. verify no broker bypass exists inside scope;
5. mutate negative fixtures;
6. reproduce package refusals;
7. replay on a clean environment; and
8. compare every public claim with the authoritative RDF ledger.

---

# Chapter 29. Conclusion

Multifractal Workflow begins where ordinary workflow systems stop asking questions. It does not
assume the process is already known, the state already authoritative, the plan already permitted,
the runtime already equivalent, the action already evidenced, or the failure already understood.
It manufactures each missing bridge as an explicit object with standing.

The central operation is recursive grafting under law:

\[
W' = W[a\mapsto W_r].
\]

That equation is meaningful only because the surrounding system supplies:

\[
\begin{aligned}
&\text{RDF admission for }O_r^{*},\\
&\text{truthful bounded planning for }W_r,\\
&\text{POWL geometry for socket }a,\\
&\text{permission containment for the child},\\
&\text{brokered execution for real effects},\\
&\text{receipt dominance for completion},\\
&\text{replay for reconstruction},\\
&\text{a descent measure for termination}.
\end{aligned}
\]

The structural multifractal claim is exact: one obligation-preserving law recurs at campaign,
release, repair, test, and actuation scales. The empirical multifractal claim remains a scientific
hypothesis about measured workflow mass. Algebra defines composition and truthful outcomes;
geometry defines causal neighborhoods, antichains, cuts, and grafts; calculus defines local change,
leverage, and constrained paths; measure theory defines nonuniform scaling. None is decorative.

Operation Dogfood is the decisive experiment because it places the complete Claude Code lifecycle
inside the machinery. The desired Rust dry-run crown is currently refused for the whole workspace,
and the dual observation-to-replay crown is not yet real under the literal-prefix criterion. Those
facts do not weaken the thesis. Rounding them upward would.

Under the Fuller canon, the civilization-scale object is controlled design capacity:

\[
\text{the ability to understand a bounded whole, search its lawful futures, find a small
load-bearing intervention, manufacture it with permission, and preserve the consequence as
reconstructible evidence}.
\]

Adoption is outside the calculus. Capability creation is inside it.

The dissertation's final invariant is therefore not “the system always succeeds.” It is:

\[
\boxed{
\text{No claim outruns its evidence, no actuation outruns its permission and receipt,
and no residue is erased merely because it is unfinished.}
}
\]

That invariant makes failure usable, recursion lawful, and future capability manufacturable.

---

# Appendix A. Consolidated Notation

## A.1 Logical and set notation

| Symbol | Meaning |
|---|---|
| \(\neg P\) | not \(P\) |
| \(P\land Q\) | \(P\) and \(Q\) |
| \(P\lor Q\) | inclusive or |
| \(P\Rightarrow Q\) | implication |
| \(P\Leftrightarrow Q\) | logical equivalence |
| \(\forall x\in X\) | for every \(x\) in \(X\) |
| \(\exists x\in X\) | there exists an \(x\) in \(X\) |
| \(\exists!x\) | there exists exactly one \(x\) |
| \(x\in X\) | \(x\) is a member of \(X\) |
| \(X\subseteq Y\) | \(X\) is a subset of \(Y\) |
| \(\mathcal{P}(X)\) | power set of \(X\) |
| \(X\times Y\) | Cartesian product |
| \(X\sqcup Y\) | disjoint tagged union |
| \(X/{\sim}\) | quotient by equivalence relation \(\sim\) |
| \(|X|\) | finite cardinality |
| \(A\triangle B\) | symmetric difference |

## A.2 Numeric and analytic notation

| Symbol | Meaning |
|---|---|
| \(\mathbb{N}\) | natural numbers including zero |
| \(\mathbb{Z}\) | integers |
| \(\mathbb{R}\) | real numbers |
| \(\mathbb{R}_{\ge0}\) | nonnegative real numbers |
| \(x^{\top}y\) | Euclidean dot product |
| \(\|x\|_2\) | Euclidean norm |
| \(f'(x)\) | ordinary derivative |
| \(\partial f/\partial x_i\) | partial derivative |
| \(\nabla f\) | gradient |
| \(H_f\) | Hessian matrix |
| \(\int f\,d\mu\) | integral with respect to measure \(\mu\) |
| \(\inf\), \(\sup\) | greatest lower and least upper bound |
| \(\arg\max\) | set of maximizing arguments |
| \(O(g(n))\) | asymptotic upper bound |
| \(f\asymp g\) | comparable scaling up to positive constants in declared regime |

## A.3 Graph and workflow notation

| Symbol | Meaning |
|---|---|
| \(G=(V,E)\) | graph with vertices and edges |
| \(E^{+}\) | transitive closure |
| \(x\prec y\) | \(x\) causally precedes \(y\) |
| \(x\parallel y\) | \(x\) and \(y\) are incomparable |
| \(\uparrow x\) | causal future/upward closure |
| \(\downarrow x\) | causal past/downward closure |
| \(\mathcal{L}(W)\) | trace language of workflow \(W\) |
| \(W[a\mapsto U]\) | graft child \(U\) into socket \(a\) of \(W\) |
| \(I_W,O_W\) | workflow entry and exit sets |
| \(\mathcal{Q}(W)\) | workflow obligations |
| \(G_{\mu}\) | micro-scale search graph |
| \(G_M\) | manufacturing history graph |

## A.4 Semantic and manufacturing notation

| Symbol | Meaning |
|---|---|
| \(O\) | raw observation set |
| \(O^{*}\) | admitted RDF dataset |
| \(L\) | admitted law and entailment profile |
| \(\Pi\) | planning task or plan context, as scoped |
| \(W\) | POWL workflow geometry |
| \(p\) | plan-bound permission |
| \(A\) | artifact or consequence |
| \(E\) | evidence |
| \(R\) | receipt |
| \(\mu\) | manufacturing operator |
| \(\chi\) | semantic contraction |
| \(\kappa\) | canonicalization |
| \(\delta\) | transition function |
| \(\gamma\) | modeled STRIPS transition |
| \(\operatorname{MinSupp}(y)\) | minimal supports of \(y\) |

## A.5 Multifractal notation

| Symbol | Meaning |
|---|---|
| \(d(x,y)\) | metric distance |
| \(B(x,r)\) | open metric ball |
| \(\mu\) | measure |
| \(\mathcal{H}^{s}\) | \(s\)-dimensional Hausdorff measure |
| \(\dim_H\) | Hausdorff dimension |
| \(\alpha_{\mu}(x)\) | local dimension |
| \(p_i(\varepsilon)\) | mass in scale-\(\varepsilon\) region |
| \(Z(q,\varepsilon)\) | partition function |
| \(\tau(q)\) | mass exponent |
| \(D_q\) | generalized dimension |
| \(f(\alpha)\) | singularity spectrum |
| \(F_q(s)\) | MF-DFA fluctuation function |
| \(h(q)\) | generalized Hurst exponent |

## A.6 Standing vocabulary

| Standing | Exact use |
|---|---|
| UNKNOWN | evidence does not decide |
| PLANNED | required target without implementation evidence |
| PARTIAL_ALIVE | a real slice exists, but stated scope is incomplete |
| ALIVE | exact scoped claim has required evidence |
| REFUSED | an admitted request failed a typed gate |
| INCONSISTENT | authoritative evidence conflicts |

---

# Appendix B. Reference Algorithms

## B.1 Recursive manufacturing driver

The following pseudocode is normative at the level of control flow, not at the level of a particular
programming language.

    Manufacture(observations O, goal g, permissionSurface P, bounds K):
        admission := Admit(O)
        match admission:
            Refused(r): return Refused(r)
            Inconsistent(k): return Inconsistent(k)
            Accepted(O*): continue

        slice := SemanticContract(O*, g)
        planning := Plan(slice, g, K.search)

        match planning:
            Exhausted(x): return Exhausted(x)
            Bounded(f): return Bounded(f)
            Unsupported(u): return Unsupported(u)
            Inconsistent(k): return Inconsistent(k)
            Found(plan): continue

        verification := VerifyPlan(slice, plan, g)
        if verification is Invalid:
            return Inconsistent(verification)

        workflow := ProjectPOWL(plan, slice)
        geometryCheck := VerifyPOWL(workflow)
        if geometryCheck is not Valid:
            return Refused(geometryCheck)

        permission := AskUser(workflow, P, K)
        if permission is not Granted:
            return Refused(permission)

        return ExecuteRecursively(O*, workflow, permission, K)

## B.2 Recursive execution driver

    ExecuteRecursively(O*, W, p, K):
        state := InitialExecutionState(O*, W, p)

        while not Terminal(state):
            frontier := EnabledFrontier(W, state)

            if frontier is empty and obligations remain:
                return Inconsistent(DeadlockReceipt(state))

            for each compatible activity a in SelectedAntichain(frontier):
                if not PermissionApplies(p, a, state):
                    return Refused(PermissionMismatch(a))

                preReceipt := DurablePrepare(a, state, p)
                result := BrokerDispatch(a, preReceipt)

                match result:
                    UnknownAfterDispatch(k):
                        state := Reconcile(k, preReceipt, state)
                    Observed(e):
                        postReceipt := DurableComplete(preReceipt, e)
                        admittedResult := Admit(state.observations union {e})

                        match admittedResult:
                            Accepted(Onew*):
                                state := Advance(state, a, Onew*, postReceipt)
                            Refused(residue):
                                if K.descent equals 0:
                                    return Bounded(DescentExhausted(residue))

                                childGoal := ExtractOrProposeContinuation(residue)
                                childPlan := Manufacture(
                                    OnewOrPriorObservations,
                                    childGoal,
                                    RestrictPermission(p, a),
                                    Decrement(K)
                                )

                                match childPlan:
                                    Found(childWorkflowAndEvidence):
                                        W := Graft(W, a, childWorkflowAndEvidence.workflow)
                                    other:
                                        return other
                            Inconsistent(k):
                                return Inconsistent(k)

        receipt := SealRun(state)
        replay := Replay(receipt)
        return TerminalOutcome(state, receipt, replay)

## B.3 Exact bounded breadth-first planner

    Plan(task Π, bounds K):
        queue := [Π.initial]
        visited := {Π.initial}
        predecessor := empty map

        while queue is not empty:
            if BoundReached(K):
                return Bounded(visited, queue, K)

            s := RemoveFront(queue)

            if GoalHolds(Π, s):
                plan := Reconstruct(predecessor, s)
                if VerifyPlan(Π, plan) is Valid:
                    return Found(plan)
                else:
                    return Inconsistent(PlannerVerifierDisagreement)

            for each action a in Π.actions:
                if Applicable(a, s):
                    next := Transition(s, a)
                    if next not in visited:
                        visited := visited union {next}
                        predecessor[next] := (s, a)
                        Append(queue, next)

        return Exhausted(visited)

## B.4 Minimal-support enumeration for a finite truth table

This exponential algorithm is a specification oracle for small finite models.

    MinimalSupports(function f over finite inputs X):
        supports := empty set

        for each subset S of X in increasing cardinality:
            if any known support is a subset of S:
                continue

            determined := true
            for each admitted assignment pair (u, v):
                if Restrict(u, S) equals Restrict(v, S)
                   and f(u) does not equal f(v):
                    determined := false
                    break

            if determined:
                supports := supports union {S}

        return supports

The algorithm terminates because \(\mathcal{P}(X)\) and the admitted assignment set are finite. Its
worst-case cost is exponential or worse in \(|X|\), so production systems use symbolic methods,
declared dependency proofs, or generated theorem artifacts.

## B.5 Triple8 admission

    Admit8(profileHash h, metadata m, subject s, predicate p, object o, stateMask M):
        if h does not equal ActiveProfileHash:
            return Inconsistent(ProfileMismatch)

        entry := Admission8[p]

        if (M bitwise-and entry.requiredMask) does not equal entry.requiredMask:
            return Refused(MissingRequiredState)

        if (M bitwise-and entry.forbiddenMask) does not equal zero:
            return Refused(ForbiddenStatePresent)

        key := Pack(m, s, p, o)
        return Found(entry.transition, key)

## B.6 Receipt verification

    VerifyReceipt(R):
        verify schema and required identities
        verify every native payload digest
        recanonicalize every RDF graph under R.profile
        verify graph and artifact digests
        verify plan witness or truthful non-Found outcome
        verify permission applies to every broker event
        verify pre-receipt precedes every dispatch
        verify post-receipt exists for every Completed event
        verify no lifecycle orphan exists
        verify hash-chain linkage
        verify signature or attestation when required
        replay the declared transition semantics
        compare result under the declared byte or semantic criterion
        return Valid only if every mandatory check succeeds

---

# Appendix C. RDF Lifecycle Model and Constraint Sketches

## C.1 Namespace policy

The following compact prefixes are illustrative. Full IRIs, ontology versions, and graph digests
belong in the real pack.

    @prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
    @prefix prov: <http://www.w3.org/ns/prov#> .
    @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
    @prefix dcat: <http://www.w3.org/ns/dcat#> .
    @prefix dct:  <http://purl.org/dc/terms/> .
    @prefix dqv:  <http://www.w3.org/ns/dqv#> .
    @prefix sh:   <http://www.w3.org/ns/shacl#> .
    @prefix skos: <http://www.w3.org/2004/02/skos/core#> .
    @prefix spdx: <http://spdx.org/rdf/terms#> .
    @prefix mfw:  <https://mfact.dev/ns/mfw#> .

The mfw namespace is an engine ABI. It must not duplicate a public term merely to make code
generation convenient.

## C.2 Intent

    <urn:mfw:run:example>
        a mfw:ManufacturingRun, prov:Activity ;
        mfw:hasIntent <urn:mfw:intent:dry-run-publish> ;
        mfw:standing mfw:Planned ;
        prov:startedAtTime "2026-07-13T00:00:00Z"^^xsd:dateTime .

    <urn:mfw:intent:dry-run-publish>
        a prov:Entity ;
        dct:title "Dry-run publish this Rust workspace" ;
        mfw:goalType mfw:RustDryRunPublish ;
        mfw:prohibits mfw:RegistryUpload ;
        mfw:subject <urn:mfw:repo:snapshot:abc> .

## C.3 Content-addressed payload

    <urn:blake3:payload-digest>
        a prov:Entity, dcat:Distribution ;
        dct:format "text/plain" ;
        dcat:byteSize "2417"^^xsd:nonNegativeInteger ;
        spdx:checksum [
            a spdx:Checksum ;
            spdx:algorithm spdx:checksumAlgorithm_blake3 ;
            spdx:checksumValue "payload-digest"
        ] ;
        prov:wasGeneratedBy <urn:mfw:event:tool-result:42> .

The blank checksum node is permissible only if it never needs independent durable identity. If it
must be individually receipted or updated, mint an IRI.

## C.4 Claim

    <urn:mfw:claim:C8>
        a mfw:Claim ;
        mfw:subject <urn:mfw:repo:snapshot:abc> ;
        mfw:proposition mfw:WholeWorkspaceDryRunSucceeds ;
        mfw:standing mfw:Refused ;
        mfw:withinBoundary <urn:mfw:boundary:release-v26.7.13> ;
        prov:wasDerivedFrom <urn:mfw:refusal:path-dependencies> ;
        mfw:promotionTest <urn:mfw:test:whole-workspace-rerun> .

## C.5 Research task and agent invocation

    <urn:mfw:task:inspect-manifests>
        a mfw:ResearchTask, prov:Activity ;
        dct:description "Find package metadata and path-dependency blockers" ;
        mfw:partOfPlan <urn:mfw:plan:dry-run> ;
        mfw:allowedToolClass mfw:ReadOnlyRepositoryTool ;
        mfw:maxDurationSeconds "300"^^xsd:positiveInteger .

    <urn:mfw:agent:claude-code:invocation:7>
        a prov:Activity, mfw:AgentInvocation ;
        prov:used <urn:mfw:repo:snapshot:abc> ;
        prov:wasAssociatedWith <urn:mfw:agent:claude-code> ;
        mfw:executesTask <urn:mfw:task:inspect-manifests> ;
        mfw:permission <urn:mfw:permission:reconnaissance> .

## C.6 Tool intent and result

    <urn:mfw:event:tool-intent:42>
        a mfw:ToolIntent, prov:Activity ;
        mfw:toolClass mfw:FileSearch ;
        mfw:argumentsDigest <urn:blake3:args-digest> ;
        mfw:planStep <urn:mfw:step:inspect-manifests> ;
        mfw:authorizedBy <urn:mfw:permission:reconnaissance> .

    <urn:mfw:event:tool-result:42>
        a mfw:ToolResult, prov:Activity ;
        mfw:respondsTo <urn:mfw:event:tool-intent:42> ;
        mfw:outcome mfw:Found ;
        prov:generated <urn:blake3:payload-digest> ;
        mfw:partOfRun <urn:mfw:run:example> .

## C.7 Plan-bound permission

    <urn:mfw:permission:repair-1>
        a odrl:Policy, mfw:PlanPermission ;
        odrl:permission [
            odrl:action mfw:EditFile ;
            odrl:target <urn:mfw:pathset:approved-repair-files>
        ] ;
        odrl:prohibition [
            odrl:action mfw:RegistryUpload
        ] ;
        mfw:planDigest "plan-digest" ;
        mfw:workflowDigest "workflow-digest" ;
        mfw:expiresAt "2026-07-13T23:59:59Z"^^xsd:dateTime ;
        prov:wasAttributedTo <urn:mfw:user:grantor> .

In production, permission and prohibition nodes requiring durable cross-graph identity receive
IRIs.

## C.8 Pre- and post-receipts

    <urn:mfw:receipt:pre:99>
        a mfw:ActuationIntentReceipt ;
        mfw:forPlanStep <urn:mfw:step:cargo-package> ;
        mfw:authorizedBy <urn:mfw:permission:repair-1> ;
        mfw:idempotencyKey "run-step-attempt-99" ;
        mfw:expectedEffect mfw:CreatePackageArchive ;
        prov:generatedAtTime "2026-07-13T12:00:00Z"^^xsd:dateTime .

    <urn:mfw:receipt:post:99>
        a mfw:ActuationOutcomeReceipt ;
        prov:wasDerivedFrom <urn:mfw:receipt:pre:99> ;
        mfw:observedOutcome mfw:Refused ;
        mfw:evidence <urn:blake3:cargo-output-digest> ;
        prov:generatedAtTime "2026-07-13T12:00:08Z"^^xsd:dateTime .

## C.9 SHACL claim shape

    mfw:ClaimShape
        a sh:NodeShape ;
        sh:targetClass mfw:Claim ;
        sh:property [
            sh:path mfw:subject ;
            sh:minCount 1 ;
            sh:maxCount 1 ;
            sh:nodeKind sh:IRI
        ] ;
        sh:property [
            sh:path mfw:standing ;
            sh:minCount 1 ;
            sh:maxCount 1 ;
            sh:in (
                mfw:Unknown
                mfw:Planned
                mfw:PartialAlive
                mfw:Alive
                mfw:Refused
                mfw:Inconsistent
            )
        ] ;
        sh:property [
            sh:path mfw:promotionTest ;
            sh:minCount 1
        ] .

## C.10 SHACL permission shape

    mfw:PlanPermissionShape
        a sh:NodeShape ;
        sh:targetClass mfw:PlanPermission ;
        sh:property [
            sh:path mfw:planDigest ;
            sh:minCount 1 ;
            sh:maxCount 1 ;
            sh:datatype xsd:string
        ] ;
        sh:property [
            sh:path mfw:workflowDigest ;
            sh:minCount 1 ;
            sh:maxCount 1 ;
            sh:datatype xsd:string
        ] ;
        sh:property [
            sh:path mfw:expiresAt ;
            sh:minCount 1 ;
            sh:maxCount 1 ;
            sh:datatype xsd:dateTime
        ] .

Temporal validity still requires a comparison with admitted current time; datatype conformance alone
does not prove the permission is unexpired.

## C.11 SHACL no-orphan sketch

A complete no-orphan check can use SHACL-SPARQL or a compiled verifier. The logical condition for a
ToolResult is:

    mfw:ToolResultShape
        a sh:NodeShape ;
        sh:targetClass mfw:ToolResult ;
        sh:property [
            sh:path mfw:respondsTo ;
            sh:minCount 1 ;
            sh:maxCount 1 ;
            sh:class mfw:ToolIntent
        ] ;
        sh:property [
            sh:path mfw:partOfRun ;
            sh:minCount 1 ;
            sh:maxCount 1 ;
            sh:class mfw:ManufacturingRun
        ] .

Cross-node equality, plan-step consistency, and digest identity require additional constraints beyond
this minimal sketch.

---

# Appendix D. Lean-Style Formalization Sketches

The declarations in this appendix are explanatory candidates. They are not labeled kernel checked
unless the exact source appears in the recorded theorem inventory and its build receipt is verified.

## D.1 Outcome non-collapse

    inductive Outcome (Found Exhaust Frontier Unsupported Conflict : Type)
      | found        : Found       → Outcome Found Exhaust Frontier Unsupported Conflict
      | exhausted    : Exhaust     → Outcome Found Exhaust Frontier Unsupported Conflict
      | bounded      : Frontier    → Outcome Found Exhaust Frontier Unsupported Conflict
      | unsupported  : Unsupported → Outcome Found Exhaust Frontier Unsupported Conflict
      | inconsistent : Conflict    → Outcome Found Exhaust Frontier Unsupported Conflict

    theorem bounded_ne_exhausted
        (f : Frontier)
        (x : Exhaust) :
        Outcome.bounded f ≠ Outcome.exhausted x := by
      intro h
      cases h

Constructor disjointness proves the result.

## D.2 Descent termination

    def grow : (fuel : Nat) → Residue → Outcome Workflow Exhaust Frontier Unsupported Conflict
      | 0, residue =>
          .bounded (Frontier.descentExhausted residue)
      | fuel + 1, residue =>
          match resolve residue with
          | .complete workflow => .found workflow
          | .continue childResidue => grow fuel childResidue

Lean's termination checker sees the recursive argument decrease from fuel plus one to fuel.

## D.3 Minimal support

    def IsSupport
        (f : Assignment → Output)
        (S : Finset Input) : Prop :=
      ∀ u v,
        agreeOn S u v →
        f u = f v

    def IsMinimalSupport
        (f : Assignment → Output)
        (S : Finset Input) : Prop :=
      IsSupport f S ∧
      ∀ T, T ⊂ S → ¬ IsSupport f T

    theorem minimal_supports_antichain
        (hS : IsMinimalSupport f S)
        (hT : IsMinimalSupport f T)
        (hne : S ≠ T) :
        ¬ S ⊆ T ∧ ¬ T ⊆ S := by
      ...

The omitted proof must be completed; ellipsis is explicit non-proof.

## D.4 Receipted state

    structure PreReceipt where
      planDigest : Digest
      permissionDigest : Digest
      idempotencyKey : Key

    structure PostReceipt where
      preDigest : Digest
      resultDigest : Digest

    inductive ExecState
      | proposed : Plan → ExecState
      | authorized : Plan → Permission → ExecState
      | prepared : Plan → Permission → PreReceipt → ExecState
      | completed :
          Plan → Permission → PreReceipt → PostReceipt → ExecState

    def IsCompleted : ExecState → Prop
      | .completed _ _ _ _ => True
      | _ => False

    theorem completed_has_receipts
        (s : ExecState)
        (h : IsCompleted s) :
        ∃ pre post p perm,
          s = .completed p perm pre post := by
      cases s <;> simp [IsCompleted] at h ⊢

The concrete-system theorem additionally requires the broker-only correspondence.

## D.5 Commutation

    def Commute (a b : State → State) : Prop :=
      ∀ s, a (b s) = b (a s)

    theorem swap_preserves
        (h : Commute a b) :
        (a ∘ b) = (b ∘ a) := by
      funext s
      exact h s

## D.6 Graft termination versus graft correctness

Lean can accept a fuel-decreasing recursive function even if the graft construction violates POWL
semantics. Two theorem families are needed:

1. termination of recursion over fuel; and
2. preservation of acyclicity, interface, obligations, and trace criteria.

Conflating them would be a formal version of rounding.

---

# Appendix E. Construct-to-Reality Traceability

| Mathematical construct | Runtime object | Verification |
|---|---|---|
| finite set of RDF triples | Oxigraph named graph | RDF parser, canonical digest |
| admission predicate | SHACL/GraphLaw pack | conformance and conflict report |
| least fixed point | bounded Datalog closure | closure trace and finite-universe proof |
| finite fluent set | Pddl8 tape or grounded task | grounding receipt |
| tagged outcome sum | Rust/Erlang result type | constructor and serialization tests |
| strict partial order | POWL precedence graph | acyclicity and transitive-order verifier |
| antichain | enabled compatible activity set | scheduler trace |
| graft substitution | graft_child and F09-to-F10 edge | interface and same-object growth test |
| descent on \(\mathbb{N}\) | DescentMeter | bound-exhaustion negative fixture |
| permission predicate | compiled ODRL/broker policy | stale/out-of-scope refusal fixtures |
| pre/post receipt types | durable receipt records | crash-window chaos tests |
| LTS simulation | AIR/runtime mapping | theorem or bounded differential corpus |
| canonicalization | canonical RDF bytes | cross-serializer test vectors |
| deterministic replay | transition fold | independent replay |
| minimal support | mfact theorem objects | kernel inventory and correspondence audit |
| Triple8 injective symbol table | ProfileSymbolTable | overflow and cross-profile refusal |
| partition function | process-analysis artifact | synthetic cascade calibration |
| gradient/leverage | roadmap scoring projection | sensitivity and same-object execution |
| ephemeralization | controlled capability/resource ledger | repeated receipted measurements |

---


# Appendix F. Position Relative to Adjacent Research

## F.1 Automated planning

STRIPS and PDDL describe possible states, actions, preconditions, and effects so a planner can search
for a plan. MFW preserves that role and adds four boundaries around it:

1. RDF observation and admission manufacture the planning instance;
2. truthful result constructors distinguish exact exhaustion from bounded search;
3. POWL turns a plan witness into richer process geometry; and
4. the broker prevents modeled effects from impersonating real effects.

The contribution is not a new claim that finite planning exists. It is the standing-preserving
composition from admitted semantic state through planning to receipted consequence.

## F.2 Business process management and workflow nets

Workflow nets, process trees, BPMN, and POWL address process representation, discovery, soundness,
and execution structure. POWL is especially relevant because partial orders avoid arbitrary
linearization of concurrency, while POWL v2 choice graphs increase expressiveness for
non-block-structured decisions and cycles.

MFW treats POWL as process-geometry authority but adds runtime-manufacturing concerns:

- a workflow may be generated from a bounded planner;
- execution residue can manufacture and graft a child;
- permission and evidence obligations are part of the socket;
- modeled workflow and manufacturing history are separate graphs; and
- topology projects across an explicit external cut.

Published POWL transformation theorems apply to their stated workflow-net classes. MFW's recursive
graft and heterogeneous runtime claims require their own proofs.

## F.3 Semantic Web and knowledge graphs

RDF provides a graph abstract data model; RDF datasets preserve named contexts; SHACL validates
graph constraints; PROV-O represents provenance; ODRL represents permissions and prohibitions.
Knowledge-graph systems often use these technologies for integration, query, or reasoning.

MFW's additional commitment is operational sovereignty:

\[
\text{RDF admitted instance state}
\to
\text{planning}
\to
\text{workflow}
\to
\text{permission}
\to
\text{actuation}
\to
\text{receipt}.
\]

The graph is not merely metadata exported after execution. It exists before and after every
authoritative transition. Native payloads remain content-addressed rather than being forced into
triples.

## F.4 Deductive databases and rule systems

Datalog fixed-point theory supplies finite monotone closure. SPARQL CONSTRUCT supplies graph
projection and materialization. N3 can express contextual rules and built-ins.

MFW's semantic-contraction doctrine orders these by expressive power:

\[
\text{direct public triples}
\prec
\text{bounded CONSTRUCT}
\prec
\text{stratified Datalog}
\prec
\text{permissioned N3}.
\]

The order is an engineering preference, not a claim that one language universally subsumes the
others in every semantic detail. The purpose is to minimize the state and authority exposed to the
planner and executor.

## F.5 Event sourcing and process mining

Event-sourced systems reconstruct state from events. Process mining discovers and evaluates
processes from event logs. OCEL represents events related to multiple objects.

MFW adds:

- pre-actuation intent receipts;
- plan- and permission-bound event identity;
- RDF as the authority from which OCEL is projected;
- explicit replay criteria;
- no-orphan constraints; and
- a feedback boundary through which measured findings must be re-admitted before changing law.

## F.6 Formal methods

Refinement, simulation, model checking, dependent types, and proof assistants can establish
properties of models and programs. Lean's kernel checks proof terms in dependent type theory.

mfact's role is manufacturing and certifying proof-bearing law artifacts. MFW deliberately separates:

\[
\text{formal validity},
\quad
\text{implementation correspondence},
\quad
\text{historical occurrence}.
\]

Many systems blur the last two by treating tests as proofs or the first and third by treating a
theorem as an audit log. This dissertation makes their product structure explicit.

## F.7 Agentic coding systems

Coding agents can inspect repositories, edit files, run tests, and coordinate subtasks. Their common
control unit is a conversational or task loop. Operation Dogfood moves the lifecycle authority
outside the agent:

- MFW admits context;
- MFW manufactures plan geometry;
- the user grants permission;
- Claude Code performs bounded cognitive and implementation work;
- MFW re-admits results;
- receipts and replay determine standing.

The agent is neither oracle nor sovereign. This permits strong capability without invisible
authority expansion.

## F.8 Build systems and release automation

Build systems represent dependency graphs and incremental regeneration. CI systems execute
prewritten jobs. Cargo packages and dry-run publishes Rust crates.

MFW differs at the workflow-discovery boundary. It is expected to recover repository-specific laws
from admitted evidence, manufacture the release workflow, recursively repair genuine blockers, and
preserve a typed refusal when publication is impossible. It does not replace Cargo's package
semantics; it governs the process that invokes and interprets them.

## F.9 Fractal and multifractal analysis

Multifractal formalism characterizes nonuniform measures using local dimensions, moment scaling,
generalized dimensions, and singularity spectra. MF-DFA estimates multifractal behavior in
nonstationary time series.

MFW contributes no license to call hierarchy a multifractal. It defines:

1. an exact recursive workflow law; and
2. an independent empirical measurement program.

Synthetic cascades validate analysis code. Workflow logs test the empirical hypothesis. Failure of
the hypothesis does not invalidate recursive manufacture.

## F.10 Comprehensive anticipatory design science

The Buckminster Fuller Institute characterizes design science as a comprehensive, anticipatory,
systematic approach to world problems and documents the World Game, Synergetics, and Geoscope.

MFW converts this canon into:

- an explicit controlled boundary;
- whole-system admitted state;
- bounded future search;
- capability/resource frontiers;
- minimal-support trimtab interventions;
- synergetic capability tests;
- closed-loop receipt accounting; and
- a Geoscope-like projection separating observation, simulation, permission, and consequence.

The system does not use adoption as evidence because adoption is outside its causal control.

---

# Appendix G. Proof-Obligation Ledger

## G.1 Admission obligations

1. Define every authoritative source class and predicate-scoped precedence rule.
2. Prove or test bounded rule closure termination.
3. Verify selected SHACL shapes over every admitted graph.
4. Refuse unresolved conflict rather than select by insertion order.
5. Bind native payload bytes to RDF identity.
6. Preserve ontology and namespace versions.
7. Prove that owned instances never rely on ephemeral blank-node identity.

## G.2 Planning obligations

1. Record exact fluent and action universe.
2. Record open-world-to-closed-world assumptions.
3. Verify every Found plan independently.
4. Return Exhausted only after frontier empties.
5. Preserve the frontier for Bounded.
6. Keep Unsupported and Inconsistent constructors distinct.
7. Prevent modeled effects from becoming admitted observations without harness evidence.

## G.3 POWL obligations

1. Verify unique activity identities.
2. Verify strict causal order.
3. Verify every selected activity lies on an entry-to-exit path.
4. Verify choice-graph references and guard semantics.
5. Verify compatible antichains before concurrent dispatch.
6. Verify external cuts come from admitted authority.
7. Verify every socket's precondition, postcondition, authority, mutation, evidence, and receipt
   interface.

## G.4 Recursive obligations

1. Extract or validate a continuation goal from real residue.
2. Decrement an explicit well-founded measure.
3. Restrict child permission.
4. Preserve parent obligations.
5. Prove graft acyclicity and context preservation.
6. Re-admit child output before parent closure.
7. Return Bounded on descent exhaustion.

## G.5 Execution obligations

1. Enforce broker-only side effects inside scope.
2. Persist pre-receipt before dispatch.
3. Persist post-receipt before Completed.
4. Represent crash ambiguity as UnknownAfterDispatch.
5. Use idempotency or disclose duplicate risk.
6. Request new permission on material scope change.
7. Represent human and machine actions with the same evidentiary discipline.

## G.6 Runtime obligations

1. Define abstract and concrete state relations.
2. Define event-label abstraction.
3. Prove or test each supported simulation edge.
4. Refuse unsupported AtomVM or WASM features.
5. Preserve stable workflow identity across OTP restart.
6. Keep PIDs and semantic identity distinct.
7. Verify Arazzo artifact and originating POWL external slice.

## G.7 Receipt and replay obligations

1. Name canonicalization and hash profile.
2. Verify every payload and graph digest.
3. Verify hash-chain order.
4. Verify plan and permission identity for every effect.
5. Verify zero lifecycle orphans.
6. Record exogenous nondeterministic inputs.
7. State byte versus semantic replay.
8. Run independent replay.

## G.8 Mathematical obligations

1. Keep structural recursion separate from empirical multifractality.
2. State measure, metric, partition, scale interval, and \(q\)-range.
3. Propagate uncertainty through \(\tau\), \(D_q\), and \(f(\alpha)\).
4. Use surrogate tests.
5. Distinguish dependency reachability, possible residue, actual residue, and minimal schedule.
6. Define all capability and resource weights.
7. Perform sensitivity analysis.
8. Keep physical units distinct from process analogies.

## G.9 Formal proof obligations

1. Pin Lean, dependencies, axioms, and source digests.
2. Verify theorem inventory rather than theorem count alone.
3. Inspect the English-to-formal correspondence.
4. Build candidate POWL and multifractal modules through the declared native path.
5. Connect production graft and DescentMeter implementations if implementation-level claims are
   made.
6. State whether mfact claim scope stops at certified law or extends to deployed runtime.

## G.10 Release obligations

1. Resolve namespace collisions.
2. Regenerate standing ledger from source.
3. Eliminate stale true markers.
4. Preserve whole-workspace dry-run refusal until a real rerun passes.
5. Require both literal-prefix crowns before contiguity is true.
6. Keep external registry upload prohibited for this dry-run release.

---

# Appendix H. Bibliography and Primary Standards

## H.1 Semantic Web standards

**[RDF12]** World Wide Web Consortium. *RDF 1.2 Concepts and Abstract Data Model*, Candidate
Recommendation Snapshot, 7 April 2026. The standard defines RDF graphs as sets of
subject-predicate-object triples and datasets as a default graph plus named graphs.
<https://www.w3.org/TR/rdf12-concepts/>

**[RDF12-SEM]** World Wide Web Consortium. *RDF 1.2 Semantics*. This is the model-theoretic
semantics referenced by the RDF 1.2 concepts document.
<https://www.w3.org/TR/rdf12-semantics/>

**[SHACL]** World Wide Web Consortium. *Shapes Constraint Language (SHACL)*, W3C Recommendation.
<https://www.w3.org/TR/shacl/>

**[PROV-O]** World Wide Web Consortium. *PROV-O: The PROV Ontology*, W3C Recommendation.
<https://www.w3.org/TR/prov-o/>

**[ODRL]** World Wide Web Consortium. *ODRL Information Model 2.2*, W3C Recommendation.
<https://www.w3.org/TR/odrl-model/>

**[SPARQL]** World Wide Web Consortium. *SPARQL 1.1 Query Language*, W3C Recommendation.
<https://www.w3.org/TR/sparql11-query/>

**[N3]** W3C Notation 3 Community Group. *Notation3 Language Specification*.
<https://w3c-cg.github.io/N3/spec/>

**[DCAT]** World Wide Web Consortium. *Data Catalog Vocabulary (DCAT), Version 3*.
<https://www.w3.org/TR/vocab-dcat-3/>

**[SOSA-SSN]** World Wide Web Consortium and Open Geospatial Consortium. *Semantic Sensor Network
Ontology*.
<https://www.w3.org/TR/vocab-ssn/>

**[DQV]** World Wide Web Consortium. *Data on the Web Best Practices: Data Quality Vocabulary*.
<https://www.w3.org/TR/vocab-dqv/>

## H.2 Planning

**[STRIPS]** Richard E. Fikes and Nils J. Nilsson. “STRIPS: A New Approach to the Application of
Theorem Proving to Problem Solving.” *Artificial Intelligence* 2, no. 3–4 (1971): 189–208.
<https://doi.org/10.1016/0004-3702(71)90010-5>

**[PDDL]** Drew McDermott et al. *PDDL—The Planning Domain Definition Language*. Technical Report
CVC TR-98-003/DCS TR-1165, Yale Center for Computational Vision and Control, 1998.
<https://www.cs.yale.edu/homes/dvm/papers/pddl.pdf>

**[PDDL2.1]** Maria Fox and Derek Long. “PDDL2.1: An Extension to PDDL for Expressing Temporal
Planning Domains.” *Journal of Artificial Intelligence Research* 20 (2003): 61–124.
<https://doi.org/10.1613/jair.1129>

## H.3 POWL and process geometry

**[POWL-WFNET]** Humam Kourani, Gyunam Park, and Wil M. P. van der Aalst. “Translating Workflow
Nets into the Partially Ordered Workflow Language.” arXiv:2503.20363, 2025.
<https://arxiv.org/abs/2503.20363>

**[POWL-CG]** Humam Kourani, Gyunam Park, and Wil M. P. van der Aalst. “Unlocking
Non-Block-Structured Decisions: Inductive Mining with Choice Graphs.” arXiv:2505.07052, 2025.
<https://arxiv.org/abs/2505.07052>

**[POWL-HD]** Humam Kourani, Gyunam Park, and Wil M. P. van der Aalst. “Hierarchical Decomposition
of Separable Workflow-Nets.” arXiv:2602.15739, 2026.
<https://arxiv.org/abs/2602.15739>

**[PO-CONCURRENCY]** Humam Kourani, Gyunam Park, and Wil M. P. van der Aalst. “Revealing Inherent
Concurrency in Event Data: A Partial Order Approach to Process Discovery.” arXiv:2509.15346, 2025.
<https://arxiv.org/abs/2509.15346>

The arXiv papers are cited as primary research records. Their exact version should be pinned in a
formal release bibliography.

## H.4 Inter-engine workflow and runtime standards

**[ARAZZO]** OpenAPI Initiative. *The Arazzo Specification*, version 1.1.0, 17 May 2026.
<https://spec.openapis.org/arazzo/latest.html>

**[WASM]** World Wide Web Consortium. *WebAssembly Core Specification*.
<https://www.w3.org/TR/wasm-core-2/>

**[ERLANG]** Ericsson and the Erlang/OTP project. *Erlang/OTP System Documentation*.
<https://www.erlang.org/doc/system/>

**[ATOMVM]** AtomVM Project. *AtomVM Documentation*.
<https://doc.atomvm.org/>

## H.5 Object-centric event logs and observability

**[OCEL2]** Object-Centric Event Log standard. *OCEL 2.0 Specification*.
<https://www.ocel-standard.org/2.0/>

**[OTEL]** Cloud Native Computing Foundation. *OpenTelemetry Specification*.
<https://opentelemetry.io/docs/specs/otel/>

## H.6 Formal proof

**[LEAN]** Lean Project. *The Lean Language Reference*. The cited edition identifies Lean as an
interactive theorem prover based on dependent type theory and describes a minimal kernel that checks
proof terms.
<https://lean-lang.org/doc/reference/latest/>

**[MATHLIB]** mathlib Community. *Mathlib Documentation*.
<https://leanprover-community.github.io/mathlib4_docs/>

The exact commit and imported axiom inventory, not the latest website alone, determine a build's
formal environment.

## H.7 Multifractal analysis

**[HALSEY]** Thomas C. Halsey, Mogens H. Jensen, Leo P. Kadanoff, Itamar Procaccia, and Boris I.
Shraiman. “Fractal Measures and Their Singularities: The Characterization of Strange Sets.”
*Physical Review A* 33, no. 2 (1986): 1141–1151.
<https://doi.org/10.1103/PhysRevA.33.1141>

**[MFDFA]** Jan W. Kantelhardt, Stephan A. Zschiegner, Eva Koscielny-Bunde, Armin Bunde, Shlomo
Havlin, and H. Eugene Stanley. “Multifractal Detrended Fluctuation Analysis of Nonstationary Time
Series.” *Physica A* 316 (2002): 87–114. Primary manuscript:
<https://arxiv.org/abs/physics/0202070>

**[DFA]** Jan W. Kantelhardt et al. “Detecting Long-Range Correlations with Detrended Fluctuation
Analysis.” *Physica A* 295 (2001): 441–454. Primary manuscript:
<https://arxiv.org/abs/cond-mat/0102214>

**[OLSEN]** Lars Olsen. “A Multifractal Formalism.” *Advances in Mathematics* 116, no. 1 (1995):
82–196.
<https://doi.org/10.1006/aima.1995.1066>

## H.8 Rust publication

**[CARGO-PUBLISH]** Rust Project. *The Cargo Book: cargo publish*. The official command
documentation defines dry-run as performing checks without uploading and locked mode as refusing
dependency changes relative to Cargo.lock.
<https://doc.rust-lang.org/cargo/commands/cargo-publish.html>

**[CARGO-PACKAGE]** Rust Project. *The Cargo Book: cargo package*.
<https://doc.rust-lang.org/cargo/commands/cargo-package.html>

## H.9 Enterprise architecture and governance

**[TOGAF]** The Open Group. *TOGAF Standard*.
<https://pubs.opengroup.org/togaf-standard/>

**[ROSS]** Jeanne W. Ross, Peter Weill, and David C. Robertson. *Enterprise Architecture as
Strategy*. Harvard Business School Press, 2006. This is a published book rather than a web standard;
case-study claims in MFW must still name their implemented fixtures.

## H.10 Buckminster Fuller canon

**[BFI-DS]** Buckminster Fuller Institute. “Design Science.”
<https://www.bfi.org/about-fuller/big-ideas/design-science/>

**[BFI-WG]** Buckminster Fuller Institute. “World Game.”
<https://www.bfi.org/about-fuller/big-ideas/world-game/>

**[BFI-SYN]** Buckminster Fuller Institute. “Synergetics.”
<https://www.bfi.org/about-fuller/big-ideas/synergetics/>

**[BFI-GEO]** Buckminster Fuller Institute. “Geoscope.”
<https://www.bfi.org/about-fuller/big-ideas/geoscope/>

These sources motivate the canon. The equations in Chapters 20 and 24 are this dissertation's
controlled-system formalization, not equations asserted by the Institute.

## H.11 Internal release sources

The local source bundle used for this edition contains:

- *Multifractal Workflow PhD Thesis v26.7.12*;
- *Operation Dogfood Manifesto*;
- *Operation Dogfood Product Requirements Document v26.7.13*;
- *Operation Dogfood Architecture Requirements Document v26.7.13*; and
- *MFW Vision 2030*.

Where those documents disagree with a current evidence record, this edition preserves the lower
standing and names the conflict.

---

# Appendix I. Defense Propositions

The following propositions summarize what may and may not be defended from this dissertation.

## I.1 Defensible mathematical propositions

1. A finite positive Datalog closure terminates under the stated finite-universe assumptions.
2. Exhaustive search of an exact finite STRIPS state graph can truthfully certify exhaustion.
3. A Bound result cannot equal an Exhausted result in a disjoint tagged sum.
4. Compatible grafting preserves acyclicity under the stated interface conditions.
5. Natural-number descent bounds recursive child depth.
6. Minimal supports form an antichain.
7. Disjoint complete supports imply noninterference for deterministic support-local outputs.
8. Disjoint read/write operations commute.
9. Swapping adjacent commuting operations preserves final state.
10. A Completed constructor containing receipts makes completed-without-receipt unconstructible in
    the abstract state type.
11. Deterministic replay from identical initial state and events yields identical final state.
12. Generalized dimensions and Legendre spectra follow the definitions and limits stated in Part
    VII when those limits and regularity conditions exist.

## I.2 Defensible architectural propositions

1. RDF can serve as authoritative lifecycle state while native bytes remain content-addressed
   payloads.
2. PDDL and POWL can model process structure without pretending modeled effects are real.
3. A broker can make permission and receipts unavoidable inside an enforced controlled boundary.
4. Claude Code can be treated as a bounded actuator and proposer rather than standing authority.
5. Search topology and manufacturing history require separate graphs and clocks.
6. Theorem, implementation correspondence, occurrence, and replay are distinct evidence axes.
7. Fuller's canon can be represented as controlled design-capacity objectives without using
   adoption as a variable.

## I.3 Claims not yet defensible as Alive

1. The entire Claude Code lifecycle is RDF-authoritative end to end in production.
2. An unfamiliar Rust repository can always be discovered without pack extension.
3. Whole-workspace v26.7.13 dry-run publication succeeds.
4. The local and external crowns both traverse a real same-object literal prefix.
5. Every current runtime is semantically equivalent.
6. Every minimal-regeneration claim computes actual instance-minimal residue.
7. Workflow event data has a stable empirical multifractal spectrum.
8. Every candidate Lean module has a native build receipt.

## I.4 The one-sentence thesis

> Multifractal Workflow is a public-semantic, bounded, permissioned, recursively graftable process
> manufacturing system in which every real consequence is admitted, receipted, and replayable, and
> every mathematical, implementation, and empirical claim is held to the exact standing its
> evidence supports.

---

# Appendix J. Fully Worked Dry-Run Derivation

This appendix constructs a small example from first principles. It is a mathematical and
architectural demonstration, not evidence that the production Operation Dogfood implementation has
completed the same path.

## J.1 Repository

Consider a workspace with two crates:

\[
C=\{c_{\mathrm{core}},c_{\mathrm{cli}}\}.
\]

The CLI crate depends on core. The observed CLI manifest contains a path dependency but no version:

    [dependencies]
    example-core = { path = "../example-core" }

The intended publishable form requires:

    [dependencies]
    example-core = { version = "1.0.0", path = "../example-core" }

The path remains useful inside the workspace; the version allows Cargo to resolve the registry
dependency when packaging.

## J.2 Raw observations

Let the raw observation set be:

\[
O=\{o_1,o_2,o_3,o_4,o_5\},
\]

where:

\[
\begin{aligned}
o_1&=\text{root Cargo.toml bytes},\\
o_2&=\text{core Cargo.toml bytes},\\
o_3&=\text{CLI Cargo.toml bytes},\\
o_4&=\text{Cargo.lock bytes},\\
o_5&=\text{user goal “dry-run publish the workspace”}.
\end{aligned}
\]

For each \(o_i\), compute digest:

\[
h_i=\operatorname{BLAKE3}(\operatorname{bytes}(o_i)).
\]

The digest is recorded as a finite hexadecimal string. The exact string is omitted from this
illustration; a real run cannot omit it.

## J.3 Candidate RDF

Mint stable IRIs:

\[
\begin{aligned}
r &= \text{urn:repo:example},\\
c_1 &= \text{urn:crate:example-core:1.0.0},\\
c_2 &= \text{urn:crate:example-cli:1.0.0},\\
d &= \text{urn:dependency:cli-to-core},\\
g &= \text{urn:goal:dry-run-publish}.
\end{aligned}
\]

Candidate triples include:

\[
(r,\operatorname{hasMember},c_1),
\]

\[
(r,\operatorname{hasMember},c_2),
\]

\[
(c_2,\operatorname{dependsThrough},d),
\]

\[
(d,\operatorname{dependencyTarget},c_1),
\]

\[
(d,\operatorname{pathValue},\text{“../example-core”}),
\]

\[
(d,\operatorname{versionValue},\operatorname{Missing}),
\]

\[
(g,\operatorname{prohibits},\operatorname{RegistryUpload}).
\]

Each triple is linked to a reified claim or provenance entity derived from the relevant manifest
payload.

## J.4 Admission

Assume the selected Rust publish shape requires every registry-intended normal dependency to have a
version. The dependency node violates:

\[
\operatorname{RegistryIntended}(d)
\Rightarrow
\exists!v,\operatorname{versionValue}(d,v).
\]

The observed graph represents absence explicitly as a candidate missing-metadata fact after a
closed manifest parse. Admission does not accept “publish-ready.” Instead it accepts the repository
facts and derives blocker:

\[
b=\operatorname{MissingDependencyVersion}(d).
\]

This distinction is important. The graph itself can be structurally admitted while the target
capability is refused.

The admitted state \(O^{*}\) contains:

\[
\operatorname{KnownWorkspace}(r),
\]

\[
\operatorname{KnownCrate}(c_1),
\qquad
\operatorname{KnownCrate}(c_2),
\]

\[
\operatorname{DependsOn}(c_2,c_1),
\]

\[
\operatorname{BlockedBy}(c_2,b).
\]

## J.5 Planning fluents

Choose finite fluent set:

\[
\begin{aligned}
F=\{&
\operatorname{observedCore},
\operatorname{observedCli},
\operatorname{lockVerified},\\
&\operatorname{coreMetadataValid},
\operatorname{cliMetadataValid},
\operatorname{corePackaged},
\operatorname{cliPackaged},\\
&\operatorname{coreCleanBuilt},
\operatorname{cliCleanBuilt},
\operatorname{coreDryRun},
\operatorname{cliDryRun},\\
&\operatorname{repairApproved}
\}.
\end{aligned}
\]

The initial state is:

\[
s_0=
\{
\operatorname{observedCore},
\operatorname{observedCli},
\operatorname{lockVerified},
\operatorname{coreMetadataValid}
\}.
\]

Notably:

\[
\operatorname{cliMetadataValid}\notin s_0.
\]

## J.6 Actions

Define action \(a_1=\operatorname{approveRepair}\):

\[
\operatorname{pre}^{+}(a_1)=\varnothing,
\]

\[
\operatorname{add}(a_1)=\{\operatorname{repairApproved}\}.
\]

This action is not performed automatically; its real implementation is the Ask boundary.

Define \(a_2=\operatorname{repairCliVersion}\):

\[
\operatorname{pre}^{+}(a_2)
=
\{
\operatorname{observedCli},
\operatorname{repairApproved}
\},
\]

\[
\operatorname{add}(a_2)
=
\{\operatorname{cliMetadataValid}\}.
\]

The modeled add effect predicts that a verified repair will make metadata valid. The real harness
must edit, reparse, and admit the result.

Define \(a_3=\operatorname{packageCore}\):

\[
\operatorname{pre}^{+}(a_3)
=
\{
\operatorname{coreMetadataValid},
\operatorname{lockVerified}
\},
\]

\[
\operatorname{add}(a_3)
=
\{\operatorname{corePackaged}\}.
\]

Define \(a_4=\operatorname{packageCli}\):

\[
\operatorname{pre}^{+}(a_4)
=
\{
\operatorname{cliMetadataValid},
\operatorname{corePackaged},
\operatorname{lockVerified}
\},
\]

\[
\operatorname{add}(a_4)
=
\{\operatorname{cliPackaged}\}.
\]

Core package precedes CLI package because the example's release profile requires dependency order.

Define clean-build actions:

\[
\operatorname{pre}^{+}(a_5)=\{\operatorname{corePackaged}\},
\qquad
\operatorname{add}(a_5)=\{\operatorname{coreCleanBuilt}\},
\]

\[
\operatorname{pre}^{+}(a_6)=\{\operatorname{cliPackaged}\},
\qquad
\operatorname{add}(a_6)=\{\operatorname{cliCleanBuilt}\}.
\]

Define dry-run actions:

\[
\operatorname{pre}^{+}(a_7)=\{\operatorname{coreCleanBuilt}\},
\qquad
\operatorname{add}(a_7)=\{\operatorname{coreDryRun}\},
\]

\[
\operatorname{pre}^{+}(a_8)
=
\{
\operatorname{cliCleanBuilt},
\operatorname{coreDryRun}
\},
\]

\[
\operatorname{add}(a_8)=\{\operatorname{cliDryRun}\}.
\]

The goal is:

\[
G^{+}=
\{
\operatorname{coreDryRun},
\operatorname{cliDryRun}
\}.
\]

## J.7 Plan witness

One valid modeled plan is:

\[
\pi=(a_1,a_2,a_3,a_4,a_5,a_6,a_7,a_8).
\]

We verify it.

Initial:

\[
s_0=
\{\operatorname{observedCore},
\operatorname{observedCli},
\operatorname{lockVerified},
\operatorname{coreMetadataValid}\}.
\]

After \(a_1\):

\[
s_1=s_0\cup\{\operatorname{repairApproved}\}.
\]

After \(a_2\):

\[
s_2=s_1\cup\{\operatorname{cliMetadataValid}\}.
\]

After \(a_3\):

\[
s_3=s_2\cup\{\operatorname{corePackaged}\}.
\]

After \(a_4\):

\[
s_4=s_3\cup\{\operatorname{cliPackaged}\}.
\]

After \(a_5\):

\[
s_5=s_4\cup\{\operatorname{coreCleanBuilt}\}.
\]

After \(a_6\):

\[
s_6=s_5\cup\{\operatorname{cliCleanBuilt}\}.
\]

After \(a_7\):

\[
s_7=s_6\cup\{\operatorname{coreDryRun}\}.
\]

After \(a_8\):

\[
s_8=s_7\cup\{\operatorname{cliDryRun}\}.
\]

Thus:

\[
G^{+}\subseteq s_8,
\]

so the sequence is a valid **modeled** plan.

## J.8 Partial-order reduction

The linear plan contains order not required by causality. Required precedence includes:

\[
a_1\prec a_2,
\]

\[
a_2\prec a_4,
\]

\[
a_3\prec a_4,
\]

\[
a_3\prec a_5,
\]

\[
a_4\prec a_6,
\]

\[
a_5\prec a_7,
\]

\[
a_6\prec a_8,
\]

\[
a_7\prec a_8.
\]

After \(a_2\) completes, \(a_3\) may already have completed or may run concurrently with other
independent verification depending on permission and resources. Activities \(a_5\) and \(a_6\) are
causally incomparable after their respective packages exist:

\[
a_5\parallel_{\prec}a_6.
\]

If they do not contend for an exclusive Cargo target directory or resource lock, then

\[
\operatorname{Concurrent}(a_5,a_6,s_4)=\mathsf{true}.
\]

The POWL geometry therefore does not preserve the arbitrary total sequence where it is unnecessary.

## J.9 Recursive repair socket

Suppose the parent workflow originally has activity:

\[
a=\operatorname{ValidateCliMetadata}.
\]

Real validation returns residue \(b\). The continuation goal is:

\[
g_b=\operatorname{CliDependencyHasVersion}(d).
\]

Child workflow \(U_b\) contains:

\[
u_1=\operatorname{InspectDependencyDeclaration},
\]

\[
u_2=\operatorname{AskForManifestEdit},
\]

\[
u_3=\operatorname{InvokeClaudeCodeEdit},
\]

\[
u_4=\operatorname{ReparseManifest},
\]

\[
u_5=\operatorname{VerifyDependencyVersion}.
\]

With order:

\[
u_1\prec u_2\prec u_3\prec u_4\prec u_5.
\]

The child promises:

\[
\operatorname{Post}(U_b)
\Rightarrow
\operatorname{CliMetadataValid}.
\]

Graft:

\[
W'=W[a\mapsto U_b].
\]

The parent package step remains after the child exit. The child does not delete package or dry-run
obligations.

## J.10 Permission

The rendered mutation surface is exactly one file:

\[
\mathcal{M}=
\{\text{example-cli/Cargo.toml}\}.
\]

Allowed operations:

\[
\mathcal{A}=
\{
\operatorname{ReadRepository},
\operatorname{EditManifest},
\operatorname{CargoMetadata},
\operatorname{CargoPackageLocked},
\operatorname{CargoPublishDryRunLocked},
\operatorname{CleanBuild}
\}.
\]

Forbidden:

\[
\operatorname{RegistryUpload}\in\mathcal{P}_{\mathrm{forbidden}}.
\]

The permission digest binds the exact plan and workflow. If Claude Code proposes changing core
source code, the broker sees:

\[
\operatorname{Mutation}(\text{core source})
\not\subseteq\mathcal{M}
\]

and refuses pending a new Ask.

## J.11 Real actuation and truthful refusal

The broker writes pre-receipt \(R^{-}_3\) for the manifest edit, invokes Claude Code with the bounded
task, observes patch \(\Delta\), writes post-receipt \(R^{+}_3\), reparses the manifest, and admits
the version.

Now suppose the real package command fails because the crate archive contains an absolute developer
path. The harness returns:

\[
o_{\mathrm{path}}
=
\operatorname{ObservedPathLeak}(
\text{archive},
\text{absolute path}
).
\]

Admission derives:

\[
r_{\mathrm{path}}
=
\mathsf{Refused}(
\operatorname{ArchivePathLeak}
).
\]

The planner's predicted effect \(\operatorname{cliPackaged}\) is **not** added to admitted execution
state. The current modeled plan has encountered new residue. MFW may manufacture another repair
child if descent and permission allow; otherwise it returns Refused or Bounded.

The truthful terminal result for this attempt is not Found merely because the first manifest repair
succeeded.

## J.12 Crash window

Suppose the Cargo package dispatch occurs after \(R^{-}_4\) is durable, but the MFW process crashes
before capturing output. Recovered state is:

\[
\mathsf{UnknownAfterDispatch}(\Pi,p,R^{-}_4,k_4).
\]

Recovery checks the idempotency key, expected archive path, process status, and filesystem
observation. It must not blindly dispatch again if duplicate package effects matter. Once
reconciled, it writes a post-receipt describing observed success, failure, or unresolved ambiguity.

## J.13 Replay

Let recorded execution events be:

\[
E=(e_1,\ldots,e_n).
\]

Replay begins from canonical admitted graph digest \(h_0\) and folds deterministic transition
\(\delta\):

\[
s_n=\delta^{*}(s_0,E).
\]

The replay verifies:

\[
\operatorname{standing}(s_n)=\mathsf{Refused},
\]

\[
\operatorname{reason}(s_n)=\operatorname{ArchivePathLeak},
\]

and every event is linked to the run, plan, permission, and receipt.

If the human report says Found while RDF replay says Refused, the run is Inconsistent and the report
is regenerated.

## J.14 Standing analysis

This example establishes on paper:

- a valid finite planning model;
- a POWL partial-order geometry;
- an admissible recursive repair shape;
- a permission predicate;
- a receipt/replay construction; and
- a truthful refusal path.

It does **not** establish:

- the production adapter emits these exact triples;
- the production planner generates this plan;
- production Claude Code events are all captured;
- the production broker has no bypass;
- the real repository has only this blocker; or
- the whole-workspace crown is Alive.

Those remain execution claims.

---

# Appendix K. Counterexamples That Bound the Thesis

## K.1 Valid plan, impossible world

The PDDL model contains action FlyWithoutFuel with effect Arrived. A planner finds it. The plan is
valid relative to the model and useless in reality. This proves model validity is conditional on
admission and action correspondence.

## K.2 Real command, wrong subject

A dry run passes for crate A. The status report marks the workspace Alive. This violates same-object
scope because the workspace includes crate B.

## K.3 Complete unit proof, missing integration edge

F08 and F09 each pass unit tests using independent fixtures. No F08 output becomes F09 input. The
edge is not Real.

## K.4 Shared source, divergent semantics

Rust and Erlang are generated from the same RDF, but one treats a missing field as false and the
other as Unknown. Common generation did not guarantee semantic equivalence.

## K.5 Receipt without occurrence truth

A compromised broker signs a receipt for a command it never sent. The receipt authenticates the
broker's assertion, not physical truth. Independent observation and trust assumptions remain needed.

## K.6 Occurrence without correctness

A receipt proves cargo package ran and exited zero. It does not prove the package contains no
license violation unless that property was separately checked.

## K.7 Post-receipt race

An external effect occurs, the process crashes before post-receipt, and a system represents the
action as Completed. This violates the type-state model. The lawful state is UnknownAfterDispatch.

## K.8 Hash equality without profile identity

Two Triple8 profiles assign byte 17 to different IRIs. Comparing the byte alone yields false
semantic equality. The profile digest is mandatory.

## K.9 Antichain with resource conflict

Two activities are causally incomparable but both require the same exclusive database migration
lock. Antichain membership alone does not permit concurrent execution.

## K.10 Dependency reachability over-regenerates

Input \(x\) changes from 2 to 4. Artifact \(y=x\bmod2\) is graph-dependent on \(x\), but its value
does not change. Reachability is conservative, not actual instance-minimal residue.

## K.11 Minimal supports are nonunique

Assume the admitted assignment domain enforces \(a=b\), and define output \(y=a\). Then
\(\{a\}\) is a minimal support because \(a\) directly determines \(y\), while \(\{b\}\) is also a
minimal support because the invariant makes \(b=a=y\). Neither singleton contains the other. A
system reporting one “the” minimal support without the assignment domain is under-specified.

## K.12 Tenant identifiers with shared global state

Tenant operations write disjoint records but increment one global metric. Their total supports
intersect at the metric. Structural isolation does not hold until shared state is included or
removed.

## K.13 Commuting labels, noncommuting effects

Two operations are both named SetConfig but write different expected versions. Label equality does
not imply commutation.

## K.14 Symmetric swap nontermination

If both \(ab\to ba\) and \(ba\to ab\) are rewrite rules, normalization loops. Semantic equivalence
can hold while the rewrite system does not terminate.

## K.15 SHACL-conformant falsehood

A graph says a package license is MIT and satisfies datatype and cardinality shapes. The repository
has no corresponding license grant. Structural conformance is not truth.

## K.16 Public vocabulary misuse

Using a PROV-O predicate with the wrong intended relation does not become interoperable because the
IRI is public. Public-first requires semantic review.

## K.17 N3 hidden oracle

An N3 built-in calls the network and changes result between runs. Treating it as pure rule closure
destroys deterministic replay. External values must be admitted events.

## K.18 Bounded masquerading as exhausted

A planner reaches a depth limit while the queue contains states. A CLI maps both Bounded and
Exhausted to “no plan.” The internal type may be sound, but the product claim is false at the CLI
boundary.

## K.19 Agent self-promotion

Claude Code edits code and writes “tests pass” into a report without running the test. The text is a
proposal, not evidence.

## K.20 Test weakened by repair

A failing test is changed to assert less. Green output does not discharge the original parent
obligation. Obligation monotonicity detects the deletion.

## K.21 Child authority expansion

A child spawned to edit one manifest performs a Git push. Recursive depth does not imply recursive
authority. The action is refused.

## K.22 Replay under new semantics

Events are replayed with a newer AIR transition function. The final state differs. This is not a
failure of deterministic replay; the transition identity changed.

## K.23 Finite data false multifractality

A heavy-tailed but independent sample produces a broad estimated spectrum. Shuffling changes
nothing. The data may be distribution-driven rather than correlation-driven.

## K.24 Hierarchy mistaken for dimension

A workflow has five nesting levels. Calling its fractal dimension five is meaningless; hierarchy
depth and Hausdorff dimension are different quantities.

## K.25 Smooth calculus on discrete law

A gradient recommends changing half a Boolean permission bit. The continuous relaxation produced an
inadmissible point. Exact discrete planning must project or refuse it.

## K.26 Ephemeralization by omitted cost

Compute moves to an external service and local cost falls. If external compute is excluded, the
ratio falsely improves. The resource boundary must include transferred cost.

## K.27 Synergy by double counting

Component capabilities overlap. Summing them as if disjoint makes the whole appear subadditive or
superadditive arbitrarily. The isolation and valuation protocol must define overlap.

## K.28 Trimtab without whole-system model

A small code change has a large local benchmark gain but breaks replay. It is not a trimtab for the
whole admitted capability functional.

## K.29 Theorem count as coverage

One hundred forty-five theorems may all concern a narrow module. Count alone does not prove crown
coverage. The theorem-to-claim mapping is required.

## K.30 Adoption as impact

Many people use a system whose receipts are incomplete. Adoption does not increase controlled
design correctness. Conversely, zero adoption does not erase an internally verified new capacity.
Adoption is intentionally absent from the impact functional.

---
