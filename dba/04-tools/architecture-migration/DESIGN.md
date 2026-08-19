---
module: architecture-migration
generated_by: deepseek
verified: none
generated_against_commit: b5114dd
---

<!--
Descriptive documentation of how `dba/04-tools/architecture-migration/migrate-architecture-synthesis-v2.py` currently works.
Explanatory only, and never an authority for behavior or structure: the code is the truth about the
actual implementation. If this note disagrees with the code, the note is stale and gets corrected.

Drafted by DeepSeek from the source and NOT verified claim-by-claim. It describes what the source
does and how; it does not explain why the module was designed this way. Treat any
explanatory-sounding sentence as unverified unless it is attributed to a comment in the source.
-->

# Module: architecture-migration

`migrate-architecture-synthesis-v2.py` is a standalone Python 3 command-line program. Its module docstring says: "One-time conversion from legacy cohort architecture to one Architecture Scope." It reads legacy architecture artifacts from a downstream project repository, validates them, and, when run with `--apply`, writes the new scope artifact and removes the legacy state. Without `--apply`, it only validates and reports a dry run.

## Entry point and command-line interface

The script defines `main() -> int` and, under `if __name__ == "__main__":`, wraps it in `SystemExit`. A `MigrationError` raised inside `main()` is caught and printed as `migration stopped: {exc}` to standard error, then exits with status 1. If PyYAML is missing at import time, the script prints `migration stopped: PyYAML is required (python3 -m pip install PyYAML)` to standard error and exits with status 2.

`argparse` accepts:

- `project` positional argument, default `"."`; it is resolved with `Path.resolve()`.
- `--apply` flag; without it, no files are changed.

## Helper functions

### `fail(message)`

Raises `MigrationError(message)`.

### `load_yaml(path)`

Reads a file as UTF-8 text and parses it with `yaml.safe_load`. If `OSError` or `yaml.YAMLError` occurs, it calls `fail("cannot read valid YAML from {path}: {exc}")`. If the parsed value is not a `dict`, it calls `fail("expected a YAML mapping in {path}")`. Returns the parsed dictionary.

### `field(text, key, table_name)`

Extracts a field value from legacy markdown.

- First tries to find all lines matching `^key:\s*([^\n#]+)` with case-insensitive multiline matching. If any match is found, the last match’s captured group is stripped and returned.
- Otherwise, it looks for a table row matching `^\|\s*table_name\s*\|\s*([^|]+)\|`; if found, the captured group is stripped and returned.
- If neither matches, returns an empty string.

### `membership(text)`

Parses a cohort membership list from a legacy markdown artifact.

- Searches for a bold marker line matching `**Cohort membership set...**`.
- If the marker is absent, fails with `legacy artifact has no cohort membership table`.
- Takes the text after the marker and cuts it at the next line matching `^##\s` or `^---\s*$`.
- Scans the block line by line. Lines starting with `|` have their first pipe-delimited cell captured. Entries whose lowercase value is `feature id`, whose value is `---`, or whose characters are all `-` are excluded.
- If no members remain, fails with `legacy artifact cohort membership table is empty`.
- Returns the list of member strings.

### `section(text, heading)`

Extracts the body of a markdown section headed by exactly `## <heading>`.

- Returns an empty string if the heading is not found.
- Takes the text after the heading line and cuts at the next line starting with `## `.
- Removes `<!-- -->` comments with a non-greedy dot-matching regex, then strips the result.

### `tracked(project, path)`

Runs:

```sh
git -C <project> ls-files --error-unmatch <path-relative-to-project>
```

with stdout and stderr sent to `DEVNULL`. Returns `True` if the exit code is 0, otherwise `False`.

### `atomic_write(path, content)`

Writes `content` as UTF-8 to a sibling temporary file named `.<path.name>.migration-tmp`, then replaces `path` with the temporary file via `Path.replace()`.

## Main validation flow

1. Resolve the `project` argument.
2. Run `git rev-parse --show-toplevel` with the project as `-C` argument.
   - Non-zero exit code produces `not a Git repository: {project}`.
   - The resolved repository root must equal the resolved project path; otherwise failure message: `project path is inside a different Git repository; pass its repository root`.
3. Check `.codeos` in the project root.
   - Fails if it is a symlink or not a directory: `establish the real project-local .codeos directory before architecture migration`.
4. Check required legacy inputs:
   - `features/registry.yaml`
   - `architecture/core-baseline.md`
   - `architecture/cohort-logical-design.md`
   - Each must be a file: `missing legacy input: {relative}`.
   - Each must be tracked by Git: `legacy input is not committed; commit it before migration: {relative}`.
