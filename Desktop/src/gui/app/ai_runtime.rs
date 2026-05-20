use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use arcadia_core::config::{
    ai_rules::{all_rules, AiRulesConfig},
    ai_skills::{all_skills, AiSkillsConfig},
    openai::OpenAiConfig,
    ConfigFile,
};
use arcadia_core::modules::ai_context;
use arcadia_core::modules::ai_tools;
use arcadia_core::ai_types::{ImageGenerationRequest, TextGenerationRequest};
use std::path::PathBuf;

const HTTP_TIMEOUT: Duration = Duration::from_secs(120);

pub enum RuntimeEvent {
    Token(String),
    Done,
    Error(String),
    /// AI proposed a file edit that was staged instead of written to disk.
    PendingEdit {
        path: String,
        original: String,
        proposed: String,
    },
    /// Image generation produced a PNG at the given absolute path.
    Image(PathBuf),
}

pub enum AiRuntimeRequest {
    Generate {
        request: TextGenerationRequest,
        routing: ProviderRouting,
    },
    GenerateImage {
        request: ImageGenerationRequest,
        routing: ImageProviderRouting,
    },
    Shutdown,
}

pub enum ProviderRouting {
    LlamaCpp {
        model_path: String,
    },
    Ollama {
        endpoint: String,
        model_name: String,
    },
    /// API key and base URL are NOT carried here — the inference thread loads
    /// them from config on demand so credentials never travel through the channel.
    OpenAi {
        provider_id: String,
        model_id: String,
    },
    /// Spawn an installed AI CLI binary (claude, codex, gemini, aider, …).
    /// Prompt is delivered via stdin pipe — no shell interpolation of user content.
    ExecCli {
        binary: String,
        model_flag: Option<String>,
        model_value: String,
    },
    /// Apple Intelligence via macOS Foundation Models framework (macOS 26+).
    /// No model selection — the system model is used.
    Apfel,
}

pub enum ImageProviderRouting {
    /// Apple ImagePlayground via macOS ImageCreator framework (macOS 15.2+).
    /// On-device, no API key, no network.
    ImagePlayground,
}

