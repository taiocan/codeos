//! DQA WO-02 cross-consumer contract: Stage 8 review readiness reports the workflow's lifecycle
//! verdicts on the same project, so the two cannot disagree on record currency, F6 currency or the
//! selected production evidence. Covers a pre-bundle acceptance, DBA-27 (Feature Workflow v11),
//! DBA-28 (v12), an accepted-but-stale feature and a revalidated-current one.

mod common;
#[path = "../../../workflow/engine/tests/fai_fixture/mod.rs"]
mod fai_fixture;

use fai_fixture::*;

/// The readiness issues `plan` reports for F-0001 at Stage 8, as `REASON: subject: detail` lines.
fn readiness_issues(fx: &Fixture) -> Vec<String> {
    fx.git(&["add", "-A"]);
    fx.git(&["commit", "-qm", "state", "--allow-empty"]);
    let fake = common::setup_fake_codex();
    let (_, stdout, stderr) = common::run_with_fake_codex(
        &fx.root,
        &fake,
        &[
            "plan",
            "F-0001",
            "8",
            ".codeos/01-specification/intents/F-0001.md",
        ],
        "success",
    );
    assert!(stdout.contains("review_readiness:"), "{stdout}\n{stderr}");
    stdout
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("- "))
        .map(str::to_string)
        .collect()
}

fn evidence_issues(issues: &[String]) -> Vec<&String> {
    issues
        .iter()
        .filter(|issue| {
            issue.starts_with("EVIDENCE_STALE")
                || issue.starts_with("F6_SELECTED_EVIDENCE_MISMATCH")
        })
        .collect()
}

#[test]
fn a_revalidated_feature_is_current_for_the_workflow_and_the_reviewer() {
    for (configuration, legacy) in [("DBA-27", false), ("DBA-28", false), ("DBA-28", true)] {
        let (fx, _, revalidated) = revalidated_upstream(configuration, legacy);
        assert_eq!(fx.evidence_current("F-0001"), Ok(()));
        let issues = readiness_issues(&fx);
        assert!(
            evidence_issues(&issues).is_empty(),
            "{configuration} legacy={legacy}: {issues:#?}"
        );
        assert_eq!(
            codeos_workflow::checker::validate_production_evidence_selection(
                &fx.project(),
                &fx.subject("F-0001"),
                &revalidated,
            )
            .map_err(|error| error.to_string()),
            Ok(())
        );
    }
}

#[test]
fn an_accepted_feature_gone_stale_is_stale_for_the_workflow_and_the_reviewer() {
    let (fx, _, _) = revalidated_upstream("DBA-28", false);
    fx.write("scripts/verify.sh", &format!("{VERIFIER}# v3\n"));
    assert_eq!(
        fx.evidence_current("F-0001"),
        Err("reality_verifier".into())
    );
    let issues = readiness_issues(&fx);
    assert!(
        issues.iter().any(|issue| issue
            == "EVIDENCE_STALE: F5/production_source_evidence: verification is bound to a different reality_verifier"),
        "{issues:#?}"
    );
}

/// An ordinary DBA-28 candidate: F6 binds the delivered state, so a test-runner configuration
/// change leaves it current for both consumers and a delivered build configuration change stales
/// it for both.
#[test]
fn f6_currency_is_the_same_for_the_workflow_and_the_reviewer() {
    let fx = routed_under(
        "DBA-28",
        "    provider: [backend]\n    web: [web]\n",
        r#"{"F-0001": {"route": ["provider", "web"]}}"#,
    );
    let evidence = fx.evidence("F-0001");
    fx.receipt("F-0001", "reconciliation", "completed", Some(&evidence));
    fx.seed("F-0001", "candidate", "project_qualification");
    codeos_workflow::candidate::prepare(
        &fx.project(),
        &fx.subject("F-0001"),
        &[],
        "ordinary",
        None,
    )
    .unwrap();
    let f6_stale = |issues: &[String]| {
        issues
            .iter()
            .any(|issue| issue.starts_with("EVIDENCE_STALE: F6:"))
    };

    fx.write(
        "web/playwright.config.ts",
        "export default { workers: 1 };\n",
    );
    assert!(fx.f6_current("F-0001"));
    let issues = readiness_issues(&fx);
    assert!(!f6_stale(&issues), "{issues:#?}");

    fx.write("web/vite.config.ts", "export default {};\n");
    assert!(!fx.f6_current("F-0001"));
    let issues = readiness_issues(&fx);
    assert!(f6_stale(&issues), "{issues:#?}");
}

/// A record binds more than the implementation: the continuous product journey also binds the
/// Charter, so changing it stales that record for both consumers.
#[test]
fn every_input_a_record_binds_is_judged_by_both_consumers() {
    let (fx, _, _) = revalidated_upstream("DBA-28", false);
    let charter = fx.root.join(".codeos/00-project/charter.md");
    let text = std::fs::read_to_string(&charter).unwrap();
    fx.write(
        ".codeos/00-project/charter.md",
        &format!("{text}\nA later note.\n"),
    );
    let project = fx.project();
    let subject = fx.subject("F-0001");
    let record = codeos_workflow::verification::VerificationStore::at(&project.codeos_dir())
        .latest("feature", "F-0001", "F3", "product_journey")
        .unwrap();
    let want = codeos_workflow::checker::verification_bindings_for_mechanic(
        &project,
        &subject,
        "F3",
        "product_journey",
    )
    .unwrap();
    assert!(matches!(
        codeos_workflow::verification::evaluate(record, &want),
        codeos_workflow::verification::RecordState::Stale { binding, .. } if binding == "charter"
    ));
    let issues = readiness_issues(&fx);
    assert!(
        issues.iter().any(|issue| issue
            == "EVIDENCE_STALE: F3/product_journey: verification is bound to a different charter"),
        "{issues:#?}"
    );
}
