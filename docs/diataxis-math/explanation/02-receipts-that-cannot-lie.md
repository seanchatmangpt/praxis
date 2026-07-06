# Receipts that cannot lie

**Source:** `docs/thesis/02_receipt_cryptography.tex`. Three theorems: `thm:faithful`
(line 387), `thm:conservation` (line 445), `thm:localize` (line 522).

## The plain-English version

Every time Praxis takes an action, it writes down a "receipt" — a small record of what
happened — and chains it cryptographically to every receipt before it, the same basic
idea as a blockchain's block-linking (each new record's hash depends on the previous
record's hash, all the way back to a fixed starting point called genesis). This paper
proves three properties that chain has to have for the receipts to actually be trustworthy
evidence, rather than just a log file someone could quietly edit.

### 1. You can't tamper with a receipt without either breaking the hash or getting caught (`thm:faithful`)

Suppose someone edits a single field in a single old receipt — say, changes what an
action's outcome was — but leaves the *final* chain hash looking exactly the same as
before, hoping no one notices. The theorem proves this is only possible if the underlying
cryptographic hash function (BLAKE3) has been broken — i.e., if two genuinely different
pieces of data hash to the same value, which cryptographers consider astronomically
unlikely by design. In other words: as long as the hash function itself holds up, *any*
edit to a committed field necessarily changes the final chain hash. A silent, undetectable
edit isn't just hard — it's provably as hard as breaking the hash function itself.

### 2. Every action that happened is provably caused by something that was actually admitted (`thm:conservation`)

This is a "no artifacts appear out of nowhere" guarantee. Every actuated outcome traces
back to exactly one admitted input; nothing gets receipted unless it genuinely passed the
admission gate first (this is enforced at the type level in the code — a "receipt" method
literally does not exist on an object that hasn't reached the admitted state, so calling it
on anything else is a compile error, not a runtime check that could be skipped). Each
action also advances the chain by exactly one link — no action is silently dropped, and
none is double-counted.

### 3. If tampering happens, the exact record it happened at is what gets flagged — never an innocent one (`thm:localize`)

This is the practical payoff of the first two properties. If a chain does get perturbed,
the verification process reports the *least* index at which something breaks — the exact
record where the tampering happened, whatever specific way it happened (a malformed field,
a stale hash, records swapped out of order, or even a coordinated attempt to forge both a
field and its stored hash together). Crucially: every record *before* that point is
provably untouched by the check. You never get a "something's wrong somewhere in this
whole file" answer — you get "here, exactly, record #j."

## Verification status

- `thm:faithful` is **deliberately excluded** from both Lean lanes. This isn't an oversight
  — the theorem's content is fundamentally about a cryptographic hash function's real-world
  collision resistance, a probabilistic, computational-hardness claim. Neither Lean's bare
  core nor Mathlib has a verified BLAKE3 implementation or a formal cryptographic-hardness
  framework to cite, so mechanizing it would mean either faking a hash (meaningless) or
  building out a verified cryptographic library from scratch (genuinely out of scope). The
  gap is named honestly rather than papered over with a fake axiom.
- `thm:conservation` and `thm:localize` are both **blocked** in both lanes — their proofs
  depend on other statements that themselves haven't been machine-verified yet, so per the
  project's own discipline, an unverified dependency means the downstream theorem stays
  unattempted rather than resting on an unproven foundation.

So this paper's results are proved rigorously on paper, in the classical mathematical
sense (each with a full written proof you can read directly in the `.tex` source), but
none of its three headline theorems currently has independent machine confirmation — an
honestly-stated gap, not a hidden one. See
[../reference/00-biggest-theorems-table.md](../reference/00-biggest-theorems-table.md).
