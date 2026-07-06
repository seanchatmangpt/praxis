# Tutorial: verify a real theorem yourself, no math background required

This walks you through getting the actual Lean 4 proof-checking program to confirm one of
the theorems from [the reference table](../reference/00-biggest-theorems-table.md) with
your own eyes, on your own machine. You do not need to understand any of the math to
follow these steps or to trust the result — the whole point of a proof assistant is that
*you* don't have to be the one checking the logic; a small, trusted kernel program does,
and either accepts the file or reports an error.

We'll verify `thm:rice` — "no algorithm can decide any non-trivial meaning-based property
of an observation" (the subject of
[explanation/00-why-nothing-can-understand-you.md](../explanation/00-why-nothing-can-understand-you.md))
— in the bare-core lane, which is the faster of the two lanes to set up.

## Step 1: confirm you have Lean's toolchain

Praxis's Lean work uses `elan`, the Lean version manager, which was already set up for
this project. Check it's on your `PATH`:

```sh
export PATH="$HOME/.elan/bin:$PATH"
lean --version
```

You should see something like `Lean (version 4.31.0, ...)`. If this command isn't found,
you don't have the toolchain installed yet — that's a one-time setup outside the scope of
this tutorial (see `tools/paper-factory/lean-lake/lean-toolchain` for the exact pinned
version this project expects).

## Step 2: look at the file you're about to check

```sh
cat tools/paper-factory/lean-pilot/thm_rice.lean
```

You'll see this is a real `.lean` file: some `axiom` declarations (things assumed true
without proof, standing in for an abstract computability model), a `theorem` declaration,
and a proof written in Lean's tactic language. You don't need to read or understand the
tactic proof — that's exactly the part the kernel is about to check for you.

## Step 3: run the actual kernel check

```sh
cd tools/paper-factory/lean-pilot
lean thm_rice.lean
echo "exit code: $?"
```

If you see no output and `exit code: 0`, that's the whole result: Lean's kernel parsed
every definition, checked every proof step against Lean's own foundational type theory,
and found no gap. This is the same kernel used throughout the corpus's 202-label
migration — nothing about this single-file check is different from what ran, file by
file, across the full corpus.

## Step 4 (optional): see it fail on purpose

To see what a *rejected* proof looks like — so you know the check is real and not just
printing "ok" unconditionally — copy the file and break something:

```sh
sed '61d' thm_rice.lean > /tmp/thm_rice_broken.lean   # deletes "intro ⟨f, hf⟩" from rice_core's proof
lean /tmp/thm_rice_broken.lean
echo "exit code: $?"
rm /tmp/thm_rice_broken.lean
```

You should now see a real Lean error message ("Tactic `apply` failed: could not unify...")
and exit code `1` — confirmed by actually running this exact command while writing this
tutorial. This is what a genuine kernel rejection looks like —
the same kind of error listed for the six labels marked `unformalized` in
[the reference table](../reference/00-biggest-theorems-table.md#reference-the-biggest-theorems-exact-citations-and-verification-status),
just artificially induced here so you can see it happen.

## Step 5 (optional): try the Mathlib-lane version

The Mathlib lane cites Mathlib's own pre-built formalization of Rice's theorem directly,
using zero corpus-specific axioms — a stronger result than the bare-core version, at the
cost of a slower first run (Mathlib's prebuilt cache is several gigabytes and the first
`import Mathlib` in a shell takes tens of seconds even with the cache):

```sh
cd tools/paper-factory/lean-lake
lake env lean Praxis/Mathlib/ThmRiceViaMathlib.lean
echo "exit code: $?"
```

Same idea: exit code `0`, no output, means accepted.

## What you've actually confirmed

You have not taken anyone's word — not this document's, not a receipts file's, not an
agent's self-report — for whether `thm:rice` is really machine-verified. You ran the same
verification command yourself and got the same result independently. This is exactly the
discipline this project tries to hold itself to throughout: a claim of "verified" should
always be reproducible by someone who wasn't in the room when it was first claimed.