pub struct AiRuntimeHandle {
    pub request_tx: SyncSender<AiRuntimeRequest>,
    pub event_rx: Receiver<RuntimeEvent>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl AiRuntimeHandle {
    pub fn start() -> Self {
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel(4);
        let (event_tx, event_rx) = std::sync::mpsc::sync_channel(512);
        let join_handle = std::thread::Builder::new()
            .name("ai-inference".to_string())
            .spawn(move || ai_thread_impl(request_rx, event_tx))
            .expect("failed to spawn ai inference thread");
        AiRuntimeHandle {
            request_tx,
            event_rx,
            join_handle: Some(join_handle),
        }
    }
}

impl Drop for AiRuntimeHandle {
    fn drop(&mut self) {
        let _ = self.request_tx.send(AiRuntimeRequest::Shutdown);
        if let Some(h) = self.join_handle.take() {
            let _ = h.join();
        }
    }
}

fn ai_thread_impl(rx: Receiver<AiRuntimeRequest>, tx: SyncSender<RuntimeEvent>) {
    #[cfg(feature = "llama-cpp")]
    let backend = match llama_cpp_2::llama_backend::LlamaBackend::init() {
        Ok(b) => Some(b),
        Err(e) => {
            eprintln!("llama.cpp backend init failed: {e}");
            None
        }
    };

    #[cfg(feature = "llama-cpp")]
    let mut loaded: Option<(String, llama_cpp_2::model::LlamaModel)> = None;

    while let Ok(req) = rx.recv() {
        match req {
            AiRuntimeRequest::Shutdown => break,
            AiRuntimeRequest::Generate { request, routing } => match routing {
                ProviderRouting::LlamaCpp { model_path } => {
                    #[cfg(feature = "llama-cpp")]
                    {
                        if let Some(ref backend) = backend {
                            run_llama_cpp(backend, &mut loaded, &model_path, request, &tx);
                        } else {
                            let _ = tx.send(RuntimeEvent::Error(
                                "llama.cpp backend failed to initialize".to_string(),
                            ));
                        }
                    }
                    #[cfg(not(feature = "llama-cpp"))]
                    {
                        let _ = request;
                        let _ = model_path;
                        let _ = tx.send(RuntimeEvent::Error(
                            "llama.cpp not compiled into this build.".to_string(),
                        ));
                    }
                }
                ProviderRouting::Ollama {
                    endpoint,
                    model_name,
                } => {
                    run_ollama(&endpoint, &model_name, request, &tx);
                }
                ProviderRouting::OpenAi {
                    provider_id,
                    model_id,
                } => {
                    run_openai(&provider_id, &model_id, request, &tx);
                }
                ProviderRouting::ExecCli {
                    binary,
                    model_flag,
                    model_value,
                } => {
                    run_exec_cli(&binary, model_flag.as_deref(), &model_value, request, &tx);
                }
                ProviderRouting::Apfel => {
                    run_apfel(request, &tx);
                }
            },
            AiRuntimeRequest::GenerateImage { request, routing } => match routing {
                ImageProviderRouting::ImagePlayground => {
                    run_image_playground(request, &tx);
                }
            },
        }
    }

    #[cfg(feature = "llama-cpp")]
    {
        drop(loaded);
        drop(backend);
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

struct PreparedSystem {
    system: String,
    effective_max_tokens: i32,
    /// Tool names blocked by active rules (never executed, model told they failed).
    forbidden_tool_names: Vec<String>,
}

/// Build the effective system prompt: inject rules/skills fragments, @mention
/// file context, then append tool definitions (minus forbidden tools).
fn prepare_system(request: &TextGenerationRequest) -> Result<PreparedSystem, String> {
    let mut system = request.system.clone();
    let mut effective_max_tokens = request.max_tokens;
    let mut forbidden_tool_names: Vec<String> = Vec::new();
    let mut skill_allowed_tools: Option<Vec<String>> = None;

    // Rules & Skills injection.
    if !request.active_rule_ids.is_empty() || !request.active_skill_ids.is_empty() {
        let rules_cfg = AiRulesConfig::load_or_create().unwrap_or_default();
        let skills_cfg = AiSkillsConfig::load_or_create().unwrap_or_default();
        let all_r = all_rules(&rules_cfg);
        let all_s = all_skills(&skills_cfg);

        for id in &request.active_rule_ids {
            if let Some(rule) = all_r.iter().find(|r| &r.id == id) {
                system = format!("{system}\n\n{}", rule.system_fragment);
                forbidden_tool_names.extend(rule.forbidden_tools.iter().cloned());
                if let Some(mt) = rule.max_tokens_override {
                    effective_max_tokens = effective_max_tokens.min(mt);
                }
            }
        }

        for id in &request.active_skill_ids {
            if let Some(skill) = all_s.iter().find(|s| &s.id == id) {
                system = format!("{system}\n\n{}", skill.system_fragment);
                if let Some(mt) = skill.max_tokens_override {
                    effective_max_tokens = effective_max_tokens.min(mt);
                }
                if !skill.allowed_tools.is_empty() {
                    let allowed = skill_allowed_tools.get_or_insert_with(Vec::new);
                    for t in &skill.allowed_tools {
                        if !allowed.contains(t) {
                            allowed.push(t.clone());
                        }
                    }
                }
            }
        }

        // Any tool not in the skill allow-list is effectively forbidden.
        if let Some(ref allowed) = skill_allowed_tools {
            for tool in &request.tools {
                if !allowed.iter().any(|a| a == tool.name)
                    && !forbidden_tool_names.iter().any(|f| f == tool.name)
                {
                    forbidden_tool_names.push(tool.name.to_string());
                }
            }
        }
    }

    // Workspace context injection.
    if let Some(ctx) = &request.workspace_context {
        let perms: Vec<&str> = [
            ("workspace.read", "read"),
            ("workspace.write", "write"),
            ("workspace.execute", "execute"),
        ]
        .iter()
        .filter(|(id, _)| ctx.granted_permissions.iter().any(|p| p == id))
        .map(|(_, label)| *label)
        .collect();
        let perm_str = if perms.is_empty() {
            "none".to_string()
        } else {
            perms.join(", ")
        };
        system = format!(
            "{system}\n\nYou are scoped to a workspace. Your working directory is: {path}\nWorkspace name: {label}\nGranted permissions: {perm_str}\nUse this path as the root for all file operations. You do not need to ask the user for the path — it is already set.\nWhen asked to explore, review, or work with code, immediately use list_files or read_file to examine the workspace contents rather than asking the user to specify files or directories.",
            label = ctx.workspace_label,
            path = ctx.workspace_path,
        );

        // File context injection from @mentions.
        let last_user = request
            .messages
            .last()
            .filter(|(role, _)| role == "user")
            .map(|(_, c)| c.as_str())
            .unwrap_or("");
        let mentions = ai_context::parse_file_mentions(last_user);
        if !mentions.is_empty() {
            let file_block = ai_context::build_file_context(&mentions, ctx)?;
            if !file_block.is_empty() {
                system = format!("{system}\n\n{file_block}");
            }
        }
    }

    // Tool definition injection — omit forbidden tools so model never sees them.
    let visible_tools: Vec<_> = request
        .tools
        .iter()
        .filter(|t| !forbidden_tool_names.iter().any(|f| f == t.name))
        .cloned()
        .collect();

    if !visible_tools.is_empty() {
        let tool_json = ai_tools::render_tool_definitions(&visible_tools);
        system = format!(
            "{system}\n\nYou have access to tools. To call a tool, emit a ```json block with this shape:\n```json\n{{\"tool_calls\":[{{\"name\":\"<tool_name>\",\"arguments\":{{...}}}}]}}\n```\nAvailable tools:\n{tool_json}"
        );
    }

    Ok(PreparedSystem {
        system,
        effective_max_tokens,
        forbidden_tool_names,
    })
}

/// Run a provider's single-shot generate fn in a tool-use loop.
fn tool_loop<F>(request: &TextGenerationRequest, tx: &SyncSender<RuntimeEvent>, mut generate: F)
where
    F: FnMut(&str, &[(String, String)], i32, &SyncSender<RuntimeEvent>) -> Option<String>,
{
    let prepared = match prepare_system(request) {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(RuntimeEvent::Error(format!("Context: {e}")));
            return;
        }
    };

    let mut messages = request.messages.clone();
    const MAX_ROUNDS: usize = 8;

    for round in 0..MAX_ROUNDS {
        let Some(output) = generate(
            &prepared.system,
            &messages,
            prepared.effective_max_tokens,
            tx,
        ) else {
            return;
        };

        if request.tools.is_empty() {
            let _ = tx.send(RuntimeEvent::Done);
            return;
        }

        let calls = ai_tools::parse_tool_calls(&output);
        if calls.is_empty() {
            let _ = tx.send(RuntimeEvent::Done);
            return;
        }

        // Execute tools and collect results.
        let mut results = String::new();
        for call in &calls {
            // Block forbidden tools — model sees an error result without execution.
            if prepared
                .forbidden_tool_names
                .iter()
                .any(|f| f == &call.name)
            {
                results.push_str(&format!(
                    "Tool `{}` error: this tool is disabled by an active rule.\n\n",
                    call.name
                ));
                continue;
            }

            let result = ai_tools::execute_tool(
                call,
                request.workspace_context.as_ref(),
                request.stage_writes,
            );

            // Detect staged writes and emit PendingEdit events.
            if result.output.starts_with("STAGED\n") {
                if let Some(rest) = result.output.strip_prefix("STAGED\n") {
                    if let Some((original, proposed)) = rest.split_once("\n---\n") {
                        let path = call.arguments["path"].as_str().unwrap_or("").to_string();
                        let _ = tx.send(RuntimeEvent::PendingEdit {
                            path,
                            original: original.to_string(),
                            proposed: proposed.to_string(),
                        });
                    }
                }
                results.push_str(&format!(
                    "Tool `{}` result:\nEdit staged for review. The user will apply it.\n\n",
                    result.name
                ));
                continue;
            }

            if result.is_error {
                results.push_str(&format!(
                    "Tool `{}` error: {}\n\n",
                    result.name, result.output
                ));
            } else {
                results.push_str(&format!(
                    "Tool `{}` result:\n{}\n\n",
                    result.name, result.output
                ));
            }
        }

        // Feed results back as next turn.
        messages.push(("assistant".to_string(), output));
        messages.push(("user".to_string(), results));

        if round == MAX_ROUNDS - 1 {
            let _ = tx.send(RuntimeEvent::Error("Max tool rounds reached".to_string()));
        }
    }
}

// ── Ollama HTTP provider ──────────────────────────────────────────────────────

fn run_ollama(
    endpoint: &str,
    model_name: &str,
    request: TextGenerationRequest,
    tx: &SyncSender<RuntimeEvent>,
) {
    let endpoint = endpoint.to_string();
    let model_name = model_name.to_string();
    let agent = ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build();

    tool_loop(&request, tx, |system, messages, _max_tokens, tx| {
        use std::io::{BufRead, BufReader};

        let mut body_messages: Vec<serde_json::Value> = Vec::new();
        if !system.trim().is_empty() {
            body_messages.push(serde_json::json!({ "role": "system", "content": system.trim() }));
        }
        for (role, content) in messages {
            body_messages.push(serde_json::json!({ "role": role, "content": content }));
        }

        let body = serde_json::json!({
            "model": model_name,
            "messages": body_messages,
            "stream": true,
        });

        let url = format!("{}/api/chat", endpoint.trim_end_matches('/'));
        let response = match agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_json(&body)
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("Ollama: {e}")));
                return None;
            }
        };

