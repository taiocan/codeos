use crate::packet::{CoverageState, ReviewPacket};
use crate::run::{ReviewerRun, RunSource};
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
    /// `OK` when every finding the reply declared was parsed; `FAILED` when any was not.
    pub parse_status: String,
    /// `COMPLETE` only when the reply carried a usable verdict and every finding was parsed.
    /// `INCOMPLETE` forces the effective concern to DO NOT ADVANCE: a record that cannot show what
    /// the reviewer found must not be able to clear a boundary.
    pub assessment_status: String,
    /// Why the assessment is incomplete. Empty when it is complete.
    pub incomplete_reason: String,
    /// Present for a negative F7 evidence reassessment when the reviewer confirms the new
    /// canonical evidence addressed the named requirement even though it did not close it.
    pub evidence_materiality: Option<String>,
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
    #[allow(dead_code)]
    pub evidence: String,
    #[allow(dead_code)]
    pub why: String,
    #[allow(dead_code)]
    pub scope_reason: Option<String>,
    pub blocker_kind: Option<String>,
    pub evidence_required: Option<String>,
}

impl Finding {
    /// The compact YAML subset actually written to the assessment frontmatter — deliberately
    /// excludes `evidence`/`why`/`scope_reason` (full prose stays in the body, unduplicated).
    fn to_yaml_entry(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("    - finding_id: {}\n", self.finding_id));
        s.push_str(&format!("      severity: {}\n", self.severity));
        s.push_str(&format!("      classification: {}\n", self.classification));
        s.push_str(&format!(
            "      summary: \"{}\"\n",
            yaml_escape(&self.summary)
        ));
        if let Some(ac) = &self.acceptance_criterion {
            s.push_str(&format!("      acceptance_criterion: {}\n", ac));
        }
        s.push_str(&format!(
            "      required_action: {}\n",
            self.required_action
        ));
        if let Some(kind) = &self.blocker_kind {
            s.push_str(&format!("      blocker_kind: {}\n", kind));
        }
        if let Some(required) = &self.evidence_required {
            s.push_str(&format!(
                "      evidence_required: \"{}\"\n",
                yaml_escape(required)
            ));
        }
        s
    }
}

/// Minimal YAML double-quoted-string escaping (backslash and double-quote only — finding
/// summaries are reviewer prose, not attacker-controlled, but escaping is cheap and correct).
fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Parse canonical `Finding` / `Severity` / `Classification` fields from either the required
/// slash-delimited header or three consecutive field lines. Both normalize to the same structured
/// `Finding`; incomplete, conflicting, or non-canonical fields are never silently dropped.
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
///
/// The `Finding`/`Severity`/`Classification` header is likewise accepted in three equivalent
/// shapes: one slash-joined line (the prompt's form), three separate lines, and a two-line form
/// where the title stands alone and `Severity: X / Classification: Y` follows on one line. The
/// two-line form is what reviewers produce when a long title breaks the prompt's single line; it
/// failed whole reviews until it was accepted.
///
/// Markdown-decorated field lines (`1. **Finding:** t`, `   **Severity: High / Classification: Y**`)
/// are unwrapped by `normalize_field_line` before matching. Every accepted shape yields the same
/// `Finding`, and every missing, invalid, or duplicate field still leaves the block unparsed.
/// Field labels whose markdown wrapper `normalize_field_line` may remove.
const FIELD_LABELS: [&str; 9] = [
    "Finding",
    "Severity",
    "Classification",
    "Evidence required",
    "Evidence",
    "Why",
    "Required action",
    "Scope reason",
    "Blocker kind",
];

/// Remove one markdown emphasis wrapper that opens at `pos` with a field label in `labels`:
/// `**Label:** value` or `**Label: value**`. Only the opening marker and the next identical
/// marker are removed. An unclosed wrapper leaves the text unchanged, so it stays malformed.
fn unwrap_label_at(text: &mut String, pos: usize, labels: &[&str]) -> bool {
    for marker in ["**", "__"] {
        let rest = &text[pos..];
        let Some(after) = rest.strip_prefix(marker) else {
            continue;
        };
        if !labels.iter().any(|label| {
            after
                .strip_prefix(label)
                .is_some_and(|tail| tail.starts_with(':'))
        }) {
            return false;
        }
        let open_end = pos + marker.len();
        let Some(close) = text[open_end..].find(marker) else {
            return false;
        };
        text.replace_range(open_end + close..open_end + close + marker.len(), "");
        text.replace_range(pos..open_end, "");
        return true;
    }
    false
}

/// Lexical, field-local normalization of one markdown-decorated field line. A line changes only
/// when, after indentation and an optional `N. ` prefix, it opens with a wrapped field label; the
/// indentation and that wrapper are removed, and on a `Finding`/`Severity` line so are wrappers
/// on a following ` / Severity:` or ` / Classification:` segment. Field values are never edited,
/// and any other line, including undecorated indented text, is returned unchanged.
fn normalize_field_line(line: &str) -> String {
    let body = line.trim_start();
    let digits = body.len() - body.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let prefix_len = if digits > 0 && body[digits..].starts_with(". ") {
        digits + 2 + body[digits + 2..].len() - body[digits + 2..].trim_start().len()
    } else {
        0
    };
    let mut text = body.to_string();
    if !unwrap_label_at(&mut text, prefix_len, &FIELD_LABELS) {
        return line.to_string();
    }
    let opened = &text[prefix_len..];
    if opened.starts_with("Finding:") || opened.starts_with("Severity:") {
        let mut cursor = prefix_len;
        while let Some(offset) = text[cursor..].find(" / ") {
            let segment = cursor + offset + 3;
            unwrap_label_at(&mut text, segment, &["Severity", "Classification"]);
            cursor = segment;
        }
    }
    text
}

/// Whether a line opens a finding block: `Finding:` at the start, optionally numbered.
fn is_finding_header_line(line: &str) -> bool {
    let rest = line.trim_start_matches(|c: char| c.is_ascii_digit());
    line.starts_with("Finding:")
        || (rest.len() < line.len() && rest.trim_start_matches(['.', ' ']).starts_with("Finding:"))
}

