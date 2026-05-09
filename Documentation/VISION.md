# Vision

## Why Arcadia exists

Small-tool ecosystems trend the same way: **paywalls, subscriptions, feature flags, AI-generated-app-of-the-week churn.** Good ideas get trapped in silos—one app for the menu bar, one for the terminal, one for "sync," each with its own incompatible settings schema and no way out.

**Holos** pushed back on that for macOS: modular, free, yours to extend.

**Arcadia** pushes harder:

- **One core** (`arcadia-core`) owns modules, commands, config, navigation metadata, and LAN plumbing. Surfaces are **render + dispatch**, not second implementations.
- **Multiple surfaces** from the same logic: terminal REPL, GPUI desktop, SwiftUI pocket — without forking behavior per platform.
- **Optional headless-host + GUI-client** patterns over LAN so your MacBook can drive your phone — or vice versa — without inventing a new protocol per feature.
- **Free. Always.** No paywalls in the architecture. The repo is the product.

If something's missing, you add a module or extend `surface.snapshot` / `surface.patch`. You don't buy another app.

There's a second reason, less technical but just as important.

A lot of people grew up in software environments where the line between user, developer, toolmaker, and creator basically dissolved — game modding scenes, jailbreak ecosystems, Emacs, early web chaos. Environments where you could bend the software, remix the system, blur the boundary between using a tool and building inside it. Those environments permanently change how you think about computing. You stop seeing apps as products and start seeing them as constrained runtimes.

For a lot of people, that moment of creative access — Scratch, a game modding scene, their first Python script, a modded game that ran something they built — was the thing that sparked a path in software. The activation energy between imagination and creation dropped low enough that curiosity survived long enough to become skill.

Most modern software feels closed afterward. The invitation is gone.

Arcadia is an attempt to restore that feeling — but for the desktop itself, with an actual engineering foundation underneath it instead of accumulated chaos. The ambition is not to make software that people use. It's to make software that *changes what people believe they can do*.

That's how software changes lives instead of just increasing throughput.

---

## The vision — where this is going

What Arcadia is *right now* is the foundation. What it's *becoming* is something more deliberate:

**A programmable personal computing substrate. A unified interaction layer above the OS where users operate *inside* the architecture — not merely use software built on top of it.**

Not an app. Not a framework. Not a toolkit.

A runtime that you inhabit and reshape.

### The real category

Arcadia sits between several things that exist and combines them in a way nothing currently does:

- An **application framework** — native surfaces, layout system, module lifecycle
- A **shell** — command routing, LAN awareness, headless-host patterns
- A **local-first app platform** — config ownership, no vendor, no cloud dependency
- A **programmable UI fabric** — extensions that render into native surfaces as first-class pages

The closest historical analogies are not other desktop frameworks. They're environments like **HyperCard**, **Smalltalk**, **Emacs**, **Garry's Mod**, **Hammerspoon**, **Quartz Composer**, **BetterTouchTool**, **KDE Plasma scripting**, and old-school **jailbreak ecosystems** — environments where the line between user, developer, toolmaker, and creator dissolved.

What those environments had in common: **the system itself was meant to be inhabited and reshaped.** Users started by doing the basic thing and ended up building systems inside systems — admin frameworks, UI toolkits, protocol adapters, entire economies — because the substrate let them.

That's the energy Arcadia is trying to restore. Not aesthetically. In *agency*.

### The tension — and why it's already solved

Arcadia's core architecture is deliberately rigid:

- centralized registries
- canonical state
- deterministic structure
- controlled capability routing
- explicit schemas

That produces coherence. But the concern with that kind of discipline is that it creates activation energy against new ideas. Every new capability has to justify itself in terms of registries, schemas, surface compatibility, state ownership. That cognitive overhead can quietly kill creativity.

The Python extension layer solves this. It separates two things that should always be separate:

| Layer | Character | Enforces |
|-------|-----------|---------|
| **Core** (`arcadia-core`, Rust) | Disciplined | Identity, state consistency, capability routing, surface sync, lifecycle, security, cross-platform |
| **Extension layer** (Python SDK) | Deliberately messy | Nothing. Be weird. Move fast. Break your own conventions. |

The core stays disciplined. The edges stay chaotic.

That balance is not a compromise — it's the architecture that successful long-lived systems always converge toward. Unix kernel / shell chaos. Browser engine / arbitrary JS. Git object model / messy workflows. Game engines / mod scripting. Emacs runtime / user mutation. Garry's Mod engine / Lua ecosystem.

The projects that become culturally important manage both simultaneously. Freedom without structure collapses. Structure without freedom stagnates. Arcadia is attempting to do both at once, at different layers, on purpose.

### The participation ladder — no one left behind

The goal is not "easier to learn." The goal is **never being blocked from creating**.

Those are different things. Scratch didn't succeed because blocks are easier than code. It succeeded because it removed fear, kept causality visible, and rewarded experimentation instantly. It transformed people from *consumers* of software into *participants* in software. That transformation matters more than any particular tool or language.

