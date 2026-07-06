import Mathlib.Data.List.Infix

/-!
# `prop:notesledger`

The audit ledger is stored under `refs/notes/praxis/audit`; appending entries
is a commit transition writing hash-chain receipts as NDJSON notes, preserving
immutability of the execution timeline.

We model the ledger as a `List String` (one NDJSON line per receipt entry).
Appending a new entry `e` to ledger `l` is `l ++ [e]`. "Preserving immutability
of the execution timeline" is exactly the statement that the prior ledger `l`
is a prefix of the ledger after appending: no previously-written entry is
altered or removed by the append transition, only a new one is added at the
end. This is a direct instance of Mathlib's `List.prefix_append`
(`l <+: l ++ l'`), so no axiom is introduced.
-/

namespace Praxis.Corpus

/-- Append one NDJSON receipt entry to the audit ledger. -/
def notesLedgerAppend (l : List String) (e : String) : List String :=
  l ++ [e]

/-- Appending an entry to the audit ledger preserves the immutability of the
existing execution timeline: the ledger before the append is a prefix of the
ledger after the append. -/
theorem notesLedgerAppend_preserves_prefix (l : List String) (e : String) :
    l <+: notesLedgerAppend l e :=
  List.prefix_append l [e]

end Praxis.Corpus
