# WO-11 — Ordinary Candidate Action After Architecture Change

ECP-5 repair · NORMAL · found during JamFree F-0001 work · Authority: Workflow Governance v4, WO-05

## Goal

For an accepted feature stale by `ARCHITECTURE_CHANGE_REQUIRED`, report the next step of the
ordinary Candidate Bundle already under way instead of a fixed instruction.

## Finding

Status replaced the checker's derivation with the fixed string "complete architecture impact and
use an ordinary Candidate Bundle". On JamFree, F-0001 already had a current ordinary candidate
assessed BLOCKED. The checker derived F7 `resolve_review_finding`, but status discarded it, so the
WO-06 answer to question 7 was wrong for F-0001.

## Rule

When a unique current ordinary Candidate Bundle exists, status reports the checker's F7 step:
`run_review` (AUTO) or `resolve_review_finding` (ASSISTED), naming the bundle. Otherwise
`reassess_architecture` (ASSISTED) remains.
