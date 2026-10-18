//! Feature Workflow v13 shared validation (DQA WO-06): one execution or observation may serve
//! several features' explicit claims; every judgment and acceptance stays feature-specific.
//! Runs under DBA-29; never against a provider.

mod fai_fixture;

use codeos_workflow::checker::{run_checks, run_shared_production_check, State};
use codeos_workflow::execution::Environment;
use codeos_workflow::session::{self, Outcome};
use codeos_workflow::verification::{VerificationRecord, VerificationStore};
use fai_fixture::*;
use std::collections::BTreeMap;
use std::fs;

const DETERMINISTIC: Option<Environment> = Some(Environment::Deterministic);

fn candidate(scopes: &str) -> Fixture {
    Fixture::under("DBA-29", scopes)
}

fn record(fx: &Fixture, feature: &str, checkpoint: &str, mechanic: &str) -> VerificationRecord {
    VerificationStore::at(&fx.root.join(".codeos"))
        .latest("feature", feature, checkpoint, mechanic)
        .unwrap()
        .unwrap_or_else(|| panic!("{feature} {checkpoint}/{mechanic} has no record"))
}

fn cited_set(fx: &Fixture, feature: &str, checkpoint: &str, mechanic: &str) -> String {
    record(fx, feature, checkpoint, mechanic).details.unwrap()["evidence_set"]
        .as_str()
        .unwrap()
        .to_string()
}

fn sets(fx: &Fixture) -> usize {
    let root = fx.root.join(".codeos/06-workflow/evidence-sets");
    fs::read_dir(&root)
        .map(|dirs| {
            dirs.map(|dir| fs::read_dir(dir.unwrap().path()).unwrap().count())
                .sum()
        })
        .unwrap_or(0)
}

fn commit(fx: &Fixture, message: &str) {
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", message, "--allow-empty"]);
}

/// Criterion 1: one compatible validation identity produces one execution record regardless of
/// how many features consume it.
#[test]
fn one_execution_serves_every_feature_with_the_same_execution_identity() {
    let fx = candidate("    provider: [backend]\n");
    fx.spec("F-0001", "mode: preserve\nreason: one", "");
    fx.spec("F-0002", "mode: preserve\nreason: two", "");
    commit(&fx, "specs");
    for feature in ["F-0001", "F-0002"] {
        fx.seed(feature, "F3", "smoke");
        fx.seed(feature, "F3", "product_journey");
    }
    let first = run_checks(&fx.project(), &fx.subject("F-0001"), DETERMINISTIC).unwrap();
    let before = sets(&fx);
    let second = run_checks(&fx.project(), &fx.subject("F-0002"), DETERMINISTIC).unwrap();
    assert!(
        second
            .iter()
            .filter(|l| l.contains("reused Validation Evidence Set"))
            .count()
            == 2,
        "{first:?}\n{second:?}"
    );
    assert_eq!(sets(&fx), before, "the second feature executed nothing");
    for mechanic in ["behavior", "repeatability"] {
        assert_eq!(
            cited_set(&fx, "F-0001", "F5", mechanic),
            cited_set(&fx, "F-0002", "F5", mechanic)
        );
    }
    assert_eq!(fx.state("F-0002", "F5").0, State::Pass);
}

/// WO-04 finding: a change to the fixture provider stales the records that consumed it, though
/// the implementation state is unchanged.
#[test]
fn a_fixture_provider_change_stales_what_consumed_it() {
    let fx = candidate("    provider: [backend]\n");
    fx.write(
        "docker-compose.yml",
        "services:\n  boundary:\n    build: ./provider-boundary\n",
    );
    fx.write("provider-boundary/server.mjs", "// emulator v1\n");
    fx.spec("F-0001", "mode: preserve\nreason: one", "");
    commit(&fx, "emulator");
    fx.seed("F-0001", "F3", "smoke");
    fx.seed("F-0001", "F3", "product_journey");
    run_checks(&fx.project(), &fx.subject("F-0001"), DETERMINISTIC).unwrap();
    assert_eq!(fx.state("F-0001", "F5").0, State::Pass);
    let implementation = fx.project().implementation_state().unwrap();

    fx.write("provider-boundary/server.mjs", "// emulator v2\n");
    assert_eq!(fx.project().implementation_state().unwrap(), implementation);
    let (state, unmet) = fx.state("F-0001", "F3");
    assert_ne!(state, State::Pass);
    assert!(
        unmet.iter().any(|u| u.contains("execution_identity")),
        "{unmet:?}"
    );
}

