#!/bin/sh
# Regenerate chapters_1_4.tex from RDF instance data. Do not edit
# generated/chapters_1_4.tex by hand -- edit rdf/thesis.ttl and re-run this.
set -e
cd "$(dirname "$0")/../../.."
REPO="$(pwd)"
cargo build --release -p ggen -q
rm -rf /tmp/doc-pack-project
mkdir -p /tmp/doc-pack-project/templates /tmp/doc-pack-project/generated
ln -s "$(pwd)/docs/thesis/math_manufacturing/rdf/thesis.ttl" /tmp/doc-pack-project/ontology.ttl
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
cp /tmp/doc-pack-project/generated/chapters_body.tex "$REPO/docs/thesis/math_manufacturing/generated/chapters_body.tex"
echo "regenerated docs/thesis/math_manufacturing/generated/chapters_body.tex"