5. Load `features/registry.yaml` with `load_yaml`.
   - `architecture_cohorts` must be a list with exactly one dictionary entry; otherwise the error message states: `migration requires exactly one legacy architecture_cohorts entry; singleton artifacts cannot be assigned safely otherwise`.
   - The cohort’s `cohort_id` is stringified and stripped. It must fully match `[a-z0-9][a-z0-9_-]*`; otherwise: `invalid legacy cohort_id for a scope filename: {scope_id!r}`.
6. Read the full text of `architecture/core-baseline.md` and `architecture/cohort-logical-design.md`.
7. Build membership lists:
   - `registry_members` from `cohort["member_features"]`; must be a non-empty list, converted to strings.
   - `feature_members` by iterating `registry["features"]`; each feature must be a dictionary, and those whose `architecture_cohort` equals `scope_id` are collected using `feature.get("feature_id", "")` stringified.
   - Baseline members via `membership(baseline)`.
   - Logical design members via `membership(logical)`.
   - The four source names are `registry cohort`, `feature entries`, `baseline`, and `logical design`.
   - For each source, the member list must have no duplicates and its set must equal `set(registry_members)`; otherwise: `ambiguous legacy membership in {owner}: {members!r}; expected {registry_members!r}`.
8. Validate cohort IDs in the two markdown artifacts:
   - `field(baseline, "cohort_id", "Cohort id")` and `field(logical, "cohort_id", "Cohort id")` must both equal `scope_id`; otherwise: `cohort id conflict: registry={scope_id!r}, baseline={baseline_id!r}, logical={logical_id!r}`.
9. Validate approval status:
   - `baseline_status = field(baseline, "status", "Status").lower()`
   - `logical_status = field(logical, "status", "Status").lower()`
   - `approved_pair` is true only when both status strings equal `"approved"`.
   - If exactly one of the two status strings equals `"approved"`, fails with `only one legacy architecture artifact is approved; human resolution is required`.
   - `registry_status = str(cohort.get("status", "")).lower()` must agree with `approved_pair`; otherwise: `approval conflict: registry={registry_status!r}, baseline={baseline_status!r}, logical={logical_status!r}`.
10. When both legacy artifacts are approved:
    - `baseline_version` and `logical_design_version` fields are read from the markdown and must match `cohort["baseline_version"]` and `cohort["logical_design_version"]` respectively; otherwise separate failures.
    - `approved_by` and `approved_at` fields from both files must be present in baseline and equal in both files; otherwise: `legacy artifacts do not establish one unambiguous joint approval`.
    - The baseline’s `approved_by` and `approved_at` are retained for the output.
11. Extract decision sections:
    - `section(baseline, "Authoritative Decisions")`
    - `section(logical, "Logical Design Decisions")`
    - If either is empty: `cannot find both legacy architecture decision sections`.
12. Compute the target scope path: `.codeos/02-architecture/scopes/{scope_id}.md`. If it exists: `target already exists: {relative}`.

## Generated scope document

The scope document contains YAML front matter built from `metadata`:

```python
{
    "features": registry_members,
    "approval": {"by": approved_by, "at": approved_at} if approved_pair else None,
}
```

The front matter is serialized with `yaml.safe_dump(metadata, sort_keys=False, default_flow_style=False).strip()`.

The body is:

```markdown
# Architecture Scope: {scope_id}

Migrated without reinterpretation from the legacy Architecture Baseline and Cohort Logical Design. Git preserves the source revisions.

## Preserved project-level decisions

{decisions}

## Preserved shared logical decisions

{logical_decisions}
```

## Implementation Profile handling

If `architecture/implementation-profile.yaml` exists:

- If `.codeos/02-architecture/implementation-profile.yaml` also exists, fails with `target already exists: {relative}`.
- Loads the profile with `load_yaml`.
- `applies_to` must be a dictionary; otherwise: `Implementation Profile applies_to must be a mapping`.
- Allowed keys in `applies_to` are `scope`, `feature_ids`, and `cohort_ids`; any other key fails with `Implementation Profile applies_to has unsupported fields: {sorted(unknown)!r}`.
- `cohort_ids`, when present, must be a list; otherwise: `Implementation Profile cohort_ids must be a list`.
- Selector transformation:
  - If `scope == "cohort_ids"`: requires `cohort_ids == [scope_id]` and `feature_ids` absent or empty; otherwise fails with `Implementation Profile cohort selector does not resolve unambiguously to the migrated scope`. Then sets `applies["scope"] = "feature_ids"`, `applies["feature_ids"] = list(registry_members)`, and removes `cohort_ids`.
  - If `scope in {"all", "feature_ids"}`: if `cohort_ids` is non-empty, fails with `Implementation Profile has populated unused cohort_ids`; removes `cohort_ids` if present.
  - Otherwise fails with `unsupported Implementation Profile selector during migration: {selector!r}`.
