# E2E LaTeX Test Report

- **Target File:** `docs/thesis/swarm_rewrite/00_foundations_rewritten.tex`
- **Timestamp:** `2026-07-04 23:54:57 UTC`

## Summary Table

| Step | Status | Issues Found | Details |
| --- | --- | --- | --- |
| Compilation | PASS | 25 | Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/swarm_rewrite/00_foundations_rewritten.tex |
| Structural & Content | FAIL | 4 | 2 mismatches, 2 hype, 0 overclaims |
| Notation Canon | FAIL | 96 | 96 violations |

## 1. Compilation & Logs

**Result:** Compilation succeeded. Command: pdflatex -interaction=nonstopmode -output-directory=docs/thesis/build docs/thesis/swarm_rewrite/00_foundations_rewritten.tex

### Warnings & Errors from Log:
- `Package: infwarerr 2019/12/03 v1.5 Providing info/warning/error messages (HO)`
- `pdfTeX warning (ext4): destination with the same identifier (name{page.i}) has`
- `LaTeX Warning: Reference `con:verification_gap' on page 1 undefined on input li`
- `LaTeX Warning: Reference `def:chatman_eq' on page 4 undefined on input line 210`
- `LaTeX Warning: Citation `rice1953' on page 5 undefined on input line 259.`
- `LaTeX Warning: Reference `thm:rice_specialized' on page 5 undefined on input li`
- `LaTeX Warning: Reference `prop:hom_sets' on page 11 undefined on input line 495`
- `LaTeX Warning: Reference `thm:uninhabited' on page 11 undefined on input line 5`
- `LaTeX Font Warning: Font shape `OT1/cmr/m/scit' undefined`
- `LaTeX Warning: Reference `con:chain_step' on page 20 undefined on input line 73`
- `LaTeX Font Warning: Font shape `OMS/cmtt/m/n' undefined`
- `LaTeX Warning: Citation `chatman2025' on page 23 undefined on input line 854.`
- `LaTeX Warning: Reference `thm:rice_specialized' on page 29 undefined on input l`
- `LaTeX Warning: Reference `thm:idempotence' on page 29 undefined on input line 1`
- `LaTeX Warning: Reference `prop:denial_mono' on page 29 undefined on input line`
- `LaTeX Warning: Reference `prop:hom_sets' on page 29 undefined on input line 101`
- `LaTeX Warning: Reference `thm:uninhabited' on page 29 undefined on input line 1`
- `LaTeX Warning: Reference `prop:single_valued' on page 29 undefined on input lin`
- `LaTeX Warning: Reference `lem:chain_commit' on page 29 undefined on input line`
- `LaTeX Warning: Reference `thm:conservation_theorem' on page 29 undefined on inp`
- `LaTeX Warning: Reference `cor:antibody_clause' on page 29 undefined on input li`
- `LaTeX Font Warning: Some font shapes were not available, defaults substituted.`
- `LaTeX Warning: There were undefined references.`
- `LaTeX Warning: Label(s) may have changed. Rerun to get cross-references right.`
- `Package rerunfilecheck Warning: File `00_foundations_rewritten.out' has changed`

## 2. Structural & Content Audit

### Theorem-Proof Structural Mismatches:
- ❌ Total environment count mismatch: 30 theorem-like environment(s) vs 12 proof(s).
- ❌ Theorem-like environment 'corollary' at line 745 has no matching proof at the end of the document.

### Hype Word Violations:
- ❌ Line 1049: found 'magic' in `\emph{The magical number seven, plus or minus two: some limits on our capacity`
- ❌ Line 1055: found 'magic' in `\emph{The magical number 4 in short-term memory: A reconsideration of mental`
- ✅ No overclaim words found.

## 3. Notation Canon Audit

