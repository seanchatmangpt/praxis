import os

os.chdir('/Users/sac/praxis/crates/multifractal-workflow')

with open('src/f25_receipts_replay.rs', 'r') as f:
    content = f.read()
content = content.replace("pub fn admit_for_replay(_receipt: &crate::f11_bcinr_runtime::OcelCausalReceipt)", "pub fn admit_for_replay(_correlation_id: &str)")
with open('src/f25_receipts_replay.rs', 'w') as f:
    f.write(content)

with open('src/f26_ontology_self_play.rs', 'r') as f:
    content = f.read()
content = content.replace("(_catalog: &crate::f20_self_play_catalog::DimensionCatalog)", "(_catalog: &OntologyDimensionCatalog)")
with open('src/f26_ontology_self_play.rs', 'w') as f:
    f.write(content)
print("done")
