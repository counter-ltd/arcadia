use std::sync::mpsc::{Receiver, SyncSender};

use arcadia_core::modules::ai_types::TextGenerationRequest;

pub enum RuntimeRequest {
    Infer {
        model_path: String,
        request: TextGenerationRequest,
    },
    Shutdown,
}

pub enum RuntimeEvent {
    Token(String),
    Done,
    Error(String),
}

pub struct LlamaCppRuntime {
    pub request_tx: SyncSender<RuntimeRequest>,
    pub event_rx: Receiver<RuntimeEvent>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl LlamaCppRuntime {
    pub fn start() -> Self {
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel::<RuntimeRequest>(4);
        let (event_tx, event_rx) = std::sync::mpsc::sync_channel::<RuntimeEvent>(512);

        let join_handle = std::thread::Builder::new()
            .name("llama-inference".to_string())
            .spawn(move || {
                #[cfg(feature = "llama-cpp")]
                inference_thread_impl(request_rx, event_tx);

                #[cfg(not(feature = "llama-cpp"))]
                {
                    while let Ok(req) = request_rx.recv() {
                        match req {
                            RuntimeRequest::Infer { .. } => {
                                let _ = event_tx.send(RuntimeEvent::Error(
                                    "llama.cpp not compiled into this build.".to_string(),
                                ));
                            }
                            RuntimeRequest::Shutdown => break,
                        }
                    }
                }
            })
            .expect("failed to spawn inference thread");

        LlamaCppRuntime { request_tx, event_rx, join_handle: Some(join_handle) }
    }
}

impl Drop for LlamaCppRuntime {
    fn drop(&mut self) {
        let _ = self.request_tx.send(RuntimeRequest::Shutdown);
        if let Some(h) = self.join_handle.take() {
            let _ = h.join();
        }
    }
}

#[cfg(feature = "llama-cpp")]
fn inference_thread_impl(
    request_rx: Receiver<RuntimeRequest>,
    event_tx: SyncSender<RuntimeEvent>,
) {
    use llama_cpp_2::{
        context::params::LlamaContextParams,
        llama_backend::LlamaBackend,
        llama_batch::LlamaBatch,
        model::{params::LlamaModelParams, AddBos, LlamaChatMessage, LlamaChatTemplate, LlamaModel, Special},
        sampling::LlamaSampler,
    };

    let backend = match LlamaBackend::init() {
        Ok(b) => b,
        Err(e) => {
            let _ = event_tx.send(RuntimeEvent::Error(format!("llama.cpp init: {e}")));
            return;
        }
    };

    let mut loaded: Option<(String, LlamaModel)> = None;

    while let Ok(req) = request_rx.recv() {
        match req {
            RuntimeRequest::Shutdown => break,
            RuntimeRequest::Infer { model_path, request } => {
                let system = request.system;
                let messages = request.messages;
                let n_predict = request.max_tokens;
                // Reload model only when path changes.
                let needs_load = loaded
                    .as_ref()
                    .map(|(p, _)| p.as_str() != model_path.as_str())
                    .unwrap_or(true);

                if needs_load {
                    let params = LlamaModelParams::default().with_n_gpu_layers(99999);
                    match LlamaModel::load_from_file(&backend, &model_path, &params) {
                        Ok(model) => loaded = Some((model_path.clone(), model)),
                        Err(e) => {
                            let _ = event_tx
                                .send(RuntimeEvent::Error(format!("Load failed: {e}")));
                            continue;
                        }
                    }
                }

                let (_, model) = loaded.as_ref().unwrap();

                // Build chat messages and apply the model's embedded chat template.
                let mut chat_messages: Vec<LlamaChatMessage> = Vec::new();
                if !system.trim().is_empty() {
                    match LlamaChatMessage::new("system".into(), system.trim().into()) {
                        Ok(m) => chat_messages.push(m),
                        Err(e) => {
                            let _ = event_tx
                                .send(RuntimeEvent::Error(format!("Chat message: {e}")));
                            continue;
                        }
                    }
                }
                let mut msg_err = false;
                for (role, content) in &messages {
                    match LlamaChatMessage::new(role.clone(), content.trim().into()) {
                        Ok(m) => chat_messages.push(m),
                        Err(e) => {
                            let _ = event_tx
                                .send(RuntimeEvent::Error(format!("Chat message: {e}")));
                            msg_err = true;
                            break;
                        }
                    }
                }
                if msg_err {
                    continue;
                }

                let tmpl = match model.chat_template(None) {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = event_tx
                            .send(RuntimeEvent::Error(format!("Chat template: {e}")));
                        continue;
                    }
                };
                let prompt = match model.apply_chat_template(&tmpl, &chat_messages, true) {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = event_tx
                            .send(RuntimeEvent::Error(format!("Chat template: {e}")));
                        continue;
                    }
                };

                let tokens = match model.str_to_token(&prompt, AddBos::Never) {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = event_tx
                            .send(RuntimeEvent::Error(format!("Tokenize: {e}")));
                        continue;
                    }
                };

                let n_ctx = std::cmp::max(tokens.len() + n_predict as usize + 64, 4096);
                let ctx_params = LlamaContextParams::default()
                    .with_n_ctx(std::num::NonZeroU32::new(n_ctx as u32));

                let mut ctx = match model.new_context(&backend, ctx_params) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = event_tx
                            .send(RuntimeEvent::Error(format!("Context: {e}")));
                        continue;
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
                    let _ = event_tx.send(RuntimeEvent::Error("Batch build failed".to_string()));
                    continue;
                }

                if let Err(e) = ctx.decode(&mut batch) {
                    let _ = event_tx.send(RuntimeEvent::Error(format!("Decode: {e}")));
                    continue;
                }

                let mut n_cur = batch.n_tokens();
                let mut sampler = LlamaSampler::greedy();

                'gen: for _ in 0..n_predict {
                    let idx = batch.n_tokens() as i32 - 1;
                    let token = sampler.sample(&ctx, idx);
                    sampler.accept(token);

                    if model.is_eog_token(token) {
                        break;
                    }

                    #[allow(deprecated)]
                    if let Ok(s) = model.token_to_str(token, Special::Plaintext) {
                        if !s.is_empty() && event_tx.send(RuntimeEvent::Token(s)).is_err() {
                            break 'gen;
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

                let _ = event_tx.send(RuntimeEvent::Done);
            }
        }
    }
    // Explicit drop ensures LlamaModel and LlamaBackend release their Metal resources
    // before this thread exits, so the GGML Metal device atexit destructor sees no live
    // resource sets.
    drop(loaded);
    drop(backend);
}