        let reader = BufReader::new(response.into_reader());
        let mut assembled = String::new();
        for line in reader.lines() {
            let line = match line {
                Ok(l) if !l.trim().is_empty() => l,
                _ => continue,
            };
            let val: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(content) = val["message"]["content"].as_str() {
                if !content.is_empty() {
                    let _ = tx.send(RuntimeEvent::Token(content.to_string()));
                    assembled.push_str(content);
                }
            }
            if val["done"].as_bool().unwrap_or(false) {
                break;
            }
        }
        Some(assembled)
    });
}

// ── OpenAI HTTP provider ──────────────────────────────────────────────────────

fn run_openai(
    provider_id: &str,
    model_id: &str,
    request: TextGenerationRequest,
    tx: &SyncSender<RuntimeEvent>,
) {
    let cfg = OpenAiConfig::load_or_create().unwrap_or_default();
    let provider = match cfg.providers.iter().find(|p| p.id == provider_id) {
        Some(p) => p.clone(),
        None => {
            let _ = tx.send(RuntimeEvent::Error(format!(
                "OpenAI provider '{provider_id}' not found in config."
            )));
            return;
        }
    };
    if provider.api_key.is_empty() {
        let _ = tx.send(RuntimeEvent::Error(
            "OpenAI API key not configured. Edit the provider in Settings → Models.".to_string(),
        ));
        return;
    }
    let api_key = provider.api_key.clone();
    let base_url = if provider.base_url.is_empty() {
        "https://api.openai.com".to_string()
    } else {
        provider.base_url.trim_end_matches('/').to_string()
    };
    let model_id = model_id.to_string();
    let agent = ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build();

    tool_loop(&request, tx, |system, messages, max_tokens, tx| {
        use std::io::{BufRead, BufReader};

        let mut body_messages: Vec<serde_json::Value> = Vec::new();
        if !system.trim().is_empty() {
            body_messages.push(serde_json::json!({ "role": "system", "content": system.trim() }));
        }
        for (role, content) in messages {
            body_messages.push(serde_json::json!({ "role": role, "content": content }));
        }

        let body = serde_json::json!({
            "model": model_id,
            "messages": body_messages,
            "stream": true,
            "max_tokens": max_tokens,
        });

        let url = format!("{base_url}/v1/chat/completions");
        let response = match agent
            .post(&url)
            .set("Authorization", &format!("Bearer {api_key}"))
            .set("Content-Type", "application/json")
            .send_json(&body)
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("OpenAI: {e}")));
                return None;
            }
        };

        let reader = BufReader::new(response.into_reader());
        let mut assembled = String::new();
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let data = match line.strip_prefix("data: ") {
                Some(d) => d.trim(),
                None => continue,
            };
            if data == "[DONE]" {
                break;
            }
            let val: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(content) = val["choices"][0]["delta"]["content"].as_str() {
                if !content.is_empty() {
                    let _ = tx.send(RuntimeEvent::Token(content.to_string()));
                    assembled.push_str(content);
                }
            }
        }
        Some(assembled)
    });
}

