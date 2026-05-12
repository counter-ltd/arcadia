# Roadmap and Known Gaps

`Documentation/GAPS.md` tracks all deliberate limitations. Summary with priority ranking:

## Completed — previously P0 / HIGH

| Item | Resolution |
|------|-----------|
| **Path traversal in `is_path_in_scope`** | `AiWorkspaceContext::is_path_in_scope` now canonicalizes both paths via `std::fs::canonicalize` before comparison. Raw `starts_with()` removed. |
| **Unrestricted `sh -c` in AI sandbox** | `sandboxed_exec` now enforces `EXEC_ALLOWLIST` — only development toolchain binaries permitted. |
| **API key in mpsc channel payload** | `ProviderRouting::OpenAi` no longer carries `api_key`. Inference thread loads `OpenAiConfig` directly at call time. |
| **No HTTP timeout on AI providers** | All `ureq` calls in `ai_runtime.rs` use `AgentBuilder::new().timeout(HTTP_TIMEOUT)`. |
| **Silent model-not-found error** | `ai_send_message` distinguishes "no model selected" from "model ID set but not found in registry" and surfaces both as user-visible error messages. |
| **Silent workspace config errors** | `.ok()` swallowing replaced with explicit `match` + `eprintln!` for config load failures. |
| **Empty default system prompt** | `AiConfig::default()` now ships a sensible built-in prompt; `#[serde(default)]` added so existing configs pick it up. |
| **Dead `_placeholder()` function** | Deleted from `ai_chat_panel.rs`. |
| **Hardcoded `rgb(0x...)` in AI chat panel** | Replaced with `theme::ui_accent_fg(cx)` and `p.content_meta`. |
| **llama.cpp model path not validated** | `llama_cpp_create_model_modal.rs` now checks `Path::new(&path).exists()` for model and mmproj files before saving. |
| **OllamaModelKind missing ImageGeneration** | Added `ImageGeneration` variant to `OllamaModelKind`; now matches `LlamaCppModelKind` and `OpenAiModelKind`. |
| **No Ollama model discovery** | "Discover Models" button in AI models panel calls `GET /api/tags` in a background thread (10s timeout); populates model list without blocking UI. |
| **OpenAI unusable without editing toml** | AI models panel now has editable API key (masked) + base URL fields; saves to `openai.toml` on Enter or Save. |

## P0 — Fix before trusting in production

| Gap | Problem | Direction |
|----|---------|-----------|
| **Revision coverage** | `surface.revision` only advances on `surface.patch`. CLI writes bypass it — clients can miss updates. | Bump revision from every `ModulesConfig::save`. |
| **Testing discipline** | No automated tests for snapshot round-trips, thin-client prefs, or LAN routing. | Add targeted `arcadia-core` unit + integration tests. |
| **iOS C ABI drift** | No CI check that `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h` matches the `extern "C"` exports in `Desktop/src/ios_lib.rs`. | Workflow step: compare ABI surface and fail on mismatch. |

## P0-Structural — Architecture debt (code compiles; correctness risk)

