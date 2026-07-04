# Frontmatter Vocabulary

Reference for the `Frontmatter` struct that defines the closed key set accepted in a ggen
template's leading `--- … ---` YAML block.

Source: `crates/ggen/src/template.rs`, struct `Frontmatter` (lines 25–63).

## Struct-level attributes

| Attribute | Line | Effect |
|---|---|---|
| `#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]` | `crates/ggen/src/template.rs:25` | Derives used on `Frontmatter`; `Deserialize` is what `serde_yaml::from_str` uses to parse the YAML block. |
| `#[serde(deny_unknown_fields)]` | `crates/ggen/src/template.rs:26` | Closes the key set: any YAML mapping key in the frontmatter block that is not one of the fields below causes deserialization to fail rather than being silently ignored. |

## Fields

| Field | Type | Default | Meaning | Line |
|---|---|---|---|---|
| `to` | `String` | none (required) | Output path relative to the project root; Tera-renderable (rendered as a template string before being used as a filesystem path). | `crates/ggen/src/template.rs:29` |
| `sparql` | `BTreeMap<String, String>` | `#[serde(default)]` → empty map | Named SPARQL queries made available to the template body under those names. | `crates/ggen/src/template.rs:31-32` |
| `construct` | `Option<String>` | `#[serde(default)]` → `None` | Optional CONSTRUCT query whose result feeds the template. | `crates/ggen/src/template.rs:34-35` |
| `inject` | `bool` | `#[serde(default)]` → `false` | Inject into an existing file instead of creating a new one. | `crates/ggen/src/template.rs:37-38` |
| `before` | `Option<String>` | `#[serde(default)]` → `None` | Inject before the first line containing this marker. | `crates/ggen/src/template.rs:40-41` |
| `after` | `Option<String>` | `#[serde(default)]` → `None` | Inject after the first line containing this marker. | `crates/ggen/src/template.rs:43-44` |
| `at_line` | `Option<usize>` | `#[serde(default)]` → `None` | Inject at this 1-based line number. | `crates/ggen/src/template.rs:46-47` |
| `skip_if` | `Option<String>` | `#[serde(default)]` → `None` | Skip the write when the existing file already contains this substring. | `crates/ggen/src/template.rs:49-50` |
| `unless_exists` | `bool` | `#[serde(default)]` → `false` | Skip the write entirely when the target file already exists. | `crates/ggen/src/template.rs:52-53` |
| `force` | `bool` | `#[serde(default)]` → `false` | Overwrite an existing, differing file instead of failing closed. | `crates/ggen/src/template.rs:55-56` |
| `when` | `Option<String>` | `#[serde(default)]` → `None` | SPARQL ASK guard: generate only when the graph satisfies it. | `crates/ggen/src/template.rs:58-59` |
| `skip_empty` | `bool` | `#[serde(default)]` → `false` | Skip the write when the rendered body is empty. | `crates/ggen/src/template.rs:61-62` |

## `deny_unknown_fields` note

`Frontmatter` carries `#[serde(deny_unknown_fields)]` at `crates/ggen/src/template.rs:26`. The
doc comment directly above the struct states the consequence explicitly: "The frontmatter key
set is closed (`Frontmatter` uses `deny_unknown_fields`), so any unrecognized key is a hard
error" (`crates/ggen/src/template.rs:4-5`). Only the 12 fields listed above are accepted in the
YAML block; any other mapping key at the top level of the frontmatter causes
`serde_yaml::from_str` to return an error, which `Template::parse` converts into an
`FM-TPL-002` error (see below).

## Related errors

| Code | Condition | Raised at | Exact remediation text |
|---|---|---|---|
| `FM-TPL-001` | Template content does not start with a `---` frontmatter block. | `crates/ggen/src/template.rs:83-89` (`AppError::fm_tpl(1, …)` call) | "template must start with a `---` frontmatter block. Remediation: begin the file with `---`, YAML keys, `---`." |
| `FM-TPL-001` | The opening `---` delimiter is not on its own line (no newline immediately follows it). | `crates/ggen/src/template.rs:91-93` (`AppError::fm_tpl(1, …)` call) | "`---` frontmatter delimiter must be on its own line" |
| `FM-TPL-001` | No closing `---` line is found (unterminated frontmatter block). | `crates/ggen/src/template.rs:94-100` (`AppError::fm_tpl(1, …)` call) | "unterminated frontmatter: no closing `---` line found. Remediation: close the YAML block with a `---` line." |
| `FM-TPL-002` | YAML fails to deserialize into `Frontmatter`, including any unknown frontmatter key (closed key set enforced by `deny_unknown_fields`). | `crates/ggen/src/template.rs:101-111` (`AppError::fm_tpl(2, …)` call) | "frontmatter rejected: {e}. Remediation: use only the closed key set (to, sparql, construct, inject, before, after, at_line, skip_if, unless_exists, force, when, skip_empty)." |

The code-number-to-string formatting (`FM-TPL-{code:03}`) is defined in
`crates/ggen/src/error.rs:104-106`, in the `AppError::fm_tpl` constructor:

```
/// FM-TPL-* failure: template frontmatter parse or render violation.
pub fn fm_tpl(code: u16, msg: impl Into<String>) -> Self {
    Self::Validation(format!("[FM-TPL-{code:03}] {}", msg.into()))
```

(`crates/ggen/src/error.rs:104-106`)

Both doc-comment references on `Template::parse` corroborate the code assignment:
"`[FM-TPL-001]` when the leading frontmatter block is missing or unterminated" and
"`[FM-TPL-002]` when the YAML fails to deserialize, including any unknown frontmatter key
(closed key set, fail closed)" (`crates/ggen/src/template.rs:78-81`).

## Test evidence

Two unit tests in `crates/ggen/src/template.rs` exercise the closed-key-set and
missing-frontmatter errors directly:

| Test | Assertion | Line |
|---|---|---|
| `unknown_frontmatter_key_is_err` | Parses `to: out.rs` plus an unrecognized `vars:` key and asserts the resulting error's `Display` string contains `"FM-TPL-002"`. | `crates/ggen/src/template.rs:312-316` |
| `missing_frontmatter_is_err` | Parses `"no frontmatter here"` (no leading `---`) and asserts the resulting error's `Display` string contains `"FM-TPL-001"`. | `crates/ggen/src/template.rs:319-322` |
