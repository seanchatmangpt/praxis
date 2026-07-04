# Master Notation Canon

## The Chatman Equation Thesis Program

## 0. Canonical Equation
$$ \mathcal{A}=\mu(\mathcal{O}^{*}) $$
$$ \mathcal{R}=\operatorname{receipt}(\mathcal{A}) $$

Expanded operational route:
$$ \mathcal{O} \xrightarrow{\alpha} \mathcal{O}^{*} \xrightarrow{\mu} \mathcal{A} \xrightarrow{\operatorname{receipt}} \mathcal{R} $$

Composite prior form:
$$ \mu_{\mathrm{prior}}=\mu\circ\alpha $$

Prior equation:
$$ \mathcal{A}=\mu(\mathcal{O}) $$

Refined equation:
$$ \mathcal{A}=\mu(\mathcal{O}^{*}) \quad\text{with}\quad \mathcal{O}^{*}=\operatorname{im}(\alpha), \qquad \alpha:\mathcal{O}\rightharpoonup\mathcal{O}^{*}\cup\{\bot\} $$

---

## 1. Core Spaces and Maps
- $\mathcal{O}$: Raw observation space (arbitrary finite records, logs, model outputs, human claims, traces, tool responses).
- $\mathcal{O}^{*}$: Admitted observation space (the decidable subspace of observations admitted by finite obligations).
- $\mathcal{A}$: Artifact / action space (outputs actuated by the system).
- $\mathcal{R}$: Receipt space (bounded, collision-committed projections of execution).
- $\Sigma$: Execution trace space.
- $\mu:\mathcal{O}^{*}\to\mathcal{A}$: Manufacturing morphism.
- $\alpha:\mathcal{O}\rightharpoonup\mathcal{O}^{*}\cup\{\bot\}$: Admission map.
- $\rho:\mathcal{O}\rightharpoonup\mathcal{O}^{*}$: Normalization map fixing admitted observations pointwise.
- $\bot$: Distinguished refusal value.
- $\operatorname{dom}(\alpha)$: Domain of the admission map.
- $\operatorname{im}(\alpha)$: Image of the admission map.
- $\operatorname{im}(\mu)$: Image of the manufacturing morphism.

---

## 2. Admission and Retraction
Admission definition:
$$ \alpha(o)= \begin{cases} \rho(o)\in\mathcal{O}^{*}, & \text{if } \bigwedge_{i=1}^{m}g_i(o)=1,\\ \bot, & \text{otherwise.} \end{cases} $$
Obligation battery: $g_1,\dots,g_m:\mathcal{O}\to\{0,1\}$
Admission as retraction: $\alpha|_{\mathcal{O}^{*}}=\operatorname{id}_{\mathcal{O}^{*}}$
Idempotence of admission: $\alpha\circ\alpha=\alpha$
Admitted space as image: $\mathcal{O}^{*}=\operatorname{im}(\alpha)$
Refusal propagation: $\mu(\bot)=\bot$
Operational manufacture: $a=\mu(\alpha(o))$ when $\alpha(o)\neq\bot$
No manufacture from raw observation: $o\notin\mathcal{O}^{*} \implies \mu(o)\ \text{undefined}$

---

## 3. Rice Boundary
Universal computation model: $\mathcal{U}$
Observation-to-program encoding: $e:\mathcal{O}\to\{\mathcal{U}\text{-programs}\}$
Non-trivial semantic property: $P$
Undecidable semantic set: $\{o\in\mathcal{O}:P(o)\}$
Rice specialization: $P \text{ non-trivial semantic} \implies \{o\in\mathcal{O}:P(o)\}\ \text{undecidable}$
Admission cannot decide arbitrary meaning: $\text{Admission}\neq\text{semantic decision}$
Admitted state as bounded finite witness surface: $\mathcal{O}^{*}\subseteq\mathcal{O}$

---

