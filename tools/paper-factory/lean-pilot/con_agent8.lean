/-
con:agent8 — Fleet status bytes and word-parallel admission sweep.

Project each agent to a status byte `b ∈ {0,1}^8` whose lanes name
`admitted, evidence-ok, within-budget, authority-bound, healthy,
conformant, receipted, replayable`. A fleet of `N` agents is a bit-packed
vector; a 64-bit word holds 8 agents; a fleet admission sweep is the
branchless mask reduction of the denial construction (`con:denial`)
applied word-parallel.

We reuse the `Word`/`wOr`/`wZero` monoid shape from `con:denial` (there
built on `Fin m` lanes for obligations) and instantiate it here with the
fixed 8-lane status byte, then bit-pack `8` agents into one 64-lane word
and define the branchless (no `if`, pure `&&`-fold) fleet admission
sweep as pointwise conjunction across the packed word.

This is a *construction*: the only proof obligation is that the file
type-checks.
-/

/-- The eight named status lanes of one agent's status byte. -/
inductive Lane : Type
  | admitted
  | evidenceOk
  | withinBudget
  | authorityBound
  | healthy
  | conformant
  | receipted
  | replayable
deriving DecidableEq

/-- A single agent's status byte: one bit per lane. -/
def StatusByte : Type := Lane → Bool

/-- An agent is admitted iff every lane of its status byte is set
(branchless: a pure `&&`-fold, no conditional). -/
def byteAdmitted (b : StatusByte) : Bool :=
  b .admitted && b .evidenceOk && b .withinBudget && b .authorityBound &&
  b .healthy && b .conformant && b .receipted && b .replayable

/-- A fleet of `N` agents: a bit-packed vector of status bytes. -/
def Fleet (N : Nat) : Type := Fin N → StatusByte

/-- One 64-bit word holds 8 agents' status bytes packed side by side:
    the pair of an agent-in-word slot (`Fin 8`) and a lane (`Lane`)
    addresses one of the 64 bits. -/
def Word64 : Type := Fin 8 → Lane → Bool

/-- Pack 8 agents (a `Fleet 8`) into one 64-bit word. -/
def pack8 (f : Fleet 8) : Word64 := fun slot lane => f slot lane

/-- Branchless per-slot admission extracted from a packed word. -/
def wordSlotAdmitted (w : Word64) (slot : Fin 8) : Bool :=
  byteAdmitted (w slot)

/-- The fleet admission sweep over one packed word: the branchless mask
reduction (conjunction, componentwise, no branching) of admission across
all 8 packed agents — the word is admitted iff every packed agent is. -/
def wordAdmissionSweep (w : Word64) : Bool :=
  wordSlotAdmitted w 0 && wordSlotAdmitted w 1 && wordSlotAdmitted w 2 &&
  wordSlotAdmitted w 3 && wordSlotAdmitted w 4 && wordSlotAdmitted w 5 &&
  wordSlotAdmitted w 6 && wordSlotAdmitted w 7

/-- Sanity check: the all-admitted word sweeps to `true`. -/
example : wordAdmissionSweep (fun _ _ => true) = true := by
  simp [wordAdmissionSweep, wordSlotAdmitted, byteAdmitted]

/-- Sanity check: a single denied lane in a single slot fails the sweep. -/
example :
    wordAdmissionSweep
      (fun slot lane => if slot = 3 ∧ lane = Lane.healthy then false else true)
      = false := by
  simp [wordAdmissionSweep, wordSlotAdmitted, byteAdmitted]
