/-
ax:refusal

There is a distinguished refusal value `Rfsl` and a space of reasons `R_f`;
a refusal is not the absence of an output but the pair `(Rfsl, r)` with
`r ∈ R_f` a machine-checkable reason, so admission is total as a map into
`Adm ∪ ({Rfsl} × R_f)`.

We model this abstractly: given a type of admitted outputs `Adm`, a type
of refusal reasons `Reasons`, and a distinguished refusal marker `Rfsl`,
there is a total map from an input type `Input` into the disjoint union
`Adm ⊕ (PUnit × Reasons)` (the `{Rfsl} × Reasons` component), i.e. every
input is either admitted or refused-with-reason, never silently dropped.
-/

axiom refusal_totality
    (Input Adm Reasons : Type)
    (Rfsl : PUnit) :
    ∃ f : Input → Adm ⊕ (PUnit × Reasons), True