## 4. Denial Monoid
Abstract denial word space: $D_n=\{0,1\}^{n}$
Denial monoid: $D=(\{0,1\}^{n},\lor,\mathbf{0})$
Componentwise join: $d\lor d'$
Bottom / clean word: $\mathbf{0}$
Support of denial word: $\operatorname{supp}(d)=\{i:d_i=1\}$
Algebraic order: $d\preceq d' \iff d\lor d'=d'$
Total denial: $d(o)=\bigvee_i d_i(o)$
Admission by denial bottom: $\alpha(o)\neq\bot \iff d(o)=\mathbf{0}$
Refusal by non-bottom denial: $\alpha(o)=\bot \iff d(o)\neq\mathbf{0}$
Denial lane maps: $d_i:\mathcal{O}\to\{0,1\}^{n}$
Obligation-indexed total denial: $d_G(o)=\bigvee_{g\in G}\delta_g(o)$
Admission set under obligation set $G$: $\mathcal{O}^{*}_{G} = \{o\in\mathcal{O}:d_G(o)=\mathbf{0}\}$
Monotonicity of obligations: $G\subseteq G' \implies d_G(o)\preceq d_{G'}(o)$
Antitonicity of admission: $G\subseteq G' \implies \mathcal{O}^{*}_{G'}\subseteq \mathcal{O}^{*}_{G}$
Denial monoid laws: $d\lor d=d$, $d\lor d'=d'\lor d$, $(d\lor d')\lor d''=d\lor(d'\lor d'')$, $d\lor\mathbf{0}=d$
Bounded join-semilattice: $(D,\lor,\mathbf{0})$
Boolean-lattice support isomorphism: $D_n\cong 2^{[n]}$

---

## 5. Concrete Denial Word / Machine Word
Concrete denial word: $\operatorname{DenialPolarity}\in u64$
Clean word: $\operatorname{ADMITTED}=\operatorname{DenialPolarity}(0)$
Composition: $\operatorname{compose}(a,b) = \operatorname{DenialPolarity}(a.0\ |\ b.0)$
Admission predicate: $\operatorname{is\_admitted}(d) \iff d.0=0$
Named-lane submonoid: $\langle L\rangle\cong D_7$
Fired-mask projection: $\pi_f:D\to\{0,1\}^{8}$
Lane indicator: $\pi_f(d)_j = \mathbf{1}\!\left[ ((d.0\gg 8j)\ \&\ 0xFF)\neq 0 \right]$
Fired-mask homomorphism: $\pi_f(\operatorname{ADMITTED})=\mathbf{0}$
$\pi_f(\operatorname{compose}(a,b)) = \pi_f(a)\lor\pi_f(b)$
Restricted isomorphism: $\pi_f|_{\langle L\rangle}: \langle L\rangle \cong \{0\}\times\{0,1\}^{7} \subseteq \{0,1\}^{8}$

---

## 6. Refusal Taxonomy
Scenario set: $S$
Category set: $C$
Eight-category refusal taxonomy: $C=\{\textsf{Identity}, \textsf{Capacity}, \textsf{Topology}, \textsf{Temporal}, \textsf{Lifecycle}, \textsf{Authorization}, \textsf{Prerequisites}, \textsf{Reserved}\}$
Category map: $\operatorname{cat}:S\to C$
Scenario-to-lane map: $\operatorname{lane}:S\to D$
Lane decoder: $\operatorname{scn}:D\rightharpoonup S$
Single-lane section identity: $\operatorname{lane}\circ\operatorname{scn} = \operatorname{id}$ on named single-lane denial words.
Partial lane decoder: $\operatorname{scn}_{\ell}:D\rightharpoonup S$
Category totality: $\forall s\in S,\quad \operatorname{cat}(s)\in C$
Reserved bucket: $\operatorname{cat}^{-1}(\textsf{Reserved})=\varnothing$

---

