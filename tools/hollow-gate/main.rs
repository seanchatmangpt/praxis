//! Hollow gate: blocks unimplemented!/todo!/placeholder stubs from reaching CI,
//! and programmatically verifies structural conformance to Post-Chatman principles.
//!
//! Exit 0 = clean & conforming. Exit 1 = issues/non-conformance detected.

const BLOCKING: &[(&str, &str)] = &[
    ("unimplemented!", "HOLLOW-001"),
    ("todo!",          "HOLLOW-002"),
    ("// TODO:",         "HOLLOW-004"),
    ("// FIXME:",        "HOLLOW-005"),
    ("// PLACEHOLDER",   "HOLLOW-006"),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Default to scanning "src" in current directory if no path argument provided
    let scan_dir = if args.len() > 1 {
        format!("{}/src", args[1])
    } else {
        "src".to_string()
    };

    println!("Scanning directory: {} for stubs and structural conformance...", scan_dir);

    let mut found_blocking = false;
    
    // Structural conformance trackers
    let mut has_phantom_data = false;
    let mut has_raw_zst = false;
    let mut has_validated_zst = false;
    let mut has_admitted_zst = false;
    let mut has_evidence_wrapper = false;
    let mut has_admit_trait = false;
    let mut has_rule_pack_server_impl = false;

    // We will walk the scan_dir
    let walk_dir = walkdir::WalkDir::new(&scan_dir);
    for entry in walk_dir.into_iter().flatten() {
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        
        let path = entry.path();
        let src = std::fs::read_to_string(path).unwrap_or_default();
        
        // Scan for forbidden stubs
        for (line_no, line) in src.lines().enumerate() {
            // First check the comment patterns on the original line
            for &(pat, code) in BLOCKING {
                if pat.starts_with("//") {
                    if line.contains(pat) {
                        eprintln!(
                            "[{}] {}:{}: {}",
                            code,
                            path.display(),
                            line_no + 1,
                            line.trim()
                        );
                        found_blocking = true;
                    }
                }
            }

            // Strip single-line comments (any text after // on a line)
            let stripped_line = if let Some(idx) = line.find("//") {
                &line[..idx]
            } else {
                line
            };

            // Scan the stripped line for todo! and unimplemented!
            for &(pat, code) in BLOCKING {
                if !pat.starts_with("//") {
                    if stripped_line.contains(pat) {
                        eprintln!(
                            "[{}] {}:{}: {}",
                            code,
                            path.display(),
                            line_no + 1,
                            line.trim()
                        );
                        found_blocking = true;
                    }
                }
            }
        }
        
        // Scan for structural conformance elements
        // 1. PhantomData
        if src.contains("PhantomData") {
            has_phantom_data = true;
        }
        // 2. ZST marker Raw: pub struct Raw; or struct Raw; or impl sealed::LifecycleState for Raw
        if src.contains("struct Raw;") || src.contains("struct Raw") {
            has_raw_zst = true;
        }
        // 3. ZST marker Validated
        if src.contains("struct Validated;") || src.contains("struct Validated") {
            has_validated_zst = true;
        }
        // 4. ZST marker Admitted
        if src.contains("struct Admitted;") || src.contains("struct Admitted") {
            has_admitted_zst = true;
        }
        // 5. Evidence wrapper: struct Evidence
        if src.contains("struct Evidence") {
            has_evidence_wrapper = true;
        }
        // 6. Admit trait: trait Admit
        if src.contains("trait Admit") {
            has_admit_trait = true;
        }
        // 7. RulePackServer implementation
        if src.contains("impl RulePackServer for") {
            has_rule_pack_server_impl = true;
        }
    }

    println!("\n=== Structural Conformance Checklist ===");
    println!("  [+] PhantomData typestates:       {}", if has_phantom_data { "DETECTED" } else { "MISSING" });
    println!("  [+] Raw ZST marker:               {}", if has_raw_zst { "DETECTED" } else { "MISSING" });
    println!("  [+] Validated ZST marker:         {}", if has_validated_zst { "DETECTED" } else { "MISSING" });
    println!("  [+] Admitted ZST marker:          {}", if has_admitted_zst { "DETECTED" } else { "MISSING" });
    println!("  [+] Evidence wrapper:             {}", if has_evidence_wrapper { "DETECTED" } else { "MISSING" });
    println!("  [+] Admit trait:                  {}", if has_admit_trait { "DETECTED" } else { "MISSING" });
    println!("  [+] RulePackServer impl:          {}", if has_rule_pack_server_impl { "DETECTED" } else { "MISSING" });

    let mut structural_conformance_failed = false;
    if !has_phantom_data {
        eprintln!("[ERROR] Missing: PhantomData typestates");
        structural_conformance_failed = true;
    }
    if !has_raw_zst {
        eprintln!("[ERROR] Missing: Raw ZST marker");
        structural_conformance_failed = true;
    }
    if !has_validated_zst {
        eprintln!("[ERROR] Missing: Validated ZST marker");
        structural_conformance_failed = true;
    }
    if !has_admitted_zst {
        eprintln!("[ERROR] Missing: Admitted ZST marker");
        structural_conformance_failed = true;
    }
    if !has_evidence_wrapper {
        eprintln!("[ERROR] Missing: Evidence wrapper");
        structural_conformance_failed = true;
    }
    if !has_admit_trait {
        eprintln!("[ERROR] Missing: Admit trait");
        structural_conformance_failed = true;
    }
    if !has_rule_pack_server_impl {
        eprintln!("[ERROR] Missing: RulePackServer implementation");
        structural_conformance_failed = true;
    }

    if found_blocking {
        eprintln!("\n[CONFORMANCE FAILURE] Forbidden placeholder/todo/unimplemented patterns detected.");
    }
    if structural_conformance_failed {
        eprintln!("\n[CONFORMANCE FAILURE] Crate does not conform structurally to Post-Chatman principles.");
    }

    if found_blocking || structural_conformance_failed {
        std::process::exit(1);
    }

    println!("\n[CONFORMANCE SUCCESS] Generated project successfully verified!");
}
