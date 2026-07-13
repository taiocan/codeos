use crate::packet::{CoverageState, ReviewPacket};
use crate::provider::RawAssessment;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct ParsedReview {
    pub codex_concern: String,
    pub effective_concern: String,
    pub evidence: String,
    pub summary_line: String,
    pub coverage_note: String,
    pub highest_impact_uncertainty: String,
}

/// Parse LOG SUMMARY, EVIDENCE, HIGHEST-IMPACT UNCERTAINTY from reviewer output.
pub fn parse_review_output(output: &str, coverage_state: &CoverageState) -> ParsedReview {
    let summary_line = output.lines()
        .filter(|l| l.starts_with("LOG SUMMARY:"))
        .last()
        .map(|l| l.to_string())
        .unwrap_or_default();

    let codex_concern = if summary_line.is_empty() {
        "UNCLASSIFIED".to_string()
    } else {
        let rest = summary_line.trim_start_matches("LOG SUMMARY:").trim();
        let concern_end = rest.find(" —").unwrap_or(rest.len());
        normalize_concern(&rest[..concern_end])
    };

    let evidence_raw = output.lines()
        .filter(|l| l.starts_with("EVIDENCE:"))
        .last()
        .map(|l| l.trim_start_matches("EVIDENCE:").trim().to_string())
        .unwrap_or_default();

    let evidence = match evidence_raw.chars().next() {
        Some(c) if "ABCDE".contains(c) => c.to_string(),
        _ if evidence_raw == "not reported" => "not reported".to_string(),
        _ if evidence_raw.is_empty() => "not reported".to_string(),
        _ => "not reported".to_string(),
    };

    let highest_impact_uncertainty = output.lines()
        .filter(|l| l.starts_with("HIGHEST-IMPACT UNCERTAINTY:"))
        .last()
        .map(|l| l.trim_start_matches("HIGHEST-IMPACT UNCERTAINTY:").trim().to_string())
        .unwrap_or_default();

    // Compute effective concern
    let floor = coverage_state.concern_floor();
    let concern_rank = concern_to_rank(&codex_concern);
    let eff_rank = concern_rank.max(floor);
    let effective_concern = rank_to_concern(eff_rank);

    let coverage_note = if eff_rank > concern_rank {
        format!("raised from '{}' to the coverage floor for {}", codex_concern, coverage_state.as_str())
    } else {
        String::new()
    };

    let final_summary_line = if summary_line.is_empty() {
        "LOG SUMMARY: UNCLASSIFIED — no parseable summary; HIGH attention, manual review required".to_string()
    } else {
        summary_line
    };

    ParsedReview {
        codex_concern: codex_concern.clone(),
        effective_concern,
        evidence,
        summary_line: final_summary_line,
        coverage_note,
        highest_impact_uncertainty,
    }
}

fn normalize_concern(s: &str) -> String {
    let upper = s.trim().to_uppercase();
    match upper.as_str() {
        "NO OBJECTION" => "NO OBJECTION".to_string(),
        "CHANGES ADVISED" => "CHANGES ADVISED".to_string(),
        "DO NOT ADVANCE" => "DO NOT ADVANCE".to_string(),
        "UNCLASSIFIED" => "UNCLASSIFIED".to_string(),
        _ => "UNCLASSIFIED".to_string(),
    }
}

fn concern_to_rank(c: &str) -> u8 {
    match c {
        "NO OBJECTION"   => 0,
        "CHANGES ADVISED" => 1,
        "UNCLASSIFIED"   => 2,
        "DO NOT ADVANCE" => 3,
        _ => 2,
    }
}

fn rank_to_concern(r: u8) -> String {
    match r {
        0 => "NO OBJECTION",
        1 => "CHANGES ADVISED",
        2 => "UNCLASSIFIED",
        _ => "DO NOT ADVANCE",
    }.to_string()
}

