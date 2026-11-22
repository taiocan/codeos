//! `review-status`: a read-only projection of review cost, derived only from `review-log.md` and
//! the packets, sidecars, and assessments it references. It writes nothing and decides nothing.

use crate::accounting::{self, Source};
use crate::config::Config;
use crate::packet::{sha256_file, ReviewPacket};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Composition, hotspots, and elapsed time describe this many most recent assessments.
const RECENT: usize = 20;
/// A p95 over fewer samples is not meaningful.
const P95_MIN_SAMPLES: usize = 10;

#[derive(Debug, Serialize)]
pub struct Status {
    pub health: &'static str,
    pub incomplete: Vec<String>,
    pub assessments: Vec<Assessment>,
    pub attempts: BTreeMap<String, Attempts>,
    pub recent: Recent,
}

#[derive(Debug, Serialize)]
pub struct Assessment {
    pub at: String,
    pub review_id: String,
    pub feature: String,
    pub stage: String,
    /// `first` for the first assessment of a feature and stage, otherwise `reassessment`.
    pub attempt: &'static str,
    pub effective_concern: Option<String>,
    pub elapsed_ms: Option<u64>,
    pub packet_bytes: Option<u64>,
    pub budget_bytes: Option<u64>,
    /// Recorded at review time; `None` (unknown) for reviews before accounting existed.
    pub accepted_feature_count_at_review: Option<u64>,
    pub contributions: Vec<accounting::Contribution>,
    /// The following fields are copied verbatim from the assessment frontmatter. `None` means
    /// the record does not state the field (older formats, external assessments, or an
    /// unverifiable assessment); nothing is inferred.
    pub candidate_outcome: Option<String>,
    pub source: Option<String>,
    pub codex_cli_version: Option<String>,
    pub reviewer_build_sha256: Option<String>,
    pub codeos_release_id: Option<String>,
    pub findings: Option<FindingCounts>,
}

/// Recorded findings of one assessment, counted by their recorded labels.
#[derive(Debug, Default, Serialize)]
pub struct FindingCounts {
    pub total: u64,
    pub by_severity: BTreeMap<String, u64>,
    pub by_classification: BTreeMap<String, u64>,
    /// Only findings that record a blocker kind are counted here.
    pub by_blocker_kind: BTreeMap<String, u64>,
    /// Finding blocks the reviewer emitted that could not be parsed, as recorded.
    pub unparsed: Option<u64>,
}

