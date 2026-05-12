use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use arcadia_core::config::{openai::OpenAiConfig, ConfigFile};
use arcadia_core::modules::ai_context;
use arcadia_core::modules::ai_tools;
use arcadia_core::modules::ai_types::TextGenerationRequest;

const HTTP_TIMEOUT: Duration = Duration::from_secs(120);

pub enum RuntimeEvent {
    Token(String),
    Done,
    Error(String),
}

pub enum AiRuntimeRequest {
    Generate {
        request: TextGenerationRequest,
        routing: ProviderRouting,
    },
    Shutdown,
}

pub enum ProviderRouting {
    LlamaCpp { model_path: String },
    Ollama   { endpoint: String, model_name: String },
    /// API key is NOT carried here — the inference thread loads it from config
    /// on demand so it never travels through the mpsc channel.
    OpenAi   { model_id: String },
}

pub struct AiRuntimeHandle {
    pub request_tx: SyncSender<AiRuntimeRequest>,
    pub event_rx:   Receiver<RuntimeEvent>,
    join_handle:    Option<std::thread::JoinHandle<()>>,
}

impl AiRuntimeHandle {
    pub fn start() -> Self {
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel(4);
        let (event_tx,   event_rx)   = std::sync::mpsc::sync_channel(512);
        let join_handle = std::thread::Builder::new()
            .name("ai-inference".to_string())
            .spawn(move || ai_thread_impl(request_rx, event_tx))
            .expect("failed to spawn ai inference thread");
        AiRuntimeHandle { request_tx, event_rx, join_handle: Some(join_handle) }
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
                ProviderRouting::Ollama { endpoint, model_name } => {
                    run_ollama(&endpoint, &model_name, request, &tx);
                }
                ProviderRouting::OpenAi { model_id } => {
                    run_openai(&model_id, request, &tx);
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

/// Build the effective system prompt: inject file context from @mentions,
/// then append tool definitions if tools are present.
fn prepare_system(request: &TextGenerationRequest) -> Result<String, String> {
    let mut system = request.system.clone();

    // File context injection.
    if let Some(ctx) = &request.workspace_context {
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

    // Tool definition injection.
    if !request.tools.is_empty() {
        let tool_json = ai_tools::render_tool_definitions(&request.tools);
        system = format!(
            "{system}\n\nYou have access to tools. To call a tool, emit a ```json block with this shape:\n```json\n{{\"tool_calls\":[{{\"name\":\"<tool_name>\",\"arguments\":{{...}}}}]}}\n```\nAvailable tools:\n{tool_json}"
        );
    }

    Ok(system)
}

/// Run a provider's single-shot generate fn in a tool-use loop.
/// `generate` takes (system, messages, max_tokens) and streams tokens via tx,
/// returning the assembled assistant turn as a String.
fn tool_loop<F>(request: &TextGenerationRequest, tx: &SyncSender<RuntimeEvent>, mut generate: F)
where
    F: FnMut(&str, &[(String, String)], i32, &SyncSender<RuntimeEvent>) -> Option<String>,
{
    let system = match prepare_system(request) {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.send(RuntimeEvent::Error(format!("Context: {e}")));
            return;
        }
    };

    let mut messages = request.messages.clone();
    const MAX_ROUNDS: usize = 8;

    for round in 0..MAX_ROUNDS {
        let Some(output) = generate(&system, &messages, request.max_tokens, tx) else {
            return; // generate already sent Error
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
            let result = ai_tools::execute_tool(call, request.workspace_context.as_ref());
            if result.is_error {
                results.push_str(&format!("Tool `{}` error: {}\n\n", result.name, result.output));
            } else {
                results.push_str(&format!("Tool `{}` result:\n{}\n\n", result.name, result.output));
            }
        }

        // Feed results back as next turn.
        messages.push(("assistant".to_string(), output));
        messages.push(("tool_result".to_string(), results));

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
        let response = match agent.post(&url)
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
    model_id: &str,
    request: TextGenerationRequest,
    tx: &SyncSender<RuntimeEvent>,
) {
    let api_key = OpenAiConfig::load_or_create()
        .map(|c| c.api_key)
        .unwrap_or_default();
    if api_key.is_empty() {
        let _ = tx.send(RuntimeEvent::Error(
            "OpenAI API key not configured. Add it in Settings → AI.".to_string(),
        ));
        return;
    }
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

        let response = match agent.post("https://api.openai.com/v1/chat/completions")
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
    let loaded_ref = &loaded.as_ref().unwrap().1;

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
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(n_ctx as u32));

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
