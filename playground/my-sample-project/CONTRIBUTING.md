# Contributing

Thanks for contributing. This repo follows the seanchatmangpt Rust house style.

## Getting started

```bash
cargo install cargo-deny typos-cli just
rustup show     # toolchain auto-installed from rust-toolchain.toml
just            # list tasks
just ci         # run everything CI runs
```

The toolchain is pinned to stable `1.82.0` via `rust-toolchain.toml`.

## Development workflow

1. Branch from `main`.
2. Make your change, with tests.
3. Run `just ci` until green (fmt, clippy `-D warnings`, test, deny, typos).
4. Add a `CHANGELOG.md` entry under `## [Unreleased]` for user-visible changes.
5. Open a PR using the pull request template; check only what you actually verified.

### Pre-commit gate

`just ci` runs the same steps CI runs:

| Step | Command |
|------|---------|
| Format check | `cargo fmt --check` |
| Lint | `cargo clippy -- -D warnings` |
| Test | `cargo test` |
| Dependency audit | `cargo deny check` |
| Spell check | `typos` |

### Commit format

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): short description
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`.

## Code style

- `cargo fmt` is law (`rustfmt.toml`, stable options only).
- No new `unwrap`/`expect`/`panic` in library code paths — use `?` and `thiserror`.
- Public items get rustdoc (`missing_docs` is on).
- Errors via `thiserror`; binaries may add context with `anyhow`.
- `todo!` and `unimplemented!` are denied by Clippy; stub with a returning `Err(...)`.
- `dbg!` is denied; remove all debug prints before committing.

## Versioning & releases

CalVer `YY.M.patch`. A release is cut by pushing a `vYY.M.patch` tag, which
triggers `.github/workflows/release.yml`.

## License

By contributing you agree your work is licensed under MIT OR Apache-2.0.
