---
change_id: CHG-20260707-002
feature_id: UPG-0038
slug: review-shim-symlink-resolution
triage_class: script-tooling
scope_axis: self-dev only
review_profile: PROFILE-3
review_series: RVS__UPG-0038__CHG-20260707-002__S1
review_state: ACCEPTED
status: COMPLETE
loop_step: 4-Reconcile
---

# Change: UPG-0038 / CHG-20260707-002 — Fix codeos-review.sh Binary Resolution for Symlinked Downstream Projects

## TRACE HEADER

```yaml
feature_id: UPG-0038
primary_feature_id: UPG-0038
change_id: CHG-20260707-002
slug: review-shim-symlink-resolution
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0038
related_features:
  - UPG-0032
  - UPG-0037
review_series: RVS__UPG-0038__CHG-20260707-002__S1
review_profile: PROFILE-3
review_state: ACCEPTED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

---

## Step 1 — Change Intent

### Problem

`scripts/codeos-review.sh` resolves `REPO_ROOT` via `git rev-parse --show-toplevel` (line 6),
then looks for the reviewer binary at `${REPO_ROOT}/tools/reviewer/target/release/
codeos-reviewer`. When invoked from within a downstream project through the `.codeos`
symlink (e.g. `.codeos/scripts/codeos-review.sh` run from `/home/rimo/projects/FundFlow`),
`git rev-parse --show-toplevel` resolves to the *calling* project's own git root (FundFlow's),
not through the symlink to Codeos — so the shim looks for a binary at
`FundFlow/tools/reviewer/target/release/codeos-reviewer`, which doesn't exist, and fails.
Direct invocation of the compiled binary works correctly against real FundFlow artifacts
(confirmed during UPG-0037's Step 3 verification); only the shim wrapper is broken.

### What changes

| File | Change |
|---|---|
| `scripts/codeos-review.sh` | Resolve the binary path from the script's own physical location (following the `.codeos` symlink) instead of the calling repo's git root |
| `docs/reviewer-pipeline.md` | Update §12's "Known limitation" paragraph (added by UPG-0037, documenting this exact bug) now that the fix lands — added at Step 4 during the cross-reference sweep |
| `backlog/UPG-0038-review-shim-symlink-resolution.md` | Feature Thread: CHG-20260707-002 activated (done) |
| `backlog/features.md` | Row → IN_PROGRESS (done) |
| `status/self-development.md` | Row activated (done) |
| `status/roadmap.md` | UPG-0038 → IN_PROGRESS (done) |

### Scope boundary — what stays the same

- **No change to the Rust binary itself** — `tools/reviewer/src/` is untouched; this is a
  pure shell-script fix, matching the backlog's own Scope statement.
- **No change to binary discovery precedence beyond path resolution** — the `command -v
  codeos-reviewer` PATH fallback (for a globally-installed binary) is preserved unchanged;
  `--provider` CLI flag and `CODEOS_REVIEWER_PROVIDER` env var precedence (both handled
  entirely inside the Rust binary, never touched by the shim) are unaffected.
- **The exec'd binary's own repo-root discovery is untouched.** The Rust binary
  independently calls its own `discover_repo_root()` (via `git rev-parse --show-toplevel`
  from whatever directory the *user* invoked the shim from) to find the project being
  reviewed (e.g. FundFlow's artifacts/diff) — that is a completely separate concern from
  *locating the binary file*, and this change does not touch it. Only where the shim looks
  for the compiled executable changes; where the executed binary looks for the project being
  reviewed does not.

### Design intent

Replace the `git rev-parse --show-toplevel`-based `REPO_ROOT` with a `SCRIPT_DIR`-based
resolution using the script's own physical location, following symlinks:

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd -P)"
BINARY="${CODEOS_ROOT}/tools/reviewer/target/release/codeos-reviewer"
```

`pwd -P` (physical, not logical) is the key detail: when invoked as `.codeos/scripts/
codeos-review.sh` from FundFlow's root, `${BASH_SOURCE[0]}` is exactly that unresolved,
symlink-relative string; plain `cd .codeos/scripts && pwd` would print the *logical* path
`FundFlow/.codeos/scripts` (preserving the symlink name, the same bug in different clothing)
— `pwd -P` instead resolves through the symlink to the real path,
`/path/to/Codeos/scripts`, so `CODEOS_ROOT` correctly becomes Codeos's own root regardless of
which project's directory tree the shim was invoked through.

