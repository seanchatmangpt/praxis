# paper-factory engine

A standalone RDF reasoning engine — the "external code" half of praxis's
paper factory: `paper_factory_engine.py` prepares/enriches a graph (SPARQL,
SHACL, OWL-RL, N3 forward-chaining via the `eye` reasoner); `ggen` (the
`tex-math-pack`) never reasons, it only ever extracts (SPARQL SELECT) and
renders (Tera) whatever triples exist by the time it runs.

## Lineage

Extracted from `/Users/sac/dev/.spec-kit.bak/src/specify_cli/ofmf/ofmf_keystone.py`,
which was tested (isolated venv, all declared dialect deps installed) and
found NOT runnable as shipped — two real bugs (a `ShEx` hard-import gate
naming the wrong module, and `SchemaLoader`/`ShExEvaluator` used but never
imported), plus an undeclared dependency on the entire `specify_cli` CLI
package despite its own docstring calling it a "One-File Runtime." This
module keeps only what was verified sound (SPARQL/SHACL/OWL-RL/N3-via-EYE +
a BLAKE3 receipt), drops ShEx and Datalog entirely, and has zero coupling to
`specify_cli`.

## Install

```
pip install -r requirements.txt
```

External: the `eye` N3 reasoner binary on `$PATH`
(`npm install -g eyereasoner`, or a native build of
[github.com/josd/eye](https://github.com/josd/eye)). A missing binary is a
hard `PaperFactoryError`, never a silent skip.

## Test

```
python3 test_paper_factory_engine.py
```

6/6 checks, including a real end-to-end N3 entailment through the actual
`eye` binary (not mocked) and a check that a missing `eye` binary hard-fails
rather than degrading.
