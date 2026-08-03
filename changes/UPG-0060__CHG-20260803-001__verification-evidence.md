# UPG-0060 CHG-20260803-001 — Verification Evidence

<!--
PURPOSE: In-repo record of the Step 3 verification for the implementer harness correction,
so the change record's factual pass claims are auditable from the review packet rather than asserted.
This exists because Step 3 R2 finding 01 (and, before it, CHG-A's own R1-01) established that claiming
"N tests pass" without the run output in the packet leaves the acceptance criteria unverified.

Not a change record. The change record is
changes/UPG-0060__CHG-20260803-001__implementer-harness-correction.md.
-->

Reproduce with: `bash scripts/tests/codeos-implement-tests.sh` (no network, no API spend — a local
stub endpoint stands in for DeepSeek and echoes the run's own nonce).

## 1. Full suite run — verbatim

```
== Group 1: the five corrections ==
  ok   C2 manifest staged byte-identical
  ok   C6 source emitted verbatim (no JSON escape artifacts)
  ok   C16 audit set complete + tokens recorded
  ok   C13 secret absent from staging tree (checker passed positive control)
  ok   C13 secret absent from invocation log
  ok   C3 exemplar labeled distinctly from approved artifacts
  ok   C8 repair input rendered as feedback
  ok   C9 script invokes no build/test/package-manager command
  ok   C9 no external tool outside the documented allowlist
== Group 1: protocol robustness (criterion 7) ==
  ok   C7a wrong-nonce marker treated as content, file not truncated
  ok   C7b nested marker -> exit 11, nothing staged
  ok   C7c unterminated block -> exit 11, nothing staged
  ok   C7d duplicate path -> exit 11, nothing staged
  ok   C7e no file blocks -> exit 11
== Group 2: preserved CHG-A properties ==
  ok   C11 absolute path rejected (exit 8), nothing staged
  ok   C11 traversal path rejected (exit 8), nothing staged
  ok   C11 outside-stage-area path rejected (exit 8), nothing staged
  ok   C12 CANDIDATE_BLOCKED.md escape hatch works
  ok   C15 two runs -> two distinct staging dirs
  ok   C14 nothing written under modules/ or tests/, no commit
  ok   C10 non-git dir -> exit 1
  ok   C10 missing args -> exit 3
  ok   C10 stage=6 -> exit 3
  ok   C10 status:disabled -> exit 4
  ok   C10 status file absent -> exit 4
  ok   C10 malformed status -> exit 5
  ok   C10 unset key -> exit 6 (pre-network)
  ok   C10 missing artifact -> exit 7
== Group 2: new exit codes (criterion 18) ==
  ok   C18 missing exemplar -> exit 9
  ok   C18 missing repair input -> exit 10
  ok   C18 unknown option -> exit 3
  ok   C10 HTTP 500 -> exit 8
  ok   C17 self-dev activation file still status: disabled

33 passed, 0 failed
```

## 2. Mutation testing — verbatim

A suite that has never been shown to fail is not evidence. Four guards were deliberately broken, one
at a time, and the suite re-run. Each time the script was restored byte-identically (`diff -q` clean)
before the next mutation.

```
--- mutation: remove nested-marker guard ---
  FAIL C7b nested marker
32 passed, 1 failed
--- mutation: disable stage-area allowlist ---
  FAIL C11 outside-stage-area
32 passed, 1 failed
--- mutation: add undocumented external tool (find) ---
  FAIL C9 undocumented external tool
32 passed, 1 failed
--- mutation 4: neuter absolute/traversal rejection (breaks all-or-nothing) ---
  FAIL C11 traversal
32 passed, 1 failed
```

**Reading mutation 4.** Neutering the absolute/traversal rejection failed `C11 traversal` but not
`C11 absolute` — not a gap in the suite. An absolute path is independently rejected by the stage-area
guard (`/etc/passwd` does not start with `modules/`), so that case has two defenses and survives the
loss of one. The traversal case (`modules/../../escape.rs` *does* start with `modules/`) depends on
the traversal guard alone, which is exactly why removing it is detected there.

## 3. Criterion coverage

| Criterion | Covered by |
|---|---|
| 2 manifest survives end to end | `C2 manifest staged byte-identical` |
| 3 exemplar labeled distinctly | `C3 exemplar labeled distinctly from approved artifacts` |
| 6 model no longer authors JSON | `C6 source emitted verbatim (no JSON escape artifacts)` |
| 7 protocol not corruptible | `C7a`–`C7e` (wrong-nonce content, nested marker, unterminated, duplicate path, no file blocks) |
| 8 repair input labeled as feedback | `C8 repair input rendered as feedback` |
| 9 Option B — no build, documented process set | `C9 script invokes no build/test/package-manager command`, `C9 no external tool outside the documented allowlist` |
| 10 CHG-A fail-closed codes preserved | `C10` ×9 (exits 1, 3, 3, 4, 4, 5, 6, 7, 8) |
| 11 path safety, all-or-nothing | `C11` ×3 (absolute, traversal, outside stage area) |
| 12 CANDIDATE_BLOCKED.md escape hatch | `C12 escape hatch works` |
| 13 secret non-leakage | `C13` ×2, gated behind a positive control on the checker itself |
| 14 write-safety | `C14 nothing written under modules/ or tests/, no commit` |
| 15 idempotency | `C15 two runs -> two distinct staging dirs` |
| 16 auditability + token instrumentation | `C16 audit set complete + tokens recorded` |
| 17 still off by default | `C17 self-dev activation file still status: disabled` |
| 18 new exit codes | `C18` ×3 (exits 9, 10, 3) |

Criteria **1, 4, 5** (prompt wording) and **19, 20** (scope preservation) are verified by reading the
prompt and the diff, not by the suite — they are text properties, not runtime behavior.

## 4. Review coverage is structurally capped — and why that is accepted

Every review of this step reports `coverage: SECRET_REDACTION`, and the reviewer task template says
that under partial coverage it must **not** issue NO OBJECTION. So this step cannot reach NO
OBJECTION while the suite is in the packet. That is a structural limitation, not an unfixed defect.

**Cause, established by inspecting the packet rather than inferred.** The redactor keys on the
variable *name* `DEEPSEEK_API_KEY=`, not on the value's shape. Packet evidence: both redactions land
on `export DEEPSEEK_API_KEY=[REDACTED]` and on the intentional empty assignment
`DEEPSEEK_API_KEY=[REDACTED]="http://…"` in the exit-6 test. An earlier round changed the canary from
a credential-shaped string to an obviously-fake one on the theory that its shape was the trigger;
coverage did not change, which is what identified the real cause.

**Why it is not worked around.** A suite that tests a tool reading `DEEPSEEK_API_KEY` has to mention
`DEEPSEEK_API_KEY`. The name could be assembled indirectly to slip past the scanner, but deliberately
evading a secret detector is a worse practice to embed in this repository than accepting reduced
coverage on one file.

**What is actually hidden.** Two lines: the dummy canary assignment, and an intentional empty
assignment. No test logic, no assertion, and no verification output is redacted — §1 and §2 above are
complete. The reduced coverage is cosmetic with respect to the evidence this file exists to provide.

**Precedent.** UPG-0037 and UPG-0039 were both accepted at SECRET_REDACTION coverage with the same
benign trigger recorded (see `status/self-development.md`). The scope-triage category is
REJECTED / structural limitation, as used for UPG-0020 F3 and UPG-0035 AC-6.
