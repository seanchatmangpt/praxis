# Role 2.4: Boole/Algebra Agent - Refusal Algebra

## Structured Notes

**1. Denial words as bit vectors:**
- A refusal is not an exception to be caught but a first-class value. This is the core of "refusal as data".
- Each refusal cause (or lane) is mapped to a specific bit in an $n$-dimensional bit vector.
- Thus, a total denial word $d \in \{0,1\}^n$ compactly encodes the presence or absence of a set of refusal scenarios. 
- $d(o) = \mathbf{0}$ indicates total admission (no causes of refusal), while $d(o) \neq \mathbf{0}$ indicates refusal.

**2. Commutative Idempotent Monoid:**
- We define the denial algebraic structure as $D = (\{0,1\}^n, \lor, \mathbf{0})$.
- **Closure:** The bitwise OR operation $\lor$ on two $n$-bit vectors yields another $n$-bit vector.
- **Associativity:** For any $a, b, c \in \{0,1\}^n$, $(a \lor b) \lor c = a \lor (b \lor c)$.
- **Identity:** The zero vector $\mathbf{0}$ (all bits $0$) acts as the identity element, i.e., $a \lor \mathbf{0} = a$.
- **Commutativity:** For any $a, b \in \{0,1\}^n$, $a \lor b = b \lor a$.
- **Idempotence:** For any $a \in \{0,1\}^n$, $a \lor a = a$.
- Because of these properties, $D$ is a commutative idempotent monoid.
- It is also a bounded join-semilattice, where $\lor$ serves as the supremum (join) operation, naturally capturing the monotonic accumulation of refusal reasons (admitting nothing new, only accumulating denials).

**3. Mapping Refusal Categories into the Denial Lattice:**
- Refusal scenarios $S$ form the discrete, single-bit atoms of the lattice.
- The mapping `category: S -> C` projects specific scenarios into broader categories.
- By assigning each concrete refusal scenario a basis vector in $\{0, 1\}^n$, we can represent any composed denial as a join $\bigvee_i d_i$.
- Because the mapping is total, every scenario has a defined lane, and every single-bit vector (atom) maps back to exactly one scenario.
- Composed denials (elements with $>1$ bit set) represent the simultaneous presence of multiple refusal categories, occupying higher positions in the denial lattice.

---

## Chapter Draft: Refusal Algebra

### 1. Refusal as Data and Denial Words
In order to handle system exceptions without compromising deterministic verification, we adopt the principle of *refusal as data*. Rather than treating rejections as unstructured runtime exceptions, we encode them as computable, first-class values. We define a *denial word* as a bit vector $d \in \{0,1\}^n$. Each dimension $i \in \{1, \dots, n\}$ in the vector corresponds to an independent refusal cause or "lane." For a given observation $o$, the bit $d_i(o) = 1$ if and only if the cause $i$ is present. The state of absolute admission is thus precisely the zero vector, $d = \mathbf{0}$.

### 2. The Denial Monoid
Let us formalize the composition of refusals. We equip the space of denial words with the bitwise logical OR operator, $\lor$. 

**Theorem:** The structure $D = (\{0,1\}^n, \lor, \mathbf{0})$ forms a commutative idempotent monoid.

*Proof:*
1. **Closure and Associativity:** The bitwise OR of two $n$-bit vectors is an $n$-bit vector. The logical OR operator is intrinsically associative.
2. **Identity:** The zero vector $\mathbf{0}$ acts as the monoid identity, since for any bit vector $a$, $a \lor \mathbf{0} = a$.
3. **Commutativity:** The order in which independent refusal reasons occur does not affect the final combined refusal state: $a \lor b = b \lor a$.
4. **Idempotence:** Evaluating the same refusal cause multiple times does not change the refusal state: $a \lor a = a$. 

Consequently, $D$ is a commutative idempotent monoid. Furthermore, this structure is a finite join-semilattice where $\lor$ is the join operation. The monotonic nature of $\lor$ guarantees that composing operations can only strictly enlarge the denial state (accumulating refusals) and never inadvertently convert a refusal back into an admission.

### 3. The Denial Lattice and Category Mapping
Refusal scenarios do not exist in isolation; they are classified into a finite taxonomy of refusal categories. Let $S$ be the finite set of concrete refusal scenarios. We define an injective mapping from $S$ to the single-bit atoms of the denial lattice $D$. That is, each concrete scenario $s \in S$ is assigned a unique single-lane denial vector $d_s \in \{0,1\}^n$, where the Hamming weight of $d_s$ is exactly 1.

The total denial of an observation is constructed by taking the join over all present scenarios:
$$ d_{total} = \bigvee_{s \in present(o)} d_s $$

Because our mapping from $S$ to the atoms of $D$ is total and well-defined, any single-bit denial can be inverted back to a specific scenario, guaranteeing full traceability. For composed denials where the Hamming weight is greater than 1, the element is located higher in the join-semilattice. Such an element does not correspond to a single scenario but represents the exact combination of multiple failure modes. This lattice-theoretic framework allows the runtime to deterministically judge and verify executions, turning error handling into bounded, Boolean algebra.
