use crate::packet::{CoverageState, ReviewPacket};
use crate::codex::CodexResult;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

pub struct ParsedReview {
    pub codex_concern: String,
    pub effective_concern: String,
    pub evidence: String,
    pub summary_line: String,
    pub coverage_note: String,
    pub highest_impact_uncertainty: String,
}

/// The exact five TRIAGE RULE labels from `dba/03-prompts/review/codeos-reviewer-task.md` — the
/// only values `parse_findings` accepts as a valid `Classification:`. A `Finding:` line
/// with any other value is treated as malformed (AC-2), not silently accepted.
const CANONICAL_CLASSIFICATIONS: [&str; 5] = [
    "IN-SCOPE BLOCKER",
    "IN-SCOPE NON-BLOCKER",
    "OUT-OF-SCOPE BACKLOG",
    "REJECTED",
    "SELF-REFERENCE / REVIEW-BOOKKEEPING",
];

/// A parsed finding block. `evidence`/`why`/`scope_reason` are parsed for the malformed-block
/// diagnostic and for verifying the body is left untouched (AC-4) — they are **not** serialized
/// to the assessment frontmatter (UPG-0047's compact-schema guardrail); only the fields covered
/// by `to_yaml_entry` are.
pub struct Finding {
    pub finding_id: String,
    pub severity: String,
    pub classification: String,
    pub summary: String,
    pub acceptance_criterion: Option<String>,
    pub required_action: String,
    pub evidence: String,
    pub why: String,
    pub scope_reason: Option<String>,
}

impl Finding {
    /// The compact YAML subset actually written to the assessment frontmatter — deliberately
    /// excludes `evidence`/`why`/`scope_reason` (full prose stays in the body, unduplicated).
    fn to_yaml_entry(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("    - finding_id: {}\n", self.finding_id));
        s.push_str(&format!("      severity: {}\n", self.severity));
        s.push_str(&format!("      classification: {}\n", self.classification));
        s.push_str(&format!("      summary: \"{}\"\n", yaml_escape(&self.summary)));
        if let Some(ac) = &self.acceptance_criterion {
            s.push_str(&format!("      acceptance_criterion: {}\n", ac));
        }
        s.push_str(&format!("      required_action: {}\n", self.required_action));
        s
    }
}

