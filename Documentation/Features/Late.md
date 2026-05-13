# Late.sh

Module name: `late`  
Config file: `late.toml`  
Platform: all  
Pages: `late.now_playing`, `late.experimental`, `late.settings`

---

## Overview

The `late` module is a native client for the Late.sh social platform. It connects via WebSocket and provides:

- **Chat rooms** — real-time messages with reactions
- **Now Playing** — music stream with track, artist, album, progress, volume
- **Music votes** — vote for next genre (lofi, ambient, classic, jazz)
- **Visualizer** — ASCII/Unicode audio visualizer frame (updated from server)
- **Bonsai** — ASCII bonsai art updated from the server
- **Activity feed** — join/leave events across rooms
- **Experimental panel** — profile, notifications, RSS, articles, showcase, games, artboard, work profiles, DMs, chips

---

## Domain types

### `LateMessage`

| Field | Type | Purpose |
|-------|------|---------|
| `id` | `String` | Unique message identifier |
| `user_id` | `String` | Sender user ID |
| `username` | `String` | Sender display name |
| `body` | `String` | Message text |
| `timestamp` | `String` | ISO-8601 timestamp |
| `reactions` | `Vec<LateReaction>` | Emoji reaction counts |

### `LateReaction`

| Field | Purpose |
|-------|---------|
| `emoji` | Emoji character |
| `count` | Reaction count |

### `LateNowPlaying`

| Field | Purpose |
|-------|---------|
| `track` | Track title |
| `artist` | Artist name |
| `album` | Album name |
| `progress_sec` | Current playback position (seconds) |
| `duration_sec` | Total track duration (seconds) |
| `volume_pct` | Volume (0–100) |

### `LateVotes`

| Field | Purpose |
|-------|---------|
| `lofi` | Vote count for lofi genre |
| `ambient` | Vote count for ambient genre |
| `classic` | Vote count for classical genre |
| `jazz` | Vote count for jazz genre |
| `next_vote_at` | Timestamp when the next vote window opens |

### `LateUser`

| Field | Purpose |
|-------|---------|
| `user_id` | Numeric user ID |
| `username` | Display name |
| `room_id` | Current room (optional) |

### `LateActivityEvent`

| Field | Purpose |
|-------|---------|
| `kind` | Event type string (e.g. `"join"`, `"leave"`) |
| `username` | Username who triggered the event |
| `room_id` | Room ID where the event occurred |
| `timestamp` | ISO-8601 timestamp |

---

## Shared state (`LateState`)

The in-memory state is a mutex-guarded `LateState`:

| Field | Bounded to |
|-------|-----------|
| `connected` | WebSocket connection status |
| `active_room` | Current room ID (u32) |
| `messages` | Last 200 messages (bounded `VecDeque`) |
| `online_users` | All users visible to the current session |
| `now_playing` | Current track info |
| `votes` | Current vote counts |
| `visualizer_frame` | Current ASCII visualizer frame string |
| `bonsai_art` | Multi-line bonsai ASCII art |
| `activity_feed` | Bounded `VecDeque` of recent join/leave events |
| `connection_error` | Last connection error string (shown in UI) |
| `state_version` | `u64` counter — incremented on every mutation so the GUI poll task knows when to repaint |

---

## Config (`late.toml`)

Managed by `LateConfig`. Contains WebSocket server URL, credentials, and connection preferences. The Late.sh settings page (`late.settings`) surfaces these fields for user configuration.

---

## GUI panels

Files: `Desktop/src/gui/app/late/`

| File | Purpose |
|------|---------|
| `panel.rs` | Root Late.sh page — chat pane, now playing, visualizer, votes |
| `chat_pane.rs` | Message list with reactions, scroll, message input |
| `sidebar.rs` | Room/user sidebar |
| `top_bar.rs` | Connection status, room selector |
| `visualizer.rs` | ASCII visualizer renderer |
| `vote_panel.rs` | Genre vote UI — four buttons with counts |
| `bonsai.rs` | Bonsai ASCII art panel |
| `experimental_panel.rs` | Experimental features: profile, notifications, RSS, articles, showcase, games, artboard, work profiles, DMs, chips |
| `settings.rs` | Late.sh settings page — server URL, credentials |
| `state_bridge.rs` | Bridge between the WebSocket state and GUI render cycles |
| `mod.rs` | Module exports |

---

## State bridge

`state_bridge.rs` connects the module's in-memory `LateState` to the GUI render cycle. The desktop GUI polls `state_version` on a timer; when the version changes, it copies the relevant state fields into its own render-local state and triggers a repaint. This avoids holding the `LateState` mutex during rendering.

---

## Dependency chain

The `late` module has no `required_modules`. It is self-contained.

---

## Pages

| Page | Notes |
|------|-------|
| `late.now_playing` | Primary panel: chat + now playing + visualizer + bonsai + votes |
| `late.experimental` | Experimental feature surface |
| `late.settings` | Settings hub child — server URL, credentials, connection prefs |

All three pages require `late` module to be enabled.
