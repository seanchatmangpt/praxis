import os

os.chdir('/Users/sac/praxis/crates/multifractal-workflow')

with open('src/f25_receipts_replay.rs', 'r') as f:
    content = f.read()
content = content.replace("(_receipt: &OcelCausalReceipt)", "(_receipt: &crate::f11_bcinr_runtime::OcelCausalReceipt)")
with open('src/f25_receipts_replay.rs', 'w') as f:
    f.write(content)

with open('src/f26_ontology_self_play.rs', 'r') as f:
    content = f.read()
content = content.replace("(_catalog: &DimensionCatalog)", "(_catalog: &crate::f20_self_play_catalog::DimensionCatalog)")
with open('src/f26_ontology_self_play.rs', 'w') as f:
    f.write(content)
print("done")
