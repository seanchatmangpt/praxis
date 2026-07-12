import os
import re

def find_always_refusing():
    src_dir = '/Users/sac/praxis/crates/multifractal-workflow/src'
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if not file.endswith('.rs'):
                continue
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Find any functions containing 'Err(Refusal::' or 'is_honestly_unimplemented'
            func_defs = re.findall(r'pub fn (\w+)[^{]*\{(?:[^{}]*\{[^{}]*\}[^{}]*)*\}', content)
            
            # Since regex for nested braces is hard, let's just grep lines with 'Err(' and the preceding 'pub fn'
            lines = content.split('\n')
            current_fn = None
            for line in lines:
                m = re.match(r'^pub fn (\w+)', line)
                if m:
                    current_fn = m.group(1)
                elif current_fn and ('Err(Refusal' in line or 'todo!' in line or 'Err(crate::' in line or 'Err(' in line):
                    if 'Placeholder' in line or 'Unimplemented' in line or 'Always' in line:
                        print(f"{file}: {current_fn} might be a placeholder: {line.strip()}")
            
            for m in re.finditer(r'fn (\w+)_is_honestly_unimplemented', content):
                print(f"{file} test: {m.group(1)}")
                
find_always_refusing()
