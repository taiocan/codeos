//! REVIEW-SCALE accounting: every byte a packet counts toward its budget has exactly one accounting
//! role, derived from the recorded reasons the input was included. Operational telemetry only: it
//! never affects readiness, packet content selection, or a review outcome.

use crate::packet::ReviewPacket;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

pub const CANDIDATE: &str = "candidate";
pub const GOVERNED: &str = "governed";
pub const CHANGED_IMPLEMENTATION: &str = "changed_implementation";
pub const PRODUCTION_EVIDENCE: &str = "production_evidence";
pub const RUNTIME_EVIDENCE: &str = "runtime_evidence";
pub const TEST_EVIDENCE: &str = "test_evidence";
pub const TEST_DEFINITION: &str = "test_definition";
pub const VERIFICATION_RECORDS: &str = "verification_records";
pub const DECLARED: &str = "declared";
pub const UNRECORDED: &str = "unrecorded";

/// Role precedence: an input's accounting role is the first of its reasons in this list, so an
/// input included for several reasons is counted once.
pub const PRECEDENCE: &[&str] = &[
    CANDIDATE,
    GOVERNED,
    CHANGED_IMPLEMENTATION,
    PRODUCTION_EVIDENCE,
    RUNTIME_EVIDENCE,
    TEST_EVIDENCE,
    TEST_DEFINITION,
    VERIFICATION_RECORDS,
    DECLARED,
];

const DIFF: &str = "(diff)";
const RECORDS: &str = "(required verification records)";

pub fn role(reasons: &BTreeSet<String>) -> Option<&'static str> {
    PRECEDENCE
        .iter()
        .find(|role| reasons.contains(**role))
        .copied()
}

/// Whether a role was recorded by the manifest, inferred from an unambiguous path prefix for a
/// packet built before accounting existed, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Recorded,
    Inferred,
    Unrecorded,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Contribution {
    pub path: String,
    pub role: &'static str,
    pub source: Source,
    pub bytes: u64,
}

fn inferred(path: &str) -> Option<&'static str> {
    [
        (".codeos/06-workflow/candidates/", CANDIDATE),
        (".codeos/00-project/", GOVERNED),
        (".codeos/01-specification/", GOVERNED),
        (".codeos/02-architecture/", GOVERNED),
        (".codeos/06-workflow/reality-evidence/", PRODUCTION_EVIDENCE),
        (".codeos/06-workflow/event-evidence/", RUNTIME_EVIDENCE),
        (".codeos/06-workflow/test-evidence/", TEST_EVIDENCE),
    ]
    .into_iter()
    .find(|(prefix, _)| path.starts_with(prefix))
    .map(|(_, role)| role)
}

/// The inclusion reasons a packet's manifest recorded; empty before accounting existed.
pub fn inclusion(packet: &ReviewPacket) -> BTreeMap<String, BTreeSet<String>> {
    packet
        .manifest
        .as_ref()
        .map(|manifest| manifest.inclusion.clone())
        .unwrap_or_default()
}

