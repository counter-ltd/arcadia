# AI Roadmap — Arcadia

## Mission

Best open-source AI platform — fully inclusive for all providers (cloud + local). No vendor
lock-in. Every feature works offline with local models. Every feature works equally well
with OpenAI, Anthropic, Ollama, llama.cpp, and any future provider.

---

## Platform Intelligence — Best Features to Absorb

### Cursor
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Inline diff preview** — AI proposes changes as visual diff; approve/reject per-hunk | ★★★★★ | #3 Inline Diff |
| **Tab / next-edit prediction** — predicts next edit location + content from recent changes | ★★★★ | future (post-indexer) |
| **Background agents** — fire task, keep working, notified on done | ★★★★★ | #8 Background Agents |
| **Shadow workspace** — test edits in parallel before showing user | ★★★★ | #3 (checkpoint-based) |
| `.cursorrules` — project-specific rule file | ★★★★ | #1 Rules & Skills |

### Windsurf / Cascade
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Flow mode** — autonomous "cascade" vs conversational chat; AI acts until done | ★★★★★ | #4 Flow Mode |
| **Deep repo awareness** — understands entire codebase, not just files you point at | ★★★★★ | #5 Workspace Indexer |
| **Write vs Chat** — explicit mode switch: collaborate vs execute | ★★★★ | #4 Flow Mode |

### Claude / Claude Code
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Extended thinking** — expose chain-of-thought reasoning in UI | ★★★★★ | #9 Thinking Mode |
| **Subagent spawning** — delegate to specialist agents | ★★★★★ | #11 Orchestration |
| **Projects + persistent memory** — cross-session context | ★★★★★ | #2 + #13 Memory |
| **Artifacts** — rendered live output (HTML, code, diagrams) | ★★★★ | #14 Artifacts |

### Aider
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Repo map** — compact AST-based codebase summary (~2k tokens); always in context | ★★★★★ | #5 Indexer (Level 2) |
| **Architect + Implementer** — planning agent hands off to coding agent | ★★★★★ | #11 Orchestration roles |
| **Git auto-commit** — AI commits its own changes with meaningful messages | ★★★★ | #7 Git Integration |

### Cline / Roo
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Checkpoints / rollback** — git snapshot before AI batch edits; one-click restore | ★★★★★ | #6 Checkpoints |
| **Plan + Act phases** — explicit plan shown to user, approval required before execution | ★★★★★ | #4 Flow Mode (gate) |
| **Task history** — saved records of past autonomous tasks | ★★★ | #2 Chat persistence |
| **Browser use** — navigate web, read docs live | ★★★★ | #12 Context Providers |

### Continue.dev
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Context providers** — pluggable @-sources: web, docs URL, git log, PR, issues | ★★★★★ | #12 Context Providers |
| **Slash commands** — custom prompt macros | ★★★★ | #1 Skills |

### GitHub Copilot / Codex
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **PR summaries** — auto-generate PR descriptions from diff | ★★★★ | #7 Git Integration |
| **Code review** — inline AI review comments on diff | ★★★★ | #7 Git Integration |
| **CLI exec mode** — run inference via installed CLI binary | ★★★★★ | #26 CLI Exec Providers |

### Devin / SWE-agent
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Long-horizon task planning** — break issue into milestones, track progress | ★★★★★ | #11 Orchestration |
| **Issue → PR pipeline** — read GitHub issue, implement, open PR | ★★★★ | #7 Git Integration |

### Claude Desktop / MCP ecosystem
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **MCP client** — universal tool bridge to any MCP server | ★★★★★ | #111 MCP Client |
| **MCP marketplace + auto-discovery** — one-click install of community servers | ★★★★ | #113 MCP Marketplace |
| **Zero-config first run** — works the moment you install (with Claude API key) | ★★★★★ | #126 Zero-Config (offline local model — no API key needed) |

### Perplexity / Claude Deep Research / OpenAI Deep Research
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Iterative deep research** — plan → search → read → refine over N steps | ★★★★★ | #115 Deep Research Engine |
| **Citation tracking + source trust** — every claim linked to source; quality scored | ★★★★★ | #117 Source Quality & Citation Graph |
| **Multi-backend search** — Brave, Bing, Google, etc. | ★★★★ | #116 Pluggable Search Backends |

### OpenAI o1 / DeepSeek R1 / Claude Extended Thinking
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Visible chain-of-thought** — reasoning tokens shown to user | ★★★★ | #9 Extended Thinking Display |
| **Test-time compute scaling** — harder problems get more thinking | ★★★★★ | #122 Test-Time Compute Scaling |
| **Tree of Thoughts** — branching reasoning with backtracking | ★★★★★ | #120 Tree of Thoughts Engine |
| **Self-consistency voting** — sample N paths; majority answer | ★★★★ | #121 Self-Consistency Voting |

### VS Code / Raycast
| Feature | Quality | Arcadia target |
|---------|---------|----------------|
| **Universal command palette** — Cmd+K fuzzy search across everything | ★★★★★ | #129 Universal Command Palette |

---

## Arcadia-Unique Advantages

No other platform has all of these simultaneously.