## 7. Pipeline Algebra
Free monoid on stages: $\mathrm{Stage}^{*}$
Pipeline aggregate denial: $\Phi_o:\mathrm{Stage}^{*}\to D$
Stage-denial map: $\delta_s(o)\in D$
Aggregate denial over pipeline: $\Phi_o(s_1s_2\cdots s_k) = \delta_{s_1}(o)\lor\delta_{s_2}(o)\lor\cdots\lor\delta_{s_k}(o)$
Empty pipeline: $\Phi_o(\varepsilon)=\mathbf{0}$
Homomorphism law: $\Phi_o(uv)=\Phi_o(u)\lor\Phi_o(v)$
Pipeline admission: $\Phi_o(w)=\mathbf{0} \iff \text{all stages admit }o$
Order independence: $d_1\lor d_2=d_2\lor d_1$
Multiplicity independence: $d\lor d=d$

---

## 8. Logic Admission / Prolog8
Bounded Horn query: $q$
Kernel: $K$
Query result trichotomy: $K(q)\in\{\operatorname{Answered},\operatorname{Denied},\operatorname{Invalid}\}$
Positive proof DAG: $\Pi^{+}$
Negative proof tree: $\Pi^{-}$
Rejection code: $\operatorname{RejectionCode}$
Proof-carrying answer: $\operatorname{Answered}(q,\Pi^{+})$
Proof-carrying denial: $\operatorname{Denied}(q,\Pi^{-})$
Invalid query: $\operatorname{Invalid}(q,\operatorname{RejectionCode})$
Embedding into denial monoid: $\eta:\{\operatorname{Answered},\operatorname{Denied},\operatorname{Invalid}\}\to D$
Answer embeds as bottom: $\eta(\operatorname{Answered})=\mathbf{0}$
Denied / invalid embed as non-bottom: $\eta(\operatorname{Denied})\neq\mathbf{0}$, $\eta(\operatorname{Invalid})\neq\mathbf{0}$

---

## 9. BRCE Byte / CPU Algebra
Boolean set: $\mathbb{B}=\{0,1\}$
Standing byte: $b\in\mathbb{B}^{8}$
Zero byte: $0=00000000_2$
Full standing byte: $1=11111111_2$
Basis bit: $e_i\in\mathbb{B}^{8}, \qquad 0\le i<8$
Standing byte decomposition: $b=b_0e_0\lor b_1e_1\lor\cdots\lor b_7e_7$
Canonical standing lanes: 
$b_0=\text{admitted}$, $b_1=\text{evidenced}$, $b_2=\text{budgeted}$, $b_3=\text{authorized}$, $b_4=\text{healthy}$, $b_5=\text{conformant}$, $b_6=\text{receipted}$, $b_7=\text{replayable}$
CPU operation alphabet: $\mathcal{I}_8=\{\wedge_8, \vee_8, \oplus_8, \neg_8, \ll_k, \gg_k, =_c, \mathrm{z}, \mathrm{nz}, \mathrm{sel}, \mathrm{pop}_8\}$
Bitwise AND, OR, XOR, NOT: $\wedge_8, \vee_8, \oplus_8, \neg_8$
Left/Right shift: $\ll_k, \gg_k$
Constant comparison: $=_c$
Zero/Nonzero test: $\mathrm{z}, \mathrm{nz}$
Branchless select: $\mathrm{sel}$
Population count: $\mathrm{pop}_8$
Lawful hot path: $w\in\mathcal{I}_8^{*}$
Denial byte: $D(b)=\neg_8 b$
Admission gate: $A_{\mathrm{gate}}(b)=\mathrm{z}(D(b))$
Admission condition: $A_{\mathrm{gate}}(b)=1 \iff b=1$
Refusal predicate: $H(b)=\mathrm{nz}(D(b))$
Composed denial: $d_{\Sigma} = d^{(1)}\vee_8 d^{(2)}\vee_8\cdots\vee_8 d^{(n)}$
Admission by composed denial: $d_{\Sigma}=0$
Policy mask: $m\in\mathbb{B}^{8}$
Policy predicate: $P_m(b)=[(b\wedge_8 m)=m]$
Missing lanes: $M_m(b)=m\wedge_8\neg_8 b$
Policy equivalence: $P_m(b)=1 \iff M_m(b)=0$
Repairing lane (i): $b'=b\vee_8 e_i$
Completeness: $C(b)=\mathrm{pop}_8(b)$
Repair distance: $R(b)=8-\mathrm{pop}_8(b)$
Predicate full-byte mask: $\widehat{p} = \begin{cases} 1,&p=1,\\ 0,&p=0. \end{cases}$
Branchless select: $\mathrm{sel}(p,u,v) = (\widehat{p}\wedge_8 u) \vee_8 (\neg_8\widehat{p}\wedge_8 v)$
Byte-to-integer decoder: $\nu(b)=\sum_{i=0}^{7}b_i2^i$
Action selector: $\sigma(b)= \begin{cases} \bot, & \nu(b)=0,\\ a_{\nu(b)}, & 1\le\nu(b)\le255. \end{cases}$
Non-null action bound: $|\operatorname{im}(\sigma)\setminus\{\bot\}|\le255$
Machine cycle quantum: $\tau$
Primitive operation cost: $\operatorname{cost}(i)=1 \qquad i\in\mathcal{I}_8$
Expression depth: $\operatorname{depth}(f)$
Timing approximation: $T(f)\approx \operatorname{depth}(f)\cdot\tau$
Admission check: $G(b)=[b=1]$
Admission depth: $\operatorname{depth}(G)=1$
Policy mask depth: $\operatorname{depth}(P_m)=2$
Picosecond-eligible bound: $\operatorname{depth}(f)\in\{1,2,3\}$
Cycle quantum range: $\tau\in[250,500]\ \mathrm{ps}$

