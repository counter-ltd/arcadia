# Feature Documentation

Each file covers one feature area in depth: architecture, data types, commands, config, GUI panels, and integration points.

| File | Feature | Module(s) |
|------|---------|-----------|
| [Terminal.md](Terminal.md) | Shell execution, PTY/TUI, MOTD banner | `terminal`, `terminal-motd` |
| [AI.md](AI.md) | AI providers, chat, tools, sandbox | `ai`, `ai-provider-*` |
| [LAN_and_Remote.md](LAN_and_Remote.md) | LAN discovery, routing, surface mirroring, thin client | `net`, `lan`, `remote-session`, `surface` |
| [Workspaces.md](Workspaces.md) | Directory registry, scoped permissions | `workspace` |
| [Extensions.md](Extensions.md) | Python extension host, style tokens, overlay companion | `python-host` |
| [Appearance.md](Appearance.md) | Theme tokens, accent palettes, extension tokens | — (theme layer) |
| [Modules.md](Modules.md) | Module system, enable/disable, dependencies, migrations | all modules |
| [Navigation.md](Navigation.md) | Pages, groups, settings hub, service-driven visibility | — (core registry) |
| [Late.md](Late.md) | Chat, now playing, votes, visualizer, bonsai | `late` |
| [Services.md](Services.md) | Service registry, start/stop, port collision recovery | — (service registry) |
| [Overlay_and_Cursor.md](Overlay_and_Cursor.md) | HUD overlay, stacking levels, global cursor | `overlay`, `cursor` |
| [Shortcuts.md](Shortcuts.md) | Keyboard shortcuts, OS-global, overrides, custom | — (shortcut registry) |
| [CodeEditor.md](CodeEditor.md) | Code editor panel, syntax highlighting, decorations | `code-editor` |
| [Permissions.md](Permissions.md) | Capability grants, module/extension subjects | `permissions` |
