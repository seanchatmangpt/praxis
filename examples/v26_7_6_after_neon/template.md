# Artifact contract — what `plan run` manufactures

`plan run --goal <ttl> --out-dir <dir>` writes exactly three files to
`<dir>`, all derived from the goal graph (never asserted in):

| File | Content | Provenance binding |
|------|---------|--------------------|
| `domain.pddl` | Manufactured PDDL8 domain text | header comment embeds the source label and the graph's BLAKE3 hash (`mfg::emit_domain`) |
| `problem.pddl` | Manufactured PDDL8 problem text | grounded from the same graph |
| `plan.json` | `{ graph_hash, plan, powl_chain_hash }` | `powl_chain_hash` is the genesis-folded BLAKE3 chain over the executed POWL frames |

Gate: the artifact is only written after `mfg::validate` re-parses,
re-grounds, and re-solves the manufactured text (`solvable == true`);
an unsolvable manufacture is a typed refusal (`"stage": "verify"`), not a
partial write.
