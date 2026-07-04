# Role 2.9: Process Geometry Agent - Planning/Conformance

## Structured Notes

*   **Manufacture as Search, Conformance as Geometry**: Mission execution involves both the search for a plan ($\mu$) and the geometric verification of the trace (B4). 
*   **Token States and Markings**: 
    *   State is represented as a marking $\bm{m}$ (vector of atom-tokens and numeric fluent valuations).
    *   The state equation governs transitions: $\bm{m} = \bm{m}_0 + \mathbf{N}\bm{x}$, where $\mathbf{N}$ is the incidence matrix and $\bm{x}$ is the transition-count (Parikh) vector.
*   **Execution Traces**:
    *   Represented as firing sequences in a safe-net specialization (e.g., lifecycle token verifier using branchless updates).
    *   Traces yield a final marking $\bm{m}^\dagger$.
*   **Lawful Trace Regions**:
    *   A lawful (conformant) trace lands inside the marking polytope, which is the translated cone $\bm{m}_0 + \text{cone}(\mathbf{N})$ intersected with the non-negative orthant.
    *   Conformance is checking membership in this cone.
*   **Farkas / Hyperplane Separation**:
    *   If a trace is non-conformant (i.e., outside the cone), Farkas' lemma provides a separating hyperplane.
    *   This hyperplane certificate $\bm{y}$ proves non-reachability ($\mathbf{N}^\top \bm{y} \le \mathbf{0}$ and $\bm{y}^\top \bm{b} > 0$, where $\bm{b} = \bm{m}^\dagger - \bm{m}_0$).
    *   The certificate dimension is $p$, verifiable in $O(\kappa)$ independent of trace length.
*   **POWL & Separability**:
    *   POWL 2.0 defines the process geometry as recursive partial orders and choice graphs.
    *   Separability acts as a "Rice quarantine", reducing arbitrary control flow to bounded-arity replay checks.

---

## Chapter Draft: Conformance as Geometry

### Introduction
The conformance of an executed process is fundamentally a geometric property. While the manufacture of a mission acts as a bounded search over a finite ground net, assessing the validity of an execution trace is evaluated through cone membership. We define execution traces as firing sequences that transform token states (markings), and establish the boundaries of lawful traces within a marking polytope.

### Token States and Execution Traces
In the process geometry, the state is defined as a marking $\bm{m} \in \mathbb{Z}_{\ge 0}^p$ composed of token bits and numeric fluents (e.g., attention capacity). A transition $t$ (an action) consumes a preset $\bm{m}_t^-$ and produces a postset $\bm{m}_t^+$. An execution trace is a sequence of these transitions, adhering to the state equation:
$$ \bm{m} = \bm{m}_0 + \mathbf{N}\bm{x} $$
where $\mathbf{N}$ is the incidence matrix of the net and $\bm{x}$ is the Parikh vector counting the firings. The lifecycle verifier evaluates these traces utilizing a branchless safe-net integer firing rule.

### Lawful Trace Regions and Cone Membership
A trace is considered lawful—or conformant—only if its resulting marking $\bm{m}^\dagger$ resides within the valid marking polytope $\mathcal{P}$. This geometric region is defined as the integer points of the translated cone $\bm{m}_0 + \text{cone}(\mathbf{N})$ restricted to the non-negative orthant. Conformance is strictly equivalent to membership in this cone, combined with a fitness metric that must evaluate to unity ($\varphi = 1$). Token-replay fitness characterizes genuine firing sequences, with any excursion outside the enabled set explicitly caught and localized.

### Farkas Certificates and Separating Hyperplanes
When a trace breaches the bounds of the lawful trace region ($\bm{m}^\dagger \notin \mathcal{P}$), its nonconformance can be mechanically certified using a Farkas separating hyperplane. By Farkas' Lemma, if the marking is not reachable, there exists a vector $\bm{y} \in \mathbb{R}^p$ such that:
$$ \mathbf{N}^\top \bm{y} \le \mathbf{0} \quad \text{and} \quad \bm{y}^\top (\bm{m}^\dagger - \bm{m}_0) > 0 $$
This hyperplane serves as a sound, $p$-dimensional proof of nonconformance. Critically, checking this certificate involves only a matrix product and two sign tests, allowing the bounded verifier to validate the trace in $O(\kappa)$ time without replaying the potentially unbounded interior of the mission execution.
