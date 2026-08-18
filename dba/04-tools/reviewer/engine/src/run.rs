/// How the reviewed text was obtained.
///
/// This is a record of provenance, not a provider abstraction: there is no dispatch, no trait, and
/// no configuration behind it. Codeos either invoked Codex itself, or a human handed it text that
/// some other model produced. The two cases carry genuinely different fields — a session id and a
/// reconnect count only exist for a process this tool ran — so they are kept apart rather than
/// filled with placeholder values that would read as if Codeos had verified something it did not.
pub enum RunSource {
    /// Codeos invoked the Codex CLI. This is the only source that counts as a review round.
    Codex {
        session_id: String,
        reconnect_count: u32,
        effort: String,
    },
    /// A human supplied an assessment produced elsewhere. `label` is descriptive metadata only:
    /// Codeos neither invoked nor verified the named model, and cannot confirm the text came from
    /// it. Advisory evidence — never a review round.
    External { label: Option<String> },
}

/// One reviewer run: the raw reply text plus where it came from.
pub struct ReviewerRun {
    pub text: String,
    pub elapsed_ms: u64,
    pub source: RunSource,
}

impl ReviewerRun {
    pub fn is_external(&self) -> bool {
        matches!(self.source, RunSource::External { .. })
    }

    /// The `reviewer:` value written into records. For an external assessment the phrasing states
    /// the limit of what Codeos knows, so no reader can mistake the label for verification.
    pub fn reviewer_field(&self) -> String {
        match &self.source {
            RunSource::Codex { session_id, .. } => format!("codex (session {session_id})"),
            RunSource::External { label } => match label {
                Some(label) => format!(
                    "external assessment ({label}, as supplied — not invoked or verified by Codeos)"
                ),
                None => {
                    "external assessment (model unstated — not invoked or verified by Codeos)"
                        .to_string()
                }
            },
        }
    }
}
