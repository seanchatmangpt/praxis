//! Benchmark corpus generation: worker roster, workload artifact sets, and
//! the 8-ary recursion tree, all seeded/deterministic and written as real
//! Turtle fixtures (never mocked or bypassed).

use std::fs;
use std::path::Path;

use crate::powl::CngRefusal;

use super::templates::{load_templates, BenchConfig, GenerateReport, Templates};
use super::{
    fill_template, short_hex, splitmix64, CATEGORIES, RWAI_PREFIX, WORKERS_PER_ROSTER_PARTITION,
};

/// Writes one workflow artifact set (2 domain fragments + up to 2 problem
/// fragments) for `worker`, category-flavored, with optional recursive
/// attachment triples pointing at `children` and an optional injected
/// missing-problem refusal case.
///
/// PROJ-609: for the content-bearing categories (`interruption`,
/// `planning`), the first domain fragment additionally carries the rendered
/// `bench-category-<category>.template.ttl` fragment pointing at
/// `content_target` (falling back to the set's own case IRI when the caller
/// supplies none — the target is always a real in-corpus IRI).
///
/// PROJ-611: when `omit_final_problem` is set, the final problem fragment is
/// rendered but WITHHELD (returned, not written) — it is the exact minimal
/// admission a bounded admission request can later grant. The Fortune-5 path
/// discards it (terminal refusal unchanged); the workday loop uses it.
///
/// Returns `(files_written, bytes_written, withheld_final_problem)`.
///
/// # Complexity
/// O(fragment bytes) template substitutions; O(children) attachment lines.
#[allow(clippy::too_many_arguments)]
pub(super) fn write_set(
    templates: &Templates,
    dir: &Path,
    rng: &mut u64,
    set_tag: &str,
    worker_iri: &str,
    category: &str,
    children: usize,
    omit_final_problem: bool,
    content_target: Option<&str>,
) -> Result<(usize, u64, Option<String>), CngRefusal> {
    fs::create_dir_all(dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", dir.display())))?;
    let domain = format!("wf-{category}-{set_tag}");
    let obj = format!("case-{set_tag}");
    let preds: Vec<String> = (0..=8)
        .map(|i| format!("s{i}-{}-{set_tag}", short_hex(splitmix64(rng))))
        .collect();
    let actions: Vec<String> = (0..8)
        .map(|i| format!("{}-{category}-{set_tag}-{i}", super::STEP_VERBS[i]))
        .collect();

    let mut files = 0usize;
    let mut bytes = 0u64;
    for half in 0..2usize {
        let mut body = templates
            .domain
            .replace("{{SUBJECT}}", &format!("art-{set_tag}-d{half}"))
            .replace("{{CATEGORY}}", category)
            .replace("{{WORKER}}", worker_iri)
            .replace("{{DOMAIN}}", &domain)
            .replace("{{OBJ}}", &obj);
        for i in 0..=4usize {
            body = body.replace(&format!("{{{{P{i}}}}}"), &preds[half * 4 + i]);
        }
        for i in 0..4usize {
            body = body.replace(&format!("{{{{A{i}}}}}"), &actions[half * 4 + i]);
        }
        // Recursive attachment facts: each activity of the FIRST fragment may
        // lawfully socket a child workflow; the runner derives children from
        // these triples in the admitted graph, never from directory listing.
        if half == 0 && children > 0 {
            let mut attach = String::new();
            for c in 0..children {
                attach.push_str(&format!(
                    "ex:art-{set_tag}-d0 ex:attachesWorkflow ex:child-{c} .\n"
                ));
            }
            body.push_str(&attach);
        }
        // PROJ-609: content-bearing category fragment (template-rendered,
        // never inline Turtle) on the FIRST fragment only.
        if half == 0 {
            if let Some(content) = templates.category_content.get(category) {
                let target = content_target.unwrap_or(&obj);
                body.push('\n');
                body.push_str(&fill_template(
                    content,
                    &[
                        ("SUBJECT", format!("art-{set_tag}-d0").as_str()),
                        ("TARGET", target),
                    ],
                ));
            }
        }
        let path = dir.join(format!("fragment-{half}.domain.ttl"));
        bytes += body.len() as u64;
        fs::write(&path, body)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
        files += 1;
    }
    // Both problem fragments are always RENDERED; when `omit_final_problem`
    // is set NEITHER is written (unchanged Fortune-5 bounded-admission case:
    // zero problem fragments → manufacture refuses CNG_R03) and the FINAL
    // body is withheld/returned — it alone reaches the full 8-step goal, so
    // it is the exact minimal admission that can later be granted
    // (PROJ-611).
    let mut withheld: Option<String> = None;
    for (goal_idx, tag) in [(4usize, "mid"), (8usize, "final")] {
        let body = templates
            .problem
            .replace("{{SUBJECT}}", &format!("art-{set_tag}-p{tag}"))
            .replace("{{CATEGORY}}", category)
            .replace("{{WORKER}}", worker_iri)
            .replace("{{PROBLEM}}", &format!("{domain}-{tag}"))
            .replace("{{DOMAIN}}", &domain)
            .replace("{{OBJ}}", &obj)
            .replace("{{INIT}}", &preds[0])
            .replace("{{GOAL}}", &preds[goal_idx]);
        if omit_final_problem {
            if tag == "final" {
                withheld = Some(body);
            }
            continue;
        }
        let path = dir.join(format!("goal-{tag}.problem.ttl"));
        bytes += body.len() as u64;
        fs::write(&path, body)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
        files += 1;
    }
    Ok((files, bytes, withheld))
}

/// Recursively writes the 8-ary recursion tree below `dir` to `depth` levels
/// (root at level 1). Every node is a full machine-generated artifact set.
#[allow(clippy::too_many_arguments)]
fn write_recursion_tree(
    templates: &Templates,
    dir: &Path,
    rng: &mut u64,
    tag: &str,
    worker_iri: &str,
    level: usize,
    depth: usize,
    files: &mut usize,
    bytes: &mut u64,
    nodes: &mut usize,
) -> Result<(), CngRefusal> {
    let children = if level < depth { 8 } else { 0 };
    let category = CATEGORIES[(splitmix64(rng) % CATEGORIES.len() as u64) as usize];
    // Content-bearing categories default to the node's own case IRI
    // (content_target = None) — recursion nodes have no tick neighbours.
    let (f, b, _withheld) = write_set(
        templates, dir, rng, tag, worker_iri, category, children, false, None,
    )?;
    *files += f;
    *bytes += b;
    *nodes += 1;
    for c in 0..children {
        let child_dir = dir.join(format!("child-{c}"));
        write_recursion_tree(
            templates,
            &child_dir,
            rng,
            &format!("{tag}-{c}"),
            worker_iri,
            level + 1,
            depth,
            files,
            bytes,
            nodes,
        )?;
    }
    Ok(())
}

/// Generates the full benchmark corpus: partitioned worker roster as
/// `roster_admitted` observation facts (rendered from the roster
/// observation template — no inline Turtle), per-worker workload artifact
/// sets, and the 8-ary recursion tree.
///
/// # Complexity
/// O(workers + sets + 8^depth) file writes, all seeded/deterministic.
pub fn generate(out_dir: &Path, cfg: &BenchConfig) -> Result<GenerateReport, CngRefusal> {
    let templates = load_templates()?;
    fs::create_dir_all(out_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", out_dir.display())))?;
    let mut files = 0usize;
    let mut bytes = 0u64;

    // 1. Roster partitions: every represented worker is a materialized
    //    roster_admitted observation fact set (identity, role, department,
    //    standing) rendered from the roster observation template, written to
    //    disk, and only ever consumed back through oxigraph.
    let roster_dir = out_dir.join("roster");
    fs::create_dir_all(&roster_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir roster: {e}")))?;
    let roles = ["reviewer", "approver", "operator", "auditor", "coordinator"];
    let departments = [
        "finance",
        "hr",
        "logistics",
        "sales",
        "engineering",
        "legal",
    ];
    let roster_template = templates.obs.get("roster").ok_or_else(|| {
        CngRefusal::IoRefused("roster observation template missing from loaded set".to_string())
    })?;
    let mut rng = cfg.seed;
    let partitions = cfg.workers.div_ceil(WORKERS_PER_ROSTER_PARTITION);
    for p in 0..partitions {
        let start = p * WORKERS_PER_ROSTER_PARTITION;
        let end = usize::min(start + WORKERS_PER_ROSTER_PARTITION, cfg.workers);
        let mut body = String::with_capacity((end - start) * 360 + 128);
        for w in start..end {
            let role = roles[(splitmix64(&mut rng) % roles.len() as u64) as usize];
            let dept = departments[(splitmix64(&mut rng) % departments.len() as u64) as usize];
            let seq = w.to_string();
            let worker_id = format!("w{w}");
            body.push_str(&fill_template(
                roster_template,
                &[
                    ("SUBJECT", format!("obs-roster-{worker_id}").as_str()),
                    ("SEQ", seq.as_str()),
                    ("SET_ID", "roster"),
                    ("WORKER_ID", worker_id.as_str()),
                    ("ROLE", role),
                    ("DEPARTMENT", dept),
                    ("STANDING", "admitted"),
                ],
            ));
            body.push('\n');
        }
        let path = roster_dir.join(format!("partition-{p:05}.ttl"));
        bytes += body.len() as u64;
        fs::write(&path, body)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
        files += 1;
    }

    // 2. Workload artifact sets, worker-attributed, category-mixed.
    let sets_dir = out_dir.join("sets");
    for s in 0..cfg.artifact_sets {
        let worker = (splitmix64(&mut rng) % cfg.workers as u64) as usize;
        let category = CATEGORIES[s % CATEGORIES.len()];
        let omit = (splitmix64(&mut rng) % 1000) < cfg.refusal_per_mille as u64;
        // PROJ-609 content targets in the Fortune-5 corpus: an interruption
        // interrupts the PREVIOUS set's in-flight case; a planning artifact
        // plans for the NEXT set's case. Both are real in-corpus IRIs.
        let content_target = match category {
            "interruption" if s > 0 => Some(format!("case-s{:06}", s - 1)),
            "planning" => Some(format!("case-s{:06}", s + 1)),
            _ => None,
        };
        let (f, b, _withheld) = write_set(
            &templates,
            &sets_dir.join(format!("set-{s:06}")),
            &mut rng,
            &format!("s{s:06}"),
            &format!("{RWAI_PREFIX}w{worker}"),
            category,
            0,
            omit,
            content_target.as_deref(),
        )?;
        files += f;
        bytes += b;
    }

    // 3. Recursion tree (8-ary, `recursion_depth` levels).
    let mut nodes = 0usize;
    if cfg.recursion_depth > 0 {
        let worker = (splitmix64(&mut rng) % cfg.workers as u64) as usize;
        write_recursion_tree(
            &templates,
            &out_dir.join("recursion").join("root"),
            &mut rng,
            "r",
            &format!("{RWAI_PREFIX}w{worker}"),
            1,
            cfg.recursion_depth,
            &mut files,
            &mut bytes,
            &mut nodes,
        )?;
    }

    let config_json = serde_json::to_string_pretty(cfg)
        .map_err(|e| CngRefusal::IoRefused(format!("config serialize: {e}")))?;
    fs::write(out_dir.join("benchmark-config.json"), &config_json)
        .map_err(|e| CngRefusal::IoRefused(format!("write config: {e}")))?;

    Ok(GenerateReport {
        out_dir: out_dir.display().to_string(),
        workers_represented: cfg.workers,
        roster_partitions: partitions,
        artifact_sets: cfg.artifact_sets,
        recursion_nodes: nodes,
        recursion_depth: cfg.recursion_depth,
        files_written: files,
        bytes_written: bytes,
    })
}
