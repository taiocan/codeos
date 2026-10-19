//! An `events`-mode feature's runtime-event evidence through ordinary acceptance and revalidation
//! (found by WO-08 in JamFree F-0002): the record `verify-events` writes must be current for the
//! Candidate Bundle and for the revalidation case, under DBA-28 and under DBA-29 (v13 execution
//! identities).

mod fai_fixture;

use codeos_workflow::event_evidence::{verify, VerifyEventArgs};
use fai_fixture::*;

const VALIDATOR: &str = r#"{
  total: length,
  invalid_events: [.[] | select(.event_type != "RouteShown")],
  privacy_check: {prohibited_payload_keys_found: [.[] | .payload | keys[] | select(. == "secret")]}
}"#;

fn events_mode(fx: &Fixture) {
    let path = fx
        .root
        .join(".codeos/01-specification/event-schemas/F-0001_schema.md");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace("**Mode:** `external-observation`", "**Mode:** `events`"),
    )
    .unwrap();
    fx.write("events/validate.jq", VALIDATOR);
    fx.write("events/runtime_events.jsonl", "");
}

fn record_events(fx: &Fixture) {
    let command: Vec<String> = [
        "bash",
        "-c",
        r#"printf '{"event_id":"%s","event_type":"RouteShown","timestamp":1,"correlation_id":"run-%s","source_module":"route","payload":{}}\n' "$RANDOM" "$RANDOM" >> events/runtime_events.jsonl"#,
    ]
    .map(String::from)
    .to_vec();
    verify(
        &fx.project(),
        VerifyEventArgs {
            subject: &fx.subject("F-0001"),
            event_log: "events/runtime_events.jsonl",
            validator: "events/validate.jq",
            command: &command,
        },
    )
    .unwrap();
}

#[test]
fn runtime_event_evidence_is_current_for_the_candidate_and_the_revalidation() {
    for configuration in ["DBA-28", "DBA-29"] {
        let fx = Fixture::under(configuration, "    provider: [backend]\n");
        fx.write(
            "claim-scopes.json",
            r#"{"F-0001": {"route": ["provider"]}}"#,
        );
        fx.spec("F-0001", ADVANCE, ROUTE);
        events_mode(&fx);
        fx.git(&["add", "-A"]);
        fx.git(&["commit", "-qm", "spec"]);
        fx.check("F-0001");
        record_events(&fx);
        let evidence = fx.evidence("F-0001");
        // Ordinary acceptance prepares a Candidate Bundle, which requires current runtime events.
        fx.accept("F-0001", Some(&evidence));

        std::thread::sleep(std::time::Duration::from_millis(1100));
        fx.write("backend/Cargo.lock", "# rustls 0.23.46\n");
        fx.git(&["add", "-A"]);
        fx.git(&["commit", "-qm", "trigger"]);
        run_checks_revalidation(&fx);
        record_events(&fx);
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
