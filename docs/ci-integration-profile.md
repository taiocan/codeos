# CI Integration Profile

> **Optional, illustrative, not enforced.** Nothing in this document is required by Codeos.
> It maps the DBA evidence types a project already produces onto concrete CI gate sketches a
> downstream DBA project can adopt, adapt, or ignore. No Codeos tool installs or requires any
> of these checks; `dba-system.md` does not reference this document.

Local evidence (a passing behavioral test suite, a clean replay run) can silently diverge
from what CI actually checks if CI was never told what DBA-specific evidence exists to check
in the first place. This document closes that gap by naming, for each evidence type Codeos
already produces, what a corresponding CI gate could look like.

Related but distinct: [`workflow-profiles.md`](workflow-profiles.md) covers branch/PR
discipline (when to branch, when to split a PR). This document covers what CI *checks* once
you have one — a different concern. Read both if you're setting up CI for a DBA project.

---

## 1. Behavioral tests

**What it checks:** the Stage 5 behavioral test suite (one test per contract scenario,
including failure paths) still passes.

**Illustrative CI step:**
```yaml
- name: Behavioral tests
  run: <your test runner> tests/behavioral/
```
Fail the build on any failure. This is the most standard check here — most CI setups already
run *some* test suite; this just names `tests/behavioral/` as the DBA-specific target.

---

## 2. Replay tests

**What it checks:** Stage 5/8 replay verification tests — that recorded or fixture runtime
event sequences conform to the approved event schema and contract-derived ordering.

**Illustrative CI step:**
```yaml
- name: Replay tests
  run: <your test runner> tests/replay/
```
Fail the build on any failure, same as behavioral tests. Keep this as a separate step (not
merged into behavioral tests) so a replay-specific failure is immediately visible in CI logs.

---

## 3. Schema conformance tests

**What it checks:** captured runtime events (from `events/runtime_events.jsonl` or a replay
fixture) conform to the approved event schema — named events only, required fields present,
no speculative telemetry.

**Illustrative CI step:** this can be folded into the replay test step above if your replay
tests already assert schema conformance, or run as its own step if you maintain a separate
schema-validation script:
```yaml
- name: Schema conformance
  run: <your schema validator> events/<feature_id>_schema.md events/runtime_events.jsonl
```

---

## 4. Lint / typecheck

**What it checks:** standard static analysis for your language — not DBA-specific, but
listed here because it's part of the evidence bar a feature should clear before merge.

**Illustrative CI step:** use your project's normal lint/typecheck tooling (e.g. `cargo
clippy`, `eslint`, `mypy`, `ruff`) — no Codeos-specific guidance needed here.

---

## 5. No unapproved event names

**What it checks:** every event name actually emitted (sampled from a runtime log or replay
fixture) appears in the feature's approved event schema. An emitted name that isn't in the
schema is exactly the hidden behavior prohibited by the active DBA doctrine.

**Illustrative CI step (sketch, not a shipped tool):**
```bash
# Extract emitted event names from a runtime log sample, diff against the schema's
# named events; fail if any emitted name is missing from the schema.
emitted=$(jq -r '.event' events/runtime_events.jsonl | sort -u)
approved=$(grep -oP '(?<=^### )\w+' events/<feature_id>_schema.md | sort -u)
comm -23 <(echo "$emitted") <(echo "$approved") | grep -q . && {
  echo "unapproved event name(s) found"; exit 1;
}
```
Adapt the extraction commands to your actual log/schema format — this is illustrative, not a
Codeos-shipped script.

---

## 6. No raw runtime log leakage

**What it checks:** `events/runtime_events.jsonl` (or excerpts committed as fixtures) don't
leak unsanitized secrets or PII into version control.

**Illustrative CI step (sketch):**
```bash
# Fail if a committed runtime-log fixture matches an obvious secret pattern.
grep -RIlE '(api[_-]?key|secret|password|token)\s*[:=]' events/*.jsonl tests/replay/ && {
  echo "possible unsanitized secret in a committed runtime log fixture"; exit 1;
}
```
This is a starting point, not a substitute for a real secret-scanning tool if your project
needs one — see the active `doctrine` component's Sanitization Status guidance for the underlying practice.

---

## 7. Stack manifest reconciliation if dependency/config changed

**What it checks:** if a dependency or config file changed (`Cargo.toml`, `package.json`,
`Dockerfile`, `config/*.yaml`, etc. — see `templates/stack-manifest.md`'s watched-file list),
a `stack-reconciliation-report.md` instance is present in the same diff.

**This one has an already-built, concrete mechanism** — no sketch needed:
```yaml
- name: Stack drift check
  run: codeos-reviewer check-drift --base ${{ github.event.pull_request.base.sha }}
```
`check-drift` (`tools/reviewer/src/cmd/check_drift.rs`) exits `0` if no watched file changed,
or if one did change but a `stack-reconciliation-report.md` is present in the same diff. It
exits `6` (`EXIT_DRIFT`) if a watched file changed with no reconciliation report. It exits
`2` (`EXIT_CONFIG`) separately if `git diff` itself fails — e.g. an invalid `--base` ref, or
git unavailable — a distinct "couldn't even run the check" failure, not a drift finding.
Treat any non-zero exit as a failed CI gate either way. Pass `--strict` to prefix the drift
message with `STRICT MODE:` (same exit-code behavior in all three cases).

---

## Adopting this profile

None of the above is required. A project can adopt all seven checks, a subset, or none —
Codeos does not gate anything on this document. If you do adopt some subset, keep the CI
step names close to the section headings above so a reader can trace a CI failure back to
the evidence type it corresponds to.