pub fn parse_findings(raw_text: &str, review_id: &str) -> (Vec<Finding>, usize) {
    let real_region = raw_text.split("\nLOG SUMMARY:").next().unwrap_or(raw_text);
    let normalized: Vec<String> = real_region.lines().map(normalize_field_line).collect();
    let lines: Vec<&str> = normalized.iter().map(String::as_str).collect();

    // The optional `N. ` prefix is a real recurring shape: reviewers number their findings. It was
    // silently dropping those findings entirely — they matched no header, so they were neither
    // parsed nor counted unparsed.
    let combined_finding_re = Regex::new(
        r"^(?:\d+\.\s+)?Finding: (.+?) / Severity: (High|Medium|Low) / Classification: (.+?)\s*$",
    )
    .expect("valid regex");
    let finding_only_re = Regex::new(r"^(?:\d+\.\s+)?Finding: (.+?)\s*$").expect("valid regex");
    let severity_only_re = Regex::new(r"^Severity: (High|Medium|Low)\s*$").expect("valid regex");
    let classification_only_re = Regex::new(r"^Classification: (.+?)\s*$").expect("valid regex");
    let severity_classification_re =
        Regex::new(r"^Severity: (High|Medium|Low) / Classification: (.+?)\s*$")
            .expect("valid regex");
    // The action is one exact canonical token. A real reviewer shape follows it with a dash and an
    // explanation; only the token carries meaning, and any other wording stays malformed.
    let combined_re = Regex::new(r"^Evidence: (.+?) / Why: (.+?) / Required action: (fix now|optional fix|backlog|reject)(?:\s+[—–-]\s+\S.*?)?\s*$")
        .expect("valid regex");
    // A third real, historical shape: Evidence/Why/Required action/Scope reason all combined
    // onto one line (earliest corpus era, e.g. 2026-06-30/2026-07-01 rounds).
    let combined_with_scope_re = Regex::new(r"^Evidence: (.+?) / Why: (.+?) / Required action: (fix now|optional fix|backlog|reject) / Scope reason: (.+)$")
        .expect("valid regex");
    let evidence_only_re = Regex::new(r"^Evidence: (.+?)\s*$").expect("valid regex");
    let why_only_re = Regex::new(r"^Why: (.+?)\s*$").expect("valid regex");
    let action_only_re = Regex::new(
        r"^Required action: (fix now|optional fix|backlog|reject)(?:\s+[—–-]\s+\S.*?)?\s*$",
    )
    .expect("valid regex");
    let scope_re = Regex::new(r"^Scope reason: (.+)$").expect("valid regex");
    let blocker_kind_re =
        Regex::new(r"^Blocker kind: (SUBSTANTIVE|EVIDENCE_INSUFFICIENT)\s*$").expect("valid regex");
    let evidence_required_re = Regex::new(r"^Evidence required: (.+)$").expect("valid regex");
    let ac_re = Regex::new(r"AC-\d+").expect("valid regex");

    let mut findings = Vec::new();
    let mut unparsed_count = 0usize;
    let mut seq = 0u32;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if !is_finding_header_line(line) {
            i += 1;
            continue;
        }

        let header = if let Some(captures) = combined_finding_re.captures(line) {
            Some((
                captures[1].trim().to_string(),
                captures[2].to_string(),
                captures[3].trim().to_string(),
                1usize,
            ))
        } else if !line.contains(" / Severity:") && !line.contains(" / Classification:") {
            finding_only_re.captures(line).and_then(|finding| {
                if let Some(joined) = lines
                    .get(i + 1)
                    .and_then(|line| severity_classification_re.captures(line))
                {
                    return Some((
                        finding[1].trim().to_string(),
                        joined[1].to_string(),
                        joined[2].trim().to_string(),
                        2usize,
                    ));
                }
                let severity = lines
                    .get(i + 1)
                    .and_then(|line| severity_only_re.captures(line));
                let classification = lines
                    .get(i + 2)
                    .and_then(|line| classification_only_re.captures(line));
                match (severity, classification) {
                    (Some(severity), Some(classification)) => Some((
                        finding[1].trim().to_string(),
                        severity[1].to_string(),
                        classification[1].trim().to_string(),
                        3usize,
                    )),
                    _ => None,
                }
            })
        } else {
            None
        };
        let Some((summary, severity, classification, header_lines)) = header else {
            unparsed_count += 1;
            eprintln!("warning: malformed finding block (Finding/Severity/Classification fields did not match an accepted canonical shape): {}", line);
            i += 1;
            continue;
        };

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
        let mut blocker_kind: Option<String> = None;
        let mut evidence_required: Option<String> = None;
        let mut duplicate_header_field = false;
        let mut k = i + header_lines;
        let window_end = (k + 9).min(lines.len());
        while k < window_end {
            let l = lines[k];
            if l.trim().is_empty() || l.starts_with("Finding:") || l.starts_with("PR decision:") {
                break;
            }
            if l.starts_with("Severity:") || l.starts_with("Classification:") {
                duplicate_header_field = true;
                break;
            }
            if evidence.is_none()
                && why.is_none()
                && required_action.is_none()
                && scope_reason.is_none()
            {
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
                if let Some(c) = evidence_only_re.captures(l) {
                    evidence = Some(c[1].trim().to_string());
                    k += 1;
                    continue;
                }
            }
            if why.is_none() {
                if let Some(c) = why_only_re.captures(l) {
                    why = Some(c[1].trim().to_string());
                    k += 1;
                    continue;
                }
            }
            if required_action.is_none() {
                if let Some(c) = action_only_re.captures(l) {
                    required_action = Some(c[1].to_string());
                    k += 1;
                    continue;
                }
            }
            if scope_reason.is_none() {
                if let Some(c) = scope_re.captures(l) {
                    scope_reason = Some(c[1].trim().to_string());
                    k += 1;
                    continue;
                }
            }
            if blocker_kind.is_none() {
                if let Some(c) = blocker_kind_re.captures(l) {
                    blocker_kind = Some(c[1].to_string());
                    k += 1;
                    continue;
                }
            }
            if evidence_required.is_none() {
                if let Some(c) = evidence_required_re.captures(l) {
                    evidence_required = Some(c[1].trim().to_string());
                    k += 1;
                    continue;
                }
            }
            break;
        }

        let (Some(evidence), Some(why), Some(required_action)) = (evidence, why, required_action)
        else {
            unparsed_count += 1;
            eprintln!("warning: malformed finding block (Evidence/Why/Required action not all found in either supported shape) near: {}", line);
            i += 1;
            continue;
        };
        if duplicate_header_field {
            unparsed_count += 1;
            eprintln!(
                "warning: malformed finding block (duplicate or conflicting header field) near: {}",
                line
            );
            i += 1;
            continue;
        }

        seq += 1;
        let acceptance_criterion = ac_re.find(&summary).map(|m| m.as_str().to_string());
        findings.push(Finding {
            finding_id: format!("FND__{}__{:02}", review_id, seq),
            severity,
            classification,
            summary,
            acceptance_criterion,
            required_action,
            evidence,
            why,
            scope_reason,
            blocker_kind,
            evidence_required,
        });
        i = k;
    }

    (findings, unparsed_count)
}