/// A schema-3 verifier: one observation per claim, with owner-qualified claim IDs, sharing one
/// provider receipt; its adapter ledger records that one call.
const SHARED_VERIFIER: &str = r#"#!/usr/bin/env bash
set -euo pipefail
printf '{"schema":1,"attempt":1,"event":"attempt","at":"2026-10-18T00:00:00Z"}\n{"schema":1,"attempt":1,"event":"success","at":"2026-10-18T00:00:01Z"}\n' >> "$CODEOS_PROVIDER_LEDGER"
h=$(printf 'x' | sha256sum | cut -c1-64)
receipts='[{"provider_host":"provider.test","provider_path":"/route","request_sha256":"'$h'","response_sha256":"'$h'","provider_request_id":null,"requested_at":"2026-10-18T00:00:00Z","received_at":"2026-10-18T00:00:01Z","http_status":200}]'
visible='"ok"'
vs=$(printf '%s' "$visible" | sha256sum | cut -c1-64)
rs=$(printf '%s' "$receipts" | sha256sum | cut -c1-64)
jq -n --slurpfile d "$CODEOS_REALITY_DEFINITION_PATH" --slurpfile s claim-scopes.json \
  --argjson r "$receipts" --argjson v "$visible" --arg vs "$vs" --arg rs "$rs" '
  $d[0] as $d | {schema: 3,
    subjects: [$d.subjects[] | {subject, claims, governed_inputs, claim_scopes: ($s[0][.subject] // {})}],
    source_kind: "production", provider: "provider.test", observed_at: "2026-10-18T00:00:00Z",
    observations: [{claim_ids: [$d.subjects[] | .subject as $f | .claims[] | "\($f)/\(.)"],
      visible_result: $v, visible_result_sha256: $vs,
      provider_receipts_sha256: $rs, provider_request_response_receipts: $r, details: {}}]}' \
  > "$CODEOS_REALITY_EVIDENCE_PATH"
"#;

/// Two features' reality-dependent claims through F4, deterministic evidence and qualification
/// current, waiting only on production evidence.
fn waiting_on_production(features: &[&str]) -> Fixture {
    let fx = candidate("    provider: [backend]\n");
    let scopes: BTreeMap<_, _> = features
        .iter()
        .map(|f| (f.to_string(), serde_json::json!({ "route": ["provider"] })))
        .collect();
    fx.write(
        "claim-scopes.json",
        &serde_json::to_string(&scopes).unwrap(),
    );
    fx.write("scripts/verify.sh", SHARED_VERIFIER);
    for feature in features {
        fx.spec(feature, "mode: preserve\nreason: reads the route", ROUTE);
    }
    commit(&fx, "specs");
    for feature in features {
        fx.seed(feature, "F3", "smoke");
        fx.seed(feature, "F3", "product_journey");
        run_checks(&fx.project(), &fx.subject(feature), DETERMINISTIC).unwrap();
        fx.seed(feature, "candidate", "project_qualification");
    }
    fx
}

/// Criteria 2 and 4: one production run serves two features' distinct claims; each record cites
/// the one set and its own evidence slice, and a feature not named cannot use it.
#[test]
fn one_production_run_serves_the_named_features_without_merging_their_claims() {
    let fx = waiting_on_production(&["F-0001", "F-0002", "F-0003"]);
    let subjects = [fx.subject("F-0001"), fx.subject("F-0002")];
    let log = run_shared_production_check(&fx.project(), &subjects).unwrap();
    assert_eq!(fx.state("F-0001", "F5").0, State::Pass, "{log:?}");
    assert_eq!(fx.state("F-0002", "F5").0, State::Pass, "{log:?}");
    assert_ne!(
        fx.state("F-0003", "F5").0,
        State::Pass,
        "an unnamed feature gained evidence"
    );
    let set = cited_set(&fx, "F-0001", "F5", "production_source_evidence");
    assert_eq!(
        set,
        cited_set(&fx, "F-0002", "F5", "production_source_evidence")
    );
    for feature in ["F-0001", "F-0002"] {
        let slice: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(
                fx.root
                    .join(".codeos/06-workflow/reality-evidence")
                    .join(format!("{}.json", fx.evidence(feature))),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(slice["subject"], feature);
        assert_eq!(
            slice["observations"][0]["claim_ids"],
            serde_json::json!(["route"])
        );
    }
    assert_ne!(fx.evidence("F-0001"), fx.evidence("F-0002"));
    let ledgers = fs::read_dir(fx.root.join(".codeos/06-workflow/provider-ledger"))
        .unwrap()
        .filter(|e| e.as_ref().unwrap().path().extension().unwrap() == "jsonl")
        .count();
    assert_eq!(ledgers, 1, "one provider run");
}

#[test]
fn a_shared_production_run_starts_nothing_unless_every_named_feature_is_ready() {
    let fx = waiting_on_production(&["F-0001"]);
    fx.spec("F-0002", "mode: preserve\nreason: reads the route", ROUTE);
    commit(&fx, "late spec");
    let subjects = [fx.subject("F-0001"), fx.subject("F-0002")];
    let log = run_shared_production_check(&fx.project(), &subjects).unwrap();
    assert!(
        log.iter().any(|l| l.contains("started nothing; F-0002")),
        "{log:?}"
    );
    assert!(!fx.root.join(".codeos/06-workflow/provider-ledger").exists());
}

/// Criterion 4 of WO-06: a stale accepted feature stays in the journey topology; position alone
/// blocks no later feature, while a claim inherited from it stays blocked until it is current.
#[test]
fn a_stale_producer_stays_in_the_topology_and_blocks_only_what_consumes_it() {
    let fx = candidate("    provider: [backend]\n");
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    commit(&fx, "spec");
    fx.check("F-0001");
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
    commit(&fx, "stale F-0001");
    assert!(
        !codeos_workflow::revalidation::current_acceptance_valid(&fx.project(), "F-0001").unwrap()
    );
    assert_eq!(
        codeos_workflow::checker::current_accepted_frontier(&fx.project()).unwrap(),
        "decided",
        "staleness does not remove the accepted step"
    );

    // A later feature in the same journey with no currentness dependency on F-0001.
    fx.spec(
        "F-0002",
        "mode: advance\nprevious_frontier: decided\nproposed_frontier: returned\ntransition: the driver returns",
        "",
    );
    // A feature inheriting F-0001's claim.
    fx.spec(
        "F-0003",
        "mode: preserve\nreason: summarizes the route",
        "| summary | Route summary | inherited | F-0001/route | as F-0001 | as F-0001 | none | simulated |\n",
    );
    commit(&fx, "later features");
    for feature in ["F-0002", "F-0003"] {
        fx.seed(feature, "F3", "smoke");
        fx.seed(feature, "F3", "product_journey");
    }
    assert_eq!(
        fx.state("F-0002", "F3").0,
        State::Pass,
        "position alone blocks nothing"
    );
    run_checks(&fx.project(), &fx.subject("F-0002"), DETERMINISTIC).unwrap();
    assert_eq!(fx.state("F-0002", "F5").0, State::Pass);
    run_checks(&fx.project(), &fx.subject("F-0003"), DETERMINISTIC).unwrap();
    let (state, unmet) = fx.state("F-0003", "F5");
    assert_ne!(
        state,
        State::Pass,
        "an inherited claim from a stale feature stays blocked"
    );
    assert!(unmet.iter().any(|u| u.contains("summary")), "{unmet:?}");
}

/// The physical run is shared; each feature's validity is not. B's missing claim fails B alone.
#[test]
fn a_feature_the_shared_run_does_not_establish_fails_alone() {
    let fx = waiting_on_production(&["F-0001", "F-0002"]);
    // The verifier observes only F-0001's claim.
    fx.write(
        "scripts/verify.sh",
        &SHARED_VERIFIER.replace(
            r#"[$d.subjects[] | .subject as $f | .claims[] | "\($f)/\(.)"]"#,
            r#"["F-0001/route"]"#,
        ),
    );
    commit(&fx, "partial verifier");
    let subjects = [fx.subject("F-0001"), fx.subject("F-0002")];
    let log = run_shared_production_check(&fx.project(), &subjects).unwrap();
    assert_eq!(fx.state("F-0001", "F5").0, State::Pass, "{log:?}");
    assert_ne!(fx.state("F-0002", "F5").0, State::Pass);
    assert!(
        log.iter()
            .any(|l| l.contains("FAILED for F-0002") && l.contains("do not cover claim route")),
        "{log:?}"
    );
}

fn claims_table(fx: &Fixture, feature: &str, rows: &str) {
    let path = fx.root.join(format!(
        ".codeos/01-specification/contracts/{feature}_contract.md"
    ));
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        path,
        format!("{text}\n## Validation Claims\n\n| Claim ID | Claim | Checkpoint | Evidence |\n|---|---|---|---|\n{rows}"),
    )
    .unwrap();
}

/// Criterion 3 of WO-05: claims Codeos cannot attribute fail closed.
#[test]
fn an_unattributable_claim_table_fails_closed() {
    for (rows, why) in [
        ("| F-0002/route | Shown. | F5 | run:behavior |\n", "is not owned by F-0001"),
        ("| F-0001/route | Shown. | F5 | run:behavior |\n| F-0001/route | Again. | F5 | run:behavior |\n", "declared twice"),
        ("| F-0001/route | Shown. | F5 | assertion:behavior |\n", "unknown evidence"),
        ("| F-0001/route | Shown. | F5 | run:playwright |\n", "does not require"),
        ("| F-0001/route | Shown. | F5 | observation:production |\n", "not an F5 reality-dependent claim"),
        ("| F-0001/route | Shown. | F4 | observation:human |\n", "which does not apply"),
    ] {
        let fx = candidate("    provider: [backend]\n");
        fx.spec("F-0001", "mode: preserve\nreason: one", "");
        claims_table(&fx, "F-0001", rows);
        let reports =
            codeos_workflow::checker::evaluate(&fx.project(), &fx.subject("F-0001")).unwrap();
        let report = reports.iter().find(|r| r.id == "validation-claims");
        assert!(
            report.is_some_and(|r| r.unmet.iter().any(|u| u.contains(why))),
            "{why}: {reports:#?}"
        );
    }
}

/// Criterion 3 of WO-05: an ID retained after its meaning changed fails closed; a successor ID
/// does not.
#[test]
fn a_retained_claim_id_with_a_changed_meaning_fails_closed() {
    let fx = candidate("    provider: [backend]\n");
    fx.spec("F-0001", "mode: preserve\nreason: one", "");
    claims_table(
        &fx,
        "F-0001",
        "| F-0001/route | The route is shown. | F5 | run:behavior |\n",
    );
    commit(&fx, "spec");
    fx.check("F-0001");
    fx.accept("F-0001", None);
    let contract = fx
        .root
        .join(".codeos/01-specification/contracts/F-0001_contract.md");
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        text.replace("The route is shown.", "The route and its   toll are shown."),
    )
    .unwrap();
    let reports = codeos_workflow::checker::evaluate(&fx.project(), &fx.subject("F-0001")).unwrap();
    assert!(
        reports.iter().any(|r| r.id == "validation-claims"
            && r.unmet
                .iter()
                .any(|u| u.contains("F-0001/route changed meaning"))),
        "{reports:#?}"
    );
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        text.replace("| F-0001/route |", "| F-0001/route-with-toll |"),
    )
    .unwrap();
    let reports = codeos_workflow::checker::evaluate(&fx.project(), &fx.subject("F-0001")).unwrap();
    assert!(
        !reports.iter().any(|r| r.id == "validation-claims"),
        "{reports:#?}"
    );
}

/// A GUI feature's Specification Package; `observe` then puts it at F4.
fn gui_spec(fx: &Fixture, feature: &str) {
    fx.spec(feature, "mode: preserve\nreason: shows the page", "");
    let contract = fx.root.join(format!(
        ".codeos/01-specification/contracts/{feature}_contract.md"
    ));
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        text.replace(
            "| GUI (Svelte) | unchanged | none |",
            "| GUI (Svelte) | changed | page |",
        ),
    )
    .unwrap();
}

