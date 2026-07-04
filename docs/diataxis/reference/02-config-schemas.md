# ggen.toml and pack.toml Schemas

Reference for the two closed-vocabulary TOML schemas parsed by `crates/ggen`: the project manifest (`ggen.toml`, deserialized as `GgenConfig`) and the pack manifest (`pack.toml`, deserialized as `PackToml`/`PackMeta`). All structs on both schemas are annotated `#[serde(deny_unknown_fields)]`, so any key not listed below causes a hard parse error rather than being silently ignored.

## `ggen.toml`

Parsed by `GgenConfig::load` / `GgenConfig::from_toml_str` (`crates/ggen/src/config.rs:82`, `crates/ggen/src/config.rs:110`), which delegate to `star_toml::load_file` / `star_toml::from_str` (`crates/ggen/src/config.rs:83`, `crates/ggen/src/config.rs:111`).

### `GgenConfig` (root document)

Struct defined at `crates/ggen/src/config.rs:19`. `deny_unknown_fields` attribute at `crates/ggen/src/config.rs:18`.

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `project` | `[project]` | `Project` (table) | yes (no `#[serde(default)]`) | `crates/ggen/src/config.rs:21` |
| `ontology` | `[ontology]` | `Ontology` (table) | yes (no `#[serde(default)]`) | `crates/ggen/src/config.rs:23` |
| `packs` | `[packs]` | `BTreeMap<String, PackRef>` | no — `#[serde(default)]` at `crates/ggen/src/config.rs:25`, defaults to empty map | `crates/ggen/src/config.rs:26` |
| `templates` | `[templates]` | `Templates` (table) | yes (no `#[serde(default)]`) | `crates/ggen/src/config.rs:28` |

Unknown top-level keys are rejected (`deny_unknown_fields` at `crates/ggen/src/config.rs:18`).

### `[project]` — `Project` struct

Struct defined at `crates/ggen/src/config.rs:34`. `deny_unknown_fields` attribute at `crates/ggen/src/config.rs:33`.

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `name` | `name` | `String` | yes (no `#[serde(default)]`) | `crates/ggen/src/config.rs:36` |

Unknown keys under `[project]` are rejected (`deny_unknown_fields` at `crates/ggen/src/config.rs:33`).

### `[ontology]` — `Ontology` struct

Struct defined at `crates/ggen/src/config.rs:42`. `deny_unknown_fields` attribute at `crates/ggen/src/config.rs:41`.

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `source` | `source` | `PathBuf` | yes (no `#[serde(default)]`) | `crates/ggen/src/config.rs:44` |
| `prefixes` | `prefixes` | `BTreeMap<String, String>` | no — `#[serde(default)]` at `crates/ggen/src/config.rs:46`, defaults to empty map | `crates/ggen/src/config.rs:47` |

Doc comment states `source` is "Path to the ontology file (Turtle), relative to the manifest" (`crates/ggen/src/config.rs:43`); `prefixes` is documented as "Prefix → namespace IRI map" (`crates/ggen/src/config.rs:45`). Unknown keys under `[ontology]` are rejected (`deny_unknown_fields` at `crates/ggen/src/config.rs:41`).

### `[packs.<name>]` — `PackRef` enum

Enum defined at `crates/ggen/src/config.rs:53`, tagged `#[serde(untagged)]` (`crates/ggen/src/config.rs:52`) — the variant is selected by which field set is present in the table, not by an explicit tag key. Each `[packs]` entry is keyed by the pack name (the `BTreeMap<String, PackRef>` key at `crates/ggen/src/config.rs:26`).

| Variant | Fields | Field types | Required? | Line |
|---|---|---|---|---|
| `Path` | `path` | `PathBuf` | yes (only field of this variant) | `crates/ggen/src/config.rs:57` |
| `Git` | `git`, `version` | `String`, `String` | yes (both fields of this variant) | `crates/ggen/src/config.rs:62`, `crates/ggen/src/config.rs:64` |

`PackRef` itself has no `deny_unknown_fields` attribute directly, but as an `untagged` enum a table matching neither variant's exact field set fails to deserialize into any variant. `PackRef::Git` is accepted by the config parser but rejected downstream: `resolve` in `crates/ggen/src/pack.rs:64` returns error code `FM-PACK-007` for any `PackRef::Git` entry, with message "git packs are not implemented" (`crates/ggen/src/pack.rs:69`-`crates/ggen/src/pack.rs:76`).

### `[templates]` — `Templates` struct

Struct defined at `crates/ggen/src/config.rs:71`. `deny_unknown_fields` attribute at `crates/ggen/src/config.rs:70`.

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `dir` | `dir` | `PathBuf` | yes (no `#[serde(default)]`) | `crates/ggen/src/config.rs:73` |

Doc comment: "Template directory, relative to the manifest" (`crates/ggen/src/config.rs:72`). Unknown keys under `[templates]` are rejected (`deny_unknown_fields` at `crates/ggen/src/config.rs:70`).

