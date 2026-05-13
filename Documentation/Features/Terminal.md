# Terminal

Module name: `terminal`  
MOTD add-on: `terminal-motd`  
Platform: all (shell execution is desktop-only; iOS routes via LAN)

---

## Overview

The terminal feature gives Arcadia an interactive shell within the GUI and exposes `shell.execute` as a routable command so remote surfaces can run commands on a host. Two execution paths exist:

- **PTY/TUI** — a full pseudo-terminal hosted in `Desktop/src/gui/tui/`, used when the GUI shell panel is open. Renders ANSI output, handles terminal resize, supports interactive programs.
- **`shell.execute`** — stateless command execution via `arcadia-core`. Output is ANSI-stripped before returning to non-PTY callers (CLI, LAN routing, surface patches).

---

## Commands

| Command | Description |
|---------|-------------|
| `shell.execute <cmd…>` | Run a shell command. macOS/Linux: `sh -c`; Windows: `cmd /C`. Routable over LAN via `ExecutionContext.net_as`. Not available on local iOS surfaces (returns an error constant; route to a host instead). |
| `shell.internal <cmd…>` | REPL-internal execution path. Registered at startup by the CLI layer via `set_internal_executor`. Returns output of the REPL's own command dispatch — not a system shell. Used when the REPL should handle a command rather than forking a process. |

### MOTD banner (`terminal-motd`)

Requires `terminal`. Renders a fastfetch-style banner when the terminal panel opens. Declared as a separate module so users can keep the terminal without the banner.

---

## PTY/TUI (Desktop only)

Files: `Desktop/src/gui/tui/`

| File | Purpose |
|------|---------|
| `session.rs` | PTY session lifecycle — spawn, resize, kill |
| `vt_history.rs` | Scrollback buffer; bounded ring |
| `ansi_line.rs` | ANSI sequence parser for the rendered line buffer |
| `colors.rs` | 256-colour + true-colour palette mapping |
| `keys.rs` | Key event → VT byte sequence translation |
| `env.rs` | Environment passed to the child process |
| `cwd.rs` | Working directory tracking |
| `cd_builtin.rs` | `cd` handled in-process (PTY CWD doesn't propagate to parent) |

The TUI session is owned by `shell/panel.rs` in the GUI layer. On resize, the panel notifies the session which sends a `SIGWINCH` / `SetConsoleWindowInfo` to the child.

---

## Shell panel (GUI)

Files: `Desktop/src/gui/app/shell/`

| File | Purpose |
|------|---------|
| `panel.rs` | Root shell panel — renders TUI screen or static output depending on mode |
| `execute.rs` | Wraps `shell.execute` for the GUI: posts command, polls for output |
| `mirror.rs` | Remote-mirror mode: routes commands over `net_as`, renders host output locally |
| `tui_screen.rs` | OpenFrame rendering of the PTY framebuffer |
| `keys.rs` | Key event routing — sends to PTY or command line field |

### Mirror mode

When `preferred_remote_route` is set in `thin-client.toml`, the shell panel operates in mirror mode: keystrokes go to the host via `shell.execute` routed with `net_as`, and the host's PTY output streams back. The shell panel on a thin client therefore renders a remote terminal session.

---

## ANSI stripping

`shell.rs` strips ANSI CSI escape sequences before returning output from `shell.execute` to non-PTY callers. The stripping is byte-level: `ESC [` … final byte (0x40–0x7E). This keeps LAN-routed and CLI output clean.

---

## Platform notes

| Platform | `shell.execute` | PTY/TUI |
|----------|-----------------|---------|
| macOS | `sh -c` | Full PTY |
| Linux | `sh -c` | Full PTY |
| Windows | `cmd /C` | Full PTY |
| iOS (local) | Returns error constant | N/A |
| iOS (thin-client) | Routes to host via LAN | Renders host output |

---

## Configuration

No dedicated config file. The terminal module reads `modules.toml` for its enabled state (like every module). The PTY shell inherits the process environment plus the overrides declared in `tui/env.rs`.

---

## Dependency chain

```
terminal-motd → terminal
```

Enabling `terminal-motd` transitively enables `terminal` if it isn't already on.