/// A scalar frontmatter value as text; YAML numbers (e.g. a CLI version) are kept as written.
fn scalar(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Number(number) => Some(number.to_string()),
        serde_json::Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

/// Copies the recorded outcome, provenance, and finding labels from a hash-verified assessment.
fn record_frontmatter(assessment: &mut Assessment, frontmatter: &serde_json::Value) {
    let reviewed = &frontmatter["reviewed"];
    assessment.candidate_outcome = scalar(&reviewed["candidate_outcome"]);
    assessment.source = scalar(&reviewed["source"]);
    assessment.codex_cli_version = scalar(&reviewed["codex_cli_version"]);
    assessment.reviewer_build_sha256 = scalar(&reviewed["reviewer_build_sha256"]);
    assessment.codeos_release_id = scalar(&reviewed["codeos_release_id"]);
    let Some(findings) = frontmatter["findings"].as_array() else {
        return;
    };
    let mut counts = FindingCounts {
        total: findings.len() as u64,
        unparsed: frontmatter["unparsed_findings_count"].as_u64(),
        ..Default::default()
    };
    for finding in findings {
        for (key, map) in [
            ("severity", &mut counts.by_severity),
            ("classification", &mut counts.by_classification),
            ("blocker_kind", &mut counts.by_blocker_kind),
        ] {
            if let Some(label) = scalar(&finding[key]) {
                *map.entry(label).or_default() += 1;
            }
        }
    }
    assessment.findings = Some(counts);
}

#[derive(Debug, Default, Serialize)]
pub struct Attempts {
    pub first: u64,
    pub reassessment: u64,
    pub execution_invalid: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct Recent {
    pub assessments: usize,
    pub median_elapsed_ms: Option<u64>,
    pub p95_elapsed_ms: Option<u64>,
    /// (role, source, bytes) over the recent assessments.
    pub composition: Vec<(String, Source, u64)>,
    pub hotspots: Vec<Hotspot>,
}

#[derive(Debug, Serialize)]
pub struct Hotspot {
    pub path: String,
    pub reviews: u64,
    pub max_bytes: u64,
}

fn resolve(root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

/// The file at `path` exists and has exactly `sha`, or why not.
fn verified(path: &Path, sha: &str) -> Result<(), String> {
    match sha256_file(&path.to_string_lossy()) {
        Ok(actual) if actual == sha => Ok(()),
        Ok(_) => Err(format!("hash mismatch: {}", path.display())),
        Err(_) => Err(format!("missing: {}", path.display())),
    }
}

fn header(entry: &str) -> Option<(String, String, String)> {
    let heading = entry.lines().next()?;
    let (at, rest) = heading.split_once(' ')?;
    let mut parts = rest.split(" — ");
    let _kind = parts.next()?;
    let feature = parts.next()?.to_string();
    let stage = parts.next()?.strip_prefix("Stage ")?.to_string();
    Some((at.to_string(), feature, stage))
}

fn wall_time_ms(entry: &str) -> Option<u64> {
    let value = crate::lineage_replacement::line(entry, "Effort: ").ok()?;
    value
        .split("Wall time: ")
        .nth(1)?
        .split("ms")
        .next()?
        .trim()
        .parse()
        .ok()
}

fn packet_header_bytes(packet: &str) -> Option<u64> {
    packet
        .lines()
        .find_map(|line| line.strip_prefix("  review_content_bytes: "))?
        .parse()
        .ok()
}

pub fn derive(cfg: &Config) -> Status {
    let mut incomplete = Vec::new();
    let log = match std::fs::read_to_string(&cfg.review_log) {
        Ok(log) => log,
        Err(_) if !cfg.review_log.exists() => String::new(),
        Err(error) => {
            incomplete.push(format!("{}: {error}", cfg.review_log.display()));
            String::new()
        }
    };
    let mut assessments: Vec<Assessment> = Vec::new();
    let mut attempts: BTreeMap<String, Attempts> = BTreeMap::new();
    for entry in log.split("\n## ").skip(1) {
        let Ok((packet_path, packet_sha)) =
            crate::lineage_replacement::reference(entry, "Reviewed packet: ")
        else {
            continue;
        };
        let Some((at, feature, stage)) = header(entry) else {
            incomplete.push(format!(
                "unreadable review heading: {}",
                entry.lines().next().unwrap_or("")
            ));
            continue;
        };
        let review_id = crate::lineage_replacement::line(entry, "Review ID: ")
            .unwrap_or("(none)")
            .to_string();
        let first = !assessments
            .iter()
            .any(|a| a.feature == feature && a.stage == stage);
        let counts = attempts.entry(feature.clone()).or_default();
        if first {
            counts.first += 1;
        } else {
            counts.reassessment += 1;
        }

        let mut frontmatter = None;
        match crate::lineage_replacement::reference(entry, "Full assessment: ") {
            Ok((path, sha)) => {
                let path = resolve(&cfg.repo_root, path);
                match verified(&path, &sha) {
                    Err(problem) => incomplete.push(format!("{review_id}: assessment {problem}")),
                    // Only a hash-verified assessment is read; one without frontmatter reports
                    // its fields as unknown rather than as a history defect.
                    Ok(()) => frontmatter = crate::lineage_replacement::frontmatter(&path).ok(),
                }
            }
            Err(_) => incomplete.push(format!("{review_id}: no assessment reference")),
        }
        let packet_path = resolve(&cfg.repo_root, packet_path);
        let mut assessment = Assessment {
            at,
            review_id: review_id.clone(),
            feature,
            stage,
            attempt: if first { "first" } else { "reassessment" },
            effective_concern: crate::lineage_replacement::line(entry, "Effective concern: ")
                .ok()
                .map(str::to_string),
            elapsed_ms: wall_time_ms(entry),
            packet_bytes: None,
            budget_bytes: None,
            accepted_feature_count_at_review: None,
            contributions: Vec::new(),
            candidate_outcome: None,
            source: None,
            codex_cli_version: None,
            reviewer_build_sha256: None,
            codeos_release_id: None,
            findings: None,
        };
        if let Some(frontmatter) = &frontmatter {
            record_frontmatter(&mut assessment, frontmatter);
        }
        match verified(&packet_path, &packet_sha) {
            Err(problem) => incomplete.push(format!("{review_id}: packet {problem}")),
            Ok(()) => {
                let sidecar = crate::cmd::plan::sidecar_path(&packet_path);
                match std::fs::read_to_string(&sidecar)
                    .map(|json| serde_json::from_str::<ReviewPacket>(&json))
                {
                    Ok(Ok(packet))
                        if packet.content_sha256.is_empty()
                            || packet.content_sha256 == packet_sha =>
                    {
                        assessment.packet_bytes = Some(packet.review_content_bytes);
                        assessment.budget_bytes = Some(packet.budget_bytes);
                        assessment.accepted_feature_count_at_review = packet
                            .manifest
                            .as_ref()
                            .and_then(|manifest| manifest.accepted_feature_count_at_review);
                        assessment.contributions =
                            accounting::contributions(&packet, &accounting::inclusion(&packet));
                    }
                    Ok(Ok(_)) => incomplete.push(format!(
                        "{review_id}: sidecar hash mismatch: {}",
                        sidecar.display()
                    )),
                    Ok(Err(error)) => incomplete.push(format!(
                        "{review_id}: malformed sidecar {}: {error}",
                        sidecar.display()
                    )),
                    // Packets before Reviewer Tool Contract v17 have no sidecar: their size is
                    // known from the packet header, their composition is not.
                    Err(_) => {
                        assessment.packet_bytes = std::fs::read_to_string(&packet_path)
                            .ok()
                            .as_deref()
                            .and_then(packet_header_bytes);
                    }
                }
            }
        }
        assessments.push(assessment);
    }

    let invalid_dir = cfg.codex_dir.join("execution-invalid");
    if let Ok(entries) = std::fs::read_dir(&invalid_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.contains("-attempt-") {
                continue;
            }
            if let Some(feature) = name.split("__").nth(1) {
                attempts
                    .entry(feature.to_string())
                    .or_default()
                    .execution_invalid += 1;
            }
        }
    }

    let recent = recent(&assessments);
    Status {
        health: if incomplete.is_empty() {
            "OK"
        } else {
            "INCOMPLETE"
        },
        incomplete,
        assessments,
        attempts,
        recent,
    }
}

/// Nearest-rank percentile of sorted values.
fn percentile(sorted: &[u64], percent: usize) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (percent * sorted.len()).div_ceil(100).max(1);
    sorted.get(rank - 1).copied()
}

fn recent(assessments: &[Assessment]) -> Recent {
    let window = &assessments[assessments.len().saturating_sub(RECENT)..];
    let mut elapsed: Vec<u64> = window.iter().filter_map(|a| a.elapsed_ms).collect();
    elapsed.sort_unstable();
    let mut composition: BTreeMap<(String, Source), u64> = BTreeMap::new();
    let mut hotspots: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    for c in window.iter().flat_map(|a| &a.contributions) {
        *composition
            .entry((c.role.to_string(), c.source))
            .or_default() += c.bytes;
        let spot = hotspots.entry(c.path.clone()).or_default();
        spot.0 += 1;
        spot.1 = spot.1.max(c.bytes);
    }
    let mut composition: Vec<(String, Source, u64)> = composition
        .into_iter()
        .map(|((role, source), bytes)| (role, source, bytes))
        .collect();
    composition.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    let mut hotspots: Vec<Hotspot> = hotspots
        .into_iter()
        .filter(|(path, _)| !path.starts_with('('))
        .map(|(path, (reviews, max_bytes))| Hotspot {
            path,
            reviews,
            max_bytes,
        })
        .collect();
    hotspots.sort_by(|a, b| {
        (b.max_bytes * b.reviews)
            .cmp(&(a.max_bytes * a.reviews))
            .then(a.path.cmp(&b.path))
    });
    hotspots.truncate(5);
    Recent {
        assessments: window.len(),
        median_elapsed_ms: percentile(&elapsed, 50),
        p95_elapsed_ms: (elapsed.len() >= P95_MIN_SAMPLES)
            .then(|| percentile(&elapsed, 95))
            .flatten(),
        composition,
        hotspots,
    }
}

fn opt<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "unknown".to_string(), |v| v.to_string())
}

