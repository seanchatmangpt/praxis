//! Pack resolution and content hashing.
//!
//! A pack is a directory containing `pack.toml`, `ontology.ttl`, and a
//! `templates/` directory of `*.tmpl` files. Packs are declared in
//! `ggen.toml` under `[packs]` and resolved fail-closed: a missing pack
//! directory, missing manifest, missing ontology, unknown manifest keys,
//! or an empty template set all refuse by name with an `FM-PACK-*` code.
//!
//! [`content_hash`] computes a deterministic BLAKE3 over the pack's
//! ontology and templates as sorted `(relative_path, bytes)` pairs.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{
    config::{GgenConfig, PackRef},
    error::{AppError, Result},
};

/// A resolved local pack, ready for the sync pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    /// Pack name (the `[packs]` key in `ggen.toml`).
    pub name: String,
    /// Version string from `pack.toml`.
    pub version: String,
    /// Human description from `pack.toml`.
    pub description: String,
    /// Absolute (resolved) pack root directory.
    pub root: PathBuf,
    /// Path to the pack's `ontology.ttl`.
    pub ontology_path: PathBuf,
    /// Sorted paths of the pack's `templates/*.tmpl` files.
    pub template_paths: Vec<PathBuf>,
}

/// On-disk `pack.toml` schema (closed key set, fail closed).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackToml {
    pack: PackMeta,
}

/// `[pack]` table of `pack.toml` (closed key set).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackMeta {
    name: String,
    version: String,
    description: String,
}

/// Resolve every pack declared in `config.packs`, in name (`BTreeMap`) order.
///
/// `PackRef::Path` entries resolve relative to `config_root` (the directory
/// containing `ggen.toml`).
///
/// # Errors
/// - `[FM-PACK-001]` pack directory missing
/// - `[FM-PACK-002]` `pack.toml` missing or unreadable
/// - `[FM-PACK-003]` `pack.toml` invalid TOML or unknown keys
/// - `[FM-PACK-004]` `ontology.ttl` missing
/// - `[FM-PACK-005]` zero templates under `templates/`
/// - `[FM-PACK-007]` `PackRef::Git` (git packs are not implemented)
pub fn resolve(config: &GgenConfig, config_root: &Path) -> Result<Vec<Pack>> {
    let mut packs = Vec::with_capacity(config.packs.len());
    for (name, pack_ref) in &config.packs {
        match pack_ref {
            PackRef::Git { git, version } => {
                return Err(AppError::fm_pack(
                    7,
                    format!(
                        "pack `{name}`: git packs are not implemented \
                         (git = `{git}`, version = `{version}`). \
                         Remediation: vendor the pack locally and use {{ path = \"…\" }}."
                    ),
                ));
            }
            PackRef::Path { path } => {
                packs.push(resolve_path_pack(name, path, config_root)?);
            }
        }
    }
    Ok(packs)
}

/// Resolve one `{ path = … }` pack rooted at `config_root`.
fn resolve_path_pack(name: &str, path: &Path, config_root: &Path) -> Result<Pack> {
    let root = config_root.join(path);
    if !root.is_dir() {
        return Err(AppError::fm_pack(
            1,
            format!(
                "pack `{name}`: directory `{}` does not exist. \
                 Remediation: fix the [packs] path or vendor the pack.",
                root.display()
            ),
        ));
    }

    let manifest_path = root.join("pack.toml");
    let manifest_raw = std::fs::read_to_string(&manifest_path).map_err(|e| {
        AppError::fm_pack(
            2,
            format!(
                "pack `{name}`: pack.toml unreadable at `{}`: {e}. \
                 Remediation: every pack must ship a pack.toml.",
                manifest_path.display()
            ),
        )
    })?;
    let manifest: PackToml = star_toml::from_str(&manifest_raw).map_err(|e| {
        AppError::fm_pack(
            3,
            format!(
                "pack `{name}`: invalid pack.toml at `{}`: {e}. \
                 Remediation: fix the TOML syntax or remove unknown keys.",
                manifest_path.display()
            ),
        )
    })?;

    let ontology_path = root.join("ontology.ttl");
    if !ontology_path.is_file() {
        return Err(AppError::fm_pack(
            4,
            format!(
                "pack `{name}`: ontology.ttl missing at `{}`. \
                 Remediation: every pack must ship an ontology.ttl.",
                ontology_path.display()
            ),
        ));
    }

    let templates_dir = root.join("templates");
    let mut template_paths: Vec<PathBuf> = Vec::new();
    if templates_dir.is_dir() {
        for entry in std::fs::read_dir(&templates_dir)? {
            let path = entry?.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "tmpl") {
                template_paths.push(path);
            }
        }
    }
    template_paths.sort();
    if template_paths.is_empty() {
        return Err(AppError::fm_pack(
            5,
            format!(
                "pack `{name}`: zero templates under `{}`. \
                 Remediation: a pack must ship at least one templates/*.tmpl.",
                templates_dir.display()
            ),
        ));
    }

    // The [packs] key in ggen.toml is the authoritative resolution name;
    // the manifest's own `name` is informational.
    let _ = &manifest.pack.name;
    Ok(Pack {
        name: name.to_string(),
        version: manifest.pack.version,
        description: manifest.pack.description,
        root,
        ontology_path,
        template_paths,
    })
}

