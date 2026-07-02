use crate::packet::ReviewPacket;
use anyhow::Result;

pub mod codex;
pub mod stubs;

/// Runtime provider configuration resolved from CLI > env > toml > default.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_name: String,
    pub reasoning_effort: String,
    pub repo_root: String,
    pub sessions_dir: String,
}

/// Raw text output from the AI provider.
pub struct RawAssessment {
    pub text: String,
    pub session_id: String,
    pub elapsed_ms: u64,
    pub reconnect_count: u32,
    pub effort: String,
}

/// The provider abstraction. Command handlers interact with this trait only;
/// no handler imports a concrete provider type.
pub trait ReviewProvider {
    fn name(&self) -> &str;
    fn invoke(&self, packet: &ReviewPacket, cfg: &ProviderConfig) -> Result<RawAssessment>;
    fn extract_session_id(&self, raw: &str) -> Option<String>;
}

/// Resolve a provider by name. Returns an error for unknown names.
pub fn resolve_provider(name: &str) -> Result<Box<dyn ReviewProvider>> {
    match name {
        "codex" => Ok(Box::new(codex::CodexProvider)),
        "opencode" => Ok(Box::new(stubs::OpenCodeProvider)),
        "gemini" => Ok(Box::new(stubs::GeminiProvider)),
        "kimi" => Ok(Box::new(stubs::KimiProvider)),
        other => anyhow::bail!("unknown provider '{}'; supported: codex, opencode, gemini, kimi", other),
    }
}
