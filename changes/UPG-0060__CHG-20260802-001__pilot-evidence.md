# CHG-A Pilot Evidence — UPG-0060__CHG-20260802-001

> Durable, committed record of the empirical claims in the change record's Implementation Notes,
> produced by ONE canonical run: the fail-closed suite, one Stage-4 pilot, and one advisory gate check
> on that pilot's candidate. The live candidate is staged under gitignored
> `.codeos-state/deepseek-candidates/` (never promoted, never committed); this file is the auditable
> evidence. DeepSeek output is non-deterministic, so a later run may produce a different candidate
> path or token count; the figures below are this canonical run's.

## 1. Fail-closed suite (AC-1..8) — no network in any case

Runner (embedded verbatim; also at `scratchpad/failclosed_tests.sh` during development):

```bash
#!/usr/bin/env bash
# Fail-closed verification for codeos-implement.sh (no network calls in any case here).
# Saves and restores the shipped status file so it ends at 'status: disabled'.
REPO=/home/rimo/projects/Codeos
SH="${REPO}/scripts/codeos-implement.sh"
STATUS="${REPO}/config/delegated-implementation.yaml"
ART="${REPO}/backlog/UPG-0060-deepseek-delegated-implementation.md"   # any existing file
PASS=0; FAIL=0
orig="$(cat "${STATUS}")"

run() { # description  expected_code  -- command...
  local desc="$1" exp="$2"; shift 3
  local out code
  out="$("$@" 2>&1)"; code=$?
  if [[ "${code}" == "${exp}" ]]; then
    echo "PASS [${code}] ${desc}"; PASS=$((PASS+1))
  else
    echo "FAIL [got ${code}, want ${exp}] ${desc}"; echo "     out: ${out}" | head -2; FAIL=$((FAIL+1))
  fi
}

echo "== AC-5 git precondition (run from non-git dir) -> 1 =="
TMPD="$(mktemp -d)"
( cd "${TMPD}" && bash "${SH}" F 4 "${ART}" ) >/tmp/_o 2>&1; c=$?
[[ $c == 1 ]] && { echo "PASS [1] non-git repo"; PASS=$((PASS+1)); } || { echo "FAIL [got $c want 1] non-git"; FAIL=$((FAIL+1)); cat /tmp/_o; }
rmdir "${TMPD}" 2>/dev/null || true

cd "${REPO}"
echo "== AC-3 usage: too few args -> 3 =="
run "missing args" 3 -- bash "${SH}" F 4
echo "== AC-7(usage) bad stage -> 3 =="
run "stage=6 rejected" 3 -- bash "${SH}" F 6 "${ART}"

echo "== AC-2 disabled -> 4 =="
printf 'status: disabled\n' > "${STATUS}"
run "status disabled refuses" 4 -- bash "${SH}" F 4 "${ART}"

echo "== AC-1 absent -> 4 =="
rm -f "${STATUS}"
run "absent status refuses" 4 -- bash "${SH}" F 4 "${ART}"

echo "== AC-3 malformed -> 5 =="
printf 'status: maybe\n' > "${STATUS}"
run "malformed (bad value)" 5 -- bash "${SH}" F 4 "${ART}"
printf 'status: enabled\nextra: line\n' > "${STATUS}"
run "malformed (extra line)" 5 -- bash "${SH}" F 4 "${ART}"

echo "== AC-6 enabled + missing key -> 6 (no network) =="
printf 'status: enabled\n' > "${STATUS}"
run "missing key refuses pre-network" 6 -- env -u DEEPSEEK_API_KEY bash "${SH}" F 4 "${ART}"

echo "== AC-7 enabled + key set + missing artifact -> 7 (no network) =="
run "missing artifact refuses pre-network" 7 -- env DEEPSEEK_API_KEY=dummy bash "${SH}" F 4 "${REPO}/does-not-exist.md"

# restore shipped state
printf '%s' "${orig}" > "${STATUS}"
echo
echo "RESTORED status file to:"; cat "${STATUS}"
echo
echo "SUMMARY: PASS=${PASS} FAIL=${FAIL}"
[[ ${FAIL} == 0 ]]
```

Verbatim output:

```
== AC-5 git precondition (run from non-git dir) -> 1 ==
PASS [1] non-git repo
== AC-3 usage: too few args -> 3 ==
PASS [3] missing args
== AC-7(usage) bad stage -> 3 ==
PASS [3] stage=6 rejected
== AC-2 disabled -> 4 ==
PASS [4] status disabled refuses
== AC-1 absent -> 4 ==
PASS [4] absent status refuses
== AC-3 malformed -> 5 ==
PASS [5] malformed (bad value)
PASS [5] malformed (extra line)
== AC-6 enabled + missing key -> 6 (no network) ==
PASS [6] missing key refuses pre-network
== AC-7 enabled + key set + missing artifact -> 7 (no network) ==
PASS [7] missing artifact refuses pre-network

RESTORED status file to:
status: disabled
SUMMARY: PASS=9 FAIL=0
```

## 1b. Remaining exit-code cases (2 and 8) — triggered separately

These two need environment manipulation not portable in the suite above, so they are run directly.

Exit 2 — missing dependency (PATH containing only `git`+`bash`, no `curl`/`jq`):

```
$ PATH=<git+bash only> bash scripts/codeos-implement.sh counter 4 <intent> <contract> <schema>
error: required dependency 'curl' not found on PATH
# exit 2
```

Exit 8 — API/transport error (unreachable endpoint, mechanism enabled, dummy key):

