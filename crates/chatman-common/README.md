# chatman-common

Shared house crate for the seanchatmangpt Rust fleet.

## Features

| Feature      | What it enables                                      |
|-------------|------------------------------------------------------|
| `serde`     | `serde` + `serde_json` support, `Error::Json` variant |
| `telemetry` | `tracing` + `tracing-subscriber`, `init_tracing()`   |
| `otel`      | Stub OTLP init (implies `telemetry`)                 |
| `cli`       | `clap`-based `GlobalArgs`, `OutputFormat`, `ColorMode`|
| `provenance`| BLAKE3 content addressing and rolling hash           |
| `testkit`   | Golden-file assertions, `tempfile` re-export         |
| `full`      | All of the above except `otel`                       |

## Quick start

```toml
[dependencies]
chatman-common = { version = "26.6.0", features = ["full"] }
```
