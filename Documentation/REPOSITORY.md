# Repository Layout

```
Shared/
  Cargo.toml                          # workspace: ArcadiaCore, ArcadiaPython
  ArcadiaCore/
    Cargo.toml                        # crate-type: lib (no staticlib/cdylib — iOS lib lives in Desktop)
    src/
      lib.rs                          # module declarations
      navigation.rs                   # PAGE_DEFINITIONS, GROUP_DEFINITIONS, registry JSON
      services.rs                     # SERVICE_DEFINITIONS, service-host helpers
      config/
        mod.rs                        # ConfigFile trait, CONFIG_ROOT_OVERRIDE (iOS sets this)
        modules.rs                    # MODULE_REGISTRY, ModulesConfig, migrations
        appearance.rs                 # appearance.toml (theme tokens)
        commandline.rs                # CLI preferences
        extension_tokens.rs           # extension theme/glyph token resolution
        late.rs                       # late.toml (WS chat config)
        thin_client.rs                # ThinClientConfig → thin-client.toml
        workspace.rs                  # WorkspacesConfig, WorkspaceEntry, WorkspacePermissionDef → workspace.toml
      modules/
        mod.rs                        # execute_command dispatcher, module_commands() lookup
        shell.rs                      # shell.execute, shell.internal
        shell_motd.rs                 # MOTD banner
        surface.rs                    # surface.snapshot / surface.patch / revision
        remote_session.rs             # routing manifest entry (no standalone commands)
        remote_mirror.rs              # host transcript queue + drain
        net.rs                        # networking foundation
        late.rs                       # WebSocket chat / now-playing / votes
        python_host.rs                # Python extension host
        python_registry.rs            # Python extension manifest registry
        workspace.rs                  # workspace.list/add/remove/grant/revoke/check commands
        lan/                          # LAN subsystem
          mod.rs, discovery.rs, handlers.rs, config.rs, peers.rs, protocol.rs
      platform/
        mod.rs, macos.rs, ios.rs, linux.rs, windows.rs, unknown.rs
  ArcadiaPython/                      # Python extension runtime (workspace member)
  Scripts/
    Builds/build-ios-app.sh           # Builds libarcadia_ios.a (for manual invocation)
    Builds/build-paths.sh             # Shared path helpers for Builds/ tree
    Installers/install-global-commands-macos.sh
    Launchers/Launcher.sh, Launcher.ps1

Libraries/
  OpenFrame/                          # Submodule (GPUI fork; supports macOS, Linux, Windows, iOS)

Extensions/                           # Submodule (extension catalog; shipped demos)

Desktop/
  Cargo.toml                          # package `arcadia`
                                      #   [[bin]] arcadia  (src/main.rs)
                                      #   [lib]   arcadia_ios (src/ios_lib.rs, crate-type=staticlib)
                                      # features: headless (default), gui, ios-gui, python-extensions
  src/
    main.rs                           # binary entry, feature-gated GUI vs headless
    ios_lib.rs                        # iOS C ABI (arcadia_ios_start, arcadia_ios_inject_touch)
    cli/
      mod.rs                          # REPL loop, startup messages
      args.rs, completion.rs, config_cmds.rs, module_cmds.rs
    gui/
      mod.rs, assets.rs
      app/
        mod.rs                        # ArcadiaRoot state
        entry.rs                      # Desktop GPUI bootstrap
        entry_ios.rs                  # iOS OpenFrame bootstrap (called from arcadia_ios_start)
        lifecycle.rs                  # focus, resize, module reload
        navigation.rs                 # nav state and page routing
        workspace_panel.rs            # Workspaces page — list + search
        workspace_row.rs              # Per-workspace row with permission toggles
        workspace_create_modal.rs     # Create-workspace modal (label + path fields)
        root/, sidebar/, shell/, modules_page/, lan_nodes/, splash/,
        appearance/, late/, python_settings/, services/, list_panel_search.rs,
        text_input_caret.rs
      theme/
        mod.rs                        # icon_path(), color constants
        chrome.rs, icons.rs, splash_colors.rs
        modules/                      # component tokens
        nav_accents/                  # per-accent palettes
      tui/                            # PTY/TUI terminal emulator (desktop only)
  assets/icons/                       # SVG icons

Mobile/iOS/
  ArcadiaApp.xcodeproj/               # Xcode project — "Build Rust (cargo)" build phase
  ArcadiaApp/
    ArcadiaApp.swift                  # UIKit @main; bootstraps Metal layer; calls arcadia_ios_start
    MetalHostView.swift               # CAMetalLayer host; forwards UITouch → arcadia_ios_inject_touch
    ArcadiaBridge.h                   # C ABI declarations matching Desktop/src/ios_lib.rs
    Info.plist, Assets.xcassets

Configuration/                        # Layout reference (runtime: ~/Arcadia/Configuration on Desktop)
  modules.toml                        # module enable/disable state
  commandline.toml                    # CLI preferences
  thin-client.toml                    # preferred_remote_route, surface_client_id
  appearance.toml                     # theme tokens
  late.toml                           # late module state

Resources/
  Wallpapers/, Sounds/, Icons/

Launchers/Development/OSX/            # SwiftPM menu bar launcher (optional, dev only)

Builds/workspace/                     # Unified Cargo target-dir (gitignored) — driven by /.cargo/config.toml
Builds/Mobile/iOS/DerivedData/        # Xcode -derivedDataPath

.github/workflows/
  core-tests.yml                      # arcadia-core unit tests
  stable-build-matrix.yml             # Desktop matrix + iOS simulator build on stable

Documentation/                        # ARCHITECTURE, BUILD, CONFIGURATION, CONTRIBUTING, GAPS, ROADMAP, VISION
CLAUDE.md                             # Contributor guide (architecture patterns)
AGENTS.md                             # Agent rules (registry discipline, anti-patterns)
```
