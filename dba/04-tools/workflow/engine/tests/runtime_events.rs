//! An `events`-mode feature's runtime-event evidence through ordinary acceptance and revalidation
//! (found by WO-08 in JamFree F-0002): the record `verify-events` writes must be current for the
//! Candidate Bundle and for the revalidation case, under DBA-28 and under DBA-29 (v13 execution
//! identities).

mod fai_fixture;

use fai_fixture::*;

#[test]
fn runtime_event_evidence_is_current_for_the_candidate_and_the_revalidation() {
    for configuration in ["DBA-28", "DBA-29"] {
        let fx = Fixture::under(configuration, "    provider: [backend]\n");
        fx.write(
            "claim-scopes.json",
            r#"{"F-0001": {"route": ["provider"]}}"#,
        );
        fx.spec("F-0001", ADVANCE, ROUTE);
        fx.events_mode("F-0001");
        fx.git(&["add", "-A"]);
        fx.git(&["commit", "-qm", "spec"]);
        fx.check("F-0001");
        fx.record_events("F-0001");
        let evidence = fx.evidence("F-0001");
        // Ordinary acceptance prepares a Candidate Bundle, which requires current runtime events.
        fx.accept("F-0001", Some(&evidence));

        std::thread::sleep(std::time::Duration::from_millis(1100));
        fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
        fx.git(&["add", "-A"]);
        fx.git(&["commit", "-qm", "trigger"]);
        run_checks_revalidation(&fx);
        fx.record_events("F-0001");
        fx.seed("F-0001", "candidate", "project_qualification");
        let case = codeos_workflow::revalidation::build_case(&fx.project(), "F-0001", "HEAD");
        assert!(case.is_ok(), "{configuration}: {:?}", case.err());
        codeos_workflow::candidate::prepare(
            &fx.project(),
            &fx.subject("F-0001"),
            &[],
            "post_acceptance_revalidation",
            Some("HEAD"),
        )
        .unwrap_or_else(|error| panic!("{configuration}: {error}"));
    }
}