fn observe(fx: &Fixture, feature: &str, surface: &str) {
    fx.seed(feature, "F3", "smoke");
    let bindings = codeos_workflow::checker::verification_bindings_for_mechanic(
        &fx.project(),
        &fx.subject(feature),
        "F3",
        "product_journey",
    )
    .unwrap();
    VerificationStore::at(&fx.root.join(".codeos"))
        .append(&VerificationRecord {
            workflow: "feature".into(),
            subject: feature.into(),
            checkpoint: "F3".into(),
            verification: "product_journey".into(),
            result: "passed".into(),
            bindings,
            command: "seeded".into(),
            timestamp: "2026-10-18T00:00:00Z".into(),
            details: Some(serde_json::json!({ "observed_surface": surface })),
        })
        .unwrap();
}

fn outcome(feature: &str, result: &str, claims: &[(&str, &str)]) -> Outcome {
    Outcome {
        feature: feature.into(),
        checkpoint: "early_preview".into(),
        result: result.into(),
        claims: claims
            .iter()
            .map(|(c, v)| (c.to_string(), v.to_string()))
            .collect(),
    }
}

/// Criterion 3 of WO-06 and WO-05 criteria 2–3: one session, mixed explicit outcomes, nothing
/// propagated, nothing added later.
#[test]
fn one_session_records_mixed_explicit_outcomes_and_nothing_more() {
    let fx = candidate("    provider: [backend]\n");
    for feature in ["F-0001", "F-0002", "F-0003"] {
        gui_spec(&fx, feature);
    }
    claims_table(
        &fx,
        "F-0001",
        "| F-0001/readable | The page is readable. | F4 | observation:human |\n",
    );
    commit(&fx, "specs");
    for feature in ["F-0001", "F-0002", "F-0003"] {
        observe(&fx, feature, "surface-1");
    }
    let project = fx.project();

    // The human claim must be stated explicitly.
    let error = session::record(
        &project,
        vec![outcome("F-0001", "direction_confirmed", &[])],
        "seen",
    )
    .unwrap_err();
    assert!(error.to_string().contains("F-0001/readable"), "{error}");
    // One outcome per feature; only a current question.
    let error = session::record(
        &project,
        vec![
            outcome("F-0002", "direction_confirmed", &[]),
            outcome("F-0002", "direction_confirmed", &[]),
        ],
        "seen",
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("more than one outcome"),
        "{error}"
    );

    let id = session::record(
        &project,
        vec![
            outcome(
                "F-0001",
                "direction_confirmed",
                &[("F-0001/readable", "met")],
            ),
            outcome("F-0002", "direction_changed", &[]),
        ],
        "one look at surface-1",
    )
    .unwrap();
    assert_eq!(fx.state("F-0001", "F4").0, State::Pass);
    let (state, unmet) = fx.state("F-0002", "F4");
    assert_ne!(state, State::Pass, "mixed result: F-0002 did not confirm");
    assert!(
        unmet.iter().any(|u| u.contains("direction_changed")),
        "{unmet:?}"
    );
    // F-0003 shares the surface but was not named: nothing propagates.
    assert_ne!(fx.state("F-0003", "F4").0, State::Pass);

    // A receipt added later that claims the sealed session is not attested by it.
    fx.receipt("F-0003", "early_preview", "direction_confirmed", None);
    let path = fx.root.join(".codeos/06-workflow/decisions.jsonl");
    let text = fs::read_to_string(&path).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let mut last: serde_json::Value = serde_json::from_str(lines.last().unwrap()).unwrap();
    last["bindings"]["session"] = id.clone().into();
    *lines.last_mut().unwrap() = serde_json::to_string(&last).unwrap();
    fs::write(&path, lines.join("\n") + "\n").unwrap();
    let (state, unmet) = fx.state("F-0003", "F4");
    assert_ne!(state, State::Pass);
    assert!(unmet.iter().any(|u| u.contains("session")), "{unmet:?}");

    // A changed surface stales the outcome it judged.
    observe(&fx, "F-0001", "surface-2");
    assert_ne!(fx.state("F-0001", "F4").0, State::Pass);
}