---

## 10. Lifecycle Category / Typestate
Lifecycle category: $\mathbf{Life}$
Lifecycle objects: $\Raw,\quad \Val,\quad \Admd,\quad \Rcpt$
Lifecycle arrows: $j:\Raw\to\Val$, $a:\Val\to\Admd$, $r:\Admd\to\Rcpt$
Lifecycle quiver: $\Raw \xrightarrow{j} \Val \xrightarrow{a} \Admd \xrightarrow{r} \Rcpt$
Full lifecycle composite: $r\circ a\circ j:\Raw\to\Rcpt$
Hom-set: $\mathbf{Life}(X,Y)$
Reachability hom-set law: $|\mathbf{Life}(X,Y)|= \begin{cases} 1,&Y\text{ reachable from }X,\\ 0,&\text{otherwise.} \end{cases}$
Raw to receipted hom-set: $\mathbf{Life}(\Raw,\Rcpt)=\{r\circ a\circ j\}$
Illegal transition: $\mathbf{Life}(X,Y)=\varnothing$
Type interpretation: $\operatorname{LawObject}\langle P,X,L\rangle$
Host type system: $\mathcal{T}$
Illegal transition type: $\operatorname{LawObject}\langle P,X,L\rangle \to \operatorname{LawObject}\langle P,Y,L\rangle$
Uninhabited if: $\mathbf{Life}(X,Y)=\varnothing$
No double receipt: $\Rcpt$ has no outgoing stage-changing arrow.

---

## 11. Token Game / POWL Lifecycle
Token marking: $M\in\{0,1\}^{4}$
Transitions: $T=\{j,a,r\}$
Token state: $M=(\operatorname{TOK\_START}, \operatorname{TOK\_JUDGED}, \operatorname{TOK\_ADMITTED}, \operatorname{TOK\_DONE})$
Input marking: $\mathbf{m}^{-}$
Output marking: $\mathbf{m}^{+}$
Firing update: $M_{\mathrm{next}} = (M\setminus\mathbf{m}^{-})\cup\mathbf{m}^{+}$
Fitness: $\varphi\in[0,1]$
Perfect fitness: $\varphi=1$
Q16.16 perfect fitness: $\varphi=0x0001\_0000$
Out-of-order firing: $\operatorname{TokenNotEnabled} \implies \varphi<1$

