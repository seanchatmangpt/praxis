//! Kernel coverage — all 11 Lord's Prayer clauses as typed nodes, the God
//! boundary as data (never an executable node), and the life-graph queries.

use praxis_synthesis::graph::{extract_ir, parse_ttl, Object, Triple};
use praxis_synthesis::kernel::{extract_kernel, kernel_hash, CANONICAL_CLAUSES, KERNEL_NS};
use praxis_synthesis::{extract_hooks, life, Admission, GraphDelta, Reference, Refusal};

const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
const WF: &str = "http://seanchatmangpt.github.io/praxis/workflow#";
const HOOK: &str = "http://seanchatmangpt.github.io/praxis/hook#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

fn kernel_doc(clauses: &[&str]) -> String {
    let mut doc = format!("@prefix pk: <{KERNEL_NS}> .\n@prefix ex: <http://e/> .\n");
    let list = clauses.iter().map(|c| format!("pk:{c}")).collect::<Vec<_>>().join(", ");
    doc.push_str(&format!("pk:K a pk:Kernel ; pk:clause {list} .\n"));
    for c in clauses {
        doc.push_str(&format!(
            "pk:{c} a pk:Clause ; pk:name \"{c}\" ; pk:problemClass \"p\" ; \
             pk:boundary \"human-only\" .\n"
        ));
    }
    doc
}

#[test]
fn all_11_clauses_extract_and_hash_is_stable_across_reorder() {
    let triples = parse_ttl(KERNEL).expect("kernel parses");
    let clauses = extract_kernel(&triples).expect("kernel extracts");
    assert_eq!(clauses.len(), 11);
    let names: Vec<&str> = clauses.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, CANONICAL_CLAUSES, "canonical scriptural order");
    let h = kernel_hash(&clauses);

    // Surface reorder: reversed triple order yields the same clauses + hash.
    let mut reversed = triples.clone();
    reversed.reverse();
    let clauses2 = extract_kernel(&reversed).expect("reordered kernel extracts");
    assert_eq!(clauses, clauses2);
    assert_eq!(h, kernel_hash(&clauses2), "kernel_hash is surface-order independent");
}

#[test]
fn ten_clause_kernel_refuses_naming_the_missing_clause() {
    let ten: Vec<&str> =
        CANONICAL_CLAUSES.iter().copied().filter(|c| *c != "deliverance").collect();
    let triples = parse_ttl(&kernel_doc(&ten)).expect("parses");
    match extract_kernel(&triples) {
        Err(Refusal::KernelIllFormed { detail, .. }) => {
            assert!(detail.contains("missing clause 'deliverance'"), "detail: {detail}");
        }
        other => panic!("expected KernelIllFormed, got {other:?}"),
    }
}

#[test]
fn unknown_clause_name_refuses() {
    let mut names: Vec<&str> = CANONICAL_CLAUSES.to_vec();
    names[10] = "vain-repetition";
    let triples = parse_ttl(&kernel_doc(&names)).expect("parses");
    match extract_kernel(&triples) {
        Err(Refusal::KernelIllFormed { detail, .. }) => {
            assert!(detail.contains("unknown clause name 'vain-repetition'"), "detail: {detail}");
        }
        other => panic!("expected KernelIllFormed, got {other:?}"),
    }
}

#[test]
fn god_is_never_typed_executable_and_deliverance_is_surrendered() {
    let triples = parse_ttl(KERNEL).expect("kernel parses");
    let executable_types = [
        format!("{WF}Workflow"),
        format!("{WF}Capability"),
        format!("{HOOK}Hook"),
        format!("{WF}Atom"),
    ];
    let handler = format!("{WF}handler");
    for t in &triples {
        // No God/Father IRI is ever typed as an executable node.
        if t.p == RDF_TYPE {
            if let Object::Iri(class) = &t.o {
                if executable_types.contains(class) {
                    // The node's local name must not DENOTE God — human acts
                    // oriented toward God (orient-to-father) are lawful.
                    let local = t.s.rsplit('#').next().unwrap_or(&t.s).to_lowercase();
                    let god_names = [
                        "creator", "almighty", "providence", "yahweh", "elohim",
                        "deity", "god", "father", "ourfather", "our-father", "lord"
                    ];
                    assert!(
                        !god_names.contains(&local.as_str()),
                        "God typed as executable: {t:?}"
                    );
                }
            }
        }
        // No wf:handler binding exists anywhere in the kernel document.
        assert_ne!(t.p, handler, "kernel must not bind handlers: {t:?}");
    }
    let clauses = extract_kernel(&triples).expect("extracts");
    let deliverance = clauses.iter().find(|c| c.name == "deliverance").unwrap();
    assert_eq!(deliverance.boundary, "god-receives-unbounded");
}