#[test]
fn a_human_claim_stated_not_met_does_not_pass() {
    let fx = candidate("    provider: [backend]\n");
    gui_spec(&fx, "F-0001");
    claims_table(
        &fx,
        "F-0001",
        "| F-0001/readable | The page is readable. | F4 | observation:human |\n",
    );
    commit(&fx, "spec");
    observe(&fx, "F-0001", "surface-1");
    session::record(
        &fx.project(),
        vec![outcome(
            "F-0001",
            "direction_confirmed",
            &[("F-0001/readable", "not-met")],
        )],
        "seen",
    )
    .unwrap();
    let (state, unmet) = fx.state("F-0001", "F4");
    assert_ne!(state, State::Pass);
    assert!(
        unmet.iter().any(|u| u.contains("F-0001/readable")),
        "{unmet:?}"
    );
}

/// Criterion 5: under DBA-28 none of this applies.
#[test]
fn dba_28_projects_keep_their_behavior() {
    let fx = Fixture::under("DBA-28", "    provider: [backend]\n");
    fx.spec("F-0001", "mode: preserve\nreason: one", "");
    claims_table(
        &fx,
        "F-0001",
        "| F-0001/route | Shown. | F5 | run:behavior |\n",
    );
    let reports = codeos_workflow::checker::evaluate(&fx.project(), &fx.subject("F-0001")).unwrap();
    assert!(
        reports.iter().any(|r| r.id == "validation-claims"
            && r.unmet
                .iter()
                .any(|u| u.contains("requires Feature Workflow v13"))),
        "{reports:#?}"
    );
    let subject = fx.subject("F-0001");
    let bindings = codeos_workflow::checker::verification_bindings_for_mechanic(
        &fx.project(),
        &subject,
        "F5",
        "behavior",
    )
    .unwrap();
    assert!(!bindings.contains_key("execution_identity"));
}