/// Parse LOG SUMMARY, EVIDENCE, HIGHEST-IMPACT UNCERTAINTY from reviewer output.
/// Count finding declarations the reply made, however it shaped them. `parse_findings` accepts one
/// accepted canonical header shape; a model that declares findings in any other shape would otherwise leave
/// `findings: []` with nothing recorded as dropped. Every canonical classification label the reply
/// applies is one declared finding, so comparing that count against what parsed turns a silent loss
/// into a visible one.
pub fn count_declared_findings(raw_text: &str) -> usize {
    let body = match raw_text.find("\nLOG SUMMARY:") {
        Some(index) => &raw_text[..index],
        None => raw_text,
    };
    body.lines()
        .filter(|line| {
            if !line.contains("Classification:") || line.contains('|') {
                // A `|` means the line enumerates the allowed labels — the reviewer prompt echoed
                // back into the transcript, not a finding the reviewer made.
                return false;
            }
            CANONICAL_CLASSIFICATIONS
                .iter()
                .filter(|label| line.contains(*label))
                .count()
                == 1
        })
        .count()
}

pub fn parse_review_output(
    output: &str,
    coverage_state: &CoverageState,
    parsed_findings: usize,
    unparsed_findings: usize,
) -> ParsedReview {
    let summary_line = output
        .lines()
        .rfind(|l| l.starts_with("LOG SUMMARY:"))
        .map(|l| l.to_string())
        .unwrap_or_default();

    let codex_concern = if summary_line.is_empty() {
        "UNCLASSIFIED".to_string()
    } else {
        let rest = summary_line.trim_start_matches("LOG SUMMARY:").trim();
        let concern_end = rest.find(" —").unwrap_or(rest.len());
        normalize_concern(&rest[..concern_end])
    };

    let evidence_raw = output
        .lines()
        .rfind(|l| l.starts_with("EVIDENCE:"))
        .map(|l| l.trim_start_matches("EVIDENCE:").trim().to_string())
        .unwrap_or_default();

    let evidence = match evidence_raw.chars().next() {
        Some(c) if "ABCDE".contains(c) => c.to_string(),
        _ if evidence_raw == "not reported" => "not reported".to_string(),
        _ if evidence_raw.is_empty() => "not reported".to_string(),
        _ => "not reported".to_string(),
    };

    let highest_impact_uncertainty = output
        .lines()
        .rfind(|l| l.starts_with("HIGHEST-IMPACT UNCERTAINTY:"))
        .map(|l| {
            l.trim_start_matches("HIGHEST-IMPACT UNCERTAINTY:")
                .trim()
                .to_string()
        })
        .unwrap_or_default();
    let evidence_materiality = output
        .lines()
        .filter_map(|line| line.strip_prefix("EVIDENCE MATERIALITY: "))
        .next_back()
        .filter(|value| matches!(*value, "MATERIAL" | "NOT_MATERIAL"))
        .map(str::to_string);

    // Fail closed on anything that means the record cannot show what the reviewer actually found.
    let declared = count_declared_findings(output);
    let accounted = parsed_findings + unparsed_findings;
    let unaccounted = declared.saturating_sub(accounted);
    let parse_status = if unparsed_findings > 0 || unaccounted > 0 {
        "FAILED"
    } else {
        "OK"
    };
    let incomplete_reason = if unaccounted > 0 {
        format!("{unaccounted} declared finding(s) did not match the required finding format and were not recorded")
    } else if unparsed_findings > 0 {
        format!("{unparsed_findings} finding block(s) were malformed and could not be recorded")
    } else if summary_line.is_empty() {
        "the reply carried no LOG SUMMARY verdict line".to_string()
    } else if codex_concern == "UNCLASSIFIED" {
        "the reply's verdict was not one of the four recognised concerns".to_string()
    } else if evidence == "not reported" {
        "the reply carried no EVIDENCE grade".to_string()
    } else {
        String::new()
    };
    let assessment_status = if incomplete_reason.is_empty() {
        "COMPLETE"
    } else {
        "INCOMPLETE"
    };

    // Compute effective concern
    let floor = coverage_state.concern_floor();
    let concern_rank = concern_to_rank(&codex_concern);
    // An incomplete assessment cannot clear a boundary, whichever model produced it.
    let incomplete_floor = if assessment_status == "INCOMPLETE" {
        3
    } else {
        0
    };
    let eff_rank = concern_rank.max(floor).max(incomplete_floor);
    let effective_concern = rank_to_concern(eff_rank);

    let coverage_note = if eff_rank > concern_rank && incomplete_floor == 3 {
        format!("raised to DO NOT ADVANCE: assessment INCOMPLETE — {incomplete_reason}")
    } else if eff_rank > concern_rank {
        format!(
            "raised from '{}' to the coverage floor for {}",
            codex_concern,
            coverage_state.as_str()
        )
    } else {
        String::new()
    };

    let final_summary_line = if summary_line.is_empty() {
        "LOG SUMMARY: UNCLASSIFIED — no parseable summary; HIGH attention, manual review required"
            .to_string()
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
        parse_status: parse_status.to_string(),
        assessment_status: assessment_status.to_string(),
        incomplete_reason,
        evidence_materiality,
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
        "NO OBJECTION" => 0,
        "CHANGES ADVISED" => 1,
        "UNCLASSIFIED" => 2,
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
    }
    .to_string()
}

