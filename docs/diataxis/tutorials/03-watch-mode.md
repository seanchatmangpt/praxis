# Watching for Changes

In this tutorial you will run `ggen` in watch mode, edit a template while it is
running in the background, and see it re-run the sync pipeline automatically.
Then you will stop it and look at the one safety guard that keeps it from
re-triggering on its own output.

You need a built `ggen` binary. If you don't have one yet, build it first:

```bash
cd /Users/sac/praxis
cargo build -p ggen
```

The binary will be at `target/debug/ggen`. The steps below assume you can run
it as `ggen` — either put `target/debug` on your `PATH`, or substitute the
full path (`/Users/sac/praxis/target/debug/ggen`) everywhere you see `ggen`.

## Step 1: Create a scratch project

Watch mode writes files continuously, so work in a throwaway directory, not
inside the `praxis` repo.

```bash
mkdir -p /tmp/ggen-watch-demo/templates
cd /tmp/ggen-watch-demo
```

Create a minimal `ggen.toml`:

```bash
cat > ggen.toml <<'EOF'
[project]
name = "watch-demo"

[ontology]
source = "ontology.ttl"

[templates]
dir = "templates"
EOF
```

Create a tiny ontology:

```bash
cat > ontology.ttl <<'EOF'
@prefix ex: <http://example.org/> .
ex:thing ex:name "world" .
EOF
```

Create a template. Note the `force: true` in the frontmatter — you need this
because you're about to overwrite the generated output on every edit, and
`ggen` refuses a silent clobber by default:

```bash
cat > templates/greeting.tmpl <<'EOF'
---
to: out/greeting.txt
force: true
---
hello
EOF
```

## Step 2: Start watch mode in the background

The `sync run` command takes a `--watch` flag, defined here:

```
crates/ggen/src/verbs/sync.rs:10-14
```

```rust
/// Run the five-stage generation pipeline: resolve, enrich, extract, render, write. --watch re-runs the pipeline on filesystem changes.
#[clap_noun_verb_macros::verb("run", "sync")]
fn sync_run(dry_run: bool, watch: bool) -> Result<serde_json::Value> {
    crate::verbs::handlers::handle_sync_run(dry_run, watch)
}
```

`--watch` is a boolean flag — pass it bare, with no value. Start it in the
background and redirect its output to a log file so you can inspect it while
it keeps running:

```bash
nohup ggen sync run --watch > watch_output.log 2>&1 &
echo $! > pid.txt
```

Wait a moment, then look at the log:

```bash
sleep 2
cat watch_output.log
```

Real output from this exact run:

```
ggen sync: 1 written, 0 skipped
watching /private/tmp/ggen-watch-demo for changes... (Ctrl-C to stop)
```

The first line is the one synchronous sync that runs before the watcher
starts:

```
crates/ggen/src/verbs/handlers.rs:36-41
```

```rust
pub fn handle_sync_run(dry_run: bool, watch: bool) -> Result<serde_json::Value> {
    let root = project_root()?;
    if watch {
        crate::watch::watch(&root, dry_run).map_err(exec_err)?;
        return Ok(serde_json::json!({ "watch": "stopped" }));
    }
```

Confirm the output file exists:

```bash
cat out/greeting.txt
```

```
hello
```

## Step 3: Edit the template and watch it re-sync

With the watcher still running in the background, edit the template's body:

```bash
cat > templates/greeting.tmpl <<'EOF'
---
to: out/greeting.txt
force: true
---
hello again from the edited template
EOF
```

Give it a second to notice the change (events are batched over a 500ms
debounce window — `crates/ggen/src/watch.rs:30`), then look at the output
file and the log again:

```bash
sleep 1
cat out/greeting.txt
```

Real output from this exact run:

```
hello again from the edited template
```

The file changed without you running `ggen` again — the background process
picked up the edit and re-ran the pipeline on its own. Checking the log's
line count before and after confirms new sync activity was appended (15
lines before the edit, 18 after, in this run):

```bash
wc -l watch_output.log
```

You'll notice the log keeps growing even beyond your one edit. That's
expected here: because your template's `to:` path (`out/greeting.txt`) is not
one of the two directories the watcher ignores, every write it makes to that
file is itself a filesystem event, so it keeps re-triggering itself. You'll
deal with that in Step 5 — for now, that's what you're seeing.

## Step 4: Stop watch mode

There's no Ctrl-C handling inside `ggen` for this — the process is designed
to be killed:

```
crates/ggen/src/watch.rs:11-12
```

```
//! No SIGINT/Ctrl-C handling: the process is expected to be killed to stop
//! watching, matching the reference implementation.
```

If you're running it in your own foreground terminal, press **Ctrl-C** now.
Since you started it in the background in this tutorial, kill it by PID
instead:

```bash
kill $(cat pid.txt)
sleep 1
ps -p $(cat pid.txt) 2>/dev/null || echo "confirmed: watch process stopped"
```

Real output from this exact run:

```
confirmed: watch process stopped
```

## Step 5: The self-trigger guard

`ggen sync` itself writes files under `.ggen-v2/` (a receipt and a receipt
log) on every run. Without a guard, those writes would be seen by the
watcher as "something changed," triggering another sync, which writes the
receipt again, forever. `should_ignore` is that guard:

```
crates/ggen/src/watch.rs:108-116
```

```rust
fn should_ignore(root: &Path, paths: &[PathBuf]) -> bool {
    if paths.is_empty() {
        return true;
    }
    let ignored: Vec<PathBuf> = IGNORED_DIRS.iter().map(|d| root.join(d)).collect();
    paths
        .iter()
        .all(|p| ignored.iter().any(|dir| p.starts_with(dir)))
}
```

A batch of changed paths is ignored only if *every* path in it falls under
`root/.ggen-v2` or `root/.git` — the two entries in `IGNORED_DIRS`:

```
crates/ggen/src/watch.rs:26
```

```rust
const IGNORED_DIRS: [&str; 2] = [".ggen-v2", ".git"];
```

This is exactly why your own generated `out/greeting.txt` kept re-triggering
in Step 3: it lives outside both ignored directories, so it doesn't qualify
for the guard. Only `.ggen-v2` (receipts) and `.git` (VCS bookkeeping) are
protected from self-triggering.

## What you built

You ran `ggen sync run --watch` against a scratch project, edited a template
while the watcher was running in the background, and confirmed with real log
output and file contents that it re-ran the sync pipeline on its own,
without you invoking `ggen` again. You also saw why generated output paths
(unlike `.ggen-v2/` and `.git/`) can retrigger the watcher, and how to stop
the process since it has no built-in Ctrl-C handling.

For guidance on structuring a project so watch mode doesn't spin on its own
output — for example, keeping every generated file's `to:` path package-local
and away from paths you also want to edit by hand — see the How-To guide on
organizing a `ggen.toml` project layout.