/// The batched interaction: list every waiting question, then seal the stated outcomes.
#[test]
fn the_session_command_lists_questions_and_seals_stated_outcomes() {
    let fx = candidate("    provider: [backend]\n");
    for feature in ["F-0001", "F-0002"] {
        gui_spec(&fx, feature);
    }
    commit(&fx, "specs");
    for feature in ["F-0001", "F-0002"] {
        observe(&fx, feature, "surface-1");
    }
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_codeos-workflow"))
            .arg("--project")
            .arg(&fx.root)
            .args(args)
            .env_remove("TOMTOM_API_KEY")
            .env_remove("PROVIDER_MODE")
            .output()
            .unwrap()
    };
    let listed = run(&["session", "--subject", "F-0001", "--subject", "F-0002"]);
    let questions: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(questions.as_array().unwrap().len(), 2, "{questions}");
    assert_eq!(questions[0]["checkpoint"], "early_preview");
    assert_eq!(questions[0]["observed_surface"], "surface-1");
    let sealed = run(&[
        "session",
        "--outcome",
        "F-0001:early_preview:direction_confirmed",
        "--outcome",
        "F-0002:early_preview:direction_confirmed",
        "--rationale",
        "one look",
    ]);
    assert!(
        sealed.status.success(),
        "{}",
        String::from_utf8_lossy(&sealed.stderr)
    );
    assert_eq!(fx.state("F-0001", "F4").0, State::Pass);
    assert_eq!(fx.state("F-0002", "F4").0, State::Pass);
}