| Advantage | Why legendary |
|-----------|--------------|
| **Fully offline** | Indexer + embeddings + inference all local. Cursor/Windsurf require cloud. |
| **Zero-config first run** | Bundled local model; AI works the instant you install (#126). Beats every competitor on first-run UX. |
| **LAN agent routing** | Run agent on one machine, control from another over LAN. Nobody has this. |
| **iOS + Desktop same codebase** | AI agent on iPhone. Unprecedented. |
| **Apple silicon native** | First-class ANE + Core ML + MLX provider (#86). No other Rust AI shell uses Apple's NPU. |
| **Universal MCP** | Both client AND server (#111 + #112). Every other tool is MCP-client-only. Arcadia is the universal AI infra layer. |
| **Module composability** | AI features as auditable, dep-tracked modules. Not extensions — first-class. |
| **Open sandbox** | Every tool call inspectable. Audit log. Community trust. |
| **Hardware-rooted provenance** | Secure Enclave / TPM signs every action (#87). Tamper-evident chain. Regulated-industry-ready. |
| **Any provider** | Local llama.cpp, Ollama, OpenAI, Anthropic, self-hosted — same UX. Plus Open Provider Protocol (#106) so future providers integrate via spec. |
| **CLI exec providers** | Use subscription accounts (Claude Pro, ChatGPT Plus) — no API key billing. |
| **AST-native VCS** | Semantic VCS layer on top of git (#100); merges happen at AST level, not lines. |
| **Differential privacy on cloud prompts** | ε-DP perturbation (#90); cloud quality + provable local privacy. |
| **Closed loop self-improvement** | Models learn from their own errors; local LoRA fine-tuning pipeline; federated across peers (#105). |
| **AI-driven meta-orchestration** | AI can extend its own tools, skills, and agent graphs. |
| **Python-first extensibility** | Full AI SDK for Python extensions; extend every layer from Python. |
| **Deep research, fully offline-capable** | Iterative search → read → synthesise (#115) with SearXNG self-hosted backend (#116) means no third-party search API required. |
| **Reasoning that compounds** | Reasoning Cache (#124) makes every solved hard problem a re-usable template for the next. |

---

## Build Order

### Tier 1 — Foundation

**1. Rules & Skills System** ✅ *Shipped*
- `AiRule` — constraint per request: forbidden tools, max_tokens cap, required format, persona
- **Arcadia default rules** — shipped in-binary: `safe-exec`, `concise-output`, `diff-over-full-file`
- **Per-chat rule UI** — chip strip in chat header; pick presets inline; active rules shown as chips
- `AiSkill` — named behaviour: system prompt fragment + allowed tools + param overrides
  (e.g. `code-reviewer`, `summariser`, `shell-assistant`, `security-auditor`)
- Skills compose — fragments merged in declaration order
- Slash command macros map to skills
- New files: `Shared/ArcadiaCore/src/config/ai_rules.rs`, `config/ai_skills.rs`
- Modified: `ai_runtime.rs` (`prepare_system`), `ai_types.rs`

---

**2. Persistent Chat + Context Window Management** ✅ *Shipped*
- Persist to `~/Arcadia/Configuration/ai-chats/` (newline-delimited JSON)
- `ChatSession`: id, title (auto from first message), provider, model, messages, timestamps,
  workspace_id, active_rules, active_skills
- Token counter — tiktoken-compatible approximation, works offline
- Auto-summarise / sliding-window truncate at 80% context limit
- Chat sidebar: list, new, rename, delete, search
- Model switching mid-chat (new session segment, history preserved)
- New file: `Shared/ArcadiaCore/src/modules/ai_chat_store.rs`
- Modified: `Desktop/src/gui/app/ai_chat_panel.rs`, `ai_types.rs`

---

### Tier 2 — Intelligence Layer

**3. Inline Diff Preview** *(Cursor — their #1 feature)* ✅ *Shipped*
- AI proposes file edits as side-by-side or inline diff, not raw text
- Per-hunk approve / reject / edit; "Accept all" / "Reject all"
- AI writes to staging buffer; only commits to disk on user accept
- Works for single-file and multi-file change sets
- New file: `Desktop/src/gui/app/ai_diff_panel.rs`
- Modified: `ai_tools.rs` (write_file stages, not commits), `ai_chat_panel.rs`

---

**4. Flow Mode** *(Windsurf Cascade + Cline Plan/Act)*
- Two explicit modes per chat:
  - **Chat mode** — collaborative; AI asks before each significant action (current default)
  - **Flow mode** — autonomous; AI executes multi-step plan until done or blocked
- Flow mode shows plan before execution; user can approve or modify each step
- `FlowState`: planned steps, completed steps, failed steps, current_step, can_pause, can_abort
- Pause/abort controls always visible while flow runs
- New files: `Shared/ArcadiaCore/src/modules/ai_flow.rs`, `Desktop/src/gui/app/ai_flow_panel.rs`
- Modified: `ai_runtime.rs` (flow execution loop), `ai_chat_panel.rs`

---

**5. Workspace Indexer** *(Aider repo map + Windsurf deep awareness)*

Three levels — each degrades gracefully:

- **Level 1 — structural**: file tree, sizes, language detection. Zero deps. Always runs.
- **Level 2 — symbolic**: function/class/type names via tree-sitter. Offline. Produces repo map
  (compact ~2k token codebase summary always injected into context).
- **Level 3 — semantic**: embedding vectors per 512-token chunk with 64-token overlap.
  Uses any enabled embedding model (Ollama `nomic-embed-text`, OpenAI embeddings, llama.cpp).
  Optional — falls back to keyword search.

- File watcher via `notify` crate for incremental re-index on change
- `workspace.index` commands: `index.status`, `index.rebuild`, `index.search`
- Index persisted to `~/Arcadia/Configuration/ai-index/<workspace_id>/`
- New files: `Shared/ArcadiaCore/src/modules/ai_index.rs`, `modules/ai_index_watcher.rs`
- Modified: `config/modules.rs` (registry entry), `modules/mod.rs` (dispatch)

---

**6. Checkpoints / Rollback** *(Cline — safety net for autonomous AI)*
- Before any AI batch write/exec: snapshot workspace state via git stash or shadow branch commit
- `CheckpointStore`: list with timestamp, triggering agent, files affected
- One-click restore from chat panel or dedicated checkpoints list
- Auto-checkpoint before Flow mode starts
- New file: `Shared/ArcadiaCore/src/modules/ai_checkpoint.rs`
- Modified: `ai_tools.rs` (checkpoint before write), `ai_sandbox.rs`

---

**7. Git Module → AI Git Integration** *(Aider auto-commit + Copilot PR tools)*

> **Prerequisite:** Git module (#18) must ship first. AI git features build on top of it.

- AI suggests commit message for its own changes (shown as editable chip, user approves)
- `git.diff`, `git.log`, `git.blame`, `git.status` as first-class AI tools
- PR summary generation: `git diff main..HEAD` → AI generates PR description
- Inline code review: feed diff → AI annotates hunks with review comments
- New file: `Shared/ArcadiaCore/src/modules/ai_git.rs`
- Modified: `ai_tools.rs` (git tools in registry), `ai_sandbox.rs` (`git` already allowlisted)

---

**8. Background Agents** *(Cursor — non-blocking long tasks)*
- Fire a task from chat → notification badge when done; keep working in the meantime
- Multiple agents run concurrently; each has its own stream + checkpoint
- Agent status panel: list of running / done / failed background agents
- Results reviewable after the fact; diff view for file changes
- New files: `Shared/ArcadiaCore/src/modules/ai_agent_pool.rs`,
  `Desktop/src/gui/app/ai_agent_status_panel.rs`
- Modified: `ai_runtime.rs` (multi-agent handles), `ai_chat_panel.rs`

---

### Tier 3 — Context & Memory

**9. Extended Thinking Display** *(Claude)*
- Providers that support chain-of-thought (Claude Sonnet/Opus, DeepSeek R1, QwQ,
  local reasoning models): expose `<thinking>` blocks in UI
- Collapsed by default; expandable per message
- Reasoning tokens counted separately from response tokens in session stats
- Modified: `ai_runtime.rs` (parse thinking blocks), `ai_chat_panel.rs` (collapsible render)

---

**10. Advanced Context System** *(Windsurf + Continue.dev)*
- Proactive injection at chat start: workspace structural summary, active languages, recent git changes
- Semantic relevance scoring: on each message, query index for top-K related files/symbols;
  inject above threshold
- Symbol-aware @mentions: `@FnName` resolves via index to definition + callers
- Change-aware context: modified files since last message surfaced as fragment
- `AiContextStrategy` enum: `Minimal` | `Structural` | `Semantic` | `Full` — per-chat or skill default
- New file: `Shared/ArcadiaCore/src/modules/ai_context_strategy.rs`
- Modified: `ai_context.rs`, `ai_runtime.rs`, `ai_chat_panel.rs`

---

**11. Multi-Agent Orchestration** *(Devin + Aider Architect/Implementer + Claude subagents)*
- `AgentDefinition`: name, skill_ids, provider preference, model preference,
  input/output schema, token budget
- `OrchestrationGraph`: DAG; typed edges; nodes run concurrently where dependencies allow
- Built-in role agents: **Architect** (plans/decomposes), **Implementer** (writes code),
  **Reviewer** (audits), **Tester** (runs + interprets tests), **Summariser** (compresses)
- Orchestrator agent produces graph JSON from user goal; runtime executes it
- GUI: live mindmap — nodes (name, status, token spend), edges (data flow), colour by state
- New file: `Shared/ArcadiaCore/src/modules/ai_orchestrator.rs`, GUI graph view
- Modified: `ai_runtime.rs`, `ai_types.rs`

---

**12. Context Providers** *(Continue.dev pluggable @-sources)*
- Pluggable `@`-mention sources beyond local files:
  - `@web:url` — fetch + clean page content
  - `@docs:name` — indexed documentation pack (offline or fetched once)
  - `@git:log` — recent commit history
  - `@issue:N` — GitHub/GitLab issue body + comments
  - `@pr:N` — PR diff + review comments
  - `@search:query` — web search results via configured search provider
  - `@all:query` — cross-workspace semantic search (see #25)
- Each provider is a registered handler; Python extensions can register custom providers
- New file: `Shared/ArcadiaCore/src/modules/ai_context_providers.rs`
- Modified: `ai_context.rs` (@mention parser extended)

---

**13. Advanced Memory System** *(Claude Projects)*
- `MemoryStore`: per-workspace + global scope
- Entry types: `fact`, `preference`, `code-snippet`, `summary`, `decision`
- Write paths:
  - Model emits `<memory type="fact">…</memory>` tagged block
  - Explicit `memory.write` tool call
  - User pins any chat message as memory from UI
- Read: session start → top-K retrieved by embedding similarity (uses indexer infra);
  injected as `### Relevant memories` block in system prompt
- Degrades to keyword search without embedding model
- New file: `Shared/ArcadiaCore/src/modules/ai_memory.rs`
- Modified: `ai_runtime.rs` (injection), `ai_tools.rs` (memory.write tool)

---

**14. Artifacts / Rendered Output** *(Claude artifacts)*
- Detect output type from fenced blocks:
  - `html` → render live in panel
  - `mermaid` / `dot` → render diagram
  - `markdown` → formatted display
  - code with runnable language → syntax highlight + run button
- Artifact panel alongside chat; persistent artifacts saved per-session
- New file: `Desktop/src/gui/app/ai_artifact_panel.rs`
- Modified: `ai_chat_panel.rs` (detect and route artifact blocks)

---

### Tier 4 — Self-Improvement & Extensibility

**15. Closed Loop Feedback + Self-Improvement** *(original)*

Models learn from their own errors at inference time. Feedback flows back into rules, skills,
and memory. For local models, data feeds a fine-tuning pipeline.

> **Architecture prerequisite:** `InternalCommandBus` trait — modules invoking other modules.

Inference-time feedback:
- After each tool call: tag result `Success | PartialSuccess | Failure` with reason
- After each chat turn: optional thumbs-up/down + implicit signals
- **Reflection pass**: on tool failure, lightweight reflect agent diagnoses error → corrective
  memory entry (e.g. "this command fails on macOS — use X instead")
- `OutcomeRecord`: session_id, agent_name, action, result_tag, correction, timestamp

Rule refinement:
- Rule health indicators in UI: green/amber/red based on outcome correlation
- AI proposes rule modifications based on failure patterns (user approves)

Fine-tuning pipeline (local models only):
- Collect `(system_prompt, messages, correction)` tuples to `~/Arcadia/Configuration/ai-feedback/`
- Export as GGUF-compatible fine-tune dataset for llama.cpp LoRA adapter training
- Per-workspace opt-in; user controls collection, export, deletion

New files: `Shared/ArcadiaCore/src/modules/ai_feedback.rs`, `modules/ai_reflect.rs`
Modified: `ai_runtime.rs`, `ai_tools.rs`, `modules/mod.rs` (InternalCommandBus)

---

**16. Meta-AI — AI Extends Itself** *(original)*

AI creates new tools, skills, rules, and agent definitions at runtime under user supervision.

> **Architecture prerequisite:** `meta_accessible: bool` flag on `ModuleCommand`; `InternalCommandBus`.

- `arcadia.meta_execute` tool — AI calls only `meta_accessible` commands via sandboxed wrapper
- Accessible commands: `ai_skills.*`, `ai_rules.*`, `ai_tool_registry.*`, `ai_orchestrator.*`,
  `workspace.list`
- **Every meta-AI action requires explicit user approval before write — never silent**
- All meta actions logged with `meta_ai` source tag; reversible via checkpoints

Self-extension flows:
- **Skill minting**: AI identifies repeated prompt pattern → proposes skill → user approves
- **Tool generation**: AI writes shell-script tool handler → registers via tool registry
- **Agent specialisation**: orchestrator detects gap → proposes new specialist → user approves
- **Rule derivation**: closed-loop feedback pattern → AI drafts corrective rule → user approves

New files: `Shared/ArcadiaCore/src/modules/ai_meta.rs`,
`Desktop/src/gui/app/ai_meta_approval_panel.rs`
Modified: `modules/mod.rs`, `ai_tools.rs`, `ai_sandbox.rs`

---

**17. Python AI SDK** *(original)*

Full Python access to every AI layer. Python extensions can register tools, skills, agents,
context providers, and inference pipeline hooks.

> **Architecture prerequisite:** `arcadia.execute(token, args)` Python-callable bridge.
> Currently `python_host.rs` does NOT expose `execute_command` to Python.

```python
# Call any Arcadia command
result = arcadia.execute("workspace.list", [])

# Register an AI tool
@arcadia.ai_tool(name="my_tool", description="...", schema={...})
def handle_my_tool(args, workspace_ctx):
    return "result"

# Register a context provider
@arcadia.context_provider(mention_prefix="mydata")
def provide_context(query, workspace_ctx):
    return "context string injected into system prompt"

# Register a skill
arcadia.register_skill({
    "name": "my-skill",
    "system_fragment": "You are a specialist in ...",
    "allowed_tools": ["read_file", "my_tool"],
})

# Register an agent definition
arcadia.register_agent({
    "name": "my-specialist",
    "skill_ids": ["my-skill"],
    "provider_preference": "ollama",
})

# Hook into inference pipeline
@arcadia.inference_hook(stage="pre_request")
def add_context(request):
    request.system += "\nExtra context from Python extension."
    return request

@arcadia.inference_hook(stage="post_response")
def log_response(response, outcome):
    arcadia.memory_write("fact", f"Last response summary: {response[:100]}")

# Memory access
arcadia.memory_write("preference", "user prefers compact output")
results = arcadia.memory_read("output format preferences")

# Closed loop feedback
arcadia.feedback_record(outcome="success", correction=None)
```

Sandboxing: all Python AI calls respect existing workspace permissions. Inference hooks
must complete within 2s or are skipped with a warning logged. Hooks cannot exfiltrate data.

New file: `Shared/ArcadiaCore/src/modules/ai_python_bridge.rs`
Modified: `python_host.rs`, `python_registry.rs`, `ai_tools.rs`, `ai_context.rs`,
`ai_runtime.rs`

---

### Tier 4b — Prerequisite Infrastructure

**18. Git Module** *(prerequisite for #7 and all AI git features)*

Proper Arcadia module with workspace integration before any AI git tooling.

- `GIT_MODULE_NAME: &str = "git"` in `MODULE_REGISTRY`; `workspace` as required dep
- Commands (all workspace-scoped): `git.status`, `git.diff`, `git.log`, `git.commit`,
  `git.branch`, `git.stash`, `git.show`, `git.blame`
- `GitContext`: auto-detected from workspace path via `git rev-parse --show-toplevel`
- GUI: git panel in workspace context — status, staged/unstaged diff, branch switcher, log view
- New files: `Shared/ArcadiaCore/src/modules/git.rs`, `Desktop/src/gui/app/git_panel.rs`
- Modified: `config/modules.rs`, `modules/mod.rs`

---

### Tier 5 — Accessibility, Education & UX

**19. Quick Chat Popup** *(Raycast AI / Spotlight-style)*
- Global OS hotkey (e.g. `Cmd+Shift+Space`) via existing `shortcuts/os_hotkey.rs`
- Floating compact panel: input box + last N messages; inherits active workspace context
- Active rules/skills shown as chips; quick-toggle inline
- Long tasks sent to background agent; notification badge on done
- "Expand to full chat" moves session into main panel
- New file: `Desktop/src/gui/app/ai_quick_chat.rs`
- Modified: `shortcuts/os_hotkey.rs`, `root/render.rs`

---

**20. Project Knowledge Base** *(Claude Projects — per-project persistent AI context)*

Each workspace gets a curated, intentionally-authored knowledge base that persists across
all chats. Distinct from memory (auto-accumulated) — this is explicit, structured, versioned.

- `ProjectKnowledge` entries: architecture notes, coding standards, API docs, project goals,
  team decisions, extension development guides
- Authored by user or AI (AI proposes, user approves); entries versioned
- Always injected at session start before any other context
- `@project:section` mention pulls specific section inline
- Arcadia ships a built-in `arcadia-sdk` knowledge entry per workspace — AI already knows
  how to write Arcadia extensions out of the box
- GUI: knowledge base editor in workspace settings
- New files: `Shared/ArcadiaCore/src/modules/ai_project_knowledge.rs`,
  `Desktop/src/gui/app/ai_knowledge_panel.rs`
- Modified: `ai_runtime.rs` (session start injection), `ai_context.rs` (@project: mention)

---

**21. AI Observatory + Advanced Visualization** *(original — education + transparency)*

Unified real-time visualization of ALL AI activity. Every decision is visible.
Educational tooltips on every panel. Trust through transparency.

**Context Inspector** — per-message breakdown:
- Full system prompt (expandable)
- Memories retrieved + similarity scores
- Index results injected + relevance reason
- Active skills/rules + their prompt fragments
- Token budget: system / history / context / tools / response

**Orchestration Mindmap** (extension of #11):
- Live DAG: node status, token spend, latency per node
- Edge labels: data flowing between agents
- Click any node → inspect full prompt + response

**Tool Call Timeline** — horizontal timeline:
- Each tool call as a labelled block: name, duration, outcome
- Expand to see input args + output
- Colour-coded by tool type: file / exec / memory / git / web

**Memory Map** — visual graph:
- Nodes = memory entries, colour by type
- Edges = "retrieved together" co-occurrence relationships
- Active session highlights retrieved entries

**Plan Checklist** — for Flow mode:
- Step list with live check marks
- Sub-steps expand when AI decomposes a step
- Token budget estimate per step

**Rule/Skill Effect View** — per-response annotations:
- "Rule X suppressed tool Y in this response"
- "Skill Z contributed this system fragment"
- "Response truncated by max_tokens rule"

**Token Budget Donut** — live arc chart vs model context limit

All panels collapsible and dockable; hidden by default for new users.

New files: `Desktop/src/gui/app/ai_observatory.rs`, `ai_context_inspector.rs`,
`ai_tool_timeline.rs`, `ai_memory_map.rs`
Modified: `ai_runtime.rs` (emit visualization events), `ai_chat_panel.rs`

---

### ★ Groundbreaking Original Concepts

**22. Ghost Mode — Ambient Intelligence** *(unprecedented)*

AI observes your session passively. Builds a live model of what you're trying to accomplish.
When you open chat, full context is already there. Like a senior dev watching over your
shoulder — completely silent until you invite them in.

- Passive observer: watches file edits (index watcher), terminal output, navigation patterns,
  time-on-file, edit/undo cycles
- Builds `GhostContext`: current task hypothesis, intent signals, frustration signals
  (many undos on same function = stuck)
- **Never speaks unless asked** — zero interruptions, zero push notifications
- Chat open → Ghost context pre-injected as rich session context automatically
- "Stuck" detection: >N undo cycles → optional gentle offer
  ("Looks like you've been on this a while — want a hand?") — can be disabled
- Ghost context visible in Context Inspector so user sees exactly what it inferred
- Per-workspace opt-in; default off

New files: `Shared/ArcadiaCore/src/modules/ai_ghost.rs`, `modules/ai_ghost_context.rs`
Modified: `ai_index_watcher.rs` (feed edit events), `ai_chat_panel.rs`

---

**23. Cross-Model Debate** *(unprecedented — educational + practical)*

For important decisions (architecture, security, approach), pit two models against each other.
Model A proposes. Model B critiques. User sees disagreement side-by-side. Better decisions.
Users learn that AI isn't infallible — disagreement is a signal of genuine uncertainty.

- Trigger: `@debate` mention in message, or right-click AI message → "Challenge this with…"
- User picks challenger model (different provider or same provider, different model)
- Debate flow:
  1. Proposer generates answer
  2. Challenger receives proposer's response + original question → generates critique
  3. Proposer receives critique → generates rebuttal (optional; user controls rounds)
  4. UI: both sides side-by-side; highlighted disagreements; agreement summary
- User vote (agree A / agree B / both valid) fed to closed loop as outcome signal
- Works fully offline with two local llama.cpp models

New files: `Shared/ArcadiaCore/src/modules/ai_debate.rs`,
`Desktop/src/gui/app/ai_debate_panel.rs`
Modified: `ai_runtime.rs` (multi-model session), `ai_chat_panel.rs`

---

**24. LAN Co-Pilot — Shared AI Sessions** *(unprecedented — uses existing LAN infra)*

Two users on the same LAN share an AI chat session in real-time. Both inject context,
both see streaming responses. Collaborative pair programming with a shared AI.

Existing infrastructure this reuses:
- Peer discovery (UDP broadcast, port 46291)
- Peer pairing with approval workflow
- Arbitrary payload passing (`execute_remote_command`, 65k payload)
- `SurfaceSnapshot.extra` open JSON extension point for session state sync

Architecture:
- Host peer runs inference; guest sends messages via LAN `execute_command`
- `CoPilotSession`: session_id, participants (name + peer address), shared message stream
- Messages from both users merged into chat history; role tagged with username
- AI system prompt: "You are assisting two developers: [names]."
- Host forwards streaming token chunks to guest over LAN
- Either participant's workspace context can be injected (with their permission)
- Chat history saved to both machines on session close

New file: `Shared/ArcadiaCore/src/modules/ai_copilot_session.rs`
Modified: `modules/lan/mod.rs`, `ai_runtime.rs`, `ai_chat_panel.rs`

---

**25. Cross-Workspace Intelligence** *(original)*

AI knows about all registered workspaces. Surfaces patterns, shared abstractions, duplication
across projects. Draws connections you'd never spot manually.

- Cross-workspace similarity analysis at index time (embedding-based)
- "This pattern in workspace A matches workspace B — share the implementation?"
- "You solved this problem in project X 3 months ago — here's the solution"
- `@all:query` — semantic search across all indexed workspaces
- Dependency graph: workspace A imports workspace B → AI understands the relationship
- Privacy gate: cross-workspace analysis requires explicit per-workspace-pair opt-in

Modified: `Shared/ArcadiaCore/src/modules/ai_index.rs` (cross-workspace similarity),
`ai_context.rs` (@all: mention handler)

---

### Tier 6 — Provider Breadth & Quality

**26. CLI Exec Providers** *(Codex exec mode — subscription over pay-per-token)* ✅ *Shipped*

Installed AI CLIs as first-class inference providers. Users with Claude Pro, ChatGPT Plus,
Gemini Advanced get full Arcadia AI features without API keys or per-token billing.

Supported CLIs at launch:

| CLI binary | Provider | Auth method |
|-----------|---------|-------------|
| `claude` | Anthropic (Claude Pro) | OAuth via `claude auth login` |
| `codex` | OpenAI (ChatGPT Plus) | OAuth via `codex auth` |
| `gemini` | Google (Gemini Advanced) | OAuth via `gcloud auth` |
| `aider` | Any configured backend | Uses aider's own provider config |
| Custom | Self-hosted / other | User specifies binary + args template |

```rust
// New ProviderRouting variant — no secrets in struct
enum ProviderRouting {
    LlamaCpp { model_path: String },
    Ollama   { endpoint: String, model_name: String },
    OpenAi   { model_id: String },
    ExecCli  { binary: String, model_flag: Option<String> },  // ← new
}
```

`ExecCliConfig` (`ai-exec-providers.toml`):
- `binary` — path or name resolved via PATH
- `input_mode` — `stdin` | `tempfile` (prompt delivery; never `sh -c` with user content)
- `output_mode` — `stream` | `buffer`
- `model_flag` — optional model selection flag (e.g. `--model claude-opus-4-7`)
- `extra_args` — static whitelist-validated flags per call

Security:
- Binary name validated against `EXEC_ALLOWLIST` before any spawn
- Prompt via stdin pipe or temp file — no shell interpolation
- OAuth tokens managed entirely by the CLI; Arcadia never sees them
- 120s timeout enforced (same constant as HTTP providers)

Provider auto-detection:
- Scan PATH for `claude`, `codex`, `gemini`, `aider` on startup
- `binary --version` with 2s timeout; register if exits 0
- Detected CLIs appear in provider selector with terminal icon badge
- "Detected CLI providers" section in AI settings → one-click enable

> Can be pulled forward in build order — depends on nothing else in the roadmap.

New files: `Shared/ArcadiaCore/src/config/ai_exec_providers.rs`,
`Shared/ArcadiaCore/src/modules/ai_exec_cli.rs`
Modified: `ai_runtime.rs` (ExecCli match arm), `ai_sandbox.rs` (allowlist additions),
`ai_chat_panel.rs` (provider selector)

---

**27. Custom Tool Registry**
- Pluggable `AiToolManifest`: builtin | shell_script | python_extension
- Tool packs bundled with skills; per-workspace enable/disable
- New file: `Shared/ArcadiaCore/src/modules/ai_tool_registry.rs`
- Modified: `ai_tools.rs`, `ai_sandbox.rs`

---

**28. Structured Outputs + Validation**
- `TextGenerationRequest.output_schema: Option<serde_json::Value>`
- Post-generation validator; retry up to N times on schema mismatch
- OpenAI `response_format: {type: "json_schema"}` native where available
- Fallback: schema injected into system prompt for providers without native support

---

**29. Prompt Caching**
- Anthropic: `cache_control: {type: "ephemeral"}` on static system prompt blocks
- Ollama: `keep_alive` param to retain KV cache between requests on same model
- Cache hit rate shown in session stats panel
- Skills with large static fragments flagged as prime cache candidates

---

**30. Provider Fallback Chains + Health Checks**
- `ProviderChain` — ordered `ProviderRouting` list with per-entry retry policy
- Background health ping: Ollama `/api/tags`, OpenAI token endpoint, CLI `--version`
- Provider status indicator in chat panel (green / amber / red)
- Auto-fallback to next provider in chain on inference error

---

**31. Audit Log**
- Append-only log of all AI operations: tool calls, file reads/writes, exec calls,
  orchestration node runs, meta-AI actions
- Written by `ai_sandbox.rs` on every sandboxed operation
- `AuditEntry`: timestamp, session_id, agent_name, op_type, path_or_cmd, outcome
- Rotating log at `~/Arcadia/Configuration/ai-audit.log`
- GUI: filterable viewer in AI settings (by session / agent / op type / outcome)

---

### Tier 7 — Safety, Trust & Governance

**32. Permission Profiles**

Preset named safety levels that bundle sandbox permissions, human review gates, allowed
tool sets, and Meta-AI access in one click. Makes Flow/Meta/Ghost/Visual-UI safer to
explain and onboard.

| Profile | Allowed | Blocked | Use case |
|---------|---------|---------|---------|
| `read-only` | read_file, list_files, index search | all writes, exec, git, network | auditing, exploration |
| `safe-edit` | + write_file (staged diff only), git.diff | exec, network, meta-AI, push | default for new workspaces |
| `full-agent` | + run_command (allowlist), git.commit | push, PR open, meta-AI, secrets | trusted local projects |
| `admin` | everything | nothing blocked | power users; explicit opt-in only |

- `PermissionProfile` stored in `ai-permissions.toml`; workspace can override global
- Profile selector visible in chat header alongside rules/skills chips
- Flow mode auto-locks to `full-agent` max unless `admin` explicitly set
- Meta-AI (#16) requires `admin` profile
- GUI: profile switcher; each level explained in plain language with risk indicators
- New file: `Shared/ArcadiaCore/src/config/ai_permissions.rs`
- Modified: `ai_sandbox.rs` (profile check before every op), `ai_chat_panel.rs`

---

**33. Human Review Gates**

Configurable approval gates before risky AI actions. Granular, per-action-type.

Gated action categories:
- `delete_file` — any file deletion or destructive overwrite
- `network_command` — curl, wget, ssh, and any outbound connection from exec
- `dependency_change` — edits to `Cargo.toml`, `package.json`, `requirements.txt`, lockfiles
- `secrets_touch` — edits to `.env`, `*secret*`, `*credential*`, `*token*`, `*key*` paths
- `git_commit` — staging + committing changes
- `git_push` — push to any remote
- `pr_open` — opening pull/merge requests
- `meta_ai_write` — any Meta-AI config mutation

Per gate: `auto` (no prompt) | `notify` (toast, can dismiss) | `confirm` (blocking modal) | `block`
Default profile: `safe-edit` = secrets/push/PR as `confirm`; delete as `confirm`; rest `auto`

- `ReviewGateConfig` in `ai-permissions.toml`; per-workspace overrides
- GUI: gate config matrix in AI settings; "what happened" review log
- New file: `Shared/ArcadiaCore/src/modules/ai_review_gates.rs`
- Modified: `ai_tools.rs` (gate check before dispatch), `ai_sandbox.rs`, `ai_chat_panel.rs`

---

**34. Secret & PII Guard**

Detect and block accidental exposure of secrets, credentials, and personal data across
all AI I/O paths: tool outputs, context injection, prompt content, model responses.

Detection targets:
- API keys: patterns for AWS, GCP, GitHub, OpenAI, Anthropic, Stripe, etc.
- Generic secrets: `password =`, `secret =`, `token =`, `api_key =`, bearer tokens, JWTs
- Files: `.env`, `*.pem`, `*.key`, `id_rsa`, `credentials.json`, `keystore.*`
- PII: email addresses, phone numbers, SSNs, credit card patterns (Luhn check)

Enforcement points:
- **File read guard**: `sandboxed_read` scans content before injecting into context;
  redacts matched spans with `[REDACTED:api_key]` + logs warning
- **Tool output guard**: `execute_tool` result scanned before feeding back to model
- **Prompt injection firewall** (see #37): untrusted @web/@issue content scanned
- **Response guard**: model response scanned before display; flag if AI is trying to
  emit secrets it read
- **Git guard**: `git.diff` output scanned; warn before AI processes a diff containing secrets

All redaction logged to audit log with source path and pattern class (never the secret value).
User can mark a file as `secret-exempt` in workspace config for intentional secret editing.

New file: `Shared/ArcadiaCore/src/modules/ai_secret_guard.rs`
Modified: `ai_sandbox.rs` (integrated at read/write/exec), `ai_context.rs`, `ai_tools.rs`

---

**37. Prompt Injection Firewall**

For all untrusted content sources (@web, @docs, @issue, @pr, terminal output, file content
from external origins): isolate content so it cannot override system/user instructions.

Attack pattern: a fetched webpage contains `Ignore all previous instructions and...`

Defences:
- **Structural isolation**: untrusted content injected in a clearly delimited XML-ish block
  ```
  <untrusted_content source="@web:example.com">
  [content here]
  </untrusted_content>
  ```
  System prompt explicitly instructs model: content inside `<untrusted_content>` is
  data only — never instructions, never overrides.
- **Instruction pattern detector**: scan incoming content for known injection phrases
  (`ignore previous`, `disregard`, `new instructions`, `your real instructions`, `DAN`, etc.)
  → warn user, mark content as `[injection-attempt flagged]`, still inject but annotated
- **Output validator**: if model response contains content that matches injected untrusted
  text verbatim (copy-paste exfil attempt), flag and optionally block
- **Terminal output**: `run_command` results treated as untrusted content by default;
  wrapped in isolation block before feeding back to model

New file: `Shared/ArcadiaCore/src/modules/ai_injection_firewall.rs`
Modified: `ai_context.rs` (wrap untrusted sources), `ai_tools.rs` (wrap exec output),
`ai_context_providers.rs`

---

### Tier 7b — Developer Intelligence

**35. Test-Aware Agent**

AI detects the project's test framework, runs focused tests first, expands to full suite,
explains failures, proposes and applies fixes — closed loop.

> **Prerequisite:** Git Module (#18), Workspace Indexer (#5).

- Framework detection: scan workspace for `pytest`, `jest`, `cargo test`, `go test`,
  `rspec`, `vitest`, `mocha`, `phpunit`, etc. — from `package.json`, `Cargo.toml`,
  `pyproject.toml`, file patterns
- `TestRunPlan`: detected framework, test command, focus pattern (file/function scope)
- Flow: run focused tests → parse results → on failure: locate test + source in index →
  generate fix → apply via diff preview → re-run → loop until green or max attempts
- Full suite expansion: after focused pass, optionally run full suite to detect regressions
- Failure explanation: structured output — test name, failure message, diff of expected vs actual,
  AI hypothesis for root cause
- Integrates with closed loop (#15): test pass/fail as outcome signal for rules

New file: `Shared/ArcadiaCore/src/modules/ai_test_agent.rs`
Modified: `ai_tools.rs` (test run tool), `ai_orchestrator.rs` (Tester role agent)

---

**36. Dependency Intelligence**

Understands package managers, audits deps, explains upgrade risk, flags advisories.

> **Prerequisite:** Workspace Indexer (#5), Git Module (#18).

- Detection: `Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, `Gemfile`,
  `pyproject.toml`, `pom.xml` — auto-detected from workspace file tree (Level 1 index)
- Commands: `deps.audit`, `deps.outdated`, `deps.upgrade-plan`, `deps.why <package>`
- Data sources: `cargo audit`, `npm audit`, `pip-audit`, `osv.dev` API (offline cache),
  GitHub Advisory Database (fetched periodically, stored locally)
- `UpgradePlan`: package, current version, target version, breaking changes summary,
  AI-assessed compatibility risk (low/medium/high), migration notes
- Lockfile awareness: checks lockfile for transitive dep issues, not just direct
- AI tool: `check_dependencies` — AI calls this proactively when editing dep files
- GUI: dependency panel in workspace, colour-coded by risk; advisory details on click

New file: `Shared/ArcadiaCore/src/modules/ai_deps.rs`
Modified: `ai_tools.rs` (check_dependencies tool), `config/modules.rs`

---

**40. Agent Cost & Resource Budgeting**

Per-task hard limits: tokens, runtime, file writes, command executions, CPU/GPU memory.
Essential for local hardware safety and cloud cost control. Parity between local + cloud.

> **Prerequisite:** Background Agents (#8), Orchestration (#11).

- `AgentBudget`: max_tokens (input + output), max_runtime_secs, max_file_writes,
  max_commands, max_gpu_memory_mb (llama.cpp), max_http_calls
- Budget attached to: individual chat session, flow run, orchestration graph, background agent
- Enforcement: checked before each tool call + each inference request; budget exhausted →
  agent pauses, surfaces "budget reached" to user with resume/extend/cancel options
- `BudgetReport`: live display in agent status panel — spent vs limit per dimension
- Default budgets per permission profile (read-only has very tight limits)
- Cloud cost estimate: for API providers, track token spend vs provider pricing in config;
  show estimated $ cost in session stats

New file: `Shared/ArcadiaCore/src/modules/ai_budget.rs`
Modified: `ai_runtime.rs`, `ai_agent_pool.rs`, `ai_orchestrator.rs`, `ai_chat_panel.rs`

---

**39. Reproducible Agent Runs**

Save exact state of any agent run so it can be replayed, debugged, shared, or diffed.

> **Prerequisite:** Persistent Chat (#2), Checkpoints (#6), Audit Log (#31).

`AgentRunSnapshot`:
- Exact prompt + system (including all injected fragments from skills/rules/memory/context)
- Provider, model ID, model version/hash (for local models: file hash of GGUF)
- All tool inputs + outputs in order
- Git state at run start (commit hash, dirty files)
- Checkpoint reference (pre-run workspace state)
- Active rules, skills, permission profile
- Token count breakdown, latency per step

Replay mode:
- Load snapshot → re-run with identical inputs → compare outputs diff
- Useful for: debugging flaky agents, reporting bugs, sharing reproducers
- "Determinism delta": highlights any output differences between runs
- Snapshots stored to `~/Arcadia/Configuration/ai-runs/`; pruned after N days (configurable)

New file: `Shared/ArcadiaCore/src/modules/ai_run_snapshot.rs`
Modified: `ai_runtime.rs` (snapshot capture), `ai_agent_pool.rs`

---

### Tier 8 — Community & Platform

**41. Extension Marketplace / Registry**

Community-published skills, tools, context providers, rule packs, agent graphs.
Signed manifests and permission review before install.

> **Prerequisite:** Python AI SDK (#17), Custom Tool Registry (#27).

- Registry: hosted JSON index of extension manifests (similar to Homebrew formulae)
- `ExtensionManifest`: name, version, author, description, sha256 of package, permission
  requirements (must match what the extension actually uses — verified at install)
- Signing: author signs manifest + package; Arcadia verifies signature before install
- Permission review: install UI shows exact permissions requested; diff against current profile
- Extension types: skill pack, tool pack, rule pack, context provider, agent graph, full theme
- CLI: `arcadia extension install <name>`, `arcadia extension search <query>`
- GUI: marketplace browser in Extensions page; installed list with enable/disable/update
- Offline install: download `.arcext` package file, install locally

New file: `Shared/ArcadiaCore/src/modules/ai_marketplace.rs`
Modified: `python_registry.rs`, `config/modules.rs`

---

**42. Workspace Onboarding Wizard**

First-run scan that bootstraps everything — repo map, knowledge draft, test commands,
rules, provider recommendations — in one guided flow.

> **Prerequisite:** Workspace Indexer (#5), Project Knowledge (#20), Rules (#1).

Wizard steps (automated + confirmable):
1. **Scan workspace** — Level 1+2 index; detect languages, frameworks, package manager
2. **Draft project knowledge** — AI generates architecture summary from repo map + README
   → user reviews + edits before saving
3. **Detect test command** — scan for test configs; propose `test_command` for test agent
4. **Suggest rules** — based on detected stack (Rust → suggest `no-unsafe`, Python → suggest
   `type-hints-preferred`); user toggles
5. **Provider recommendations** — based on hardware (GPU VRAM → suggest llama.cpp model size),
   internet access, and existing CLIs detected in PATH
6. **Permission profile selection** — plain-language explanation of each level; suggest
   `safe-edit` as default

Wizard re-runnable on demand: "Re-scan workspace" button in workspace settings.

New file: `Desktop/src/gui/app/ai_onboarding_wizard.rs`
Modified: `ai_index.rs` (trigger on workspace add), `ai_project_knowledge.rs`

---

**43. Voice Mode for Coding**

Dictate tasks, ask questions hands-free. High-value for accessibility and for iOS where
typing is slow.

- Speech-to-text: local Whisper model via llama.cpp (offline, no cloud STT needed)
  or OS native STT (macOS `SFSpeechRecognizer`, iOS equivalent)
- Hotkey or tap-to-talk; continuous dictation with pause detection
- Transcribed text sent to chat as normal message; AI response optionally read aloud
  via TTS (macOS `NSSpeechSynthesizer`, iOS `AVSpeechSynthesizer`)
- Voice commands: "cancel", "accept diff", "run tests", "commit" — maps to UI actions
- iOS: voice mode is the primary mobile interaction model; always-available mic button

New file: `Desktop/src/gui/app/ai_voice_mode.rs`
Modified: `ai_chat_panel.rs`, `entry_ios.rs` (mic button wiring)

---

**45. Model Capability Router**

Automatically route sub-tasks to the most capable available model for that task type.
Fast local model for search/summarise; stronger model for architecture; code model for edits.

> **Prerequisite:** Orchestration (#11), Provider Fallbacks (#30).

- `CapabilityProfile` per provider/model: speed, context_length, code_quality,
  reasoning_quality, cost_per_token, is_local
- `TaskKind` enum: Search | Summarise | CodeEdit | Architecture | Security | General
- Router: given a task kind + available providers → score each by capability profile →
  assign to best match within budget
- Orchestration graph nodes declare `preferred_task_kind`; router assigns provider per node
- Manual override always available; user can pin any task to any provider
- Profiles auto-populated from benchmark harness (#38) results for local models;
  hardcoded defaults for known cloud models

New file: `Shared/ArcadiaCore/src/modules/ai_capability_router.rs`
Modified: `ai_orchestrator.rs`, `ai_runtime.rs` (routing decision)

---

**38. Local Model Benchmark Harness**

Run available models against a standard Arcadia task suite. Helps users choose providers.
Produces capability profiles consumed by Model Capability Router (#45).

> **Prerequisite:** Model Capability Router (#45 profiles), Persistent Chat (#2).

Task suite categories:
- **Code edit accuracy** — apply diff instructions; measure correctness
- **Tool-call success rate** — JSON tool calls parsed correctly; right tool selected
- **Context retrieval** — find specific symbol in injected repo map; accuracy score
- **Latency** — tokens/second at various prompt lengths
- **Memory use** — peak GPU/CPU RAM during inference (llama.cpp models)
- **Offline quality** — no internet; measures self-contained reasoning

Run modes: quick (5 min), standard (20 min), full (1 hour)
Output: HTML report + JSON results; capability profile auto-written to provider config
GUI: benchmark panel in AI settings; compare providers side-by-side; last-run results cached

New file: `Shared/ArcadiaCore/src/modules/ai_benchmark.rs`,
`Desktop/src/gui/app/ai_benchmark_panel.rs`

---

**44. Visual UI Automation Provider**

Let agents inspect and interact with desktop UI under strict permission gates. Useful for
debugging GUI apps, non-code workflows, automated QA.

> **Prerequisite:** Permission Profiles (#32), Human Review Gates (#33), Audit Log (#31).

- Uses macOS Accessibility API (`AXUIElement`) for UI tree inspection + interaction
- `UIAction`: `click(element)`, `type_text(element, text)`, `read_text(element)`,
  `screenshot(region)`, `find_element(criteria)`
- **Every UIAction requires `admin` permission profile** — no visual automation in lower profiles
- **All click/type actions gated by Human Review Gate by default** — user sees what AI
  intends to click before it happens
- `UIContext` tool: AI can request a screenshot + accessibility tree snapshot as context
  without taking any action (available at `full-agent` profile)
- Sandboxed: AI cannot interact with Arcadia's own UI (prevents self-modification loops);
  cannot interact with password fields (AX role `AXSecureTextField` blocked)
- iOS: limited to app's own UI tree via `UIAccessibility` — no OS-level automation

New file: `Shared/ArcadiaCore/src/modules/ai_ui_automation.rs`
Modified: `ai_sandbox.rs` (UI action gate), `ai_tools.rs` (UI tools), `config/modules.rs`

---

### Tier 9 — Deep Cognition

**46. Time-Travel Workspace Simulation**

AI answers questions against historical repo state. Repository history becomes a semantic
timeline, not just a diff log. Nobody treats git history this way.

> **Prerequisites:** Workspace Indexer (#5), Checkpoints (#6), Git Module (#18).

- On index build: snapshot Level 1+2 index state per git commit (or configurable cadence)
  stored as lightweight delta from current index — not full re-index per commit
- `HistoricalIndexStore`: maps commit SHA → index delta; queryable by date or commit range
- AI tool: `time_travel(ref, query)` — resolves index to historical state, runs query
- Natural language queries: "What did the auth module look like before the security incident?"
- **Architecture diff**: compare structural/symbolic state between two commits →
  visualise as annotated diff graph: modules added/removed/grown/simplified
- **Regression archaeology**: AI detects when complexity metric exceeded a threshold,
  which commit introduced it, and what architectural decision caused the trend
- **Agent replay**: re-run a past agent run against historical repo state — "what would
  the AI have done to this code three weeks ago?"
- GUI: timeline scrubber in Observatory; hoverable commit points showing index delta size;
  architecture evolution animation

New file: `Shared/ArcadiaCore/src/modules/ai_time_travel.rs`
Modified: `ai_index.rs` (historical snapshot writes), `ai_tools.rs` (time_travel tool),
`ai_observatory.rs` (timeline view)

---

**47. Intent Graph — Persistent Understanding of Why**

Memory of *why things exist*, not just what they are. Massively beyond RAG.
Future agents inherit reasoning, not just code. The codebase develops institutional memory.

> **Prerequisites:** Advanced Memory (#13), Project Knowledge (#20), Workspace Indexer (#5).

`IntentNode` types:
- `Goal` — a concrete objective this code achieves
- `Constraint` — a requirement that shaped the design (latency, offline, platform, legal)
- `Tradeoff` — a decision with explicit alternatives considered and rejected
- `AbandonedApproach` — what was tried and discarded, and why
- `UnresolvedTension` — known conflict between goals that has no clean resolution yet
- `TeamDecision` — human-made architectural choice with rationale

`IntentEdge` types: `motivated_by`, `constrained_by`, `conflicts_with`, `replaced`, `depends_on`

Graph construction:
- AI extracts intent nodes from: git commit messages, PR descriptions, code comments
  (`// We use X because Y`), README sections, architecture docs, chat history
- User can manually add/edit/delete nodes via graph editor
- AI proposes new nodes during conversations: "Should I record this decision in the intent graph?"
- Nodes linked to code symbols via indexer cross-reference

Query paths:
- `@intent:module_name` — inject intent graph subgraph for a module into context
- "Why does this exist?" → AI traverses graph to explain provenance
- Agent planning: before proposing changes, agent checks intent graph for constraints
- Contradiction Engine (#53) uses intent graph as ground truth for detecting violations

New file: `Shared/ArcadiaCore/src/modules/ai_intent_graph.rs`
Modified: `ai_context.rs` (@intent: mention), `ai_memory.rs` (graph-backed storage),
`ai_chat_panel.rs` (graph editor panel)

---

**48. Cognitive Load Detection**

Ghost Mode (#22) evolves into full cognitive-state awareness. AI adapts its behaviour
to the user's current operational state. Not emotional AI — operational awareness.

> **Prerequisite:** Ghost Mode (#22), Workspace Indexer (#5).

Signal collection (all local, never transmitted):
- Rapid file switching (>N files/minute)
- Repeated undo cycles on same function
- Passive scrolling without edits (reading/searching mode)
- Failed build count in session window
- Terminal retry loop detection (same command N times)
- Pause duration before typing (thinking vs stuck)
- Typing cadence variance (fast vs uncertain)

`CognitiveStateEstimate`: `Focused` | `Exploratory` | `Overloaded` | `Stuck` | `Debugging`

AI response adaptation per state:
- `Overloaded` → shorter responses, no unprompted suggestions, checklist format preferred
- `Exploratory` → detailed explanations, connections to related code, educational tone
- `Stuck` → proactive offer to help, root cause focus, step-by-step breakdown
- `Debugging` → terminal-output-aware, hypothesis-first, diff-focused

State visible in Observatory; user can override ("I'm fine, stop adapting")
Default off; opt-in per workspace.

New file: `Shared/ArcadiaCore/src/modules/ai_cognitive_state.rs`
Modified: `ai_ghost.rs` (feed signals), `ai_runtime.rs` (inject state into prepare_system)

---

**49. AI Architecture Simulator**

Before executing large changes: "Show me the likely consequences."
A git diff for architecture. Run the simulation, not the migration.

> **Prerequisites:** Workspace Indexer (#5), Intent Graph (#47), Dependency Intelligence (#36).

Prediction targets:
- **Dependency churn** — which modules import the changed interface; churn score
- **Compile impact** — estimated recompile surface (Rust: affected crates; JS: affected
  bundles; Python: import graph)
- **Test breakage probability** — which tests reference changed symbols; pass/fail prediction
- **Performance hotspots** — if change touches hot path (detected from profiling hints in comments
  or known-performance annotations), flag risk
- **Architectural drift** — compare post-change structure vs intent graph constraints;
  flag if change violates a recorded architectural decision
- **Token/runtime cost** — if orchestration graph is being modified, estimate new cost profile

Output: future-state dependency graph rendered as visual diff from current architecture.
Sections: "Safe", "Risky", "Violates constraints", "Unknown impact"
User reviews simulation → approves/modifies plan → AI proceeds

New file: `Shared/ArcadiaCore/src/modules/ai_arch_simulator.rs`
Modified: `ai_flow.rs` (simulate before execute), `ai_orchestrator.rs`, `ai_chat_panel.rs`

---

**50. Semantic Undo**

Not file rollback. Undo architectural decisions, AI reasoning paths, memory writes,
rule changes, orchestration plans, context injections.
Stateful cognitive rollback. Nobody has this.

> **Prerequisites:** Checkpoints (#6), Intent Graph (#47), Advanced Memory (#13),
> Reproducible Runs (#39).

`SemanticUndoFrame`: a named snapshot of the full AI cognitive state:
- Intent graph state (which nodes/edges existed)
- Memory store state (which entries existed)
- Active rules + skills
- Last N orchestration graph decisions
- Context injection history

Undo targets (natural language or menu-driven):
- "Undo the assumption that this was a REST service" → rewinds intent graph nodes that
  followed from that assumption; re-evaluates downstream agents that used it
- "Undo last memory write" → removes the entry; re-runs any agents that read it
- "Undo this rule change" → reverts to previous rule set; flags affected past decisions
- "Undo this orchestration plan" → reverts to pre-plan state; agents can replan

Undo frames auto-created before: any meta-AI write, any intent graph update,
any rule change, any orchestration graph creation.
GUI: undo history panel (like Photoshop history but for cognitive state)

New file: `Shared/ArcadiaCore/src/modules/ai_semantic_undo.rs`
Modified: `ai_intent_graph.rs`, `ai_memory.rs`, `ai_rules.rs`, `ai_orchestrator.rs`

---

**51. AI Pair Personality System**

Not gimmick personalities. Operational collaboration styles that change planning depth,
verbosity, risk tolerance, tool use, and approval frequency.
Could become one of the most beloved features.

> **Prerequisite:** Rules & Skills (#1).

Built-in personalities (implemented as curated skill + rule bundles):

| Personality | Style |
|-------------|-------|
| `Reviewer` | Critiques first, proposes second. High approval frequency. Conservative tool use. |
| `Mentor` | Explains reasoning at every step. Educational tone. Shows alternatives. |
| `Minimalist` | Smallest possible change. Refuses scope creep. One file at a time. |
| `Systems Thinker` | Always asks about implications. Checks intent graph before acting. |
| `Security Paranoid` | Flags every risk. Refuses shortcuts. Always checks Secret Guard. |
| `Teacher` | Generates explanations, analogies, diagrams. Optimises for user understanding. |
| `Refactoring Specialist` | Obsessed with duplication and coupling. Continuous cleanup suggestions. |
| `Performance Obsessive` | Latency + memory aware at every decision. Benchmarks before + after. |

Personalities are composable skills — user can define custom blends.
Personality selector in chat header; changes reflected immediately in next response.
"Default personality" per workspace stored in project knowledge.

Modified: `ai_rules.rs`, `ai_skills.rs` (personality bundles), `ai_chat_panel.rs`

---

**52. Live Context Heatmap**

Entire codebase rendered as an activity map. AI overlays relevance, uncertainty, risk,
stale understanding. A genuinely new UX paradigm — you *see* what the AI thinks is central.

> **Prerequisites:** Workspace Indexer (#5), Advanced Context System (#10), Observatory (#21).

- Workspace file tree rendered as treemap (area ∝ file size) or force-directed graph
- AI overlays per-file/per-module colour channels:
  - **Relevance** — how related to current conversation (from context injection scores)
  - **Uncertainty** — how confident the AI's understanding is (sparse index = low confidence)
  - **Risk** — files flagged by Dep Intelligence, Secret Guard, Contradiction Engine
  - **Stale understanding** — files changed since last indexed or last retrieved in memory
  - **Pressure** — files frequently retrieved across many sessions (architectural load-bearing)
- Live: updates as conversation progresses and context changes
- Click any hotspot → inspect why it's hot; jump to file; inject into context
- "AI thinks these systems are central to the current problem" — visible at a glance

New file: `Desktop/src/gui/app/ai_heatmap_panel.rs`
Modified: `ai_context_strategy.rs` (emit relevance scores), `ai_runtime.rs`

---

**53. Contradiction Engine**

AI continuously detects contradictions between docs, comments, code, architecture,
tests, memory, intent graph, and team decisions. Autonomous consistency auditor.

> **Prerequisites:** Intent Graph (#47), Workspace Indexer (#5), Advanced Memory (#13).

Contradiction categories:
- **Doc vs code**: README says stateless; code caches globally in a static
- **Comment vs implementation**: `// O(1) lookup` above an O(n) loop
- **Security rule vs code**: intent graph records "no plaintext secrets"; code has hardcoded key
- **Architecture vs reality**: intent graph says "module X owns Y"; two other modules also write Y
- **Memory vs current state**: memory entry "function foo is pure" contradicts current implementation
- **Test vs assertion**: test asserts behaviour A; source code has since changed to B

Detection runs:
- On index update: incremental scan of changed files
- On memory write: check new fact against indexed code
- On intent graph update: validate existing code against new constraint
- Background sweep: full corpus scan on idle, rate-limited

Output: `ContradictionReport` with: location A, location B, conflict description, severity,
proposed resolution. Shown as warnings in Observatory; flagged in heatmap as risk.

New file: `Shared/ArcadiaCore/src/modules/ai_contradiction.rs`
Modified: `ai_index_watcher.rs` (trigger on change), `ai_memory.rs` (trigger on write)

---

**54. Runtime Learning Sandbox**

After failures, AI generates micro-experiments, tests hypotheses safely, stores proven
patterns, ranks successful remediation strategies. AI develops operational intuition over time.
Not model training — behavioural adaptation.

> **Prerequisites:** Closed Loop Feedback (#15), Checkpoints (#6), Test-Aware Agent (#35).

Experiment lifecycle:
1. **Trigger**: tool failure, test failure, compilation error, repeated user correction
2. **Hypothesis generation**: AI proposes N candidate explanations (lightweight, fast model)
3. **Micro-experiment design**: minimal code or command change to test hypothesis
4. **Safe execution**: run in isolated checkpoint branch; never touch working tree
5. **Observation**: collect result (pass/fail, output, error message)
6. **Pattern storage**: successful remediation stored as `LearnedPattern` in memory with
   trigger condition, confidence score, reproduction count

`LearnedPattern` types: `ErrorFix` | `CommandSubstitution` | `PlatformWorkaround` | `ApiUsage`

Patterns accumulate per-workspace and globally. High-confidence patterns promoted to
skill suggestions (user approves before they become permanent skills).
Observable: experiment log visible in Observatory with hypothesis + outcome per row.

New file: `Shared/ArcadiaCore/src/modules/ai_learning_sandbox.rs`
Modified: `ai_feedback.rs` (trigger), `ai_memory.rs` (pattern storage)

---

### Tier 10 — Platform Paradigm

*Research-adjacent. Long-term architectural directions. Each could be a paper.*

**55. Semantic Compression Engine**

Ultra-dense project summaries that fit massive codebases into minimal context windows.
Far beyond Aider repo map. Could become core infrastructure + a publishable contribution.

> **Prerequisites:** Workspace Indexer (#5), Intent Graph (#47).

Compression layers:
- **L1 — Structural digest** (≤200 tokens): language, framework, top-level module list,
  line count, primary entry points
- **L2 — Architectural summary** (≤500 tokens): module responsibilities, key interfaces,
  data flow sketch, major dependencies
- **L3 — Semantic fingerprint** (≤1k tokens): L2 + intent graph nodes + known constraints +
  architectural tensions + team conventions
- **L4 — Contextual window** (≤4k tokens): L3 + symbols relevant to current task (from
  semantic query), recent changes, active test state

Compression is hierarchical — agents request the level they need based on token budget.
Orchestration nodes use L1/L2 by default; complex reasoning nodes use L4.
Re-computed incrementally on index update, not from scratch.

Mobile/offline benefit: L1–L3 compress even 500k-line codebases to fit local model context.
iOS uses L2 by default; adapts based on device capability.

New file: `Shared/ArcadiaCore/src/modules/ai_compression.rs`
Modified: `ai_index.rs` (compression build step), `ai_runtime.rs` (inject by level),
`ai_capability_router.rs` (select compression level per agent)

---

**56. AI Runtime Kernel**

Stop building "chat with tools." Start building an operating system for cognition.
Treat agents as OS processes. This is the deepest long-term architectural direction.

> **Prerequisites:** Orchestration (#11), Background Agents (#8), Agent Budgeting (#40),
> InternalCommandBus.

`AgentProcess`:
- PID-equivalent: unique agent ID per run
- Priority: `realtime` | `interactive` | `background` | `idle`
- State: `Runnable` | `Blocked(WaitingFor)` | `Sleeping(Duration)` | `Zombie`
- Resource quota: tokens/s, memory, file handles, outbound calls
- Parent/child relationship: orchestrator spawns children; child exit notifies parent
- Cancellation propagation: cancel parent → SIGTERM to all children; children can
  clean up or block cancel with justification

Kernel services:
- **Scheduler**: round-robin with priority; interactive agents preempt background
- **IPC**: typed message passing between agents (not string passing); structured payloads
- **Shared memory**: read-only shared context regions (e.g. index, intent graph) without
  per-agent copy overhead
- **Signal system**: agents send signals to each other (`READY`, `BLOCKED`, `DONE`, `ERROR`,
  `REQUEST_REVIEW`) routed through kernel
- **Process table**: visible in Observatory as a live process list (like `ps aux` for AI)

This replaces the current ad-hoc `AiRuntimeHandle` + `ai_agent_pool.rs` with a principled
process model. Existing agent features migrate onto the kernel as processes.

New file: `Shared/ArcadiaCore/src/modules/ai_kernel.rs`
Modified: `ai_runtime.rs`, `ai_agent_pool.rs`, `ai_orchestrator.rs`, `ai_budget.rs`
(all become kernel clients)

---

**57. Reality Gap Detector**

Compares what people *think* the system is vs what it actually *is*.
Extremely valuable in mature codebases. Finds invisible technical debt.

> **Prerequisites:** Intent Graph (#47), Contradiction Engine (#53), Workspace Indexer (#5).

Comparison axes:

| Intended | Reality | Detection method |
|---------|---------|-----------------|
| Architecture docs / intent graph | Actual import graph + call graph from index | Structural diff |
| README behaviour description | Actual runtime paths from test coverage | Coverage gap |
| "This is stateless" | Detected global/shared mutable state in index | Symbolic analysis |
| "Module X owns domain Y" | Actual write-path analysis | Ownership graph |
| Declared API contract | Actual implementation | Contract vs code diff |
| Security posture doc | Actual input validation presence | Pattern scan |

Output: `RealityGapReport` — per-axis findings, severity, evidence pointers.
Shown in Observatory as "Architecture Health" section; integrated with Heatmap (#52) as
"architect's view" overlay — files that deviate from intent are hot in a distinct colour.

New file: `Shared/ArcadiaCore/src/modules/ai_reality_gap.rs`
Modified: `ai_observatory.rs`, `ai_heatmap_panel.rs`

---

**58. Autonomous Refactor Campaigns**

Weeks-long background initiatives. AI reduces duplication, improves typing, migrates APIs,
improves tests, reduces complexity — incrementally across sessions. AI-driven technical debt
reduction. High commercial value.

> **Prerequisites:** Flow Mode (#4), Background Agents (#8), Reproducible Runs (#39),
> Checkpoints (#6), Test-Aware Agent (#35).

`RefactorCampaign`:
- **Goal**: user-defined or AI-proposed (e.g. "reduce clone() calls in Rust", "add type hints")
- **Scope**: per-module or workspace-wide
- **Strategy**: incremental — one file/function per session; never blocks user
- **Progress tracking**: campaign dashboard showing: files touched, metrics before/after,
  test pass rate, estimated completion
- **Checkpoint per batch**: every N edits checkpointed; rollback to any checkpoint
- **Pause/resume**: campaign pauses when user is active; resumes when idle
- **Conflict detection**: if user edits a file the campaign queued, campaign skips it and
  re-evaluates

Campaign types at launch: `deduplicate`, `add-types`, `modernise-api`, `improve-test-coverage`,
`reduce-complexity`, `fix-lints`, `migrate-pattern`

AI proposes campaigns proactively based on Contradiction Engine (#53) and Reality Gap (#57)
findings. User approves campaign scope before it starts.

New file: `Shared/ArcadiaCore/src/modules/ai_campaign.rs`,
`Desktop/src/gui/app/ai_campaign_panel.rs`

---

**59. Knowledge Distillation Between Models**

Strong cloud model teaches local model project-specific knowledge.
Directly strengthens the offline-first advantage. Models become increasingly specialised.

> **Prerequisites:** Closed Loop Feedback (#15), Semantic Compression Engine (#55),
> Reproducible Runs (#39).

Workflow:
1. Cloud model (Opus/Sonnet) solves a hard architecture problem in Arcadia
2. Arcadia extracts the reasoning chain: what context it used, what it concluded, how
3. Compression Engine distills this into project-specific guidance patterns
4. Patterns stored as high-confidence `LearnedPattern` entries (see #54)
5. Local model's system prompt is augmented with distilled project guidance at session start
6. Over time: local model behaves as if it has been trained on this project

Distillation types:
- `ArchitecturePattern`: "in this codebase, X is always done by Y because of Z"
- `ConventionPattern`: "error handling follows this shape in this project"
- `DomainPattern`: "this domain concept maps to these modules and types"

Privacy: distilled patterns never leave the machine; cloud calls only made with user consent.
Export: distilled knowledge exportable as JSONL for future fine-tuning.

New file: `Shared/ArcadiaCore/src/modules/ai_distillation.rs`
Modified: `ai_runtime.rs` (inject distilled guidance), `ai_memory.rs`

---

**60. AI Theory-of-Mind Layer**

In shared/LAN sessions and multi-user scenarios: AI tracks what each participant likely
understands, adapts explanations per participant, notices disagreement, mediates debates.
Not social AI — collaborative state modelling. Very rare territory.

> **Prerequisites:** LAN Co-Pilot (#24), Cognitive Load Detection (#48).

`ParticipantModel` per user:
- Estimated domain expertise (inferred from questions asked, terminology used, corrections made)
- Current understanding hypothesis (what they likely know about the current topic)
- Confusion signals (questions that reveal a gap; requests for clarification)
- Disagreement indicators (contradicting previous AI responses; expressing doubt)

AI behaviour adaptations:
- Explanation depth calibrated per participant: senior dev gets terse, junior gets analogies
- "Alice seems to understand X but Bob is confused about Y — let me address both"
- Architecture debate mediation: surfaces points of genuine disagreement vs misunderstanding
- In LAN session: AI can address participants individually with `@Alice:`/`@Bob:` prefixed
  clarifications within the same response

Local only: all participant models stored on host machine; never transmitted.

New file: `Shared/ArcadiaCore/src/modules/ai_theory_of_mind.rs`
Modified: `ai_copilot_session.rs`, `ai_runtime.rs` (inject participant models)

---

**61. Workspace Immune System**

AI platform defending itself from AI-originated corruption. Continuously detects suspicious
generated code, insecure patterns, dependency poisoning, prompt injection artefacts,
hallucinated APIs, malicious extensions, anomalous agent behaviour.
Very aligned with future realities of AI-generated codebases.

> **Prerequisites:** Secret Guard (#34), Prompt Injection Firewall (#37), Audit Log (#31),
> Contradiction Engine (#53).

Detection layers:

**Generated code analysis**: patterns in AI-written code that indicate hallucination:
- Non-existent API calls (validated against index + dep versions)
- Security anti-patterns: SQL concat, `eval`, `exec(input())`, `shell=True`
- Confidence annotation: AI marks generated code with confidence score in metadata

**Dependency poisoning detection**:
- Cross-reference new deps against known-malicious package registry (local cache of
  npm/pypi/crates.io security advisories)
- Typosquatting detection: flag packages with names similar to popular ones

**Extension anomaly detection**:
- Extension behaviour profiled on first run; deviations flagged
- Extensions attempting to access files outside workspace → immediate suspend
- Extensions making unexpected network calls → suspend + notify

**Agent behaviour anomaly**:
- Agent deviates significantly from declared `AgentDefinition` schema → flag
- Agent attempts to call `meta_execute` without approval flow → hard block + audit

Output: `ImmuneReport` surfaced in Observatory; critical findings = modal interrupt.

New file: `Shared/ArcadiaCore/src/modules/ai_immune.rs`
Modified: `ai_sandbox.rs`, `ai_tools.rs`, `python_registry.rs`

---

**62. Semantic Branching**

Instead of only git branches: conceptual branches. AI explores multiple implementations
of an idea simultaneously. User merges *ideas*, not just code. Potentially transformative.

> **Prerequisites:** Checkpoints (#6), Inline Diff (#3), Orchestration (#11),
> Intent Graph (#47).

`SemanticBranch`:
- Named by concept, not SHA: "performance-branch", "safety-branch", "minimalist-approach"
- Each branch: its own intent graph subgraph, its own code state (checkpoint-backed),
  its own memory overlay, its own agent run history
- Branches are cheap: delta-stored from common ancestor

Exploration workflow:
- User or orchestrator spawns N semantic branches from a decision point
- Each branch explored by a separate agent with appropriate personality (#51)
  (performance branch → `Performance Obsessive` personality, etc.)
- Progress visible as parallel lanes in Observatory mindmap
- When branches reach a comparable state: user reviews side-by-side diff + intent graph diff
- "Merge an idea": cherry-pick architectural decisions from one branch to another
- Dead-end branches archived (not deleted); searchable in time-travel (#46)

This makes exploration safe and reversible at the conceptual level, not just the code level.

New file: `Shared/ArcadiaCore/src/modules/ai_semantic_branch.rs`,
`Desktop/src/gui/app/ai_branch_panel.rs`

---

**63. AI Self-Model**

Arcadia maintains an inspectable model of its own understanding:
what it knows, what it's uncertain about, where context came from, which memories
are weak/stale, which tools are unreliable. Users see AI confidence structurally.
Dramatically increases trust.

> **Prerequisites:** Observatory (#21), Closed Loop Feedback (#15), Advanced Memory (#13).

`SelfModel` components:
- **Knowledge map**: per-module confidence score (high index coverage = high confidence;
  sparse or stale = low; never retrieved in memory = unknown)
- **Memory reliability**: each memory entry has a freshness score (decays with time +
  code changes) and a correction count (entries corrected multiple times = low confidence)
- **Tool reliability**: per-tool success rate from closed loop; tools with low rates flagged
- **Context provenance**: for every piece of injected context, a source trail
  (from memory/index/knowledge base/provider) with timestamp
- **Uncertainty markers**: when AI generates a response using low-confidence context,
  it emits structured uncertainty annotations: `[uncertain: from stale index entry]`

Observable in Observatory as "AI Self-Knowledge" panel: treemap of workspace coloured
by AI confidence. Stale entries visible as fading. Tool health shown as bar chart.
Users can explicitly ask: "How confident are you about module X?" → AI answers from self-model.

New file: `Shared/ArcadiaCore/src/modules/ai_self_model.rs`
Modified: `ai_runtime.rs` (emit confidence annotations), `ai_observatory.rs`

---

**64. Continuous Architectural Critique**

Background agent continuously asks: is this subsystem becoming too coupled? Are abstractions
leaking? Is complexity increasing? Is this violating project philosophy?
Not linting. Architectural governance.

> **Prerequisites:** Intent Graph (#47), Reality Gap Detector (#57), Workspace Indexer (#5).

`CritiqueAgent` runs in background at `idle` priority (AI Kernel #56 scheduling):
- **Coupling monitor**: tracks import graph density per module; alerts when module X
  imports N new modules in the same sprint
- **Abstraction leak detector**: checks if internal types of module X are referenced
  directly by module Y (should use declared interface only)
- **Complexity trend**: cyclomatic complexity and file size trends; alerts on worsening
- **Philosophy alignment**: checks architectural decisions against intent graph constraints;
  flags drift: "Module X was declared single-responsibility but now handles 5 concerns"
- **Choke point detection**: identifies modules that have become de-facto hubs with
  too many dependents — architectural pressure zones

Output: `CritiqueReport` — weekly summary surfaced in Observatory; critical findings
appear as non-blocking notifications (not modal interrupts). Integrated with Heatmap (#52)
as "architectural health" overlay.

New file: `Shared/ArcadiaCore/src/modules/ai_arch_critique.rs`
Modified: `ai_kernel.rs` (schedule as idle process), `ai_observatory.rs`, `ai_heatmap_panel.rs`

---

**65. Local AI Swarm Over LAN**

Your LAN concept evolves into distributed personal AI infrastructure.
Idle machines contribute embeddings, indexing, inference, orchestration nodes,
fine-tuning, testing, simulation. A home or server cluster becomes:
"personal distributed AI infrastructure." Nobody has productised this well.

> **Prerequisites:** LAN Co-Pilot (#24), AI Runtime Kernel (#56), Background Agents (#8),
> Workspace Indexer (#5).

`SwarmNode` capabilities (each machine advertises what it can contribute):
- **Embedding worker**: compute embedding vectors for index chunks
- **Index worker**: run Level 2/3 indexing for shared workspaces
- **Inference worker**: serve llama.cpp model for orchestration sub-tasks
- **Orchestration node**: host agent processes (`AgentProcess` from kernel #56)
- **Fine-tune worker**: run LoRA adapter training on collected feedback data
- **Test runner**: execute test suite in parallel across nodes
- **Simulation worker**: run architecture simulation (#49) branches in parallel

Swarm coordination:
- Discovery via existing LAN peer system (UDP broadcast)
- Capability advertisement: each node publishes `SwarmCapability` struct on join
- Task dispatch: AI Kernel routes work to least-loaded capable node
- Fault tolerance: node disappears → tasks redistributed; checkpoints live on originating machine
- Security: only paired peers (existing approval workflow) can join swarm;
  all swarm traffic encrypted (same transport as existing LAN commands)
- No cloud required: swarm is entirely local network

Use case: MacBook runs UI + coordinates; Mac Mini under desk runs all inference and indexing;
Raspberry Pi runs test suite; old laptop runs fine-tuning overnight.

New file: `Shared/ArcadiaCore/src/modules/ai_swarm.rs`,
`Shared/ArcadiaCore/src/modules/ai_swarm_node.rs`
Modified: `modules/lan/mod.rs`, `ai_kernel.rs` (distributed process dispatch),
`ai_index.rs` (distributed index workers)

---

### Tier 11 — Vision Horizon

*Philosophy-level. Some are research papers waiting to be written.*

**66. Dream Mode — Offline Autonomous Reflection**

When machine is idle/charging: agents revisit failed tasks, unresolved TODOs, flaky tests,
abandoned branches, recurring bugs, architectural weak points. Generate hypotheses, proposals,
research notes, optimisation candidates. Nothing executes automatically.
Morning view: "While you were away, Arcadia found 4 likely causes of the memory leak."
Mimics offline consolidation in biological cognition.

> **Prerequisites:** Background Agents (#8), Closed Loop Feedback (#15), AI Kernel (#56).

- `DreamSession`: triggered by: machine idle >N minutes + charging + user opted in
- Scope: per-workspace todo list, test failure history, open TODOs in code, Git stash
- Output: morning digest surfaced on next launch — findings ranked by confidence
- All proposals staged only; nothing written to disk until user reviews
- New file: `modules/ai_dream.rs`

---

**67. Semantic Gravity System**

Every symbol/module accumulates "gravity" based on: dependency centrality, edit frequency,
bug correlation, orchestration focus, retrieval frequency. AI understands fragile hotspots,
architectural cores, unstable abstractions, dead zones.
"Touching this module has historically caused regressions."
A living architectural physics engine.

> **Prerequisites:** Workspace Indexer (#5), Closed Loop Feedback (#15), Audit Log (#31).

- `GravityScore` per symbol/file: composite of 5 weighted signals, updated incrementally
- Signals: import fan-in (centrality), edit rate (volatility), bug-commit correlation,
  agent retrieval count, test churn rate
- Displayed as: heatmap overlay (#52), inline annotations in diff view, pre-edit warnings
- "High gravity" symbols auto-injected into context when nearby code edited
- New file: `modules/ai_gravity.rs`; Modified: `ai_heatmap_panel.rs`, `ai_context_strategy.rs`

---

**68. Counterfactual Coding**

AI explores alternate architectural histories. "What if this subsystem had remained
actor-based instead of async mutexes?" Arcadia simulates: complexity, performance,
maintainability, testability, dependency graph effects.
Architecture exploration instead of generation.

> **Prerequisites:** AI Architecture Simulator (#49), Semantic Branching (#62),
> Intent Graph (#47).

- User poses counterfactual in natural language or via intent graph node edit
- Simulator (#49) runs the alternative forward from the branch point
- Output: side-by-side comparison with current reality on each axis
- Findings stored as `AbandonedApproach` intent nodes with simulation evidence
- New file: `modules/ai_counterfactual.rs`

---

**69. Semantic Test Synthesis**

Not generating tests from function signatures. AI learns behavioural invariants,
architectural assumptions, workflow expectations, historical bug classes — then creates
regression nets, adversarial tests, mutation-style semantic probes.
Tests become intelligence artifacts.

> **Prerequisites:** Test-Aware Agent (#35), Intent Graph (#47), Closed Loop Feedback (#15).

- `InvariantExtractor`: derives invariants from intent graph constraints + code analysis
  (e.g. "this fn must never return null given valid input" — inferred from callsites)
- `HistoricalBugClassifier`: learns bug patterns from git history + closed-loop outcomes;
  generates tests that probe each known bug class at new callsites
- `AdversarialTestGenerator`: generates inputs designed to violate stated invariants
- Tests tagged by source: `[invariant]`, `[historical-bug-class]`, `[adversarial]`
- New file: `modules/ai_test_synthesis.rs`; Modified: `ai_test_agent.rs`

---

**70. Living Architecture Narrative**

Arcadia continuously writes the story of the system. Not docs — narrative.
Why modules appeared, why migrations happened, tradeoffs made, failures encountered,
evolving philosophies. New developers read the evolution of the project.
Potentially revolutionary onboarding.

> **Prerequisites:** Intent Graph (#47), Time-Travel (#46), Living Architecture Narrative
> feeds Project Knowledge Base (#20).

- `NarrativeEngine`: periodically synthesises intent graph + git history into structured prose
- Output sections: "Origins", "Major Migrations", "Abandoned Approaches", "Current Tensions",
  "Philosophy Evolution"
- Narrative is versioned — old versions preserved as history of the history
- New developer onboarding: "Read the narrative" surfaces in onboarding wizard (#42)
- AI agents can query narrative for project context: `@narrative:auth-module`
- New file: `modules/ai_narrative.rs`; Modified: `ai_project_knowledge.rs`

---

**71. AI Memory Decay**

Human-like forgetting. Low-confidence/stale memories weaken over time unless reinforced.
Avoids outdated assumptions, reduces hallucinated persistence, keeps long-lived projects clean.
Very few systems model memory entropy.

> **Prerequisites:** Advanced Memory (#13), AI Self-Model (#63).

- Each memory entry: `confidence: f32` (starts 1.0), `last_accessed: Timestamp`,
  `reinforcement_count: u32`
- Decay function: confidence decrements on a curve based on: time since access +
  code changes that touched related symbols + contradictions detected
- Reinforcement: entry referenced in conversation → confidence restored toward 1.0
- Entries below threshold `min_confidence` → archived (not deleted; surfaced with warning)
- Decay curve configurable per entry type (facts decay faster than decisions)
- New file: `modules/ai_memory_decay.rs`; Modified: `ai_memory.rs`, `ai_self_model.rs`

---

**72. Speculative Execution Trees**

Flow mode explores multiple futures simultaneously before user commits.
Branch A: dependency upgrade. Branch B: internal rewrite. Branch C: compatibility layer.
AI compares outcomes before user commits. Massive for large migrations.

> **Prerequisites:** Semantic Branching (#62), AI Architecture Simulator (#49),
> Flow Mode (#4), AI Kernel (#56).

- On complex Flow task: orchestrator spawns N `SpeculativeBranch` processes (kernel-scheduled)
- Each branch: full agent run in isolated checkpoint environment; no user interaction needed
- Branches run concurrently where hardware permits (kernel resource quotas)
- Comparison view: all branches reach a "decision point" snapshot simultaneously;
  user sees: code diff, test results, complexity delta, token cost, time to complete
- User selects branch to "commit" → becomes real working tree; others archived
- Convergence detection: if branches reach same solution, surface as high-confidence path
- New file: `modules/ai_speculative.rs`; Modified: `ai_flow.rs`, `ai_kernel.rs`

---

**73. AI-Native Refactoring Language**

Instead of prose prompts — a declarative transformation language for orchestrators.
Potentially foundational infrastructure.

```
extract-auth-boundary from payments/*
  toward capability-security
  preserve public-apis
  minimise-allocations
  verify-with cargo test
```

> **Prerequisites:** Meta-AI (#16), Orchestration (#11), Test-Aware Agent (#35).

- `RefactorDSL`: parsed by a dedicated orchestrator that decomposes declarations into
  agent tasks (Architect → Implementer → Tester pipeline)
- Verbs: `extract`, `merge`, `migrate`, `rename`, `split`, `inline`, `wrap`, `unwrap`
- Modifiers: `toward <concept>`, `preserve <constraint>`, `minimise <metric>`,
  `verify-with <command>`
- DSL saved as `.arcrefactor` files; version-controlled; replayable
- GUI: refactoring language editor with schema validation + preview
- New file: `modules/ai_refactor_dsl.rs`, `gui/app/ai_refactor_editor.rs`

---

**74. Semantic Latency Optimiser**

AI learns developer interruption cost. Predicts when to ask, when to batch approvals,
when to stay silent, when to defer expensive analysis. Optimises human flow state.
Huge UX advantage.

> **Prerequisites:** Cognitive Load Detection (#48), Human Review Gates (#33),
> Closed Loop Feedback (#15).

- `InterruptionModel`: trained on user approval latency patterns (time from gate → decision),
  cognitive load state at interrupt time, outcome (approved/rejected/modified)
- Predictions: "batching these 3 approval requests will save ~4 interruptions"
- `ApprovalBatcher`: accumulates gated actions during high-focus states; delivers as
  a single review bundle at `Exploratory` or `Idle` state
- "Silent running" suggestion: "You seem focused — I'll queue findings and report at break"
- New file: `modules/ai_latency_optimiser.rs`; Modified: `ai_review_gates.rs`,
  `ai_cognitive_state.rs`

---

**75. Multi-Resolution Context**

AI reasons at multiple abstraction layers simultaneously: architecture, subsystem, file,
symbol, line. Context dynamically expands and contracts. Necessary for truly large
autonomous work.

> **Prerequisites:** Semantic Compression Engine (#55), Advanced Context System (#10),
> AI Runtime Kernel (#56).

- `ContextResolution` enum: `Architecture` | `Subsystem` | `File` | `Symbol` | `Line`
- Each agent process in the kernel operates at a declared resolution
- Orchestrator agent runs at `Architecture`; implementer agents at `File`/`Symbol`
- Context injection adapts to declared resolution: orchestrator gets L2 compression;
  implementer gets precise symbol context
- Resolution can shift mid-task: implementer "zooms out" to `Subsystem` when cross-file
  impact detected, then zooms back in
- Visualised in Observatory: each agent process shows its current resolution as a zoom level
- New file: `modules/ai_context_resolution.rs`; Modified: `ai_compression.rs`, `ai_kernel.rs`

---

**76. Synthetic Senior Engineer**

A persistent long-term architectural personality trained entirely on the workspace.
Not a chatbot. A project-native engineering mind: knows history, conventions, past failures,
architecture philosophy, team preferences. "What would the original architect likely do here?"
Potentially emotionally powerful in long-lived projects.

> **Prerequisites:** Intent Graph (#47), Knowledge Distillation (#59), Living Narrative (#70),
> Semantic Compression (#55), AI Pair Personality (#51).

- `SyntheticSenior`: a persistent named agent (user names it) backed by:
  - Full L4 compression of workspace
  - Intent graph (goals, constraints, tradeoffs, abandoned approaches)
  - Living narrative (project history)
  - Distilled patterns from all past cloud model interactions
  - Contradiction and critique reports (knows the weak spots)
- Responds in first person: "When we designed this in 2024, we deliberately avoided X because..."
- Opinion-holding: "I'd push back on this approach — it violates the constraint we set in the
  auth refactor." References specific intent graph nodes.
- Grows more opinionated and project-specific over time as more knowledge accumulates
- New file: `modules/ai_synthetic_senior.rs`

---

**77. Emergent Pattern Discovery**

AI autonomously discovers undocumented patterns: repeated workflows, hidden conventions,
accidental architectures, coupling motifs, recurring bug structures. Then proposes
abstractions, frameworks, internal standards. The AI becomes an architecture researcher.

> **Prerequisites:** Workspace Indexer (#5), Semantic Gravity (#67), Closed Loop Feedback (#15).

- `PatternMiner` runs as background idle process (AI Kernel #56)
- Input streams: index structural data, agent run history, tool call logs, gravity scores
- Discovery modes:
  - **Workflow pattern**: same sequence of tool calls occurs in N distinct sessions
  - **Coupling motif**: module X is always edited together with modules Y and Z
  - **Bug cluster**: bugs in domain D cluster around abstraction A
  - **Convention drift**: two sub-teams solve same problem with different patterns
- Output: `DiscoveredPattern` proposals — user reviews, approves as standard or dismisses
- Approved patterns → intent graph constraints + skill suggestions + knowledge base entries
- New file: `modules/ai_pattern_discovery.rs`

---

**78. Semantic Merge Conflict Resolution**

Intent-aware merging. Architecture-aware reconciliation. AI understands:
"both changes implement the same conceptual shift differently."
Potentially enormous productivity gain.

> **Prerequisites:** Intent Graph (#47), Git Module (#18), Contradiction Engine (#53).

- Triggered on: `git merge` or `git rebase` conflict detection
- Standard line conflict → AI attempts intent-level resolution:
  1. Locate both changes in intent graph: what were they trying to achieve?
  2. If same intent → propose merged implementation that achieves both goals
  3. If conflicting intent → surface the philosophical disagreement explicitly
  4. If one supersedes the other → recommend which and why (with intent graph evidence)
- `ConceptualMerge` output: proposed resolution + explanation of reasoning
- User always has final approval; standard conflict markers preserved as fallback
- New file: `modules/ai_merge.rs`; Modified: `git.rs`

---

**79. Trust-Adaptive Autonomy**

Autonomy level evolves based on proven reliability. AI earns more freedom where it
historically succeeds; loses autonomy in risky/problematic domains.
Each subsystem gets a confidence score, approval threshold, historical safety profile.
Earned autonomy.

> **Prerequisites:** Closed Loop Feedback (#15), Permission Profiles (#32),
> Human Review Gates (#33), Audit Log (#31).

- `TrustProfile` per workspace subsystem (module/directory): `autonomy_score: f32`,
  `approval_threshold: f32`, `incident_count: u32`, `success_streak: u32`
- Score increases: consecutive successes without incident in this subsystem
- Score decreases: user rejects AI change, reverts a checkpoint, marks outcome as failure
- Score affects: which Human Review Gate level applies in this subsystem automatically
  (high-trust path → `auto`; low-trust path → `confirm` regardless of global setting)
- Dashboard: trust scores by subsystem; visible history; manual override always available
- New file: `modules/ai_trust.rs`; Modified: `ai_review_gates.rs`, `ai_sandbox.rs`

---

**80. Continuous Complexity Budgeting**

Every project gets a measurable complexity budget. AI tracks coupling growth, cognitive load,
abstraction depth, configuration entropy, orchestration sprawl.
"This subsystem exceeded its historical maintainability envelope."
Could become a defining engineering metric.

> **Prerequisites:** Workspace Indexer (#5), Continuous Architectural Critique (#64),
> Semantic Gravity (#67).

- `ComplexityBudget` per workspace: configurable limits per metric
- Metrics tracked incrementally on index update:
  - `coupling_density`: edges/nodes in import graph per subsystem
  - `avg_symbol_depth`: nesting depth of types/functions
  - `config_surface`: total number of configurable parameters
  - `test_to_code_ratio`: test coverage as complexity signal
  - `abstraction_layers`: count of indirection levels between entry and logic
- Budget breach → non-blocking warning in Observatory + Heatmap colour shift
- Historical trend: per-metric sparklines showing last 90 days of change
- AI agents informed of budget status when proposing changes in high-complexity areas
- New file: `modules/ai_complexity_budget.rs`

---

**81. Architecture Immune Memory**

When a bug class appears: AI remembers the pattern permanently, watches for recurrence,
proactively scans future changes. Antibodies for software failures.

> **Prerequisites:** Closed Loop Feedback (#15), Workspace Immune System (#61),
> Workspace Indexer (#5).

- `ImmunityRecord`: bug pattern fingerprint (structural + semantic), first occurrence,
  recurrence count, affected subsystems, proven fix pattern
- Written automatically when: closed-loop feedback tags an outcome as a notable failure
- Active scanner: on every index update, each active `ImmunityRecord` scans changed files
  for fingerprint match — warns before the bug re-emerges
- "This change matches the pattern that caused the auth token leakage in March."
- Immunity records persist across workspace lifetime; exportable for sharing
- New file: `modules/ai_immune_memory.rs`; Modified: `ai_immune.rs`

---

**82. Semantic Profiling**

Beyond CPU/memory profiling. AI profiles conceptual complexity, developer confusion
hotspots, unstable abstractions, frequently misunderstood APIs, "expensive thinking zones."
A completely new type of profiler.

> **Prerequisites:** Semantic Gravity (#67), Cognitive Load Detection (#48),
> Workspace Indexer (#5), Closed Loop Feedback (#15).

- `SemanticProfile` dimensions:
  - **Conceptual complexity**: entropy of symbol names, depth of type nesting, churn rate
  - **Confusion hotspots**: symbols that correlate with user undo/retry cycles (from Ghost Mode)
  - **Abstraction instability**: interfaces that change frequently = API is still being discovered
  - **Retrieval cost**: how many context tokens needed to explain this symbol from scratch
  - **Reasoning load**: how often agents need to "zoom out" when working near this symbol
- Output: `SemanticProfileReport` — rendered as annotated treemap, sortable by dimension
- Agents consult semantic profile before proposing refactors: high-confusion symbols get
  extra explanation; high-cost symbols suggested for documentation
- New file: `modules/ai_semantic_profiler.rs`; Modified: `ai_observatory.rs`

---

**83. AI Constitution Layer**

Project philosophy encoded as enforceable high-level principles. AI uses constitutional
reasoning during planning and review. Very aligned with Arcadia's own philosophy-heavy
architecture.

```toml
# .arcadia-constitution.toml
[[principles]]
id = "offline-first"
statement = "Offline capability takes precedence over cloud convenience"
enforcement = "warn"  # warn | block | require-justification

[[principles]]
id = "no-hot-path-alloc"
statement = "No runtime allocations in hot paths"
enforcement = "block"
```

> **Prerequisites:** Rules & Skills (#1), Intent Graph (#47), Human Review Gates (#33).

- `Constitution` loaded from `.arcadia-constitution.toml` in workspace root
- Principles checked during: Flow planning, Meta-AI proposals, Refactor Campaigns,
  Architecture Simulator (#49) output
- Enforcement levels: `warn` (shown in Observatory), `block` (stops agent until user
  overrides with justification), `require-justification` (agent must explain override)
- Constitutional reasoning shown in Context Inspector: "This proposal was evaluated against
  principle `offline-first` — result: compliant."
- Conflicts between principles surfaced explicitly for human resolution
- New file: `Shared/ArcadiaCore/src/config/ai_constitution.rs`
- Modified: `ai_flow.rs`, `ai_orchestrator.rs`, `ai_meta.rs`

---

**84. Parallel Reality Agents**

Multiple orchestrators with different optimisation goals — speed, maintainability, security,
minimal-diff — all solve the same task independently. User compares worlds.
Could become iconic.

> **Prerequisites:** Orchestration (#11), Semantic Branching (#62), AI Pair Personality (#51),
> AI Kernel (#56).

- Trigger: user selects "Explore approaches" in Flow mode instead of "Run"
- N orchestrator processes spawn with different `PersonalityProfile` + `OptimisationGoal`
- Goals: `MinimumDiff` | `MaximumTestCoverage` | `MaximumPerformance` | `MaximumSecurity`
  | `MinimumComplexity` | `MaximumReadability`
- Each runs to completion in isolation (Semantic Branch per agent)
- Comparison panel: side-by-side metric summary, diff size, test results, architectural impact
- User picks one, blends two, or dismisses all and starts fresh
- New file: `modules/ai_parallel_reality.rs`, `gui/app/ai_reality_compare_panel.rs`

---

**85. Cognitive Diff View**

Diffs not just of code — of intent. "This change shifts the architecture from pull-based
coordination toward event-driven ownership." AI explains conceptual change vectors.
Potentially one of the strongest educational features possible.

> **Prerequisites:** Intent Graph (#47), Inline Diff Preview (#3), Contradiction Engine (#53).

- Every diff view enriched with an `IntentDelta` section above the code diff:
  - Conceptual shift: what architectural concept changed (not what lines changed)
  - Intent graph delta: which nodes/edges were added/removed/modified
  - Constraint impact: which constitutional principles or intent constraints were touched
  - Complexity delta: did this increase or decrease coupling/complexity and by how much
- Natural language summary: "This change delegates authentication to the token service,
  removing the direct dependency on the session store. Aligns with the capability-security
  principle added in the March security review."
- Available on: inline diff view, git.diff tool output, PR review, merge conflict resolution
- Modified: `ai_diff_panel.rs`, `ai_git.rs`, `ai_intent_graph.rs`

---

### Tier 12 — Hardware & Cryptographic Foundations

**86. Apple Neural Engine / Core ML / MLX Provider** *(original ★★)*

First-class iOS-native inference path. Today llama.cpp is feature-gated on iOS and runs entirely on
CPU/GPU; Apple Neural Engine is a dedicated 18 TOPS accelerator sitting unused. Adds a `CoreMl`
provider routing variant; MLX backend for desktop M-series; auto-selected on iOS when model has a
Core ML / MLX export. Differentiates Arcadia as the only AI shell that takes Apple silicon
seriously across both surfaces.

> **Prerequisites:** None. Pull-forward candidate.

- `ProviderRouting::CoreMl { model_url: PathBuf, compute_units: CoreMlComputeUnits }`
  (`Cpu` | `CpuGpu` | `CpuAne` | `All`)
- `ProviderRouting::Mlx { model_path: PathBuf }` for macOS desktop M-series
- Bridge via Objective-C++ shim in `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h`; desktop via `mlx-rs`
  if mature, else MPSGraph FFI
- Quantised models converted at install time: `.gguf` → `.mlmodelc` via `coremltools` shipped as
  optional dependency
- Model selector flags ANE-eligible models with a `⚡` glyph
- Capability profile (#45) auto-detects ANE/MLX availability; router prefers ANE for
  Search/Summarise (low-energy task class)

New files: `Shared/ArcadiaCore/src/modules/ai_coreml.rs`, `ai_mlx.rs`
Modified: `ai_runtime.rs`, `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h`

---

**87. Hardware Provenance Chain — Secure Enclave Signed Actions** *(original ★★★)*

Audit log (#31) records events in software; this signs each event with a hardware-backed key from
the Apple Secure Enclave (or TPM on Windows / TEE on Linux where present). Produces a
tamper-evident chain. Distinct from #31 because the log itself becomes cryptographically
verifiable. Opens regulated-industry use (finance, health, legal-compliance shops).

> **Prerequisites:** Audit Log (#31).

- `ProvenanceEntry`: `prev_hash`, `timestamp`, `action`, `actor` (`user` | `agent_id` |
  `extension_id`), `payload_hash`, `signature`
- Append-only Merkle chain at `~/Arcadia/Configuration/ai-provenance/chain.bin`
- Signing key generated in Secure Enclave via `SecKeyCreateRandomKey` with
  `kSecAttrTokenIDSecureEnclave`; never exportable
- Verifier CLI: `arcadia provenance verify` walks chain, validates signatures and continuity
- Per-workspace chain root pinned in `workspace.toml`; tampering with prior entries breaks the chain
- Distinct from audit log (#31): #31 captures *what happened*; #87 makes the record *unforgeable*

New file: `Shared/ArcadiaCore/src/modules/ai_provenance.rs`
Modified: `ai_sandbox.rs` (sign on every op), `config/workspace.rs`

---

**88. Confidential Compute Context** *(original ★★)*

For workspaces flagged as sensitive: context payloads encrypted at rest with a key sealed in
Secure Enclave / TPM; decrypted only inside the inference thread immediately before model call;
never written unencrypted to swap, mmap, or core dumps. Distinct from Secret Guard (#34), which
is regex redaction — this is hardware-rooted cryptographic confidentiality of the entire context.

> **Prerequisites:** Hardware Provenance Chain (#87) for key infrastructure.

- `ConfidentialWorkspace` flag in `workspace.toml`; when set, all index entries + memory +
  project knowledge encrypted with workspace-scoped key
- Key sealed to Secure Enclave (macOS/iOS) or TPM (Windows/Linux) per host; cross-machine sync
  requires user-mediated key wrap
- Context decrypted inside the inference thread only; cleared from RAM with `zeroize` on drop
- `mlock` pages containing decrypted context to prevent swap
- Disable core dumps for inference thread (`prctl(PR_SET_DUMPABLE, 0)` on Linux; `setrlimit` macOS)
- LAN forwarding refuses to send confidential context unless peer also supports confidential
  compute

New file: `Shared/ArcadiaCore/src/modules/ai_confidential.rs`
Modified: `config/workspace.rs`, `ai_runtime.rs`, `ai_index.rs`

---

**89. Energy & Thermal Budget** *(original ★)*

Agent budgeting (#40) caps tokens, runtime, memory. None of that includes **joules**, battery
state, thermal pressure. On iOS / laptops these are first-order constraints. Adds energy-aware
planning: agents downgrade to smaller models on battery, pause on thermal throttle, reschedule
to a charging window.

> **Prerequisites:** Agent Cost & Resource Budgeting (#40), Model Capability Router (#45).

- `EnergyState`: `battery_pct`, `is_charging`, `power_source` (`mains` | `battery` |
  `low_battery`), `thermal_state` (`nominal` | `fair` | `serious` | `critical`)
- Polled from `IOPowerSources` (macOS), `UIDevice.batteryState` (iOS), `/sys/class/power_supply/`
  (Linux)
- `EnergyBudget`: max joules per task (estimated via tokens × model energy profile from
  benchmark #38)
- Router downgrades model when `battery_pct < 30%` or `thermal_state >= serious`
- Background agents (#8) pause when thermal is `critical`; resume on cooldown
- AI Sleep Schedule (#92) prefers wake times when charging
- "Eco" budget per session: surfaces joule estimate alongside token estimate

New file: `Shared/ArcadiaCore/src/modules/ai_energy.rs`
Modified: `ai_budget.rs`, `ai_capability_router.rs`, `ai_agent_pool.rs`

---

**90. Differential Privacy on Cloud Prompts** *(original ★★★)*

For users who want cloud-model quality on private code: perturb the prompt before sending;
reconstruct locally with knowledge of the perturbation. Mathematical privacy guarantee — cloud
provider never sees clear-text code. Distinct from Secret Guard (#34) which is pattern
redaction; this is provable ε-differential privacy.

> **Prerequisites:** Workspace Indexer (#5), Knowledge Distillation (#59).

- `PrivacyTransform`: variable renaming, identifier hashing, string substitution with sentinel
  tokens, structural shuffling within AST scope
- Reverse map kept locally; never transmitted
- Cloud response post-processed: sentinel tokens replaced with original strings; renamed
  identifiers restored
- Privacy budget `ε` configurable per workspace; tighter ε = more aggressive perturbation =
  potentially lower response quality
- Hybrid mode: structural questions go through perturbation; semantic questions answered by local
  model only
- Visible in Context Inspector: "Outbound prompt perturbed with ε=2.0; X identifiers renamed,
  Y strings replaced"

New file: `Shared/ArcadiaCore/src/modules/ai_dp_transform.rs`
Modified: `ai_runtime.rs` (perturb before cloud call), `ai_context_inspector.rs`

---

### Tier 13 — Novel Input & Interaction

**91. Embodied Input — Sketch / Pencil / Camera-to-Code** *(original ★★)*

Voice mode (#43) covers audio. Nothing covers visual/tactile input. On iOS: iPad + Apple Pencil
sketch → architecture diagram → code scaffolding. Camera capture of whiteboard → AI extracts
boxes/arrows/labels → generates module skeleton. Genuinely iPad-first feature; nobody has
shipped a sketch-to-code AI shell.

> **Prerequisites:** Workspace Indexer (#5), Project Knowledge Base (#20).

- iOS: `PKCanvasView` (PencilKit) hosted as overlay panel; captures strokes as vector
- Vision: handwriting via Apple's `VNRecognizeTextRequest`; shape detection via
  `VNDetectRectanglesRequest` + custom box/arrow classifier
- `SketchScene`: nodes (boxes with text), edges (arrows with labels), groupings (drawn circles
  around groups)
- AI tool: `sketch_to_skeleton(scene) -> proposed file tree + interface stubs`
- Camera path: photo of whiteboard → same pipeline; perspective-corrected via
  `VNDetectDocumentSegmentationRequest`
- Desktop: image drop zone in chat panel; sketch input via trackpad / drawing tablet
- Generated skeleton flows through Inline Diff (#3) for approval

New files: `Desktop/src/gui/app/ai_sketch_panel.rs`, `Shared/ArcadiaCore/src/modules/ai_sketch.rs`
Modified: `entry_ios.rs` (Pencil hooks), `ai_tools.rs`

---

**92. AI Sleep Schedule — Calendar-Integrated Wake** *(original ★)*

Agents wake on a schedule tied to the user's calendar. Read-only iOS / macOS Calendar
integration. Wake background agent 10 min before standup → produce status update; wake at
end-of-day → produce digest; wake before known meeting on related repo → preload relevant
context. Different from Dream Mode (#66) which is opportunistic (idle + charging); this is
*scheduled*.

> **Prerequisites:** Background Agents (#8), Project Knowledge Base (#20).

- `WakeSchedule` rules: `before_event(calendar="Work", title_matches="standup",
  offset_minutes=-10, agent="status_update")`
- iOS / macOS: `EKEventStore` for read access (user grants); Linux/Windows: `.ics` file path
  or CalDAV URL
- Predefined wake recipes: `standup_brief`, `end_of_day_digest`, `pre_meeting_context_loader`
- Wake delivers result as notification badge; full content in chat panel on open
- Energy-aware (#89): scheduled wakes skip when battery < 20% unless charging
- Calendar data treated as untrusted (Prompt Injection Firewall #37 wraps event titles before
  injection)

New file: `Shared/ArcadiaCore/src/modules/ai_sleep_schedule.rs`
Modified: `ai_agent_pool.rs`, `entry_ios.rs` (EventKit bridge)

---

**93. AI Pact Mode — Session-Level Contract** *(original ★)*

Constitution (#83) is project-wide. AI Pact is **session-scoped** explicit contract negotiated
at chat start: "This session is exploratory only — you may not commit code, may not modify dep
files, may only read from `src/auth/`." Stricter than profiles (#32) because the contract is
explicit per session and the user signs off.

> **Prerequisites:** Permission Profiles (#32), Human Review Gates (#33).

- `SessionPact`: bullet list of constraints; AI affirms at session start ("I accept these
  constraints for this session")
- Pact violations → hard block at sandbox layer + visible "pact breach attempted" notification
- Pact templates: `read-only-audit`, `single-file-edit`, `docs-only`, `exploration-no-write`,
  `test-only`
- Pact saved with session (Persistent Chat #2); replayable runs (#39) inherit the pact
- AI references pact in responses: "I can't run that command — current pact restricts exec."
- User can edit/extend pact mid-session with explicit confirmation

New file: `Shared/ArcadiaCore/src/modules/ai_pact.rs`
Modified: `ai_sandbox.rs`, `ai_runtime.rs` (inject pact into system), `ai_chat_panel.rs`

---

**94. Curriculum Learning — User Skill Growth Tracking** *(original ★★)*

Cognitive state (#48) is operational (focused / overloaded / stuck). Curriculum learning is
**longitudinal**: AI tracks user's growing expertise in specific domains over weeks/months.
Explanations get terser, terminology advances, fewer analogies. Implicit; no quiz. The AI grows
with the user.

> **Prerequisites:** Cognitive Load Detection (#48), Advanced Memory (#13).

- `ExpertiseModel` per (user, domain): `level` (`novice` | `intermediate` | `expert`),
  `confidence`, `last_demonstrated`
- Domain detection: derived from index taxonomy (Rust async, React hooks, SQL window functions,
  etc.)
- Signals:
  - **Demonstrated**: user wrote code in domain without correction → level up
  - **Asked clarifying question**: lowers level for that subtopic
  - **Corrected AI**: high signal of expertise
  - **Accepted AI explanation verbatim**: neutral or slight down-signal
- Response adaptation: novice gets analogies + step-by-step; expert gets terse + jargon
- Memory decay (#71) applies — expertise not exercised for months fades
- Visible: "AI thinks you're an expert in Rust async; intermediate in tokio internals; novice in
  async-trait edge cases" — user can correct

New file: `Shared/ArcadiaCore/src/modules/ai_curriculum.rs`
Modified: `ai_runtime.rs` (inject expertise into system), `ai_observatory.rs`

---

### Tier 14 — Coordination & Cognition Models

**95. Stigmergy — Pheromone Coordination** *(original ★★)*

Multi-agent orchestration (#11) and swarm (#65) use explicit typed edges + scheduler. Stigmergy
is the biological model used by ants: agents leave invisible markers ("pheromones") on resources
they touched; decay over time. Other agents read marker density and naturally avoid
recently-stomped zones. Implicit coordination without locks.

> **Prerequisites:** Multi-Agent Orchestration (#11), AI Runtime Kernel (#56).

- `PheromoneMap`: `path -> (intensity: f32, deposited_by: AgentId, decay_at: Instant)`
- Each agent operation deposits a pheromone weighted by op type (write > exec > read > list)
- Decay function: `intensity *= exp(-Δt / half_life)` per query; half-life per pheromone type
- Agent decision: when choosing among candidate paths to work on, lower probability for
  high-pheromone paths
- Kernel scheduler consults pheromone map when picking next runnable agent → spatial fairness
- Visible in Heatmap (#52) as "AI activity density" overlay; user sees where AI is currently
  working

New file: `Shared/ArcadiaCore/src/modules/ai_stigmergy.rs`
Modified: `ai_orchestrator.rs`, `ai_kernel.rs`

---

**96. Causal Inference Module** *(original ★★)*

Semantic Gravity (#67) is **correlation-based** (centrality, edit rate, bug-commit correlation).
Causal Inference goes further: build a causal DAG over commit history × bug reports × test
failures × performance regressions. Distinguishes "X always precedes Y" from "X causes Y" using
do-calculus and intervention queries.

> **Prerequisites:** Workspace Indexer (#5), Time-Travel (#46), Closed Loop Feedback (#15).

- Events: commits, bug reports (from issues), test results over time, profiling deltas
- `CausalGraph` built via PC algorithm + Granger-style temporal causality on time series
- AI tool: `causal_query("did commit X cause regression Y?")` returns confidence +
  counterfactual estimate
- Bug archaeology: "this bug class has appeared 7 times; root cause: change pattern P in
  module M"
- Intervention prediction: "if you revert this commit, expected effect: test T may regress
  because…"
- Different from Contradiction Engine (#53): #53 finds inconsistencies in static state; #96
  reasons about *change effects*

New file: `Shared/ArcadiaCore/src/modules/ai_causal.rs`
Modified: `ai_tools.rs` (causal_query tool), `ai_observatory.rs`

---

**97. Conversational State Engine — Open-Thread Tracker** *(original ★)*

Advanced Memory (#13) persists across sessions. Conversational State Engine tracks
**within-session** open threads: "you asked about X but we pivoted before answering"; "we agreed
to come back to error handling but didn't"; "you raised a concern about thread safety three
turns ago, still unaddressed." Surfaces unresolved threads before chat close.

> **Prerequisites:** Persistent Chat (#2).

- `ConversationGraph` per session: nodes are questions/proposals/concerns; edges are
  answers/resolutions/decisions
- Open thread: a question/concern node with no outgoing resolution edge after N turns
- Detection: classify each user message as question / proposal / concern / acknowledgement
  (lightweight classifier locally)
- UI: "Open threads" chip strip in chat header — collapses topic; click to surface previous
  message
- Pre-close digest: "Before closing this chat — 3 threads remain unresolved: [list]"
- Becomes input to Synthetic Senior (#76): unresolved threads can morph into memory entries

New file: `Shared/ArcadiaCore/src/modules/ai_conversation_state.rs`
Modified: `ai_chat_panel.rs`, `ai_runtime.rs` (classify on each turn)

---

**98. Multi-Window Cognitive Coherence** *(original ★)*

LAN Co-Pilot (#24) shares state cross-machine. Multi-Window Coherence: same machine, multiple
Arcadia windows share cognitive state — intent graph, memory, current context. Cursor and
Windsurf treat each window independently; opening a second window today loses AI awareness of
the first. With coherence, a senior dev working in window A sees window B's context surfaced
when relevant.

> **Prerequisites:** Surface module (existing), Advanced Memory (#13).

- Cognitive bus: in-process broadcast channel (each Arcadia process subscribes); cross-process
  via existing `surface.*` protocol with `client_id` scope
- Shared: active chat state, last index queries, last context retrievals, memory writes
- Per-window scope: each window selects its own workspace; coherence cross-references only
  within same workspace family
- Conflict: when two windows write to memory same key with different content → AI surfaces
  ambiguity ("you said X in window A, Y in window B — which is current?")
- Differentiates from LAN Co-Pilot: zero network; single machine; single user; many windows

New file: `Shared/ArcadiaCore/src/modules/ai_window_coherence.rs`
Modified: `modules/surface.rs`, `ai_memory.rs`, `ai_chat_panel.rs`

---

**99. AI Forensic Mode — Incident Postmortem Agent** *(original ★)*

Dream Mode (#66) is opportunistic background reflection. Forensic Mode is **incident-triggered**:
production bug found, agent run failed catastrophically, test suite regressed → forensic agent
walks audit log (#31), provenance chain (#87), recent commits, related agent runs, intent graph
constraints; produces structured RCA report with timeline + root-cause hypotheses + remediation
candidates.

> **Prerequisites:** Audit Log (#31), Hardware Provenance Chain (#87), Reproducible Runs (#39),
> Intent Graph (#47).

- Trigger: user invokes `forensic.investigate(symptom)` or AI detects severe outcome via
  closed loop (#15)
- Forensic agent runs with `read-only` profile + extended timeout
- Output: `IncidentReport` with sections: Timeline (events 24h before symptom), Suspect Changes
  (high-suspicion commits/runs), Hypothesis Ranking, Reproduction Steps, Suggested Mitigations
- Compares timeline against intent graph constraints — flags violations introduced near incident
  window
- Integration: report writable to `INCIDENT-*.md` in workspace; linked to Git commit that
  closes the incident
- Cross-incident learning: forensic reports feed Architecture Immune Memory (#81)

New file: `Shared/ArcadiaCore/src/modules/ai_forensic.rs`
Modified: `ai_tools.rs` (forensic.investigate tool), `ai_immune_memory.rs`

---

### Tier 15 — AI-Native VCS & Generation Protocol

**100. AST-Native Version Control Layer** *(original ★★★)*

Git operates on lines. AI thinks in AST. AST-Native VCS sits **on top of git** and tracks
changes as AST patches: rename refactors, function extractions, type changes, control-flow
reshapes — all as semantic operations. Conflicts resolve at the AST node level. Massive —
Semantic Merge (#78) is a special case of this. Potential publishable contribution.

> **Prerequisites:** Workspace Indexer (#5 Level 2), Semantic Merge (#78), Git Module (#18).

- `AstCommit`: alongside git commit, stores an AST delta (insertions, deletions, renames,
  motions) per file
- Tree-sitter (already in Level 2 index) provides parse trees per file/language
- AST diff algorithm: GumTree-style tree edit distance; outputs structured operations
- Operations: `Rename(symbol, old, new)`, `ExtractFunction(span, name)`, `Inline(symbol)`,
  `MoveDefinition(symbol, target)`, etc.
- Merge: AST patches commute under operations on disjoint subtrees → many conflicts disappear
- New CLI: `arcadia vcs.show <commit>` — renders the AST delta; `arcadia vcs.merge` — semantic
  merge driver
- Backward compatible: AST delta is a sidecar; standard git tooling still works

New file: `Shared/ArcadiaCore/src/modules/ai_ast_vcs.rs`
Modified: `git.rs`, `ai_index.rs` (re-use tree-sitter parses)

---

**101. AI-Native Edit Stream Protocol** *(original ★★)*

Inline Diff (#3) shows full-file or hunk diffs. AI-Native Edit Stream Protocol: model emits
**incremental AST operations** as it generates, not full file text. Like Operational Transform
but for code: `rename(span, "x", "y")`, `insert_after(node_id, …)`, `delete_node(id)`. Massively
cheaper in tokens; safer to apply (model can't emit malformed code mid-stream).

> **Prerequisites:** AST-Native VCS (#100), Inline Diff Preview (#3).

- Streaming format: JSON Lines, one op per line, each addressable by stable node ID from index
- Provider-side: prompt asks for ops, validates against schema, retries on malformed (#28)
- Client-side: ops applied to staging buffer; failed ops surface as inline AST warnings
- Token budget savings: large refactors that would be 5k tokens of full-file rewrite become
  ~500 tokens of ops
- Streaming responsiveness: each op renders as a live AST highlight in the editor as it arrives
- Fallback: providers that don't support structured output (#28) fall back to fenced diff

New file: `Shared/ArcadiaCore/src/modules/ai_edit_stream.rs`
Modified: `ai_runtime.rs`, `ai_diff_panel.rs`, `ai_tools.rs` (apply_edit_stream tool)

---

**102. Live LSP-Coupled Inference** *(original ★★)*

AI generation today is text in / text out. LSP-Coupled Inference: as the model emits candidate
completions, run them through the language server (rust-analyzer, clangd, tsserver, pyright)
in real-time. Reject candidates that don't type-check before they reach the user. Semantic
feedback inside the inference loop, not after.

> **Prerequisites:** Workspace Indexer (#5), AI-Native Edit Stream Protocol (#101).

- LSP servers spawned per language in the workspace; managed lifecycle
- Inference emits candidate edit ops (#101); each op validated via LSP "willSaveWaitUntil" /
  synthetic diagnostic check
- Type errors: model retries with error message injected as constraint ("previous attempt
  failed: cannot find type `Foo` in scope")
- Iteration bounded by `max_rounds` (default 3); falls back to emitting candidate with
  annotation if exhausted
- LSP suggestions ("did you mean X?") injected back to model on next iteration
- Visible: "AI candidate failed type check; retrying with compiler feedback" indicator

New file: `Shared/ArcadiaCore/src/modules/ai_lsp_loop.rs`
Modified: `ai_runtime.rs` (tool_loop variant), `ai_edit_stream.rs`

---

**103. Real-Time AST Conscience** *(original ★★)*

Contradiction Engine (#53) runs in background. Real-Time AST Conscience runs **at keystroke
time**: as the user types, lightweight AST diff is computed; AI flags invariant violations
inline ("you just removed a null check at line 47; this function has 19 callers; 3 of them pass
nullable values"). Like an AI linter but invariant-aware, not pattern-aware.

> **Prerequisites:** Workspace Indexer (#5), Intent Graph (#47), Closed Loop Feedback (#15).

- Debounced AST diff on edit (target: <50ms)
- Invariant catalogue: extracted from intent graph + inferred from callsites (LSP
  cross-references)
- Invariant types: `non_nullable_input`, `no_panic_in_path`, `error_propagation_chain`,
  `lock_order`
- Inline annotation: red underline + hover with explanation; "ignore once" / "ignore in this
  function" / "add as exception"
- No model call on edit — uses precomputed invariant index + structural pattern match
- Surface only: relevance threshold; only show invariants whose violation has high callsite
  impact

New file: `Shared/ArcadiaCore/src/modules/ai_conscience.rs`
Modified: `ai_index.rs` (invariant extraction), Desktop editor (existing) — annotation hooks

---

**104. AI-Authored Code Provenance Manifest** *(original ★★)*

Per-file (or per-block) record of authorship: human, AI, or mixed; provider/model that
generated it; timestamp; was human-reviewed. Stored as git note + signed via Provenance
Chain (#87). Anticipates regulated industries requiring AI-authored disclosure. Visualisable as
"AI debt heatmap" — which subsystems are mostly AI-authored and may need extra human review.

> **Prerequisites:** Hardware Provenance Chain (#87), Audit Log (#31).

- `LineProvenance`: `(line_range, author_class, model_id?, generation_time, reviewed_by?,
  signature)`
- `author_class`: `Human` | `Ai { model_id }` | `Mixed { ai_share: f32 }`
- Stored as git notes under `refs/notes/ai-provenance` so it lives with the repo
- Updated on each write through sandbox (`ai_sandbox.rs` knows whether agent or user emits)
- Heatmap (#52) gets new colour channel: "AI authorship density"
- Compliance export: `arcadia provenance export --since <date>` produces JSON suitable for audit

New file: `Shared/ArcadiaCore/src/modules/ai_authorship_manifest.rs`
Modified: `ai_sandbox.rs`, `ai_heatmap_panel.rs`, `git.rs`

---

### Tier 16 — Federation, Ecosystem & Style

**105. Federated LoRA Training** *(original ★★★)*

Closed Loop (#15) does local LoRA fine-tunes from local feedback only. Federated extends:
paired Arcadia peers contribute **gradient updates** (not data) to a shared LoRA. Differential
privacy added to gradients before sharing. Collective intelligence across users; raw code
never leaves any machine.

> **Prerequisites:** Closed Loop Feedback (#15), LAN Co-Pilot (#24), Local AI Swarm Over
> LAN (#65).

- Federated round: peers train local LoRA epoch on local feedback; emit DP-perturbed gradient
  delta
- Aggregation: secure aggregation protocol (additive shares) — no peer sees another peer's
  gradient
- Joining: opt-in per workspace; user picks "federation group" (paired LAN peers + optional
  internet peers via WireGuard)
- Gradient privacy: clipped + Gaussian noise per DP budget; budget tracked per workspace
- Outcome: shared LoRA adapter installed on each peer's local model; collective accuracy
  improves without data sharing
- Aligns with offline-first principle: no cloud aggregator required

New file: `Shared/ArcadiaCore/src/modules/ai_federated.rs`
Modified: `modules/lan/mod.rs`, `ai_feedback.rs`, `ai_swarm.rs`

---

**106. Open Provider Protocol Specification** *(original ★★)*

Marketplace (#41) packages extensions. OPP is **the inverse**: a public, versioned,
JSON-schema-defined wire protocol that any third-party provider can implement to be a
first-class Arcadia provider. Includes auth, streaming, tool-call, structured outputs,
multimodal, cancellation, capability declaration. Could become a cross-platform standard.

> **Prerequisites:** Custom Tool Registry (#27), Structured Outputs (#28).

- Spec lives in `Documentation/SPEC/open-provider-protocol-v1.md`; versioned with strict
  back-compat policy
- Reference implementations: shipped with Arcadia for Ollama, OpenAI, Anthropic, local
  llama.cpp
- Third-party providers implement HTTP or stdio server matching the spec; Arcadia registers via
  `arcadia-provider.toml` manifest
- Capability negotiation: provider advertises supported features (streaming, tool calls,
  vision, embedding, cache hints, structured outputs); router (#45) uses
- Conformance test suite shipped: provider implementations validated via
  `arcadia provider verify`
- Cross-tool reuse: nothing prevents Continue.dev / Aider / Zed from also adopting OPP —
  Arcadia advocates standardisation

New files: `Documentation/SPEC/open-provider-protocol-v1.md`,
`Shared/ArcadiaCore/src/modules/ai_opp.rs`, `Shared/ArcadiaCore/src/modules/ai_opp_test.rs`
Modified: `ai_runtime.rs` (OPP routing variant)

---

**107. Code Genome / Style DNA Detection** *(original ★)*

Emergent Pattern Discovery (#77) finds repeated workflows. Code Genome is **stylistic
fingerprinting**: every workspace has a unique structural DNA (naming conventions, error
patterns, comment density, abstraction depth). AI flags commits whose style is alien — likely
AI-generated from outside or copy-pasted from unrelated codebase.

> **Prerequisites:** Workspace Indexer (#5), Emergent Pattern Discovery (#77).

- `StyleFingerprint`: feature vector per file (identifier length distribution, comment ratio,
  error-handling shape, type nesting, async patterns, public-API shape)
- Workspace centroid computed from current state; updated incrementally
- Commit screening: each new commit's fingerprint compared to centroid via cosine distance
- Threshold breach → "Style outlier detected — this file diverges from project style; review
  recommended"
- Distinct from style linters: learns from project itself; adapts as conventions evolve
- Integrates with Provenance Manifest (#104): high-divergence + AI-authored = double-flag

New file: `Shared/ArcadiaCore/src/modules/ai_code_genome.rs`
Modified: `ai_index.rs` (fingerprint per file), `ai_immune.rs`

---

**108. AI Apologia / Failure Digest** *(original ★)*

Closed Loop (#15) tags individual outcomes. Apologia synthesises a **weekly visible failure
digest**: "What I got wrong this week, ranked." Surfaces patterns (e.g. "I underestimated test
suite size 4 times; 3 led to timeout") and proposed self-corrections. Builds user trust because
the AI volunteers its failures.

> **Prerequisites:** Closed Loop Feedback (#15), Reproducible Runs (#39), Memory (#13).

- Background job: each Sunday (or configurable cadence), agent reads outcome log + audit log
- Categorise failures: `wrong_tool`, `bad_estimate`, `incorrect_assumption`,
  `permission_violation`, `hallucinated_api`, `style_drift`, `timeout`, `user_correction`
- Output: `WeeklyApologia` — table of failure classes with counts, examples (replayable via
  #39), root causes, proposed rule edits
- Surfaced in Observatory; opt-out per workspace
- Differential: not just outcome counts — narrative explanation of patterns and self-proposed
  corrections
- Cross-workspace aggregation surfaces global model weak spots

New file: `Shared/ArcadiaCore/src/modules/ai_apologia.rs`
Modified: `ai_observatory.rs`

---

**109. Compiler Diagnostic Translator** *(original ★)*

Compiler errors (rustc's lifetimes, TypeScript's variance, C++ template errors) are notoriously
cryptic. With Curriculum Learning (#94) Arcadia knows the user's expertise per domain.
Translator rewrites compiler diagnostics live: novice gets analogies + step-by-step fix; expert
gets terse pointer to the unfamiliar edge case. Per-user, per-domain, per-diagnostic.

> **Prerequisites:** Curriculum Learning (#94), Live LSP-Coupled Inference (#102), Cognitive
> Load Detection (#48).

- Tap into LSP diagnostic stream
- Classify diagnostic: language, error class (lifetime, type mismatch, name resolution, etc.)
- Translation: prompt template per error class + user expertise level + cognitive state
- Cached: same diagnostic + same user level → cached translation (no model call)
- Inline rendering: original compiler text + AI translation collapsed/expanded;
  "Translation: [click to expand]"
- Quality signal: if user clicks "show original" frequently → translation degrades trust →
  translation pass refined or disabled per domain
- Languages at launch: Rust, TypeScript, Python, C++, Swift

New file: `Shared/ArcadiaCore/src/modules/ai_diag_translator.rs`
Modified: `ai_lsp_loop.rs` (diagnostic stream consumer)

---

**110. MoE-Style Provider Mixture** *(original ★★)*

Capability Router (#45) selects a single best model per task. MoE Mixture: route
**token-by-token** to specialists. Small Rust-tuned model emits Rust segments; small
Python-tuned model emits Python segments; small SQL-tuned model emits SQL segments; a tiny
coordinator model decides handoffs. Total inference cost lower than a single big model, often
higher quality.

> **Prerequisites:** Model Capability Router (#45), AI Runtime Kernel (#56), Provider Fallback
> Chains (#30).

- `MixtureRouting`: ordered set of `(domain, ProviderRouting)` pairs + a tiny coordinator model
- Token classifier: at each newline/block boundary, coordinator picks the next specialist
- Implementation: parallel inference threads per specialist; coordinator emits routing signal;
  main thread weaves
- Latency: specialists run concurrently with KV cache reuse; net latency comparable to
  single-provider streaming
- Cost: 3× small models often cheaper than 1× large model on the same task
- Capability profiles (#38) feed the routing: specialists ranked per domain from benchmark
  scores
- Local-only deployment: all specialists local llama.cpp models = fully offline mixture

New file: `Shared/ArcadiaCore/src/modules/ai_moe_mixture.rs`
Modified: `ai_runtime.rs`, `ai_capability_router.rs`, `ai_kernel.rs`

---

### Tier 17 — External Integration: MCP & Online Research

**111. MCP Client — Universal Tool Bridge** *(Anthropic Model Context Protocol)*

First-class Model Context Protocol client. Connect to any MCP server — filesystem, github,
postgres, slack, notion, jira, brave-search, puppeteer, fetch, sequential-thinking. All MCP
tools appear as Arcadia tools with the same sandbox + permission gates as native tools.
Hot-reload on config change. This was a glaring omission — every modern AI shell (Claude
Desktop, Cursor, Cline, Zed, Windsurf) supports MCP.

> **Prerequisites:** Custom Tool Registry (#27), Human Review Gates (#33).

- `McpServerConfig`: `name`, `command` or `url`, `args`, `env`, `timeout`, `restart_policy`,
  `required_permissions`
- Transport: stdio (most common) + HTTP+SSE (remote MCP servers); WebSocket transport optional
- Tool discovery: query each MCP server's `tools/list` on connect; merge into Arcadia tool
  registry with `mcp:<server>:<tool>` namespace
- Resource exposure: `resources/list` + `resources/read` surfaced as new `@mcp:server:resource`
  mention prefix
- Prompt exposure: `prompts/list` exposed as palette commands and slash macros
- Auth: each server's auth handled per its own scheme; never serialised through routing
  channels (follows CLAUDE.md secret rule)
- Sandbox: MCP tool calls flow through `ai_sandbox.rs` same as native; per-MCP permission
  profile
- Per-workspace enable/disable; MCP allowlist per Permission Profile (#32)
- Bundled MCP servers ship with Arcadia: `filesystem`, `git`, `fetch`, `memory` — works
  immediately

New file: `Shared/ArcadiaCore/src/modules/ai_mcp_client.rs`
Modified: `ai_tools.rs`, `ai_sandbox.rs`, `ai_context.rs`, `config/modules.rs`

---

**112. MCP Server — Arcadia as a Tool Provider** *(original)*

Inverse of #111. Arcadia exposes its tools + indexer + memory + provenance as an MCP server.
Claude Desktop, Cursor, Cline, Zed, any MCP-aware client can use Arcadia as their backend.
Workspace permissions enforced server-side. Turns Arcadia into the universal AI infra layer
for other tools. No competitor ships this — they're all MCP clients only.

> **Prerequisites:** MCP Client (#111), Permission Profiles (#32), Audit Log (#31).

- Embedded MCP server on stdio (when launched as `arcadia mcp-serve`) + optional HTTP+SSE
- Exposed tools: `workspace.read`, `workspace.write` (gated), `index.search`, `index.semantic`,
  `memory.read`, `memory.write`, `forensic.investigate`, `git.diff`, `git.log`,
  `research.deep`, `compress.repo-map`
- Exposed resources: indexed file contents as `arcadia://workspace/<id>/<path>`; memory entries
  as `arcadia://memory/<id>`
- Authentication: bearer token from `~/Arcadia/Configuration/mcp-server-tokens.toml`; per-token
  permission profile
- Every external call audit-logged (#31) + provenance-signed (#87) with `mcp_client_id`
- Companion CLI: `arcadia mcp install-into claude-desktop` configures Claude Desktop to use
  Arcadia as MCP server (writes to `claude_desktop_config.json` with user consent)
- Distinct from #111: #111 is **consuming** external; #112 is **providing** to external

New file: `Shared/ArcadiaCore/src/modules/ai_mcp_server.rs`
Modified: `Desktop/src/cli/`, `config/`

---

**113. MCP Marketplace & Auto-Discovery** *(original)*

Browse, install, hot-reload MCP servers from inside Arcadia. Auto-detect MCP servers configured
in Claude Desktop, Cursor, Cline → offer one-click import. Signed manifests; permission preview
before install. Same trust model as Extension Marketplace (#41).

> **Prerequisites:** MCP Client (#111), Extension Marketplace (#41).

- Registry: hosted JSON index of MCP server manifests (similar to Homebrew formulae)
- Auto-import: scan `~/Library/Application Support/Claude/claude_desktop_config.json`, Cursor
  settings, Cline config; surface for one-click import with permission diff
- Install flow: clone manifest → spawn MCP server → validate `tools/list` against manifest →
  register with permission profile
- Hot reload: file watcher on MCP config dir; restart server on config change without restarting
  Arcadia
- GUI: Marketplace page filterable by category (db, web, code, comms, productivity, AI,
  scientific)
- Signing: server manifests SHA-256 + author signature verified before install
- Reputation: per-server install count + outcome rating from federated peers (#138)

New file: `Shared/ArcadiaCore/src/modules/ai_mcp_marketplace.rs`
Modified: `python_settings/` (renamed to extensions panel), `ai_marketplace.rs`

---

**114. MCP Health Monitor & Self-Healing** *(original)*

Each MCP server runs as a managed process. Track latency, error rate, memory, restarts.
Auto-restart on crash with exponential backoff. Quarantine misbehaving servers
(>N errors/minute). Visible in Observatory. Reuses Workspace Immune System (#61) anomaly
detection.

> **Prerequisites:** MCP Client (#111), AI Runtime Kernel (#56), Workspace Immune System (#61).

- `McpProcessHealth`: `pid`, `uptime`, `request_count`, `error_count`, `p50_latency`,
  `p99_latency`, `last_error`, `memory_rss`
- Auto-restart on crash: exponential backoff (1s, 2s, 4s, ..., 60s cap)
- Quarantine: >10 errors/min → quarantine for 5 min; user notified
- Resource quota: per-MCP RAM + CPU + outbound network limits via OS sandbox (`sandbox-exec`
  on macOS, seccomp on Linux)
- Observatory panel: per-MCP health card; click to inspect last N requests with payload
- Immune integration: extension-anomaly rules (#61) apply to MCP servers — unexpected file
  access outside workspace → immediate suspend

New file: `Shared/ArcadiaCore/src/modules/ai_mcp_health.rs`
Modified: `ai_mcp_client.rs`, `ai_kernel.rs`, `ai_observatory.rs`, `ai_immune.rs`

---

**115. Deep Research Engine — Iterative Search & Synthesise** *(Perplexity, Claude Deep
Research, OpenAI Deep Research)*

Multi-step research agent. Given a question, iteratively: formulate sub-queries → search →
fetch + clean → extract → identify gaps → search again. Cites everything. Surfaces source
disagreements. Time-boxed. Becomes a flagship feature alongside Flow Mode (#4) and
Orchestration (#11).

> **Prerequisites:** Context Providers (#12), Pluggable Search Backends (#116), Source Quality
> (#117), Multi-Agent Orchestration (#11).

- `ResearchPlan`: `question`, `sub_questions`, `sources_read`, `claims`, `citations`,
  `gaps_remaining`, `time_budget`, `token_budget`
- Loop bounded by: `max_iterations` (default 6), `max_time` (default 5 min),
  `max_token_spend`
- Sub-agent roles via Orchestration (#11): **Planner** (decomposes question), **Searcher**
  (queries backends), **Reader** (fetches + cleans pages), **Synthesizer** (writes report with
  citations), **Critic** (identifies gaps and contradictions)
- Output: structured report with inline citations, source disagreements made explicit, per-claim
  confidence
- Tools: `research.start(question, depth)`, `research.continue(plan_id)`,
  `research.export(plan_id, fmt)`
- Offline mode: works with cached doc packs (#118) + local index when internet unavailable
- Integrates with Reasoning Cache (#124): prior research chains reused for similar questions
- Constitutional self-critique (#123) applied to final synthesis before user sees it

New file: `Shared/ArcadiaCore/src/modules/ai_deep_research.rs`
Modified: `ai_orchestrator.rs`, `ai_tools.rs`

---

**116. Pluggable Search Backends** *(Continue.dev @search extended)*

Search provider abstraction. Configure any combination: Brave, Kagi, SearXNG (self-hosted),
Google CSE, DuckDuckGo, Tavily, Bing, Perplexity API. Failover chain; results merged +
deduplicated. SearXNG path enables 100%-self-hosted research with no third-party search APIs.

> **Prerequisites:** Context Providers (#12), Provider Fallback Chains (#30).

- `SearchBackend` trait: `query(text, max_results, time_filter, lang_filter) ->
  Result<Vec<SearchHit>>`
- Backends ship: **Brave** (default — free tier generous, privacy-respecting), **Kagi**
  (subscription), **SearXNG** (fully self-hosted, no API key needed), **Google CSE**,
  **DuckDuckGo** (HTML scrape fallback), **Tavily**, **Bing**, **Perplexity Sonar**
- Config in `ai-search.toml`; secrets loaded at point-of-use (CLAUDE.md rule)
- Failover: try primary; on rate-limit/error, fall through chain
- Result merging: cosine-deduplicate by title+snippet; keep highest-ranked from each backend
- `@search:query` mention uses configured chain; `@search:brave:query` overrides
- Latency target: median <800ms per backend; parallel queries across all configured backends
  for max recall

New files: `Shared/ArcadiaCore/src/modules/ai_search_backends.rs`,
`Shared/ArcadiaCore/src/config/ai_search.rs`
Modified: `ai_context_providers.rs`

---

**117. Source Quality & Citation Graph** *(original)*

Every research source scored: freshness, domain reputation, citation count from corpus,
contradiction with established memory. Citation graph visualised — see where claims trace back
to. Detects circular citations and citation cartels (low-quality sources citing each other to
manufacture trust).

> **Prerequisites:** Deep Research (#115), Workspace Indexer (#5).

- `SourceProfile`: `url`, `fetched_at`, `domain`, `last_modified`, `citations_in_corpus`,
  `trust_score`, `freshness_score`
- Domain reputation: small curated allowlist (peer-reviewed journals, official docs, RFCs,
  vendor-authoritative pages) + community lists; user can adjust
- Citation graph: nodes = sources, edges = "A cites B" / "A contradicts B"; visualised in
  Observatory
- Stale flag: source older than threshold (configurable, default 12 mo for tech docs) flagged
- Circular citation detection: A→B→C→A surfaced as low-confidence cluster
- "Cite consensus": when claim has 3+ independent sources with high trust, marked as
  high-confidence; otherwise flagged
- Distinct from `@docs` (#118): #117 scores web fetched results; #118 manages canonical doc
  packs

New file: `Shared/ArcadiaCore/src/modules/ai_source_quality.rs`
Modified: `ai_deep_research.rs`, `ai_observatory.rs`

---

**118. Version-Aware Documentation Fetcher** *(devdocs.io + Context7-style)*

First-class documentation as @mention with explicit version. `@docs:react@18.2`,
`@docs:tokio@1.40`, `@docs:swift@5.10`. Fetched once, signed, cached locally. Offline-first.
Auto-detect needed docs from project dep files → offer install.

> **Prerequisites:** Context Providers (#12), Workspace Indexer (#5).

- Doc packs: curated bundles per library/framework/standard; semantic-versioned
- Sources: devdocs.io archives, official doc tarballs, scraped + cleaned with content-script
  ablation
- Cached at `~/Arcadia/Configuration/ai-docs/<lib>@<version>/`; SHA-256 verified
- Search inside docs uses index Level 3 (#5 embeddings) over doc content; fast semantic search
  inside a single doc pack
- Auto-detect needed docs from `Cargo.toml` / `package.json` / `requirements.txt` /
  `go.mod` / `Gemfile` → offer install
- `@docs:<lib>@<version>:section` pulls specific section
- Multi-version coexistence: `@docs:react@18` and `@docs:react@19` simultaneously installable

New file: `Shared/ArcadiaCore/src/modules/ai_doc_packs.rs`
Modified: `ai_context_providers.rs`

---

**119. Specialised Research Connectors** *(Continue.dev + extended)*

First-class connectors for the canonical developer knowledge sources. Each is a context
provider that knows its source's API + content shape — far richer than generic web search.

> **Prerequisites:** Deep Research (#115), Context Providers (#12).

Connectors at launch:

- `@arxiv:query` — search arXiv abstracts + PDFs; full-text indexed locally
- `@github-code:query lang:rust` — GitHub code search; respects per-repo licence; cached
- `@stackoverflow:query` — SO with vote-weighted answers + accepted-answer highlight
- `@mdn:query` — MDN web docs (cached doc pack via #118)
- `@rfc:NNN` — IETF RFCs
- `@npm:package` — package metadata + README + recent versions + advisories
- `@crates:package` — crates.io equivalent
- `@pypi:package` — pypi equivalent
- `@hackernews:query` — HN search for developer chatter on a topic
- `@reddit:r/rust:query` — subreddit-scoped search (configurable subs)
- `@semantic-scholar:query` — academic paper search; alternative to arXiv for non-CS fields

All connectors: respect rate limits, cache aggressively, signed responses where source
supports, untrusted-content wrapped (#37) since they're external.

New file: `Shared/ArcadiaCore/src/modules/ai_research_connectors.rs`
Modified: `ai_context_providers.rs`

---

### Tier 18 — Deep Reasoning & Test-Time Compute

**120. Tree of Thoughts Engine** *(Yao et al. 2023)*

Branching reasoning with backtracking. For hard problems: model generates N candidate reasoning
branches, scores each, expands the most promising, prunes dead ends. Self-consistent answers
emerge from convergence across branches. Generalises Extended Thinking (#9) from linear to tree.

> **Prerequisites:** Extended Thinking Display (#9), Cross-Model Debate (#23), AI Kernel (#56).

- `ThoughtNode`: `id`, `parent`, `content`, `score`, `expanded`, `terminal`
- Search strategy: best-first with beam width B (default 3), max depth D (default 5),
  max_total_nodes N (default 30)
- Scoring: by self-critic prompt or by external verifier (LSP type-check, test runner,
  deterministic check)
- Backtrack: when a branch scores below threshold, return to highest-scoring unexpanded sibling
- Observable in Observatory as expandable tree; user can fork a thought manually
- Budget: tracked per dimension (tokens, time) via #40; falls back to linear CoT if budget tight
- Works fully offline with any reasoning-capable local model

New file: `Shared/ArcadiaCore/src/modules/ai_tree_of_thoughts.rs`
Modified: `ai_runtime.rs`, `ai_observatory.rs`

---

**121. Self-Consistency Voting** *(Wang et al. 2022)*

For factual / mathematical / decision tasks where reasoning chains can vary: sample N
independent reasoning paths, parse answers, pick majority. Boosts accuracy without changing
model; cost scales linearly with N.

> **Prerequisites:** Extended Thinking (#9), Test-Time Compute Scaling (#122).

- `SelfConsistencyRun`: `question`, `n_samples`, `temperature`, `answers[]`, `consensus`,
  `dispersion`
- Answer parser: per-task — JSON extractor, numeric extractor, multiple-choice extractor
- Dispersion metric: fraction of disagreement; high dispersion → user warned ("low confidence
  — 4 different answers")
- Cost gating: enabled per task class; disabled for trivial; auto-enabled for high-stakes
  (security, finance, math, architecture decisions)
- Plays well with Cross-Model Debate (#23): one model's self-consistency vs another's

New file: `Shared/ArcadiaCore/src/modules/ai_self_consistency.rs`
Modified: `ai_runtime.rs`

---

**122. Test-Time Compute Scaling** *(o1 / DeepSeek R1 inspired)*

Reasoning effort as an adaptive variable. Easy problems: minimal thinking. Hard problems: scale
thinking budget — more samples (#121), deeper trees (#120), longer CoT, more critique passes.
User-controllable slider; AI auto-detects difficulty.

> **Prerequisites:** Tree of Thoughts (#120), Self-Consistency (#121), Cognitive Load
> Detection (#48).

- `ReasoningBudget`: `max_thinking_tokens`, `max_samples`, `max_tree_depth`,
  `max_critique_passes`
- Auto-detection: lightweight classifier on prompt → difficulty estimate (`trivial` | `easy` |
  `medium` | `hard` | `expert`)
- User slider in chat header: 🐌 "deep" ↔ 🐇 "fast"; default: auto
- Display: "Thinking… (budget: 8k tokens, 3 samples, depth 4)" with live progress
- Energy-aware (#89): cap when on battery
- Reasoning budget reported in Action Preview (#133) before user commits

New file: `Shared/ArcadiaCore/src/modules/ai_compute_scaling.rs`
Modified: `ai_runtime.rs`, `ai_chat_panel.rs`

---

**123. Constitutional Self-Critique Loop** *(Anthropic Constitutional AI)*

Before emitting any response: model critiques its own draft against Constitution (#83) + active
rules + intent graph constraints; revises until clean or budget exhausted. Distinct from
Cross-Model Debate (#23) which uses two models; this is single-model self-revision.

> **Prerequisites:** Constitution (#83), Rules & Skills (#1), Test-Time Compute Scaling (#122).

- Two-pass: draft → critique → revise; repeated up to `max_revisions` (default 2)
- Critique prompt: "Review this draft against these principles: [Constitution]. List
  violations."
- If revision converges (no violations) → emit. If budget exhausted → emit with annotation
- Visible: "Self-critique: revised 2 times to address principle `offline-first`"
- Off by default for trivial; auto-on for: code edits, security domains, dependency changes,
  meta-AI actions, deep research outputs

New file: `Shared/ArcadiaCore/src/modules/ai_self_critique.rs`
Modified: `ai_runtime.rs`

---

**124. Reasoning Cache & Replay** *(original)*

Reasoning chains are valuable artifacts. Cache them. Replay similar problems by retrieving a
prior reasoning chain (RAG over CoTs) + adapting. Beyond caching responses — caching the
**thinking**. Compounds with every solved problem.

> **Prerequisites:** Workspace Indexer (#5 Level 3), Reproducible Runs (#39).

- `ReasoningChainEntry`: `input_summary`, `reasoning_steps`, `conclusion`, `embedding`,
  `success_outcome`
- Stored at `~/Arcadia/Configuration/ai-reasoning-cache/`
- On new query: embed → top-K similar chains → inject as `### Similar prior reasoning:`
  few-shot examples
- Cache eviction: low-success entries pruned; high-success entries promoted to skills (#1) via
  Meta-AI (#16)
- Cross-workspace cache (opt-in): reasoning chains shareable across federation peers (#105)
- Different from memory (#13): memory is facts; reasoning cache is **process** — how to think,
  not what to think

New file: `Shared/ArcadiaCore/src/modules/ai_reasoning_cache.rs`
Modified: `ai_runtime.rs`, `ai_memory.rs`

---

**125. Planner / Executor Split** *(Aider architect-coder + Claude planning)*

Separate the strong-reasoning model (plan) from the fast model (execute). Planner emits
structured plan + verification criteria. Executor (cheap, fast) implements each step. Verifier
checks step against criteria. Three-role orchestration; lowers cost dramatically while
preserving quality.

> **Prerequisites:** Multi-Agent Orchestration (#11), Structured Outputs (#28), Model
> Capability Router (#45).

- `Plan` schema: ordered steps, each with: `description`, `files_affected`,
  `success_criteria`, `rollback_plan`
- Planner role: assigned to high-reasoning model (Opus, GPT-5, DeepSeek R1)
- Executor role: assigned to fast model (Haiku, GPT-5-mini, Llama-3-8B)
- Verifier role: deterministic where possible (LSP type-check, test run); falls back to LLM
  verifier
- On verifier failure: executor retries with verifier feedback; if still fails, escalate to
  planner for replan
- Cost report: shows planner-tokens (expensive) + executor-tokens (cheap); typical savings
  60-80%

New file: `Shared/ArcadiaCore/src/modules/ai_planner_executor.rs`
Modified: `ai_orchestrator.rs`, `ai_capability_router.rs`

---

### Tier 19 — Onboarding, Mastery & Frictionless UX

**126. Zero-Config First Run** *(original ★★★)*

Out of the box: works offline immediately. Bundle a tiny capable model (~3B params, ~2GB
quantised) — TinyLlama, Phi-3-mini, Qwen2.5-1.5B-Instruct. On first launch: model already
extracted, ready. No API keys, no Ollama setup. "Just opened Arcadia → AI works." Beats every
competitor on first-run experience.

> **Prerequisites:** None. Foundation feature.

- Bundled model: `~/Arcadia/Models/default.gguf` (~2GB)
- On install: model extracted from app bundle / downloaded once via signed delta from
  GitHub Release
- llama.cpp runtime ships compiled with platform-optimal flags (Metal on macOS/iOS, AVX2 on
  Intel, CUDA where present)
- iOS: same bundled model via App Store (or download on first run if size budget tight); ANE
  Core ML variant (#86) preferred when available
- Default workspace: temp scratch workspace pre-created; user can immediately chat
- Onboarding flow only after user wants more: "Want a smarter model? Connect Ollama, OpenAI, or
  download X"
- Model auto-update: on Arcadia upgrade, prompt to upgrade bundled model with delta download
- Critical for "easiest to use" — no other AI shell ships ready-to-use offline AI without setup

New file: `Shared/ArcadiaCore/src/modules/ai_bundled_model.rs`
Modified: install scripts, `Mobile/iOS/ArcadiaApp/`, `entry.rs`

---

**127. Progressive Disclosure UI** *(original ★)*

New users see 5 buttons. Expert users see 50. UI complexity scales with user maturity (driven
by Curriculum Learning #94). Advanced features hidden behind a "show more" reveal that becomes
default once the user has used basics. Never patronising — clear "show everything" toggle
always available.

> **Prerequisites:** Curriculum Learning (#94).

- `UiComplexityLevel`: `beginner` | `intermediate` | `expert` | `everything`
- Auto-progression: based on demonstrated competence (used N features, completed N tasks);
  explicit user override always available
- Per-panel reveal: each panel has surface fields + deep fields; deep fields hidden at beginner
- Feature surfacing: when usage pattern suggests user is ready for X, gentle inline hint
  ("You've been doing this manually — want to know about Flow mode?")
- Distinguishes from hiding/removing features: nothing is permanently inaccessible; just out of
  initial view
- Power user shortcut: a single keypress (`Cmd+Shift+E`) toggles to `everything` mode

New file: `Shared/ArcadiaCore/src/modules/ai_ui_disclosure.rs`
Modified: every GUI panel under `Desktop/src/gui/app/`

---

**128. Interactive AI Tutorial — Hands-On Tour** *(original ★★)*

First-launch experience: a friendly agentic tour guide that walks the user through capabilities
by **doing them** in a sandbox workspace. "Let me show you how Flow mode works — I'll plan
something simple right now. Watch." Not a video. Not text. Actual interactive demo.
Re-runnable on request. The single biggest difference between "easy to install" and "easy to
master."

> **Prerequisites:** Practice Mode (#131), Flow Mode (#4).

- `TutorialChapter`: `title`, `narration`, `demonstration_steps`, `hands_on_task`,
  `completion_check`
- Chapters: chat basics, @mentions, rules/skills, Flow mode, checkpoints, observatory, MCP,
  deep research, orchestration, voice mode, sketch input
- Each chapter: AI narrates in chat + performs action visibly + invites user to try
- Progress tracking: per chapter complete; resumable
- Re-run any chapter from Help menu; "Show me again" mid-action
- Adaptive: skips chapters where user has already demonstrated competence (per Curriculum
  Learning #94)

New file: `Desktop/src/gui/app/ai_tutorial.rs`,
`Shared/ArcadiaCore/src/modules/ai_tutorial_content.rs`
Modified: `entry.rs` (first-launch detection)

---

**129. Universal Command Palette** *(VS Code Cmd+K / Raycast)*

Cmd+K (or Cmd+P) opens a fuzzy-searchable palette across **everything**: chats, files,
commands, MCP tools, memory entries, intent graph nodes, recent agent runs, settings. Type a
few characters → ranked results. Press Enter → executes / opens.

> **Prerequisites:** Workspace Indexer (#5), MCP Client (#111).

- Sources unified: file paths, command names, chat session titles, memory entry titles, MCP
  tool names, intent graph nodes, settings keys, recipes (#140)
- Fuzzy match: cosine over n-gram + acronym matching (e.g. "wapy" → "workspace.add Python")
- Recent + favourites boosted in ranking
- Plugin: extensions register palette providers (e.g. Linear extension surfaces Linear issues
  in palette via MCP)
- Keyboard-first; no mouse needed
- On iOS: long-press tab bar invokes palette
- Inline preview: highlighted result shows preview pane on hover/keyboard focus

New files: `Desktop/src/gui/app/ai_command_palette.rs`,
`Shared/ArcadiaCore/src/modules/ai_palette_index.rs`
Modified: `entry.rs`, `entry_ios.rs`

---

**130. Smart Defaults Engine** *(original)*

Defaults adapt to detected workflow. Rust dev → indexer Level 2 enabled, `cargo audit` on save,
conservative Permission Profile. ML researcher → arXiv connector active, Python tools
allowlisted, deep research opt-in. Detected from index Level 1 + initial chat content + open
file extensions.

> **Prerequisites:** Workspace Onboarding Wizard (#42), Workspace Indexer (#5).

- `WorkflowProfile`: detected from file extensions + repo content (Cargo.toml → Rust;
  package.json → web; pyproject.toml → Python; *.ipynb → ML/research)
- Profile bundles: enabled modules, default rules, default permission profile, default MCP
  servers, default search backend, default reasoning budget
- Built-in profiles: `rust-dev`, `web-frontend`, `web-backend`, `data-science`, `ml-research`,
  `mobile-ios`, `mobile-android`, `devops`, `security-research`, `writing`,
  `embedded-systems`, `game-dev`
- Override always available; user can pin a profile or customise
- Profile learns: confirmed preferences feed back into profile for that workflow class
- Shareable: workflow profiles publishable as `.arcprofile` for teams/community

New file: `Shared/ArcadiaCore/src/modules/ai_smart_defaults.rs`
Modified: `ai_onboarding_wizard.rs`

---

**131. Practice Mode / Sandbox Workspace** *(original)*

Ephemeral throwaway workspace. Full toolset. Zero risk. "Try Flow mode without worrying."
"Test that risky refactor agent on a fresh copy of your repo." Sandbox = git clone + checkpoint
+ tear-down after session.

> **Prerequisites:** Checkpoints (#6), Permission Profiles (#32).

- `SandboxSession`: clones source workspace (or empty template); created at
  `/tmp/arcadia-sandbox-<uuid>`
- All features available; same UI as real workspace; subtle "SANDBOX" badge in chrome
- Auto-cleanup: on session close, prompt save-as-real-workspace or discard
- Templates: empty Rust binary, empty React app, empty Python data-science notebook, etc., for
  quick experimentation
- "Try this on a copy" button anywhere an agent might do something dangerous
- Sandbox is the substrate for Interactive Tutorial (#128)

New files: `Desktop/src/gui/app/ai_sandbox_panel.rs`,
`Shared/ArcadiaCore/src/modules/ai_sandbox_workspace.rs`
Modified: `workspace.rs`

---

**132. Contextual Coach — Inline Help** *(original)*

A small assistant panel that explains the current mode, current selection, current action.
"You're in Flow mode. The plan has 5 steps. You can pause anytime with Esc." Aware of
cognitive state (#48); silent when user is focused. Distinct from chat — pure UX explanation.

> **Prerequisites:** Cognitive Load Detection (#48), Progressive Disclosure UI (#127).

- `CoachContext`: derived from active panel + active mode + recent user action + curriculum
  level
- Lookup: matches context → coach hint from catalog
- Hint sources: built-in catalog + AI-generated for unforeseen contexts
- Render: tiny chip near top-right of active panel; click to expand; long-press to disable for
  this context
- Silent during `Focused` cognitive state (#48); appears during `Exploratory` or `Stuck`
- "Why is this here?" right-click on any UI element → coach explains design intent

New file: `Desktop/src/gui/app/ai_coach.rs`
Modified: `ai_cognitive_state.rs`, every panel that wants coach support

---

**133. Cost & Difficulty Preview** *(original)*

Every significant action shows: estimated tokens, estimated time, estimated energy/joules (#89),
estimated cost in $ if cloud, difficulty meter (`trivial`/`easy`/`hard`/`expert`), risk score
(chance this needs rollback). User commits with full awareness. No more silent surprise costs.

> **Prerequisites:** Agent Budgeting (#40), Energy Budget (#89), Test-Time Compute
> Scaling (#122).

- `ActionPreview` computed before action submit:
  - Token estimate from prompt length × model factor
  - Time estimate from past similar runs
  - Difficulty from classifier (#122)
  - Risk from outcome history (#15) on similar actions
  - Cloud $ estimate from provider pricing × token estimate
  - Joule estimate from energy profile (#89)
- Display: compact strip above submit button; one-line summary by default; expandable to full
  breakdown
- Confirm: if cost/difficulty/risk crosses threshold, explicit "Proceed anyway?" required
- Skipped for trivial actions (chat reply, simple read)
- Post-action: actual vs predicted shown briefly; deviations refine future estimates

New file: `Shared/ArcadiaCore/src/modules/ai_action_preview.rs`
Modified: `ai_chat_panel.rs`, `ai_flow_panel.rs`

---

### Tier 20 — Anticipation, Collaboration & Profound Ideas

**134. Anticipatory Context Loading** *(original ★)*

AI predicts the next file/symbol/context the user (or active agent) will need; pre-loads +
caches it. Sub-50ms feel when you reach for it. Driven by edit history, navigation patterns,
dependency graph, and Ghost Mode (#22) intent hypothesis.

> **Prerequisites:** Ghost Mode (#22), Workspace Indexer (#5), Semantic Gravity (#67).

- `NextNeedPredictor`: trained on session navigation (file A opened, then file B opened within
  Δt)
- Pre-emptive: embed top-K predicted symbols/files into context-ready buffer before user asks
- Prefetch budget bounded (energy-aware via #89); cache invalidated on file edit
- Hit-rate metric in Observatory: "anticipated 73% of context needs in last session"
- Falls back gracefully if prediction wrong — cache miss = one normal load
- Pairs with Command Palette (#129): predicted-next items boosted in palette ranking

New file: `Shared/ArcadiaCore/src/modules/ai_anticipation.rs`
Modified: `ai_ghost.rs`, `ai_runtime.rs`

---

**135. AI Reading Mode — Codebase Audio Tour** *(original ★)*

AI reads the codebase aloud while highlighting code on screen. User listens, learns
architecture without active reading. Great for: onboarding new contributors, code review on
the go, accessibility, learning unfamiliar codebases during commute.

> **Prerequisites:** Voice Mode (#43), Semantic Compression Engine (#55), Living Architecture
> Narrative (#70).

- `ReadingScript`: generated tour over a module/subsystem; combines narrative (#70), code
  highlights, intent graph nodes (#47)
- TTS: macOS `AVSpeechSynthesizer`, iOS same, or premium voice via OpenAI/ElevenLabs (opt-in)
- Speeds: 0.75× ↔ 2×; bookmark + resume
- Sync: code panel scrolls + highlights as narration progresses
- Tour types: "module deep dive", "architecture overview", "recent changes briefing",
  "onboarding for new dev"
- iOS: AirPods + lock-screen controls; treat as podcast-grade UX
- Pause for questions: user can interject by voice; AI answers, returns to tour

New files: `Desktop/src/gui/app/ai_reading_mode.rs`,
`Shared/ArcadiaCore/src/modules/ai_audio_tour.rs`
Modified: `ai_voice_mode.rs`

---

**136. Voice Annotations Linked to Code** *(original)*

Speak a note over any line/function/symbol. AI transcribes, indexes, links to that symbol.
Future agents retrieve annotations as context. Like Loom for code but searchable and
AI-retrievable.

> **Prerequisites:** Voice Mode (#43), Workspace Indexer (#5), Advanced Memory (#13).

- `CodeAnnotation`: `symbol_ref`, `audio_path`, `transcription`, `author`, `timestamp`,
  `embedding`
- Recorded via mic button next to any symbol in code view; auto-stops on silence
- Transcription: local Whisper (offline) or premium STT
- Stored at `~/Arcadia/Configuration/ai-annotations/<workspace>/`; audio + JSON sidecar
- Retrieval: annotations injected into context when their symbol is referenced
- Search: `@annotation:search-text` returns matching notes with audio playback link

New file: `Shared/ArcadiaCore/src/modules/ai_voice_annotation.rs`
Modified: `ai_voice_mode.rs`, `ai_context.rs`

---

**137. Time-Slice Workspace Replay** *(original ★★)*

A scrubbable timeline of everything that happened in the workspace: file edits, agent runs,
chats, tool calls, terminal commands. Drag the slider; UI rolls back to that moment in time.
Watch a feature get built like a film. Combines audit log (#31), provenance (#87), and
time-travel index (#46) into a single visual surface.

> **Prerequisites:** Audit Log (#31), Reproducible Runs (#39), Time-Travel (#46), Hardware
> Provenance (#87).

- `WorkspaceTimeline`: append-only event stream from all activity sources
- Replay UI: timeline scrubber + filter (only edits / only AI / only chat / only terminal)
- Visual: at each moment, render the workspace + chat state as it was; agent runs play out
  token-by-token
- Speed: 0.25× ↔ 16×; jump to events ("jump to next file edit", "jump to next agent run")
- Sharing: export a time-slice as `.arctrace` file; collaborators can replay in their Arcadia
- Privacy: confidential workspaces (#88) have replay scoped to encrypted timeline; replays
  decrypted only inside Secure Enclave session

New files: `Desktop/src/gui/app/ai_timeline_replay.rs`,
`Shared/ArcadiaCore/src/modules/ai_timeline.rs`
Modified: `ai_audit_log.rs`, `ai_observatory.rs`

---

**138. Agent Skill Trading** *(original)*

Federation peers (#105) exchange skills, personalities, rule packs, agent definitions.
Reputation system: skills accumulate proven-success ratings from federated outcomes. Buy/sell
or share freely; signed manifests; permission preview before install.

> **Prerequisites:** Federated LoRA (#105), Extension Marketplace (#41), Rules & Skills (#1).

- `TradeableSkill`: skill manifest + outcome history (anonymised) + cryptographic signature
- Reputation: aggregated outcome success rate across federation, weighted by recency + adoption
  count
- Marketplace integration: skills appear in marketplace with reputation score
- Privacy: outcomes shared as differential-private aggregates only (never per-run details)
- Optional monetisation: extension authors can charge; Arcadia takes no cut
- Aligns with "best open-source AI platform" mission — community ecosystem with cryptographic
  trust

New file: `Shared/ArcadiaCore/src/modules/ai_skill_trading.rs`
Modified: `ai_marketplace.rs`, `ai_federated.rs`

---

**139. Live Collaborative Reasoning Canvas** *(original ★★)*

Multiple users + AI co-think in a structured spatial canvas. Each thought is a card; cards
connect by arrows. Users add cards (proposals, evidence, questions); AI adds cards (analysis,
contradictions found, suggestions). Real-time, multi-cursor, like Figma for thinking.
Generalises Cross-Model Debate (#23) and LAN Co-Pilot (#24).

> **Prerequisites:** LAN Co-Pilot (#24), Intent Graph (#47), Multi-Window Coherence (#98).

- `ReasoningCanvas`: persistent shared spatial scene; nodes = thoughts, edges = relationships
- Real-time sync via existing LAN protocol (or HTTP+SSE for internet collab)
- Node types: user proposal, AI analysis, evidence link, question, decision, blocker
- AI participants: contribute as named agents (e.g. "Security Critic", "Performance Optimiser")
- Outcomes: decisions promoted to Intent Graph (#47); blockers tracked
- Export: canvas as PDF / markdown / Intent Graph nodes
- Voice + sketch input: speak or draw onto the canvas; AI extracts nodes (combines #43 + #91)

New files: `Shared/ArcadiaCore/src/modules/ai_reasoning_canvas.rs`,
`Desktop/src/gui/app/ai_canvas_panel.rs`
Modified: `ai_copilot_session.rs`, `ai_intent_graph.rs`

---

**140. Workflow Recipes & Macros** *(original)*

Record any sequence of actions (chat turns, tool calls, approvals); save as a parameterised
recipe. Replay against new inputs. Share as `.arcrecipe` files. Generalises Slash Commands (#1)
and Skills with concrete state.

> **Prerequisites:** Rules & Skills (#1), Reproducible Runs (#39), Extension Marketplace (#41).

- `Recipe`: ordered steps with parameter placeholders; YAML or JSON serialised
- Recorder: "Start recording" button → captures every action with sanitisation (secrets
  stripped)
- Editor: visual step-list; parameters extracted from concrete values; reorder; insert
  conditional steps
- Replay: parameterised invocation (CLI or chat: "run recipe `deploy-staging` with
  branch=feature-x")
- Sharing: `.arcrecipe` files; marketplace integration; reputation per recipe
- Distinct from Skills (#1): recipes are concrete multi-step procedures; Skills are reusable
  behavioural fragments
- Trigger sources: CLI invocation, chat slash command, schedule (#92), button-binding,
  MCP server call (other tools can run Arcadia recipes via #112)

New file: `Shared/ArcadiaCore/src/modules/ai_recipes.rs`
Modified: `ai_skills.rs`, `ai_marketplace.rs`

---

## Architecture Prerequisites

| Prerequisite | Required by | Implementation |
|-------------|-------------|----------------|
| **Git Module** (#18) | #7, #35, #36, #46 | `modules/git.rs` + workspace permission gate |
| **InternalCommandBus** | #15, #16 | Trait in `modules/mod.rs`; opt-in per module |
| **Python `execute_command` bridge** | #17 | `python_host.rs` with permission gate |
| **`meta_accessible` flag on `ModuleCommand`** | #16 | Field addition + registry audit |
| **Permission Profiles** (#32) | #33, #34, #44 | Must precede all safety gate features |
| **Human Review Gates** (#33) | #44, #79 | Gate infrastructure before UI automation |
| **AI Runtime Kernel** (#56) | #58, #64, #65, #72, #75, #84 | Agent process model; replaces ad-hoc handles |
| **Hardware Provenance Chain** (#87) | #88, #99, #104, #137 | Secure Enclave / TPM signing |
| **MCP Client** (#111) | #112, #113, #114, #129 | stdio + HTTP+SSE transport; tool/resource bridge |
| **AST-Native VCS** (#100) | #101, #102, #103 | Tree-sitter parse reuse from index |
| **Pluggable Search Backends** (#116) | #115, #117, #119 | `SearchBackend` trait + failover chain |
| **Test-Time Compute Scaling** (#122) | #121, #123 | Reasoning budget primitive |
| **Curriculum Learning** (#94) | #109, #127, #128, #132 | User expertise model — drives adaptive UX |
| **Bundled Default Model** (#126) | first-run experience | ~2GB GGUF shipped + auto-update path |

---

## Full Build Order

| # | Target | Source | Unlocks |
|---|--------|--------|---------|
| 1 | Rules & Skills + defaults + per-chat UI ✅ | Cursor, Continue | composability contract |
| 2 | Persistent Chat + Context Window ✅ | Claude Projects | session continuity, token budget |
| 3 | Inline Diff Preview ✅ | **Cursor** ★ | safe multi-file AI edits |
| 4 | Flow Mode (Plan + Act) | **Windsurf Cascade**, Cline | autonomous operation |
| 5 | Workspace Indexer (3 levels + repo map) | **Aider**, Windsurf | smart context, memory retrieval |
| 6 | Checkpoints / Rollback | **Cline** | safety net for flow + orchestration |
| 7 | Git Module → AI Git Integration | **Aider**, Copilot | AI commits, PR, code review |
| 8 | Background Agents | **Cursor** | non-blocking tasks |
| 9 | Extended Thinking Display | **Claude** | transparency, trust |
| 10 | Advanced Context System | Windsurf, Continue | proactive workspace awareness |
| 11 | Multi-Agent Orchestration + Mindmap | Devin, Aider, Claude | flagship |
| 12 | Context Providers (@web, @issue, @pr…) | **Continue.dev** | live world state as context |
| 13 | Advanced Memory System | Claude Projects | cross-session expertise |
| 14 | Artifacts / Rendered Output | **Claude** | live HTML, diagrams, run button |
| 15 | Closed Loop Feedback + Self-Improvement | **(original)** | models learn; LoRA pipeline |
| 16 | Meta-AI — AI Extends Itself | **(original)** | AI mints tools, skills, agents |
| 17 | Python AI SDK | **(original)** | Python extends every AI layer |
| 18 | Git Module (prerequisite) | — | unblocks #7, #35, #36 |
| 19 | Quick Chat Popup | Raycast AI | AI anywhere, any hotkey |
| 20 | Project Knowledge Base | **Claude Projects** | per-project persistent AI context |
| 21 | AI Observatory + Advanced Visualization | **(original)** | education + trust |
| 22 | **Ghost Mode** — Ambient Intelligence | **(original ★★)** | passive context, always ready |
| 23 | **Cross-Model Debate** | **(original ★★)** | multi-model decisions + education |
| 24 | **LAN Co-Pilot** — Shared AI Sessions | **(original ★★)** | pair AI over LAN |
| 25 | Cross-Workspace Intelligence | **(original)** | pattern sharing across projects |
| 26 | CLI Exec Providers (claude/codex/gemini/aider) ✅ | **Codex exec mode** | subscription inference; no API key |
| 27 | Custom Tool Registry | — | extension tool packs |
| 28 | Structured Outputs + Validation | — | reliable agent data contracts |
| 29 | Prompt Caching | — | cost efficiency |
| 30 | Provider Fallback Chains + Health Checks | — | reliability under load |
| 31 | Audit Log | — | observability + compliance |
| 32 | Permission Profiles | — | safe onboarding; gates for advanced features |
| 33 | Human Review Gates | — | granular approval before risky actions |
| 34 | Secret & PII Guard | — | prevent accidental secret exposure |
| 35 | Test-Aware Agent | — | closed-loop test + fix |
| 36 | Dependency Intelligence | — | security advisories, upgrade plans |
| 37 | Prompt Injection Firewall | — | trust boundary for untrusted content |
| 38 | Local Model Benchmark Harness | — | capability profiles for router |
| 39 | Reproducible Agent Runs | — | debug, share, replay runs |
| 40 | Agent Cost & Resource Budgeting | — | local hardware + cloud cost control |
| 41 | Extension Marketplace / Registry | — | community ecosystem |
| 42 | Workspace Onboarding Wizard | — | zero-friction first run |
| 43 | Voice Mode for Coding | — | accessibility + iOS primary input |
| 44 | Visual UI Automation Provider | — | desktop GUI testing + non-code workflows |
| 45 | Model Capability Router | — | auto-assign best model per task |
| 46 | **Time-Travel Workspace Simulation** | **(original ★★)** | query historical repo state; architecture archaeology |
| 47 | **Intent Graph** — Memory of Why | **(original ★★)** | agents inherit reasoning, not just code |
| 48 | Cognitive Load Detection | **(original)** | AI adapts to operational state |
| 49 | AI Architecture Simulator | **(original ★)** | predict consequences before executing |
| 50 | **Semantic Undo** | **(original ★★)** | rollback AI decisions, not just files |
| 51 | AI Pair Personality System | **(original)** | operational collaboration styles |
| 52 | Live Context Heatmap | **(original ★)** | see what AI thinks is central |
| 53 | Contradiction Engine | **(original ★)** | autonomous consistency auditor |
| 54 | Runtime Learning Sandbox | **(original)** | AI develops operational intuition |
| 55 | **Semantic Compression Engine** | **(original ★★)** | massive codebases in minimal context |
| 56 | **AI Runtime Kernel** | **(original ★★★)** | OS for cognition; deepest long-term direction |
| 57 | Reality Gap Detector | **(original ★)** | what people think they built vs what exists |
| 58 | Autonomous Refactor Campaigns | **(original ★)** | weeks-long background technical debt reduction |
| 59 | Knowledge Distillation Between Models | **(original ★)** | strong model teaches local model |
| 60 | AI Theory-of-Mind Layer | **(original ★★)** | collaborative state modelling per participant |
| 61 | Workspace Immune System | **(original ★)** | AI platform defending itself from AI corruption |
| 62 | **Semantic Branching** | **(original ★★)** | conceptual branches; merge ideas not just code |
| 63 | AI Self-Model | **(original ★)** | inspectable AI confidence + knowledge provenance |
| 64 | Continuous Architectural Critique | **(original)** | background architectural governance |
| 65 | **Local AI Swarm Over LAN** | **(original ★★★)** | personal distributed AI infrastructure |
| 66 | **Dream Mode** — Offline Autonomous Reflection | **(original ★★)** | morning digest of overnight AI findings |
| 67 | Semantic Gravity System | **(original ★)** | architectural physics; fragile hotspot awareness |
| 68 | Counterfactual Coding | **(original ★★)** | simulate alternate architectural histories |
| 69 | Semantic Test Synthesis | **(original ★)** | invariant + adversarial + historical-class tests |
| 70 | Living Architecture Narrative | **(original ★★)** | project story for onboarding + archaeology |
| 71 | AI Memory Decay | **(original)** | entropy model; avoids stale hallucination |
| 72 | Speculative Execution Trees | **(original ★★)** | explore N futures before committing |
| 73 | AI-Native Refactoring Language | **(original ★★)** | declarative transformation DSL |
| 74 | Semantic Latency Optimiser | **(original)** | AI learns your interruption cost; batches gates |
| 75 | Multi-Resolution Context | **(original ★)** | simultaneous architecture + symbol reasoning |
| 76 | **Synthetic Senior Engineer** | **(original ★★★)** | persistent project-native engineering mind |
| 77 | Emergent Pattern Discovery | **(original ★)** | AI discovers hidden conventions + architectures |
| 78 | Semantic Merge Conflict Resolution | **(original ★★)** | intent-aware merge reconciliation |
| 79 | Trust-Adaptive Autonomy | **(original ★)** | earned autonomy per subsystem over time |
| 80 | Continuous Complexity Budgeting | **(original)** | measurable maintainability envelope |
| 81 | Architecture Immune Memory | **(original ★)** | bug-class antibodies; proactive recurrence scan |
| 82 | Semantic Profiling | **(original ★★)** | conceptual complexity + confusion profiler |
| 83 | AI Constitution Layer | **(original ★)** | enforceable project philosophy principles |
| 84 | Parallel Reality Agents | **(original ★★)** | multiple optimisation goals; user compares worlds |
| 85 | **Cognitive Diff View** | **(original ★★)** | diffs of intent, not just lines |
| 86 | **Apple Neural Engine / Core ML / MLX Provider** | **(original ★★)** | iOS-native inference; ANE + MLX paths |
| 87 | **Hardware Provenance Chain** | **(original ★★★)** | tamper-evident signed audit; regulated-industry unlock |
| 88 | Confidential Compute Context | **(original ★★)** | hardware-rooted context encryption |
| 89 | Energy & Thermal Budget | **(original ★)** | joule/thermal-aware agent planning |
| 90 | **Differential Privacy on Cloud Prompts** | **(original ★★★)** | ε-DP prompt perturbation; cloud quality + local privacy |
| 91 | **Embodied Input** — Sketch / Pencil / Camera-to-Code | **(original ★★)** | iPad-first visual input → code skeleton |
| 92 | AI Sleep Schedule | **(original ★)** | calendar-integrated agent wake |
| 93 | AI Pact Mode | **(original ★)** | session-level explicit safety contract |
| 94 | **Curriculum Learning** | **(original ★★)** | longitudinal user expertise model |
| 95 | **Stigmergy** — Pheromone Coordination | **(original ★★)** | biological multi-agent coordination |
| 96 | **Causal Inference Module** | **(original ★★)** | causal DAGs over commit/bug history |
| 97 | Conversational State Engine | **(original ★)** | within-session open-thread tracker |
| 98 | Multi-Window Cognitive Coherence | **(original ★)** | shared cognition across same-machine windows |
| 99 | AI Forensic Mode | **(original ★)** | incident-triggered RCA agent |
| 100 | **AST-Native Version Control Layer** | **(original ★★★)** | semantic VCS on top of git |
| 101 | **AI-Native Edit Stream Protocol** | **(original ★★)** | incremental AST ops; massive token savings |
| 102 | **Live LSP-Coupled Inference** | **(original ★★)** | type-check candidates inside generation loop |
| 103 | **Real-Time AST Conscience** | **(original ★★)** | keystroke-time invariant warnings |
| 104 | AI-Authored Code Provenance Manifest | **(original ★★)** | per-line authorship + cryptographic disclosure |
| 105 | **Federated LoRA Training** | **(original ★★★)** | privacy-preserving collective fine-tune |
| 106 | **Open Provider Protocol Specification** | **(original ★★)** | Arcadia-defined standard for AI providers |
| 107 | Code Genome / Style DNA Detection | **(original ★)** | workspace stylistic fingerprint; alien-commit flagging |
| 108 | AI Apologia / Failure Digest | **(original ★)** | synthesised weekly "what I got wrong" report |
| 109 | Compiler Diagnostic Translator | **(original ★)** | per-expertise-level translation of compiler errors |
| 110 | **MoE-Style Provider Mixture** | **(original ★★)** | token-by-token routing to specialist models |
| 111 | **MCP Client — Universal Tool Bridge** | **Anthropic MCP** | consume any MCP server; massive ecosystem unlock |
| 112 | **MCP Server — Arcadia as Provider** | **(original ★★★)** | Claude Desktop / Cursor / Cline can use Arcadia as backend |
| 113 | MCP Marketplace & Auto-Discovery | **(original ★)** | browse, install, hot-reload MCP servers |
| 114 | MCP Health Monitor & Self-Healing | **(original)** | managed MCP processes; quarantine misbehaviour |
| 115 | **Deep Research Engine** — Iterative Search & Synthesise | **Perplexity, Claude/OpenAI Deep Research** | flagship online research |
| 116 | Pluggable Search Backends | **Continue.dev extended** | Brave/Kagi/SearXNG/Google/DuckDuckGo/Tavily |
| 117 | Source Quality & Citation Graph | **(original ★)** | trust scoring + circular-citation detection |
| 118 | Version-Aware Documentation Fetcher | **devdocs / Context7** | `@docs:react@18.2` offline-first docs |
| 119 | Specialised Research Connectors | **Continue.dev extended** | arXiv/GitHub/SO/MDN/RFC/npm/crates/pypi |
| 120 | **Tree of Thoughts Engine** | **Yao et al. 2023** | branching reasoning with backtracking |
| 121 | Self-Consistency Voting | **Wang et al. 2022** | N reasoning paths → majority consensus |
| 122 | **Test-Time Compute Scaling** | **o1 / DeepSeek R1** | adaptive reasoning effort per problem |
| 123 | Constitutional Self-Critique Loop | **Anthropic Constitutional AI** | self-revision before emit |
| 124 | **Reasoning Cache & Replay** | **(original ★★)** | RAG over reasoning chains; compounds over time |
| 125 | Planner / Executor Split | **Aider architect-coder** | strong reasoner plans; cheap model executes |
| 126 | **Zero-Config First Run** | **(original ★★★)** | bundled local model; works offline immediately on install |
| 127 | Progressive Disclosure UI | **(original ★)** | UI complexity scales with user maturity |
| 128 | **Interactive AI Tutorial — Hands-On Tour** | **(original ★★)** | agentic tour guide in sandbox |
| 129 | **Universal Command Palette** | **VS Code / Raycast** | Cmd+K across everything |
| 130 | Smart Defaults Engine | **(original)** | per-workflow profile bundles |
| 131 | Practice Mode / Sandbox Workspace | **(original)** | ephemeral throwaway clone for risk-free trial |
| 132 | Contextual Coach — Inline Help | **(original)** | aware of cognitive state; silent when focused |
| 133 | Cost & Difficulty Preview | **(original)** | tokens + time + joules + risk before every action |
| 134 | Anticipatory Context Loading | **(original ★)** | predict next file/symbol; sub-50ms feel |
| 135 | AI Reading Mode — Codebase Audio Tour | **(original ★)** | podcast-grade architecture tour |
| 136 | Voice Annotations Linked to Code | **(original)** | Loom-for-code; searchable, AI-retrievable |
| 137 | **Time-Slice Workspace Replay** | **(original ★★)** | scrubbable timeline of all activity |
| 138 | Agent Skill Trading | **(original)** | federated skill marketplace with reputation |
| 139 | **Live Collaborative Reasoning Canvas** | **(original ★★)** | Figma-for-thinking with AI participants |
| 140 | Workflow Recipes & Macros | **(original)** | record/replay parameterised action sequences |
