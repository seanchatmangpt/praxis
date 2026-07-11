#!/bin/sh
# Regenerate ontology.ttl as the union of the source vocabularies/instances.
# ggen sync loads one ontology file per pack (crates/ggen/src/sync.rs pack
# loop: read_to_string + insert_turtle) and does not resolve owl:imports,
# so the union is materialized here. Turtle permits prefix redeclaration.
set -eu
cd "$(dirname "$0")"
{
  echo "# GENERATED union — do not edit. Sources:"
  echo "#   crates/cng/ontologies/arazzo.ttl"
  echo "#   crates/cng/examples/arazzo-api-orchestration.ttl"
  echo "#   packs/arazzo-pack/engines-local.ttl"
  echo "# Regenerate: packs/arazzo-pack/make-ontology.sh"
  cat ../../crates/cng/ontologies/arazzo.ttl
  echo
  cat ../../crates/cng/examples/arazzo-api-orchestration.ttl
  echo
  cat engines-local.ttl
} > ontology.ttl