/// Write the assessment file and return (assessment_path, assessment_sha256).
#[allow(clippy::too_many_arguments)]
pub fn write_assessment(
    review_id: &str,
    findings: &[Finding],
    unparsed_findings_count: usize,
    packet: &ReviewPacket,
    raw: &ReviewerRun,
    parsed: &ParsedReview,
    outdir: &Path,
    packet_saved: &Path,
    packet_hash: &str,
) -> Result<(PathBuf, String)> {
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let short_sha = &packet.review_sha[..7.min(packet.review_sha.len())];
    let ts_clean = ts.replace(':', "");
    let closure_suffix = if review_id.contains("__TERMINAL-CLOSURE-") {
        format!("-{}", &crate::packet::sha256_str(review_id)[..8])
    } else {
        String::new()
    };
    let filename = format!(
        "{}-{}-stage-{}-{}{}.md",
        ts_clean, packet.feature, packet.stage, short_sha, closure_suffix
    );
    let assessment_file = outdir.join(&filename);

    std::fs::create_dir_all(outdir).context("create assessment dir")?;

    let mut content = String::new();
    content.push_str("---\n");
    content.push_str(&format!("review_id: {}\n", review_id));
    if review_id.contains("__TERMINAL-CLOSURE-FAILURE-") {
        content.push_str("counts_as_semantic_closure: false\n");
    }
    if findings.is_empty() {
        content.push_str("findings: []\n");
    } else {
        content.push_str("findings:\n");
        for f in findings {
            content.push_str(&f.to_yaml_entry());
        }
    }
    content.push_str(&format!(
        "unparsed_findings_count: {}\n",
        unparsed_findings_count
    ));
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
    content.push_str(&format!(
        "  coverage_state: {}\n",
        packet.coverage_state.as_str()
    ));
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
    let packet_basename = packet_saved
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    content.push_str(&format!("  reviewed_packet: packets/{}\n", packet_basename));
    content.push_str(&format!("  reviewed_packet_sha256: {}\n", packet_hash));
    if let Some(manifest) = &packet.manifest {
        content.push_str(&format!(
            "  reviewed_snapshot: {}\n",
            manifest.reviewed_snapshot
        ));
        if let Some(bundle) = &manifest.candidate_bundle {
            content.push_str(&format!("  candidate_bundle: {}\n", bundle));
        }
        // An external assessment is advisory (Review Policy v16); it never carries an outcome.
        if manifest.candidate_bundle.is_some() && !raw.is_external() {
            let outcome = if parsed.parse_status == "OK"
                && parsed.assessment_status == "COMPLETE"
                && parsed.effective_concern == "NO OBJECTION"
                && !findings
                    .iter()
                    .any(|finding| finding.classification == "IN-SCOPE BLOCKER")
            {
                "PASSED"
            } else {
                "BLOCKED"
            };
            content.push_str(&format!("  candidate_outcome: {}\n", outcome));
        }
    }
    content.push_str(&format!(
        "  source: {}\n",
        if raw.is_external() {
            "external"
        } else {
            "codex"
        }
    ));
    content.push_str(&format!("  reviewer: \"{}\"\n", raw.reviewer_field()));
    // The key names who reported the concern. `codex_concern` is the established name across
    // existing review records and stays; an external assessment gets a name that is true of it.
    content.push_str(&format!(
        "  {}: {}\n",
        if raw.is_external() {
            "reported_concern"
        } else {
            "codex_concern"
        },
        parsed.codex_concern
    ));
    content.push_str(&format!(
        "  effective_concern: {}\n",
        parsed.effective_concern
    ));
    if !parsed.coverage_note.is_empty() {
        content.push_str(&format!(
            "  effective_concern_note: {}\n",
            serde_json::to_string(&parsed.coverage_note)
                .context("serialize effective concern note")?
        ));
    }
    content.push_str(&format!("  evidence: {}\n", parsed.evidence));
    content.push_str(&format!("  parse_status: {}\n", parsed.parse_status));
    content.push_str(&format!(
        "  assessment_status: {}\n",
        parsed.assessment_status
    ));
    if !parsed.incomplete_reason.is_empty() {
        content.push_str(&format!(
            "  incomplete_reason: \"{}\"\n",
            yaml_escape(&parsed.incomplete_reason)
        ));
    }
    if let Some(materiality) = &parsed.evidence_materiality {
        content.push_str(&format!("  evidence_materiality: {}\n", materiality));
    }
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
    // Effort, reconnects, and wall time describe a process this tool ran. An external assessment
    // has none of them, and inventing zeroes would read as measurement rather than absence.
    match &raw.source {
        RunSource::Codex {
            reconnect_count,
            effort,
            model,
            cli_version,
            ..
        } => {
            content.push_str(&format!("  reasoning_effort: {}\n", effort));
            content.push_str(&format!("  model: {}\n", model));
            content.push_str(&format!("  codex_cli_version: {}\n", cli_version));
            content.push_str(&format!(
                "  reviewer_build_sha256: {}\n",
                env!("CODEOS_REVIEWER_BUILD_SHA256")
            ));
            content.push_str(&format!(
                "  codeos_release_id: {}\n",
                env!("CODEOS_RELEASE_ID")
            ));
            content.push_str(&format!("  reconnect_count: {}\n", reconnect_count));
            content.push_str(&format!("  elapsed_ms: {}\n", raw.elapsed_ms));
        }
        RunSource::External { .. } => {
            content.push_str("  counts_as_review_round: false\n");
        }
    }
    content.push_str("---\n\n");
    content.push_str(&raw.text);

    std::fs::write(&assessment_file, &content).with_context(|| {
        format!(
            "could not write assessment to {}",
            assessment_file.display()
        )
    })?;

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
    if packet.feature.is_empty() {
        errs.push("missing:feature");
    }
    if packet.stage.is_empty() {
        errs.push("missing:stage");
    }
    if packet.base_sha.is_empty() {
        errs.push("missing:base_commit");
    }
    if packet.review_sha.is_empty() {
        errs.push("missing:review_commit");
    }
    if packet.diff_hash.is_empty() {
        errs.push("missing:diff_hash");
    }
    if packet_hash.is_empty() {
        errs.push("missing:reviewed_packet_sha256");
    }

    let valid_coverage = [
        "FULL_COVERAGE",
        "PARTIAL_COVERAGE",
        "SECRET_REDACTION",
        "CRITICAL_OMISSION",
        "EMPTY_PACKET",
    ];
    if !valid_coverage.contains(&packet.coverage_state.as_str()) {
        errs.push("enum:coverage_state");
    }
    let valid_reported_concern = [
        "NO OBJECTION",
        "CHANGES ADVISED",
        "DO NOT ADVANCE",
        "UNCLASSIFIED",
    ];
    if !valid_reported_concern.contains(&parsed.codex_concern.as_str()) {
        errs.push("enum:codex_concern");
    }
    let valid_effective_concern = [
        "NO OBJECTION",
        "CHANGES ADVISED",
        "DO NOT ADVANCE",
        "UNCLASSIFIED",
        "EVIDENCE CLOSED — PROGRESSION PERMITTED",
        "INFRASTRUCTURE EVIDENCE REVALIDATED — PROGRESSION PERMITTED",
        "TERMINAL REVIEW CLOSED — PROGRESSION PERMITTED",
        "LINEAGE REPLACEMENT CLOSED — PROGRESSION PERMITTED",
        "LINEAGE REPLACEMENT CORRECTION CLOSED — PROGRESSION PERMITTED",
    ];
    if !valid_effective_concern.contains(&parsed.effective_concern.as_str()) {
        errs.push("enum:effective_concern");
    }
    let valid_evidence = ["A", "B", "C", "D", "E", "not reported"];
    if !valid_evidence.contains(&parsed.evidence.as_str()) {
        errs.push("enum:evidence");
    }
    if packet.artifacts.is_empty()
        && !matches!(
            packet.coverage_state,
            CoverageState::CriticalOmission | CoverageState::EmptyPacket
        )
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
        parse_review_output(text, &CoverageState::FullCoverage, 0, 0)
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

    fn make_raw(text: &str) -> ReviewerRun {
        ReviewerRun {
            text: text.to_string(),
            elapsed_ms: 1000,
            source: RunSource::Codex {
                session_id: "test-session".to_string(),
                reconnect_count: 0,
                effort: "high".to_string(),
                model: "test-model".to_string(),
                cli_version: "0.162.0".to_string(),
            },
        }
    }

    #[allow(dead_code)]
    fn make_external_raw(text: &str, label: Option<&str>) -> ReviewerRun {
        ReviewerRun {
            text: text.to_string(),
            elapsed_ms: 0,
            source: RunSource::External {
                label: label.map(str::to_string),
            },
        }
    }

    #[test]
    fn parses_no_objection() {
        let text = "Some findings.\n\nPR decision: ADVANCE\nLOG SUMMARY: NO OBJECTION — all good\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let parsed = parse_review_output(text, &CoverageState::FullCoverage, 0, 0);
        assert_eq!(parsed.codex_concern, "NO OBJECTION");
        assert_eq!(parsed.effective_concern, "NO OBJECTION");
        assert_eq!(parsed.evidence, "A");
    }

    #[test]
    fn parses_changes_advised() {
        let text = "LOG SUMMARY: CHANGES ADVISED — fix required\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: something\n";
        let parsed = parse_review_output(text, &CoverageState::FullCoverage, 0, 0);
        assert_eq!(parsed.codex_concern, "CHANGES ADVISED");
        assert_eq!(parsed.evidence, "B");
    }

    #[test]
    fn coverage_floor_escalates_concern() {
        let text = "LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: x\n";
        let parsed = parse_review_output(text, &CoverageState::CriticalOmission, 0, 0);
        assert_eq!(parsed.codex_concern, "NO OBJECTION");
        assert_eq!(parsed.effective_concern, "DO NOT ADVANCE");
        assert!(!parsed.coverage_note.is_empty());
    }

    #[test]
    fn assessment_writer_quotes_effective_concern_note_as_valid_yaml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo_root = dir.path();
        for args in [
            ["init"].as_slice(),
            ["config", "user.email", "t@t.test"].as_slice(),
            ["config", "user.name", "T"].as_slice(),
        ] {
            assert!(std::process::Command::new("git")
                .args(args)
                .current_dir(repo_root)
                .status()
                .expect("git command")
                .success());
        }
        std::fs::write(repo_root.join("tracked.md"), "# tracked\n").expect("write");
        assert!(std::process::Command::new("git")
            .args(["add", "tracked.md"])
            .current_dir(repo_root)
            .status()
            .expect("git add")
            .success());
        assert!(std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(repo_root)
            .status()
            .expect("git commit")
            .success());

        let mut toolkit_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..4 {
            toolkit_root.pop();
        }
        let packet = crate::packet::build(&crate::packet::PacketBuildOptions {
            feature: "UPG-TEST".to_string(),
            stage: "selfdev-step-1".to_string(),
            artifacts: vec!["tracked.md".to_string()],
            sha_only_paths: vec![],
            basis: crate::packet::ReviewBasis::Ordinary { base: None },
            repo_root: repo_root.to_string_lossy().into_owned(),
            toolkit_root: toolkit_root.to_string_lossy().into_owned(),
            scope: None,
            inclusion: Default::default(),
            route_geometry_projection: false,
        })
        .expect("build packet");
        let raw = make_raw(
            "LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n",
        );
        let mut parsed = parse_review_output(&raw.text, &packet.coverage_state, 0, 0);
        parsed.coverage_note =
            "raised to DO NOT ADVANCE: assessment INCOMPLETE — exact historical shape".into();
        let outdir = dir.path().join("codex-out");
        let (path, _) = write_assessment(
            "REV__UPG-TEST__selfdev-step-1__R1",
            &[],
            0,
            &packet,
            &raw,
            &parsed,
            &outdir,
            Path::new("packet.txt"),
            "cafefeed",
        )
        .expect("write assessment");
        let content = std::fs::read_to_string(path).expect("read assessment");
        let frontmatter = content
            .strip_prefix("---\n")
            .unwrap()
            .split_once("\n---")
            .unwrap()
            .0;
        let yaml: serde_yaml::Value = serde_yaml::from_str(frontmatter).expect("valid YAML");
        assert_eq!(
            yaml["reviewed"]["effective_concern_note"].as_str(),
            Some(parsed.coverage_note.as_str())
        );
        assert!(content.contains(
            "  effective_concern_note: \"raised to DO NOT ADVANCE: assessment INCOMPLETE"
        ));
    }

    #[test]
    fn missing_summary_gives_unclassified() {
        let text = "No summary line here.\n";
        let parsed = parse_review_output(text, &CoverageState::FullCoverage, 0, 0);
        assert_eq!(parsed.codex_concern, "UNCLASSIFIED");
        assert!(parsed.summary_line.contains("UNCLASSIFIED"));
    }

    #[test]
    fn write_assessment_includes_review_id_in_frontmatter() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo_root = dir.path();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_root)
            .output()
            .expect("git init");
        std::process::Command::new("git")
            .args(["config", "user.email", "t@t.test"])
            .current_dir(repo_root)
            .output()
            .ok();
        std::process::Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(repo_root)
            .output()
            .ok();
        std::fs::write(repo_root.join("tracked.md"), "# tracked\n").expect("write");
        std::process::Command::new("git")
            .args(["add", "tracked.md"])
            .current_dir(repo_root)
            .output()
            .expect("git add");
        std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(repo_root)
            .output()
            .expect("git commit");

        let mut toolkit_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..4 {
            toolkit_root.pop();
        }

        let opts = crate::packet::PacketBuildOptions {
            feature: "UPG-TEST".to_string(),
            stage: "selfdev-step-1".to_string(),
            artifacts: vec!["tracked.md".to_string()],
            sha_only_paths: vec![],
            basis: crate::packet::ReviewBasis::Ordinary { base: None },
            repo_root: repo_root.to_string_lossy().into_owned(),
            toolkit_root: toolkit_root.to_string_lossy().into_owned(),
            scope: None,
            inclusion: Default::default(),
            route_geometry_projection: false,
        };
        let packet = crate::packet::build(&opts).expect("build packet");
        let raw = make_raw(
            "LOG SUMMARY: NO OBJECTION — ok\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n",
        );
        let parsed = parse_review_output(&raw.text, &packet.coverage_state, 0, 0);

        let outdir = dir.path().join("codex-out");
        let review_id = "REV__UPG-TEST__selfdev-step-1__R1";
        let (assessment_file, _hash) = write_assessment(
            review_id,
            &[],
            0,
            &packet,
            &raw,
            &parsed,
            &outdir,
            Path::new("packet.txt"),
            "cafefeed",
        )
        .expect("write_assessment");

        let content = std::fs::read_to_string(&assessment_file).expect("read assessment");
        assert!(
            content.starts_with(&format!("---\nreview_id: {}\n", review_id)),
            "review_id must be the first frontmatter field: {}",
            content
        );

        // AC-5: review_id is a content field only — the filename keeps its pre-existing
        // <ts>-<feature>-stage-<stage>-<sha>.md shape, never a REV__... shape.
        let filename = assessment_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        assert!(
            !filename.starts_with("REV__"),
            "assessment filename must not be renamed to the REV__ shape: {}",
            filename
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

    /// Real shape from the JamFree F-0003 DBA-22 pilot: each field on its own line and the exact
    /// action token followed by an explanation.
    #[test]
    fn parse_findings_accepts_exact_action_token_with_explanation() {
        let text = "Finding: Production evidence reports the wrong observed journey. / Severity: Medium / Classification: IN-SCOPE BLOCKER  \nEvidence: `scripts/verify-production-claims.mjs:98`.  \nWhy: `details.journey` states the wrong route.  \nRequired action: fix now — derive the journey description from the observed route labels.  \nScope reason: Truthful evidence belongs to the stated verification output.  \nBlocker kind: SUBSTANTIVE\n\nLOG SUMMARY: CHANGES ADVISED — correct the evidence\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert_eq!(unparsed, 0);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].required_action, "fix now");
        assert_eq!(findings[0].blocker_kind.as_deref(), Some("SUBSTANTIVE"));

        let combined = "Finding: x / Severity: Low / Classification: IN-SCOPE NON-BLOCKER\nEvidence: e / Why: w / Required action: backlog – track separately\nScope reason: s\n\nLOG SUMMARY: NO OBJECTION — fine\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n";
        let (findings, unparsed) = parse_findings(combined, RID);
        assert_eq!((findings.len(), unparsed), (1, 0));
        assert_eq!(findings[0].required_action, "backlog");
    }

    #[test]
    fn parse_findings_rejects_non_canonical_action_wording() {
        for action in ["fix later — after release", "fix now-ish", "must fix now"] {
            let text = format!("Finding: x / Severity: Low / Classification: IN-SCOPE NON-BLOCKER\nEvidence: e\nWhy: w\nRequired action: {action}\nScope reason: s\n\nLOG SUMMARY: NO OBJECTION — fine\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: none\n");
            let (findings, unparsed) = parse_findings(&text, RID);
            assert_eq!((findings.len(), unparsed), (0, 1), "{action}");
        }
    }

    #[test]
    fn sr3_reentry_multiline_header_normalizes_like_the_required_single_line() {
        let single = "Finding: Departure correspondence ignores the origin’s timezone / Severity: High / Classification: IN-SCOPE BLOCKER\nEvidence: backend/src/provider.rs:421 / Why: the represented instant is not validated in the origin timezone / Required action: fix now\nScope reason: explicit targeted repair\nBlocker kind: SUBSTANTIVE\n\nLOG SUMMARY: CHANGES ADVISED — departure validation incomplete\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: provider offset guarantee\n";
        let multiline = "Finding: Departure correspondence ignores the origin’s timezone  \nSeverity: High\nClassification: IN-SCOPE BLOCKER\nEvidence: backend/src/provider.rs:421\nWhy: the represented instant is not validated in the origin timezone\nRequired action: fix now\nScope reason: explicit targeted repair\nBlocker kind: SUBSTANTIVE\n\nLOG SUMMARY: CHANGES ADVISED — departure validation incomplete\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: provider offset guarantee\n";

        let (single_findings, single_unparsed) = parse_findings(single, RID);
        let (multiline_findings, multiline_unparsed) = parse_findings(multiline, RID);

        assert_eq!(single_unparsed, 0);
        assert_eq!(multiline_unparsed, 0);
        assert_eq!(single_findings.len(), 1);
        assert_eq!(multiline_findings.len(), 1);
        let single = &single_findings[0];
        let multiline = &multiline_findings[0];
        assert_eq!(multiline.finding_id, single.finding_id);
        assert_eq!(multiline.summary, single.summary);
        assert_eq!(multiline.severity, single.severity);
        assert_eq!(multiline.classification, single.classification);
        assert_eq!(multiline.required_action, single.required_action);
        assert_eq!(multiline.evidence, single.evidence);
        assert_eq!(multiline.why, single.why);
        assert_eq!(multiline.scope_reason, single.scope_reason);
        assert_eq!(multiline.blocker_kind, single.blocker_kind);
    }

    #[test]
    fn every_accepted_header_and_decoration_shape_parses_to_the_same_finding() {
        let tail =
            "\n\nLOG SUMMARY: DO NOT ADVANCE — x\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: u\n";
        let shapes = [
            // Canonical single line.
            "Finding: Snapshot activation accepts incomplete membership / Severity: High / Classification: IN-SCOPE BLOCKER\nEvidence: `lib.rs:263` — `verify_snapshot`\nWhy: empty maps pass / verification\nRequired action: fix now\nScope reason: explicit guarantee",
            // Three separate lines.
            "Finding: Snapshot activation accepts incomplete membership\nSeverity: High\nClassification: IN-SCOPE BLOCKER\nEvidence: `lib.rs:263` — `verify_snapshot`\nWhy: empty maps pass / verification\nRequired action: fix now\nScope reason: explicit guarantee",
            // Two-line header (AII F-0001 Stage 8, attempts 1-2).
            "Finding: Snapshot activation accepts incomplete membership  \nSeverity: High / Classification: IN-SCOPE BLOCKER  \nEvidence: `lib.rs:263` — `verify_snapshot`  \nWhy: empty maps pass / verification  \nRequired action: fix now  \nScope reason: explicit guarantee",
            // Labels bolded (AII 20261122T213124Z attempt 3).
            "1. **Finding:** Snapshot activation accepts incomplete membership  \n   **Severity:** High / **Classification:** IN-SCOPE BLOCKER  \n   **Evidence:** `lib.rs:263` — `verify_snapshot`  \n   **Why:** empty maps pass / verification  \n   **Required action:** fix now  \n   **Scope reason:** explicit guarantee",
            // Whole fields bolded (AII 20261122T213930Z attempt 3).
            "1. **Finding: Snapshot activation accepts incomplete membership**  \n   **Severity: High / Classification: IN-SCOPE BLOCKER**  \n   **Evidence:** `lib.rs:263` — `verify_snapshot`  \n   **Why:** empty maps pass / verification  \n   **Required action: fix now**  \n   **Scope reason:** explicit guarantee",
        ];
        let parsed: Vec<_> = shapes
            .iter()
            .map(|shape| {
                let text = format!("{shape}{tail}");
                let (findings, unparsed) = parse_findings(&text, RID);
                assert_eq!((findings.len(), unparsed), (1, 0), "{shape}");
                let parsed = parse_review_output(&text, &CoverageState::FullCoverage, 1, 0);
                assert_eq!(parsed.parse_status, "OK", "{shape}");
                assert_eq!(parsed.assessment_status, "COMPLETE", "{shape}");
                findings[0].to_yaml_entry()
                    + &format!(
                        "{:?}",
                        (
                            &findings[0].evidence,
                            &findings[0].why,
                            &findings[0].scope_reason
                        )
                    )
            })
            .collect();
        for other in &parsed[1..] {
            assert_eq!(other, &parsed[0]);
        }
        assert!(parsed[0].contains("empty maps pass / verification"));
    }

    #[test]
    fn decorated_and_two_line_headers_remain_fail_closed() {
        for text in [
            // Two-line form: invalid severity, non-canonical label, missing classification.
            "Finding: t\nSeverity: Critical / Classification: IN-SCOPE BLOCKER\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            "Finding: t\nSeverity: High / Classification: MAYBE\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            "Finding: t\nSeverity: High /\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            // Two-line form followed by a competing field.
            "Finding: t\nSeverity: High / Classification: IN-SCOPE BLOCKER\nClassification: REJECTED\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            "Finding: t\nSeverity: High / Classification: IN-SCOPE BLOCKER\nSeverity: Low\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            // Duplicates that only become visible after unwrapping.
            "**Finding:** t\n**Severity:** High / **Classification:** IN-SCOPE BLOCKER\n**Severity:** Low\n**Evidence:** e\n**Why:** w\n**Required action:** fix now\n",
            "**Finding:** t\n**Severity: High**\n**Classification: IN-SCOPE BLOCKER**\n   **Classification:** REJECTED\n**Evidence:** e\n**Why:** w\n**Required action:** fix now\n",
            // Unclosed wrapper stays malformed rather than being guessed.
            "**Finding: t\n**Severity: High / Classification: IN-SCOPE BLOCKER\n**Evidence:** e\n**Why:** w\n**Required action:** fix now\n",
            // Decorated action must still be one canonical token.
            "**Finding:** t\n**Severity:** High / **Classification:** IN-SCOPE BLOCKER\n**Evidence:** e\n**Why:** w\n**Required action:** fix it soon\n",
        ] {
            // An unclosed wrapper is not a header at all; the declared-finding count still
            // catches it, so the review-level status is the guard asserted for every case.
            let text = format!("{text}\nLOG SUMMARY: DO NOT ADVANCE — x\nEVIDENCE: B\n");
            let (findings, unparsed) = parse_findings(&text, RID);
            let parsed =
                parse_review_output(&text, &CoverageState::FullCoverage, findings.len(), unparsed);
            assert!(findings.is_empty(), "{text}");
            assert_eq!(parsed.parse_status, "FAILED", "{text}");
            assert_eq!(parsed.assessment_status, "INCOMPLETE", "{text}");
        }
    }

    #[test]
    fn normalization_leaves_non_field_and_undecorated_lines_unchanged() {
        for line in [
            "    Finding: / Severity: High|Medium|Low / Classification: <one of the TRIAGE RULE labels>",
            "   Evidence: indented but undecorated",
            "**Note:** not a field",
            "| **Finding:** | table cell |",
            "Why: value with **bold** inside",
        ] {
            assert_eq!(normalize_field_line(line), line);
        }
        assert_eq!(
            normalize_field_line("   **Why:** value with **bold** / **Classification:** inside"),
            "Why: value with **bold** / **Classification:** inside"
        );
    }

    #[test]
    fn multiline_header_remains_fail_closed_for_missing_invalid_or_duplicate_fields() {
        for text in [
            "Finding: missing severity\nClassification: IN-SCOPE BLOCKER\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            "Finding: invalid severity\nSeverity: Critical\nClassification: IN-SCOPE BLOCKER\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            "Finding: invalid classification\nSeverity: High\nClassification: MAYBE\nEvidence: e\nWhy: w\nRequired action: fix now\n",
            "Finding: conflicting severity\nSeverity: High\nClassification: IN-SCOPE BLOCKER\nSeverity: Low\nEvidence: e\nWhy: w\nRequired action: fix now\n",
        ] {
            let (findings, unparsed) = parse_findings(text, RID);
            assert!(findings.is_empty(), "{text}");
            assert!(unparsed > 0, "{text}");
        }
    }

    #[test]
    fn parses_f7_blocker_kind_and_evidence_requirement() {
        let text = "Finding: Runtime failures are not proven / Severity: High / Classification: IN-SCOPE BLOCKER\nEvidence: bounded set has success only\nWhy: five failure outcomes are absent\nRequired action: fix now\nScope reason: final verification requires the missing proof\nBlocker kind: EVIDENCE_INSUFFICIENT\nEvidence required: canonical failure events for all five classifications\n\nPR decision: DO NOT ADVANCE\nLOG SUMMARY: DO NOT ADVANCE — evidence missing\nEVIDENCE: A\nHIGHEST-IMPACT UNCERTAINTY: failure conformance\n";
        let (findings, unparsed) = parse_findings(text, RID);
        assert_eq!(unparsed, 0);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].blocker_kind.as_deref(),
            Some("EVIDENCE_INSUFFICIENT")
        );
        assert_eq!(
            findings[0].evidence_required.as_deref(),
            Some("canonical failure events for all five classifications")
        );
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
            assert_eq!(
                findings.len(),
                1,
                "label {} should produce exactly one finding",
                label
            );
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
        assert!(
            findings.is_empty(),
            "a non-canonical classification must not produce a finding"
        );
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
        assert_eq!(
            unparsed, 1,
            "a malformed Finding line must be counted, not silently dropped"
        );
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
        assert!(findings[0]
            .evidence
            .contains("smoke_review_id_first_round_is_r1"));
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
        assert_eq!(
            findings.len(),
            1,
            "the echoed transcript must not double-count the same finding"
        );
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
        assert!(
            !yaml.contains("some evidence text"),
            "evidence must not be serialized"
        );
        assert!(
            !yaml.contains("some why text"),
            "why must not be serialized"
        );
        assert!(
            !yaml.contains("some scope text"),
            "scope_reason must not be serialized"
        );
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
        for _ in 0..4 {
            repo_root.pop();
        }
        let codex_dir = repo_root.join("maintenance/archive/self-development/reviews/codex");

        let mut total_finding_lines = 0usize;
        let mut total_parsed = 0usize;
        let mut total_unparsed = 0usize;
        let mut files_checked = 0usize;
        let mut files_with_unaccounted = 0usize;
        let mut total_unaccounted = 0usize;

        for entry in std::fs::read_dir(&codex_dir).expect("read reviews/codex") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(&path).expect("read assessment");
            let body = content
                .split_once("---\n\n")
                .map(|(_, body)| body)
                .or_else(|| content.splitn(3, "---\n").nth(2))
                .unwrap_or(content.as_str());
            let real_region = body.split("\nLOG SUMMARY:").next().unwrap_or(body);
            // Uses the parser's own header rule, so the numbered shape counts on both sides.
            total_finding_lines += real_region
                .lines()
                .filter(|l| is_finding_header_line(l))
                .count();

            let (findings, unparsed) = parse_findings(body, "REV__CORPUS-CHECK__x__R1");
            // The fail-closed rule keys off `count_declared_findings`. If that over-counted on real
            // Codex output, every historical review would retroactively be INCOMPLETE — so assert
            // here that it never claims more findings than the parser accounted for.
            let declared = count_declared_findings(body);
            if declared > findings.len() + unparsed {
                files_with_unaccounted += 1;
                total_unaccounted += declared - (findings.len() + unparsed);
            }
            total_parsed += findings.len();
            total_unparsed += unparsed;
            files_checked += 1;
        }

        eprintln!(
            "corpus: {files_checked} files, {total_finding_lines} finding lines, \
             {files_with_unaccounted} files with unaccounted declarations, {total_unaccounted} unaccounted"
        );
        assert!(
            files_checked > 0,
            "expected historical assessments in the self-development archive"
        );
        // Findings declared in a shape no header rule accepts. On the corpus at the time of writing
        // this is 2 of 510 files — two genuine, individually distinct prose declarations that the
        // record silently lost. The ceiling catches a *systematic* new shape emerging (which would
        // start marking real Codex reviews INCOMPLETE) without being brittle to that residual.
        assert!(
            files_with_unaccounted * 100 <= files_checked * 2,
            "too many files declare findings in an unrecognised shape: {files_with_unaccounted}/{files_checked} ({total_unaccounted} findings)"
        );
        assert!(
            total_finding_lines > 0,
            "expected at least some real Finding: lines in the corpus"
        );
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
            unparsed_pct,
            total_unparsed,
            total_finding_lines,
            files_checked
        );
    }
}
