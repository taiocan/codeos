---
component_question: How should a downstream project migrate a supported legacy Codeos layout to the active layout?
out_of_scope: Current DBA behavior, unsupported historical layouts, compatibility aliases, and migration history already retained by Git.
---

# Downstream Upgrade

> This is non-authoritative migration guidance. Current semantics and canonical locations belong
> to `.codeos/toolkit/dba-system.md` and its selected components.

The active layout uses one real project-local `.codeos/` directory and one ignored
`.codeos/toolkit` symlink to the shared toolkit. Only the active layout is supported. Commit the
project before migration and stop whenever existing content has ambiguous ownership; do not merge,
overwrite, or relocate it by guesswork.

## Migrate the Toolkit Mount

For a project where `.codeos` is currently the shared-toolkit symlink:

1. Confirm `.codeos` is a symlink and that `.codeos/dba-system.md` and `.codeos/dba/00-entry/`
   resolve. Record the exact `readlink .codeos` result.
2. Confirm the Git worktree is clean. Remove only the validated `.codeos` symlink, create a real
   `.codeos/` directory, and recreate the recorded target as `.codeos/toolkit`.
3. Add `/.codeos/toolkit` and `/.codeos-state/` to the root `.gitignore`. Do not commit the
   machine-local symlink or transient operational state.
4. Confirm `.codeos/toolkit/dba-system.md` and `.codeos/toolkit/dba/00-entry/` resolve before
   continuing.

Do not run `dba-init.sh` against the legacy top-level symlink. The initializer stops and points to
this migration rather than replacing the mount automatically.

## Migrate Project Instructions

The canonical substantive project instructions move to `.codeos/00-project/CLAUDE.md`. Root
`CLAUDE.md` becomes the small adapter from
`.codeos/toolkit/dba/05-guidance/templates/project-root-CLAUDE.md`; root `AGENTS.md` remains the
adapter from `project-AGENTS.md`.

Review existing root files manually. If either contains project-owned instructions, preserve that
content in the canonical project document before replacing the root file. Do not use an automatic
merge. If ownership or intended meaning is unclear, stop and obtain a human decision.

## Convert Legacy Architecture Synthesis

Do this before moving durable architecture artifacts: the bounded converter deliberately accepts
only the committed legacy registry, Architecture Baseline, Cohort Logical Design, and Implementation
Profile at their legacy locations.

```bash
python3 .codeos/toolkit/dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py .
python3 .codeos/toolkit/dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py . --apply
```

The first command is a non-mutating preview. The second writes current Architecture Scope and
Implementation Profile artifacts under `.codeos/02-architecture/` and removes only the validated
legacy inputs. Resolve ambiguity or an existing target explicitly; never delete a target merely to
force migration through. Projects that never used those legacy artifacts skip this conversion.

## Migrate Durable Artifacts

Create destinations only for capabilities the project already uses, then move known Codeos-owned
artifacts with Git-aware moves:

| Legacy location | Canonical location |
|---|---|
| `intents/<feature-id>.md` | `.codeos/01-specification/intents/<feature-id>.md` |
| `contracts/<feature-id>_contract.md` | `.codeos/01-specification/contracts/<feature-id>_contract.md` |
| `events/<feature-id>_schema.md` | `.codeos/01-specification/event-schemas/<feature-id>_schema.md` |
| `architecture/scopes/<scope-id>.md` | `.codeos/02-architecture/scopes/<scope-id>.md` |
| `architecture/implementation-profile.yaml` | `.codeos/02-architecture/implementation-profile.yaml` |
| `architecture/proposals/implementation-profile-v<N>.yaml` | `.codeos/02-architecture/proposals/implementation-profile-v<N>.yaml` |
| `architecture/history/implementation-profile-v<N>.yaml` | `.codeos/02-architecture/history/implementation-profile-v<N>.yaml` |
| `architecture/delegated-implementation.yaml` | `.codeos/02-architecture/delegated-implementation.yaml` |
| `discovery/<topic>.md` | `.codeos/00-discovery/<topic>.md` |
| project refinement records | `.codeos/04-refinement/<feature-id>-<slug>.md` |
| `reviewer.toml` | `.codeos/05-review/reviewer.toml` |
| `reviews/` | `.codeos/05-review/reviews/` |
| review-process measurements | `.codeos/05-review/measurements/` |

Move only Event Schema files out of `events/`. Keep `events/runtime_events.jsonl` and any other
runtime-produced evidence in the project runtime area. Do not move project-native source, tests,
or unrelated architecture documentation merely because it shares an old parent directory.

Update paths recorded inside Intent, Contract, and Event Schema artifacts after moving them. Treat
the three artifacts as one Specification Package and verify their approval record and mutual
consistency after the path-only migration.

## Migrate Reviewer Configuration

The reviewer is Codex-only. In `.codeos/05-review/reviewer.toml`, remove any `provider` key and keep
only an optional `reasoning_effort` value. Remove `CODEOS_REVIEWER_PROVIDER` from project automation.
The retired `--provider`, `--mode`, `--print-packet`, `--dry-run`, and `stage-start` interfaces have
no compatibility aliases: use `plan` to inspect evidence, omit `--base` for full evidence, and pass
`--base <ref>` for delta evidence. Unknown configuration keys fail clearly so stale automation is
found during upgrade rather than silently preserved.

## Update References and Verify

Replace active toolkit references with `.codeos/toolkit/...`. In particular:

| Legacy reference | Active reference |
|---|---|
| `.codeos/dba-system.md` | `.codeos/toolkit/dba-system.md` |
| `.codeos/dba/<path>` | `.codeos/toolkit/dba/<path>` |
| `.codeos/prompts/<file>` | `.codeos/toolkit/dba/03-prompts/workflow/<file>` |
| `.codeos/scripts/<tool>` | the corresponding `.codeos/toolkit/dba/04-tools/` path |
| `.codeos/templates/<file>` | `.codeos/toolkit/dba/05-guidance/templates/<file>` |

Before resuming DBA work, verify:

```bash
test -d .codeos
test ! -L .codeos
test -L .codeos/toolkit
test -f .codeos/toolkit/dba-system.md
test -f .codeos/00-project/CLAUDE.md
test -d .codeos/01-specification/intents
test -d .codeos/01-specification/contracts
test -d .codeos/01-specification/event-schemas
rg -n '\.codeos/(dba-system\.md|dba/)' --glob '!downstream-upgrade.md' .
```

The final `rg` must return no active legacy references. Inspect the complete Git diff, run relevant
project tests, and run the Architecture Scope inspector when scopes exist. Do not add aliases,
fallback lookup, or dual-path detection. Exact execution of historical behavior requires checking
out the corresponding historical Codeos revision.