/// Deterministic BLAKE3 content hash of a pack: sorted
/// `(relative_path, bytes)` pairs over `ontology.ttl` plus every template.
/// For each pair the path string bytes are hashed, then the file bytes,
/// in sorted relative-path order.
///
/// # Errors
/// `[FM-PACK-006]` when a pack file becomes unreadable between resolution
/// and hashing.
pub fn content_hash(pack: &Pack) -> Result<[u8; 32]> {
    let mut entries: Vec<(String, PathBuf)> = Vec::with_capacity(1 + pack.template_paths.len());
    entries.push((rel_string(&pack.ontology_path, &pack.root), pack.ontology_path.clone()));
    for tpl in &pack.template_paths {
        entries.push((rel_string(tpl, &pack.root), tpl.clone()));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = blake3::Hasher::new();
    for (rel, path) in &entries {
        let bytes = std::fs::read(path).map_err(|e| {
            AppError::fm_pack(
                6,
                format!(
                    "pack `{}`: file `{}` unreadable while hashing: {e}. \
                     Remediation: do not mutate a pack during sync.",
                    pack.name,
                    path.display()
                ),
            )
        })?;
        hasher.update(rel.as_bytes());
        hasher.update(&bytes);
    }
    Ok(*hasher.finalize().as_bytes())
}

// ---------------------------------------------------------------------------
// ggen.lock — deterministic pack lockfile
// ---------------------------------------------------------------------------

/// Lockfile name, at the project root next to `ggen.toml`.
pub const LOCK_FILE_NAME: &str = "ggen.lock";

/// One resolved lock entry: name, `source` string, and BLAKE3 hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockEntry {
    /// Pack name (the `[packs]` key).
    pub name: String,
    /// Source as written in `ggen.toml`, prefixed (`path:…`).
    pub source: String,
    /// `blake3:<hex>` content hash.
    pub content_hash: String,
}

/// On-disk `ggen.lock` shape (closed key set, fail closed).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LockDoc {
    #[serde(default)]
    packs: std::collections::BTreeMap<String, LockDocEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LockDocEntry {
    source: String,
    content_hash: String,
}

/// The `source = "…"` string for a pack ref, as written in `ggen.toml`.
#[must_use]
pub fn source_string(pack_ref: &PackRef) -> String {
    match pack_ref {
        PackRef::Path { path } => format!("path:{}", path.display()),
        PackRef::Git { git, version } => format!("git:{git}@{version}"),
    }
}

/// Build lock entries (name → source + `blake3:<hex>`) for resolved packs.
///
/// # Errors
/// Propagates [`content_hash`] failures.
pub fn lock_entries(config: &GgenConfig, packs: &[Pack]) -> Result<Vec<LockEntry>> {
    let mut entries = Vec::with_capacity(packs.len());
    for pack in packs {
        let source = config.packs.get(&pack.name).map(source_string).unwrap_or_default();
        let hash = content_hash(pack)?;
        entries.push(LockEntry {
            name: pack.name.clone(),
            source,
            content_hash: format!("blake3:{}", crate::sync::hex32(&hash)),
        });
    }
    Ok(entries)
}

