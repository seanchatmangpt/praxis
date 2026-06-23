# {{project-name}} — WASM Developer Guide

**Version:** CalVer `YY.M.patch`
**Target:** `wasm32-unknown-unknown` (via wasm-pack)
**License:** MIT OR Apache-2.0

---

## WASM-Specific Rules

### Never use `strip = true` in `[profile.release]`

Stripping WASM binaries through rustc corrupts them — the resulting `.wasm` file
will fail to parse. Size reduction must go through `wasm-opt`:

```bash
just build-opt   # wasm-pack build + wasm-opt -Os
```

This profile is already correct in this template — do not add `strip = true`.

### Always use `BTreeMap` over `HashMap` in WASM-exposed types

WASM guest functions are pure functions — the same input must always produce the
same output. `HashMap` iteration order is randomised by hash seeds at startup,
which makes output non-deterministic. Use `std::collections::BTreeMap` for any
map that crosses the WASM boundary or appears in serialised output.

### Handle-based API (never raw pointers)

Store Rust objects in `Store<T>` (`src/store.rs`). Return opaque hex string
handles to JavaScript. Never return raw pointers across the WASM boundary:

```rust
// ✓ Correct: handle-based
#[wasm_bindgen]
pub fn create_session(name: &str) -> String {
    SESSIONS.with(|s| s.borrow_mut().insert(Session::new(name)))
}

// ✗ Wrong: raw pointer — unsafe and breaks on GC
#[wasm_bindgen]
pub fn create_session_bad(name: &str) -> *mut Session { ... }
```

### Always initialise the panic hook

`console_error_panic_hook::set_once()` must be called in `#[wasm_bindgen(start)]`.
Without it, Rust panics are completely silent in browsers.

### `getrandom` JS feature

Any crate that uses `getrandom` (directly or transitively) must enable the `js`
feature for WASM targets:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

The same applies to `uuid` and any other crate that generates random data.

---

## Build

```bash
just build-wasm    # wasm-pack web target (release)
just build-opt     # wasm-pack + wasm-opt -Os (size optimised)
just build-mobile  # opt-level = "z" (extreme size, mobile browsers)
just wasm-size     # print binary sizes (+ twiggy symbol breakdown if installed)
just test          # native unit tests (fast, no wasm-pack needed)
just ci            # full native gate (fmt + lint + test)
```

## Size Budgets

| Profile | Cargo profile | Use case | Target |
|---|---|---|---|
| `just build-mobile` | `release-mobile` (`opt-level = "z"`) | Mobile browsers | ≤ 500 KB |
| `just build-wasm` | `release` (`opt-level = "s"`) | Standard web | ≤ 1 MB |
| `just build-opt` | `release` + `wasm-opt -Os` | Optimised web | ≤ 800 KB |

Run `just wasm-size` after each build to verify the binary stays within budget.

## CI

CI is two-phase:
1. `test-native` — `cargo fmt`, `cargo clippy`, `cargo test --lib` (fast, runs first)
2. `build-wasm` — `wasm-pack build --target web --release` (runs after phase 1 passes)

Both phases must pass for `ci-success` to green.

## Adding a New API Surface

1. Add a struct or function to `src/lib.rs` (or a new module).
2. Annotate with `#[wasm_bindgen]`.
3. If the function manages state, store it in `Store<T>` and return a handle.
4. Add a `#[cfg(test)]` unit test in the same file.
5. Run `just build-wasm` to verify the WASM build.
