# REMAINING GAPS LEDGER: Swarm Thesis Foundations Paper Stress-Test

This ledger details the mathematical, logical, and semantic inconsistencies identified during the adversarial stress-test of the rewritten foundations paper: `/Users/sac/praxis/docs/thesis/swarm_rewrite/00_foundations_rewritten.tex`.

---

## 1. Logical Type Discrepancy in the Conservation of Consequence Theorem (Requirement 1)

### Observation
- **Theorem 9.1 (Conservation of Consequence)** states:
  > "...the map $a \mapsto h_+(a)$ from actuated artifacts to receipt-chain positions is well-defined and injective up to hash collision..."

### Logic Chain
1. An actuated artifact $a \in \mathcal{A}$ represents a system change produced by the manufacturing morphism $\mu: \mathcal{O}^* \cup \{\bot\} \to \mathcal{A} \cup \{\bot\}$.
2. In a normal execution trace of a system, the same artifact $a$ can be actuated multiple times at different steps (e.g., at step $t_1$ and step $t_2$, where $t_1 \neq t_2$).
3. Because the receipt chain is append-only and commits to history recursively, the chain hashes at steps $t_1$ and $t_2$ must be distinct (i.e., $h_+(t_1) \neq h_+(t_2)$).
4. If $a$ is actuated at both steps, the mapping $a \mapsto h_+(a)$ must map $a$ to both $h_+(t_1)$ and $h_+(t_2)$.
5. Therefore, the mapping $a \mapsto h_+(a)$ is multi-valued and **not a well-defined function** on the domain of actuated artifacts $\mathcal{A}$.

### Critique
The theorem conflates the *actuated artifact* $a$ (an element of the action space $\mathcal{A}$) with the *actuation event* or *step index* $t$ in the trace. 
- If the domain is the set of **actuation events/steps**, the map to receipt-chain positions is well-defined and injective (bijective, assuming no hash collisions).
- If the domain is the set of **actuated artifacts**, the map $a \mapsto h_+(a)$ is not a function.
- If we look at the inverse direction (receipt-chain positions to actuated artifacts), the map is well-defined but **not injective**, since different steps can yield the same artifact.

### Mitigation
Redefine the mapping in Theorem 9.1 as a map from **actuation events** (or receipt frames) to receipt-chain positions, rather than from actuated artifacts.

---

## 2. Inconsistent Signature of $\mu$ and Definition of Admitted Space $\mathcal{O}^*$ (Requirement 2 & 4)

### Observation
- **Definition 2.1 (Refined Chatman Equation)** defines:
  > "$\mathcal{O}^* = \operatorname{im}(\alpha)$"
- **Definition 4.2 (Admission Map)** defines:
  > "$\alpha: \mathcal{O}_\bot \to \mathcal{O}^* \cup \{\bot\}$" where $\alpha(o) = \bot$ if obligations fail.
- **Construction 2.1 (ii)** states:
  > "$\mathcal{O}^* \subseteq \mathcal{O}$, the admitted syntactic subspace."

### Logic Chain
1. Since $\mathcal{O}$ is the raw observation space (which excludes $\bot$), the subset $\mathcal{O}^* \subseteq \mathcal{O}$ cannot contain $\bot$ (i.e., $\bot \notin \mathcal{O}^*$).
2. The admission map $\alpha$ maps to $\mathcal{O}^* \cup \{\bot\}$. If obligations fail or the input is $\bot$, the output is $\bot$.
3. Thus, the image of $\alpha$ is $\operatorname{im}(\alpha) = \mathcal{O}^* \cup \{\bot\}$.
4. However, Definition 2.1 defines $\mathcal{O}^*$ as $\operatorname{im}(\alpha)$. Since $\bot \in \operatorname{im}(\alpha)$, this requires $\bot \in \mathcal{O}^*$, which directly contradicts Construction 2.1 (ii) stating $\mathcal{O}^* \subseteq \mathcal{O}$.

### Critique
This is a core domain/codomain signature contradiction:
- If $\mathcal{O}^* = \operatorname{im}(\alpha)$, then $\mathcal{O}^*$ contains $\bot$, violating syntactic quarantine boundaries.
- If $\mathcal{O}^* \subseteq \mathcal{O}$, then $\operatorname{im}(\alpha) \neq \mathcal{O}^*$ (it is $\mathcal{O}^* \cup \{\bot\}$).
- Consequently, the boxed governing equation $\mathcal{A} = \mu(\mathcal{O}^*)$ in Definition 2.1 is broken: if $\mathcal{O}^*$ includes $\bot$, then $\mu(\mathcal{O}^*)$ contains $\mu(\bot) = \bot$, which means $\bot \in \mathcal{A}$ (the action space), conflating action with refusal.

### Mitigation
Strictly define $\mathcal{O}^*$ as the subset of raw observations $\mathcal{O}$ that pass obligations. State the image of $\alpha$ as $\mathcal{O}^* \cup \{\bot\}$. Update the governing equation to map only the admitted subspace $\mathcal{O}^*$ to $\mathcal{A}$ via $\mu$, while mapping $\bot$ to $\bot$.

---

## 3. Categorical vs. Set-Theoretic Mismatch of "Retraction" (Requirement 3)

### Observation
- **Construction 5.2 (Factored Retraction)** states:
  > "The single-step retraction $\alpha$ is factored as: $\alpha = a \circ j$"
- **Definition 5.1 (Lifecycle Category $\mathcal{L}\mathit{ife}$)** defines the free category on the linear quiver:
  > "$\mathsf{Raw} \xrightarrow{j} \mathsf{Validated} \xrightarrow{a} \mathsf{Admitted} \xrightarrow{r} \mathsf{Receipted}$"

