# {{project-name}} — Integration Test Guide

Integration tests live here, separate from unit tests in `src/`, because they:

- May require Docker, a running service, or an active network connection
- Run slower than unit tests (seconds to minutes, not milliseconds)
- Should not block `cargo test --lib` (the fast feedback loop)

---

## Running

```bash
# Unit tests only — fast, no external deps
cargo test --lib

# Integration tests — may start Docker containers
cargo test --test integration

# Integration tests with verbose output
RUST_LOG=debug cargo test --test integration -- --nocapture
```

## Patterns

### `allocate_ephemeral_port()`

Bind port `0`, let the OS assign a free port, immediately release the listener,
return the port number. Use when you need to start a test server on a known port.

There is an inherent TOCTOU race between release and your next bind — another
process could grab the port. For isolated test environments this is acceptable.

### `skip_without_docker!()`

Call at the very top of any test that requires Docker. Checks `/var/run/docker.sock`
and TCP port 2375. Skips the test (`return`) when Docker is not reachable —
the test is **not** marked as failed.

```rust
#[test]
fn needs_real_db() {
    skip_without_docker!();
    // ... docker-dependent body ...
}
```

### `#[serial_test::serial]`

Tests that bind a fixed port, write to a shared file, or otherwise cannot run
in parallel must be annotated with `#[serial_test::serial]`. Add `serial_test = "3"`
to `[dev-dependencies]`.

### Async tests

Use `#[tokio::test]` for async integration tests. Add `tokio = { version = "1",
features = ["full"] }` to `[dev-dependencies]`.

### Docker containers with testcontainers-rs

Add `testcontainers = "0.23"` to `[dev-dependencies]`. Containers start in the
test body and are stopped when the `ContainerAsync<I>` guard is dropped (RAII).

```rust
use testcontainers::{clients::Cli, images::postgres::Postgres};

#[tokio::test]
async fn with_real_postgres() {
    skip_without_docker!();
    let docker = Cli::default();
    let container = docker.run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);
    // connect to postgres on port, run assertions
}
```

## CI

Integration tests run in a separate `integration.yml` workflow, not in `ci.yml`.
This keeps the main CI gate fast while still requiring integration tests to pass
before release.

The `integration.yml` workflow runs on:
- Every push to `main`
- Every pull request
- Manual `workflow_dispatch` trigger

Docker is available by default on `ubuntu-latest` GitHub Actions runners.
Add `services:` blocks to `integration.yml` to start containers (Postgres, Redis, etc.)
before tests run.