// ── llama.cpp local provider ──────────────────────────────────────────────────

#[cfg(feature = "llama-cpp")]
fn run_llama_cpp(
    backend: &llama_cpp_2::llama_backend::LlamaBackend,
    loaded: &mut Option<(String, llama_cpp_2::model::LlamaModel)>,
    model_path: &str,
    request: TextGenerationRequest,
    tx: &SyncSender<RuntimeEvent>,
) {
    #[allow(deprecated)]
    use llama_cpp_2::{
        context::params::LlamaContextParams,
        llama_batch::LlamaBatch,
        model::{params::LlamaModelParams, AddBos, LlamaChatMessage, LlamaModel, Special},
        sampling::LlamaSampler,
    };

    // Lazy model load — only reload when path changes.
    let needs_load = loaded
        .as_ref()
        .map(|(p, _)| p.as_str() != model_path)
        .unwrap_or(true);

    if needs_load {
        let params = LlamaModelParams::default().with_n_gpu_layers(99999);
        match LlamaModel::load_from_file(backend, model_path, &params) {
            Ok(model) => *loaded = Some((model_path.to_string(), model)),
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("Load failed: {e}")));
                return;
            }
        }
    }

    // tool_loop handles context injection, tool definition injection, and multi-turn tool use.
    // The closure runs a single generation turn and returns the assembled output string.
    let backend_ref = &*backend;
    let loaded_ref = match loaded.as_ref() {
        Some((_, model)) => model,
        None => {
            let _ = tx.send(RuntimeEvent::Error("Model failed to load".to_string()));
            return;
        }
    };

    tool_loop(&request, tx, |system, messages, n_predict, tx| {
        let model = loaded_ref;

        let mut chat_messages: Vec<LlamaChatMessage> = Vec::new();
        if !system.trim().is_empty() {
            match LlamaChatMessage::new("system".into(), system.trim().into()) {
                Ok(m) => chat_messages.push(m),
                Err(e) => {
                    let _ = tx.send(RuntimeEvent::Error(format!("Chat message: {e}")));
                    return None;
                }
            }
        }
        for (role, content) in messages {
            match LlamaChatMessage::new(role.clone(), content.trim().into()) {
                Ok(m) => chat_messages.push(m),
                Err(e) => {
                    let _ = tx.send(RuntimeEvent::Error(format!("Chat message: {e}")));
                    return None;
                }
            }
        }

        let tmpl = match model.chat_template(None) {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("Chat template: {e}")));
                return None;
            }
        };
        let prompt = match model.apply_chat_template(&tmpl, &chat_messages, true) {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("Chat template: {e}")));
                return None;
            }
        };

        let tokens = match model.str_to_token(&prompt, AddBos::Never) {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("Tokenize: {e}")));
                return None;
            }
        };

        let n_ctx = std::cmp::max(tokens.len() + n_predict as usize + 64, 4096);
        let ctx_params =
            LlamaContextParams::default().with_n_ctx(std::num::NonZeroU32::new(n_ctx as u32));

        let mut ctx = match model.new_context(backend_ref, ctx_params) {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("Context: {e}")));
                return None;
            }
        };

        let n_tokens = tokens.len();
        let mut batch = LlamaBatch::new(n_tokens.max(1), 1);
        let mut batch_err = false;
        for (i, &token) in tokens.iter().enumerate() {
            if batch.add(token, i as i32, &[0], i == n_tokens - 1).is_err() {
                batch_err = true;
                break;
            }
        }
        if batch_err {
            let _ = tx.send(RuntimeEvent::Error("Batch build failed".to_string()));
            return None;
        }

        if let Err(e) = ctx.decode(&mut batch) {
            let _ = tx.send(RuntimeEvent::Error(format!("Decode: {e}")));
            return None;
        }

        let mut n_cur = batch.n_tokens();
        let mut sampler = LlamaSampler::greedy();
        let mut assembled = String::new();

        'gen: for _ in 0..n_predict {
            let idx = batch.n_tokens() as i32 - 1;
            let token = sampler.sample(&ctx, idx);
            sampler.accept(token);

            if model.is_eog_token(token) {
                break;
            }

            #[allow(deprecated)]
            if let Ok(s) = model.token_to_str(token, Special::Plaintext) {
                if !s.is_empty() {
                    if tx.send(RuntimeEvent::Token(s.clone())).is_err() {
                        break 'gen;
                    }
                    assembled.push_str(&s);
                }
            }

            batch.clear();
            if batch.add(token, n_cur, &[0], true).is_err() {
                break;
            }
            if ctx.decode(&mut batch).is_err() {
                break;
            }
            n_cur += 1;
        }

        Some(assembled)
    });
}