### `ggen.toml` load/parse error codes

| Method | Line | Condition | Error code | Line of `AppError::fm_config` call |
|---|---|---|---|---|
| `GgenConfig::load` | `crates/ggen/src/config.rs:82` | file missing (`star_toml::Error::FileNotFound`) | `FM-CONFIG-001` | `crates/ggen/src/config.rs:84` |
| `GgenConfig::load` | `crates/ggen/src/config.rs:82` | I/O error (`star_toml::Error::Io`) | `FM-CONFIG-001` | `crates/ggen/src/config.rs:91` |
| `GgenConfig::load` | `crates/ggen/src/config.rs:82` | any other `star_toml::Error` (syntax error or unknown key) | `FM-CONFIG-002` | `crates/ggen/src/config.rs:95` |
| `GgenConfig::from_toml_str` | `crates/ggen/src/config.rs:110` | any `star_toml::Error` (syntax error or unknown key) | `FM-CONFIG-002` | `crates/ggen/src/config.rs:112` |

Module-level doc comment states `star_toml` env-expands values before parsing (`crates/ggen/src/config.rs:5`) and applies `deny_unknown_fields` "on every table… so any unknown key is a hard error (fail closed)" (`crates/ggen/src/config.rs:6`-`crates/ggen/src/config.rs:7`).

---

## `pack.toml`

Parsed inline inside `resolve_path_pack` (`crates/ggen/src/pack.rs:87`) via `star_toml::from_str` at `crates/ggen/src/pack.rs:111`, against the private `PackToml` struct. Both `PackToml` and `PackMeta` are private (`struct`, not `pub struct`) — not part of `crates/ggen`'s public API, only its on-disk pack contract. Module-level doc comment: a pack is "a directory containing `pack.toml`, `ontology.ttl`, and a `templates/` directory of `*.tmpl` files" (`crates/ggen/src/pack.rs:3`-`crates/ggen/src/pack.rs:4`).

### `PackToml` (root document)

Struct defined at `crates/ggen/src/pack.rs:39`. `deny_unknown_fields` attribute at `crates/ggen/src/pack.rs:38`.

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `pack` | `[pack]` | `PackMeta` (table) | yes (no `#[serde(default)]`) | `crates/ggen/src/pack.rs:40` |

Unknown top-level keys are rejected (`deny_unknown_fields` at `crates/ggen/src/pack.rs:38`).

### `[pack]` — `PackMeta` struct

Struct defined at `crates/ggen/src/pack.rs:46`. `deny_unknown_fields` attribute at `crates/ggen/src/pack.rs:45`. Doc comment: "`[pack]` table of `pack.toml` (closed key set)" (`crates/ggen/src/pack.rs:43`).

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `name` | `name` | `String` | yes (no `#[serde(default)]`) | `crates/ggen/src/pack.rs:47` |
| `version` | `version` | `String` | yes (no `#[serde(default)]`) | `crates/ggen/src/pack.rs:48` |
| `description` | `description` | `String` | yes (no `#[serde(default)]`) | `crates/ggen/src/pack.rs:49` |

Unknown keys under `[pack]` are rejected (`deny_unknown_fields` at `crates/ggen/src/pack.rs:45`).

Note: `manifest.pack.name` (the `pack.toml` `name` field) is read but discarded — `crates/ggen/src/pack.rs:158` explicitly binds it with `let _ = &manifest.pack.name;` and a comment stating "The `[packs]` key in `ggen.toml` is the authoritative resolution name; the manifest's own `name` is informational" (`crates/ggen/src/pack.rs:156`-`crates/ggen/src/pack.rs:157`). `version` and `description` are propagated into the resolved `Pack` struct (`crates/ggen/src/pack.rs:161`-`crates/ggen/src/pack.rs:162`).

### On-disk layout implied by `pack.toml` (non-schema, filesystem-adjacent facts)

| Path relative to pack root | Purpose | Required? | Enforcement line |
|---|---|---|---|
| `pack.toml` | manifest, parsed as `PackToml` | yes | read at `crates/ggen/src/pack.rs:100`-`crates/ggen/src/pack.rs:101`; missing/unreadable → `FM-PACK-002` at `crates/ggen/src/pack.rs:102` |
| `ontology.ttl` | pack's RDF ontology | yes | checked at `crates/ggen/src/pack.rs:122`-`crates/ggen/src/pack.rs:123`; missing → `FM-PACK-004` at `crates/ggen/src/pack.rs:124` |
| `templates/*.tmpl` | Tera template files | yes, at least one | collected at `crates/ggen/src/pack.rs:134`-`crates/ggen/src/pack.rs:144`; zero found → `FM-PACK-005` at `crates/ggen/src/pack.rs:146` |

### `pack.toml` parse/resolution error codes