---

## 12. Manufacture
Manufacturing morphism: $\mu:\mathcal{O}^{*}\to\mathcal{A}$
Determinism: $x=x' \implies \mu(x)=\mu(x')$
Boundedness: $\mu$ factors through a bounded representation
Manufactured artifact: $a=\mu(x)$
Partial composite on raw observations: $o\mapsto \mu(\alpha(o))$
Image exhaustion: $\mathcal{A}=\operatorname{im}(\mu)$
No side-channel actuation: $a\in\mathcal{A} \implies \exists x\in\mathcal{O}^{*}:a=\mu(x)$

---

## 13. Receipt Chain
Collision-resistant hash: $H:\{0,1\}^{*}\to\{0,1\}^{256}$
BLAKE3 hash: $\mathsf{H}$
Payload bytes: $b$
Payload digest: $\operatorname{dg}(b)=H(b)$
Previous chain value: $h_{-}$
Successor chain value: $h_{+}$
Step-(t) chain value: $h_t$
Genesis commitment: $g=H(0^{32})$
Metadata: $\theta$
Frame: $\mathrm{fr}$
Frame hash body: $\beta(\mathrm{fr})$
Canonical frame: $\mathrm{fr}=\langle\theta,H(b)\rangle$
Byte concatenation: $\Vert$
Receipt chain step: $h_{+}=H(h_{-}\Vert \mathrm{fr})$
Step recurrence: $h_{t+1}=H(h_t\Vert\mathrm{fr}_{t+1})$
Receipt tuple: $r=(\operatorname{verdict},h_{+},\varphi,\operatorname{reason}) \in\mathcal{R}$
Receipt size bound: $\dim(r)\le\kappa$
Receipt ledger / chain: $C$
Receipt lake: $L$
Actuation-to-terminal-commitment map: $\Psi:\mathcal{A}\to\{0,1\}^{256}$ (Replacing $\Phi$ here to avoid overloading)
Injective up to collision: $a\neq a' \land \Psi(a)=\Psi(a') \implies \text{hash collision}$
Negligible collision / forgery probability: $\varepsilon(\lambda)$
Security parameter: $\lambda=256$
Embedded verifying key: $k$
Pinned verifying key: $k^{\star}$
Signing key: $sk$
Obligation battery: $G$
Obligation digest: $\operatorname{dg}(G)$
OCEL export map: $\Omega$

---

## 14. Cryptographic Claims
Chain commitment: $H(h_{-}\Vert\mathrm{fr}) = H(h'_{-}{\Vert}\mathrm{fr}') \land (h_{-},\mathrm{fr})\neq(h'_{-},\mathrm{fr}') \implies \text{collision of }H$
Prefix binding: $h_t \text{ binds } (\mathrm{fr}_1,\dots,\mathrm{fr}_t)$
Tamper detection: $\mathrm{fr}_i\neq\mathrm{fr}'_i \implies h_t\neq h'_t$ except with probability $\varepsilon(\lambda)$
Integrity boundary: $\text{Receipt integrity}\neq\text{virtue}$
Receipt proves committed trace structure: $\text{receipt}\Rightarrow \text{unaltered, attributed, conformant-as-checked}$
Receipt does not prove: $\text{wise obligations}$, $\text{good artifact}$, $\text{true observation}$.

---

