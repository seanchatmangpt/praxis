#!/bin/sh
# Regenerate chapters_body.tex from RDF instance data. Do not edit
# generated/chapters_body.tex by hand -- edit rdf/thesis.ttl and re-run
# this script.
set -e
cd "$(dirname "$0")/../../../.."
REPO="$(pwd)"
cargo build --release -p ggen -q
rm -rf /tmp/doc-pack-project
mkdir -p /tmp/doc-pack-project/templates /tmp/doc-pack-project/generated
ln -s "$(pwd)/docs/thesis/consolidated/paper_c_scale/rdf/thesis.ttl" /tmp/doc-pack-project/ontology.ttl
ln -s "$(pwd)/packs/doc-pack/templates/document.tex.tmpl" /tmp/doc-pack-project/templates/document.tex.tmpl
cat > /tmp/doc-pack-project/ggen.toml <<EOF
[project]
name = "doc-pack-project"

[ontology]
source = "ontology.ttl"

[ontology.prefixes]
doc = "http://seanchatmangpt.github.io/packs/doc#"

[templates]
dir = "templates"
EOF
(cd /tmp/doc-pack-project && "$REPO/target/release/ggen" sync run)
cp /tmp/doc-pack-project/generated/chapters_body.tex "$REPO/docs/thesis/consolidated/paper_c_scale/generated/chapters_body.tex"
echo "regenerated docs/thesis/consolidated/paper_c_scale/generated/chapters_body.tex"
