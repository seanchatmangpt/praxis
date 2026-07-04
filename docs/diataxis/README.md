# praxis / ggen documentation (Diataxis)

This documentation set follows the [Diataxis](https://diataxis.fr) framework,
which organizes documentation along two axes — practical vs. theoretical, and
study vs. work — into four distinct quadrants:

- **Tutorials** (`tutorials/`) — learning-oriented. Lessons that take a
  newcomer by the hand through a series of steps to complete a meaningful
  project, building confidence and orientation rather than explaining
  everything up front.
- **How-to guides** (`how-to/`) — problem-oriented. Directions for
  accomplishing a specific, real-world task that a user already knows they
  need to do (e.g. "inject into a file", "recover a broken receipt chain").
- **Reference** (`reference/`) — information-oriented. Dry, accurate,
  structured descriptions of the machinery — config schemas, CLI commands,
  error codes, ontology vocabulary — consulted while working, not read
  start-to-finish.
- **Explanation** (`explanation/`) — understanding-oriented. Discussion that
  illuminates *why* the system is built the way it is — its design
  decisions, trade-offs, and the reasoning that connects the parts.

Every factual or behavioral claim about `crates/ggen` in these documents is
cited to a `file:line` location in this repository, and every command
example is real captured output from actually running the command against a
scratch project — nothing here is fabricated.

## Contents

### Tutorials — learning-oriented

| # | File | Topic |
|---|------|-------|
| 1 | [tutorials/01-your-first-sync.md](tutorials/01-your-first-sync.md) | Set up a minimal `ggen.toml` + ontology and run your first `ggen sync run` |
| 2 | [tutorials/02-your-first-pack.md](tutorials/02-your-first-pack.md) | Author and consume a local template pack |
| 3 | [tutorials/03-watch-mode.md](tutorials/03-watch-mode.md) | Run `ggen sync run --watch` and observe debounced regeneration on file changes |
| 4 | [tutorials/04-verifying-receipts.md](tutorials/04-verifying-receipts.md) | Inspect, verify, and chain-check the receipts a sync produces |
| 5 | [tutorials/05-composing-packs.md](tutorials/05-composing-packs.md) | Combine multiple packs in one project |

### How-to guides — problem-oriented

| # | File | Problem solved |
|---|------|-----------------|
| 1 | [how-to/01-inject-into-a-file.md](how-to/01-inject-into-a-file.md) | Inject generated content into an existing file instead of overwriting it |
| 2 | [how-to/02-add-a-cli-verb.md](how-to/02-add-a-cli-verb.md) | Add a new noun-verb command to the `ggen` CLI |
| 3 | [how-to/03-diagnose-corrupted-pack.md](how-to/03-diagnose-corrupted-pack.md) | Diagnose and fix a corrupted or malformed pack |
| 4 | [how-to/04-recover-broken-chain.md](how-to/04-recover-broken-chain.md) | Recover from a broken receipt chain (`FM-CHAIN-*` errors) |
| 5 | [how-to/05-git-packs-not-yet.md](how-to/05-git-packs-not-yet.md) | Work around git-sourced packs not yet being implemented |

### Reference — information-oriented

| # | File | Covers |
|---|------|--------|
| 1 | [reference/01-frontmatter-vocabulary.md](reference/01-frontmatter-vocabulary.md) | Every template frontmatter field (`to`, `sparql`, `inject`, `before`/`after`/`at_line`, `skip_if`, ...) |
| 2 | [reference/02-config-schemas.md](reference/02-config-schemas.md) | The full `ggen.toml` schema (`GgenConfig` and its nested tables) |
| 3 | [reference/03-cli-commands.md](reference/03-cli-commands.md) | Every CLI noun-verb command and its options |
| 4 | [reference/04-error-codes.md](reference/04-error-codes.md) | The `FM-*` error code catalog |
| 5 | [reference/05-ontology-vocabulary.md](reference/05-ontology-vocabulary.md) | The RDF/Turtle ontology vocabulary consumed by `ggen` |

### Explanation — understanding-oriented

| # | File | Discusses |
|---|------|-----------|
| 1 | [explanation/01-why-idempotent.md](explanation/01-why-idempotent.md) | Why sync is designed to be idempotent |
| 2 | [explanation/02-delta-algebra.md](explanation/02-delta-algebra.md) | The delta algebra used to compute what changed between syncs |
| 3 | [explanation/03-status-never-asserted.md](explanation/03-status-never-asserted.md) | Why status is always recomputed, never merely asserted |
| 4 | [explanation/04-self-hosting-loop.md](explanation/04-self-hosting-loop.md) | The self-hosting loop: `ggen` generating parts of its own sources |
| 5 | [explanation/05-llm-never-evidence.md](explanation/05-llm-never-evidence.md) | Why an LLM's say-so is never treated as evidence of correctness |
