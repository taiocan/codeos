# Self-Development Change: UPG-0060__CHG-20260803-003 — jq-arg-size-limit

<!--
PURPOSE: Narrow bugfix to a shipped CHG-A defect in scripts/codeos-implement.sh. Not part of the
DeepSeek adoption verdict — the tool was unable to build a request at realistic packet size, which is
a defect regardless of whether delegation is ever adopted. Human-directed (2026-08-03): "Fix the
confirmed jq --arg issue narrowly. Treat it as a real CHG-A defect, not part of the DeepSeek verdict."
-->

```yaml
feature_id: UPG-0060
primary_feature_id: UPG-0060
change_id: CHG-20260803-003
slug: jq-arg-size-limit
state: COMPLETE
current_step: 4-Reconcile
implements: [UPG-0060]
related_features: []
review_series: RVS__UPG-0060__CHG-20260803-003__S4
review_profile: PROFILE-1   # single-line fail-closed bugfix with a mutation-verified regression test
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: CHG-20260802-001
```

---

## Change Intent

**Why.** `scripts/codeos-implement.sh` built its request body with `jq --arg usr "${USR}"`, passing
the entire packet as a single argv element. Linux caps one argument at 128 KiB (`MAX_ARG_STRLEN`)
independently of the much larger total `ARG_MAX`, so any packet above that size aborted with
`/usr/bin/jq: Argument list too long` and exit 126 — before any network call, with no candidate
produced.

Latent since CHG-A. The first pilot packet was 101 KB, under the limit. Adding a layout exemplar and
a repair input took EA-0003's packet to 133,523 bytes and tripped it. **It was found by running the
tool on a realistic downstream feature, not by review** — six review rounds over the harness never
touched it, because they examined claims about the tool rather than its behavior at realistic scale.

This affects the single-shot path too: any project whose approved artifacts exceed ~128 KB could
never have used the tool.

**What changes:**
- `scripts/codeos-implement.sh` — `--arg sys/usr` becomes `--rawfile sys/usr`, reading the task
  prompt and the already-written `user_content.txt` from disk. Packet size becomes bounded by memory
  rather than by an exec limit. No behavior change other than removing the ceiling.
- `scripts/tests/codeos-implement-tests.sh` — one regression test asserting a packet above 128 KiB
  builds and runs.

**Scope boundary.** No change to the output protocol, exit codes, staging, fail-closed ordering,
activation, or the prompt. Nothing about the DeepSeek adoption question is touched. The mechanism
stays `status: disabled`.

**Class:** script-tooling · **Scope axis:** self-dev only

---

## Acceptance Criteria

| # | Criterion | Verification |
|---|---|---|
| 1 | A packet larger than 128 KiB builds a request and completes a run | New test `REG packet of N bytes (>128 KiB) builds and runs` |
| 2 | The regression test genuinely detects the old bug | Mutation: revert to `--arg` and confirm the test fails |
| 3 | No existing behavior regresses | Full suite still passes |
| 4 | Nothing outside the two named files changes | `git diff --stat` |

---

## Implementation Notes

`--rawfile` reads a file into a jq variable without passing it through argv. Both inputs already
existed as files (`${TASK_PROMPT}` and `${STAGE_DIR}/user_content.txt`), so the fix reads what was
already on disk. A minor incidental improvement: `--rawfile` preserves trailing newlines, which
`$(cat …)` strips.

**Verification — 34 tests pass** (33 prior + 1 regression). The regression test was mutation-verified:
reverting the line to `--arg` fails exactly `REG oversized packet` (33 passed, 1 failed) and nothing
else; restoring returns to 34/0.

**Assumption:** the 128 KiB single-argument cap is a Linux constant, so the test asserts the
behavior (a large packet works) rather than the mechanism, and would remain meaningful on a platform
with different limits.

---

## Reconciliation

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | >128 KiB packet builds and runs | PASS | `REG packet of 288001 bytes (>128 KiB) builds and runs` |
| 2 | Test detects the old bug | PASS | Mutation to `--arg` → `FAIL REG oversized packet`, 33/1 |
| 3 | No regression | PASS | 34 passed, 0 failed |
| 4 | Scope | PASS | Only `scripts/codeos-implement.sh` and `scripts/tests/codeos-implement-tests.sh` |

**Review.** Run at PROFILE-1: a one-line fail-closed bugfix whose correctness is demonstrated by a
mutation-verified regression test, executed under a standing human instruction to fix it narrowly and
not spend further review rounds on harness governance. Recorded as a deliberate profile choice, not
an omission — the human may direct a full round if they prefer.

**Findings scope-triage:** none raised.

**Note for UPG-0062.** This defect is why that feature's Step 1 packet initially showed scope
contamination: the fix sat uncommitted in the working tree across a feature boundary. The lesson is
AJ-017's, already journaled — commit completed work before opening the next change.