/// Every budget contributor with its one accounting role. The bytes sum to
/// `review_content_bytes` because they are exactly the packet's budget contributors.
pub fn contributions(
    packet: &ReviewPacket,
    inclusion: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Contribution> {
    packet
        .budget_contributors
        .iter()
        .map(|(path, bytes)| {
            let (role, source) = match path.as_str() {
                DIFF => (CHANGED_IMPLEMENTATION, Source::Recorded),
                RECORDS => (VERIFICATION_RECORDS, Source::Recorded),
                _ => match inclusion.get(path).and_then(role) {
                    Some(role) => (role, Source::Recorded),
                    None => match inferred(path) {
                        Some(role) => (role, Source::Inferred),
                        None => (UNRECORDED, Source::Unrecorded),
                    },
                },
            };
            Contribution {
                path: path.clone(),
                role,
                source,
                bytes: *bytes,
            }
        })
        .collect()
}

/// Bytes by role in precedence order, then `unrecorded`.
pub fn by_role(contributions: &[Contribution]) -> Vec<(&'static str, u64)> {
    PRECEDENCE
        .iter()
        .chain(std::iter::once(&UNRECORDED))
        .filter_map(|role| {
            let bytes: u64 = contributions
                .iter()
                .filter(|c| c.role == *role)
                .map(|c| c.bytes)
                .sum();
            (bytes > 0).then_some((*role, bytes))
        })
        .collect()
}

/// The manifest lines that say why `path` is in the packet.
pub fn manifest_lines(inclusion: &BTreeMap<String, BTreeSet<String>>, path: &str) -> String {
    match inclusion.get(path) {
        Some(reasons) => format!(
            "      role: {}\n      reasons: {}\n",
            role(reasons).unwrap_or(UNRECORDED),
            reasons.iter().cloned().collect::<Vec<_>>().join(", ")
        ),
        None => String::new(),
    }
}

pub fn kb(bytes: u64) -> u64 {
    bytes.div_ceil(1024)
}

/// Packet composition by role and its three largest contributors.
pub fn print_composition(packet: &ReviewPacket, inclusion: &BTreeMap<String, BTreeSet<String>>) {
    let contributions = contributions(packet, inclusion);
    let percent = |bytes: u64, of: u64| if of == 0 { 0 } else { bytes * 100 / of };
    println!(
        "  packet: {} KB / {} KB budget [{}%]",
        kb(packet.review_content_bytes),
        kb(packet.budget_bytes),
        percent(packet.review_content_bytes, packet.budget_bytes)
    );
    for (role, bytes) in by_role(&contributions) {
        let inferred = contributions
            .iter()
            .any(|c| c.role == role && c.source == Source::Inferred);
        println!(
            "    {role:<22} {:>5} KB{}",
            kb(bytes),
            if inferred { "  (inferred)" } else { "" }
        );
    }
    let mut largest = contributions;
    largest.sort_by(|a, b| b.bytes.cmp(&a.bytes).then(a.path.cmp(&b.path)));
    for c in largest.iter().take(3) {
        println!("    largest: {} {} KB ({})", c.path, kb(c.bytes), c.role);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ReviewInputManifest;

    fn packet(contributors: &[(&str, u64)], inclusion: &[(&str, &[&str])]) -> ReviewPacket {
        let mut manifest = ReviewInputManifest::default();
        for (path, reasons) in inclusion {
            manifest.inclusion.insert(
                path.to_string(),
                reasons.iter().map(|r| r.to_string()).collect(),
            );
        }
        let mut packet = ReviewPacket::from_immutable_record(
            "F-1".into(),
            "8".into(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            crate::packet::CoverageState::FullCoverage,
            false,
            0,
            false,
            Vec::new(),
            Vec::new(),
            String::new(),
        );
        packet.budget_contributors = contributors
            .iter()
            .map(|(path, bytes)| (path.to_string(), *bytes))
            .collect();
        packet.review_content_bytes = contributors.iter().map(|(_, bytes)| bytes).sum();
        packet.manifest = Some(manifest);
        packet
    }

    #[test]
    fn a_changed_file_that_defines_tests_counts_once_as_changed_implementation() {
        let packet = packet(
            &[
                ("backend/src/main.rs", 45),
                ("web/a.e2e.ts", 40),
                (DIFF, 3),
                (RECORDS, 7),
            ],
            &[
                (
                    "backend/src/main.rs",
                    &[TEST_DEFINITION, CHANGED_IMPLEMENTATION],
                ),
                ("web/a.e2e.ts", &[TEST_DEFINITION]),
            ],
        );
        let contributions = contributions(&packet, &inclusion(&packet));
        assert_eq!(contributions[0].role, CHANGED_IMPLEMENTATION);
        assert_eq!(
            by_role(&contributions),
            [
                (CHANGED_IMPLEMENTATION, 48),
                (TEST_DEFINITION, 40),
                (VERIFICATION_RECORDS, 7)
            ]
        );
        let total: u64 = by_role(&contributions).iter().map(|(_, b)| b).sum();
        assert_eq!(total, packet.review_content_bytes);
        assert_eq!(
            manifest_lines(&packet.manifest.as_ref().unwrap().inclusion, "backend/src/main.rs"),
            "      role: changed_implementation\n      reasons: changed_implementation, test_definition\n"
        );
    }

    #[test]
    fn older_packets_get_inferred_or_unrecorded_roles_never_recorded_ones() {
        let packet = packet(
            &[
                (".codeos/06-workflow/reality-evidence/x.json", 100),
                ("backend/src/provider.rs", 48),
            ],
            &[],
        );
        let contributions = contributions(&packet, &inclusion(&packet));
        assert_eq!(
            (contributions[0].role, contributions[0].source),
            (PRODUCTION_EVIDENCE, Source::Inferred)
        );
        assert_eq!(
            (contributions[1].role, contributions[1].source),
            (UNRECORDED, Source::Unrecorded)
        );
    }
}