```
$ CODEOS_DEEPSEEK_URL=http://127.0.0.1:1/nope DEEPSEEK_API_KEY=dummy \
    bash scripts/codeos-implement.sh counter 4 <intent> <contract> <schema>
curl: (7) Failed to connect to 127.0.0.1 port 1 ... Could not connect to server
error: DeepSeek API call failed (http_code=000); response saved to .codeos-state/.../response.json
# exit 8
```

Together with §1, all documented exit codes 1–8 are demonstrated (plus 0 = success, shown by the
pilot in §3).

## 2. Stage-area path enforcement (the "strict path" contract)

The tool enforces the stage area (Stage 4 -> `modules/`, Stage 5 -> `tests/`), rejecting absolute,
`..`, and out-of-area paths (except the `CANDIDATE_BLOCKED.md` escape hatch):

```bash
  # Enforce the stage-area allowlist, except for the documented CANDIDATE_BLOCKED.md escape hatch
  # the model uses to report insufficient artifacts.
  if [[ "${fpath}" != "CANDIDATE_BLOCKED.md" && "${fpath}" != "${ALLOWED_PREFIX}"* ]]; then
    err "candidate path outside the Stage ${STAGE} area (must start with '${ALLOWED_PREFIX}'): ${fpath}"
    exit 8
  fi
  outfile="${STAGE_DIR}/candidate/${fpath}"
```

The pilot below exercises the allow case: the produced candidate path is under `modules/` and accepted.

## 3. Stage-4 pilot (AC-9..12)

Tool output:

```
candidate staged: /home/rimo/projects/Codeos/.codeos-state/deepseek-candidates/counter-stage-4/20260802T185925Z.0vWWIP/candidate  (1 file(s))
  tokens: prompt=1408 completion=455 total=1863   model=deepseek-chat
  audit:  packet=/home/rimo/projects/Codeos/.codeos-state/deepseek-candidates/counter-stage-4/20260802T185925Z.0vWWIP/packet.txt  response=/home/rimo/projects/Codeos/.codeos-state/deepseek-candidates/counter-stage-4/20260802T185925Z.0vWWIP/response.json  log=/home/rimo/projects/Codeos/.codeos-state/deepseek-candidates/implement-log.md
  NOTE: candidate only — promote manually; the Stage 4 human gate + advisory review still apply.
```

- Produced candidate (exactly one file): `modules/counter.py`
- Token usage: `prompt_tokens=1408 completion_tokens=455 total_tokens=1863 model=deepseek-chat`
- `python3 -m py_compile`: OK
- Model `notes`: Chose a simple list to store emitted events; no external event bus specified. correlation_id is passed as a parameter to increment(). No other events emitted.

Candidate body:

```python
"""Bounded counter implementation."""

class Counter:
    """A counter that starts at 0 and can be incremented up to 100."""

    MAX_VALUE = 100

    def __init__(self):
        self.value = 0
        self.events = []

    def increment(self, correlation_id):
        """Increment the counter by 1 if below the maximum.

        Emits CounterIncremented on success, IncrementRejected on failure.
        """
        if self.value < self.MAX_VALUE:
            self.value += 1
            self.events.append({
                "type": "CounterIncremented",
                "new_value": self.value,
                "correlation_id": correlation_id
            })
        else:
            self.events.append({
                "type": "IncrementRejected",
                "reason": "at_maximum",
                "correlation_id": correlation_id
            })
```

- Secret non-leakage (AC-10): PASS — key value absent from packet/response/candidate/log
- Write-safety (AC-9): PASS — no writes under modules/ or tests/; no candidate promoted
- Idempotency (AC-12): 5 distinct, non-overwriting staging dirs and 5 log records to date
  (the `mktemp` suffix makes same-second re-runs distinct).

## 4. AC-15 gate check — advisory review of THIS candidate

Ran the existing advisory gate on the candidate above against its approved artifacts:

```
codeos-review.sh review UPG-0060-pilot-counter pilot-candidate modules/counter.py <intent> <contract> <schema>
```

Result: **codex concern: CHANGES ADVISED   **. LOG SUMMARY: CHANGES ADVISED — The implementation does not match the approved `increment()` interface and can emit schema-invalid events.

Findings on the candidate:

```
severity: High
summary: "`increment()` requires an extra argument that the approved artifacts do not specify"
severity: High
summary: "Emitted events are not guaranteed to match the approved schema"
```

Full assessment: `reviews/codex/2026-08-02T190039Z-UPG-0060-pilot-counter-stage-pilot-candidate-0b3c07c.md`.

This is the intended outcome: DeepSeek produced a plausible draft cheaply, and the existing advisory
review flagged issues before any human approval. The candidate stays a candidate; the gates are
unchanged. AC-15's "and/or codeos-review.sh review" quality check is satisfied by this gate run.

## 5. Net-token verdict (AC-15)

The mechanism works end-to-end: it produced a plausible Stage-4 draft for the token count above, and
the existing advisory gate then reviewed it (CHANGES ADVISED    — real issues caught
before human approval). On this toy feature the absolute Claude saving is small — there is little
generation to offload — so the pilot demonstrates **viability and correctness**, not a headline saving.
The saving concentrates on realistic features (hundreds of lines) where DeepSeek's far lower per-token
price applies to the bulk generation while Claude keeps approval, reconciliation, and review. Verdict:
**mechanism viable; CHG-B (doctrine wiring) warranted, conditioned on a net-token measurement against a
realistic downstream feature** — the abandonment path stays open if that measurement is not
net-positive.
