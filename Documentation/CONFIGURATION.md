# Configuration

## Config files

Runtime config root: `~/Arcadia/Configuration/` on Desktop. On iOS, `arcadia_ios_start` calls `arcadia_core::config::set_config_root(path)` with the app's Documents directory before any config reads.

| File | Struct | Purpose |
|------|--------|---------|
| `modules.toml` | `ModulesConfig` | Per-module on/off state |
| `commandline.toml` | `CommandlineConfig` | CLI preferences (scaffold) |
| `thin-client.toml` | `ThinClientConfig` | `preferred_remote_route`, `surface_client_id`, `navigation_from_host_only` |
| `appearance.toml` | `AppearanceConfig` | Theme tokens |
| `late.toml` | `LateConfig` | Late module state |
| `workspace.toml` | `WorkspacesConfig` | Registered workspace directories and their per-workspace permission grants |

Config migrations live in `ModulesConfig::merge_defaults()`. When renaming a module, add a migration entry there — do not do ad-hoc renames at call sites.

---

## Prerequisites

| Tool | Required for |
|------|-------------|
| Rust (`rustup`, `cargo`) | Core + Desktop |
| Xcode + CLI tools | iOS app build (the Xcode project shells out to cargo) |
| `rustup target add aarch64-apple-ios aarch64-apple-ios-sim` | iOS device + simulator builds |
| Swift (via Xcode) | iOS app + dev launcher |

---

## Environment variables

| Variable | Surface | Purpose |
|----------|---------|---------|
| `ARCADIA_NET_AS` | Desktop GUI, iOS | Bootstrap `net_as` on startup (e.g. `lan:192.168.1.5`). Overrides `thin-client.toml` preferred route. |
| `ARCADIA_IOS_DEVICE_NAME` | iOS deploy scripts | Pin device by name |
| `ARCADIA_IOS_FORCE_UNINSTALL` | iOS deploy scripts | Uninstall before install |
