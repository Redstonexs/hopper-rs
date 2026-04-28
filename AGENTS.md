# Agent Instructions for Hopper

## Project Overview
Hopper is a lightweight Rust-based reverse proxy for Minecraft servers. Single crate, straightforward architecture: config parsing → server setup → async TCP relay with metrics collection.

## Build & Run

### Dev build
```sh
cargo build
cargo run
```

### Release build (optimized with LTO and symbol stripping)
```sh
cargo build --release
```
Binary: `target/release/hopper`

### Test configuration
Create a `Config.toml` in the working directory (required for runtime). Hopper expects this path relative to `WorkingDirectory` (see systemd config in README).

### Docker
```sh
docker build -t hopper .
docker run -p 25565:25565 -v ./Config.toml:/Config.toml hopper
```

## Key Architecture

**Entrypoint:** `src/main.rs:run()` (src/main.rs:43)
- Reads `Config.toml` via `ServerConfig::read()`
- Sets up TCP listener on configured address
- Spawns `Hopper::new()` server instance
- Supports hot reload on Linux via SIGHUP signal

**Core modules:**
- `config.rs` + `config/` — TOML parsing for routes, metrics, IP forwarding modes
- `server.rs` + `server/` — TCP relay logic (Hopper struct)
- `protocol/` — Minecraft protocol handling (packet routing, player handshake)
- `metrics.rs` + `metrics/` — InfluxDB integration (optional, injector pattern)

**IP Forwarding modes:** bungeecord, realip, proxy_protocol, none

## Config Location & Format

**File:** `Config.toml` in working directory (or via volume mount in Docker)

**Required fields:**
```toml
listen = "0.0.0.0:25565"  # Hopper listens here

[routing]
default = { ip = "127.0.0.1:12345" }  # optional fallback

[routing.routes]
"hostname.example.com" = { ip = "backend:25565", ip-forwarding = "none" }
```

See README.md for full config examples (load balancing, metrics, IP forwarding).

## Logging

**Environment variable:** `RUST_LOG`
- Default: `info`
- Options: `off`, `error`, `info`, `debug`

Example:
```sh
RUST_LOG=debug ./hopper
```

## Testing & Validation

No automated test suite in this repo. Manual validation:
1. Build succeeds: `cargo build --release`
2. Config parses: `RUST_LOG=debug ./hopper` (exits cleanly if Config.toml invalid)
3. Docker image builds: `docker build -t hopper .`

## Release Process

- Releases are triggered on git tags
- CI workflows (`build.yml`, `docker.yml`) auto-run on tag push
- Builds binaries for: Windows (x86_64-gnu), Linux (x86_64-musl), macOS (x86_64-darwin)
- Docker image pushed to `ghcr.io/bra1l0r/hopper-rs:latest`

## Common Tasks

### Add a config field
1. Update `src/config.rs` (deserialize struct)
2. Pass it through `src/server.rs` if it affects runtime behavior
3. Test by creating a Config.toml with the new field

### Add an IP forwarding mode
1. Define packet transform in `src/protocol/` (or extend existing handlers)
2. Add mode string to `ip-forwarding` enum in config
3. Route packet flow in `src/server.rs` based on selected mode

### Debug connection issues
- Check `RUST_LOG=debug ./hopper` output (packet handshake, routing decisions)
- Verify Config.toml routes match client hostname and backend is reachable
- Backend must support the configured IP forwarding mode (bungeecord/RealIP enabled in target server)

## Platform Notes

- Hot reload (SIGHUP) only works on Linux; Windows/macOS use polling or manual restart
- Zerocopy feature (optional): requires libc, used for zero-copy packet handling
- RealIP max supported version: v2.4 (older dependency, may lack newer MC versions)
