-- prop:planetary
-- Formal core of the planetary-control-plane argument built on con:agent8's
-- word-parallel admission sweep: the branchless, word-packed sweep over one
-- 64-bit word is correct — it reports admitted exactly when every one of the
-- 8 packed agents is individually admitted. This is the arithmetic fact
-- (part (a) of the informal statement) that licenses treating a fleet sweep
-- as `N/8` word operations instead of `N` per-agent LLM admission decisions:
-- packing costs nothing semantically, so the feasibility argument for the
-- bit-parallel sweep versus a per-agent-LLM control plane rests on a sound
-- reduction, not a lossy approximation.

/-- The eight named status lanes of one agent's status byte
(reproduced from con:agent8, self-contained per file). -/
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

/-- One 64-bit word holds 8 agents' status bytes packed side by side. -/
def Word64 : Type := Fin 8 → Lane → Bool

/-- Pack 8 agents (a `Fleet 8`) into one 64-bit word. -/
def pack8 (f : Fleet 8) : Word64 := fun slot lane => f slot lane

/-- Branchless per-slot admission extracted from a packed word. -/
def wordSlotAdmitted (w : Word64) (slot : Fin 8) : Bool :=
  byteAdmitted (w slot)

/-- The fleet admission sweep over one packed word: branchless conjunction
across all 8 packed agents. -/
def wordAdmissionSweep (w : Word64) : Bool :=
  wordSlotAdmitted w 0 && wordSlotAdmitted w 1 && wordSlotAdmitted w 2 &&
  wordSlotAdmitted w 3 && wordSlotAdmitted w 4 && wordSlotAdmitted w 5 &&
  wordSlotAdmitted w 6 && wordSlotAdmitted w 7

/-- prop:planetary. Packing a fleet of 8 agents into one 64-bit word and
running the branchless word-parallel sweep is exactly equivalent to every
agent individually being admitted: the word-parallel reduction is a sound,
lossless encoding of per-agent admission. This is the arithmetic fact
underlying the planetary-scale argument — a fleet sweep is `N/8` word
operations that decide exactly the same thing as `N` individual admission
checks, with no approximation, which is what makes the affordability
comparison against per-agent LLM decisions meaningful rather than a
category error. -/
theorem prop_planetary (f : Fleet 8) :
    wordAdmissionSweep (pack8 f) = true ↔
      byteAdmitted (f 0) = true ∧ byteAdmitted (f 1) = true ∧ byteAdmitted (f 2) = true ∧
      byteAdmitted (f 3) = true ∧ byteAdmitted (f 4) = true ∧ byteAdmitted (f 5) = true ∧
      byteAdmitted (f 6) = true ∧ byteAdmitted (f 7) = true := by
  unfold wordAdmissionSweep wordSlotAdmitted pack8
  simp [and_assoc]
