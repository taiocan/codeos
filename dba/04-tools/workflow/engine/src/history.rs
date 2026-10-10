//! Process history: factual counts and elapsed durations derived read-only from existing records.
//! A metric counts or times only what a record directly establishes; a time window uses only a
//! timestamp CodeOS records as the time of the underlying event; no metric attributes cause.
//! A breakdown locates where recorded friction signals occurred, using only the subject and
//! checkpoint each counted record carries; it measures neither effort nor cause.

use crate::candidate::CandidateStore;
use crate::hashing;
use crate::observations::ObservationStore;
use crate::project::Project;
use crate::receipts::ReceiptStore;
use crate::verification::VerificationStore;
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Days covered when `--since` is omitted.
pub const DEFAULT_WINDOW_DAYS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    Observed,
    Unobserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Window {
    /// Counted from records whose event time falls in the window.
    InWindow,
    /// The records carry no event time, so the window cannot apply.
    AllTime,
}

#[derive(Debug, Serialize)]
pub struct Metric {
    pub metric: &'static str,
    pub state: State,
    pub value: Option<u64>,
    pub source: &'static str,
    pub window: Window,
    /// Where the counted events occurred, ordered by key. With `unattributed` it sums to `value`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<Part>>,
    /// Counted events whose record carries no subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unattributed: Option<u64>,
}

/// The counted events of one metric that share a subject and checkpoint.
#[derive(Debug, Serialize)]
pub struct Part {
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<String>,
    pub value: u64,
}

/// Admitted events keyed by the subject and checkpoint their record carries. A metric's value,
/// breakdown, and unattributed count all derive from this one tally.
#[derive(Default)]
struct Tally(BTreeMap<(Option<String>, Option<String>), u64>);

impl Tally {
    fn add(&mut self, subject: Option<&str>, checkpoint: Option<&str>) {
        let key = (subject.map(str::to_owned), checkpoint.map(str::to_owned));
        *self.0.entry(key).or_default() += 1;
    }
}

/// One endpoint of an elapsed duration: the record and its event time.
#[derive(Debug, Serialize)]
pub struct Point {
    pub record: String,
    pub at: String,
}

/// Elapsed time between two records. It is never attributed to a cause.
#[derive(Debug, Serialize)]
pub struct Duration {
    pub metric: &'static str,
    pub subject: String,
    pub state: State,
    pub from: Option<Point>,
    pub to: Option<Point>,
    pub elapsed_seconds: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct History {
    /// Inclusive start date (UTC, 00:00:00), or `all`.
    pub since: String,
    pub metrics: Vec<Metric>,
    pub durations: Vec<Duration>,
}

/// Parse `--since`: a `YYYY-MM-DD` date, `all`, or `None` for the default window.
pub fn window_start(since: Option<&str>, today: NaiveDate) -> Result<Option<NaiveDate>> {
    match since {
        Some("all") => Ok(None),
        Some(date) => NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map(Some)
            .with_context(|| format!("--since must be YYYY-MM-DD or all, got '{date}'")),
        None => Ok(Some(today - chrono::Duration::days(DEFAULT_WINDOW_DAYS))),
    }
}

fn event_time(timestamp: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(timestamp)
        .with_context(|| format!("record timestamp '{timestamp}' is not RFC 3339"))?
        .with_timezone(&Utc))
}

fn observed(metric: &'static str, tally: Tally, source: &'static str, window: Window) -> Metric {
    let (mut breakdown, mut unattributed) = (Vec::new(), 0);
    for ((subject, checkpoint), value) in tally.0 {
        match subject {
            Some(subject) => breakdown.push(Part {
                subject,
                checkpoint,
                value,
            }),
            None => unattributed += value,
        }
    }
    Metric {
        metric,
        state: State::Observed,
        value: Some(breakdown.iter().map(|part| part.value).sum::<u64>() + unattributed),
        source,
        window,
        breakdown: Some(breakdown),
        unattributed: Some(unattributed),
    }
}