// ── CLI exec provider ─────────────────────────────────────────────────────────

fn run_exec_cli(
    binary: &str,
    model_flag: Option<&str>,
    model_value: &str,
    request: TextGenerationRequest,
    tx: &SyncSender<RuntimeEvent>,
) {
    use arcadia_core::modules::ai_sandbox::is_binary_allowed;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    use std::time::Instant;

    if !is_binary_allowed(binary) {
        let _ = tx.send(RuntimeEvent::Error(format!(
            "CLI exec denied: '{binary}' is not in the allowed binary list."
        )));
        return;
    }

    let binary = binary.to_string();
    let model_flag = model_flag.map(str::to_string);
    let model_value = model_value.to_string();

    tool_loop(&request, tx, move |system, messages, _max_tokens, tx| {
        // Build a plain-text prompt from the current system + message history.
        // Each tool-loop round appends tool results as user turns so the CLI
        // sees the full context on re-invocation.
        let mut prompt = String::with_capacity(1024);
        if !system.is_empty() {
            prompt.push_str("System: ");
            prompt.push_str(system);
            prompt.push_str("\n\n");
        }
        for (role, content) in messages {
            let label = if role == "user" { "User" } else { "Assistant" };
            prompt.push_str(label);
            prompt.push_str(": ");
            prompt.push_str(content);
            prompt.push_str("\n\n");
        }
        prompt.push_str("Assistant:");

        let mut cmd = Command::new(&binary);
        if let Some(ref flag) = model_flag {
            if !model_value.is_empty() {
                cmd.arg(flag).arg(&model_value);
            }
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!(
                    "Failed to spawn '{binary}': {e}"
                )));
                return None;
            }
        };

        let prompt_bytes = prompt.into_bytes();
        let stdin_handle = {
            let mut stdin = child.stdin.take();
            std::thread::spawn(move || {
                if let Some(ref mut s) = stdin {
                    let _ = s.write_all(&prompt_bytes);
                }
            })
        };

        let deadline = Instant::now() + HTTP_TIMEOUT;
        let mut output = String::new();
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = stdin_handle.join();
                    let _ = tx.send(RuntimeEvent::Error(
                        "CLI exec timed out after 120s.".to_string(),
                    ));
                    return None;
                }
                match line {
                    Ok(l) => {
                        output.push_str(&l);
                        output.push('\n');
                    }
                    Err(_) => break,
                }
            }
        }

        let _ = stdin_handle.join();
        let status = child.wait();

        let text = output.trim().to_string();
        if text.is_empty() {
            let code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
            let _ = tx.send(RuntimeEvent::Error(format!(
                "'{binary}' produced no output (exit code {code}). \
                 Check that the CLI is logged in and supports non-interactive stdin mode."
            )));
            None
        } else {
            let _ = tx.send(RuntimeEvent::Token(text.clone()));
            Some(text)
        }
    });
}

