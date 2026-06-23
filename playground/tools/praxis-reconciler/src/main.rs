use clap::Parser;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

const MONITORED_FILES: &[&str] = &[
    "deny.toml",
    "typos.toml",
    "rustfmt.toml",
    "rust-toolchain.toml",
    "SECURITY.md",
    ".github/workflows/ci.yml",
    ".github/workflows/release.yml",
    ".github/dependabot.yml",
    ".editorconfig",
];

#[derive(Parser, Debug)]
#[command(name = "praxis-reconciler")]
struct Args {
    #[arg(long, default_value = ".")]
    project: PathBuf,

    #[arg(long, default_value = "/Users/sac/praxis/template")]
    template: PathBuf,
}

fn reconcile_file(template_dir: &Path, project_dir: &Path, rel_path: &str) -> anyhow::Result<bool> {
    let src = template_dir.join(rel_path);
    let dst = project_dir.join(rel_path);

    if !src.exists() {
        return Ok(false);
    }

    let needs_update = if !dst.exists() {
        true
    } else {
        let src_content = std::fs::read(&src)?;
        let dst_content = std::fs::read(&dst)?;
        src_content != dst_content
    };

    if needs_update {
        println!("Reconciling/Restoring file: {:?}", rel_path);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&src, &dst)?;
        return Ok(true);
    }

    Ok(false)
}

fn reconcile_all(template_dir: &Path, project_dir: &Path, files: &[&str]) {
    for &f in files {
        if let Err(e) = reconcile_file(template_dir, project_dir, f) {
            eprintln!("Error reconciling file {}: {:?}", f, e);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // We want the absolute paths so matching works reliably.
    let project = std::fs::canonicalize(&args.project).unwrap_or_else(|_| args.project.clone());
    let template = std::fs::canonicalize(&args.template).unwrap_or_else(|_| args.template.clone());

    println!("Starting praxis-reconciler...");
    println!("Project directory: {:?}", project);
    println!("Template directory: {:?}", template);

    // Initial reconciliation on startup
    reconcile_all(&template, &project, MONITORED_FILES);

    // Start background polling thread (every 1 second)
    let poller_project = project.clone();
    let poller_template = template.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        reconcile_all(&poller_template, &poller_project, MONITORED_FILES);
    });

    // Start file watcher
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    // Watch target project recursively.
    if project.exists() {
        watcher.watch(&project, RecursiveMode::Recursive)?;
        println!("Watching {:?} for changes...", project);
    } else {
        eprintln!(
            "Project directory {:?} does not exist yet. Watcher not started.",
            project
        );
    }

    for res in rx {
        match res {
            Ok(event) => {
                let mut triggered = false;
                for path in event.paths {
                    if let Ok(rel) = path.strip_prefix(&project) {
                        let path_str = rel.to_string_lossy().replace('\\', "/");
                        if MONITORED_FILES.contains(&path_str.as_str()) {
                            triggered = true;
                            break;
                        }
                    }
                }
                if triggered {
                    reconcile_all(&template, &project, MONITORED_FILES);
                }
            }
            Err(e) => {
                eprintln!("Watcher error: {:?}", e);
            }
        }
    }

    Ok(())
}
