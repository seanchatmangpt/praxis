/-
est:wall

At BLAKE3 throughput 1--3 GB/s/core and a few-hundred-byte frame, t_chainH ~ 10^-7 s/frame;
a four-field boundary compare is << 10^-7 s; per-message verification with c ~ 10 spot
frames costs ~ 10^-6 s; these are order-of-magnitude estimates, the constancy-in-T point
is the proved Proposition prop:invariance.

Formalized content: an `estimate` record, expressed in nanoseconds (Nat) to stay in bare
Lean core, pinning down the order-of-magnitude figures quoted in the prose:
  - `tChainHNanos`  : per-frame BLAKE3 recomputation time, ~10^-7 s = 100 ns.
  - `tCmpNanos`     : four-field boundary compare, << 10^-7 s, taken as 1 ns.
  - `spotFrames`    : c ~ 10 spot frames.
  - `wallTimeNanos` : the resulting per-message estimate CL·t_cmp + c·t_chainH, instantiated
    at these figures via the `wallTime` function from prop:invariance, giving ~10^-6 s.

This is an order-of-magnitude estimate, not a theorem: no proof obligation beyond
type-checking. The invariance-in-T claim itself is discharged by `prop_invariance.lean`.
-/

structure ExecutedManufacture where
  T : Nat
  CL : Nat
  tChainH : Nat

def wallTime (tCmp c : Nat) (σ : ExecutedManufacture) : Nat :=
  σ.CL * tCmp + c * σ.tChainH

/-- Order-of-magnitude figures (in nanoseconds) underlying `est:wall`. -/
def tChainHNanos : Nat := 100        -- ~10^-7 s per frame

def tCmpNanos : Nat := 1             -- << 10^-7 s, four-field boundary compare

def spotFrames : Nat := 10           -- c ~ 10 spot frames

/-- A representative executed manufacture with a boundary-field count `CL := 4`
(matching the "four-field boundary compare"), for instantiating the wall-time
estimate at the quoted order-of-magnitude figures. -/
def sampleManufacture : ExecutedManufacture :=
  { T := 0, CL := 4, tChainH := tChainHNanos }

/-- The per-message wall-time estimate instantiated at the quoted figures:
CL · t_cmp + c · t_chainH, in nanoseconds. At CL = 4, t_cmp = 1ns, c = 10,
t_chainH = 100ns this is 4·1 + 10·100 = 1004 ns, i.e. ~10^-6 s, matching the
"~10^-6 s" order-of-magnitude estimate in the prose. -/
def wallTimeNanos : Nat :=
  wallTime tCmpNanos spotFrames sampleManufacture