/// Write the assessment file and return (assessment_path, assessment_sha256).
pub fn write_assessment(
    review_id: &str,
    packet: &ReviewPacket,
    raw: &RawAssessment,
    parsed: &ParsedReview,
    outdir: &Path,
    packet_saved: &Path,
    packet_hash: &str,
) -> Result<(PathBuf, String)> {
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let short_sha = &packet.review_sha[..7.min(packet.review_sha.len())];
    let ts_clean = ts.replace(':', "");
    let filename = format!("{}-{}-stage-{}-{}.md", ts_clean, packet.feature, packet.stage, short_sha);
    let assessment_file = outdir.join(&filename);

    std::fs::create_dir_all(outdir).context("create assessment dir")?;

    let mut content = String::new();
    content.push_str("---\n");
    content.push_str(&format!("review_id: {}\n", review_id));
    content.push_str("reviewed:\n");
    content.push_str(&format!("  feature: {}\n", packet.feature));
    content.push_str(&format!("  stage: {}\n", packet.stage));
    content.push_str(&format!("  branch: {}\n", packet.branch));
    content.push_str(&format!("  base_commit: {}\n", packet.base_sha));
    content.push_str(&format!("  review_commit: {}\n", packet.review_sha));
    if packet.artifacts.is_empty() {
        content.push_str("  artifacts: []\n");
    } else {
        content.push_str("  artifacts:\n");
        for a in &packet.artifacts {
            content.push_str(&format!("    - path: {}\n", a.path));
            if !a.sha256.is_empty() {
                content.push_str(&format!("      sha256: {}\n", a.sha256));
            }
            content.push_str(&format!("      visibility: {}\n", a.visibility));
        }
    }
    content.push_str(&format!("  diff_hash: {}\n", packet.diff_hash));
    content.push_str(&format!("  coverage_state: {}\n", packet.coverage_state.as_str()));
    content.push_str(&format!("  workspace_dirty: {}\n", packet.workspace_dirty));
    content.push_str(&format!("  redaction_count: {}\n", packet.redaction_count));
    content.push_str(&format!("  secret_redaction: {}\n", packet.secret_flag));
    if packet.excluded_paths.is_empty() {
        content.push_str("  excluded_paths: []\n");
    } else {
        content.push_str("  excluded_paths:\n");
        for (p, reason, section) in &packet.excluded_paths {
            content.push_str(&format!("    - path: \"{}\"\n", p));
            content.push_str(&format!("      reason: \"{}\"\n", reason));
            content.push_str(&format!("      affected_section: {}\n", section));
        }
    }
    let packet_basename = packet_saved.file_name().and_then(|n| n.to_str()).unwrap_or("");
    content.push_str(&format!("  reviewed_packet: packets/{}\n", packet_basename));
    content.push_str(&format!("  reviewed_packet_sha256: {}\n", packet_hash));
    content.push_str(&format!("  reviewer: \"codex (session {})\"\n", raw.session_id));
    content.push_str(&format!("  codex_concern: {}\n", parsed.codex_concern));
    content.push_str(&format!("  effective_concern: {}\n", parsed.effective_concern));
    if !parsed.coverage_note.is_empty() {
        content.push_str(&format!("  effective_concern_note: {}\n", parsed.coverage_note));
    }
    content.push_str(&format!("  evidence: {}\n", parsed.evidence));
    content.push_str(&format!("  reasoning_effort: {}\n", raw.effort));
    content.push_str(&format!("  reconnect_count: {}\n", raw.reconnect_count));
    content.push_str(&format!("  elapsed_ms: {}\n", raw.elapsed_ms));
    content.push_str("---\n\n");
    content.push_str(&raw.text);

    std::fs::write(&assessment_file, &content)
        .with_context(|| format!("could not write assessment to {}", assessment_file.display()))?;

    let sha = crate::packet::sha256_str(&content);
    Ok((assessment_file, sha))
}