pub fn print(status: &Status) {
    let kb = accounting::kb;
    let seconds = |ms: u64| ms.div_ceil(1000);
    println!("REVIEW STATUS  {}", status.health);
    let (first, reassessment, invalid) = status.attempts.values().fold((0, 0, 0), |t, a| {
        (
            t.0 + a.first,
            t.1 + a.reassessment,
            t.2 + a.execution_invalid,
        )
    });
    println!(
        "Assessments    {} (first {first}, reassessment {reassessment}); execution-invalid attempts {invalid}",
        status.assessments.len()
    );
    let recent = &status.recent;
    println!(
        "Elapsed        median {} s, p95 {} (last {}; observational only)",
        opt(recent.median_elapsed_ms.map(seconds)),
        recent.p95_elapsed_ms.map_or_else(
            || format!("needs n >= {P95_MIN_SAMPLES}"),
            |ms| format!("{} s", seconds(ms))
        ),
        recent.assessments
    );

    println!("\nRecent assessments");
    let start = status.assessments.len().saturating_sub(10);
    for a in &status.assessments[start..] {
        let budget = match (a.packet_bytes, a.budget_bytes) {
            (Some(bytes), Some(budget)) if budget > 0 => format!("{:>3}%", bytes * 100 / budget),
            _ => "   ?".to_string(),
        };
        println!(
            "  {} {:<7} {:<3} {:<12} {:>6} KB {budget}  {:>4} s  features {:<7}  {}",
            a.at,
            a.feature,
            a.stage,
            a.attempt,
            opt(a.packet_bytes.map(kb)),
            opt(a.elapsed_ms.map(seconds)),
            opt(a.accepted_feature_count_at_review),
            a.effective_concern.as_deref().unwrap_or("?")
        );
    }

    println!("\nComposition (last {})", recent.assessments);
    let total: u64 = recent.composition.iter().map(|c| c.2).sum();
    for (role, source, bytes) in &recent.composition {
        let marker = match source {
            Source::Recorded => "",
            Source::Inferred => "  inferred",
            Source::Unrecorded => "",
        };
        println!(
            "  {role:<22} {:>6} KB {:>3}%{marker}",
            kb(*bytes),
            if total > 0 { bytes * 100 / total } else { 0 }
        );
    }

    println!("\nFirst reviews by accepted feature count");
    let mut unknown = 0;
    for a in status.assessments.iter().filter(|a| a.attempt == "first") {
        match (a.accepted_feature_count_at_review, a.packet_bytes) {
            (Some(count), Some(bytes)) => {
                println!("  {count:>3} features  {:>6} KB  {}", kb(bytes), a.feature)
            }
            _ => unknown += 1,
        }
    }
    println!("  unknown: {unknown} first reviews before feature counts were recorded");

    println!("\nHotspots (last {})", recent.assessments);
    for spot in &recent.hotspots {
        println!(
            "  {:<60} {:>5} KB x {}",
            spot.path,
            kb(spot.max_bytes),
            spot.reviews
        );
    }

    if !status.incomplete.is_empty() {
        println!("\nINCOMPLETE");
        for problem in &status.incomplete {
            println!("  {problem}");
        }
    }
}

