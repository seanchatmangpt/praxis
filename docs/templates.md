# Project Templates (`templates/` & `template-*`)

Praxis includes standard blueprints for quick scaffolding of new modules, applications, and integration suites. These conform to seanchatmangpt fleet conventions and MSRV bounds.

All templates are located under:
- **[`templates/`](file:///Users/sac/praxis/templates/)**: Custom template definitions.
- **[`template/`](file:///Users/sac/praxis/template/)**: Standard single-crate library or executable CLI project.
- **[`template-wasm/`](file:///Users/sac/praxis/template-wasm/)**: Optimized target configurations for WebAssembly libraries.
- **[`template-mcp/`](file:///Users/sac/praxis/template-mcp/)**: Model Context Protocol servers to expose CLI tools to LLM models.
- **[`template-integration/`](file:///Users/sac/praxis/template-integration/)**: Containerized integration test suites.
- **[`template-workspace/`](file:///Users/sac/praxis/template-integration/)**: Scaffold for multi-crate cargo workspaces.

## Structure and Generation

Templates utilize placeholders processed by `cargo-generate`. A generated project outputs:
- Standard `justfile` tasks (`just ci`, `just test`, `just lint`).
- A `rust-toolchain.toml` targeting stable Rust 1.82.
- A `CLAUDE.md` developer guide detailing coding patterns and instructions.
- Integrated `chatman-common` bindings for error handling and logging.