/// Validate required frontmatter fields are non-empty (fail-closed schema check).
pub fn validate_schema(
    packet: &ReviewPacket,
    parsed: &ParsedReview,
    packet_hash: &str,
) -> Result<()> {
    let mut errs = Vec::new();
    if packet.feature.is_empty() { errs.push("missing:feature"); }
    if packet.stage.is_empty() { errs.push("missing:stage"); }
    if packet.base_sha.is_empty() { errs.push("missing:base_commit"); }
    if packet.review_sha.is_empty() { errs.push("missing:review_commit"); }
    if packet.diff_hash.is_empty() { errs.push("missing:diff_hash"); }
    if packet_hash.is_empty() { errs.push("missing:reviewed_packet_sha256"); }

    let valid_coverage = ["FULL_COVERAGE","PARTIAL_COVERAGE","SECRET_REDACTION","CRITICAL_OMISSION","EMPTY_PACKET"];
    if !valid_coverage.contains(&packet.coverage_state.as_str()) {
        errs.push("enum:coverage_state");
    }
    let valid_concern = ["NO OBJECTION","CHANGES ADVISED","DO NOT ADVANCE","UNCLASSIFIED"];
    if !valid_concern.contains(&parsed.codex_concern.as_str()) {
        errs.push("enum:codex_concern");
    }
    if !valid_concern.contains(&parsed.effective_concern.as_str()) {
        errs.push("enum:effective_concern");
    }
    let valid_evidence = ["A","B","C","D","E","not reported"];
    if !valid_evidence.contains(&parsed.evidence.as_str()) {
        errs.push("enum:evidence");
    }
    if packet.artifacts.is_empty()
        && !matches!(packet.coverage_state, CoverageState::CriticalOmission | CoverageState::EmptyPacket)
    {
        errs.push("missing:artifacts");
    }

    if !errs.is_empty() {
        anyhow::bail!(
            "v0 schema validation failed (fail-closed): {}\nreview NOT logged. See docs/reviewer-artifact-schemas.md",
            errs.join(" ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::CoverageState;

    fn make_raw(text: &str) -> RawAssessment {
        crate::provider::RawAssessment {
            text: text.to_string(),
            session_id: "test-session".to_string(),
            elapsed_ms: 1000,
            reconnect_count: 0,
            effort: "high".to_string(),
        }
    }

    #[test]
    fn parses_no_objection() {
        let text = "Some findings.\n\nPR decision: ADVANCE\nLOG SUMMARY: NO OBJECTION — all good\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let parsed = parse_review_output(text, &CoverageState::FullCoverage);
        assert_eq!(parsed.codex_concern, "NO OBJECTION");
        assert_eq!(parsed.effective_concern, "NO OBJECTION");
        assert_eq!(parsed.evidence, "A");
    }

    #[test]
    fn parses_changes_advised() {
        let text = "LOG SUMMARY: CHANGES ADVISED — fix required\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: something\n";
        let parsed = parse_review_output(text, &CoverageState::FullCoverage);
        assert_eq!(parsed.codex_concern, "CHANGES ADVISED");
        assert_eq!(parsed.evidence, "B");
    }

    #[test]
    fn coverage_floor_escalates_concern() {
        let text = "LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: x\n";
        let parsed = parse_review_output(text, &CoverageState::CriticalOmission);
        assert_eq!(parsed.codex_concern, "NO OBJECTION");
        assert_eq!(parsed.effective_concern, "DO NOT ADVANCE");
        assert!(!parsed.coverage_note.is_empty());
    }

    #[test]
    fn missing_summary_gives_unclassified() {
        let text = "No summary line here.\n";
        let parsed = parse_review_output(text, &CoverageState::FullCoverage);
        assert_eq!(parsed.codex_concern, "UNCLASSIFIED");
        assert!(parsed.summary_line.contains("UNCLASSIFIED"));
    }

    #[test]
    fn write_assessment_includes_review_id_in_frontmatter() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo_root = dir.path();
        std::process::Command::new("git").args(["init"]).current_dir(repo_root).output().expect("git init");
        std::process::Command::new("git").args(["config", "user.email", "t@t.test"]).current_dir(repo_root).output().ok();
        std::process::Command::new("git").args(["config", "user.name", "T"]).current_dir(repo_root).output().ok();
        std::fs::write(repo_root.join("tracked.md"), "# tracked\n").expect("write");
        std::process::Command::new("git").args(["add", "tracked.md"]).current_dir(repo_root).output().expect("git add");
        std::process::Command::new("git").args(["commit", "-m", "init"]).current_dir(repo_root).output().expect("git commit");

        let mut toolkit_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        toolkit_root.pop(); toolkit_root.pop(); // tools/reviewer -> Codeos/

        let opts = crate::packet::PacketBuildOptions {
            feature: "UPG-TEST".to_string(),
            stage: "selfdev-step-1".to_string(),
            artifacts: vec!["tracked.md".to_string()],
            sha_only_paths: vec![],
            delta_mode: false,
            delta_base: None,
            fresh_session: false,
            repo_root: repo_root.to_string_lossy().into_owned(),
            toolkit_root: toolkit_root.to_string_lossy().into_owned(),
        };
        let packet = crate::packet::build(&opts).expect("build packet");
        let raw = make_raw("LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n");
        let parsed = parse_review_output(&raw.text, &packet.coverage_state);

        let outdir = dir.path().join("codex-out");
        let review_id = "REV__UPG-TEST__selfdev-step-1__R1";
        let (assessment_file, _hash) = write_assessment(
            review_id, &packet, &raw, &parsed, &outdir, Path::new("packet.txt"), "cafefeed",
        ).expect("write_assessment");

        let content = std::fs::read_to_string(&assessment_file).expect("read assessment");
        assert!(
            content.starts_with(&format!("---\nreview_id: {}\n", review_id)),
            "review_id must be the first frontmatter field: {}", content
        );

        // AC-5: review_id is a content field only — the filename keeps its pre-existing
        // <ts>-<feature>-stage-<stage>-<sha>.md shape, never a REV__... shape.
        let filename = assessment_file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert!(
            !filename.starts_with("REV__"),
            "assessment filename must not be renamed to the REV__ shape: {}", filename
        );
        assert!(
            filename.contains("-UPG-TEST-stage-selfdev-step-1-"),
            "assessment filename must keep the legacy <ts>-<feature>-stage-<stage>-<sha> shape: {}",
            filename
        );
    }
}