fn unobserved(metric: &'static str, source: &'static str) -> Metric {
    Metric {
        metric,
        state: State::Unobserved,
        value: None,
        source,
        window: Window::InWindow,
        breakdown: None,
        unattributed: None,
    }
}

pub fn derive(project: &Project, start: Option<NaiveDate>) -> Result<History> {
    let start = start.map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    let in_window = |timestamp: &str| -> Result<bool> {
        match start {
            None => Ok(true),
            Some(start) => Ok(event_time(timestamp)? >= start),
        }
    };
    let codeos = project.codeos_dir();
    let receipts = ReceiptStore::at(&codeos).all()?;
    let observations = ObservationStore::at(&codeos);

    let mut accepted = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut additional = Tally::default();
    for receipt in &receipts {
        // Repetition is judged against all earlier receipts, before the window applies.
        let key = (&receipt.workflow, &receipt.subject, &receipt.checkpoint);
        let repeated = !seen.insert(key);
        if !in_window(&receipt.timestamp)? {
            continue;
        }
        if repeated {
            additional.add(Some(&receipt.subject), Some(&receipt.checkpoint));
        }
        if receipt.checkpoint == "acceptance" && receipt.result == "accepted" {
            accepted.insert(receipt.subject.clone());
        }
    }
    let mut accepted_features = Tally::default();
    for subject in &accepted {
        accepted_features.add(Some(subject), None);
    }
    let mut failed = Tally::default();
    for record in VerificationStore::at(&codeos).all()? {
        if record.result != "passed" && in_window(&record.timestamp)? {
            failed.add(Some(&record.subject), Some(&record.checkpoint));
        }
    }
    let (non_progressing, execution_invalid) = review_log_counts(project, &in_window)?;
    let mut opened = Tally::default();
    for observation in observations.observations()? {
        if in_window(&observation.timestamp)? {
            opened.add(None, None);
        }
    }
    let mut closed = Tally::default();
    for closure in observations.closures()? {
        if in_window(&closure.timestamp)? {
            closed.add(None, None);
        }
    }
    let mut defect_routes = Tally::default();
    for route in observations.routes()? {
        if route.route == "implementation_defect" && in_window(&route.timestamp)? {
            defect_routes.add(route.target_id.as_deref(), None);
        }
    }
    let mut bundles = Tally::default();
    for (_, bundle) in CandidateStore::at(&codeos).all()? {
        bundles.add(Some(&bundle.feature), None);
    }

    use Window::*;
    let metrics = vec![
        observed(
            "accepted_features",
            accepted_features,
            "distinct subjects with an accepted F9 receipt in the window (06-workflow/decisions.jsonl)",
            InWindow,
        ),
        observed(
            "additional_decision_receipts",
            additional,
            "in-window decision receipts with an earlier receipt, at any time, for the same workflow, subject, and checkpoint",
            InWindow,
        ),
        observed(
            "failed_verifications",
            failed,
            "verification records whose result is not passed (06-workflow/verifications.jsonl)",
            InWindow,
        ),
        observed(
            "non_progressing_assessments",
            non_progressing,
            "review-log entries whose effective concern does not permit progression",
            InWindow,
        ),
        observed(
            "execution_invalid",
            execution_invalid,
            "review-log REVIEW EXECUTION INVALID entries",
            InWindow,
        ),
        observed(
            "candidate_bundles",
            bundles,
            "Candidate Bundles (06-workflow/candidates); they carry no event time",
            AllTime,
        ),
        observed(
            "observations_opened",
            opened,
            "Observation v2 records; they name no subject",
            InWindow,
        ),
        observed(
            "observations_closed",
            closed,
            "Observation v2 closure records; they name no subject",
            InWindow,
        ),
        observed(
            "implementation_defect_routes",
            defect_routes,
            "Observation v2 routes to implementation_defect",
            InWindow,
        ),
        unobserved(
            "implementation_escapes",
            "no record establishes that a defect applied to the accepted state",
        ),
        unobserved("not_ready", "review readiness failures are not persisted"),
        unobserved(
            "evidence_invalidations",
            "evidence invalidation events are not persisted",
        ),
        unobserved(
            "attributed_waiting",
            "no record establishes why work waited",
        ),
    ];

    let mut durations = Vec::new();
    let closures: BTreeMap<_, _> = observations
        .closures()?
        .into_iter()
        .map(|closure| (closure.observation.clone(), closure))
        .collect();
    for observation in observations.observations()? {
        let Some(closure) = closures.get(&observation.id) else {
            continue;
        };
        if !in_window(&closure.timestamp)? {
            continue;
        }
        durations.push(elapsed(
            "observation_closure",
            &observation.id,
            Point {
                record: "Observation v2 record".into(),
                at: observation.timestamp.clone(),
            },
            Point {
                record: "Observation v2 closure".into(),
                at: closure.timestamp.clone(),
            },
        )?);
    }
    for receipt in &receipts {
        if receipt.checkpoint != "acceptance"
            || receipt.result != "accepted"
            || !in_window(&receipt.timestamp)?
        {
            continue;
        }
        let to = Point {
            record: "F9 acceptance receipt".into(),
            at: receipt.timestamp.clone(),
        };
        durations.push(match accepted_package_approval(project, receipt)? {
            Some(approved_at) => elapsed(
                "specification_to_acceptance",
                &receipt.subject,
                Point {
                    record: "accepted Specification Package approval (approved_at)".into(),
                    at: approved_at,
                },
                to,
            )?,
            None => Duration {
                metric: "specification_to_acceptance",
                subject: receipt.subject.clone(),
                state: State::Unobserved,
                from: None,
                to: Some(to),
                elapsed_seconds: None,
            },
        });
    }

    Ok(History {
        since: start.map_or_else(|| "all".into(), |s| s.format("%Y-%m-%d").to_string()),
        metrics,
        durations,
    })
}

