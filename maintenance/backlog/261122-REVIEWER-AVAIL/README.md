# REVIEWER-AVAIL — Reviewer Unavailability Diagnostics

Planning context only. Git remains authoritative.

When Codex runs out of quota, the reviewer reports a clearly named, typed unavailability instead of
a generic provider failure. Codex remains the only reviewer whose assessment can close a gate.

**Invariant:** Provider-failure classification may improve diagnostics; it must never select or
authorize another reviewer.

Out of scope: automatic Claude invocation, Claude closing gates, reviewer prompt changes, and any
new review policy, reviewer contract, or DBA configuration. A Claude reviewer that can close gates
would be a separate PROTECTED reviewer-authority redesign.

| WO | Class | Title | Status |
|---|---|---|---|
| [WO-01](WO-01-reviewer-unavailable-diagnostic.md) | NORMAL | Typed REVIEWER_UNAVAILABLE diagnostic for Codex quota exhaustion | Implemented 2026-11-22 — awaiting human acceptance |

This table is the only place WO status is kept.