### Logic Chain
1. In category theory, a morphism $f: X \to Y$ is a retraction if there exists a section $g: Y \to X$ such that $f \circ g = \operatorname{id}_Y$.
2. In the category $\mathcal{L}\mathit{ife}$, the stages $\mathsf{Raw}$ and $\mathsf{Admitted}$ are distinct objects.
3. The quiver defining $\mathcal{L}\mathit{ife}$ is linear and directed, with no backward edges.
4. Hence, the hom-set $\mathcal{L}\mathit{ife}(\mathsf{Admitted}, \mathsf{Raw})$ is empty; there is no morphism that can serve as a section.
5. Therefore, the morphism $\alpha = a \circ j: \mathsf{Raw} \to \mathsf{Admitted}$ is **not a retraction** in the category-theoretic sense.

### Critique
The paper conflates the set-theoretic retraction (an idempotent map $\alpha: \mathcal{O}_\bot \to \mathcal{O}_\bot$ where $\alpha \circ \alpha = \alpha$) with the category-theoretic morphism in $\mathcal{L}\mathit{ife}$ between distinct objects ($\mathsf{Raw} \to \mathsf{Admitted}$).
- Categorically, the composition $\alpha \circ \alpha$ is type-incorrect because the codomain ($\mathsf{Admitted}$) does not match the domain ($\mathsf{Raw}$).
- The linear quiver structure prevents any backward paths, making category-theoretic retraction mathematically impossible.

### Mitigation
Clarify that $\alpha$ is a retraction in the concrete set category $\mathbf{Set}$ (where the objects are sets and the maps are functions on the same base set $\mathcal{O}_\bot$), and that the category $\mathcal{L}\mathit{ife}$ is an abstract representation of stage transitions rather than a concrete category of sets and functions.

---

## 4. Coarse Mathematical Treatment of $\bot$ vs. Rich Rust Typestate (Requirement 4)

### Observation
- **Definition 4.1** represents refusal as a single distinguished constant $\bot \in \mathcal{O}_\bot$.
- **Section 4.4 (Operational Correspondence)** maps $\bot$ to `Andon::Halted` and `RefusalScenario` in Rust.

### Logic Chain
1. The mathematical retraction maps all failures to a single, featureless constant $\bot$, discarding all payload and diagnostic information.
2. In the Rust implementation (`crates/praxis-core/src/law.rs`), validation failure returns a rich `LawObject` in the `Raw` stage carrying `Andon::Halted` with the original payload, a list of unmet obligations, a checklist of refusal scenarios, and a timestamp.
3. This rich state is essential for auditing, the andon-ring defect signaling, and the override/recovery path (`Andon::Overridden`).
4. Collapsing this structure to a single set-theoretic constant $\bot$ creates a significant gap between the theoretical guarantees and the actual system capability.

### Mitigation
Formalize the refusal space as a set of halted configurations $\mathcal{H} = \mathcal{O} \times D_{\text{refuse}} \times \mathbb{N}$ (carrying the payload, the non-zero denial word, and the timestamp). Define the extended space as $\mathcal{O}_\bot = \mathcal{O} \cup \mathcal{H} \cup \{\bot\}$. This allows the mathematics to model diagnostic retention and override gates.

---

## 5. Other Logical Gaps and Loose Definitions (Requirement 5)

### A. Circular Proof of Causation in Theorem 9.1
- **Observation**: The proof of causation in Theorem 9.1 states:
  > "By the Admission Gate invariant (B1), $a = \muop(x)$ for some $x = \adm(o) \neq \Rfsl$. This guarantees causation: $a$ is the image under $\muop$ of at least one admitted input $x$."
- **Critique**: This is a trivial tautology. Invariant B1 is an assumed system constraint that forbids actuation unless it goes through the admission gate. The theorem claims to *prove* causation under the invariants, but the proof simply restates B1. There is no independent deduction of causation; it is assumed by definition of the invariant.

### B. Loose Definition of "Verification Cost" in Divergence of Comprehension
- **Observation**: Construction 1.2 and Theorem 1.1 model the cost to verify the boundary projection as $\mathrm{Ver}_{\partial}(\sigma) = O(\kappa)$, claiming it remains constant.
- **Critique**: If the trace length is $N$, the receipt chain contains $N$ frames. Recomputing and verifying the cryptographic chain of hashes is a linear computational task ($O(N)$). The paper conflates **human cognitive verification cost** (which is $O(\kappa)$ as the human only inspects the final signature/verdict) with **machine computational verification cost** (which is $O(N)$). Without this distinction, the claim of constant-time verification is misleading.

### C. Undefined Domain of Partial Normalization Map $\rho$
- **Observation**: Definition 4.2 defines $\rho: \mathcal{O} \rightharpoonup \mathcal{O}^*$ as a partial function.
- **Critique**: If $\rho$ is a partial function, the admission map $\alpha(o)$ is undefined when obligations pass but $o \notin \operatorname{dom}(\rho)$. To ensure $\alpha$ is a total function on $\mathcal{O}_\bot$, the paper must explicitly assume the obligation compatibility condition:
  $$\{o \in \mathcal{O} : \bigwedge_{i=1}^m g_i(o) = 1\} \subseteq \operatorname{dom}(\rho)$$
  Alternatively, define $\rho$ as a total function mapping to $\mathcal{O}^* \cup \{\bot\}$, reflecting that parser failure itself is a first-class refusal trigger.