/// Minimal YAML double-quoted-string escaping (backslash and double-quote only — finding
/// summaries are reviewer prose, not attacker-controlled, but escaping is cheap and correct).
fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Parse `Finding: / Severity: / Classification:` blocks from a reviewer's raw response text
/// into structured `Finding`s, plus a count of `Finding:` lines that did not match the full
/// expected shape (never silently dropped).
///
/// Only the region **before the first line-anchored `LOG SUMMARY:`** is scanned. This is not
/// an arbitrary truncation: `raw_text` is the full CLI transcript, which — after the real
/// answer — goes on to echo the packaged reviewer-task prompt (containing a *literal,
/// indented* `LOG SUMMARY: <NO OBJECTION | ...>` placeholder line) and then, in this codebase's
/// observed CLI output, a second verbatim echo of the real answer itself. Scanning the whole
/// text would double-count every real finding once for the answer and once for its echo, and
/// risks matching the placeholder instructional text. `parse_review_output` above sidesteps the
/// same issue for `LOG SUMMARY:` itself by taking the *last* match; findings are collectively a
/// list rather than one scalar, so the correct equivalent is to only scan the first, real
/// occurrence of the answer — i.e. everything before the first line-anchored `LOG SUMMARY:`,
/// which the required output format (`dba/03-prompts/review/codeos-reviewer-task.md`) always places after
/// every finding.
///
/// The `Evidence:`/`Why:`/`Required action:` block is accepted in **either** of two real,
/// currently-coexisting shapes — confirmed against this repo's own corpus (not assumed):
/// `dba/03-prompts/review/codeos-reviewer-task.md` asks for them combined on one line
/// (`Evidence: X / Why: Y / Required action: Z`), but Codex does not reliably follow that —
/// the three-separate-line form (`Evidence: X` / `Why: Y` / `Required action: Z`, each its own
/// line) appears throughout the corpus's *entire* date range, including this repo's own
/// `UPG-0045`/`UPG-0046` review rounds from this same session. This is ongoing model output
/// variance, not a resolved historical format version — both shapes are permanently supported,
/// not one treated as legacy.
pub fn parse_findings(raw_text: &str, review_id: &str) -> (Vec<Finding>, usize) {
    let real_region = raw_text.split("\nLOG SUMMARY:").next().unwrap_or(raw_text);
    let lines: Vec<&str> = real_region.lines().collect();

    let finding_re = Regex::new(r"^Finding: (.+?) / Severity: (High|Medium|Low) / Classification: (.+?)\s*$")
        .expect("valid regex");
    let combined_re = Regex::new(r"^Evidence: (.+?) / Why: (.+?) / Required action: (fix now|optional fix|backlog|reject)\s*$")
        .expect("valid regex");
    // A third real, historical shape: Evidence/Why/Required action/Scope reason all combined
    // onto one line (earliest corpus era, e.g. 2026-06-30/2026-07-01 rounds).
    let combined_with_scope_re = Regex::new(r"^Evidence: (.+?) / Why: (.+?) / Required action: (fix now|optional fix|backlog|reject) / Scope reason: (.+)$")
        .expect("valid regex");
    let evidence_only_re = Regex::new(r"^Evidence: (.+?)\s*$").expect("valid regex");
    let why_only_re = Regex::new(r"^Why: (.+?)\s*$").expect("valid regex");
    let action_only_re = Regex::new(r"^Required action: (fix now|optional fix|backlog|reject)\s*$").expect("valid regex");
    let scope_re = Regex::new(r"^Scope reason: (.+)$").expect("valid regex");
    let ac_re = Regex::new(r"AC-\d+").expect("valid regex");

    let mut findings = Vec::new();
    let mut unparsed_count = 0usize;
    let mut seq = 0u32;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if !line.starts_with("Finding:") {
            i += 1;
            continue;
        }

        let Some(fcaps) = finding_re.captures(line) else {
            unparsed_count += 1;
            eprintln!("warning: malformed finding block (Finding/Severity/Classification line did not match the expected shape): {}", line);
            i += 1;
            continue;
        };

        let classification = fcaps[3].trim().to_string();
        if !CANONICAL_CLASSIFICATIONS.contains(&classification.as_str()) {
            unparsed_count += 1;
            eprintln!(
                "warning: malformed finding block (Classification '{}' is not one of the five canonical TRIAGE RULE labels): {}",
                classification, line
            );
            i += 1;
            continue;
        }

        // Scan forward for Evidence/Why/Required action (either shape) and an optional Scope
        // reason, stopping at a blank line or the next Finding:/PR decision: boundary.
        let mut evidence: Option<String> = None;
        let mut why: Option<String> = None;
        let mut required_action: Option<String> = None;
        let mut scope_reason: Option<String> = None;
        let mut k = i + 1;
        let window_end = (i + 1 + 6).min(lines.len());
        while k < window_end {
            let l = lines[k];
            if l.trim().is_empty() || l.starts_with("Finding:") || l.starts_with("PR decision:") {
                break;
            }
            if evidence.is_none() && why.is_none() && required_action.is_none() && scope_reason.is_none() {
                if let Some(c) = combined_with_scope_re.captures(l) {
                    evidence = Some(c[1].trim().to_string());
                    why = Some(c[2].trim().to_string());
                    required_action = Some(c[3].to_string());
                    scope_reason = Some(c[4].trim().to_string());
                    k += 1;
                    continue;
                }
            }
            if evidence.is_none() && why.is_none() && required_action.is_none() {
                if let Some(c) = combined_re.captures(l) {
                    evidence = Some(c[1].trim().to_string());
                    why = Some(c[2].trim().to_string());
                    required_action = Some(c[3].to_string());
                    k += 1;
                    continue;
                }
            }
            if evidence.is_none() {
                if let Some(c) = evidence_only_re.captures(l) { evidence = Some(c[1].trim().to_string()); k += 1; continue; }
            }
            if why.is_none() {
                if let Some(c) = why_only_re.captures(l) { why = Some(c[1].trim().to_string()); k += 1; continue; }
            }
            if required_action.is_none() {
                if let Some(c) = action_only_re.captures(l) { required_action = Some(c[1].to_string()); k += 1; continue; }
            }
            if scope_reason.is_none() {
                if let Some(c) = scope_re.captures(l) { scope_reason = Some(c[1].trim().to_string()); k += 1; continue; }
            }
            break;
        }

        let (Some(evidence), Some(why), Some(required_action)) = (evidence, why, required_action) else {
            unparsed_count += 1;
            eprintln!("warning: malformed finding block (Evidence/Why/Required action not all found in either supported shape) near: {}", line);
            i += 1;
            continue;
        };

        seq += 1;
        let summary = fcaps[1].trim().to_string();
        let acceptance_criterion = ac_re.find(&summary).map(|m| m.as_str().to_string());
        findings.push(Finding {
            finding_id: format!("FND__{}__{:02}", review_id, seq),
            severity: fcaps[2].to_string(),
            classification,
            summary,
            acceptance_criterion,
            required_action,
            evidence,
            why,
            scope_reason,
        });
        i = k;
    }

    (findings, unparsed_count)
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
    findings: &[Finding],
    unparsed_findings_count: usize,
    packet: &ReviewPacket,
    raw: &CodexResult,
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
    if findings.is_empty() {
        content.push_str("findings: []\n");
    } else {
        content.push_str("findings:\n");
        for f in findings {
            content.push_str(&f.to_yaml_entry());
        }
    }
    content.push_str(&format!("unparsed_findings_count: {}\n", unparsed_findings_count));
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
    // The reviewer task prompt requires LOG SUMMARY, EVIDENCE, and HIGHEST-IMPACT UNCERTAINTY
    // as its last three lines. The first two are promoted above; promote the third too, so the
    // review policy's verification round-trip can be triggered from the structured record
    // rather than only from the raw body below. Omitted when the reviewer did not report one.
    if !parsed.highest_impact_uncertainty.is_empty() {
        content.push_str(&format!(
            "  highest_impact_uncertainty: \"{}\"\n",
            yaml_escape(&parsed.highest_impact_uncertainty)
        ));
    }
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
            "assessment validation failed (fail-closed): {}\nreview NOT logged",
            errs.join(" ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod uncertainty_tests {
    use super::*;

    fn parse(text: &str) -> ParsedReview {
        parse_review_output(text, &CoverageState::FullCoverage)
    }

    #[test]
    fn highest_impact_uncertainty_is_parsed_from_the_last_three_lines() {
        let parsed = parse(
            "LOG SUMMARY: NO OBJECTION — fine\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: the base ref may not be the reviewed commit\n",
        );
        assert_eq!(
            parsed.highest_impact_uncertainty,
            "the base ref may not be the reviewed commit"
        );
    }

    #[test]
    fn absent_uncertainty_stays_empty_rather_than_becoming_a_placeholder() {
        let parsed = parse("LOG SUMMARY: NO OBJECTION — fine\nEVIDENCE: A\n");
        assert!(parsed.highest_impact_uncertainty.is_empty());
    }

    #[test]
    fn uncertainty_containing_a_colon_is_quoted_and_escaped_for_yaml() {
        // Free-text reviewer prose routinely contains ':' and '"', which would break the
        // frontmatter if emitted bare.
        let raw = "scope: the \"base\" ref";
        let escaped = yaml_escape(raw);
        assert_eq!(escaped, "scope: the \\\"base\\\" ref");
        assert!(!escaped.contains("\"base\""));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::CoverageState;

    fn make_raw(text: &str) -> CodexResult {
        crate::codex::CodexResult {
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
        for _ in 0..4 { toolkit_root.pop(); }

        let opts = crate::packet::PacketBuildOptions {
            feature: "UPG-TEST".to_string(),
            stage: "selfdev-step-1".to_string(),
            artifacts: vec!["tracked.md".to_string()],
            sha_only_paths: vec![],
            delta_mode: false,
            delta_base: None,
            repo_root: repo_root.to_string_lossy().into_owned(),
            toolkit_root: toolkit_root.to_string_lossy().into_owned(),
        };
        let packet = crate::packet::build(&opts).expect("build packet");
        let raw = make_raw("LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n");
        let parsed = parse_review_output(&raw.text, &packet.coverage_state);

        let outdir = dir.path().join("codex-out");
        let review_id = "REV__UPG-TEST__selfdev-step-1__R1";
        let (assessment_file, _hash) = write_assessment(
            review_id, &[], 0, &packet, &raw, &parsed, &outdir, Path::new("packet.txt"), "cafefeed",
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

    const RID: &str = "REV__UPG-TEST__selfdev-step-3__R1";

    #[test]
    fn parse_findings_single_finding() {
        let text = "Finding: Test names not preserved / Severity: High / Classification: IN-SCOPE BLOCKER  \nEvidence: diff shows renamed tests / Why: breaks traceability / Required action: fix now  \nScope reason: directly affects AC-7.\n\nPR decision: DO NOT ADVANCE\nLOG SUMMARY: DO NOT ADVANCE — test names not preserved\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert_eq!(unparsed, 0);
        assert_eq!(findings.len(), 1);
        let f = &findings[0];
        assert_eq!(f.finding_id, format!("FND__{}__01", RID));
        assert_eq!(f.severity, "High");
        assert_eq!(f.classification, "IN-SCOPE BLOCKER");
        assert_eq!(f.summary, "Test names not preserved");
        assert_eq!(f.required_action, "fix now");
        assert_eq!(f.evidence, "diff shows renamed tests");
        assert_eq!(f.why, "breaks traceability");
        assert_eq!(f.scope_reason.as_deref(), Some("directly affects AC-7."));
    }

    #[test]
    fn parse_findings_multiple_findings_get_stable_ordered_ids() {
        let text = "Finding: First issue / Severity: Medium / Classification: IN-SCOPE BLOCKER  \nEvidence: e1 / Why: w1 / Required action: fix now  \nScope reason: s1\n\nFinding: Second issue / Severity: Low / Classification: IN-SCOPE NON-BLOCKER  \nEvidence: e2 / Why: w2 / Required action: optional fix  \nScope reason: s2\n\nPR decision: REQUEST CHANGES\nLOG SUMMARY: CHANGES ADVISED — two issues\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert_eq!(unparsed, 0);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].finding_id, format!("FND__{}__01", RID));
        assert_eq!(findings[1].finding_id, format!("FND__{}__02", RID));
        assert_eq!(findings[0].summary, "First issue");
        assert_eq!(findings[1].summary, "Second issue");
    }

    #[test]
    fn parse_findings_accepts_all_five_canonical_classifications() {
        let labels = [
            "IN-SCOPE BLOCKER",
            "IN-SCOPE NON-BLOCKER",
            "OUT-OF-SCOPE BACKLOG",
            "REJECTED",
            "SELF-REFERENCE / REVIEW-BOOKKEEPING",
        ];
        for label in labels {
            let text = format!(
                "Finding: x / Severity: Low / Classification: {}  \nEvidence: e / Why: w / Required action: backlog  \n\nLOG SUMMARY: CHANGES ADVISED — x\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n",
                label
            );
            let (findings, unparsed) = parse_findings(&text, RID);
            assert_eq!(unparsed, 0, "label {} should parse cleanly", label);
            assert_eq!(findings.len(), 1, "label {} should produce exactly one finding", label);
            assert_eq!(
                findings[0].classification, label,
                "compound label must parse as one classification value, not split on its own '/'"
            );
        }
    }

    #[test]
    fn parse_findings_rejects_non_canonical_classification() {
        // AC-2: only the five canonical TRIAGE RULE labels are accepted. An invented sixth
        // label must be treated as malformed (counted unparsed), never silently accepted.
        let text = "Finding: x / Severity: Low / Classification: MAYBE-BLOCKER  \nEvidence: e / Why: w / Required action: backlog  \n\nLOG SUMMARY: CHANGES ADVISED — x\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert!(findings.is_empty(), "a non-canonical classification must not produce a finding");
        assert_eq!(unparsed, 1, "a non-canonical classification must be counted as unparsed, not silently dropped or accepted");
    }

    #[test]
    fn parse_findings_no_findings_produces_empty_list() {
        let text = "No issues found.\n\nPR decision: ADVANCE\nLOG SUMMARY: NO OBJECTION — all good\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert_eq!(unparsed, 0);
        assert!(findings.is_empty());
    }

    #[test]
    fn parse_findings_malformed_block_counts_unparsed_never_drops_silently() {
        // Missing "Classification:" entirely — the Finding line still starts with "Finding:"
        // but does not match the full expected shape.
        let text = "Finding: something is wrong / Severity: High\nEvidence: e / Why: w / Required action: fix now  \n\nLOG SUMMARY: CHANGES ADVISED — malformed\nEVIDENCE: C\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert_eq!(findings.len(), 0);
        assert_eq!(unparsed, 1, "a malformed Finding line must be counted, not silently dropped");
    }

    #[test]
    fn parse_findings_summary_containing_slash_still_parses_correctly() {
        // Regression guard for keyword-anchored (not naive " / "-split) parsing: this is a real
        // shape seen in this repo's own history (UPG-0046 Step 3 R1).
        let text = "Finding: AC-2's end-to-end round increment verification is not provided as stated / Severity: Low / Classification: IN-SCOPE BLOCKER  \nEvidence: AC-2 specifies `smoke_review_id_first_round_is_r1` / `smoke_review_id_increments_across_rounds` that “run `review` twice” / Why: partially supported / Required action: fix now  \nScope reason: part of the PR's own acceptance contract.\n\nLOG SUMMARY: DO NOT ADVANCE — x\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert_eq!(unparsed, 0);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, "Low");
        assert_eq!(findings[0].classification, "IN-SCOPE BLOCKER");
        assert_eq!(findings[0].required_action, "fix now");
        assert!(findings[0].evidence.contains("smoke_review_id_first_round_is_r1"));
        assert_eq!(findings[0].acceptance_criterion.as_deref(), Some("AC-2"));
    }

    #[test]
    fn parse_findings_ignores_duplicate_transcript_echo_after_log_summary() {
        // Simulates this codebase's own observed CLI behavior: the real answer is followed by
        // a banner and a second, verbatim echo of the same findings later in the same text.
        // Only the first (real) occurrence, before the first line-anchored LOG SUMMARY:, must
        // be counted.
        let real_answer = "Finding: Real finding / Severity: High / Classification: IN-SCOPE BLOCKER  \nEvidence: e / Why: w / Required action: fix now  \n\nLOG SUMMARY: DO NOT ADVANCE — real\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let echoed_transcript = "OpenAI Codex v0.142.5\n--------\nuser\nFinding: Real finding / Severity: High / Classification: IN-SCOPE BLOCKER  \nEvidence: e / Why: w / Required action: fix now  \n\nLOG SUMMARY: DO NOT ADVANCE — real\n";
        let text = format!("{}{}", real_answer, echoed_transcript);
        let (findings, unparsed) = parse_findings(&text, RID);
        assert_eq!(unparsed, 0);
        assert_eq!(findings.len(), 1, "the echoed transcript must not double-count the same finding");
    }

    #[test]
    fn parse_findings_deterministic_across_repeated_parses() {
        let text = "Finding: A / Severity: Low / Classification: IN-SCOPE NON-BLOCKER  \nEvidence: e / Why: w / Required action: optional fix  \n\nLOG SUMMARY: CHANGES ADVISED — a\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (f1, u1) = parse_findings(text, RID);
        let (f2, u2) = parse_findings(text, RID);
        assert_eq!(u1, u2);
        assert_eq!(f1.len(), f2.len());
        assert_eq!(f1[0].finding_id, f2[0].finding_id);
    }

    #[test]
    fn finding_yaml_entry_omits_evidence_why_scope_reason() {
        let text = "Finding: X / Severity: Medium / Classification: IN-SCOPE BLOCKER  \nEvidence: some evidence text / Why: some why text / Required action: fix now  \nScope reason: some scope text\n\nLOG SUMMARY: CHANGES ADVISED — x\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, _) = parse_findings(text, RID);
        let yaml = findings[0].to_yaml_entry();
        assert!(yaml.contains("finding_id:"));
        assert!(yaml.contains("severity: Medium"));
        assert!(yaml.contains("classification: IN-SCOPE BLOCKER"));
        assert!(yaml.contains("required_action: fix now"));
        assert!(!yaml.contains("some evidence text"), "evidence must not be serialized");
        assert!(!yaml.contains("some why text"), "why must not be serialized");
        assert!(!yaml.contains("some scope text"), "scope_reason must not be serialized");
    }

    #[test]
    fn parse_findings_corpus_regression_check() {
        // Runs the parser against every real historical assessment file in this repo's archive,
        // not a synthetic fixture. Deliberately avoids a hardcoded exact
        // finding count (the corpus grows with every review round, including this test's own
        // future runs); instead it asserts the invariant that actually matters: every
        // `Finding:` line in the real (pre-echo) region is accounted for as either parsed or
        // explicitly counted unparsed (never silently lost).
        //
        // This parser supports three real, currently-recurring `Evidence:`/`Why:`/
        // `Required action:`/`Scope reason:` shapes (see `parse_findings`'s own doc comment).
        // After adding all three, a residual ~7% of the corpus (23/317 real finding lines at
        // the time this test was written) remains unparseable — traced to one-off formatting
        // from the project's earliest sessions (e.g. a 4-label triage era, before the current
        // 5-category rule existed) and a small number of individually distinct anomalies that
        // do not share a common recurring shape. Chasing each one individually would be
        // unbounded scope creep for diminishing, non-recurring value; the fail-closed guardrail
        // (flag as unparseable, never silently drop) is what actually matters, and it holds.
        // The 15% ceiling below catches a *future* regression (a new systematic shape emerging
        // and going unsupported) without being brittle to this already-understood residual.
        let mut repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..4 { repo_root.pop(); }
        let codex_dir = repo_root.join("maintenance/archive/self-development/reviews/codex");

        let mut total_finding_lines = 0usize;
        let mut total_parsed = 0usize;
        let mut total_unparsed = 0usize;
        let mut files_checked = 0usize;

        for entry in std::fs::read_dir(&codex_dir).expect("read reviews/codex") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(&path).expect("read assessment");
            let body = content.splitn(2, "---\n\n").nth(1)
                .or_else(|| content.splitn(3, "---\n").nth(2))
                .unwrap_or(content.as_str());
            let real_region = body.split("\nLOG SUMMARY:").next().unwrap_or(body);
            total_finding_lines += real_region.lines().filter(|l| l.starts_with("Finding:")).count();

            let (findings, unparsed) = parse_findings(body, "REV__CORPUS-CHECK__x__R1");
            total_parsed += findings.len();
            total_unparsed += unparsed;
            files_checked += 1;
        }

        assert!(files_checked > 0, "expected historical assessments in the self-development archive");
        assert!(total_finding_lines > 0, "expected at least some real Finding: lines in the corpus");
        assert_eq!(
            total_parsed + total_unparsed, total_finding_lines,
            "every Finding: line in the real (pre-echo) region must be either parsed or counted unparsed \
             (parsed={}, unparsed={}, finding_lines={}, files={})",
            total_parsed, total_unparsed, total_finding_lines, files_checked
        );
        let unparsed_pct = (total_unparsed as f64 / total_finding_lines as f64) * 100.0;
        assert!(
            unparsed_pct <= 15.0,
            "unparsed rate {:.1}% ({}/{} findings across {} files) exceeds the known-residual \
             ceiling — investigate whether a new systematic (recurring) format shape appeared",
            unparsed_pct, total_unparsed, total_finding_lines, files_checked
        );
    }
}
