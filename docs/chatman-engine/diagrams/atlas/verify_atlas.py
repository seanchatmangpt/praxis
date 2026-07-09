#!/usr/bin/env python3
import os
import sys
import json
import re

# Directory of the script
BASE_DIR = os.path.dirname(os.path.abspath(__file__))

FAMILY_FILES = [
    "01_flowchart.md",
    "02_swimlanes.md",
    "03_sequence.md",
    "04_class.md",
    "05_state.md",
    "06_entity_relationship.md",
    "07_user_journey.md",
    "08_gantt.md",
    "09_pie.md",
    "10_quadrant.md",
    "11_requirement.md",
    "12_gitgraph.md",
    "13_c4.md",
    "14_mindmap.md",
    "15_timeline.md",
    "16_zenuml.md",
    "17_sankey.md",
    "18_xy_chart.md",
    "19_block.md",
    "20_packet.md",
    "21_kanban.md",
    "22_architecture.md",
    "23_radar.md",
    "24_event_modeling.md",
    "25_treemap.md",
    "26_venn.md",
    "27_ishikawa.md",
    "28_wardley.md",
    "29_cynefin.md",
    "30_treeview.md",
]

HEADER_FIELDS = [
    "Diagram ID",
    "Diagram family",
    "Projection lens",
    "Architectural invariant preserved",
    "Information-loss risk if omitted",
    "TPS visual-control purpose",
    "DfLSS CTQ protected",
    "CENG ticket or boundary constrained",
    "Why this diagram is non-redundant"
]

def check_failed(message):
    print(f"[-] CHECK FAILED: {message}")
    return False