**Single unified resolution, no special-cased fallback branch — a deliberate simplification
from the backlog's literal wording, flagged for approval.** The backlog text suggests
"falling back to the current `git rev-parse` behavior only for Codeos's own self-development
use." That fallback turns out to be unnecessary: for Codeos's own repo (no symlink involved),
`SCRIPT_DIR`'s parent directory *is* Codeos's root — physically identical to what `git
rev-parse --show-toplevel` would have returned anyway, since there's no symlink to resolve
through in that case. One code path serves both usage modes correctly; a second branch would
be unneeded complexity. The backlog's "Risk" section calls for "a compatibility check against
both usage modes" — read as a *verification* requirement (test both scenarios explicitly),
not a requirement for two implementation branches; Step 4 verifies both directly against a
real Codeos invocation and a real FundFlow `.codeos`-symlinked invocation.

The `command -v codeos-reviewer` PATH-fallback branch and the "binary not found" error
message (updated to reference `${CODEOS_ROOT}` instead of `${REPO_ROOT}` for the suggested
build command) are otherwise unchanged in structure.

### Triage

- Class: `script-tooling`
- Scope axis: `self-dev only`
- Review profile: `PROFILE-3`
- Originating backlog id: `UPG-0038`

---

## Step 2 — Acceptance Criteria

### Functional resolution

**AC-1 — Correct resolution: Codeos self-development usage**
Invoked from within Codeos's own repo (any subdirectory, any relative/absolute path to the
script), the shim resolves `BINARY` to `<Codeos-root>/tools/reviewer/target/release/
codeos-reviewer` — the real, existing compiled binary.
_Verify in Step 4:_ run the shim from Codeos's repo root and from a subdirectory
(`tools/reviewer/`), both via relative and absolute script paths; confirm identical resolved
binary path in all cases.

**AC-2 — Correct resolution: symlinked downstream usage (the actual bug)**
Invoked from within a real downstream project through its `.codeos` symlink (FundFlow), the
shim resolves `BINARY` to the exact same real file as AC-1 — Codeos's compiled binary — not a
path under the downstream project's own tree.
_Verify in Step 4:_ run `.codeos/scripts/codeos-review.sh diagnose` from FundFlow's actual
repo root; confirm it succeeds (previously failed with "binary not found") and that the
resolved path (confirm via a debug echo or `readlink -f`) is identical to AC-1's.

**AC-3 — Resolution independent of caller's current working directory**
The resolved binary path does not depend on the directory the user was in when they invoked
the shim — only on the shim script's own physical location.
_Verify in Step 4:_ invoke the shim (Codeos-internal case) from at least two different
caller cwds using a relative path to the script each time; confirm identical resolution.

### No-regression guarantees

**AC-4 — PATH fallback preserved**
If the script-relative path resolves to a non-existent or non-executable file, the existing
`command -v codeos-reviewer` fallback still activates correctly.
_Verify in Step 4:_ temporarily rename/hide the compiled binary (or point at a fixture repo
with no build), confirm the fallback branch is reached (or the correct error emitted if
also absent from PATH).