## 15. BRCE Invariants
BRCE: $\mathrm{BRCE}$
B1 — Admission gate: $\forall a\in\mathcal{A},\quad \exists o\in\mathcal{O}, x=\alpha(o)\neq\bot, a=\mu(x)$
B2 — Bounded manufacture: $\mu$ is deterministic and bounded
B3 — Receipt totality: $\forall a\in\mathcal{A},\quad \exists! r\in\mathcal{R}: r=\operatorname{receipt}(a)$
B4 — Conformance: $\varphi(a)=1$
Conservation of Consequence: $a\mapsto h_{+}(a)$ is well-defined and injective up to hash collision.
No consequence without admitted cause: $a\in\mathcal{A} \implies \exists x\in\mathcal{O}^{*}:a=\mu(x)$
No consequence without receipt: $a\in\mathcal{A} \implies \exists! h_{+}(a)$
Overclaim set: $\mathcal{V} = \{a\in\mathcal{A}: \nexists r=\operatorname{receipt}(a)\}$
Antibody clause: $\mathcal{V}=\varnothing$
Claim magnitude bound: $\operatorname{claim}(a) \preceq \operatorname{receipt}(a)$

---

## 16. Comprehension–Verification Gap
Human working-memory / verification capacity: $\kappa$
Trace dimension: $\dim\Sigma$
Verification cost: $C_V$
Comprehension cost: $C_C$
Bounded verification: $C_V=O(\kappa)$
Unbounded / trace-scale comprehension: $C_C=\Omega(\dim\Sigma)$
Gap: $\Gamma = \frac{C_C}{C_V}$
Canonical gap form: $\Gamma = \frac{\Omega(\dim\Sigma)}{O(\kappa)}$
Divergence: $\dim\Sigma\to\infty \implies \Gamma\to\infty$
Interior token count: $N$
Receipt field count: $\rho$
Measured gap estimate: $\Gamma_N=\frac{N}{\rho}$
Canonical instance: $N\approx2.5\times10^{6}$, $\rho\approx4$, $\Gamma_N\approx6\times10^{5}$

---

## 17. Planetary / Fleet Admission
Agent population: $N_{\mathrm{agents}}$
Agent byte: $b_a\in\mathbb{B}^{8}$
Fleet state: $B\in(\mathbb{B}^{8})^{N_{\mathrm{agents}}}$
Machine word width: $w=64$
Agents per machine word per lane: $64$
Fleet admission sweep: $\bigvee_{a=1}^{N_{\mathrm{agents}}}D(b_a)$
Fleet admits iff: $\bigvee_{a=1}^{N_{\mathrm{agents}}}D(b_a)=0$
Population byte memory: $M_{\mathrm{bytes}}=N_{\mathrm{agents}}$
For $10^{10}$ agents: $M_{\mathrm{bytes}}=10^{10}\ \mathrm{bytes}=10\ \mathrm{GB}$
Admission decision rate: $10^{8}\text{--}10^{10} \quad \text{decisions/sec}$
Branchless reduction depth: $7$
Bit-parallel affordability condition: $C_{\mathrm{bitparallel}}\ll C_{\mathrm{comprehension}}$

---

## 18. Planning Geometry
Plan / execution trace: $\sigma$
Trace space: $\Sigma$
Marking: $M$
Transition incidence matrix: $N$
Firing count vector: $x$
Initial marking: $M_0$
Final marking: $M_f$
State equation: $M_f=M_0+Nx$
Lawful marking polytope: $\mathcal{P}$
Polytope constraints: $Ax\le b$
Conformance as membership: $x\in\mathcal{P}$
Violation: $x\notin\mathcal{P}$
Separating hyperplane: $y$
Farkas certificate: $y^{\top}A=0, \qquad y^{\top}b<0$
Verification of certificate: $O(1)$ relative to trace length, under fixed certificate dimension
Fitness as distance / normalized conformance: $\varphi(\sigma)\in[0,1]$
Perfect conformance: $\varphi(\sigma)=1$

---