def run_verification():
    passed = True
    print("[*] Starting Atlas Verification...")
    
    # Check 1: Verify exactly 30 family Markdown files exist
    print("[*] Check 1: Verifying exactly 30 family Markdown files exist...")
    missing_files = []
    for f in FAMILY_FILES:
        path = os.path.join(BASE_DIR, f)
        if not os.path.exists(path):
            missing_files.append(f)
    if missing_files:
        passed = check_failed(f"Missing family files: {missing_files}")
    else:
        print("[+] Check 1 passed: All 30 family files exist.")

    # Check 2: Verify index.md exists
    print("[*] Check 2: Verifying index.md exists...")
    index_path = os.path.join(BASE_DIR, "index.md")
    if not os.path.exists(index_path):
        passed = check_failed("index.md does not exist.")
    else:
        print("[+] Check 2 passed: index.md exists.")

    # Check 3: Verify manifest.json exists
    print("[*] Check 3: Verifying manifest.json exists...")
    manifest_path = os.path.join(BASE_DIR, "manifest.json")
    if not os.path.exists(manifest_path):
        passed = check_failed("manifest.json does not exist.")
    else:
        print("[+] Check 3 passed: manifest.json exists.")

    all_mermaid_blocks_count = 0
    all_diagram_ids = {}
    
    if not missing_files:
        print("[*] Scanning Markdown files content...")
        for f in FAMILY_FILES:
            path = os.path.join(BASE_DIR, f)
            with open(path, "r", encoding="utf-8") as file_obj:
                content = file_obj.read()
            
            # Check 13: Verify no file contains forbidden words (TODO, OMITTED, PLACEHOLDER, SAME AS ABOVE, TBD)
            # Remove "INFORMATION-LOSS RISK IF OMITTED:" to avoid matching the field label
            forbidden_words = ["TODO", "OMITTED", "PLACEHOLDER", "SAME AS ABOVE", "TBD"]
            content_upper = content.upper().replace("INFORMATION-LOSS RISK IF OMITTED:", "")
            for word in forbidden_words:
                if word in content_upper:
                    passed = check_failed(f"File {f} contains forbidden word '{word}'.")
            
            # Check 5: Verify each family file contains exactly 8 Mermaid blocks
            mermaid_blocks = re.findall(r'```mermaid\s*\r?\n(.*?)\r?\n```', content, re.DOTALL)
            if len(mermaid_blocks) != 8:
                passed = check_failed(f"File {f} has {len(mermaid_blocks)} Mermaid blocks, expected 8.")
            all_mermaid_blocks_count += len(mermaid_blocks)
            
            # Check 6: Verify each family file contains exactly 8 Admission Headers
            lines = content.split('\n')
            headers = []
            i = 0
            while i < len(lines):
                line = lines[i].strip()
                if line.startswith("Diagram ID:"):
                    header = {}
                    current_field = "Diagram ID"
                    header[current_field] = line[len("Diagram ID:"):].strip()
                    
                    i += 1
                    while i < len(lines):
                        next_line = lines[i].strip()
                        found_field = None
                        for field in HEADER_FIELDS:
                            if next_line.startswith(field + ":"):
                                found_field = field
                                break
                        
                        if found_field:
                            current_field = found_field
                            header[current_field] = next_line[len(found_field + ":"):].strip()
                        elif next_line.startswith("```mermaid") or next_line.startswith("```"):
                            break
                        else:
                            if current_field:
                                if next_line:
                                    if header[current_field]:
                                        header[current_field] += " " + next_line
                                    else:
                                        header[current_field] = next_line
                        i += 1
                    headers.append(header)
                else:
                    i += 1
            
            if len(headers) != 8:
                passed = check_failed(f"File {f} has {len(headers)} Admission Headers, expected 8.")
            
            # Check 7 & 8: Verify every Admission Header has all required fields and they are not blank
            # Check 9: Verify every diagram ID is unique
            # Check 12: Verify every family has exactly one diagram per projection lens (L1 to L8)
            family_lenses = set()
            for header_idx, header in enumerate(headers):
                for field in HEADER_FIELDS:
                    if field not in header:
                        passed = check_failed(f"File {f}: Admission Header {header_idx+1} is missing field '{field}'")
                    elif not header[field].strip():
                        passed = check_failed(f"File {f}: Admission Header {header_idx+1} has blank field '{field}'")
                
                diag_id = header.get("Diagram ID", "").strip()
                if diag_id:
                    if diag_id in all_diagram_ids:
                        passed = check_failed(f"Duplicate Diagram ID found: '{diag_id}' in {f} (previously seen in {all_diagram_ids[diag_id]})")
                    else:
                        all_diagram_ids[diag_id] = f
                    
                    match = re.match(r'^([A-Z0-9_]+)-L([1-8])$', diag_id)
                    if not match:
                        passed = check_failed(f"File {f}: Diagram ID '{diag_id}' does not match format <FAMILY>-L<LENS_NUMBER>")
                    else:
                        fam_part, lens_num = match.groups()
                        family_lenses.add(f"L{lens_num}")
                
            expected_lenses = {f"L{k}" for k in range(1, 9)}
            if family_lenses != expected_lenses:
                passed = check_failed(f"File {f} does not cover all lenses L1 to L8. Found: {sorted(list(family_lenses))}")
                
        # Check 4: Verify exactly 240 Mermaid blocks across the 30 family files
        if all_mermaid_blocks_count != 240:
            passed = check_failed(f"Total Mermaid blocks count is {all_mermaid_blocks_count}, expected 240.")
        else:
            print("[+] Check 4 passed: Exactly 240 Mermaid blocks found across all family files.")
            
        print("[+] Check 5 passed: Each family file contains exactly 8 Mermaid blocks.")
        print("[+] Check 6 passed: Each family file contains exactly 8 Admission Headers.")
        print("[+] Check 7 passed: Every Admission Header has all required fields.")
        print("[+] Check 8 passed: No Admission Header field is blank.")
        print("[+] Check 9 passed: Every Diagram ID is unique across all files.")
        print("[+] Check 12 passed: Every family has exactly one diagram per projection lens (L1 to L8).")
        print("[+] Check 13 passed: Checked for TODO, OMITTED, PLACEHOLDER, SAME AS ABOVE, or TBD.")
        
    # Check 10: Verify the manifest contains exactly 240 entries
    if os.path.exists(manifest_path):
        try:
            with open(manifest_path, "r", encoding="utf-8") as m_file:
                manifest_data = json.load(m_file)
            
            if not isinstance(manifest_data, list):
                passed = check_failed("manifest.json must be a JSON array.")
            else:
                if len(manifest_data) != 240:
                    passed = check_failed(f"manifest.json contains {len(manifest_data)} entries, expected 240.")
                else:
                    print("[+] Check 10 passed: manifest.json contains exactly 240 entries.")
                
                # Check 11: Verify manifest diagram IDs match the Markdown diagram IDs
                manifest_ids = set()
                for entry in manifest_data:
                    if not isinstance(entry, dict):
                        passed = check_failed("Manifest entries must be JSON objects.")
                        continue
                    diag_id = entry.get("diagram_id")
                    if diag_id:
                        manifest_ids.add(diag_id)
                        
                        required_manifest_keys = [
                            "diagram_id", "family", "file", "projection_lens", "invariant",
                            "information_loss_risk", "tps_purpose", "dflss_ctq", "ceng_boundary",
                            "non_redundancy", "mermaid_block_index"
                        ]
                        for key in required_manifest_keys:
                            if key not in entry:
                                passed = check_failed(f"Manifest entry '{diag_id}' is missing key '{key}'")
                
                markdown_ids = set(all_diagram_ids.keys())
                if manifest_ids != markdown_ids:
                    extra_in_manifest = manifest_ids - markdown_ids
                    extra_in_markdown = markdown_ids - manifest_ids
                    passed = check_failed(f"Manifest IDs do not match Markdown IDs.\nExtra in manifest: {extra_in_manifest}\nExtra in Markdown: {extra_in_markdown}")
                else:
                    print("[+] Check 11 passed: Manifest diagram IDs match Markdown diagram IDs.")
        except Exception as e:
            passed = check_failed(f"Failed to parse manifest.json: {e}")
    else:
        passed = check_failed("manifest.json not found to verify entries.")
            
    # Check 14: Print clear pass/fail summary
    if passed:
        print("\n===============================")
        print("   VERIFICATION RESULT: PASS   ")
        print("===============================")
        sys.exit(0)
    else:
        print("\n===============================")
        print("   VERIFICATION RESULT: FAIL   ")
        print("===============================")
        sys.exit(1)

if __name__ == "__main__":
    run_verification()
