# Arcadia

**One Rust core. One Python SDK. An infinite extension surface. Zero rent.**

Arcadia is a multi-platform runtime and shell — an **open platform for building system-integrated applications**. A single `arcadia-core` crate owns modules, commands, navigation, LAN protocol, and config; GPUI (desktop), SwiftUI (iOS), and a CLI are thin surfaces over that core.

Built on the same DNA as **[Holos](https://github.com/stack-node/holos)** — *utility over monetization, ownership over subscriptions* — with a hard rule: **no duplicated truth between platforms, no hardcoded IDs in surface code, no growing dispatch chains that break the next time a module ships.**

---

## Documentation

In-depth guides live under [`Documentation/`](Documentation/). Use this as the map:

| Doc | What it covers |
|-----|----------------|
| [**Vision**](Documentation/vision.md) | Why Arcadia exists and where it is headed |
| [**Architecture**](Documentation/architecture.md) | Command model, modules, navigation, thin-client, FFI |
| [**Module & navigation reference**](Documentation/reference.md) | Every module and page in the registry |
| [**Repository layout**](Documentation/repository.md) | Directory map of the whole repo |
| [**Configuration**](Documentation/configuration.md) | Config files, prerequisites, environment variables |
| [**Build & run**](Documentation/build.md) | All targets, scripts, and platform builds |
| [**Contributing**](Documentation/contributing.md) | Conventions, adding features, testing |
| [**Roadmap & known gaps**](Documentation/roadmap.md) | Priorities, limitations, security posture, CI |
| [**Lineage & about**](Documentation/about.md) | History, creator, supporting the project |

---

## License

Arcadia is released under the **Arcadia Community License (ACL) v1.6** ([`LICENSE.md`](LICENSE.md)).

In short: the software is meant for **people** — personal, educational, research, accessibility, and community use — with **attribution** and rules that keep the **core** improvements open when distributed. **Large corporations** and **profit-focused commercialization of Arcadia itself** (or selling Arcadia-targeted extensions without permission) are out of scope unless the copyright holder grants written permission. Education and learning get a broad, explicit welcome. The full text defines thresholds, conditions, and philosophy; **do not rely on this paragraph alone** for compliance.

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
- APIs (especially FFI and `surface.*`) may evolve — see [**Roadmap**](Documentation/roadmap.md).
- Building from source is the surest way to stay current.
- Stable tagged builds will appear as the project matures; CI exercises desktop + iOS simulator paths.

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
