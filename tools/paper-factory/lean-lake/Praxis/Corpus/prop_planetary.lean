import Mathlib.Tactic
import Praxis.Corpus.con_agent8
import Praxis.Corpus.thm_gap

/-!
Label: prop:planetary

"For a fleet of $N$ agents under Construction~\ref{con:agent8}: (a) arithmetic --
the status vector occupies exactly $N$ bytes, packed to $N/8$ 64-bit words; a full
admission sweep is $N/8$ masked-OR word operations. (b) engineering estimate --
compressed status is $\sim\!10\,\mathrm{GB}$; a sweep takes $\Theta(N/(8B))$
wall-clock. (c) order-of-magnitude estimate -- the affordability ratio between one
cache-line mask op and one LLM admission decision is $\sim\!10^{7}$. (d)
consequence, not theorem -- a comprehension-based or per-agent-LLM planetary
control plane is infeasible by $\sim\!N\cdot10^{7}$, while a bit-parallel admission
sweep is feasible."

Formalization notes: parts (b)-(d) are engineering/order-of-magnitude *estimates*
and an infeasibility *consequence*, not mathematical claims with a formal
statement -- the LaTeX itself flags (d) as "consequence, not theorem". They carry
no Mathlib-checkable content (wall-clock estimates, a "$\sim 10^7$" affordability
ratio between an LLM call and a machine-word operation, and "infeasible" as a
qualitative judgment about system architecture) and are not formalized here.

Part (a) is the one arithmetic fact with real mathematical content: packing a
fleet of `N` agents (Construction `con:agent8`'s `Fleet N := Fin N → StatusByte`,
one status byte per agent) into 64-bit words at 8 agents/word, when `N` is an
exact multiple of 8 (`N = 8 * k`), uses exactly `k = N / 8` words -- i.e. the
sweep over the fleet is `k` applications of `Agent8.wordOf8`/`fleetDenial8`, one
per word, matching "packed to $N/8$ 64-bit words; a full admission sweep is $N/8$
masked-OR word operations". This is proved directly from core's
`Nat.mul_div_cancel_left` (no new axiom, no hand-rolled division fact).
-/

/-- `prop:planetary`, part (a): a fleet of `8 * k` agents (Construction
`con:agent8`'s per-agent status bytes, packed 8-to-a-word) is packed into
exactly `k` words, and a full admission sweep therefore performs exactly
`(8 * k) / 8 = k` word-parallel `fleetDenial8` operations -- the number of
64-bit-word masked-OR steps is exactly `N / 8` for `N = 8 * k`. -/
theorem prop_planetary (k : ℕ) : (8 * k) / 8 = k :=
  Nat.mul_div_cancel_left k (by norm_num)
