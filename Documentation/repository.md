# Repository Layout

```
Shared/
  ArcadiaCore/
    Cargo.toml                        # crate-type: staticlib + cdylib + lib
    src/
      lib.rs                          # root, exports + UniFFI scaffolding
      ffi.rs                          # UniFFI → Swift (iOS bridge)
      navigation.rs                   # PAGE_DEFINITIONS, GROUP_DEFINITIONS, registry JSON
      config/
        mod.rs                        # ConfigFile trait, config root path
        modules.rs                    # MODULE_REGISTRY, ModulesConfig, migrations
        commandline.rs                # CLI preferences
        thin_client.rs                # ThinClientConfig → thin-client.toml
      modules/
        mod.rs                        # execute_command dispatcher, module lifecycle
        shell.rs                      # shell.execute, shell.internal, PTY
        shell_motd.rs                 # MOTD banner
        surface.rs                    # surface.snapshot / surface.patch
        remote_session.rs             # routing manifest entry (no standalone commands)
        remote_mirror.rs              # host transcript queue + FFI drain
        net.rs                        # networking foundation
        lan/                          # LAN subsystem (see reference.md)
          mod.rs, discovery.rs, handlers.rs, config.rs, peers.rs, protocol.rs
      platform/
        mod.rs, macos.rs, ios.rs, linux.rs, windows.rs, unknown.rs
  Scripts/
    Builds/build-ios-framework.sh               # Rebuild xcframework + Swift bindings
    Installers/install-global-commands-macos.sh # Install ~/.local/bin wrappers
    Installers/install-git-hooks.sh             # Enable repo .githooks (FFI pre-commit, etc.)
    Launchers/Launcher.sh / Launcher.ps1        # Shell launcher menus
  Tools/uniffi-bindgen/               # UniFFI bindgen binary (workspace member)

Desktop/
  Cargo.toml                          # features: headless (default), gui
  src/
    main.rs                           # binary entry, feature-gated GUI vs headless
    cli/
      mod.rs                          # REPL loop, startup messages
      args.rs                         # argument parsing
      completion.rs                   # shell completion
      config_cmds.rs                  # module/config CLI commands
      module_cmds.rs                  # module shortcut commands
    gui/
      mod.rs
      assets.rs                       # embedded SVG asset loading
      app/
        mod.rs                        # ArcadiaRoot state, ShellMode enum
        entry.rs                      # GPUI initialization
        lifecycle.rs                  # focus, resize, module reload
        navigation.rs                 # nav state and page routing
        root/mod.rs, render.rs        # root layout + render
        root/top_bar.rs               # title bar, session chip, shell mode toggle
        sidebar/mod.rs, layout.rs, nav_items.rs
        shell/mod.rs, panel.rs, execute.rs, keys.rs, tui_screen.rs, mirror.rs
        modules_page/mod.rs, panel.rs, row.rs, requirements_modal.rs
        lan_nodes/mod.rs, panel.rs
        splash/mod.rs, view.rs, draw_*.rs, math.rs
      theme/
        mod.rs                        # icon_path(), color constants
        chrome.rs                     # window chrome
        icons.rs                      # icon metadata
        splash_colors.rs
        modules/                      # component tokens (buttons, panel, rows, toggles, typography)
        nav_accents/                  # per-accent palettes (mod.rs, palette.rs, 9 accents)
      tui/
        mod.rs, session.rs            # PTY session lifecycle
        ansi_line.rs                  # ANSI escape parsing
        colors.rs                     # terminal color palette
        cd_builtin.rs, cwd.rs, env.rs # shell builtins + CWD tracking
        keys.rs                       # PTY keyboard events
        vt_history.rs                 # VT100 history buffer
  assets/icons/                       # SVG icons (home, terminal, logs, settings, nodes, modules, tools)

Mobile/iOS/
  ArcadiaApp/
    ArcadiaApp.swift                  # @main, config root setup
    ContentView.swift                 # top-level coordinator + Actions/Layout/NavigationState/Registry extensions
    NavigationModels.swift            # Swift structs mirroring NavigationRegistry
    AppTheme.swift                    # all iOS colors as computed properties
    SidebarView.swift                 # sidebar rendering + remote session picker
    SplashView.swift                  # animated splash
    ShellView.swift                   # shell command input + history
    ModulesView.swift                 # module toggle list
    LanNodesView.swift                # LAN peer discovery + pairing
    ModuleNames.swift                 # string constants mirroring MODULE_REGISTRY
    GlassComponents.swift             # reusable glassmorphism components
  ArcadiaCore/                        # Generated Swift + ArcadiaCore.xcframework (rebuild after ffi.rs changes)

Configuration/                        # Layout reference (runtime: ~/Arcadia/Configuration on Desktop)
  modules.toml                        # module enable/disable state
  commandline.toml                    # CLI preferences
  thin-client.toml                    # preferred_remote_route, surface_client_id

Resources/
  Wallpapers/                         # Landscape.png, Portrait.png, Landscape-Refined.png
  Sounds/                             # Notification_* (Warm, Pop, Minimal, Glass, Deep, Airy)
  Icons/                              # App icon prototypes + Final-1-appicon.png

Launchers/Development/OSX/            # SwiftPM menu bar launcher (optional, dev only)
  Package.swift
  Sources/ArcadiaDevelopmentLauncher/main.swift
  build-app.sh, README.md

.github/workflows/
  stable-build-matrix.yml             # Desktop + iOS simulator CI
  FUNDING.yml                         # GitHub Sponsors

gaps.md                               # Deliberate limitations and next-tier work
CLAUDE.md                             # Contributor guide (architecture patterns)
AGENTS.md                             # Agent rules (registry discipline, anti-patterns)
```
