---
feature_id: UPG-0070
slug: reviewer-packet-integrity-gaps
title: Two Reviewer Packet-Integrity Gaps Found by the UPG-0069 Codex Arm
status: DONE — both gaps repaired 2026-08-27; reproduction tests carry the guarantee
priority: P2
depends_on: []
related_features: [UPG-0069]
supersedes: []
superseded_by: []
---

# Upgrade: two reviewer packet-integrity gaps

## Why this exists

UPG-0069's Codex arm reviewed the external-assessment change and reported two defects the DeepSeek
arm missed. Both were verified against the reviewed packet bytes and **both are still live at HEAD**.
UPG-0069 reserved exactly this case — "no further engineering unless a new integrity defect is
found" — so they are recorded here rather than repaired inside a finished experiment.

Neither is DeepSeek-related. They are reviewer-integrity defects in the Rust engine.

## The two gaps

**1. An imported packet is not bound to its sidecar.** `load_exported_packet`
(`dba/04-tools/reviewer/engine/src/cmd/review.rs`) checks feature, stage, and artifact set against
the sidecar and never hashes the packet content. Altered or emptied packet bytes can therefore be
imported under a sidecar claiming valid coverage and recorded as reviewed evidence. The
`EMPTY_PACKET` guard does not catch it either: it reads `coverage_state` from that same sidecar
rather than from the content.

**2. Untracked-file discovery fails open.** `git_untracked_files`
(`dba/04-tools/reviewer/engine/src/packet.rs`) ends in `.output().map(…).unwrap_or_default()`,
ignoring both a spawn failure and a non-zero Git exit. A Git failure yields an empty list, so the
packet can report full coverage while omitting untracked files — the same class of gap the
untracked-file work existed to close.

## Shape of a fix

Record the packet's own hash in the sidecar and verify it on import; make untracked-file discovery
fail closed by downgrading coverage when Git cannot be consulted. Both are small, local, and testable
against the existing engine test suite. Neither changes reviewer authority, so this is NORMAL work.

Deciding whether to do it, and when, is the human's call — this brief only ensures the finding is not
lost with the experiment that produced it.

## Repaired 2026-08-27

**Gap 1 — corrected framing.** This brief said the import path "never hashes the packet content",
which was imprecise: the engine already hashed the packet into `reviewed_packet_sha256`, and
`log.rs` already compared a stored record against the packet on disk. The actual defect was narrower
— *the sidecar made coverage claims about bytes it was not bound to*, so pairing sidecar A with
packet bytes B produced a record that faithfully hashed B while inheriting A's `coverage_state`,
`artifacts`, and `redaction_count`.

`ReviewPacket` now carries `content_sha256`, written at build time and verified in
`load_exported_packet`. **Compatibility was the real decision here**, because the frozen UPG-0069
sidecar predates the field and is committed evidence: an absent hash warns that integrity is
unverifiable and proceeds, a present-but-mismatched hash fails closed. That mirrors the pattern
`log.rs` already used, keeps every frozen sidecar importable, and closes the gap for every packet
written from now on.

**Gap 2.** `git_untracked_files` returns `Option<Vec<String>>` and yields `None` on a spawn failure
*or* a non-zero exit — the original ignored the exit status entirely, so a failing Git read as an
empty working tree. An unavailable answer records a visible exclusion and forces `CRITICAL_OMISSION`.
Unknown coverage is treated as at least as severe as a known omission, because a packet cannot
enumerate the files it failed to discover. `CRITICAL_OMISSION` warns the reviewer and blocks
NO OBJECTION on that basis without hard-gating the run.

**Both tests were positive-controlled against the unfixed code**, which mattered: the first
untracked-file test passed against the original defect because a blanket-failing `git` shim broke
branch and diff resolution too, so the packet failed for an unrelated reason. The test now shims only
`ls-files --others` and passes everything else through to the real binary, and it fails against the
original `unwrap_or_default()`.

The two defects were reported by exactly one of four independent review arms. Codex found both;
DeepSeek V4-Pro and Gemini 3.7 Flash each listed both among their *satisfied* claims (UPG-0071,
UPG-0072). Integrity that only one reviewer detects belongs in a test, not in review.
