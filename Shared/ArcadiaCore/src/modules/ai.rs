use super::ModuleCommand;

pub const NAME: &str = "ai";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}

pub struct AiProviderManifest {
    pub module_name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
}

pub const AI_PROVIDER_REGISTRY: &[AiProviderManifest] = &[
    AiProviderManifest {
        module_name: "ai-provider-llama-cpp",
        display_name: "llama.cpp",
        description: "Local inference via llama.cpp. Runs models directly on this machine.",
    },
    AiProviderManifest {
        module_name: "ai-provider-ollama",
        display_name: "Ollama",
        description: "Local inference via Ollama HTTP API.",
    },
    AiProviderManifest {
        module_name: "ai-provider-openai",
        display_name: "OpenAI",
        description: "Cloud inference via OpenAI API.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-claude",
        display_name: "Claude (CLI)",
        description: "Claude Pro via installed `claude` CLI. No API key required.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-codex",
        display_name: "Codex (CLI)",
        description: "ChatGPT Plus via installed `codex` CLI. No API key required.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-gemini",
        display_name: "Gemini (CLI)",
        description: "Gemini Advanced via installed `gemini` CLI. No API key required.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-aider",
        display_name: "Aider (CLI)",
        description: "Aider with its configured backend. No API key required.",
    },
];

pub fn enabled_ai_providers(module_rows: &[(String, bool)]) -> Vec<&'static AiProviderManifest> {
    AI_PROVIDER_REGISTRY
        .iter()
        .filter(|p| {
            module_rows
                .iter()
                .any(|(name, enabled)| name == p.module_name && *enabled)
        })
        .collect()
}

pub fn any_ai_provider_enabled(module_rows: &[(String, bool)]) -> bool {
    AI_PROVIDER_REGISTRY.iter().any(|p| {
        module_rows
            .iter()
            .any(|(name, enabled)| name == p.module_name && *enabled)
    })
}

/// Returns true when at least one AI provider module is enabled.
pub fn is_ai_provider_available(module_rows: &[(String, bool)]) -> bool {
    any_ai_provider_enabled(module_rows)
}
