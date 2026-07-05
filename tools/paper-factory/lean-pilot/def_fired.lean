/-
def:fired

$\firedmap:\Deny\to\{0,1\}^{8}$ sends a word $d$ to the bit vector whose bit $j$
equals the nonzero-indicator of byte lane $j$ of $d$:
$\firedmap(d)_j=\mathbf 1[(d.0\gg 8j)\ \&\ \text{0xFF}\ne 0]$, computed
branchlessly via $\mathbf 1[x\ne 0]=(x\mathbin{|}(-x))\gg 63$ on each lane.
-/

structure DenialPolarity where
  val : UInt64
deriving DecidableEq, Repr

namespace DenialPolarity

/-- Extract byte lane `j` (0-indexed) of the underlying word. -/
def lane (d : DenialPolarity) (j : UInt64) : UInt64 :=
  (d.val >>> (8 * j)) &&& 0xFF

/-- Branchless nonzero indicator: `1[x ≠ 0] = (x | (-x)) >> 63`, using the
top bit of `x | (-x)` (which is set iff `x ≠ 0` for two's-complement words). -/
def nonzeroIndicator (x : UInt64) : UInt64 :=
  (x ||| (0 - x)) >>> 63

/-- `firedmap` sends a denial word to the bit vector (as a function on
`Fin 8`) whose bit `j` is the nonzero-indicator of byte lane `j` of `d`. -/
def firedmap (d : DenialPolarity) : Fin 8 → Bool :=
  fun j => nonzeroIndicator (d.lane (UInt64.ofNat j.val)) ≠ 0

end DenialPolarity