// ── Apple Intelligence provider (macOS Foundation Models) ─────────────────────

fn run_apfel(request: TextGenerationRequest, tx: &SyncSender<RuntimeEvent>) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Write the Swift runner to a temp file. The script uses Foundation Models
    // (available macOS 26+) and streams output to stdout.
    let swift_src = r#"
import FoundationModels
import Foundation

let promptData = FileHandle.standardInput.readDataToEndOfFile()
let prompt = String(data: promptData, encoding: .utf8) ?? ""

Task {
    do {
        let session = LanguageModelSession()
        let stream = try session.streamResponse(to: prompt)
        var prevLen = 0
        for try await fragment in stream {
            let full = fragment.content
            let delta = String(full.dropFirst(prevLen))
            if !delta.isEmpty {
                print(delta, terminator: "")
                fflush(stdout)
            }
            prevLen = full.count
        }
        print("")
    } catch {
        fputs("Apfel error: \(error)\n", stderr)
    }
    exit(0)
}

RunLoop.main.run()
"#;

    let tmp_path = std::env::temp_dir().join("arcadia_apfel_runner.swift");
    if let Err(e) = std::fs::write(&tmp_path, swift_src) {
        let _ = tx.send(RuntimeEvent::Error(format!(
            "Failed to write Apfel runner: {e}"
        )));
        return;
    }

    tool_loop(&request, tx, move |system, messages, _max_tokens, tx| {
        let mut prompt = String::with_capacity(1024);
        if !system.is_empty() {
            prompt.push_str("System: ");
            prompt.push_str(system);
            prompt.push_str("\n\n");
        }
        for (role, content) in messages {
            let label = if role == "user" { "User" } else { "Assistant" };
            prompt.push_str(label);
            prompt.push_str(": ");
            prompt.push_str(content);
            prompt.push_str("\n\n");
        }
        prompt.push_str("Assistant:");

        let mut child = match Command::new("swift")
            .arg(tmp_path.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!("Failed to spawn swift: {e}")));
                return None;
            }
        };

        let prompt_bytes = prompt.into_bytes();
        let stdin_handle = {
            let mut stdin = child.stdin.take();
            std::thread::spawn(move || {
                if let Some(ref mut s) = stdin {
                    let _ = s.write_all(&prompt_bytes);
                }
            })
        };

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        let mut output = String::new();

        if let Some(stdout) = child.stdout.take() {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines() {
                if std::time::Instant::now() > deadline {
                    let _ = tx.send(RuntimeEvent::Error(
                        "Apfel timed out after 120 s.".to_string(),
                    ));
                    let _ = child.kill();
                    return None;
                }
                match line {
                    Ok(l) => {
                        let _ = tx.send(RuntimeEvent::Token(l.clone()));
                        output.push_str(&l);
                        output.push('\n');
                    }
                    Err(_) => break,
                }
            }
        }

        let _ = stdin_handle.join();
        let _ = child.wait();

        let text = output.trim().to_string();
        if text.is_empty() {
            let _ = tx.send(RuntimeEvent::Error(
                "Apple Intelligence returned no output. \
                 Requires macOS 26 or later with Foundation Models available."
                    .to_string(),
            ));
            None
        } else {
            Some(text)
        }
    });
}

