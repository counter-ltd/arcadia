# Arcadia

**One Rust core. One Python SDK. An infinite extension surface. Zero rent.**

Arcadia is a multi-platform runtime, shell, and — ultimately — an **open platform for building system-integrated applications**. One `arcadia-core` crate owns every module, command, navigation structure, LAN protocol, and config schema, consumed by two native surfaces (GPUI desktop, SwiftUI iOS) plus a CLI.

Built on the same DNA as **[Holos](https://github.com/stack-node/holos)** — *utility over monetization, ownership over subscriptions* — but with a harder engineering mandate: **no duplicated truth between platforms, no hardcoded IDs in surface code, no growing if-else chains that break the next time a module is added.**

---

## What Arcadia is

- **A runtime and shell** — execute commands locally or route them across your LAN with the same `execute_command` API.
- **A module registry** — enable/disable capabilities (`shell`, `lan`, `net`, `surface`, `remote-session`, `shell-motd`) from any surface; the registry enforces dependencies.
- **A navigation system** — page and group definitions live in `navigation.rs`, serialized to JSON for iOS, consumed by Desktop directly. No surface hardcodes page IDs.
- **A thin-client protocol** — `surface.snapshot` mirrors host state (modules, nav registry, revision) to clients; `surface.patch` lets clients push changes back.
- **A cross-platform core** — the same Rust crate (`arcadia-core`) builds as a staticlib for iOS (via UniFFI), a native library for Desktop GPUI, and a CLI binary.

---

## What you can do with it now

| Capability | How |
|------------|-----|
| Native shell / PTY terminal | `shell.execute` (routable), `shell.internal` (REPL), full PTY/TUI on Desktop |
| Shell welcome banner | `shell-motd` module — fastfetch-style on shell open |
| Manage modules | CLI (`module enable/disable`) or GUI toggle; same `modules.toml` |
| Discover LAN peers | `lan.scan`, `lan.node`, LAN nodes UI on Desktop and iOS |
| Route commands to another machine | `ExecutionContext.net_as = "lan:IP"`, session chip on Desktop, route picker on iOS |
| Mirror host UI state to clients | `surface.snapshot` — modules + nav registry + revision |
| Push module changes from client to host | `surface.patch` with `modules_set` op |
| Run headless as a host | `cargo run` (default `headless` feature) |
| Rebuild iOS after FFI changes | `bash Shared/Scripts/Builds/build-ios-framework.sh` |
| Install global CLI wrappers | `bash Shared/Scripts/Installers/install-global-commands-macos.sh` |

---

## Development status

Moves fast. Breaks occasionally. That's intentional.

- Features land continuously on `development`.
- APIs (especially FFI and `surface.*`) may evolve — see [Roadmap](Documentation/roadmap.md) for deliberate limitations.
- Building from source is the surest way to stay current.
- Stable tagged builds will appear as the project matures; CI exercises desktop + iOS simulator paths.

Known gaps are tracked in-repo instead of pretending shipping equals finished.

---

## Quick start

**Prerequisites:**

| Tool | Required for |
|------|-------------|
| Rust (`rustup`, `cargo`) | Core + Desktop |
| Xcode + CLI tools | iOS app + xcframework build |
| `rustup target add aarch64-apple-ios aarch64-apple-ios-sim` | `Builds/build-ios-framework.sh` |

**Build:**

```sh
# Desktop GUI
cd Desktop && cargo run --features gui

# Desktop CLI (headless)
cd Desktop && cargo run

# Core tests
cd Shared && cargo test -p arcadia-core

# iOS framework (after ffi.rs changes)
bash Shared/Scripts/Builds/build-ios-framework.sh
```

---

## Documentation

- [Vision](Documentation/vision.md) — why this exists and where it's going
- [Architecture](Documentation/architecture.md) — philosophy, command model, module system, navigation, thin-client, FFI
- [Module & Navigation reference](Documentation/reference.md) — all modules and pages
- [Repository layout](Documentation/repository.md) — full directory map
- [Configuration](Documentation/configuration.md) — config files, prerequisites, environment variables
- [Build & run](Documentation/build.md) — all build targets and scripts
- [Contributing](Documentation/contributing.md) — rules, adding features, testing
- [Roadmap & known gaps](Documentation/roadmap.md) — P0–P3 gaps, security posture, CI
- [Lineage & about](Documentation/about.md) — history, creator, donations
