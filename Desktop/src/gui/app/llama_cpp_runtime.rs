use std::sync::mpsc::{Receiver, SyncSender};

pub enum RuntimeRequest {
    Infer {
        model_path: String,
        prompt: String,
        n_predict: i32,
    },
}

pub enum RuntimeEvent {
    Token(String),
    Done,
    Error(String),
}

pub struct LlamaCppRuntime {
    pub request_tx: SyncSender<RuntimeRequest>,
    pub event_rx: Receiver<RuntimeEvent>,
}

impl LlamaCppRuntime {
    pub fn start() -> Self {
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel::<RuntimeRequest>(4);
        let (event_tx, event_rx) = std::sync::mpsc::sync_channel::<RuntimeEvent>(512);

        std::thread::Builder::new()
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
                        }
                    }
                }
            })
            .expect("failed to spawn inference thread");

        LlamaCppRuntime { request_tx, event_rx }
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
        model::{params::LlamaModelParams, AddBos, LlamaModel, Special},
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
            RuntimeRequest::Infer { model_path, prompt, n_predict } => {
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

                let tokens = match model.str_to_token(&prompt, AddBos::Always) {
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

                let eos = model.token_eos();
                let mut n_cur = batch.n_tokens();
                let mut sampler = LlamaSampler::greedy();

                'gen: for _ in 0..n_predict {
                    let idx = batch.n_tokens() as i32 - 1;
                    let token = sampler.sample(&ctx, idx);
                    sampler.accept(token);

                    if token == eos {
                        break;
                    }

                    #[allow(deprecated)]
                    if let Ok(s) = model.token_to_str(token, Special::Tokenize) {
                        if event_tx.send(RuntimeEvent::Token(s)).is_err() {
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
}

/// Build a simple instruct-format prompt from message history.
/// Works with most instruction-tuned GGUF models.
pub fn build_chat_prompt(
    system_prompt: &str,
    messages: &[(String, String)], // (role, content)
) -> String {
    let mut out = String::new();
    if !system_prompt.trim().is_empty() {
        out.push_str(system_prompt.trim());
        out.push_str("\n\n");
    }
    for (role, content) in messages {
        if role == "user" {
            out.push_str("User: ");
            out.push_str(content.trim());
            out.push_str("\n\n");
        } else if role == "assistant" {
            out.push_str("Assistant: ");
            out.push_str(content.trim());
            out.push_str("\n\n");
        }
    }
    out.push_str("Assistant:");
    out
}