### Notation Canon Violations:
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 74 is used outside the 6 allowed Chatman Equations.
Context: We factor the raw, undecidable observation space $\mathcal{O}$ by introducing the extended observation space $\mathcal{O}_\bot = \mathcal{O} \cup \{\bot\}$ and defining a computable retraction map $\alpha: \mathcal{O}_\bot \to \mathcal{O}^* \cup \{\bot\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 74 is used outside the 6 allowed Chatman Equations.
Context: We factor the raw, undecidable observation space $\mathcal{O}$ by introducing the extended observation space $\mathcal{O}_\bot = \mathcal{O} \cup \{\bot\}$ and defining a computable retraction map $\alpha: \mathcal{O}_\bot \to \mathcal{O}^* \cup \{\bot\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 74 is used outside the 6 allowed Chatman Equations.
Context: We factor the raw, undecidable observation space $\mathcal{O}$ by introducing the extended observation space $\mathcal{O}_\bot = \mathcal{O} \cup \{\bot\}$ and defining a computable retraction map $\alpha: \mathcal{O}_\bot \to \mathcal{O}^* \cup \{\bot\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 74 is used outside the 6 allowed Chatman Equations.
Context: We factor the raw, undecidable observation space $\mathcal{O}$ by introducing the extended observation space $\mathcal{O}_\bot = \mathcal{O} \cup \{\bot\}$ and defining a computable retraction map $\alpha: \mathcal{O}_\bot \to \mathcal{O}^* \cup \{\bot\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 74 is used outside the 6 allowed Chatman Equations.
Context: We factor the raw, undecidable observation space $\mathcal{O}$ by introducing the extended observation space $\mathcal{O}_\bot = \mathcal{O} \cup \{\bot\}$ and defining a computable retraction map $\alpha: \mathcal{O}_\bot \to \mathcal{O}^* \cup \{\bot\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 75 is used outside the 6 allowed Chatman Equations.
Context: This retraction maps raw, unstructured inputs onto a decidable, normalized syntactic subspace $\mathcal{O}^*$ while mapping validation failures to the first-class refusal value $\bot$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{O}' at line 82 is used outside the 6 allowed Chatman Equations.
Context: We model the manufacturing morphism $\mu: \mathcal{O}^* \cup \{\bot\} \to \mathcal{A} \cup \{\bot\}$ as a deterministic, cost-bounded map that translates admitted payloads into actuated artifacts while propagating refusal.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{A}' at line 82 is used outside the 6 allowed Chatman Equations.
Context: We model the manufacturing morphism $\mu: \mathcal{O}^* \cup \{\bot\} \to \mathcal{A} \cup \{\bot\}$ as a deterministic, cost-bounded map that translates admitted payloads into actuated artifacts while propagating refusal.
- ❌ Notation Violation: Calligraphic/Macro symbol '\mathcal{R}' at line 85 is used outside the 6 allowed Chatman Equations.
Context: Execution traces are projected onto a compact receipt space $\mathcal{R}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 173 is used outside the 6 allowed Chatman Equations.
Context: Let $\Obs$ be the raw observation space containing arbitrary finite byte strings.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 174 is used outside the 6 allowed Chatman Equations.
Context: Let $\Rfsl$ be the distinguished refusal constant representing failure, halt, or invalid input, where $\Rfsl \notin \Obs$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 177 is used outside the 6 allowed Chatman Equations.
Context: \Obsbot = \Obs \cup \mathcal{H} \cup \{\Rfsl\}
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 180 is used outside the 6 allowed Chatman Equations.
Context: Let $\Obs^* \subseteq \Obs$ be the admitted syntactic subspace consisting of raw observations that pass all obligations.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 180 is used outside the 6 allowed Chatman Equations.
Context: Let $\Obs^* \subseteq \Obs$ be the admitted syntactic subspace consisting of raw observations that pass all obligations.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 189 is used outside the 6 allowed Chatman Equations.
Context: where $\Act$ is the action space representing actuated system changes.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 190 is used outside the 6 allowed Chatman Equations.
Context: The admission map is a function $\adm: \Obsbot \to \Obs^* \cup \{\Rfsl\}$ whose image is $\im(\adm) = \Obs^* \cup \{\Rfsl\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 190 is used outside the 6 allowed Chatman Equations.
Context: The admission map is a function $\adm: \Obsbot \to \Obs^* \cup \{\Rfsl\}$ whose image is $\im(\adm) = \Obs^* \cup \{\Rfsl\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 191 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism is a map $\muop: \Obs^* \to \Act$, which is extended to the domain $\Obs^* \cup \{\Rfsl\}$ by propagating the refusal constant such that $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 191 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism is a map $\muop: \Obs^* \to \Act$, which is extended to the domain $\Obs^* \cup \{\Rfsl\}$ by propagating the refusal constant such that $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 191 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism is a map $\muop: \Obs^* \to \Act$, which is extended to the domain $\Obs^* \cup \{\Rfsl\}$ by propagating the refusal constant such that $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 197 is used outside the 6 allowed Chatman Equations.
Context: \item $\Obs$, the raw observation space containing arbitrary finite byte strings.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 198 is used outside the 6 allowed Chatman Equations.
Context: \item $\Obs^* \subseteq \Obs$, the admitted syntactic subspace.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 198 is used outside the 6 allowed Chatman Equations.
Context: \item $\Obs^* \subseteq \Obs$, the admitted syntactic subspace.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 199 is used outside the 6 allowed Chatman Equations.
Context: \item $\muop: \Obs^* \to \Act$, the manufacturing morphism, extended as $\muop: \Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 199 is used outside the 6 allowed Chatman Equations.
Context: \item $\muop: \Obs^* \to \Act$, the manufacturing morphism, extended as $\muop: \Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 199 is used outside the 6 allowed Chatman Equations.
Context: \item $\muop: \Obs^* \to \Act$, the manufacturing morphism, extended as $\muop: \Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 199 is used outside the 6 allowed Chatman Equations.
Context: \item $\muop: \Obs^* \to \Act$, the manufacturing morphism, extended as $\muop: \Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 200 is used outside the 6 allowed Chatman Equations.
Context: \item $\Act$, the artifact/action space representing actuated system changes.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Rec' at line 201 is used outside the 6 allowed Chatman Equations.
Context: \item $\Rec$, the receipt space containing bounded execution projections.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 206 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism $\muop$ is well-defined on all elements of the admitted space $\Obs^*$ and propagates the refusal constant $\Rfsl$ without evaluating raw payloads.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 210 is used outside the 6 allowed Chatman Equations.
Context: By Definition~\ref{def:chatman_eq}, $\muop$ maps $\Obs^*$ to $\Act$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 210 is used outside the 6 allowed Chatman Equations.
Context: By Definition~\ref{def:chatman_eq}, $\muop$ maps $\Obs^*$ to $\Act$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 211 is used outside the 6 allowed Chatman Equations.
Context: For $x \in \Obs^*$, $\muop(x)$ is a well-defined element of $\Act$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 211 is used outside the 6 allowed Chatman Equations.
Context: For $x \in \Obs^*$, $\muop(x)$ is a well-defined element of $\Act$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 213 is used outside the 6 allowed Chatman Equations.
Context: Since $\Obs \setminus \Obs^*$ is excluded from the domain of $\muop$ (being mapped to $\Rfsl$ by the admission map $\adm$), no unadmitted raw observation bypasses the admission filter.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 213 is used outside the 6 allowed Chatman Equations.
Context: Since $\Obs \setminus \Obs^*$ is excluded from the domain of $\muop$ (being mapped to $\Rfsl$ by the admission map $\adm$), no unadmitted raw observation bypasses the admission filter.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 222 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism $\muop$ cannot be evaluated on elements of $\Obs \setminus \Obs^*$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 222 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism $\muop$ cannot be evaluated on elements of $\Obs \setminus \Obs^*$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 226 is used outside the 6 allowed Chatman Equations.
Context: \item \textbf{What was admitted:} The signature of $\muop$ on the admitted domain $\Obs^*$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 248 is used outside the 6 allowed Chatman Equations.
Context: There is a set $\Obs$, the observation space, whose elements are arbitrary finite records.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 249 is used outside the 6 allowed Chatman Equations.
Context: $\Obs$ carries no decidable semantics; the predicate "does this observation mean what it purports to mean" is not assumed computable.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 253 is used outside the 6 allowed Chatman Equations.
Context: Let $\mathcal{U}$ be a universal model of computation and let observations in $\Obs$ range over finite encodings that may denote arbitrary $\mathcal{U}$-programs.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 255 is used outside the 6 allowed Chatman Equations.
Context: Then the set $\{o\in\Obs : P(o)\}$ is undecidable.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 259 is used outside the 6 allowed Chatman Equations.
Context: This is Rice's theorem \cite{rice1953} instantiated at $\Obs$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 269 is used outside the 6 allowed Chatman Equations.
Context: There is no algorithm that admits an observation $o\in\Obs$ by deciding a non-trivial semantic property of what $o$ denotes.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 310 is used outside the 6 allowed Chatman Equations.
Context: Let $\Obs$ be the raw observation space.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 314 is used outside the 6 allowed Chatman Equations.
Context: \mathcal{H} = \Obs \times D_{\text{refuse}} \times \mathbb{N}
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 319 is used outside the 6 allowed Chatman Equations.
Context: \Obsbot = \Obs \cup \mathcal{H} \cup \{\Rfsl\}
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 325 is used outside the 6 allowed Chatman Equations.
Context: Let $\Obs^* \subseteq \Obs$ be the admitted space.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 325 is used outside the 6 allowed Chatman Equations.
Context: Let $\Obs^* \subseteq \Obs$ be the admitted space.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 326 is used outside the 6 allowed Chatman Equations.
Context: Let $g_1, \dots, g_m: \Obs \to \{0,1\}$ be total computable obligations.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 327 is used outside the 6 allowed Chatman Equations.
Context: The admission map $\adm: \Obsbot \to \Obs^* \cup \{\Rfsl\}$ is:
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 330 is used outside the 6 allowed Chatman Equations.
Context: \rho(o) \in \Obs^* & \text{if } x = o \in \Obs \text{ and } \bigwedge_{i=1}^m g_i(o) = 1, \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 330 is used outside the 6 allowed Chatman Equations.
Context: \rho(o) \in \Obs^* & \text{if } x = o \in \Obs \text{ and } \bigwedge_{i=1}^m g_i(o) = 1, \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 334 is used outside the 6 allowed Chatman Equations.
Context: where $\rho: \Obs \rightharpoonup \Obs^*$ is a computable normalization fixing $\Obs^*$ pointwise.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 334 is used outside the 6 allowed Chatman Equations.
Context: where $\rho: \Obs \rightharpoonup \Obs^*$ is a computable normalization fixing $\Obs^*$ pointwise.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 334 is used outside the 6 allowed Chatman Equations.
Context: where $\rho: \Obs \rightharpoonup \Obs^*$ is a computable normalization fixing $\Obs^*$ pointwise.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 337 is used outside the 6 allowed Chatman Equations.
Context: \{o \in \Obs : \bigwedge_{i=1}^m g_i(o) = 1\} \subseteq \operatorname{dom}(\rho)
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 349 is used outside the 6 allowed Chatman Equations.
Context: For $x = o \in \Obs$ where obligations fail, $\adm(o) = \Rfsl$, so $\adm(\adm(o)) = \adm(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 350 is used outside the 6 allowed Chatman Equations.
Context: For $x = o \in \Obs$ where obligations pass, $y = \adm(o) = \rho(o) \in \Obs^* \subseteq \Obs$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 350 is used outside the 6 allowed Chatman Equations.
Context: For $x = o \in \Obs$ where obligations pass, $y = \adm(o) = \rho(o) \in \Obs^* \subseteq \Obs$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 350 is used outside the 6 allowed Chatman Equations.
Context: For $x = o \in \Obs$ where obligations pass, $y = \adm(o) = \rho(o) \in \Obs^* \subseteq \Obs$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 351 is used outside the 6 allowed Chatman Equations.
Context: Since $\rho$ fixes $\Obs^*$ pointwise and obligations pass on $\Obs^*$, $\adm(y) = y$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 351 is used outside the 6 allowed Chatman Equations.
Context: Since $\rho$ fixes $\Obs^*$ pointwise and obligations pass on $\Obs^*$, $\adm(y) = y$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 357 is used outside the 6 allowed Chatman Equations.
Context: For a subset of active obligations $G \subseteq \{1, \dots, n\}$, and an observation $o \in \Obs$, let $d_i(o) \in \{0, 1\}$ be the binary indicator of the failure of obligation $i$ (where $d_i(o) = 1$ if obligation $i$ is violated, and $0$ otherwise).
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 484 is used outside the 6 allowed Chatman Equations.
Context: where $j: \Obsbot \to \Obsbot \cup \{\Rfsl\}$ transitions $\Raw \to \Val$ by evaluating obligations, and $a: \Obsbot \cup \{\Rfsl\} \to \Obs^* \cup \{\Rfsl\}$ transitions $\Val \to \Admd$ by normalizing the payload.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 558 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism $\muop: \Obs^* \to \Act$, extended to the domain $\Obs^* \cup \{\Rfsl\}$ by propagating the refusal constant such that $\muop(\Rfsl) = \Rfsl$, is a computable map satisfying:
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 558 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism $\muop: \Obs^* \to \Act$, extended to the domain $\Obs^* \cup \{\Rfsl\}$ by propagating the refusal constant such that $\muop(\Rfsl) = \Rfsl$, is a computable map satisfying:
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 558 is used outside the 6 allowed Chatman Equations.
Context: The manufacturing morphism $\muop: \Obs^* \to \Act$, extended to the domain $\Obs^* \cup \{\Rfsl\}$ by propagating the refusal constant such that $\muop(\Rfsl) = \Rfsl$, is a computable map satisfying:
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 566 is used outside the 6 allowed Chatman Equations.
Context: Under determinism (M1), $\muop$ is a mathematical function on $\Obs^*$, and the composite $\muop \circ \adm$ is a function on $\Obsbot$ mapping to $\Act \cup \{\Rfsl\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 566 is used outside the 6 allowed Chatman Equations.
Context: Under determinism (M1), $\muop$ is a mathematical function on $\Obs^*$, and the composite $\muop \circ \adm$ is a function on $\Obsbot$ mapping to $\Act \cup \{\Rfsl\}$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 570 is used outside the 6 allowed Chatman Equations.
Context: By (M1), for any $x_1, x_2 \in \Obs^*$, $x_1 = x_2 \implies \muop(x_1) = \muop(x_2)$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Rec' at line 606 is used outside the 6 allowed Chatman Equations.
Context: A receipt $r \in \Rec$ is a tuple:
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 673 is used outside the 6 allowed Chatman Equations.
Context: A system satisfies the \BRCE\ if for every actuated artifact $a \in \Act$ it enforces four invariants:
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 712 is used outside the 6 allowed Chatman Equations.
Context: The actuation-to-terminal-commitment map $\Psi: \Act \to \{0,1\}^{256}$ associates each actuated artifact $a \in \Act$ with its corresponding terminal receipt-chain hash $h_+ = \Psi(a)$ under the receipt totality invariant (B3).
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 712 is used outside the 6 allowed Chatman Equations.
Context: The actuation-to-terminal-commitment map $\Psi: \Act \to \{0,1\}^{256}$ associates each actuated artifact $a \in \Act$ with its corresponding terminal receipt-chain hash $h_+ = \Psi(a)$ under the receipt totality invariant (B3).
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 717 is used outside the 6 allowed Chatman Equations.
Context: Under \BRCE\ invariants, the map $e \mapsto h_+(e)$ from actuation events (or trace steps) to receipt-chain positions (represented by receipt-chain hashes) is well-defined and injective up to hash collision, and the actuated artifact $a_e \in \Act$ associated with each event $e$ is caused by at least one admitted observation.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 724 is used outside the 6 allowed Chatman Equations.
Context: We prove by induction on the trace length $N$ that for every step $t \in \{1, \dots, N\}$, if an actuation event occurs yielding artifact $a_t$, then $a_t = \muop(x)$ for some admitted observation $x \in \Obs^*$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 726 is used outside the 6 allowed Chatman Equations.
Context: Assume that for all traces of length $k$, any actuated artifact $a_t$ (for $t \le k$) is the image under $\muop$ of some $x \in \Obs^*$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 731 is used outside the 6 allowed Chatman Equations.
Context: Thus, actuation at step $k+1$ is only executed if the incoming observation $o_{k+1}$ is successfully admitted, yielding $x_{k+1} = \adm(o_{k+1}) \in \Obs^*$, and producing $a_{k+1} = \muop(x_{k+1})$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 803 is used outside the 6 allowed Chatman Equations.
Context: The admitted payload $x \in \Obs^*$ compiles deterministically into the delivery record $a = \muop(x)$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 916 is used outside the 6 allowed Chatman Equations.
Context: \item \textbf{Extended Observation Space:} $\Obsbot = \Obs \cup \mathcal{H} \cup \{\Rfsl\}$ (incorporating halted configurations $\mathcal{H}$) is the domain of the admission map.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 917 is used outside the 6 allowed Chatman Equations.
Context: \item \textbf{Morphism Signature:} $\muop: \Obs^* \to \Act$, extended to the domain $\Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 917 is used outside the 6 allowed Chatman Equations.
Context: \item \textbf{Morphism Signature:} $\muop: \Obs^* \to \Act$, extended to the domain $\Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 917 is used outside the 6 allowed Chatman Equations.
Context: \item \textbf{Morphism Signature:} $\muop: \Obs^* \to \Act$, extended to the domain $\Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 917 is used outside the 6 allowed Chatman Equations.
Context: \item \textbf{Morphism Signature:} $\muop: \Obs^* \to \Act$, extended to the domain $\Obs^* \cup \{\Rfsl\} \to \Act \cup \{\Rfsl\}$ with $\muop(\Rfsl) = \Rfsl$.
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 927 is used outside the 6 allowed Chatman Equations.
Context: $\Obs$ & Raw observation space \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 928 is used outside the 6 allowed Chatman Equations.
Context: $\Obsbot$ & Extended observation space ($\Obs \cup \mathcal{H} \cup \{\Rfsl\}$) \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 929 is used outside the 6 allowed Chatman Equations.
Context: $\Obs^*$ & Admitted syntactic subspace \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 930 is used outside the 6 allowed Chatman Equations.
Context: $\adm$ & Admission map / retraction ($\adm: \Obsbot \to \Obs^* \cup \{\Rfsl\}$) \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 932 is used outside the 6 allowed Chatman Equations.
Context: $\muop$ & Manufacturing morphism ($\muop: \Obs^* \to \Act$, extended with $\muop(\Rfsl) = \Rfsl$) \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 932 is used outside the 6 allowed Chatman Equations.
Context: $\muop$ & Manufacturing morphism ($\muop: \Obs^* \to \Act$, extended with $\muop(\Rfsl) = \Rfsl$) \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 933 is used outside the 6 allowed Chatman Equations.
Context: $\Act$ & Artifact/action space \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Rec' at line 934 is used outside the 6 allowed Chatman Equations.
Context: $\Rec$ & Receipt space \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Act' at line 940 is used outside the 6 allowed Chatman Equations.
Context: $\Psi$ & Actuation-to-terminal-commitment map ($\Psi: \Act \to \{0,1\}^{256}$) \\
- ❌ Notation Violation: Calligraphic/Macro symbol '\Obs' at line 963 is used outside the 6 allowed Chatman Equations.
Context: $\Obs$, raw observation & JSON payload check & \code{src/verbs/law.rs} \\

## Verdict

🔴 **FAILED** (Structural / Content failure, Exit Code 3)