fn elapsed(metric: &'static str, subject: &str, from: Point, to: Point) -> Result<Duration> {
    let seconds = (event_time(&to.at)? - event_time(&from.at)?).num_seconds();
    Ok(Duration {
        metric,
        subject: subject.into(),
        state: State::Observed,
        from: Some(from),
        to: Some(to),
        elapsed_seconds: Some(seconds),
    })
}

/// The approval time of the exact package the acceptance bound: only when every current artifact
/// still hashes to the accepted binding, so its `approved_at` belongs to that package. A date
/// is taken as 00:00:00 UTC.
fn accepted_package_approval(
    project: &Project,
    receipt: &crate::receipts::Receipt,
) -> Result<Option<String>> {
    let mut latest: Option<DateTime<Utc>> = None;
    for (key, path) in [
        ("intent", project.intent(&receipt.subject)),
        ("contract", project.contract(&receipt.subject)),
        ("event_schema", project.event_schema(&receipt.subject)),
    ] {
        let Some(bound) = receipt.bindings.get(key) else {
            return Ok(None);
        };
        if hashing::file_sha256(&path)?.as_ref() != Some(bound) {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path)?;
        let approved_at = crate::evidence::split_frontmatter(&text)
            .0
            .and_then(|fm| serde_yaml::from_str::<crate::evidence::SpecFrontmatter>(fm).ok())
            .and_then(|fm| fm.approved_at)
            .and_then(|value| match value {
                serde_yaml::Value::String(s) => Some(s),
                _ => None,
            });
        let Some(at) = approved_at.and_then(|v| approval_time(&v)) else {
            return Ok(None);
        };
        latest = Some(latest.map_or(at, |l| l.max(at)));
    }
    Ok(latest
        .filter(|at| event_time(&receipt.timestamp).is_ok_and(|accepted| *at <= accepted))
        .map(|at| at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)))
}

fn approval_time(value: &str) -> Option<DateTime<Utc>> {
    event_time(value).ok().or_else(|| {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .ok()
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
    })
}