// ── Apple ImagePlayground provider (macOS ImageCreator) ───────────────────────

fn run_image_playground(request: ImageGenerationRequest, tx: &SyncSender<RuntimeEvent>) {
    use arcadia_core::config::image_playground::{ImagePlaygroundConfig, ImagePlaygroundStyle};
    use std::process::{Command, Stdio};
    use std::time::{SystemTime, UNIX_EPOCH};

    if request.prompt.trim().is_empty() {
        let _ = tx.send(RuntimeEvent::Error(
            "ImagePlayground prompt is empty.".to_string(),
        ));
        return;
    }

    // Resolve style: request override → config default → animation.
    let cfg = ImagePlaygroundConfig::load_or_create().unwrap_or_default();
    let style_token = request
        .style
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| cfg.default_style.as_swift_token().to_string());
    // Validate against the known set so the Swift runner never sees garbage.
    let style_token = match style_token.as_str() {
        "animation" => ImagePlaygroundStyle::Animation.as_swift_token(),
        "illustration" => ImagePlaygroundStyle::Illustration.as_swift_token(),
        "sketch" => ImagePlaygroundStyle::Sketch.as_swift_token(),
        _ => ImagePlaygroundStyle::Animation.as_swift_token(),
    };

    // Resolve output path under the cache dir.
    let cache_dir = match cfg.resolve_cache_dir() {
        Ok(d) => d,
        Err(e) => {
            let _ = tx.send(RuntimeEvent::Error(format!(
                "ImagePlayground cache dir: {e}"
            )));
            return;
        }
    };
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let pid = std::process::id();
    let out_path = cache_dir.join(format!("ip_{stamp}_{pid}.png"));

    // The Swift runner reads the prompt from stdin, takes style + output path
    // via argv. It exits 0 with the output path on stdout on success, or with
    // a non-zero status and an error on stderr.
    let swift_src = r#"
import Foundation
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

#if canImport(ImagePlayground)
import ImagePlayground