- Exceptions:
  - `profile.get("exceptions") or []` must be a list.
  - Each exception must be a dict with keys limited to `scope`, `id`, `language`, `rationale`.
  - `scope == "feature_id"` adds the exception to `explicit`.
  - `scope == "cohort_id"` requires `id == scope_id`, otherwise fails with `Implementation Profile exception names an unknown legacy cohort`; adds it to `cohort_exceptions`.
  - Any other scope fails with `unsupported Implementation Profile exception selector: {scope!r}`.
- Nested function `add_exception(feature_id, source)`:
  - Fails on empty `feature_id`.
  - Requires `language` and `rationale` to both be strings.
  - Trims both; failure if either becomes empty.
  - Tracks `resolved_languages[feature_id]`; if the same feature appears twice with a different language, fails with `Implementation Profile exceptions disagree for feature {feature_id}`. A duplicate with the same language is ignored.
  - Appends a normalized dict: `{"scope": "feature_id", "id": feature_id, "language": language, "rationale": rationale}`.
- Explicit exceptions: `exception["id"]` must be a string; `add_exception(feature_id.strip(), exception)` is called.
- Cohort exceptions: `add_exception(feature_id, exception)` is called for every member of `registry_members`.
- If `migrated_exceptions != exceptions`, the profile’s `exceptions` key is replaced with `migrated_exceptions`.
- After all profile transformations, if the legacy profile is not tracked by Git, fails with `Implementation Profile is not committed; commit it before migration`.

## History file validation

The script builds `legacy_history` as the sorted list of paths matching:

- `architecture/history/core-baseline-v*.md`
- `architecture/history/cohort-logical-design-v*.md`

Every matched file must be tracked by Git; otherwise: `legacy history is not committed; commit it before migration: {relative}`.

## Dry run output

After validation succeeds, the script prints:

```text
validated legacy scope {scope_id}: {len(registry_members)} features, target state {target_state}
```

where `target_state` is `"approved"` if `approved_pair` is true, otherwise `"draft"`.

If `--apply` was not given, it prints:

```text
dry run only; rerun with --apply to convert and remove legacy state
```

and returns 0.

## Apply sequence

When `--apply` is present, the script, in order:

1. Creates the scope directory: `scope_path.parent.mkdir(parents=True, exist_ok=True)`.
2. Writes the scope document with `atomic_write(scope_path, output)`.
3. Iterates `registry.get("features", [])`; for each dictionary entry, removes `architecture_cohort` if present.
4. Removes `architecture_cohorts` from the registry root.
5. Rewrites `features/registry.yaml` with `yaml.safe_dump(registry, sort_keys=False)` via `atomic_write`.
6. If a legacy Implementation Profile exists:
   - Creates the target profile directory.
   - Writes the transformed profile to `.codeos/02-architecture/implementation-profile.yaml` via `atomic_write`.
   - Deletes `architecture/implementation-profile.yaml` with `Path.unlink()`.
7. Deletes `architecture/core-baseline.md`, `architecture/cohort-logical-design.md`, and every path in `legacy_history`.
8. Prints:

```text
migrated to {scope_path.relative_to(project)}; legacy state removed
```

and returns 0.

## External effects and dependencies

- Requires PyYAML; the import failure exits with status 2.
- Invokes the `git` command-line executable:
  - `git -C <project> rev-parse --show-toplevel`
  - `git -C <project> ls-files --error-unmatch <relative-path>`
- Reads files:
  - `features/registry.yaml`
  - `architecture/core-baseline.md`
  - `architecture/cohort-logical-design.md`
  - `architecture/implementation-profile.yaml` if present
- Writes files:
  - `.codeos/02-architecture/scopes/<scope_id>.md`
  - `features/registry.yaml` on apply
  - `.codeos/02-architecture/implementation-profile.yaml` on apply if a legacy profile existed
  - temporary sibling files used by `atomic_write`
- Deletes on apply:
  - `architecture/core-baseline.md`
  - `architecture/cohort-logical-design.md`
  - every file matching the history globs
  - `architecture/implementation-profile.yaml` if a legacy profile existed

## Error behavior

All validation failures raised through `fail()` become `MigrationError`, are caught by the `__main__` block, printed as `migration stopped: {message}` to standard error, and exit with status 1. `load_yaml` converts file-read and YAML-parse errors into `MigrationError`. `tracked` suppresses subprocess output and treats non-zero exit as “not tracked”. `atomic_write` does not catch I/O exceptions. Subprocess calls do not explicitly handle a missing `git` executable; an uncaught `FileNotFoundError` would propagate.
