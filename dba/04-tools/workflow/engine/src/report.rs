//! Rendering `status` and `next` output. Always concrete: PASS items, MISSING items with why each
//! applies, and one next action — never a bare pass/fail.

use crate::checker::{CheckpointReport, State};

pub fn render_status(workflow: &str, subject: &str, reports: &[CheckpointReport]) -> String {
    let mut out = format!("workflow: {workflow}   subject: {subject}\n");
    for r in reports {
        out.push_str(&format!("\n{} — {}: {}\n", r.id, r.title, r.state.label()));
        for m in &r.met {
            out.push_str(&format!("  ✓ {m}\n"));
        }
        for u in &r.unmet {
            out.push_str(&format!("  ✗ {u}\n"));
        }
    }
    if let Some(n) = next_report(reports) {
        out.push_str(&format!(
            "\nNEXT\n  {} — {}\n  {}\n",
            n.id,
            n.title,
            n.next_action
                .clone()
                .unwrap_or_else(|| "resolve the unmet condition above".into())
        ));
    } else {
        out.push_str("\nNEXT\n  nothing blocked — every applicable checkpoint is PASS\n");
    }
    out
}

pub fn render_next(reports: &[CheckpointReport]) -> String {
    match next_report(reports) {
        Some(n) => format!(
            "{} — {}: {}\n{}",
            n.id,
            n.title,
            n.state.label(),
            n.next_action
                .clone()
                .unwrap_or_else(|| "resolve the unmet condition".into())
        ),
        None => "nothing blocked — every applicable checkpoint is PASS".to_string(),
    }
}

/// The first checkpoint that is not PASS / not-applicable.
fn next_report(reports: &[CheckpointReport]) -> Option<&CheckpointReport> {
    reports
        .iter()
        .find(|r| matches!(r.state, State::Blocked | State::Waiting))
}

/// Exit code for `status`/`check`: 0 if all applicable checkpoints PASS, 2 if any is blocked/waiting.
pub fn overall_exit(reports: &[CheckpointReport]) -> i32 {
    if next_report(reports).is_some() {
        crate::EXIT_BLOCKED
    } else {
        crate::EXIT_SUCCESS
    }
}