/// A successor ID removes the illegal identity reuse but inherits no evidence: a human claim's
/// successor needs its own session outcome.
#[test]
fn a_successor_human_claim_is_unproven_until_its_own_outcome() {
    let fx = candidate("    provider: [backend]\n");
    gui_spec(&fx, "F-0001");
    claims_table(
        &fx,
        "F-0001",
        "| F-0001/readable | The page is readable. | F4 | observation:human |\n",
    );
    commit(&fx, "spec");
    observe(&fx, "F-0001", "surface-1");
    session::record(
        &fx.project(),
        vec![outcome(
            "F-0001",
            "direction_confirmed",
            &[("F-0001/readable", "met")],
        )],
        "seen",
    )
    .unwrap();
    assert_eq!(fx.state("F-0001", "F4").0, State::Pass);

    let contract = fx
        .root
        .join(".codeos/01-specification/contracts/F-0001_contract.md");
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        text.replace(
            "| F-0001/readable | The page is readable. |",
            "| F-0001/readable-at-night | The page is readable in dark mode. |",
        ),
    )
    .unwrap();
    observe(&fx, "F-0001", "surface-1");
    let reports = codeos_workflow::checker::evaluate(&fx.project(), &fx.subject("F-0001")).unwrap();
    assert!(
        !reports.iter().any(|r| r.id == "validation-claims"),
        "the identity block is clear"
    );
    assert_ne!(
        fx.state("F-0001", "F4").0,
        State::Pass,
        "the predecessor's outcome does not prove the successor"
    );
    // A new session must state the successor; the predecessor's ID no longer counts.
    let error = session::record(
        &fx.project(),
        vec![outcome(
            "F-0001",
            "direction_confirmed",
            &[("F-0001/readable", "met")],
        )],
        "seen again",
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("F-0001/readable-at-night"),
        "{error}"
    );
    session::record(
        &fx.project(),
        vec![outcome(
            "F-0001",
            "direction_confirmed",
            &[("F-0001/readable-at-night", "met")],
        )],
        "seen again",
    )
    .unwrap();
    assert_eq!(fx.state("F-0001", "F4").0, State::Pass);
}

