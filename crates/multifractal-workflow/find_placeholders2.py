import os
import re

def find_placeholders():
    src_dir = '/Users/sac/praxis/crates/multifractal-workflow/src'
    target_files = [
        'f08_pddl_planning.rs',
        'f08_pddl_planning/hook_binder.rs',
        'f08_pddl_planning/effect_trace.rs',
        'f09_mfw_growth.rs',
        'f11_bcinr_runtime.rs',
        'f12_external_cut.rs',
        'f13_arazzo_artifact.rs',
        'f14_wasm4pm_arazzo.rs',
        'f16_otp_runner.rs',
        'f17_atomvm_runtime.rs',
        'f22_compensation.rs',
        'f24_ocel_construct.rs',
        'f25_receipts_replay.rs',
        'f26_ontology_self_play.rs',
        'f28_multi_breed_science.rs'
    ]
    
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if not file.endswith('.rs'):
                continue
            path = os.path.join(root, file)
            rel_path = os.path.relpath(path, src_dir)
            if not any(rel_path.startswith(tf.replace('.rs', '')) for tf in target_files):
                continue

            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()

            # Find pub fn that return Result<...> and whose body contains NotYetImplemented, unimplemented, or AlwaysRefuse
            func_pattern = re.compile(r'pub fn (\w+)\s*(?:<[^>]+>)?\s*\([^)]*\)\s*->\s*Result<[^>]+>\s*\{[^{}]*\}', re.MULTILINE)
            for match in func_pattern.finditer(content):
                body = match.group(0)
                if 'NotYetImplemented' in body or 'Unimplemented' in body or 'Placeholder' in body or 'Err(' in body:
                    # check if the body is just an Err
                    body_inner = re.search(r'\{([^{}]+)\}', body)
                    if body_inner:
                        inner = body_inner.group(1).strip()
                        if inner.startswith('Err(') and inner.endswith(')'):
                            print(f"{rel_path}: {match.group(1)}")
                        elif 'NotYetImplemented' in inner or 'Placeholder' in inner or 'AlwaysRefuse' in inner:
                            print(f"{rel_path}: {match.group(1)} (by keyword)")
                        
find_placeholders()
