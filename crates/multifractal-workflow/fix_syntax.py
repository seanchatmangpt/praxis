import os

files_to_fix = [
    "src/f08_pddl_planning/hook_binder.rs",
    "src/f09_mfw_growth.rs",
    "src/f11_bcinr_runtime.rs",
    "src/f12_external_cut.rs",
    "src/f13_arazzo_artifact.rs",
    "src/f14_wasm4pm_arazzo.rs",
    "src/f16_otp_runner.rs",
    "src/f17_atomvm_runtime.rs",
    "src/f22_compensation.rs",
    "src/f24_ocel_construct.rs",
    "src/f25_receipts_replay.rs",
    "src/f26_ontology_self_play.rs",
    "src/f28_multi_breed_science.rs"
]

os.chdir('/Users/sac/praxis/crates/multifractal-workflow')

for filepath in files_to_fix:
    if not os.path.exists(filepath):
        continue
    with open(filepath, 'r') as f:
        content = f.read()
    
    # replace "}\n})" with "}\n}" and "})\n" with "}\n" inside the specific functions
    # Actually just replace "Ok(..)\n})" with "Ok(..)\n}"
    import re
    new_content, n = re.subn(r'Ok\((.*?)\)\n\}\)', r'Ok(\1)\n}', content)
    if n > 0:
        with open(filepath, 'w') as f:
            f.write(new_content)
        print(f"Fixed {filepath}")