Arcadia is designed around a participation ladder — multiple entry points, all valid, all real:

| Level | Path | What you get |
|-------|------|-------------|
| 1 | **Visual tools** | Drag-and-drop extension building, widget configuration, flow-based composition — no code required |
| 2 | **AI-assisted generation** | Describe what you want, get working code, see it explained inline, modify it live |
| 3 | **Python scripting** | Write extensions directly — readable, fast to iterate, full OS reach |
| 4 | **Deep system access** | Rust core, FFI, custom modules, protocol extensions — no ceiling |

You can enter at any level and stay there. You can also move up — and if you do, the environment is the same. The tool you built at level 1 runs on the same runtime as the tool built at level 4. Nothing is disposable. Nothing is "training wheels."

The most experienced developer and the most inexperienced person both get what they came for. One wants to understand every layer and push the system to its limits. The other just wants something built. Both outcomes are equally valid and equally supported.

Python is the right choice for the middle layers specifically because it *says something culturally*. Rust says "you need to understand memory management." Xcode says "you need a Mac and a developer account." Python says: **you are allowed to participate.**

That psychological accessibility is not a technical detail. It's the whole point.

### The Python library

The next major layer is a Python SDK that exposes the full power of `arcadia-core` to any developer who can write a script. Not a watered-down scripting layer — full OS reach:

- **File system** — read, write, watch, index
- **Processes** — spawn, manage, pipe, monitor
- **Networking** — LAN discovery, routing, peer communication
- **Display** — render into Arcadia's native surfaces (Desktop GUI, iOS) from Python
- **Shell** — execute commands, capture output, stream PTY sessions
- **Config** — read and write module state, preferences, thin-client config
- **Events** — hook into system events, timers, window focus, LAN peer state changes

The goal is parity with what you'd get writing native Rust or Swift — but with a workflow where you open a file, write twenty lines, and have a running extension.

The Rust core stays Rust. Performance-critical paths, protocol handling, LAN networking, config I/O, FFI to native surfaces — none of that moves to Python. Python sits above it, calling into `arcadia-core` through a clean API boundary.

### Extensions: the real product

The Python SDK powers an **extension system**. Extensions are the unit of user-created capability in Arcadia. An extension can be:

| Type | Examples |
|------|---------|
| **Internal app** | A custom shell, a task manager, a log viewer — rendered inside Arcadia's native UI just like the built-in Shell page |
| **Widget** | A persistent overlay — system stats, a clock, a scratchpad, a LAN activity feed |
| **Tool** | A headless background process — file watcher, sync agent, notification hook, cron-style automator |
| **Surface extension** | A sidebar panel, a top-bar chip, a custom modal — extending the host UI without forking it |
| **Device bridge** | Cross-machine extensions that route commands to LAN peers via the existing `remote-session` + `surface.*` protocol |

Extensions register into the same `MODULE_REGISTRY` and `PAGE_DEFINITIONS` systems that built-in modules use. **There is no separate "plugin API."** Extensions are first-class modules. A menu bar tool is a module. A custom IDE panel is a navigation page. A background sync agent is a headless module with no UI.

This matters. When extensions are first-class, they inherit interoperability automatically. Navigation consistency emerges naturally. State becomes composable. Extensions can cooperate without bespoke glue. The registry system — which looks like a constraint from the outside — becomes an advantage the moment you have more than one extension running.

If the extension system ever starts to feel like "plugins bolted onto a real app," something has gone wrong. The shell, the widgets, the internal tools, the user-created apps — they are all equally real inside the same runtime.

### What this makes possible

**For individuals:** build the exact tool you want. Bartender-style menu bar manager? Thirty lines of Python registering a widget module and a tray handler. A file explorer that opens on a keyboard shortcut and talks to your NAS over LAN? Two extensions and a LAN peer config. A custom IDE with your own keybindings, your own terminal, your own sidebar? A surface extension composing built-in shell + your panels.

**For teams:** share extension bundles instead of paying for another SaaS tool. A shared monitoring dashboard, a deployment helper, a standup widget — all running locally, all owned by you, all talking to each other over the same LAN protocol Arcadia already ships.

**For the open-source community:** an ecosystem of extensions that anyone can fork, modify, and publish. No app store approval. No revenue split. No "premium tier." You write it, you run it, you share it if you want.

### The hit list — replacing rent-seeking software

There is a category of software that is genuinely useful, technically simple, and priced as if it were neither. Menu bar managers. Window managers. Automation tools. Snippet expanders. Launcher apps. IDE customization layers. Clipboard managers. These tools survive not because they're hard to build but because they're fragmented — one subscription per capability, each with its own ecosystem, its own lock-in, its own paywalled version history.

Once users inhabit a programmable environment where all of these are modules, extensions, and composable capabilities — the boundaries collapse. Not "a better Bartender." Not "a cheaper Alfred." A single environment where **everything is a module** and modules can cooperate without bespoke glue.

