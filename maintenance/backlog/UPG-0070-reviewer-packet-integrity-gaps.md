---
feature_id: UPG-0070
slug: reviewer-packet-integrity-gaps
title: Two Reviewer Packet-Integrity Gaps Found by the UPG-0069 Codex Arm
status: PROPOSED
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