/// Fail closed if `ggen.lock` exists and any pack's content hash differs.
///
/// A missing lockfile is fine (first sync). Packs absent from the lock are
/// fine (they get locked on the next successful sync).
///
/// # Errors
/// - `[FM-PACK-009]` `ggen.lock` unreadable or malformed
/// - `[FM-PACK-008]` pack content hash differs from the locked hash
pub fn check_lock(root: &Path, entries: &[LockEntry]) -> Result<()> {
    let lock_path = root.join(LOCK_FILE_NAME);
    if !lock_path.is_file() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(&lock_path).map_err(|e| {
        AppError::fm_pack(9, format!("ggen.lock unreadable at `{}`: {e}", lock_path.display()))
    })?;
    let doc: LockDoc = star_toml::from_str(&raw).map_err(|e| {
        AppError::fm_pack(
            9,
            format!(
                "ggen.lock malformed at `{}`: {e}. \
                 Remediation: fix or delete the lockfile.",
                lock_path.display()
            ),
        )
    })?;
    for entry in entries {
        if let Some(locked) = doc.packs.get(&entry.name) {
            if locked.content_hash != entry.content_hash {
                return Err(AppError::fm_pack(
                    8,
                    format!(
                        "pack `{}` (source `{}`) content hash mismatch: ggen.lock \
                         has `{}` but the pack on disk hashes to `{}`. \
                         Remediation: restore the pack, or delete ggen.lock \
                         to intentionally re-lock.",
                        entry.name, locked.source, locked.content_hash, entry.content_hash
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Write `ggen.lock` deterministically (sorted by pack name, no timestamps).
///
/// Idempotent: if the file already exists with identical content, no write
/// occurs (the lockfile's mtime is left untouched). This matters for
/// [`crate::watch`] mode, which re-runs the pipeline on every debounced
/// filesystem change under the project root — `ggen.lock` lives at the
/// root (not under an ignored directory like `.ggen-v2`), so an
/// unconditional rewrite on every sync would retrigger the watcher on its
/// own output forever.
///
/// # Errors
/// I/O failure reading the existing lockfile (other than "not found") or
/// writing the new one.
pub fn write_lock(root: &Path, entries: &[LockEntry]) -> Result<()> {
    use std::fmt::Write as _;
    let mut sorted: Vec<&LockEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from("# ggen.lock — generated by `ggen sync`. Do not edit.\n");
    for entry in sorted {
        let _ = write!(
            out,
            "\n[packs.{}]\nsource = \"{}\"\ncontent_hash = \"{}\"\n",
            entry.name, entry.source, entry.content_hash
        );
    }
    let lock_path = root.join(LOCK_FILE_NAME);
    match std::fs::read_to_string(&lock_path) {
        Ok(existing) if existing == out => return Ok(()),
        Ok(_) | Err(_) => {}
    }
    std::fs::write(lock_path, out)?;
    Ok(())
}

/// Relative path of `path` under `root`, with `/` separators.
fn rel_string(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn entry(name: &str) -> LockEntry {
        LockEntry {
            name: name.to_string(),
            source: "path:../pack".to_string(),
            content_hash: "blake3:deadbeef".to_string(),
        }
    }

    /// Regression test for the watch-mode infinite-loop bug: writing an
    /// identical lockfile must not touch the file on disk (no new mtime),
    /// or `--watch` would retrigger on its own output forever (`ggen.lock`
    /// lives at the project root, not under an ignored directory).
    #[test]
    fn write_lock_is_idempotent_when_content_is_unchanged() {
        let dir = TempDir::new().expect("tempdir");
        let entries = vec![entry("widget")];

        write_lock(dir.path(), &entries).expect("first write");
        let mtime_1 = std::fs::metadata(dir.path().join(LOCK_FILE_NAME))
            .expect("metadata")
            .modified()
            .expect("mtime");

        std::thread::sleep(std::time::Duration::from_millis(20));
        write_lock(dir.path(), &entries).expect("second write");
        let mtime_2 = std::fs::metadata(dir.path().join(LOCK_FILE_NAME))
            .expect("metadata")
            .modified()
            .expect("mtime");

        assert_eq!(mtime_1, mtime_2, "unchanged lockfile content must not be rewritten");
    }

    #[test]
    fn write_lock_rewrites_when_content_changes() {
        let dir = TempDir::new().expect("tempdir");
        write_lock(dir.path(), &[entry("widget")]).expect("first write");
        write_lock(dir.path(), &[entry("widget"), entry("gadget")]).expect("second write");

        let contents =
            std::fs::read_to_string(dir.path().join(LOCK_FILE_NAME)).expect("read lockfile");
        assert!(contents.contains("gadget"));
    }
}
