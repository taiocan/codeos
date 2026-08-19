---
module: initializer
generated_by: deepseek
verified: none
generated_against_commit: b5114dd
---

<!--
Descriptive documentation of how `dba/04-tools/initializer/dba-init.sh` currently works.
Explanatory only, and never an authority for behavior or structure: the code is the truth about the
actual implementation. If this note disagrees with the code, the note is stale and gets corrected.

Drafted by DeepSeek from the source and NOT verified claim-by-claim. It describes what the source
does and how; it does not explain why the module was designed this way. Treat any
explanatory-sounding sentence as unverified unless it is attributed to a comment in the source.
-->

# initializer

`dba-init.sh` is a Bash script that initializes a minimal project-local Codeos structure in the current working directory. It reads two optional positional arguments, performs preflight checks, then creates directories, a toolkit symlink, `.gitignore` rules, root adapter files, a project-local `CLAUDE.md`, and optionally a Git repository and remote.

## Invocation and inputs

- `$1` is the project name. If omitted, it defaults to `basename "$PROJECT_DIR"`.
- `$2` is a remote Git URL. If omitted, it is empty and no remote is configured.
- The script is run from the directory it will initialize; `PROJECT_DIR` is captured as the canonicalized current working directory via `pwd -P`.

The script has no flag parsing. It uses `set -euo pipefail`, so an unhandled command failure or unset variable terminates the script.

## Path resolution

The script derives these paths at startup:

- `SCRIPT_DIR`: canonical directory containing the script.
- `CODEOS_PATH`: canonical directory three levels above `SCRIPT_DIR` (`$SCRIPT_DIR/../../..`). It is used as the toolkit target and as the base for template paths.
- `PROJECT_DIR`: canonical current working directory.
- `CODEOS_DIR`: `$PROJECT_DIR/.codeos`
- `TOOLKIT_LINK`: `$CODEOS_DIR/toolkit`
- `ROOT_CLAUDE`: `$PROJECT_DIR/CLAUDE.md`
- `ROOT_AGENTS`: `$PROJECT_DIR/AGENTS.md`
- `ROOT_CLAUDE_TEMPLATE`: `$CODEOS_PATH/dba/05-guidance/templates/project-root-CLAUDE.md`
- `ROOT_AGENTS_TEMPLATE`: `$CODEOS_PATH/dba/05-guidance/templates/project-AGENTS.md`
- `PROJECT_CLAUDE`: `$CODEOS_DIR/00-project/CLAUDE.md`

## Preflight checks

All failing checks happen before any filesystem mutation.

- The project name must not contain a newline or carriage return.
- The script runs `git -C "$PROJECT_DIR" rev-parse --show-toplevel`. If that succeeds:
  - The output is canonicalized as `GIT_ROOT`.
  - If `GIT_ROOT` differs from `PROJECT_DIR`, the script fails, reporting that the project directory is inside an existing Git repository and that the repository root should be initialized instead.
  - If `GIT_ROOT` equals `PROJECT_DIR`, the project is already a Git repository root and `PROJECT_HAS_GIT` is set to `true`.
  - If `git rev-parse` fails, `PROJECT_HAS_GIT` remains `false`.
- If `$ROOT_CLAUDE` exists as a regular file and differs from `$ROOT_CLAUDE_TEMPLATE` when compared with `cmp -s`, the script fails with a migration message.
- If `$ROOT_CLAUDE` exists but is not a regular file, the script fails.
- The same two checks are applied to `$ROOT_AGENTS` against `$ROOT_AGENTS_TEMPLATE`.
- If `$CODEOS_DIR` is a symlink, the script fails, calling it a legacy toolkit symlink.
- If `$CODEOS_DIR` exists but is not a directory, the script fails.
- If `$TOOLKIT_LINK` is a symlink, it must point to a valid toolkit: both `$TOOLKIT_LINK/dba-system.md` and `$TOOLKIT_LINK/dba/00-entry` must exist. Otherwise the script fails.
- If `$TOOLKIT_LINK` exists but is not a symlink, the script fails.
- If `$PROJECT_CLAUDE` exists but is not a regular file, the script fails.
- If `$PROJECT_DIR/.gitignore` exists but is not a regular file, the script fails.

Failure messages are written to stderr in the form `[error] ...` and the script exits with status `1`.

## Mutating steps

After preflight passes, the script performs these steps in order:

1. Creates directories:
   - `$CODEOS_DIR/00-project`
   - `$CODEOS_DIR/01-specification/intents`
   - `$CODEOS_DIR/01-specification/contracts`
   - `$CODEOS_DIR/01-specification/event-schemas`

2. Configures the toolkit link:
   - If `$TOOLKIT_LINK` is already a valid symlink, the script prints a skip message.
   - Otherwise it creates the symlink with `ln -s "$CODEOS_PATH" "$TOOLKIT_LINK"`.

3. Ensures `.gitignore` contains two exact lines:
   - `/.codeos/toolkit`
   - `/.codeos-state/`
   - For each missing rule, the script checks with `grep -Fxq`.
   - If any are missing, it appends them to `$PROJECT_DIR/.gitignore`. If the file exists and is non-empty, it appends a blank line before the rules.
   - If all rules are present, it prints a skip message.

4. Creates `.codeos/00-project/CLAUDE.md` if absent:
   - The project name is escaped by replacing backslashes with `\\`, `&` with `\&`, and `|` with `\|`.
   - `sed` replaces every occurrence of `[PROJECT_NAME]` in `dba/05-guidance/templates/project-CLAUDE.md` with the escaped project name, writing to `$PROJECT_CLAUDE`.
   - If `$PROJECT_CLAUDE` already exists, it is left unchanged and a skip message is printed.

5. Creates the root `CLAUDE.md` if absent by copying `$ROOT_CLAUDE_TEMPLATE` to `$ROOT_CLAUDE`. If it already exists, it is left unchanged and a skip message is printed.

6. Creates the root `AGENTS.md` if absent by copying `$ROOT_AGENTS_TEMPLATE` to `$ROOT_AGENTS`. If it already exists, it is left unchanged and a skip message is printed.

7. Initializes Git if needed:
   - If `PROJECT_HAS_GIT` is `true`, it prints a skip message.
   - Otherwise it runs `git -C "$PROJECT_DIR" init -b main`, sending stdout to `/dev/null`.

8. Configures the Git remote if a remote URL was supplied:
   - If `$REMOTE_URL` is non-empty and `git -C "$PROJECT_DIR" remote get-url origin` succeeds, it prints a skip message.
   - If `$REMOTE_URL` is non-empty and no `origin` remote URL is found, it runs `git -C "$PROJECT_DIR" remote add origin "$REMOTE_URL"`.
   - If `$REMOTE_URL` is empty, no remote action is taken.

The script ends by printing a message saying initialization is done and directing the user to fill in `.codeos/00-project/CLAUDE.md` and to use a named workflow file to establish a Solution Charter.

## Output

To stdout, the script prints:

- A header: `DBA Project Init`
- The resolved project name, project directory, and toolkit path.
- Per-step lines beginning with `[ok]` for actions it performs or `[skip]` for items it leaves in place.
- A final instruction line.

To stderr, it prints only failure messages beginning with `[error]`.

## External dependencies

The script invokes the following external programs: `git`, `mkdir`, `ln`, `grep`, `printf`, `sed`, `cp`, `cmp`, `dirname`, `basename`, and `pwd`.