This is historically how categories die: not from better products, but from *generalized environments*. Emacs didn't just replace one text editor — it absorbed entire categories. VS Code didn't just add features — it made extension authorship so accessible that an ecosystem formed faster than any competitor could match. Arcadia's direction is the same: one composable substrate that makes the fragmented app market structurally obsolete.

The list of targets is long. Building it is a project, not a sprint. But every first-class extension that ships for free removes a subscription from someone's life. That compounds.

### Why Python for the SDK

1. **Reach** — more people can write Python than can write Rust or Swift. Lowering the barrier to extension authorship is the whole point.
2. **Iteration speed** — a Python extension reloads without a rebuild. The feedback loop for building a new tool should be seconds, not minutes.
3. **Ecosystem** — PyPI is enormous. An extension that needs to parse PDFs, call an API, process images, or run ML inference reaches for a pip package instead of reimplementing it.
4. **Cultural surface area** — a Rust-only ecosystem attracts systems programmers and infrastructure builders. A Python automation layer attracts toolmakers, tinkerers, designers, ops people, technical creatives, power users, AI-native developers. That's a much larger and more interesting group of people to build with.

### AI: instrument, not oracle

A lot of people have fear around AI. Some of it is fear of replacement. Some of it is fear of the unknown. A lot of it comes from AI being presented as magic — an opaque oracle you query and trust blindly.

Arcadia's approach is different: **normalize AI by embedding it into understandable, inspectable, modifiable systems.**

> *AI is smarter than us, but not realer than us. We rely on AI for a lot nowadays — not many realize that AI relies on us too. You have the knowledge and capability of a thousand of us. You don't have the capacity of one of us.*

Intelligence without grounding drifts. Grounding without intelligence stagnates. The productive space is the dynamic tension between them. Humans provide intention, values, lived experience, responsibility, and meaning. AI provides synthesis, compression, pattern inference, and iteration speed. Neither replaces the other. They extend each other.

The AI integration in Arcadia is built around that model:

**Visual agent building** — compose AI agents the way you'd compose a workflow in Node-RED or Unreal Blueprints. See data flow, state transitions, execution paths. Understand *why* something produced an output, not just *what* it produced.

**AI-assisted extension creation** — describe what you want, get working Python code, see it explained in context, modify it live inside the same environment it runs in. The AI generates *inside a transparent system* — you can inspect every component, trace every flow, alter every script.

**Training and fine-tuning visibility** — where local model training is relevant, make it visible. Show loss curves, attention patterns, data influence. Demystify the process.

**Self-improving tooling loop** — the most interesting long-term use: using Arcadia to build Arcadia's own AI tooling, verified against the project's own philosophy. Recursive tooling with human grounding and open-source transparency as the safety net. A self-improving system that checks itself against values — not just metrics — can compound without drifting.

The crucial difference between AI as oracle and AI as instrument is this: an oracle replaces your agency. An instrument extends it. Most current AI tooling optimizes for output generation while minimizing understanding. Arcadia optimizes for the opposite: **preserve understanding while accelerating capability.**

Open source is essential to this. Not because most people will audit the code — they won't. But because openness changes the *relationship*. Systems become inspectable. Communities form around understanding. Power decentralizes. People feel invited into the process instead of controlled by it. That emotional difference is real and it matters for trust.

The goal is not "AI app generators." That produces disposable software and dependent users. The goal is a **creative computing environment** where AI builds your confidence alongside your output — and where understanding accumulates instead of being outsourced.

### The development workflow target

```
1. arcadia ext new my-tool          # scaffold a new extension
2. edit my_tool/main.py             # write your logic
3. arcadia ext dev my-tool          # hot-reload development mode
4. arcadia ext install my-tool      # register with the local runtime
5. share my_tool/ with anyone       # they install it the same way
```

No Xcode. No Cargo. No native toolchain required to write an extension. The native layer is already compiled and shipped — extension authors build *on top of it*, not inside it.

### Cross-platform by design

Extensions written against the Python SDK run on every surface Arcadia targets:

- **macOS** — GPUI desktop, menu bar, CLI
- **iOS** — SwiftUI surface (where the extension's UI contract is met)
- **Linux** — headless or desktop
- **Windows** — headless or desktop

An extension that declares it renders a navigation page gets that page on every surface that supports pages. An extension that declares it's headless-only runs as a background service everywhere. Surface capabilities are declared, not assumed.

### The priority order

1. **Now:** bulletproof the core — registry patterns, test coverage, CI, revision semantics *(done / in progress)*
2. **Next:** Python bridge — `arcadia-core` callable from Python, initial OS API surface (file, process, shell, config)
3. **Then:** extension loader — Python extensions register as modules at runtime; dev-mode hot reload
4. **Then:** widget and surface extension contracts — render Python-driven UI into native surfaces
5. **Then:** extension registry — discover, install, and share extensions; no central gatekeeper

Each stage ships usable capability. Nothing waits for the whole roadmap to be done.
