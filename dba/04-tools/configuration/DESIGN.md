---
module: configuration
generated_by: deepseek
verified: none
generated_against_commit: b5114dd
---

<!--
Descriptive documentation of how `dba/04-tools/configuration/` (`layout-contract.sh`, `guidance-contract.sh`, `dba-config-boundaries.sh`) currently works.
Explanatory only, and never an authority for behavior or structure: the code is the truth about the
actual implementation. If this note disagrees with the code, the note is stale and gets corrected.

Drafted by DeepSeek from the source and NOT verified claim-by-claim. It describes what the source
does and how; it does not explain why the module was designed this way. Treat any
explanatory-sounding sentence as unverified unless it is attributed to a comment in the source.
-->

# configuration module

The `dba/04-tools/configuration` directory contains three Bash scripts: `layout-contract.sh`, `guidance-contract.sh`, and `dba-config-boundaries.sh`. Each script computes `CODEOS_ROOT` as the repository path three levels above the script directory and uses a `fail()` helper that writes a message to stderr and exits with status 1. `layout-contract.sh` and `guidance-contract.sh` use `set -euo pipefail`; `dba-config-boundaries.sh` uses `set -uo pipefail`.

Only `layout-contract.sh` invokes another script in this directory. At its end it runs `dba-config-boundaries.sh` against the active configuration.

## `layout-contract.sh`

`layout-contract.sh` performs a sequence of repository-wide checks. Any failed check prints `Codeos layout contract failed: <reason>` to stderr and exits 1. A successful run prints `Codeos layout contract OK`.

### Required root files and directories

It requires these files at the repository root:

- `README.md`
- `AGENTS.md`
- `CLAUDE.md`
- `dba-system.md`

It requires these directories:

- `dba/00-entry`
- `dba/01-doctrine`
- `dba/02-policies`
- `dba/03-prompts`
- `dba/04-tools`
- `dba/05-guidance`
- `dba/06-reference`
- `maintenance/backlog`
- `maintenance/reviews`
- `maintenance/config`
- `maintenance/archive`

### Legacy directory absence

It fails if any of these paths exist at the repository root:

- `Archive`
- `backlog`
- `config`
- `docs`
- `patterns`
- `prompts`
- `reviews`
- `scripts`
- `templates`
- `tools`
- `dba/configurations`
- `dba/doctrine`
- `dba/policies`
- `dba/tools`

### Active reference scan

It runs `rg` over these paths:

- `README.md`
- `CLAUDE.md`
- `dba-system.md`
- `dba`
- `maintenance/backlog`
- `maintenance/config`
- `maintenance/reviews`

The scan excludes `downstream-upgrade.md` and looks for references to legacy toolkit paths with the pattern:

