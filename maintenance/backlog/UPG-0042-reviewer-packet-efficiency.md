---
feature_id: UPG-0042
slug: reviewer-packet-efficiency
title: Reduce Reviewer Packet Bloat for Large Stable Files
status: PROPOSED
priority: P2
depends_on: [UPG-0027, UPG-0032]
related_features: [UPG-0043]
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-packet-efficiency — Reduce Reviewer Packet Bloat for Large Stable Files

**Priority**: P2
**Status**: PROPOSED
**Type**: script-tooling

## Problem

Reviewer packets repeatedly include large stable files in full, even when only small portions changed or when the file is structurally unchanged. Discovered during UPG-0041 (CHG-20260707-005), where every review round included the full 121 KB `smoke.rs` file, causing:

- **Packet bloat**: 192-202 KB packets vs. 50 KB budget (4× overage)
- **Token waste**: ~50-60k tokens per round × 8 rounds = 400k+ potential tokens
- **Review noise**: Reviewer must process full stable file content instead of targeted evidence
- **Context pressure**: Large packets consume context that could be used for deeper analysis
- **Human inspectability**: 200KB packet files are hard to inspect and diff

The issue is not cache efficiency (97% hit rate helped) but **packet evidence design**: large stable files are included as full review evidence when a targeted diff, test list, or hash-pinned reference would often suffice.

Cache hit rate does not solve review packet quality, context pressure, or human inspectability.

## Current Behavior

When `codeos-review.sh` builds a packet:
- All specified artifact paths are included in full
- No distinction between "changed file" vs "stable context file"
- No size-based policy or warnings
- No delta mode for subsequent review rounds of the same artifact

Example from UPG-0041 Step 3 R1:
```
smoke.rs: 121588 bytes (unchanged from Step 1)
Total packet: 202298 bytes
Warning: exceeds budget of 50000 bytes
```

All 8 review rounds included the full `smoke.rs` even though:
- Step 1-2: file not yet modified
- Step 3-4: only new tests added (could show diff + test list)

## Upgrade

Not decided by this brief — questions for implementer to resolve deliberately:

### 1. Packet-Size Policy

Should the packet builder:
- Refuse to include files exceeding a threshold (e.g., 50KB) without explicit override?
- Auto-switch to delta mode for large files after R1?
- Emit warnings when a single file dominates packet size (>40% of total)?
- Support a "snippet mode" that includes only changed regions + N lines of context?

### 2. Evidence Modes

Should artifacts support multiple evidence modes:
- **full**: entire file (current default)
- **delta**: git diff only
- **snippet**: changed regions + context
- **hash-only**: SHA256 + "unchanged since baseline"
- **summary**: test count, line count, key metrics (for test files)

When would each mode be appropriate?

### 3. Large Stable File Detection

Should the reviewer:
- Track which files are "stable" (unchanged across rounds)?
- Auto-downgrade stable files from `full` to `hash-only` after R1?
- Require explicit justification to include large stable files in full?

### 4. Test File Handling

For test files specifically (smoke.rs, integration tests):
- Show: test names added/removed/changed
- Show: relevant test snippets (not full file)
- Show: `cargo test --list | wc -l` output
- Show: git diff --stat + targeted snippets
- Hash-only for unchanged sections

### 5. Reviewer Packet Evidence Mode

Should there be a structured evidence format:
```yaml
artifacts:
  - path: smoke.rs
    mode: delta
    summary:
      tests_added: 9
      tests_changed: 15
      tests_removed: 0
      total_tests: 119
    diff: (targeted diff)
    hash_unchanged_sections: sha256:...
```

### 6. Delta Mode After R1

Should subsequent review rounds of the same step automatically use delta mode unless explicitly overridden?

## Scope

`dba/04-tools/reviewer/codeos-review.sh`, packet builder logic, evidence assembly, and potentially:
- `dba/04-tools/reviewer/engine/src/packet.rs` if evidence modes need structured representation
- Review assessment prompt to handle different evidence modes
- Documentation of when to use which evidence mode

May require UPG-0043 (split monolithic smoke.rs) as a complementary fix if smoke.rs keeps growing.

## Value

Medium-high. Without this:
- Every downstream-doctrine change with test modifications will be expensive
- Review packets will continue exceeding budget
- Reviewers will waste context on stable file content
- Human inspection of review artifacts will be painful

Benefits:
- Smaller, more focused packets (50KB target vs. 200KB actual)
- Better token efficiency (50-150k token savings per multi-round change)
- Clearer review evidence (show what changed, not what didn't)
- Better human inspectability of review artifacts
- Less context pressure = room for deeper analysis

## Risk

Deciding hastily risks:
- Over-compression: losing necessary context for review
- Mode confusion: when to use which evidence mode
- Baseline drift: hash-only references becoming stale
- Breaking existing review workflows

Do not implement without:
1. Clear policy on when each evidence mode is appropriate
2. Explicit override mechanism for "full file needed"
3. Backward compatibility for existing packet consumers
4. Verification that delta mode provides sufficient review evidence

## Guardrail

**No silent evidence truncation.** If a large file is excluded or summarized, the packet must clearly state:
- What was excluded
- Why (size, stability, or explicit policy)
- Where to find the full content if needed

The reviewer must know what evidence is missing, not discover it mid-assessment.

**Evidence reduction must be safe-by-default:** Changed files or changed regions remain reviewable as content or diff. Hash-only is allowed only for:
- Unchanged large files (stable across rounds)
- Explicitly out-of-scope generated content
- Files whose substantive content is not under review in the current CHG

Changed behavior must never be hidden behind hash-only or excessive summarization. When in doubt, include the diff.

## Related

- **UPG-0043**: Split monolithic smoke.rs (complementary fix)
- **UPG-0027**: Lean review runner (established packet architecture)
- **UPG-0032**: Rust reviewer engine (packet consumer)

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| (none yet) | — | — | PROPOSED |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
