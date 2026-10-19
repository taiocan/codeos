# REVIEW-BUDGET — Shared production evidence in Stage 8 packets

Status: Proposed 2026-10-19 (human decision: override now, fix later).

## Problem

A DBA-29 shared production run writes one reality-evidence file for every subject in the run (JamFree:
147 KB for F-0001..F-0005). The reviewer includes the whole file in each subject's packet, so
JamFree F-0001 (458 KB) and F-0002 (455 KB) exceeded the 450 KB packet budget on derived inputs
alone. Their revalidation reviews ran under a human-approved `CODEOS_PACKET_BUDGET_MODE=warn`.

## Proposed fix

Show a subject only its own claims and its own legs of shared production evidence, keeping the
whole file's digest bound in the packet. The budget itself is unchanged.