| Item | Problem | Direction |
|----|---------|-----------|
| **Page dispatch if-else chain** | `navigation.rs:254–492` and `root/render.rs:253–257` use 19+ hardcoded `if active_page_id == "..."` checks. Anti-pattern #3 in `AGENTS.md`; TODO acknowledged at `navigation.rs:280`. | Replace with a registry-driven dispatch map built from `PAGE_DEFINITIONS`. New pages must not require edits to the dispatch chain. |
| **`root/render.rs` monolith (1026 lines)** | Window setup, sidebar layout, modal stack, and context menu all in one file. | Split into `root/window_frame.rs`, `root/modal_stack.rs`, `root/content_area.rs`; `render.rs` becomes a thin compositor. |
| **`late.rs` monolith (1324 lines)** | Chat, playback, bonsai art, and activity feed mixed in one module file. | Split into `modules/late/mod.rs` + `chat.rs`, `playback.rs`, `bonsai.rs`, `activity.rs`. |
| **97 `rgb(0x...)` inline colors** | Remaining violations in `code_editor_panel.rs`, `navigation.rs`, `ai_settings_panel.rs`, `shortcuts_create_modal.rs`, `llama_cpp_create_model_modal.rs`, `workspace_create_modal.rs`, `appearance/` panels, `sidebar/nav_items.rs`, and others. | Add `theme/editor_palette.rs`; replace all inline colors with named theme tokens. `grep -r "rgb(0x" Desktop/src/gui/ --include="*.rs"` must return zero results. |
| **Production `expect()` in services + scheduler** | `services.rs:173,209` and `scheduling.rs:210,221,408` use `expect()` on service resolution and thread spawn. | Return `Result<>` from service resolution; handle thread spawn failure gracefully. |
| **UI `unwrap()`/`expect()` in production paths** | `shortcuts/mod.rs:175`, `shortcuts/os_hotkey.rs:152`, `shortcuts_create_modal.rs:159`, `llama_cpp_create_model_modal.rs:86`, `workspace_create_modal.rs:85`, `ai_settings_panel.rs:166`. (`llama_cpp_runtime.rs` deleted — those entries resolved.) | Replace with `Option`/`Result` + user-visible error state. |
| **`gui` feature couples Python + llama-cpp** | `--features gui` silently enables `python-extensions` and `llama-cpp`, bloating compile time for GUI-only users. | Decouple: `gui` = base GUI only; users compose `--features gui,python-extensions,llama-cpp`. |
| **`webpki-roots` dual-version** | `v0.26.11` and `v1.0.7` both in dep tree via `ureq` → old `rustls`. Risk: two TLS cert validation paths. | Bump `ureq` to `3.x` or force-resolve `webpki-roots` to one version. |

## P1 — Needed for real multi-user / multi-surface use

| Gap | Problem | Direction |
|----|---------|-----------|
| **Stale UI detection** | Desktop has `last_surface_revision` but never compares it — no "host changed under you" warning. | Compare revision on timer/focus/after routed command; optional banner + reload. |
| **Multi-writer** | Multiple GUIs on same host = last write wins, no merge, no locks. | Document as permanent constraint OR add optimistic concurrency (generation tokens on save). |
| **Transport** | Command routing is request/response UDP. No long-lived session, no ordering guarantees, no subscription for deltas. | Optional WebSocket/TCP sidecar for continuous thin-shell workflows. |

## P2 — Required before leaving trusted LAN

| Gap | Problem | Direction |
|----|---------|-----------|
| **Security posture** | No wire encryption, no auth beyond "approved node," no scoped capabilities. `shell.execute` routable to anyone approved. | Threat model doc + TLS or pairing secrets + capability tokens. |
| **Identity** | `client_id` is attribution only — no authz, no rate limits, no per-client filtering. | Host-side policy module or capability tokens if multi-tenant. |

## P1.5 — Feature completeness gaps found in audit

| Gap | Problem | Direction |
|----|---------|-----------|
| **`input.capture` not enforced** | Permission registered in catalog and toggeable via CLI/GUI, but never checked at `arcadia_ios_inject_touch` or desktop input injection sites. The toggle is cosmetic today. | Add enforcement gate at `ios_lib.rs::arcadia_ios_inject_touch` and the desktop equivalent before input reaches panels. |
| **`late/experimental_panel.rs` dead code** | Entire module has `#![allow(dead_code)]`; `_root` param unused; visualizer in `late/visualizer.rs` is a placeholder (literal `"· · · · · ·"` dots). | Either implement or remove; dead experimental modules rot and confuse contributors. |
| **Page dispatch refactor 50% done** | Was 19+ hardcoded `active_page_id` checks; now ~7 remain after partial migration. Anti-pattern acknowledged but incomplete. | Finish registry-driven dispatch — no `if active_page_id == "..."` checks anywhere. |
| **WAN placeholder** | CLI args error message says `"(wan: coming later)"` with no architecture or stubs. | Either remove the hint or add a design note in this doc; the current state is misleading. |
| **`webpki-roots` dual TLS version** | `v0.26.11` and `v1.0.7` both in dep tree — two cert validation code paths. | Bump `ureq` to `3.x` or pin `webpki-roots` to one version. |

## P3 — Polish and convergence

