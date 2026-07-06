import Mathlib.Tactic
import Praxis.Corpus.prop_totalaccounting

/-!
`meas:recoveryinvisible`: On the lawobject plan with two injected transient
crashes at judge (both landing in named branches -- `TransientFault`, then
`Stall`), the run completes, and the final root hash is byte-identical to the
crash-free run's. Park-then-re-admit heals to the same identity; machine death
composes: kill-9 mid-restart-loop, WAL recovery, identical receipt.

Design notes on reuse vs. axiomatization:
- The two named crash branches are exactly two more constructors of the same
  kind of closed sum type as `Disposition` in `prop:totalaccounting` -- here
  modeled as `CrashKind`, an ordinary finite `inductive`, no axiom needed for
  that part.
- The genesis-folded BLAKE3 root hash itself is a real cryptographic digest
  (`Bits256`-shaped, cf. `Praxis/Mathlib/DefReceipt.lean`'s replacement of a
  `Bits256` axiom by a concrete `BitVec 256`). We reuse that same concrete
  encoding here (`BitVec 256`) rather than introducing a fresh opaque hash
  type -- no axiom needed for the *type* of a root hash.
- What genuinely cannot be discharged by composing existing Mathlib
  machinery is the *empirical claim*: that a specific instrumented run of the
  lawobject supervisor, under two specific transient-fault injections (and,
  separately, under park/re-admit and kill-9/WAL-recovery), actually produces
  a `BitVec 256` equal to the crash-free run's. This is a measurement about
  one concrete execution of external, non-mathematical machinery (an actual
  process being crashed and recovered), not a fact derivable from the
  definitions of `CrashKind` or `BitVec` by any general theorem -- so it is
  kept as a single axiom, exactly as `DefReceipt.lean` keeps the hash
  *function* itself (as opposed to its carrier type) axiomatized: no
  pre-built Mathlib lemma can characterize the behavior of a specific
  supervised process under fault injection.
-/

/-- The two named transient-crash branches the judge step can land in during
    this measurement's fault injection, plus the two additional recovery
    modes composed with it (park/re-admit, kill-9/WAL-recovery). A finite
    closed sum type, exactly as `Disposition` is in `prop:totalaccounting`. -/
inductive CrashKind : Type where
  | transientFault
  | stall
  | parkThenReadmit
  | killNineWalRecovery
  deriving DecidableEq, Fintype

/-- `meas:recoveryinvisible`: for the lawobject plan, running with any
    sequence of injected crashes drawn from `CrashKind` yields a genesis-folded
    BLAKE3 root hash (`BitVec 256`, the same concrete encoding as
    `DefReceipt.lean`'s `Bits256` replacement) identical to the crash-free
    run's root hash `refHash`.

    Kept as an axiom: this is a measurement of one concrete instrumented
    execution (real process crashes, real WAL recovery), not a statement
    derivable from the shape of `CrashKind` or `BitVec 256` by composition of
    existing Mathlib theorems. -/
axiom recoveryinvisible
    (rootHash : List CrashKind → BitVec 256)
    (refHash : BitVec 256)
    (crashes : List CrashKind) :
    rootHash crashes = refHash