/// A `run:` successor is satisfied only by a record bound to the current Contract: the record that
/// proved the predecessor is stale once the Contract changes.
#[test]
fn a_successor_run_claim_needs_a_record_bound_to_the_current_contract() {
    let fx = candidate("    provider: [backend]\n");
    fx.spec("F-0001", "mode: preserve\nreason: one", "");
    claims_table(
        &fx,
        "F-0001",
        "| F-0001/route | The route is shown. | F5 | run:behavior |\n",
    );
    commit(&fx, "spec");
    fx.check("F-0001");
    fx.accept("F-0001", None);
    let contract = fx
        .root
        .join(".codeos/01-specification/contracts/F-0001_contract.md");
    let text = fs::read_to_string(&contract).unwrap();
    fs::write(
        &contract,
        text.replace(
            "| F-0001/route | The route is shown. |",
            "| F-0001/route-with-toll | The route and its toll are shown. |",
        ),
    )
    .unwrap();
    fx.seed("F-0001", "F3", "smoke");
    fx.seed("F-0001", "F3", "product_journey");
    let (state, unmet) = fx.state("F-0001", "F5");
    assert_ne!(
        state,
        State::Pass,
        "the predecessor's record does not prove the successor"
    );
    assert!(
        unmet.iter().any(|u| u.contains("'contract' changed")),
        "{unmet:?}"
    );
}

/// Found by the WO-07 JamFree replay: with stable topology, the journey a mid-chain feature builds
/// on is the accepted chain cut before its own step, so a later accepted advance never makes the
/// chain unreachable.
#[test]
fn a_mid_chain_feature_builds_on_the_chain_before_its_own_step() {
    let fx = candidate("    provider: [backend]\n");
    fx.write(
        "claim-scopes.json",
        r#"{"F-0001": {"route": ["provider"]}}"#,
    );
    fx.spec("F-0001", ADVANCE, ROUTE);
    commit(&fx, "F-0001");
    fx.check("F-0001");
    let evidence = fx.evidence("F-0001");
    fx.accept("F-0001", Some(&evidence));
    fx.spec(
        "F-0002",
        "mode: advance\nprevious_frontier: decided\nproposed_frontier: returned\ntransition: the driver returns",
        "",
    );
    commit(&fx, "F-0002");
    fx.check("F-0002");
    fx.accept("F-0002", None);
    let project = fx.project();
    assert_eq!(
        codeos_workflow::checker::current_accepted_frontier(&project).unwrap(),
        "returned"
    );
    for (feature, before) in [("F-0001", "entry"), ("F-0002", "decided")] {
        assert_eq!(
            codeos_workflow::checker::accepted_frontier_before(&project, &fx.subject(feature))
                .unwrap(),
            before
        );
    }
}