| Gap | Problem | Direction |
|----|---------|-----------|
| **iOS OpenFrame migration** | Initial OpenFrame-on-iOS surface is in place via `--features ios-gui` and a 2-function C ABI; OpenFrame iOS embedding, accessibility, and lifecycle polish are still ongoing. | Continue closing parity gaps inside `gui/app/entry_ios.rs` and OpenFrame's iOS platform layer. |
| **Surface parity** | Desktop has PTY/TUI paths; iOS is `shell.execute` only; not all panels are execute-only. | Converge per capability class with explicit "unavailable on this surface" from core. |
| **Renderer-only client** | Surfaces still bundle compiled nav — no enforced "remote-only" profile. | Optional build flag that refuses static nav when `remote_route` is mandatory. |
| **`extra` schema** | `extra.navigation_registry` is wired; broader extra buckets and corresponding `SurfacePatch` variants are undefined. | Define schema + version fields inside `extra`; extend `SurfacePatch` incrementally. |

---

## AI system gaps (incomplete features)

| Item | Status | Direction |
|------|--------|-----------|
| **Chat persistence** | In-memory only; history lost on app close | Serialize `AiChat` to `~/Arcadia/Configuration/ai-chats.toml` |
| **Provider selector UI** | `active_ai_provider_module` has no view to change it | Settings or top-bar picker |
| **Model loading progress** | No feedback during llama.cpp model load (can take 30s) | Loading spinner + status text in AI panel |
| **AI settings panel** | System prompt, max tokens, temperature have no UI | `ai_settings_panel.rs` (partially implemented) |
| **Multi-chat streaming** | Only one chat can stream at a time (`ai_stream_chat_id: Option<usize>`) | Per-chat `is_loading` state; multi-slot runtime queue |
| **Execute command confirmation** | `workspace.execute` grant allows AI to run allowlisted commands without per-invocation approval | Pre-flight confirmation dialog in GUI before `sandboxed_exec` |
| **GPU layer count** | `with_n_gpu_layers(99999)` hardcoded in `run_llama_cpp` | Read from `LlamaCppModel` config field |
| **Tool error visibility** | Tool call failures feed as text to model but no UI indicator | Tool result annotations in chat bubble |
| **CLI provider commands** | `modules/ai.rs`, `ollama.rs`, `openai.rs`, `llama_cpp.rs` all return empty `commands()`; no REPL surface for model list, config, or inference | Implement `ollama.list`, `openai.models`, etc. |
| **Provider config migrations** | `merge_defaults()` is a no-op stub in `config/ollama.rs`, `config/openai.rs`, `config/llama_cpp.rs` — no version migration path if fields change | Implement migration pattern following `config/modules.rs` |
| **OpenAI model management UI** | Models can be viewed from `openai.toml` but no add/remove UI | Add/remove model rows in AI models panel |
| **Ollama model kind after discovery** | Discovered models default to `TextGeneration`; no kind inference from Ollama model details | Use `details.families` from `/api/tags` response to infer kind |

## Security posture

Current trust model: **LAN pairing + locally approved peers.** Assume trusted network.

What this means in practice:
- Any approved LAN peer can execute any command the host has enabled, including `shell.execute`.
- `surface.patch` is unauthenticated beyond `client_id` (which is just a UUID, not a secret).
- There is no encryption on the wire.

**AI sandbox (implemented):**
- All AI file/exec operations route through `ai_sandbox.rs` — no direct `std::fs` or `Command` calls from providers.
- `sandboxed_exec` enforces `EXEC_ALLOWLIST`; only named development toolchain binaries (cargo, npm, python, git, …) are permitted regardless of workspace permissions.
- `is_path_in_scope` uses `fs::canonicalize` on both paths — `../` traversal attacks blocked.
- API keys are loaded in the inference thread at call time — never travel through mpsc channels or live in `enum` variants.

**Do not expose Arcadia to untrusted networks without addressing P2 gaps above.** This is a home-network / trusted-LAN tool today. Production-grade multi-tenant use requires TLS, capability tokens, and a real threat model document first.

---

## CI

`.github/workflows/` — `stable-build-matrix.yml` builds Desktop targets and iOS simulator configs on selected branches. See individual workflow files for triggers and matrix.

Gaps in CI coverage: iOS C ABI drift detection, core integration tests. See [CONTRIBUTING.md](CONTRIBUTING.md).
