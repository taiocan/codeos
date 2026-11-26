//! Charter obligations (Doctrine v11, Feature Workflow v16): derive each obligation's accountable
//! owner, mandatory checks and state from the Charter, Contracts and existing verification records.
//! Nothing here is stored; the report is a derived view.

use crate::checker::Subject;
use crate::contract::Workflow;
use crate::project::Project;
use crate::test_evidence::{TestCase, TestEvidence};
use crate::verification::{RecordState, VerificationStore};
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The `details` key under which a v16 record keeps the obligation meanings it was produced for.
pub const DETAILS_KEY: &str = "obligation_meaning";

/// Feature verifications whose test evidence can pass a scenario check.
const TEST_VERIFICATIONS: [&str; 3] = ["behavior", "repeatability", "playwright"];

/// Records that keep obligation meanings under v16.
pub const STAMPED: [&str; 5] = [
    "behavior",
    "repeatability",
    "playwright",
    "product_journey",
    "project_qualification",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    Unassessed,
    Allocated,
    Specified,
    Verified,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Obligation {
    pub id: String,
    /// The row's cells after the ID, normalized, as the reviewer is shown them.
    pub text: String,
    pub meaning: String,
    pub aggregates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scenario {
    pub id: String,
    pub covers: Vec<String>,
    pub contributes: Vec<String>,
}

/// Evidence for one feature's scenario checks: the union of the test cases bound by its current
/// required test verifications, and the obligation meanings those records were produced for.
/// `None` meanings mark legacy (pre-v16) evidence.
#[derive(Debug, Clone, Default)]
pub struct FeatureEvidence {
    pub cases: Vec<TestCase>,
    pub meanings: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct FeatureInput {
    pub contract_approved: bool,
    pub scenarios: Vec<Scenario>,
    /// `None` when any required test verification lacks a current record.
    pub evidence: Option<FeatureEvidence>,
}

#[derive(Debug, Clone, Default)]
pub struct QualificationEvidence {
    pub statuses: BTreeMap<String, String>,
    pub meanings: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct Inputs {
    pub obligations: BTreeMap<String, Obligation>,
    pub charter_approved: bool,
    pub features: BTreeMap<String, FeatureInput>,
    pub qualification: Option<QualificationEvidence>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Row {
    pub id: String,
    pub state: State,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<String>,
    /// Why the obligation is not in a higher state.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Report {
    pub obligations: Vec<Row>,
    pub findings: Vec<String>,
}

fn valid_id(id: &str) -> bool {
    let Some((kind, rest)) = id.split_once('-') else {
        return false;
    };
    let number =
        |s: &str| !s.is_empty() && !s.starts_with('0') && s.chars().all(|c| c.is_ascii_digit());
    matches!(kind, "O" | "R" | "C" | "M")
        && match rest.split_once('.') {
            Some((n, k)) => number(n) && number(k),
            None => number(rest),
        }
}

fn parent_of(id: &str) -> Option<&str> {
    id.split_once('.').map(|(parent, _)| parent)
}

fn cells(line: &str) -> Option<Vec<String>> {
    let line = line.trim();
    (line.starts_with('|') && line.len() > 1).then(|| {
        line.trim_matches('|')
            .split('|')
            .map(|cell| cell.split_whitespace().collect::<Vec<_>>().join(" "))
            .collect()
    })
}

fn is_separator(cells: &[String]) -> bool {
    cells
        .iter()
        .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

/// Split a list cell or line of IDs on commas and whitespace, dropping brackets.
fn id_list(text: &str) -> Vec<String> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .map(|v| v.trim_matches(|c| c == '[' || c == ']'))
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect()
}

/// Every obligation row of a Charter, with its meaning. Duplicate IDs fail closed.
pub fn parse_charter(text: &str) -> Result<BTreeMap<String, Obligation>> {
    let mut out = BTreeMap::new();
    let mut header: Option<Vec<String>> = None;
    let mut previous_was_row = false;
    for line in text.lines() {
        let Some(row) = cells(line) else {
            previous_was_row = false;
            header = None;
            continue;
        };
        if !previous_was_row {
            header = Some(row);
            previous_was_row = true;
            continue;
        }
        previous_was_row = true;
        if is_separator(&row) {
            continue;
        }
        let Some(id) = row.first().filter(|id| valid_id(id)).cloned() else {
            continue;
        };
        let rest = &row[1..];
        let aggregates = if id.starts_with("M-") {
            header
                .as_ref()
                .and_then(|h| h.iter().position(|c| c.eq_ignore_ascii_case("aggregates")))
                .and_then(|index| row.get(index))
                .map(|cell| id_list(cell))
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let obligation = Obligation {
            id: id.clone(),
            text: rest.join(" | "),
            meaning: crate::hashing::text_sha256(&rest.join("\n")),
            aggregates,
        };
        anyhow::ensure!(
            out.insert(id.clone(), obligation).is_none(),
            "Charter declares obligation {id} more than once"
        );
    }
    Ok(out)
}

fn scenario_id(heading: &str) -> Option<String> {
    let inner = heading.trim_end().strip_suffix(']')?;
    let (_, id) = inner.rsplit_once('[')?;
    let k = id.strip_prefix("S-")?;
    (!k.is_empty() && !k.starts_with('0') && k.chars().all(|c| c.is_ascii_digit()))
        .then(|| id.to_string())
}

/// The identified scenarios of a Contract with their `covers:` and `contributes:` lines.
pub fn parse_contract(text: &str) -> Result<Vec<Scenario>> {
    let mut out: Vec<Scenario> = Vec::new();
    let mut current: Option<usize> = None;
    let mut fenced = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if trimmed.starts_with('#') {
            current = match scenario_id(trimmed) {
                Some(id) => {
                    anyhow::ensure!(
                        out.iter().all(|s| s.id != id),
                        "Contract declares scenario {id} more than once"
                    );
                    out.push(Scenario {
                        id,
                        ..Scenario::default()
                    });
                    Some(out.len() - 1)
                }
                None => None,
            };
            continue;
        }
        for (key, is_covers) in [("covers:", true), ("contributes:", false)] {
            if let Some(list) = trimmed.strip_prefix(key) {
                let index =
                    current.with_context(|| format!("`{key}` outside an identified scenario"))?;
                let ids = id_list(list);
                if is_covers {
                    out[index].covers.extend(ids);
                } else {
                    out[index].contributes.extend(ids);
                }
            }
        }
    }
    Ok(out)
}

/// The scenario tag token for `S-k` of `F-nnnn`, e.g. `f0004s3`.
pub fn tag(feature: &str, scenario: &str) -> String {
    format!(
        "f{}s{}",
        feature.trim_start_matches("F-"),
        scenario.trim_start_matches("S-")
    )
    .to_lowercase()
}

fn carries_tag(case_id: &str, tag: &str) -> bool {
    case_id
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|token| token == tag)
}

/// At least one current passed tagged case and no failed or flaky tagged case. Skipped cases
/// establish nothing.
pub fn scenario_passes(cases: &[TestCase], tag: &str) -> bool {
    let tagged: Vec<_> = cases.iter().filter(|c| carries_tag(&c.id, tag)).collect();
    tagged.iter().any(|c| c.status == "passed")
        && !tagged
            .iter()
            .any(|c| matches!(c.status.as_str(), "failed" | "flaky"))
}

fn meaning_current(
    meanings: &Option<BTreeMap<String, String>>,
    obligation: &Obligation,
) -> std::result::Result<(), String> {
    match meanings {
        None => Err("legacy evidence (recorded before Feature Workflow v16) does not count".into()),
        Some(map) => match map.get(&obligation.id) {
            Some(meaning) if *meaning == obligation.meaning => Ok(()),
            Some(_) => {
                Err("evidence was produced for an earlier meaning of this obligation".into())
            }
            None => Err("evidence does not record this obligation's meaning".into()),
        },
    }
}

/// Derive every obligation's owner, checks and state.
pub fn evaluate(inputs: &Inputs) -> Report {
    let obligations = &inputs.obligations;
    let parents: BTreeSet<&str> = obligations.keys().filter_map(|id| parent_of(id)).collect();
    let mut findings = Vec::new();
    let mut owners: BTreeMap<&str, BTreeMap<&str, Vec<&str>>> = BTreeMap::new();
    let mut contributors: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (feature, input) in &inputs.features {
        for scenario in &input.scenarios {
            for (ids, covering) in [(&scenario.covers, true), (&scenario.contributes, false)] {
                for id in ids {
                    let verb = if covering { "covers" } else { "contributes to" };
                    if !obligations.contains_key(id) {
                        findings.push(format!(
                            "{feature} {} {verb} unknown or retired obligation {id}",
                            scenario.id
                        ));
                    } else if parents.contains(id.as_str()) {
                        findings.push(format!(
                            "{feature} {} {verb} {id}, which has clauses; name the clauses",
                            scenario.id
                        ));
                    } else if covering {
                        owners
                            .entry(id)
                            .or_default()
                            .entry(feature)
                            .or_default()
                            .push(&scenario.id);
                    } else {
                        contributors.entry(id).or_default().insert(feature);
                    }
                }
            }
        }
    }

    let mut rows: BTreeMap<String, Row> = BTreeMap::new();
    // Leaves first: everything that is neither a parent nor a Milestone.
    for obligation in obligations.values() {
        if parents.contains(obligation.id.as_str()) || obligation.id.starts_with("M-") {
            continue;
        }
        let row = leaf_row(inputs, obligation, &owners, &contributors, &mut findings);
        rows.insert(obligation.id.clone(), row);
    }
    // Parents of leaf clauses: the weakest clause state.
    for parent in parents.iter().filter(|p| !p.starts_with("M-")) {
        if !obligations.contains_key(*parent) {
            continue;
        }
        let clauses: Vec<&Row> = rows
            .values()
            .filter(|r| parent_of(&r.id) == Some(*parent))
            .collect();
        let state = clauses
            .iter()
            .map(|r| r.state)
            .min()
            .unwrap_or(State::Unassessed);
        let notes = clauses
            .iter()
            .filter(|r| r.state == state && state != State::Verified)
            .map(|r| format!("weakest clause {}", r.id))
            .collect();
        rows.insert(
            parent.to_string(),
            Row {
                id: parent.to_string(),
                state,
                owner: None,
                checks: Vec::new(),
                contributors: Vec::new(),
                notes,
            },
        );
    }
    // Milestones aggregate other obligations; resolved in dependency order, cycles reported.
    let milestones: Vec<&Obligation> = obligations
        .values()
        .filter(|o| o.id.starts_with("M-") && !parents.contains(o.id.as_str()))
        .collect();
    let mut pending: Vec<&Obligation> = milestones;
    loop {
        let before = pending.len();
        pending.retain(|milestone| {
            let ready = milestone.aggregates.iter().all(|id| {
                rows.contains_key(id) || !obligations.contains_key(id) || *id == milestone.id
            });
            if ready {
                let row = milestone_row(milestone, obligations, &rows, &mut findings);
                rows.insert(milestone.id.clone(), row);
            }
            !ready
        });
        if pending.is_empty() || pending.len() == before {
            break;
        }
    }
    for milestone in pending {
        findings.push(format!(
            "{} aggregates obligations in a cycle",
            milestone.id
        ));
        rows.insert(
            milestone.id.clone(),
            Row {
                id: milestone.id.clone(),
                state: State::Unassessed,
                owner: Some("project".into()),
                checks: milestone.aggregates.clone(),
                contributors: Vec::new(),
                notes: vec!["aggregation cycle".into()],
            },
        );
    }
    // Milestone parents, if any.
    for parent in parents.iter().filter(|p| p.starts_with("M-")) {
        if !obligations.contains_key(*parent) {
            continue;
        }
        let state = rows
            .values()
            .filter(|r| parent_of(&r.id) == Some(*parent))
            .map(|r| r.state)
            .min()
            .unwrap_or(State::Unassessed);
        rows.insert(
            parent.to_string(),
            Row {
                id: parent.to_string(),
                state,
                owner: None,
                checks: Vec::new(),
                contributors: Vec::new(),
                notes: Vec::new(),
            },
        );
    }
    findings.sort();
    findings.dedup();
    let mut obligations: Vec<Row> = rows.into_values().collect();
    obligations.sort_by_key(|row| natural_key(&row.id));
    Report {
        obligations,
        findings,
    }
}

/// Order `O`, `R`, `C`, `M`, then by number and clause: `C-2` before `C-10`, `R-8` before `R-8.1`.
fn natural_key(id: &str) -> (usize, u64, u64) {
    let (kind, rest) = id.split_once('-').unwrap_or((id, ""));
    let (n, k) = rest.split_once('.').unwrap_or((rest, "0"));
    (
        ["O", "R", "C", "M"]
            .iter()
            .position(|v| *v == kind)
            .unwrap_or(4),
        n.parse().unwrap_or(0),
        k.parse().unwrap_or(0),
    )
}

fn leaf_row(
    inputs: &Inputs,
    obligation: &Obligation,
    owners: &BTreeMap<&str, BTreeMap<&str, Vec<&str>>>,
    contributors: &BTreeMap<&str, BTreeSet<&str>>,
    findings: &mut Vec<String>,
) -> Row {
    let id = obligation.id.as_str();
    let mut row = Row {
        id: id.to_string(),
        state: State::Unassessed,
        owner: None,
        checks: Vec::new(),
        contributors: contributors
            .get(id)
            .map(|c| c.iter().map(|v| v.to_string()).collect())
            .unwrap_or_default(),
        notes: Vec::new(),
    };
    let covering = owners.get(id);
    match covering.map(|m| m.len()).unwrap_or(0) {
        0 if id.starts_with("C-") && parent_of(id).is_none() => {
            row.owner = Some("project qualification".into());
            row.checks = vec![format!("qualification {id}")];
            row.state = State::Allocated;
            if !inputs.charter_approved {
                row.notes.push("Charter not approved".into());
                return row;
            }
            row.state = State::Specified;
            match &inputs.qualification {
                None => row
                    .notes
                    .push("no current project qualification record".into()),
                Some(q) => match meaning_current(&q.meanings, obligation) {
                    Err(note) => row.notes.push(note),
                    Ok(()) => match q.statuses.get(id).map(String::as_str) {
                        Some("pass") => row.state = State::Verified,
                        Some(status) => row.notes.push(format!("qualification reports {status}")),
                        None => row.notes.push("qualification does not report it".into()),
                    },
                },
            }
        }
        0 => {
            if row.contributors.is_empty() {
                row.notes.push("no accountable owner".into());
            } else {
                row.notes
                    .push("contributions only; no accountable owner covers it".into());
            }
            findings.push(format!("{id} has no accountable verification owner"));
        }
        1 => {
            let (feature, scenarios) = covering.unwrap().iter().next().unwrap();
            let input = &inputs.features[*feature];
            row.owner = Some(feature.to_string());
            row.checks = scenarios.iter().map(|s| format!("{feature} {s}")).collect();
            row.state = State::Allocated;
            if !input.contract_approved {
                row.notes.push(format!("{feature} Contract not approved"));
                return row;
            }
            row.state = State::Specified;
            let Some(evidence) = &input.evidence else {
                row.notes.push(format!(
                    "{feature} has no current test verification records"
                ));
                return row;
            };
            if let Err(note) = meaning_current(&evidence.meanings, obligation) {
                row.notes.push(note);
                return row;
            }
            let failing: Vec<String> = scenarios
                .iter()
                .filter(|s| !scenario_passes(&evidence.cases, &tag(feature, s)))
                .map(|s| {
                    format!("{feature} {s} has no passing tagged test, or a tagged test failed")
                })
                .collect();
            if failing.is_empty() {
                row.state = State::Verified;
            } else {
                row.notes.extend(failing);
            }
        }
        _ => {
            let names: Vec<&str> = covering.unwrap().keys().copied().collect();
            row.notes.push(format!("covered by {}", names.join(", ")));
            findings.push(format!(
                "{id} is covered by more than one feature ({}); exactly one owner is allowed",
                names.join(", ")
            ));
        }
    }
    row
}

fn state_name(state: State) -> &'static str {
    match state {
        State::Unassessed => "UNASSESSED",
        State::Allocated => "ALLOCATED",
        State::Specified => "SPECIFIED",
        State::Verified => "VERIFIED",
    }
}

fn milestone_row(
    milestone: &Obligation,
    obligations: &BTreeMap<String, Obligation>,
    rows: &BTreeMap<String, Row>,
    findings: &mut Vec<String>,
) -> Row {
    let mut row = Row {
        id: milestone.id.clone(),
        state: State::Unassessed,
        owner: Some("project".into()),
        checks: milestone.aggregates.clone(),
        contributors: Vec::new(),
        notes: Vec::new(),
    };
    if milestone.aggregates.is_empty() {
        findings.push(format!("{} aggregates no obligation", milestone.id));
        row.notes.push("aggregates nothing".into());
        return row;
    }
    let mut invalid = false;
    for id in &milestone.aggregates {
        if *id == milestone.id {
            findings.push(format!("{} aggregates itself", milestone.id));
            invalid = true;
        } else if !obligations.contains_key(id) {
            findings.push(format!(
                "{} aggregates unknown obligation {id}",
                milestone.id
            ));
            invalid = true;
        }
    }
    if invalid {
        row.notes.push("invalid aggregation".into());
        return row;
    }
    let weakest = milestone
        .aggregates
        .iter()
        .map(|id| rows[id].state)
        .min()
        .unwrap_or(State::Unassessed);
    row.state = weakest.max(State::Allocated);
    for id in &milestone.aggregates {
        if rows[id].state < State::Verified {
            row.notes
                .push(format!("{id} is {}", state_name(rows[id].state)));
        }
    }
    row
}

// ---- Project glue -------------------------------------------------------------------------

/// The obligations a feature's Contract covers, with their current meanings.
fn covered_meanings(project: &Project, feature: &str) -> Result<BTreeMap<String, String>> {
    let charter = parse_charter(&std::fs::read_to_string(project.charter())?)?;
    let contract = std::fs::read_to_string(project.contract(feature)).unwrap_or_default();
    let mut out = BTreeMap::new();
    for scenario in parse_contract(&contract)? {
        for id in scenario.covers {
            if let Some(obligation) = charter.get(&id) {
                out.insert(id, obligation.meaning.clone());
            }
        }
    }
    Ok(out)
}

/// Under Feature Workflow v16, add the obligation meanings a record was produced for to its
/// details: the subject's covered obligations, or every unclaused `C-n` for qualification. A
/// recorded value, never a dependency.
pub fn stamp(
    project: &Project,
    subject: &str,
    verification: &str,
    details: Option<Value>,
) -> Result<Option<Value>> {
    if project.feature_workflow_version()? < 16 || !STAMPED.contains(&verification) {
        return Ok(details);
    }
    let meanings = if verification == "project_qualification" {
        let charter = parse_charter(&std::fs::read_to_string(project.charter())?)?;
        charter
            .values()
            .filter(|o| o.id.starts_with("C-") && parent_of(&o.id).is_none())
            .map(|o| (o.id.clone(), o.meaning.clone()))
            .collect()
    } else {
        covered_meanings(project, subject)?
    };
    let mut details = details.unwrap_or_else(|| Value::Object(Default::default()));
    if let Some(object) = details.as_object_mut() {
        object.insert(DETAILS_KEY.into(), serde_json::to_value(meanings)?);
    }
    Ok(Some(details))
}

fn record_meanings(details: Option<&Value>) -> Option<BTreeMap<String, String>> {
    details
        .and_then(|d| d.get(DETAILS_KEY))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

/// Current evidence for a feature's scenario checks, under the unchanged record validity rules.
fn feature_evidence(project: &Project, feature: &str) -> Result<Option<FeatureEvidence>> {
    let Ok(subject) = Subject::resolve(project, Workflow::Feature, feature) else {
        return Ok(None);
    };
    let store = VerificationStore::at(&project.codeos_dir());
    let mut records = Vec::new();
    for (checkpoint, verification) in
        crate::revalidation::required_verifications(project, &subject)?
    {
        if !(TEST_VERIFICATIONS.contains(&verification) || verification == "product_journey") {
            continue;
        }
        let want = crate::checker::record_bindings(project, &subject, checkpoint, verification)?;
        let have = store.latest("feature", feature, checkpoint, verification)?;
        let state = crate::verification::evaluate(have, &want);
        let evidence = match &state {
            RecordState::Current(record) if TEST_VERIFICATIONS.contains(&verification) => record
                .details
                .as_ref()
                .and_then(|d| d.get(crate::test_evidence::DETAILS_KEY))
                .and_then(Value::as_str)
                .map(|identity| -> Result<TestEvidence> {
                    let path = project
                        .root
                        .join(crate::test_evidence::DIR)
                        .join(format!("{identity}.json"));
                    Ok(serde_json::from_slice(
                        &std::fs::read(&path)
                            .with_context(|| format!("read {}", path.display()))?,
                    )?)
                })
                .transpose()?,
            _ => None,
        };
        records.push((verification, state, evidence));
    }
    Ok(combine(records))
}

/// Combine a feature's required records into scenario-check evidence. Any required test or
/// product-journey record that is absent, failed or stale, or a test record without test evidence,
/// yields `None`: no scenario check can pass. Any legacy record makes the meanings legacy.
pub fn combine(records: Vec<(&str, RecordState, Option<TestEvidence>)>) -> Option<FeatureEvidence> {
    let mut cases = Vec::new();
    let mut meanings: Option<BTreeMap<String, String>> = None;
    let mut legacy = false;
    let mut any_tests = false;
    for (verification, state, evidence) in records {
        let RecordState::Current(record) = state else {
            return None;
        };
        if TEST_VERIFICATIONS.contains(&verification) {
            let evidence = evidence?;
            any_tests = true;
            cases.extend(evidence.runs.into_iter().flat_map(|run| run.cases));
        }
        match record_meanings(record.details.as_ref()) {
            None => legacy = true,
            Some(this) => {
                meanings = Some(match meanings {
                    None => this,
                    Some(seen) => seen
                        .into_iter()
                        .filter(|(id, meaning)| this.get(id) == Some(meaning))
                        .collect(),
                })
            }
        }
    }
    any_tests.then_some(FeatureEvidence {
        cases,
        meanings: if legacy { None } else { meanings },
    })
}

/// The latest current candidate project qualification record, compared without the whole-Charter
/// binding: clause-level currency uses each obligation's own meaning instead.
fn qualification_evidence(project: &Project) -> Result<Option<QualificationEvidence>> {
    let store = VerificationStore::at(&project.codeos_dir());
    let Some(record) = store
        .all()?
        .into_iter()
        .rfind(|r| r.checkpoint == "candidate" && r.verification == "project_qualification")
    else {
        return Ok(None);
    };
    let Ok(subject) = Subject::resolve(project, Workflow::Feature, &record.subject) else {
        return Ok(None);
    };
    let want =
        crate::checker::record_bindings(project, &subject, "candidate", "project_qualification")?;
    Ok(qualification_from_record(&record, &want))
}

/// Clause-level currency of a qualification record: every recorded dependency except the
/// whole-Charter `charter` binding must match `want`; obligation meanings take its place. Record
/// validity itself is untouched and still compares `charter`.
pub fn qualification_from_record(
    record: &crate::verification::VerificationRecord,
    want: &BTreeMap<String, String>,
) -> Option<QualificationEvidence> {
    let mut want = want.clone();
    let mut recorded = record.clone();
    recorded.bindings.remove("charter");
    want.remove("charter");
    recorded.result = "passed".into(); // per-constraint status is judged by evaluate()
    if !matches!(
        crate::verification::evaluate(Some(recorded), &want),
        RecordState::Current(_)
    ) {
        return None;
    }
    let statuses = record
        .details
        .as_ref()
        .and_then(|d| d.get("constraints"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    Some((
                        item.get("id")?.as_str()?.to_string(),
                        item.get("status")?.as_str()?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    Some(QualificationEvidence {
        statuses,
        meanings: record_meanings(record.details.as_ref()),
    })
}

/// Gather the inputs for a project from its Charter, Contracts and records.
pub fn inputs(project: &Project) -> Result<Inputs> {
    let charter_path = project.charter();
    let obligations = parse_charter(&std::fs::read_to_string(&charter_path)?)?;
    let mut features = BTreeMap::new();
    for feature in project.feature_ids()? {
        let path = project.contract(&feature);
        let scenarios = parse_contract(&std::fs::read_to_string(&path)?)
            .with_context(|| format!("parse {}", path.display()))?;
        let contract_approved = crate::evidence::spec_artifact_approved(&path)?;
        let evidence = if scenarios.iter().any(|s| !s.covers.is_empty()) {
            feature_evidence(project, &feature)?
        } else {
            None
        };
        features.insert(
            feature,
            FeatureInput {
                contract_approved,
                scenarios,
                evidence,
            },
        );
    }
    Ok(Inputs {
        obligations,
        charter_approved: crate::evidence::approval_recorded(&charter_path)?,
        features,
        qualification: qualification_evidence(project)?,
    })
}

pub fn report(project: &Project) -> Result<Report> {
    Ok(evaluate(&inputs(project)?))
}

/// The obligations `feature` owns that are not VERIFIED: what blocks its F9 under v16.
pub fn unverified_owned(project: &Project, feature: &str) -> Result<Vec<String>> {
    Ok(report(project)?
        .obligations
        .into_iter()
        .filter(|r| r.owner.as_deref() == Some(feature) && r.state != State::Verified)
        .map(|r| r.id)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHARTER: &str = "\
## Outcomes

| ID | Outcome | How success is measured |
|---|---|---|
| O-1 | Readers act | answers |

## Solution Requirements

| ID | Requirement | Serves |
|---|---|---|
| R-8 | Factual integrity | O-1 |
| R-8.1 | Code checks quotes | O-1 |
| R-8.2 | Statement verdicts | O-1 |
| R-9 | Economic status | O-1 |

## System Constraints

| ID | Constraint | Type | Verification route |
|---|---|---|---|
| C-1 | Budget ten calls | quality | tests |
| C-2 | Secrets out of Git | quality | qualification |

## Milestones

| ID | Acceptance condition | Aggregates |
|---|---|---|
| M-1 | Full journey | R-8.1, C-2 |
";

    fn case(id: &str, status: &str) -> TestCase {
        TestCase {
            id: id.into(),
            file: None,
            status: status.into(),
        }
    }

    fn scenario(id: &str, covers: &[&str]) -> Scenario {
        Scenario {
            id: id.into(),
            covers: covers.iter().map(|v| v.to_string()).collect(),
            contributes: Vec::new(),
        }
    }

    fn base() -> Inputs {
        let obligations = parse_charter(CHARTER).unwrap();
        Inputs {
            obligations,
            charter_approved: true,
            ..Inputs::default()
        }
    }

    fn meanings(inputs: &Inputs, ids: &[&str]) -> Option<BTreeMap<String, String>> {
        Some(
            ids.iter()
                .filter_map(|id| inputs.obligations.get(*id))
                .map(|o| (o.id.clone(), o.meaning.clone()))
                .collect(),
        )
    }

    fn with_feature(mut inputs: Inputs, scenarios: Vec<Scenario>, cases: Vec<TestCase>) -> Inputs {
        let covered: Vec<String> = scenarios.iter().flat_map(|s| s.covers.clone()).collect();
        let refs: Vec<&str> = covered.iter().map(String::as_str).collect();
        let m = meanings(&inputs, &refs);
        inputs.features.insert(
            "F-0004".into(),
            FeatureInput {
                contract_approved: true,
                scenarios,
                evidence: Some(FeatureEvidence { cases, meanings: m }),
            },
        );
        inputs
    }

    fn state(report: &Report, id: &str) -> State {
        report
            .obligations
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .state
    }

    #[test]
    fn parses_obligation_rows_only_and_rejects_duplicates() {
        let o = parse_charter(CHARTER).unwrap();
        let report = evaluate(&Inputs {
            obligations: o.clone(),
            ..Inputs::default()
        });
        let order: Vec<&str> = report.obligations.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            order,
            ["O-1", "R-8", "R-8.1", "R-8.2", "R-9", "C-1", "C-2", "M-1"]
        );
        let ids: Vec<&str> = o.keys().map(String::as_str).collect();
        assert_eq!(
            ids,
            ["C-1", "C-2", "M-1", "O-1", "R-8", "R-8.1", "R-8.2", "R-9"]
        );
        assert_eq!(o["M-1"].aggregates, ["R-8.1", "C-2"]);
        assert!(parse_charter("| ID | X |\n|---|---|\n| R-1 | a |\n| R-1 | b |").is_err());
        assert!(
            parse_charter("| ID | X |\n|---|---|\n| R-01 | a |\n| R-1x | b |")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn meaning_ignores_whitespace_but_not_words() {
        let a = parse_charter("| ID | X |\n|---|---|\n| R-1 | ten   calls |").unwrap();
        let b = parse_charter("| ID | X |\n|---|---|\n|R-1|ten calls|").unwrap();
        let c = parse_charter("| ID | X |\n|---|---|\n| R-1 | ten calls with retries |").unwrap();
        assert_eq!(a["R-1"].meaning, b["R-1"].meaning);
        assert_ne!(a["R-1"].meaning, c["R-1"].meaning);
    }

    #[test]
    fn parses_scenarios_covers_and_ignores_fences() {
        let s = parse_contract(
            "### Happy Path [S-1]\n\ncovers: R-8.1, C-1\ncontributes: R-9\n```gherkin\ncovers: R-9\n```\n### Notes\n",
        )
        .unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].covers, ["R-8.1", "C-1"]);
        assert_eq!(s[0].contributes, ["R-9"]);
        assert!(parse_contract("### A [S-1]\n### B [S-1]\n").is_err());
        assert!(parse_contract("### Untagged\ncovers: R-1\n").is_err());
    }

    #[test]
    fn tag_tokens_are_exact() {
        assert_eq!(tag("F-0004", "S-3"), "f0004s3");
        assert!(carries_tag("budget::tests::f0004s3_quota_pause", "f0004s3"));
        assert!(carries_tag("F0004S3 quota pause", "f0004s3"));
        assert!(!carries_tag("budget::tests::f0004s30_quota", "f0004s3"));
        assert!(!carries_tag("budget::tests::xf0004s3", "f0004s3"));
    }

    #[test]
    fn verified_requires_every_mandatory_check() {
        let inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"]), scenario("S-2", &["R-8.1"])],
            vec![
                case("t::f0004s1_ok", "passed"),
                case("t::f0004s2_ok", "passed"),
            ],
        );
        assert_eq!(state(&evaluate(&inputs), "R-8.1"), State::Verified);
        let inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"]), scenario("S-2", &["R-8.1"])],
            vec![case("t::f0004s1_ok", "passed")],
        );
        assert_eq!(state(&evaluate(&inputs), "R-8.1"), State::Specified);
    }

    #[test]
    fn untagged_passing_test_cannot_verify() {
        let inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"])],
            vec![case("t::quota_pause_ok", "passed")],
        );
        assert_eq!(state(&evaluate(&inputs), "R-8.1"), State::Specified);
    }

    #[test]
    fn failed_flaky_or_only_skipped_tagged_test_cannot_verify() {
        for extra in [
            vec![
                case("t::f0004s1_a", "passed"),
                case("t::f0004s1_b", "failed"),
            ],
            vec![
                case("t::f0004s1_a", "passed"),
                case("t::f0004s1_b", "flaky"),
            ],
            vec![case("t::f0004s1_a", "skipped")],
        ] {
            let inputs = with_feature(base(), vec![scenario("S-1", &["R-8.1"])], extra);
            assert_eq!(state(&evaluate(&inputs), "R-8.1"), State::Specified);
        }
    }

    #[test]
    fn stale_tagged_test_cannot_verify() {
        // No current required record: feature evidence is absent.
        let mut inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"])],
            vec![case("t::f0004s1_ok", "passed")],
        );
        inputs.features.get_mut("F-0004").unwrap().evidence = None;
        assert_eq!(state(&evaluate(&inputs), "R-8.1"), State::Specified);
    }

    #[test]
    fn clause_text_change_under_same_id_requires_reassessment() {
        let inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"])],
            vec![case("t::f0004s1_ok", "passed")],
        );
        let mut changed = inputs.clone();
        let revised =
            parse_charter(&CHARTER.replace("Code checks quotes", "Code checks quotes and retries"))
                .unwrap();
        changed.obligations = revised;
        let report = evaluate(&changed);
        assert_eq!(state(&report, "R-8.1"), State::Specified);
        assert!(report
            .obligations
            .iter()
            .any(|r| r.id == "R-8.1" && r.notes.iter().any(|n| n.contains("earlier meaning"))));
    }

    #[test]
    fn unrelated_charter_change_keeps_clause_results_current() {
        let inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"])],
            vec![case("t::f0004s1_ok", "passed")],
        );
        let mut changed = inputs.clone();
        changed.obligations =
            parse_charter(&CHARTER.replace("Economic status", "Economic status always")).unwrap();
        assert_ne!(
            crate::hashing::text_sha256(CHARTER),
            crate::hashing::text_sha256(
                &CHARTER.replace("Economic status", "Economic status always")
            )
        );
        assert_eq!(state(&evaluate(&changed), "R-8.1"), State::Verified);
    }

    #[test]
    fn legacy_evidence_never_counts() {
        let mut inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"])],
            vec![case("t::f0004s1_ok", "passed")],
        );
        inputs
            .features
            .get_mut("F-0004")
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .meanings = None;
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "R-8.1"), State::Specified);
        assert!(report
            .obligations
            .iter()
            .any(|r| r.id == "R-8.1" && r.notes.iter().any(|n| n.contains("legacy"))));
    }

    #[test]
    fn contributions_alone_never_establish_compliance() {
        let mut inputs = base();
        inputs.features.insert(
            "F-0002".into(),
            FeatureInput {
                contract_approved: true,
                scenarios: vec![Scenario {
                    id: "S-1".into(),
                    covers: Vec::new(),
                    contributes: vec!["R-8.1".into()],
                }],
                evidence: Some(FeatureEvidence {
                    cases: vec![case("t::f0002s1", "passed")],
                    meanings: Some(BTreeMap::new()),
                }),
            },
        );
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "R-8.1"), State::Unassessed);
        assert!(report
            .findings
            .iter()
            .any(|f| f.contains("R-8.1 has no accountable verification owner")));
    }

    #[test]
    fn two_owners_unknown_and_parent_covers_are_findings() {
        let mut inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1", "R-77", "R-8"])],
            vec![case("t::f0004s1", "passed")],
        );
        inputs.features.insert(
            "F-0005".into(),
            FeatureInput {
                contract_approved: true,
                scenarios: vec![scenario("S-1", &["R-8.1"])],
                evidence: None,
            },
        );
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "R-8.1"), State::Unassessed);
        let joined = report.findings.join("\n");
        assert!(joined.contains("more than one feature"));
        assert!(joined.contains("unknown or retired obligation R-77"));
        assert!(joined.contains("R-8, which has clauses"));
    }

    #[test]
    fn parent_takes_weakest_clause_state() {
        let inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"])],
            vec![case("t::f0004s1", "passed")],
        );
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "R-8.1"), State::Verified);
        assert_eq!(state(&report, "R-8.2"), State::Unassessed);
        assert_eq!(state(&report, "R-8"), State::Unassessed);
    }

    #[test]
    fn qualification_owns_unclaused_constraints() {
        let mut inputs = base();
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "C-2"), State::Specified);
        inputs.qualification = Some(QualificationEvidence {
            statuses: [("C-1", "fail"), ("C-2", "pass")]
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect(),
            meanings: meanings(&inputs, &["C-1", "C-2"]),
        });
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "C-2"), State::Verified);
        assert_eq!(state(&report, "C-1"), State::Specified);
        inputs.charter_approved = false;
        assert_eq!(state(&evaluate(&inputs), "C-2"), State::Allocated);
    }

    #[test]
    fn milestone_aggregates_and_reports_invalid_aggregation() {
        let mut inputs = with_feature(
            base(),
            vec![scenario("S-1", &["R-8.1"])],
            vec![case("t::f0004s1", "passed")],
        );
        assert_eq!(state(&evaluate(&inputs), "M-1"), State::Specified);
        inputs.qualification = Some(QualificationEvidence {
            statuses: [("C-2".to_string(), "pass".to_string())]
                .into_iter()
                .collect(),
            meanings: meanings(&inputs, &["C-2"]),
        });
        assert_eq!(state(&evaluate(&inputs), "M-1"), State::Verified);
        inputs.obligations =
            parse_charter(&CHARTER.replace("| R-8.1, C-2 |", "| R-404, M-1 |")).unwrap();
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "M-1"), State::Unassessed);
        let joined = report.findings.join("\n");
        assert!(joined.contains("aggregates unknown obligation R-404"));
        assert!(joined.contains("aggregates itself"));
        inputs.obligations = parse_charter(&CHARTER.replace("| R-8.1, C-2 |", "| |")).unwrap();
        assert!(evaluate(&inputs)
            .findings
            .iter()
            .any(|f| f.contains("M-1 aggregates no obligation")));
    }

    #[test]
    fn unrelated_charter_edit_keeps_clause_results_while_whole_charter_provenance_holds() {
        let before = CHARTER;
        let after = CHARTER.replace("Economic status", "Economic status always");
        let obligations = parse_charter(before).unwrap();
        let record = crate::verification::VerificationRecord {
            workflow: "feature".into(),
            subject: "F-0004".into(),
            checkpoint: "candidate".into(),
            verification: "project_qualification".into(),
            result: "failed".into(),
            bindings: [
                ("charter", crate::hashing::text_sha256(before)),
                ("qualification_script", "script-1".to_string()),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
            command: "scripts/codeos-qualify".into(),
            timestamp: "2026-11-26T00:00:00Z".into(),
            details: Some(serde_json::json!({
                "constraints": [{"id": "C-1", "status": "fail"}, {"id": "C-2", "status": "pass"}],
                DETAILS_KEY: {"C-1": obligations["C-1"].meaning, "C-2": obligations["C-2"].meaning},
            })),
        };
        let want = |charter: &str, script: &str| -> BTreeMap<String, String> {
            [
                ("charter", crate::hashing::text_sha256(charter)),
                ("qualification_script", script.to_string()),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
        };
        // Full-artifact provenance: the record itself is stale on the whole-Charter hash.
        assert!(matches!(
            crate::verification::evaluate(Some(record.clone()), &want(&after, "script-1")),
            RecordState::Stale { ref binding, .. } if binding == "charter"
        ));
        // Clause level: the unrelated edit leaves C-2 VERIFIED.
        let qualification = qualification_from_record(&record, &want(&after, "script-1"));
        let inputs = Inputs {
            obligations: parse_charter(&after).unwrap(),
            charter_approved: true,
            qualification,
            ..Inputs::default()
        };
        assert_eq!(state(&evaluate(&inputs), "C-2"), State::Verified);
        assert_eq!(state(&evaluate(&inputs), "C-1"), State::Specified);
        // Editing C-2 itself stales it at clause level.
        let edited = CHARTER.replace("Secrets out of Git", "Secrets out of Git and logs");
        let inputs = Inputs {
            obligations: parse_charter(&edited).unwrap(),
            charter_approved: true,
            qualification: qualification_from_record(&record, &want(&edited, "script-1")),
            ..Inputs::default()
        };
        assert_eq!(state(&evaluate(&inputs), "C-2"), State::Specified);
        // Every other dependency still governs: a changed qualification script stales it.
        assert!(qualification_from_record(&record, &want(&after, "script-2")).is_none());
    }

    fn record(
        verification: &str,
        result: &str,
        meanings: Value,
    ) -> crate::verification::VerificationRecord {
        crate::verification::VerificationRecord {
            workflow: "feature".into(),
            subject: "F-0004".into(),
            checkpoint: "F5".into(),
            verification: verification.into(),
            result: result.into(),
            bindings: BTreeMap::new(),
            command: "test".into(),
            timestamp: "2026-11-26T00:00:00Z".into(),
            details: Some(serde_json::json!({ DETAILS_KEY: meanings })),
        }
    }

    fn evidence(cases: Vec<TestCase>) -> TestEvidence {
        TestEvidence {
            schema: 1,
            verification: "behavior".into(),
            runs: vec![crate::test_evidence::TestRun {
                runner: "libtest".into(),
                attempt: 1,
                cases,
                duration_ms: None,
            }],
        }
    }

    #[test]
    fn passing_feature_scenarios_cannot_verify_when_the_integrated_journey_fails() {
        let inputs = base();
        let m = serde_json::json!({ "R-8.1": inputs.obligations["R-8.1"].meaning });
        let behavior = || {
            (
                "behavior",
                RecordState::Current(record("behavior", "passed", m.clone())),
                Some(evidence(vec![case("t::f0004s1", "passed")])),
            )
        };
        let passing = combine(vec![
            behavior(),
            (
                "product_journey",
                RecordState::Current(record("product_journey", "passed", m.clone())),
                None,
            ),
        ]);
        let failing = combine(vec![
            behavior(),
            (
                "product_journey",
                RecordState::Failed(record("product_journey", "failed", m.clone())),
                None,
            ),
        ]);
        let stale = combine(vec![(
            "behavior",
            RecordState::Stale {
                record: record("behavior", "passed", m.clone()),
                binding: "implementation_state".into(),
                was: "a".into(),
                now: "b".into(),
                comparisons: Vec::new(),
            },
            Some(evidence(vec![case("t::f0004s1", "passed")])),
        )]);
        for (evidence, expected) in [
            (passing, State::Verified),
            (failing, State::Specified),
            (stale, State::Specified),
        ] {
            let mut inputs = base();
            inputs.features.insert(
                "F-0004".into(),
                FeatureInput {
                    contract_approved: true,
                    scenarios: vec![scenario("S-1", &["R-8.1"])],
                    evidence,
                },
            );
            assert_eq!(state(&evaluate(&inputs), "R-8.1"), expected);
        }
    }

    #[test]
    fn legacy_charter_reports_unassessed_not_verified() {
        let legacy = "| ID | Outcome | Measure |\n|---|---|---|\n| O-1 | a | b |\n\n| ID | Constraint | Type | Route |\n|---|---|---|---|\n| C-1 | c | quality | q |\n";
        let inputs = Inputs {
            obligations: parse_charter(legacy).unwrap(),
            charter_approved: true,
            ..Inputs::default()
        };
        let report = evaluate(&inputs);
        assert_eq!(state(&report, "O-1"), State::Unassessed);
        assert_eq!(state(&report, "C-1"), State::Specified);
        assert!(report
            .obligations
            .iter()
            .all(|r| r.state != State::Verified));
    }
}
