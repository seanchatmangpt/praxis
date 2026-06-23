//! Integration tests. These tests may require Docker or other external services.
//!
//! Use `skip_without_docker!()` at the top of any test that needs a running
//! Docker daemon. Such tests are skipped (not failed) on CI agents without Docker.
//!
//! Run: cargo test --test integration

use std::net::TcpListener;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Bind to port 0, let the OS assign a free port, immediately release it,
/// and return the port number.
///
/// There is an inherent TOCTOU race between the release and your subsequent bind.
/// For tests that need *a* free port (not a guaranteed-reserved one) this is fine.
fn allocate_ephemeral_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind to port 0")
        .local_addr()
        .expect("local_addr")
        .port()
}

/// Skip the calling test if Docker is unreachable.
/// Checks both the UNIX socket and TCP port 2375 (Docker daemon default).
macro_rules! skip_without_docker {
    () => {
        let docker_available = std::net::TcpStream::connect("127.0.0.1:2375").is_ok()
            || std::path::Path::new("/var/run/docker.sock").exists();
        if !docker_available {
            eprintln!("SKIP: Docker daemon not reachable — skipping test");
            return;
        }
    };
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn ephemeral_port_is_in_user_range() {
    let port = allocate_ephemeral_port();
    assert!(port > 1024, "OS should assign an unprivileged port, got {port}");
}

#[test]
fn ephemeral_ports_differ() {
    // Two successive calls should (almost always) produce different ports.
    let p1 = allocate_ephemeral_port();
    let p2 = allocate_ephemeral_port();
    // Not strictly guaranteed, but a collision would be extraordinary.
    assert_ne!(p1, p2, "expected different ports; got {p1} twice");
}

/// Example integration test requiring Docker.
/// Replace the body with your actual scenario (e.g., spin up a postgres container).
#[test]
fn example_docker_test() {
    skip_without_docker!();
    // With Docker available, use testcontainers-rs or docker CLI here.
    // Example (with testcontainers):
    //   let docker = clients::Cli::default();
    //   let container = docker.run(images::postgres::Postgres::default());
    //   let conn_str = format!("postgres://postgres@localhost:{}/postgres",
    //       container.get_host_port_ipv4(5432));
    println!("Docker is available — integration test placeholder passed");
}

/// Example: test that must not run concurrently (e.g., mutates a global or binds a port).
/// Uncomment `#[serial_test::serial]` and add `serial_test` to dev-dependencies.
// #[serial_test::serial]
// #[test]
// fn exclusive_resource_test() { ... }

/// Example: async integration test hitting a real HTTP endpoint.
#[tokio::test]
async fn async_http_placeholder() {
    let port = allocate_ephemeral_port();
    // Real test would: start a server on `port`, issue a reqwest call, assert response.
    assert!(port > 1024);
}
