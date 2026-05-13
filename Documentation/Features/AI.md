# AI

Module name: `ai` (base)  
Provider modules: `ai-provider-ollama`, `ai-provider-llama-cpp`, `ai-provider-openai`  
Platform: desktop (inference threads); iOS routes to a desktop host

---

## Overview

Arcadia's AI system is a modular inference layer. The base `ai` module defines the provider registry and shared types; each provider is an independent module that must be enabled alongside `ai`. The chat panel and runtime live in the GUI layer and rely on the workspace system for file/command access.

```
ai_chat_panel.rs   — UI, @mention parsing, request assembly
ai_runtime.rs      — inference thread; dispatches to provider fns
ai_settings_panel.rs / ai_models_panel.rs — configuration UI
ai_sandbox.rs      — SINGLE gate for all file/exec ops from AI
ai_context.rs      — @mention parsing, file context injection
ai_tools.rs        — tool definitions + execution dispatch
ai_types.rs        — shared types (requests, responses, workspace context)
```

---

## Providers

### Provider registry

`AI_PROVIDER_REGISTRY` in `modules/ai.rs` declares the three built-in providers. The GUI reads this to populate the provider selector; no surface edits needed when adding a new provider.

| Module | Display name | Description |
|--------|-------------|-------------|
| `ai-provider-llama-cpp` | llama.cpp | Local inference — runs `.gguf` models directly on-device |
| `ai-provider-ollama` | Ollama | Local inference via Ollama HTTP API (`http://localhost:11434`) |
| `ai-provider-openai` | OpenAI | Cloud inference via OpenAI-compatible HTTP API |

### Ollama (`ollama.toml`)

| Field | Default | Purpose |
|-------|---------|---------|
| `endpoint` | `http://localhost:11434` | Ollama API base URL |
| `models` | `[]` | Registered models: `id`, `name`, `model_tag`, `model_kind` |

Model kinds: `text_generation`, `image_generation`, `vision`, `embedding`.

The settings panel includes an auto-discovery button that queries `GET /api/tags` on the configured endpoint and populates the model list.

### llama.cpp (`llama-cpp.toml`)

| Field | Purpose |
|-------|---------|
| `models[].id` | Stable internal identifier |
| `models[].name` | Display name |
| `models[].model_type` | `text_generation` / `image_generation` / `vision` / `embedding` |
| `models[].path` | Absolute path to `.gguf` model file |
| `models[].mmproj_path` | Optional multi-modal projector path (vision models) |

### OpenAI (`openai.toml`)

| Field | Default | Purpose |
|-------|---------|---------|
| `api_key` | `""` | API key (plaintext; future: OS keychain) |
| `base_url` | `https://api.openai.com` | API base — override for compatible endpoints |
| `models` | `[]` | Registered models: `id`, `name`, `model_id`, `model_kind` |

**Security note:** API key is stored in `openai.toml` in the config root. It is never put in an enum variant, struct field crossing a thread boundary, or message payload. The inference thread loads it directly from config at call time via `OpenAiConfig::load_or_create()`.

### General AI config (`ai.toml`)

| Field | Default | Purpose |
|-------|---------|---------|
| `default_system_prompt` | See below | System prompt prepended to every request |
| `max_tokens` | `512` | Token ceiling for text generation responses |

Default system prompt: *"You are a helpful assistant integrated into Arcadia. Be concise and accurate. When working with files, prefer showing diffs over repeating entire file contents."*

---

## Request types

All request types live in `ai_types.rs`:

| Type | Fields |
|------|--------|
| `TextGenerationRequest` | `system`, `messages: Vec<(role, content)>`, `max_tokens`, `workspace_context`, `tools` |
| `ImageGenerationRequest` | `prompt`, `negative_prompt`, `width`, `height`, `workspace_context` |
| `EmbeddingRequest` | `input`, `workspace_context` |
| `VisionRequest` | `system`, `messages`, `image_data: Vec<u8>`, `workspace_context` |

---

## Workspace tools

When a workspace is active in the chat panel, four tools are injected into the system prompt:

| Tool | Permission required | Description |
|------|---------------------|-------------|
| `read_file` | `workspace.read` | Read a file by path (relative to workspace root or absolute) |
| `write_file` | `workspace.write` | Write/create a file |
| `list_files` | `workspace.read` | List directory contents |
| `run_command` | `workspace.execute` | Run a shell command scoped to the workspace root |

Tool call format in model output: a fenced ` ```json ` block containing `{"tool_calls":[{"name":"…","arguments":{…}}]}`.

---

## AI sandbox

All file and exec operations from the AI layer go through `ai_sandbox.rs`. No provider or tool handler calls `std::fs` or `Command` directly.

### Sandbox operations

| Fn | Checks |
|----|--------|
| `sandboxed_read(ctx, path)` | Path in workspace scope + `workspace.read` granted |
| `sandboxed_write(ctx, path, content)` | Path in scope + `workspace.write` granted |
| `sandboxed_list(ctx, path)` | Path in scope + `workspace.read` granted |
| `sandboxed_exec(ctx, cmd)` | `workspace.execute` granted + binary in `EXEC_ALLOWLIST` |

### Path scope check

`is_path_in_scope` canonicalizes both the workspace root and the target path before comparing with `starts_with`. For paths that don't exist yet (new file writes), the parent directory is canonicalized and the filename appended — prevents traversal attacks via `../` sequences.

### Exec allowlist

Only the leading binary name is checked. Allowed binaries (defined in `EXEC_ALLOWLIST`):

```
cargo rustc rustfmt clippy-driver
npm npx node yarn pnpm
python python3 pip pip3 uv
git gh
make cmake ninja
ls find grep rg cat head tail wc echo printf mkdir cp mv
swift swiftc
go
java javac mvn gradle
ruby gem bundle
```

Shells (`sh`, `bash`, `zsh`) and destructive tools (`rm`, `curl`, `dd`) are intentionally excluded. Commands are passed to `sh -c` after the allowlist check, so chaining (`&&`, `;`, pipes) is permitted within the allowed binary constraint.

---

## HTTP timeouts

Every outbound HTTP call uses `ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build()` with `HTTP_TIMEOUT = 120s`. Direct `ureq::post()` without an agent is not permitted in production inference paths.

---

## @mention context injection

`ai_context.rs` parses `@file/path` mentions in chat input and injects file contents into the request before dispatching to the inference thread. Injected files are subject to the same sandbox scope check as `read_file` tool calls.

---

## Module dependency

The `ai` module has no declared `required_modules`. Each provider module (`ai-provider-*`) declares `ai` as a required module — enabling a provider transitively enables the base `ai` module.

---

## Adding a new provider

1. Add a `ModuleManifest` to `MODULE_REGISTRY` in `config/modules.rs` with `required_modules: &[AI_MODULE_NAME]`.
2. Add an `AiProviderManifest` to `AI_PROVIDER_REGISTRY` in `modules/ai.rs`.
3. Create a provider module file (e.g. `modules/myprovider.rs`) implementing inference fns.
4. Add the binary name(s) your provider shells out to into `EXEC_ALLOWLIST` in `ai_sandbox.rs` if needed.
5. Use `ureq::AgentBuilder` with `HTTP_TIMEOUT` for all HTTP calls.
6. Load credentials in the inference thread from config at call time — never in `ProviderRouting` or any struct crossing a thread boundary.