## 19. Projection Principle
Projection: $\pi$
Faithful projection: $\pi:\Sigma\to\mathcal{R}$
Receipt projection: $r=\pi(\sigma)$
Projection dimension bound: $\dim\operatorname{codom}(\pi)\le\kappa$
Faithfulness condition: $\pi(\sigma)=\pi(\sigma') \implies \sigma=\sigma'$ up to accepted cryptographic collision boundary.
Collision-bounded faithfulness: $\pi(\sigma)=\pi(\sigma') \land \sigma\neq\sigma' \implies \text{collision}$
Verifier: $V$
Verifier decision: $V(r)\in\{\operatorname{accept},\operatorname{reject}\}$
Verifier cost: $\operatorname{cost}(V)=O(\kappa)$

---

## 20. Manufactured Trust / Oracle Retraction
Correctness question: $Q$
Undecidable semantic target: $Q_{\infty}$
Decidable oracle: $\mathcal{O}_{\mathrm{oracle}}$
Retraction onto oracle: $\beta:Q_{\infty}\rightharpoonup\mathcal{O}_{\mathrm{oracle}}\cup\{\bot\}$
Differential agreement oracle: $\mathcal{D}(x) = [f_1(x)=f_2(x)]$
False-accept probability: $p_{\mathrm{fa}}$
Boundary fuzzing input space: $\mathcal{F}$
Mutation operator: $m$
Mutant set: $\mathcal{M}$
Validator: $V$
Mutant killed: $\operatorname{kill}(m) \iff V(m(\sigma))=\operatorname{reject}$
Mutant Kill Theorem: $\operatorname{kill}(m) \iff \text{validator rejects at correct stage}$ under staged soundness and completeness assumptions.

---

## 21. ggen / Projection Engine Notation
Artifact: $A$
Ontology: $O$
Runtime log: $L$
Three-pole isomorphism: $A\cong O\cong L$
Forward projection: $\mu:O\to A$
Inverse projection: $\mu^{-1}:A\to O$
Runtime-log projection: $\lambda:A\to L$
Closure operator: $\mu_2$
Idempotence: $\mu_2\circ\mu_2=\mu_2$
Filesystem state: $F_n$
Next filesystem state: $F_{n+1}$
ggen projection: $F_{n+1} = \mu(O^{*},C^{*},P^{*},T^{*},F_n)$
Configuration: $C$
Packs: $P$
Templates: $T$
Filesystem delta: $\Delta F=F_{n+1}-F_n$
Sync idempotence: $\operatorname{sync}(\operatorname{sync}(F))= \operatorname{sync}(F)$
Receipt for filesystem mutation: $R_{n+1}=\operatorname{receipt}(F_{n+1}-F_n)$
Delta: $\Delta$
Delta inverse: $\Delta^{-1}$
Delta cancellation: $\Delta\oplus\Delta^{-1}=0$
Three-way merge: $\operatorname{merge}(B,L,R)$ where $B$ is base, $L$ is left mutation, and $R$ is right mutation.

---

## 22. Reality Addressing
Content address: $\operatorname{addr}_{\mathrm{content}}(x)=H(x)$
Reality address: $\operatorname{addr}_{\mathrm{reality}}(r)$
Receipt address: $\operatorname{addr}_{\mathrm{receipt}}(a)=H(\operatorname{receipt}(a))$
Address triple: $(\operatorname{addr}_{\mathrm{content}}, \operatorname{addr}_{\mathrm{reality}}, \operatorname{addr}_{\mathrm{receipt}})$
Public ontology binding: $\operatorname{bind}: \mathcal{O}^{*}\to \mathrm{PROV}\times\mathrm{OWLTime}\times\mathrm{GeoSPARQL}$
Reality-address refusal: $\operatorname{RealityAddressIllFormed}(s)$
Public coordinate requirement: $\exists t,w,p: (s,t)\in\mathrm{OWLTime} \lor (s,w)\in\mathrm{GeoSPARQL} \lor (s,p)\in\mathrm{PROV}$
No public anchor: $\neg\exists t,w,p \implies \bot_{\mathrm{reality}}$

---

## 23. Standing and Claim Discipline
Claim: $c$
Standing predicate: $\operatorname{Standing}(c)$
Receipted standing: $\operatorname{Standing}(c) \iff \exists r\in\mathcal{R}:r=\operatorname{receipt}(c)$
Unsupported claim: $c\notin\operatorname{im}(\operatorname{receipt})$
Refusal of unsupported claim: $c\mapsto\bot$
Claim standing classes: $\{\operatorname{PROVED}, \operatorname{CITED}, \operatorname{IMPLEMENTED}, \operatorname{PARTIAL\_ALIVE}, \operatorname{UNSUPPORTED}, \operatorname{REFUSED}\}$

---

## 24. Minimal Symbol Table

| Symbol | Meaning |
| :--- | :--- |
| $\mathcal{O}$ | raw observation space |
| $\mathcal{O}^{*}$ | admitted observation space |
| $\alpha$ | admission map / retraction |
| $\rho$ | normalization map |
| $\bot$ | refusal |
| $g_i$ | obligation predicate |
| $d_i$ | denial lane map |
| $d(o)$ | total denial word |
| $D$ | denial monoid |
| $\lor$ | denial composition / join |
| $\mathbf{0}$ | clean denial word |
| $\operatorname{supp}(d)$ | fired-lane support |
| $\pi_f$ | fired-mask projection |
| $S$ | refusal scenario set |
| $C$ | refusal category set |
| $\operatorname{cat}$ | scenario-to-category map |
| $\operatorname{lane}$ | scenario-to-denial-lane map |
| $\operatorname{scn}$ | lane-to-scenario decoder |
| $\mu$ | manufacturing morphism |
| $\mathcal{A}$ | action / artifact space |
| $\Sigma$ | execution trace space |
| $\mathcal{R}$ | receipt space |
| $H$ | collision-resistant hash |
| $\operatorname{dg}$ | digest |
| $\mathrm{fr}$ | receipt frame |
| $h_-$ | previous chain hash |
| $h_+$ | next chain hash |
| $h_t$ | chain hash at step $t$ |
| $g$ | genesis commitment |
| $\theta$ | frame metadata |
| $\varphi$ | conformance fitness |
| $\kappa$ | bounded human verification capacity |
| $\mathbf{Life}$ | lifecycle category |
| $\Raw,\Val,\Admd,\Rcpt$ | lifecycle stages |
| $j,a,r$ | judge, admit, receipt arrows |
| $M$ | token marking |
| $\mathbf{m}^{-},\mathbf{m}^{+}$ | consumed / produced token markings |
| $P_m$ | policy-mask predicate |
| $M_m$ | missing-lane mask |
| $b$ | BRCE standing byte |
| $e_i$ | basis bit |
| $\mathcal{I}_8$ | register-operation alphabet |
| $\nu$ | byte-to-integer decoder |
| $\sigma$ | action selector |
| $\tau$ | machine cycle quantum |
| $\lambda$ | cryptographic security parameter |
| $\varepsilon(\lambda)$ | negligible failure probability |
| $\pi$ | projection |
| $\Gamma$ | comprehension–verification gap |
| $N$ | population / interior size, context-dependent |
| $F_n$ | filesystem state at sync step $n$ |
| $\Delta$ | delta |
| $\Delta^{-1}$ | inverse delta |
| $\Omega$ | OCEL export map |
| $\Psi$ | actuation-to-terminal-commitment map (formerly overloaded as $\Phi$) |
| $\Phi$ | pipeline aggregate denial |

---
**CRITICAL MANDATE FOR REWRITE SWARM:** 
1. **Avoid reusing $\Phi$**. Use $\Phi$ exclusively for pipeline aggregate denial. I have replaced the actuation-to-terminal commitment map with $\Psi$ to resolve the collision. 
2. **Reserve Calligraphic $\mathcal{A}, \mathcal{O}, \mathcal{R}$** exclusively for the Chatman Equation. When discussing `ggen`, use plain $A, O, L$.
