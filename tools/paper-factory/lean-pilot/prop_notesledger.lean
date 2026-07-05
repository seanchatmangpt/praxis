/-
prop:notesledger

The audit ledger is stored under `refs/notes/praxis/audit`; appending entries
is a commit transition writing hash-chain receipts as NDJSON notes, preserving
immutability of the execution timeline.

We model the ledger as a list of NDJSON note entries (most recent entry
appended at the end, one per line). "Appending entries is a commit
transition ... preserving immutability of the execution timeline" is modeled
as: after appending a new entry to a ledger, the original ledger is exactly
recovered by taking its own length worth of entries from the front of the
new ledger — i.e. every prior entry, in its original order, survives the
append unchanged.
-/

abbrev Entry := String

/-- An audit ledger is a list of NDJSON note entries. -/
abbrev Ledger := List Entry

/-- Appending an entry is a commit transition writing one more NDJSON note. -/
def appendEntry (l : Ledger) (e : Entry) : Ledger :=
  l ++ [e]

/-- Appending preserves immutability of the execution timeline: the original
ledger's entries, in order, are exactly the first `l.length` entries of the
ledger after appending. -/
theorem notesledger_immutable (l : Ledger) (e : Entry) :
    (appendEntry l e).take l.length = l := by
  simp [appendEntry, List.take_left]