/// Count review-log entries, using each entry's heading timestamp as its event time:
/// entries whose effective concern does not permit progression, and execution-invalid entries.
/// Each is located by its heading's subject and stage; an entry whose heading does not parse
/// still counts, unattributed.
fn review_log_counts(
    project: &Project,
    in_window: &dyn Fn(&str) -> Result<bool>,
) -> Result<(Tally, Tally)> {
    let (mut non_progressing, mut invalid) = (Tally::default(), Tally::default());
    let path = project.review_records_dir().join("review-log.md");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok((non_progressing, invalid));
    };
    for entry in text.split("\n## ").skip(1) {
        let line = entry.lines().next().unwrap_or_default();
        let heading = crate::evidence::review_heading(line);
        let (timestamp, kind) = match &heading {
            Some(heading) => (heading.timestamp, heading.kind),
            None => match line.split_once(' ') {
                Some(parts) => parts,
                None => continue,
            },
        };
        if !in_window(timestamp)? {
            continue;
        }
        let stage = heading.as_ref().map(|h| format!("Stage {}", h.stage));
        let (subject, stage) = (heading.as_ref().map(|h| h.subject), stage.as_deref());
        // The heading the reviewer writes for REVIEW_EXECUTION_INVALID (Reviewer Tool Contract).
        if kind.starts_with("REVIEW EXECUTION INVALID") {
            invalid.add(subject, stage);
        }
        let concern = entry
            .lines()
            .find_map(|line| line.strip_prefix("Effective concern: "));
        if concern.is_some_and(|c| !crate::evidence::review_permits_progression(c)) {
            non_progressing.add(subject, stage);
        }
    }
    Ok((non_progressing, invalid))
}

pub fn render(history: &History) -> String {
    let mut out = match history.since.as_str() {
        "all" => "HISTORY — all records\n".to_string(),
        since => format!("HISTORY — since {since} 00:00 UTC (inclusive)\n"),
    };
    for metric in &history.metrics {
        let value = match metric.value {
            Some(value) => value.to_string(),
            None => "UNOBSERVED".into(),
        };
        let scope = if metric.window == Window::AllTime {
            "  (all time)"
        } else {
            ""
        };
        let parts = metric.breakdown.as_deref().unwrap_or_default();
        let unattributed = metric.unattributed.unwrap_or_default();
        let place = |subject: &str, checkpoint: &Option<String>| match checkpoint {
            Some(checkpoint) => format!("{subject} {checkpoint}"),
            None => subject.to_string(),
        };
        let inline = match parts {
            [part] if unattributed == 0 => {
                format!("  ({})", place(&part.subject, &part.checkpoint))
            }
            _ => String::new(),
        };
        out.push_str(&format!("  {:<30}{value}{scope}{inline}\n", metric.metric));
        if inline.is_empty() && !parts.is_empty() {
            for part in parts {
                let place = place(&part.subject, &part.checkpoint);
                out.push_str(&format!("    {place:<27} {}\n", part.value));
            }
            if unattributed > 0 {
                out.push_str(&format!("    {:<27} {unattributed}\n", "unattributed"));
            }
        }
    }
    out.push_str("\nDURATIONS (elapsed time, not attributed waiting)\n");
    if history.durations.is_empty() {
        out.push_str("  none in window\n");
    }
    for duration in &history.durations {
        let value = duration
            .elapsed_seconds
            .map_or_else(|| "UNOBSERVED".into(), human);
        out.push_str(&format!(
            "  {:<30}{:<10}{value}\n",
            duration.metric, duration.subject
        ));
    }
    out.trim_end().to_string()
}

fn human(seconds: i64) -> String {
    let (days, hours) = (seconds / 86_400, (seconds % 86_400) / 3_600);
    if days > 0 {
        format!("{days}d {hours}h")
    } else {
        format!("{hours}h {}m", (seconds % 3_600) / 60)
    }
}
