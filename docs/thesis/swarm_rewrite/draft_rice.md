# Rice/Turing Agent - Computability Boundary Notes & Draft

## Structured Notes

### 1. The Exact Semantic Decision Problem Being Refused
The Chatman Equation refuses the semantic decision problem of evaluating "what an observation means". Formally, an observation $o \in \mathcal{O}$ is a finite record (e.g., a JSON payload) that may denote arbitrary logic or partial functions. The system refuses the problem of deciding if an observation satisfies any non-trivial semantic property $P$ of its denoted function (i.e., "does this observation mean what it purports to mean?").

### 2. The Rice Theorem Specialization to Observations
Let $\mathcal{U}$ be a universal model of computation, and let the observation space $\mathcal{O}$ range over finite encodings that may denote arbitrary $\mathcal{U}$-programs. Let $P$ be any non-trivial semantic property of those denoted functions (meaning $P$ depends only on the function, not the encoding, and is true for some functions and false for others). 
**Theorem:** The set $\{o \in \mathcal{O} : P(o)\}$ is structurally undecidable.

### 3. Proof: Why Admission Cannot Be Semantic Understanding
1. An admission procedure must be a total and terminating algorithm—it must definitively admit or refuse any given observation $o \in \mathcal{O}$.
2. If an admission procedure admitted observations by understanding their meaning, it would act as a decider for a non-trivial semantic property $P$.
3. By the specialized Rice's theorem, no such decider can exist for arbitrary observations.
4. **Conclusion:** Admission cannot be semantic decision. A total, terminating procedure must instead evaluate a *syntactic*, decidable surrogate. Admission is not "understanding"; it is the mathematical retraction of the observation space $\mathcal{O}$ onto a restricted, decidable sub-language (the "Rice quarantine"), structurally refusing any observation outside that fragment.

---

## Chapter Draft: The Computability Boundary

The foundation of the Bounded Receipted Chatman Equation rests on the recognition of a strict computability boundary. The observation space $\mathcal{O}$ is composed of arbitrary finite records—raw outputs, logs, or external tool payloads. A naïve agentic system might attempt to map these records to lawful actions by inferring or verifying their semantic meaning. However, this approach encounters a hard theoretical limit.

We specialize Rice's theorem to observations: if $\mathcal{O}$ contains encodings that denote universal computation, any non-trivial semantic property of those denoted functions is undecidable. There exists no algorithm that can determine if an arbitrary observation structurally "means" what it purports to mean. 

Consequently, the process of *admission*—the filtering of raw observations into a trusted subspace—cannot be a semantic decision. Any total, terminating procedure must be restricted to decidable predicates. Thus, admission is redefined not as an attempt to understand an observation, but as a rigid syntactic retraction. The system retracts $\mathcal{O}$ onto a finite, decidable sub-language (a "Rice quarantine"). If an observation falls within this decidable fragment (for example, bounded Horn logic without mutation), it is admitted; if it falls outside, it is refused. 

By enforcing this boundary, the agent abandons semantic omniscience in favor of computable obligations, guaranteeing that every standing action is derived from a bounded, logically quarantined, and fully receipted derivation.
