# Why Sync Must Be Idempotent

`ggen sync` is the operator that turns an ontology and a set of templates
into files on disk. The single property that makes it trustworthy enough
to run in a loop, in CI, or after a merge conflict, is that running it
twice must not do anything the first run didn't already do:

```
sync(sync(F)) = sync(F)
```

This document is about why that law is non-negotiable, what "equal"
actually means once you look closely at the write layer, and where the
honest edges of the guarantee are.

## The five stages, and where idempotence has to hold

`sync` is documented as five stages — Resolve, Enrich, Extract, Render,
Write (`/Users/sac/praxis/crates/ggen/src/sync.rs:1`) — composed in that
order inside the `sync` function
(`/Users/sac/praxis/crates/ggen/src/sync.rs:100`). The module doc is
explicit that Enrich is currently a **single pass**, not iterated to a
fixed point: "constructs that depend on other constructs' output require a
second `sync` run" (`/Users/sac/praxis/crates/ggen/src/sync.rs:4-6`). That
sentence alone tells you why `sync∘sync = sync` can't be assumed as a
free theorem — it has to be *engineered*, stage by stage, and the module
docs are honest that Enrich's closure isn't finished yet.

`docs/ggen-theory.md` gives the algebraic reason this matters at all.
Stage `μ₂` (inference/enrich) is supposed to be a **closure operator**:
extensive, monotone, and idempotent —
`μ₂(μ₂(O)) = μ₂(O)` — and "idempotence is the algebraic reason 'run
inference twice' must be a no-op — if it isn't, `μ₂` isn't a closure, and
downstream determinism has no foundation" (ggen-theory.md §1, "Objects."
paragraph on `μ₂`). The single-pass caveat in `sync.rs` is the concrete,
present-day gap between the aspirational closure property and what's
actually implemented: today, a construct that reads another construct's
output needs the operator applied twice by hand to reach the fixed point,
which is precisely the thing a closure operator is supposed to make
unnecessary.