/// Adversarial finding: `extract_kernel`'s closed-world sweep only ever
/// inspected the `prayer-kernel:` namespace, so a clause node cross-typed
/// with a SECOND `rdf:type` into `wf:Capability`/`wf:Workflow`/`hook:Hook`
/// sailed through uninspected — the only thing that ever caught the
/// canonical PoC (`pk:our-father` cross-typed) was a name-heuristic test,
/// which a differently-IRI'd clause node defeats trivially. This proves
/// the closed-world check itself now refuses cross-typing by shared
/// subject identity, independent of what the IRI or `pk:name` spell.
#[test]
fn clause_cross_typed_as_executable_is_refused_even_with_a_non_denoting_iri() {
    let mut doc =
        format!("@prefix pk: <{KERNEL_NS}> .\n@prefix wf: <{WF}> .\n@prefix ex: <http://e/> .\n");
    let subject_for = |c: &str| if c == "our-father" { "pk:c1".to_string() } else { format!("pk:{c}") };
    let list = CANONICAL_CLAUSES.iter().map(|c| subject_for(c)).collect::<Vec<_>>().join(", ");
    doc.push_str(&format!("pk:K a pk:Kernel ; pk:clause {list} .\n"));
    for c in CANONICAL_CLAUSES {
        doc.push_str(&format!(
            "{} a pk:Clause ; pk:name \"{c}\" ; pk:problemClass \"p\" ; \
             pk:boundary \"human-only\" .\n",
            subject_for(c)
        ));
    }
    // The attack: cross-type the our-father clause (now living on the
    // non-denoting IRI `pk:c1`) as an executable `wf:Capability` too.
    doc.push_str("pk:c1 a wf:Capability ; wf:name \"c1\" ; wf:params 0 ; wf:cost 1 .\n");

    let triples = parse_ttl(&doc).expect("parses");
    match extract_kernel(&triples) {
        Err(Refusal::KernelIllFormed { subject, detail }) => {
            assert_eq!(subject, format!("{KERNEL_NS}c1"));
            assert!(detail.contains("also typed executable"), "detail: {detail}");
        }
        other => panic!("expected KernelIllFormed cross-typing refusal, got {other:?}"),
    }
}

#[test]
fn life_graph_queries_return_correct_subjects() {
    let doc = format!(
        "@prefix prx: <{ns}> .\n@prefix ex: <http://e/> .\n\
         ex:r1 a prx:ResentmentLoop .\n\
         ex:r2 a prx:ResentmentLoop .\n\
         ex:act1 prx:releases ex:r1 .\n\
         ex:d1 a prx:Debt .\n\
         ex:d2 a prx:Debt .\n\
         ex:amend prx:repairs ex:d2 .\n\
         ex:t1 a prx:UnboundedThreat .\n\
         ex:m1 a prx:ReceiptMissing .\n\
         ex:w a prx:DayWindow .\n\
         ex:task1 prx:scheduledIn ex:w .\n\
         ex:task2 prx:scheduledIn ex:w .\n\
         ex:task3 prx:scheduledIn ex:other .\n",
        ns = life::LIFE_NS
    );
    let triples = parse_ttl(&doc).expect("parses");
    assert_eq!(life::open_resentments(&triples), ["http://e/r2"]);
    assert_eq!(life::open_debts(&triples), ["http://e/d1"]);
    assert_eq!(life::unbounded_threats(&triples), ["http://e/t1"]);
    assert_eq!(life::missing_receipts(&triples), ["http://e/m1"]);
    assert_eq!(
        life::scheduled_in_window(&triples, "http://e/w"),
        ["http://e/task1", "http://e/task2"]
    );
    assert_eq!(life::subjects_of(&triples, life::DAY_WINDOW), ["http://e/w"]);
}

#[test]
fn raw_scripture_is_quarantined_data_not_law() {
    // A raw scripture string admitted as a plain literal triple is DATA:
    // it extracts as no kernel, no hook, and no workflow.
    let base = "@prefix ex: <http://e/> .\nex:day ex:open 1 .\n";
    let reference = Reference::genesis(base).expect("base admits");
    let delta = GraphDelta::parse(
        "<http://e/verse> <http://e/text> \
         \"Our Father who art in heaven, hallowed be thy name\" .",
        "",
    )
    .expect("delta parses");
    let event = Admission::admit(&reference, &delta).expect("admits");

    assert!(event.post().iter().any(|t: &Triple| t.s == "http://e/verse"));
    assert!(
        matches!(extract_kernel(event.post()), Err(Refusal::KernelIllFormed { .. })),
        "scripture text is not a kernel"
    );
    assert!(extract_hooks(event.post()).expect("no hooks").is_empty(), "no hooks fire from prose");
    assert!(extract_ir(event.post()).is_err(), "no workflow extracts from prose");
}
