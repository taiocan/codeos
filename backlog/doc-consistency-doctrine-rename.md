# Backlog: doc-consistency pass after CLAUDE.md → dba-system.md split

**Status:** RESOLVED by change `0002-doc-consistency-rename` (2026-06-27). Renamed the 24
doctrine-attribution refs in `docs/codeos-manual.md` and 1 in `docs/oap-adoption-candidates.md`
to `dba-system.md`. The `oap-codeos-integration.md` and `reviewer-pipeline.md` references were
intentionally left as historical naming-collision / governance refs ("rename only where it
makes sense"). See `changes/0002-doc-consistency-rename.md`.

**Original status:** open (filed by change `0001-claude-split`, Reconcile step)
**Class (when worked):** documentation (normative) — 4-step self-dev loop
**Scope axis:** downstream doctrine only

## Problem

The CLAUDE.md split (change `0001-claude-split`) moved the master DBA doctrine from
`CLAUDE.md` to `dba-system.md`. The doctrine-**loading** paths were all repointed and
verified. But several *descriptive* docs still attribute the master doctrine (the 9-step
loop, the six non-negotiable rules, Truth Authority, DBA vocabulary, Review Logging) to a
file named `CLAUDE.md`. After the split, that content lives in `dba-system.md`.

## Stale references to update

- `docs/codeos-manual.md` — ~25 "Source basis: `CLAUDE.md`" / "master system instructions
  (`CLAUDE.md`)" attributions now point to the wrong filename. They should read
  `dba-system.md` (the toolkit's own self-development `CLAUDE.md` is a separate, smaller
  surface and only a few references genuinely mean it).
- `docs/oap-adoption-candidates.md` — one reference to "the existing CLAUDE.md stance".
- `docs/oap-codeos-integration.md` — several references treating `CLAUDE.md` as the
  constitution/doctrine ("Both claim `CLAUDE.md`", "do not touch `CLAUDE.md`…"). These are
  analysis/planning docs; lower priority, partly historical.

## Why it was not done in change 0001

Out-of-scope of the split itself: no loading path depends on these references, and a careful
manual rewrite (distinguishing doctrine refs → `dba-system.md` from genuine project/self-dev
`CLAUDE.md` refs) is a contained documentation pass better done as its own change than folded
into the split. Triaged at Reconcile as **OUT-OF-SCOPE BACKLOG**.

## Acceptance (when worked)

- Every `docs/*.md` reference that means "the master DBA doctrine" names `dba-system.md`.
- References that genuinely mean the project `CLAUDE.md` or the toolkit self-development
  `CLAUDE.md` are left correct and disambiguated.
- `grep -rn "CLAUDE\.md" docs/` reviewed line-by-line; no remaining misattribution.
