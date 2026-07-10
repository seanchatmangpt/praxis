#!/bin/sh
# Regenerate ontology.ttl as the union of the two source vocabularies.
# ggen sync loads one ontology file per pack (crates/ggen/src/sync.rs pack
# loop: read_to_string + insert_turtle) and does not resolve owl:imports,
# so the union is materialized here. Turtle permits prefix redeclaration.
set -eu
cd "$(dirname "$0")"
{
  echo "# GENERATED union — do not edit. Sources:"
  echo "#   crates/praxis-graphlaw/ontologies/core/ocel2.ttl"
  echo "#   crates/praxis-graphlaw/ontologies/core/bench-obs.ttl"
  echo "# Regenerate: packs/ocel-bench-pack/make-ontology.sh"
  cat ../../crates/praxis-graphlaw/ontologies/core/ocel2.ttl
  echo
  cat ../../crates/praxis-graphlaw/ontologies/core/bench-obs.ttl
} > ontology.ttl