```
\.codeos/(dba-system\.md|dba/|(prompts|scripts|templates|patterns|tools)(/|`)|terminology\.md)|dba/(configurations|doctrine|policies|tools)/
```

A match fails with `an active file references a legacy toolkit path`.

### Retired mechanism checks

The script defines a `supported_runtime_paths` array containing `dba-system.md`, selected entry/config files, prompt and tool directories, and the guidance templates directory. It runs `rg` over those paths, excluding `tests/**`, for these patterns:

- `Controlled Plain English`
- `controlled-plain-english`
- `controlled_plain_english`
- `writing-discipline`
- `codeos-cpe-status`
- `CPE_STATUS`
- `CPE_CONFIG`
- `Layer [ABCD][12]?`

A match fails with `a retired mechanism is referenced by the supported runtime/configuration surface`.

It then requires these retired paths not exist:

- `maintenance/config/writing-discipline.yaml`
- `dba/05-guidance/patterns/controlled-plain-english.md`
- `dba/02-policies/controlled-plain-english/v1.md`
- `dba/03-prompts/workflow/00-full-solution-concept.md`
- `dba/05-guidance/templates/full-solution-concept.md`

A remaining path fails with `retired active artifact remains: <path>`.

A source comment states that retired mechanisms must not re-enter the supported runtime/configuration surface, that historical records remain valid outside the checked list, and that this is an active-layout invariant rather than a history scan.

### Workflow prompt governance

For stages `01` through `09`, it runs `find` over `dba/03-prompts/workflow` with `maxdepth 1`, matching `NN-*.md`. For each stage it requires exactly one such file and that the file’s first line declares `# Stage N:` where `N` is the stage number without the leading zero. Otherwise it fails with either a stage count message or a missing declaration message.

It then compares the sorted set of `support-*.md` files in that directory against this exact expected inventory:

```text
support-architecture-synthesis.md
support-existing-codebase-intake.md
support-feature-decomposition.md
support-session-handoff.md
support-session-orientation.md
support-solution-charter.md
support-solution-framing.md
```

A mismatch fails with `unexpected support workflow inventory`.

For every `*.md` workflow prompt, the script skips filenames beginning with `0[1-9]-`. Any other file must have a descriptive `support-...` filename matching:

```regex
^support-[a-z0-9]+(-[a-z0-9]+)*\.md$
```

and must not contain a `^# Stage [0-9]` heading. Violations fail with either `non-stage workflow prompt lacks a descriptive support- filename` or `support prompt masquerades as a governed stage`.

### Reviewer surface check

It runs `rg` over these paths:

- `dba/03-prompts/review`
- `dba/04-tools/reviewer/contract`
- `dba/04-tools/reviewer/engine/src`
- `dba/04-tools/reviewer/engine/tests`

looking for quoted or backticked forms of `discovery`, `brief`, or `onboarding`. A match fails with `an active reviewer surface uses a retired support-workflow identifier`.

### Canonical paths in `dba-system.md`

The script requires `dba-system.md` to contain each of these literal paths:

```text
.codeos/00-project/CLAUDE.md
.codeos/00-project/charter.md
.codeos/00-project/learnings.md
.codeos/00-project/terminology.md
.codeos/01-specification/intents/<feature-id>.md
.codeos/01-specification/contracts/<feature-id>_contract.md
.codeos/01-specification/event-schemas/<feature-id>_schema.md
.codeos/02-architecture/scopes/<scope-id>.md
.codeos/02-architecture/implementation-profile.yaml
.codeos/03-design/<module-slug>.md
.codeos/00-discovery/<topic-slug>.md
.codeos/04-refinement/<feature-id>-<slug>.md
.codeos/05-review/reviewer.toml
.codeos/05-review/reviews/
.codeos/05-review/measurements/<name>.md
.codeos/toolkit
.codeos-state/
events/runtime_events.jsonl
```

If any is missing, it fails with `downstream layout owner omits canonical path: <path>`.

### Producer-output map

The script declares an associative array mapping workflow prompts to the canonical output path each prompt must mention:

```text
support-solution-charter.md          -> .codeos/00-project/charter.md
support-solution-framing.md          -> .codeos/00-discovery/<topic-slug>.md
support-feature-decomposition.md     -> .codeos/00-discovery/<topic-slug>.md
support-existing-codebase-intake.md  -> .codeos/01-specification/intents/<feature-id>.md
01-intent.md                         -> .codeos/01-specification/intents/<feature-id>.md
02-contract.md                       -> .codeos/01-specification/contracts/<feature-id>_contract.md
03-event-schema.md                   -> .codeos/01-specification/event-schemas/<feature-id>_schema.md
support-architecture-synthesis.md    -> .codeos/02-architecture/scopes/<scope-id>.md
09-refine.md                         -> .codeos/04-refinement/<feature-id>-<slug>.md
```

For each prompt, the file must contain the mapped canonical path, and the mapped path must itself be present in the `canonical_paths` list. Failures report `artifact-producing prompt omits canonical output` or `producer output is not a canonical path`.

A source comment explains that the map is deliberately kept because the prompt-to-artifact relation is many-to-many and partial, and is not derivable from the layout.

### Placeholder-notation check

It runs:

```bash
grep -rn '\.codeos/[^ `"]*\[[a-z_-]*\]' "${CODEOS_ROOT}/dba" "${CODEOS_ROOT}/dba-system.md"
```

capturing output with `|| true`. Any non-empty result fails with:

```text
canonical path uses a non-canonical placeholder notation (use <kebab-case>): <match>
```

A source comment states that a second spelling of the same governed fact is refused rather than bridged, and that content placeholders inside template bodies are not matched.

### Doctrine adapter agreement

The active configuration is fixed in this script as:

```text
dba/00-entry/configurations/DBA-3.yaml
```

It extracts the value of the `doctrine:` line with `awk`, requires it to be non-empty, resolves it under `CODEOS_ROOT`, and requires that file to exist.

It parses the selected doctrine file for the section starting at `### Doctrine Adapter Boundaries`, collecting one backticked `[a-z-]+` name per `- \`name\`` list item until the next `## ` heading, sorted with `LC_ALL=C`.

It also collects all `DOCTRINE ADAPTER: <name>` occurrences under `dba/03-prompts`, sorted. It fails if the prompt-declared list contains duplicates, with `a doctrine adapter is declared by more than one prompt`. It then requires the doctrine-declared adapter list and the prompt-declared adapter list to be identical, failing with `adapter boundaries disagree`.

### Boundary script invocation

Finally, it runs:

```bash
bash "${CODEOS_ROOT}/dba/04-tools/configuration/dba-config-boundaries.sh" \
  dba/00-entry/configurations/DBA-3.yaml >/dev/null
```

The boundary script’s stdout is discarded; only its failure status or messages matter.

## `guidance-contract.sh`

`guidance-contract.sh` verifies the active guidance inventory and the presence of specific workflow boundary phrases in selected prompts. A failed check prints `guidance contract failed: <reason>` and exits 1. Success prints `guidance contract: PASS`.

### Guidance inventory

It requires the files under `dba/05-guidance` to match exactly this sorted inventory:

```text
patterns/rust-project-structure.md
patterns/shared-infrastructure-boundary.md
patterns/vocabulary-architecture.md
templates/architecture-scope.md
templates/charter.md
templates/contract.md
templates/event-schema.md
templates/feature-decomposition.md
templates/implementation-profile.yaml
templates/intent.md
templates/learning-register.md
templates/module-design-note.md
templates/project-AGENTS.md
templates/project-CLAUDE.md
templates/project-root-CLAUDE.md
templates/project-terminology.md
templates/refinement.md
templates/review-file.md
templates/review-package.md
templates/reviewer.toml
terminology.md
```

It obtains the actual inventory with:

```bash
find . -type f -printf '%P\n' | LC_ALL=C sort
```

A difference fails with `unexpected active guidance inventory`. A source comment notes that `LC_ALL=C` keeps collation byte-ordered so the expected list is not locale-dependent.

### Prompt boundary phrases

The script assigns these paths:

| Variable | Prompt |
|---|---|
| `SESSION_START` | `dba/03-prompts/workflow/support-session-orientation.md` |
| `INTENT_PROMPT` | `dba/03-prompts/workflow/01-intent.md` |
| `FRAMING_PROMPT` | `dba/03-prompts/workflow/support-solution-framing.md` |
| `CHARTER_PROMPT` | `dba/03-prompts/workflow/support-solution-charter.md` |
| `ARCHITECTURE_PROMPT` | `dba/03-prompts/workflow/support-architecture-synthesis.md` |
| `IMPLEMENT_PROMPT` | `dba/03-prompts/workflow/04-implement.md` |

It then uses `rg -q` to require these exact phrases:

| Prompt | Required phrase | Failure message |
|---|---|---|
| `SESSION_START` | `partially drafted Specification Package is normal` | `partial packages are not accepted` |
| `SESSION_START` | `terminology.md.*exists` | `optional project terminology is not loaded` |
| `SESSION_START` | `Its absence is valid` | `project terminology became mandatory` |
| `INTENT_PROMPT` | `architecture-scope membership` | `feature-id allocation does not scan every identity owner` |
| `INTENT_PROMPT` | `incompatible artifacts claim the same identity` | `identity-conflict boundary is missing` |
| `INTENT_PROMPT` | `Define only feature-local` | `Intent terminology ownership is unclear` |
| `FRAMING_PROMPT` | `when promoted into and approved in the Solution Charter` | `Solution Framing promotion boundary is missing` |
| `FRAMING_PROMPT` | `MUST NOT resolve that concern into components` | `Solution Framing architecture boundary is missing` |
| `CHARTER_PROMPT` | `only their promotion into an approved Charter makes them` | `Solution Charter promotion boundary is missing` |
| `ARCHITECTURE_PROMPT` | `Stage 4 owns local implementation design` | `Architecture Synthesis design boundary is missing` |
| `IMPLEMENT_PROMPT` | `feature-local design decisions inside approved architectural boundaries` | `Stage 4 local design ownership is missing` |

## `dba-config-boundaries.sh`

`dba-config-boundaries.sh` validates a candidate DBA configuration file by checking the Markdown components it selects. The script’s first comment says it verifies the minimum responsibility boundary for components selected by a candidate DBA config.

### Arguments and resolution

It requires exactly one argument. The argument is either an absolute path or a path resolved relative to `CODEOS_ROOT`. The configuration file must exist.

### Component selection

It extracts components with `sed` from lines matching:

```regex
^([a-z_][a-z0-9_]*):[[:space:]]*([^[:space:]#][^[:space:]#]*\.md)[[:space:]]*$
```

Each captured line becomes an entry of the form `key|path`. If no Markdown components are selected, it fails with `candidate configuration selects no Markdown components: <path>`.

### Boundary-contract validation

For each selected component, the script resolves the path under `CODEOS_ROOT` and requires the file to exist. It then reads the first four lines. The component passes only if:

- line 1 is exactly `---`
- line 2 is `component_question: <non-empty value>`
- line 3 is `out_of_scope: <non-empty value>`
- line 4 is exactly `---`

Failure messages distinguish missing files, missing boundary metadata, empty `component_question`, empty `out_of_scope`, and unsupported boundary metadata.

On success it prints:

```text
DBA boundary contract OK: <path> (<count> checked)
```

where `<count>` is the number of Markdown components checked.
