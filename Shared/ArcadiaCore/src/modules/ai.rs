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

pub const AI_PROVIDER_REGISTRY: &[AiProviderManifest] = &[AiProviderManifest {
    module_name: "ai-provider-llama-cpp",
    display_name: "llama.cpp",
    description: "Local inference via llama.cpp. Runs models directly on this machine.",
}];

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