pub fn run(cfg: &Config, json: bool) -> i32 {
    let status = derive(cfg);
    if json {
        match serde_json::to_string_pretty(&status) {
            Ok(text) => println!("{text}"),
            Err(error) => {
                eprintln!("error: {error}");
                return crate::EXIT_WRITE;
            }
        }
    } else {
        print(&status);
    }
    crate::EXIT_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::sha256_str;

    fn config(root: &Path) -> Config {
        Config {
            reasoning_effort: "high".into(),
            reviewer_model: "m".into(),
            codex_cli_version: "v".into(),
            packet_budget_mode: crate::config::PacketBudgetMode::Fail,
            repo_root: root.to_path_buf(),
            toolkit_root: root.to_path_buf(),
            review_log: root.join("reviews/review-log.md"),
            codex_dir: root.join("reviews/codex"),
            state_dir: root.join("state"),
        }
    }

    fn write(root: &Path, rel: &str, text: &str) -> String {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        sha256_str(text)
    }

    fn entry(
        id: &str,
        feature: &str,
        assessment: &str,
        a_sha: &str,
        packet: &str,
        p_sha: &str,
    ) -> String {
        format!(
            "\n## 2026-11-16T00:00:00Z REVIEW — {feature} — Stage 8\nReview ID: {id}\nEffort: high   Wall time: 90000ms   Reconnects: 0\nEffective concern: NO OBJECTION\nFull assessment: {assessment} (sha256:{a_sha})\nReviewed packet: {packet} (sha256:{p_sha})\n"
        )
    }

    #[test]
    fn status_classifies_attempts_and_roles_and_reports_incomplete_history() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let a_sha = write(root, "reviews/a.md", "assessment");
        // A current-format assessment: its outcome, provenance, and findings are recorded.
        let b_sha = write(
            root,
            "reviews/b.md",
            "---\nreview_id: ID-1\nfindings:\n    - finding_id: F1\n      severity: High\n      classification: IN-SCOPE BLOCKER\n      summary: \"x\"\n      required_action: fix now\n      blocker_kind: SUBSTANTIVE\n    - finding_id: F2\n      severity: Low\n      classification: OUT-OF-SCOPE BACKLOG\n      summary: \"y\"\n      required_action: backlog\nunparsed_findings_count: 0\nreviewed:\n  feature: F-1\n  stage: 8\n  candidate_outcome: BLOCKED\n  source: codex\n  codex_cli_version: 0.163.0\n  reviewer_build_sha256: abc\n  codeos_release_id: DBA-36\n---\nbody\n",
        );
        // An accounting-era packet whose sidecar records roles and the feature count.
        let p1 = "packet one\n  review_content_bytes: 30\n";
        let p1_sha = write(root, "reviews/p1.packet.txt", p1);
        let mut manifest = crate::manifest::ReviewInputManifest {
            accepted_feature_count_at_review: Some(3),
            ..Default::default()
        };
        manifest.inclusion.insert(
            "src/lib.rs".into(),
            [
                "changed_implementation".to_string(),
                "test_definition".to_string(),
            ]
            .into(),
        );
        let sidecar = serde_json::json!({
            "feature": "F-1", "stage": "8", "branch": "", "review_sha": "", "base_sha": "",
            "diff_hash": "", "coverage_state": "FullCoverage", "workspace_dirty": false,
            "redaction_count": 0, "secret_flag": false, "artifacts": [], "excluded_paths": [],
            "review_content_bytes": 30, "estimated_review_tokens": 7, "budget_bytes": 60,
            "over_budget": false, "diff_bytes": 0,
            "budget_contributors": [["src/lib.rs", 20], [".codeos/00-project/charter.md", 10]],
            "content_sha256": p1_sha, "manifest": manifest,
        });
        write(
            root,
            "reviews/p1.packet.txt.meta.json",
            &sidecar.to_string(),
        );
        // An older packet with no sidecar, a missing packet, and a hash-mismatched one.
        let p2_sha = write(
            root,
            "reviews/p2.packet.txt",
            "old\n  review_content_bytes: 12\n",
        );
        let mut log = String::from("# log\n");
        log += &entry(
            "ID-1",
            "F-1",
            "reviews/b.md",
            &b_sha,
            "reviews/p1.packet.txt",
            &p1_sha,
        );
        log += &entry(
            "ID-2",
            "F-1",
            "reviews/a.md",
            &a_sha,
            "reviews/p2.packet.txt",
            &p2_sha,
        );
        log += &entry(
            "ID-3",
            "F-2",
            "reviews/a.md",
            &a_sha,
            "reviews/gone.packet.txt",
            &p2_sha,
        );
        log += &entry(
            "ID-4",
            "F-2",
            "reviews/a.md",
            &a_sha,
            "reviews/p2.packet.txt",
            &"0".repeat(64),
        );
        log += "\n## 2026-11-16T00:00:00Z HUMAN DECISION — F-1 — Stage 8\nDecision: APPROVE\n";
        write(root, "reviews/review-log.md", &log);
        write(
            root,
            "reviews/codex/execution-invalid/t-ASSESS__F-2__8__C-attempt-1.txt",
            "x",
        );

        let status = derive(&config(root));
        assert_eq!(status.health, "INCOMPLETE");
        assert_eq!(status.incomplete.len(), 2, "{:?}", status.incomplete);
        assert!(status.incomplete[0].starts_with("ID-3: packet missing"));
        assert!(status.incomplete[1].starts_with("ID-4: packet hash mismatch"));
        let kinds: Vec<_> = status.assessments.iter().map(|a| a.attempt).collect();
        assert_eq!(kinds, ["first", "reassessment", "first", "reassessment"]);
        assert_eq!(status.attempts["F-2"].execution_invalid, 1);

        let recorded = &status.assessments[0];
        assert_eq!(recorded.accepted_feature_count_at_review, Some(3));
        let roles: Vec<_> = recorded
            .contributions
            .iter()
            .map(|c| (c.role, c.source))
            .collect();
        assert_eq!(
            roles,
            [
                ("changed_implementation", Source::Recorded),
                ("governed", Source::Inferred)
            ]
        );
        let total: u64 = recorded.contributions.iter().map(|c| c.bytes).sum();
        assert_eq!(Some(total), recorded.packet_bytes);

        assert_eq!(recorded.candidate_outcome.as_deref(), Some("BLOCKED"));
        assert_eq!(recorded.source.as_deref(), Some("codex"));
        assert_eq!(recorded.codex_cli_version.as_deref(), Some("0.163.0"));
        assert_eq!(recorded.reviewer_build_sha256.as_deref(), Some("abc"));
        assert_eq!(recorded.codeos_release_id.as_deref(), Some("DBA-36"));
        let findings = recorded.findings.as_ref().expect("findings recorded");
        assert_eq!(findings.total, 2);
        assert_eq!(findings.by_severity["High"], 1);
        assert_eq!(findings.by_severity["Low"], 1);
        assert_eq!(findings.by_classification["IN-SCOPE BLOCKER"], 1);
        assert_eq!(findings.by_blocker_kind.len(), 1);
        assert_eq!(findings.by_blocker_kind["SUBSTANTIVE"], 1);
        assert_eq!(findings.unparsed, Some(0));

        let older = &status.assessments[1];
        // No frontmatter: every recorded-only field is unknown, never defaulted.
        assert_eq!(older.candidate_outcome, None);
        assert_eq!(older.source, None);
        assert!(older.findings.is_none());
        assert_eq!(older.packet_bytes, Some(12));
        assert_eq!(older.accepted_feature_count_at_review, None);
        assert!(older.contributions.is_empty());
        assert_eq!(
            status.recent.p95_elapsed_ms, None,
            "p95 needs at least 10 samples"
        );
    }
}