@available(macOS 15.2, *)
func run(prompt: String, styleToken: String, outPath: String) async -> Int32 {
    let style: ImagePlaygroundStyle
    switch styleToken {
    case "illustration": style = .illustration
    case "sketch":       style = .sketch
    default:             style = .animation
    }
    do {
        let creator = try await ImageCreator()
        let concepts: [ImagePlaygroundConcept] = [.text(prompt)]
        let stream = creator.images(for: concepts, style: style, limit: 1)
        for try await image in stream {
            let cg = image.cgImage
            let url = URL(fileURLWithPath: outPath)
            guard let dest = CGImageDestinationCreateWithURL(
                url as CFURL,
                UTType.png.identifier as CFString,
                1,
                nil
            ) else {
                FileHandle.standardError.write(Data("Failed to create PNG destination\n".utf8))
                return 2
            }
            CGImageDestinationAddImage(dest, cg, nil)
            if !CGImageDestinationFinalize(dest) {
                FileHandle.standardError.write(Data("Failed to finalize PNG\n".utf8))
                return 2
            }
            print(outPath)
            return 0
        }
        FileHandle.standardError.write(Data("ImagePlayground produced no image\n".utf8))
        return 3
    } catch {
        FileHandle.standardError.write(Data("ImagePlayground error: \(error)\n".utf8))
        return 4
    }
}

let args = CommandLine.arguments
guard args.count >= 3 else {
    FileHandle.standardError.write(Data("usage: runner <style> <out_path>\n".utf8))
    exit(64)
}
let styleToken = args[1]
let outPath = args[2]
let promptData = FileHandle.standardInput.readDataToEndOfFile()
let prompt = String(data: promptData, encoding: .utf8) ?? ""

if #available(macOS 15.2, *) {
    let sem = DispatchSemaphore(value: 0)
    var status: Int32 = 1
    Task {
        status = await run(prompt: prompt, styleToken: styleToken, outPath: outPath)
        sem.signal()
    }
    sem.wait()
    exit(status)
} else {
    FileHandle.standardError.write(Data("ImagePlayground requires macOS 15.2 or later\n".utf8))
    exit(5)
}

#else
FileHandle.standardError.write(Data("ImagePlayground framework not available on this system\n".utf8))
exit(6)
#endif
"#;

    let tmp_path = std::env::temp_dir().join("arcadia_image_playground_runner.swift");
    if let Err(e) = std::fs::write(&tmp_path, swift_src) {
        let _ = tx.send(RuntimeEvent::Error(format!(
            "Failed to write ImagePlayground runner: {e}"
        )));
        return;
    }

    let mut child = match Command::new("swift")
        .arg(tmp_path.as_os_str())
        .arg(style_token)
        .arg(&out_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(RuntimeEvent::Error(format!("Failed to spawn swift: {e}")));
            return;
        }
    };

    // Send the user prompt over stdin in a background thread.
    let prompt_bytes = request.prompt.clone().into_bytes();
    let stdin_handle = {
        use std::io::Write;
        let mut stdin = child.stdin.take();
        std::thread::spawn(move || {
            if let Some(ref mut s) = stdin {
                let _ = s.write_all(&prompt_bytes);
            }
        })
    };

    // Deadline guard. ImageCreator typically returns within ~10 s on M-series hardware.
    let deadline = std::time::Instant::now() + HTTP_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if std::time::Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = stdin_handle.join();
                    let _ = tx.send(RuntimeEvent::Error(
                        "ImagePlayground timed out after 120 s.".to_string(),
                    ));
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                let _ = tx.send(RuntimeEvent::Error(format!(
                    "ImagePlayground runner wait: {e}"
                )));
                return;
            }
        }
    }
    let _ = stdin_handle.join();

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            let _ = tx.send(RuntimeEvent::Error(format!(
                "ImagePlayground runner output: {e}"
            )));
            return;
        }
    };

    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let msg = if stderr_text.is_empty() {
            format!(
                "ImagePlayground runner failed (exit {}). \
                 Requires macOS 15.2 or later with Apple Intelligence enabled.",
                output.status.code().unwrap_or(-1),
            )
        } else {
            stderr_text
        };
        let _ = tx.send(RuntimeEvent::Error(msg));
        return;
    }

    // Sanity-check the file actually exists.
    if !out_path.exists() {
        let _ = tx.send(RuntimeEvent::Error(
            "ImagePlayground reported success but no PNG was written.".to_string(),
        ));
        return;
    }

    let _ = tx.send(RuntimeEvent::Image(out_path));
    let _ = tx.send(RuntimeEvent::Done);
}