| Condition | Error code | Line of `AppError::fm_pack` call |
|---|---|---|
| pack directory does not exist | `FM-PACK-001` | `crates/ggen/src/pack.rs:90` |
| `pack.toml` unreadable | `FM-PACK-002` | `crates/ggen/src/pack.rs:102` |
| `pack.toml` invalid TOML or unknown keys | `FM-PACK-003` | `crates/ggen/src/pack.rs:112` |
| `ontology.ttl` missing | `FM-PACK-004` | `crates/ggen/src/pack.rs:124` |
| zero `templates/*.tmpl` files | `FM-PACK-005` | `crates/ggen/src/pack.rs:146` |
| `PackRef::Git` variant used in `[packs]` | `FM-PACK-007` | `crates/ggen/src/pack.rs:69` |
| pack file unreadable during `content_hash` | `FM-PACK-006` | `crates/ggen/src/pack.rs:189` |

Function-level doc comment on `resolve` (`crates/ggen/src/pack.rs:52`-`crates/ggen/src/pack.rs:63`) enumerates the same code set as its `# Errors` section.

---

## `ggen.lock` (generated, not hand-authored)

Not a user-authored schema, but shares the `deny_unknown_fields` pattern and is produced from `ggen.toml`/`pack.toml` data — included here because it round-trips through the same TOML machinery.

### `LockDoc` (root document)

Struct defined at `crates/ggen/src/pack.rs:225`. `deny_unknown_fields` attribute at `crates/ggen/src/pack.rs:224`.

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `packs` | `[packs.<name>]` | `BTreeMap<String, LockDocEntry>` | no — `#[serde(default)]` at `crates/ggen/src/pack.rs:226`, defaults to empty map | `crates/ggen/src/pack.rs:227` |

### `[packs.<name>]` — `LockDocEntry` struct

Struct defined at `crates/ggen/src/pack.rs:232`. `deny_unknown_fields` attribute at `crates/ggen/src/pack.rs:231`.

| Field | TOML key | Type | Required? | Line |
|---|---|---|---|---|
| `source` | `source` | `String` | yes (no `#[serde(default)]`) | `crates/ggen/src/pack.rs:233` |
| `content_hash` | `content_hash` | `String` | yes (no `#[serde(default)]`) | `crates/ggen/src/pack.rs:234` |

Constant `LOCK_FILE_NAME` = `"ggen.lock"` at `crates/ggen/src/pack.rs:209`, documented as living "at the project root next to `ggen.toml`" (`crates/ggen/src/pack.rs:208`). `source` format: `path:<path>` for `PackRef::Path` or `git:<git>@<version>` for `PackRef::Git`, per `source_string` at `crates/ggen/src/pack.rs:239`-`crates/ggen/src/pack.rs:243`. `content_hash` format: `blake3:<hex>`, per `lock_entries` at `crates/ggen/src/pack.rs:262`. Parse errors for `ggen.lock` (missing is not an error; malformed is) surface as `FM-PACK-009` (`crates/ggen/src/pack.rs:282`, `crates/ggen/src/pack.rs:285`-`crates/ggen/src/pack.rs:293`); a content-hash mismatch against the lock surfaces as `FM-PACK-008` (`crates/ggen/src/pack.rs:297`).

---

## Closed-vocabulary (`deny_unknown_fields`) summary

Every struct below rejects any TOML key not in its field list, causing a hard parse error rather than a silent ignore.

| Struct | File | `deny_unknown_fields` line | Struct definition line |
|---|---|---|---|
| `GgenConfig` | `crates/ggen/src/config.rs` | `crates/ggen/src/config.rs:18` | `crates/ggen/src/config.rs:19` |
| `Project` | `crates/ggen/src/config.rs` | `crates/ggen/src/config.rs:33` | `crates/ggen/src/config.rs:34` |
| `Ontology` | `crates/ggen/src/config.rs` | `crates/ggen/src/config.rs:41` | `crates/ggen/src/config.rs:42` |
| `Templates` | `crates/ggen/src/config.rs` | `crates/ggen/src/config.rs:70` | `crates/ggen/src/config.rs:71` |
| `PackToml` | `crates/ggen/src/pack.rs` | `crates/ggen/src/pack.rs:38` | `crates/ggen/src/pack.rs:39` |
| `PackMeta` | `crates/ggen/src/pack.rs` | `crates/ggen/src/pack.rs:45` | `crates/ggen/src/pack.rs:46` |
| `LockDoc` | `crates/ggen/src/pack.rs` | `crates/ggen/src/pack.rs:224` | `crates/ggen/src/pack.rs:225` |
| `LockDocEntry` | `crates/ggen/src/pack.rs` | `crates/ggen/src/pack.rs:231` | `crates/ggen/src/pack.rs:232` |

`PackRef` (`crates/ggen/src/config.rs:53`) has no `deny_unknown_fields` attribute of its own; it is `#[serde(untagged)]` (`crates/ggen/src/config.rs:52`), so closedness comes from each variant's fixed field set failing to match tables with extra or missing keys, not from an explicit attribute.