`docs/ggen-theory.md` §3 ("Calculus: differential and integral
generation") goes further and explains *why* this specific idempotence is
load-bearing for the whole pipeline, not just for stage `μ₂` in isolation:
"The chain rule across `μ₁…μ₅`... because `μ = μ₅∘μ₄∘μ₃∘μ₂∘μ₁`, a change at
`μ₁`... propagates through every later stage. This is why `μ₂`'s
idempotence... matters for the calculus too: if inference weren't
idempotent, 'small change in, run inference, run generation' would not
have a stable meaning... which breaks the well-definedness of `∂μ/∂Δ`
itself" (ggen-theory.md §3, "The chain rule across μ₁…μ₅" paragraph). In
other words: idempotence at the sync level isn't a nice-to-have UX
property ("don't annoy the user with spurious diffs"). Without it, the
notion of "the effect of one incremental change" stops being well-defined,
because the same delta could produce different downstream artifacts
depending on how many times some earlier stage happened to already have
run. The whole incremental-generation story in §3 (`∂μ/∂Δ ≈ μ(O⊕Δ) − μ(O)`)
presupposes that re-running a stage on already-stable input is a no-op.

## Where the real decision gets made: `plan_write`

Enrich and Render can be as pure and closure-like as you like; none of
that matters if the last stage, Write, doesn't also refuse to churn the
filesystem on a repeat run. `plan_write` in
`/Users/sac/praxis/crates/ggen/src/write.rs:50` is where every rendered
body meets the actual disk, and its module docs state the decision order
explicitly as "first match wins"
(`/Users/sac/praxis/crates/ggen/src/write.rs:9-16`):

1. path escapes root / traversal → `Err`
2. `unless_exists` && target exists → `Skipped`
3. `skip_if` substring present in existing file → `Skipped`
4. `inject` → insert into existing file → `Err` if target/marker missing
5. `force` → overwrite → `Written`
6. default: absent → `Written`; identical → `Skipped`; differs → `Err`

Read that order again, specifically rule 5 versus rule 6. Rule 6 is the
one that actually encodes "if the file already has exactly the content we
were about to write, do nothing" — it's implemented as the third arm of
the `match existing` block, `Some(ref content) if content ==
rendered_body => Ok(WriteOutcome::Skipped("unchanged: content
identical"...))` (`/Users/sac/praxis/crates/ggen/src/write.rs:105-107`).
But rule 5 — `force` — sits *before* rule 6 in the match, and its arm
`Some(_) if frontmatter.force => { std::fs::write(...); Ok(WriteOutcome::Written) }`
(`/Users/sac/praxis/crates/ggen/src/write.rs:101-104`) never even inspects
whether the content is unchanged. It writes unconditionally whenever the
target already exists and `force` is set, full stop.

### The finding, stated honestly

That ordering has a direct, observable consequence: a template with
`force: true` in its frontmatter reports `"written"` on *every* sync, even
the second, third, and Nth run against unchanged input. This isn't a
hypothetical read of the code — it is exactly what the property test in
the workspace documents and what running the binary confirms.

The test comment states it as a finding, not a bug report:

> "FINDING (documented-table consistent, but worth knowing): with
> `force: true` the write decision hits rule 5 (force → overwrite) BEFORE
> the identical-content Skip of rule 6, so a re-sync of force-templates
> reports 'written' again rather than 'skipped: unchanged'. Idempotence
> therefore holds at the byte / receipt-payload level (asserted here), not
> at the decision level."
> (`/Users/sac/praxis/crates/ggen/tests/combinatorial_matrix.rs:287-295`)

I reproduced this directly rather than trust the comment. A scratch
project under `/tmp/ggen-idempotence-demo` with two templates — one plain,
one `force: true`, both projecting the same SPARQL row set — gives this
after `ggen sync run` twice in a row (binary built at
`/Users/sac/praxis/target/debug/ggen` via `cargo build -p ggen` from
`/Users/sac/praxis`):

Run 1 decisions (from `.ggen-v2/receipt.json`):
```json
{
  "out/forced.txt": "written",
  "out/plain.txt": "written"
}
```

Run 2 decisions, same project, no source changes:
```json
{
  "out/forced.txt": "written",
  "out/plain.txt": "skipped: unchanged: content identical"
}
```

And the byte content, checked with `shasum -a 256` before and after the
second run:
```
646475fa548bc09dd60753b2b61ccbc628ac0029559e9addb1d8bd8fab5e2840  out/plain.txt
646475fa548bc09dd60753b2b61ccbc628ac0029559e9addb1d8bd8fab5e2840  out/forced.txt
646475fa548bc09dd60753b2b61ccbc628ac0029559e9addb1d8bd8fab5e2840  out/plain.txt
646475fa548bc09dd60753b2b61ccbc628ac0029559e9addb1d8bd8fab5e2840  out/forced.txt
```

Identical hashes before and after. The `force` template's *decision label*
flips from `written` to `written` again (never settling to `skipped`), but
the *bytes on disk* — and, as the property test also asserts, the entire
receipt payload — are byte-for-byte the same as the previous run
(`/Users/sac/praxis/crates/ggen/tests/combinatorial_matrix.rs:310-323`).
The `graph_hash_hex` in both receipts above is also identical
(`8268314e9dac2e4d998be16002ba502b6989f6b0850210e75b3d3d20fb147d71`),
because `graph_hash_hex` is computed once from the post-Enrich graph
state, independent of the per-template write decisions
(`/Users/sac/praxis/crates/ggen/src/sync.rs:145`).

### State-idempotent vs. trace-idempotent

This is worth naming precisely, because "idempotent" is doing two
different jobs here and conflating them would be dishonest:

- **State-idempotent** — the *observable outcome* (the bytes at each
  output path, the graph hash, the receipt payload) is a fixed point:
  `state(sync(sync(F))) = state(sync(F))`. This is what the property test
  asserts and what the reproduction above confirms, for every template
  regardless of `force`.
- **Trace-idempotent** — the *decision label* recorded for how that state
  was reached (`written` vs. `skipped: unchanged`) is also a fixed point.
  This holds for ordinary templates (rule 6 kicks in on the second run)
  but *not* for `force: true` templates, because rule 5 always wins the
  match before rule 6 gets a chance to compare content.

`sync(sync(F)) = sync(F)` as stated at the top of this document is true in
the state-idempotent sense unconditionally, and true in the
trace-idempotent sense only for the non-`force` path. A `force` write is,
by construction, an escape hatch that says "don't ask, just overwrite" —
and the price of that escape hatch is that the write *log* can no longer
distinguish "nothing changed" from "I clobbered it again with the same
bytes." The write module's own doc comment calls `force` overwrite exactly
what it is — rule 5, unconditional, ahead of the identical-content check —
so this isn't a hidden quirk; it's a documented and now empirically
confirmed consequence of where `force` sits in the precedence order
(`/Users/sac/praxis/crates/ggen/src/write.rs:9-16`).

### Why this ordering, and not the other one

It would be possible to special-case rule 5 to check for identical content
first and only report `written` when content actually changed. The
current code doesn't do that, and there's a real trade-off behind leaving
it alone: `force` exists as an unconditional override *because* the
default path (rule 6) already refuses to clobber differing content —
`Some(_) => Err(AppError::fm_write(5, ...))`
(`/Users/sac/praxis/crates/ggen/src/write.rs:108-115`), with the
remediation message telling the operator to "set `force: true` to
overwrite intentionally." `force` is the deliberate bypass of that
refusal. Making `force` re-derive "is this actually a no-op" before
overwriting would mean the bypass silently reintroduces the same content
comparison it exists to skip — at which point `force` and the default
path converge, and the frontmatter flag stops meaning what its name says.
Leaving rule 5 unconditional keeps `force`'s semantics simple and
auditable ("this template always writes, no exceptions") at the cost of a
noisier decision trace. That is a defensible choice, but it is a choice,
and it's the kind of thing that should be visible to anyone reading the
receipt log and expecting `written` to mean "content changed."

## Connecting back to the receipt chain

The reason any of this is checkable at all, rather than a claim you have
to take on faith, is the receipt mechanism built in `write_receipt`
(`/Users/sac/praxis/crates/ggen/src/sync.rs:414`). Every non-dry-run sync
binds every decision target that exists on disk into a
`graph_hash` + per-output BLAKE3 map, chains a `ReceiptRecord` over that
payload via `recompute_chain_hash`
(`/Users/sac/praxis/crates/ggen/src/sync.rs:476-479`), and appends it to
`.ggen-v2/receipt-log.jsonl`
(`/Users/sac/praxis/crates/ggen/src/sync.rs:66-67`,
`/Users/sac/praxis/crates/ggen/src/sync.rs:488-501`). `docs/ggen-theory.md`
§1.2 makes the receipt hash a homomorphism rather than an oracle, and §1.3
treats chained hashes as a monoid homomorphism into a chain — the
practical upshot is that re-syncing an unchanged project doesn't just
*look* the same, it produces a receipt whose payload hash you can
literally diff against the previous one. That's what the property test
does: it reads `receipt.payload` bytes back out via `serde_json::to_vec`
and asserts equality across two consecutive syncs
(`/Users/sac/praxis/crates/ggen/tests/combinatorial_matrix.rs:278-281`,
`/Users/sac/praxis/crates/ggen/tests/combinatorial_matrix.rs:319-323`).
Idempotence, in this codebase, isn't asserted by a docstring — it's a
property with a machine-checkable witness, and the witness is honest
enough to record that `force` templates keep re-announcing "written" even
when nothing actually happened underneath.

## The broader point

`sync(sync(F)) = sync(F)` matters because ggen is meant to be run
repeatedly against a moving ontology, not once against a fixed one. A CI
job that runs `ggen sync` on every commit, a developer who runs it out of
habit before opening a PR, a watch-mode loop that re-triggers on every
filesystem event (`/Users/sac/praxis/crates/ggen/src/watch.rs`) — none of
these callers can afford a generator that treats "nothing changed" as
license to rewrite files, bump timestamps, or grow the receipt log with
spurious churn. The Enrich stage's documented single-pass limitation, and
the Write stage's documented `force`-before-identical precedence, are
both places where the theory in `docs/ggen-theory.md` §1 and §3 states
what *should* hold and the implementation is transparent about exactly how
far it currently gets there. That transparency — module docs that name
the gap, a property test that names the finding instead of hiding it, and
a receipt chain that makes the claim falsifiable — is what makes it
possible to trust the idempotence law in the first place, precisely
because the one place it's incomplete is documented rather than papered
over.