**AC-5 — Error message references the corrected root**
When the binary is not found and not on PATH, the emitted build-command suggestion
references `${CODEOS_ROOT}` (the script's own resolved root), not the old, broken
`${REPO_ROOT}` — so the suggested `cargo build --manifest-path ...` command is actually
correct when run from a downstream project.
_Verify in Step 4:_ trigger the not-found path from within FundFlow; confirm the printed
build command's path is Codeos's real path, not FundFlow's.

**AC-6 — The exec'd binary's own project discovery is unaffected**
When invoked from FundFlow, the underlying `codeos-reviewer` binary still correctly
discovers *FundFlow* as the project being reviewed (its own internal `discover_repo_root()`,
using the caller's cwd) — this change only fixes where the shim finds the *executable*, not
what project the executable operates on.
_Verify in Step 4:_ run `.codeos/scripts/codeos-review.sh diagnose` from FundFlow; confirm
the output reflects FundFlow's own state (e.g. FundFlow's `reviewer.toml`/provider config),
not Codeos's.

**AC-7 — `--provider` flag and env var precedence unchanged**
Passing `--provider <name>` or setting `CODEOS_REVIEWER_PROVIDER` through the shim still
reaches the binary and takes effect exactly as before this change — the shim's `"$@"`
pass-through and environment are untouched.
_Verify in Step 4:_ run `diagnose` through the fixed shim with an explicit `--provider`
override; confirm the reported provider matches the override.

**AC-8 — Script remains valid, idiomatic bash**
`bash -n scripts/codeos-review.sh` (syntax check) passes; `set -euo pipefail` is preserved.
_Verify in Step 4:_ run `bash -n` against the modified script.

### Cross-reference integrity

**AC-9 — No change to the Rust binary**
_Verify in Step 4:_ `git diff --stat -- tools/reviewer/` is empty for this change.

**AC-10 — No change to binary discovery precedence beyond path resolution, and the
original "caller must be inside a git repository" precondition is preserved exactly**
No new flag, environment variable, or config file is introduced by this change. The
pre-existing precondition — the shim exits 1 with `error: not inside a git repository` if
the *caller* isn't inside any git repository at all — is kept as an explicit, separate
check (using `git rev-parse --show-toplevel` solely for validation, discarding its value),
running before binary resolution, exactly as before. The only genuine behavioral difference
is which physical path `BINARY` resolves to once that precondition passes.
_Verify in Step 4:_ diff the script; confirm the git-repo precondition check is present and
unchanged in exit code/message/ordering; run the shim from a directory with no git
repository at all and confirm identical `exit 1` / message to the pre-change script.

---

## Step 3 — Implement

### What was done

| File | Change |
|---|---|
| `scripts/codeos-review.sh` | Replaced `REPO_ROOT` (`git rev-parse --show-toplevel`) with `SCRIPT_DIR`/`CODEOS_ROOT` (`pwd -P` from the script's own physical location); updated the error message to reference `${CODEOS_ROOT}`; added explanatory comments. `command -v codeos-reviewer` PATH fallback structure unchanged. Restored (Step 3 R2, after a review finding) the original "caller must be inside a git repository" precondition as an explicit, separate check preceding binary resolution — the initial R1 implementation had silently dropped it. |

### Verification (AC-1 through AC-10), all against real invocations, no mocks

- **AC-1**: ran from Codeos's repo root and from `tools/reviewer/` via a relative script
  path — both resolved `repo_root: /home/rimo/projects/Codeos`, identical.
- **AC-2 (the actual bug)**: ran `bash .codeos/scripts/codeos-review.sh diagnose` from
  FundFlow's real repo root — succeeded (previously failed with "binary not found"),
  `toolkit_root: /home/rimo/projects/Codeos` confirms the binary was correctly located
  through the `.codeos` symlink.
- **AC-3**: the subdirectory invocation in AC-1 doubles as this — resolution was identical
  regardless of caller cwd.
- **AC-4/AC-5**: temporarily moved the compiled binary aside, ran the shim — got `error:
  binary not found at /home/rimo/projects/Codeos/tools/reviewer/target/release/
  codeos-reviewer and not on PATH` and the build-command suggestion correctly referencing
  Codeos's real path, exit code 2; binary restored immediately after.
- **AC-6**: FundFlow's real invocation (AC-2) shows `repo_root:
  /home/rimo/projects/FundFlow` — the exec'd binary independently discovered FundFlow as the
  project being reviewed, unaffected by this change to binary-location resolution.
- **AC-7**: `--provider codex` through the shim reported `provider: codex (source: cli
  flag)` — override reaches the binary and takes effect correctly.
- **AC-8**: `bash -n scripts/codeos-review.sh` — syntax OK.
- **AC-9**: `git diff --stat -- tools/reviewer/` → empty.
- **AC-10**: ran the shim from `/tmp` (no git repository at all) — got the exact
  pre-change `error: not inside a git repository`, exit 1, confirming the restored
  precondition check preserves the original contract; diff otherwise shows only the
  `SCRIPT_DIR`/`CODEOS_ROOT` resolution change and the error-message path update.

### Scope check

`git status --short` shows only `scripts/codeos-review.sh` as content, plus the declared
backlog/status bookkeeping — no other file touched.

---

## Step 4 — Reconcile

### Acceptance criteria verification (fresh evidence)

| AC | Verified by | Result |
|---|---|---|
| AC-1 Codeos self-dev resolution | Repo root + subdirectory invocation, `repo_root: /home/rimo/projects/Codeos` both times | PASS |
| AC-2 Real FundFlow symlinked resolution (the actual bug) | `.codeos/scripts/codeos-review.sh diagnose` from FundFlow's real root — succeeds, `toolkit_root: Codeos` | PASS |
| AC-3 cwd-independence | Subdirectory invocation identical to repo-root invocation | PASS |
| AC-4 PATH fallback preserved | Binary moved aside, PATH fallback path correctly reached (absent from PATH → correct error) | PASS |
| AC-5 Error message references corrected root | `Build: cargo build ... --manifest-path /home/rimo/projects/Codeos/tools/reviewer/Cargo.toml` | PASS |
| AC-6 Exec'd binary's own project discovery unaffected | FundFlow invocation shows `repo_root: FundFlow`, `provider: ... (source: reviewer.toml)` — FundFlow's own state | PASS |
| AC-7 `--provider` override reaches the binary | `provider: codex (source: cli flag)` | PASS |
| AC-8 Valid bash | `bash -n` → OK | PASS |
| AC-9 No Rust binary change | `git diff --stat -- tools/reviewer/` → empty | PASS |
| AC-10 Precondition preserved, no overclaim | Ran from `/tmp` — `error: not inside a git repository`, exit 1, byte-identical to original | PASS |

### Cross-reference sweep

- `git status --short` — `scripts/codeos-review.sh` and `docs/reviewer-pipeline.md` as
  content (both declared in "What changes"), plus declared bookkeeping — no other file
  touched.
- No other file in the repo invokes or documents the old `REPO_ROOT` variable name from
  this script (swept `docs/*.md`, `prompts/*.md`, other `scripts/*` for references — none
  found; the script was self-contained).
- `docs/reviewer-pipeline.md`'s §12 "Known limitation" paragraph (added by UPG-0037,
  documenting this exact bug and pointing to UPG-0038) needed updating now that the fix is
  complete — done: retitled from "Known limitation — invoke the binary directly for now"
  to "Invoking the shim from a downstream project," with both example commands switched
  from direct-binary invocation to the now-working `.codeos/scripts/codeos-review.sh`
  shim form (direct invocation kept as a documented, still-valid alternative). Included in
  this same change's "What changes" as an in-scope fix (small doc update directly
  describing the bug being fixed, not a new feature), not filed as a follow-up.

### Reviewer scope triage (Step 4 findings)

Step 1 R1 (NO OBJECTION): no findings. Step 2 R1 (NO OBJECTION): no findings. Step 3 R1 (DO
NOT ADVANCE) found one genuine IN-SCOPE BLOCKER: the initial implementation silently dropped
the shim's original "not inside a git repository" precondition check, a real behavior change
beyond the claimed "only path resolution changes" — fixed by restoring it as an explicit,
separate check; R2 came back clean. Step 4 R1 (DO NOT ADVANCE) found one genuine IN-SCOPE
BLOCKER: after adding `docs/reviewer-pipeline.md` to this change for its stale §12 "Known
limitation" paragraph, §10's own description of the same script was ALSO stale (predating
even this change — "15-line," "the entire file," and "no conditional logic" were already
inaccurate against the pre-change script's existing PATH-fallback conditional, and are more
so now) — fixed by correcting §10's line count, code excerpt, and conditional-logic
description while preserving its still-true core claim (reviewer capability lives entirely
in the Rust engine, never the bash shim).

### Outcome

All 10 ACs verified against the final artifacts with fresh, real-invocation evidence
(table above), including the actual bug scenario against real FundFlow. No in-scope
blockers open. No scope drift — `tools/reviewer/`'s Rust code untouched, discovery
precedence beyond path resolution unchanged, the original git-repo precondition preserved
exactly. Step 4 NO OBJECTION on R3; human APPROVE_STAGE recorded (2026-07-07). Change
record, `status/self-development.md`, `status/roadmap.md`, `backlog/features.md`, and
`backlog/UPG-0038-review-shim-symlink-resolution.md` updated to COMPLETE in this same pass,
following that approval.
