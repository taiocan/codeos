use super::{ProviderConfig, RawAssessment, ReviewProvider};
use crate::packet::ReviewPacket;
use anyhow::Result;

pub struct OpenCodeProvider;
pub struct GeminiProvider;
pub struct KimiProvider;

impl ReviewProvider for OpenCodeProvider {
    fn name(&self) -> &str { "opencode" }
    fn invoke(&self, _packet: &ReviewPacket, _cfg: &ProviderConfig) -> Result<RawAssessment> {
        anyhow::bail!("OpenCodeProvider is not yet implemented")
    }
    fn extract_session_id(&self, _raw: &str) -> Option<String> { None }
}

impl ReviewProvider for GeminiProvider {
    fn name(&self) -> &str { "gemini" }
    fn invoke(&self, _packet: &ReviewPacket, _cfg: &ProviderConfig) -> Result<RawAssessment> {
        anyhow::bail!("GeminiProvider is not yet implemented")
    }
    fn extract_session_id(&self, _raw: &str) -> Option<String> { None }
}

impl ReviewProvider for KimiProvider {
    fn name(&self) -> &str { "kimi" }
    fn invoke(&self, _packet: &ReviewPacket, _cfg: &ProviderConfig) -> Result<RawAssessment> {
        anyhow::bail!("KimiProvider is not yet implemented")
    }
    fn extract_session_id(&self, _raw: &str) -> Option<String> { None }
}
