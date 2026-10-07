# Gemini/Laya review-triage pilot

This directory implements the gated experiment described in
[`codeos-jev-capability-analysis.md`](../../codeos-jev-capability-analysis.md). It is advisory and
cannot update a CodeOS review, decision, or workflow state.

The experiment deliberately stops before Laya. Phase 0 must first establish both a material human
classification workload and a usable corpus; direct Gemini must then pass its locked shadow
evaluation. Laya v1.9.3 (`bbd6c799a255ff7eee1731effe1d23ec78538de9`) may be added only as a
separate, byte-identical arm after those gates pass.

## Reproduce Phase 0

Requires Python 3.11+ and PyYAML. Commands write only beneath the output directory supplied by the
caller.

```bash
cd maintenance/reviews/experiments/laya-gemini-triage-pilot
python3 pilot.py census \
  --reviews ../../../archive/self-development/reviews/codex \
  --output runs/phase0
cp workload-observations.example.csv runs/phase0/workload-observations.csv
# Replace the example rows with blinded observations.
python3 pilot.py phase0 \
  --census runs/phase0/census.json \
  --workload runs/phase0/workload-observations.csv \
  --output runs/phase0/phase0-decision.json
```

`phase0` returns one of `FULL_H1`, `NARROW_H1`, `NO_MATERIAL_CLASSIFICATION_BOTTLENECK`, or
`INSUFFICIENT_EVALUATION_DATA`. It never calls a model. Two independently produced gold-label
files are required before a run can be frozen.

```bash
python3 pilot.py freeze \
  --phase0 runs/phase0/phase0-decision.json \
  --cases cases.jsonl --labels-a labels-a.jsonl --labels-b labels-b.jsonl \
  --output runs/frozen
```

The freeze command refuses label disagreements. Resolve them in a separately retained adjudication
file, then pass the agreed files. It creates a hash manifest and immutable run configuration.
Case files must not contain gold, historical-label, or confidence fields; the freeze command rejects
them as leakage. The runner sends only the bounded fields declared in `pilot.py`.
Before freezing, replace the explicitly marked placeholder confidence operating point in
`protocol.json` using calibration-set evidence. Locked-set results must never be used for this.

## Direct Gemini shadow arm

The runner uses Gemini's REST API directly, omits deprecated sampling parameters, validates every
structured response locally, retries once, and fails closed. It logs hashes, model version, latency,
tokens, retry state, and estimated cost; it does not log the API key.

```bash
GEMINI_API_KEY=... python3 pilot.py run-direct \
  --frozen runs/frozen --output runs/direct/results.jsonl
# On the predeclared 12-case stability subset, use --repetitions 4.
python3 pilot.py evaluate \
  --frozen runs/frozen --results runs/direct/results.jsonl \
  --output runs/direct/evaluation.json
```

The hard direct-arm limits are 150 requests, 600,000 input tokens, 75,000 output tokens, and $4.
Retries count. The runner checks the persisted ledger before every request, so rerunning cannot
reset a budget. The pricing inputs in `protocol.json` are an accounting snapshot and must be
re-frozen if changed.

Validated results are append-only JSONL. Each provider envelope is retained beneath `raw/` and
bound to its result by SHA-256; API credentials are never written.

`evaluate` reports class metrics, error severity, confidence bands, automatic coverage for all
three abstention strategies, and repeatability where repeated case IDs exist. It uses the required
conclusion wording: zero observed critical errors supports continued shadow evaluation only.

## Experimental boundaries

- Historical Codex labels are an external comparator, never gold truth.
- Diagnostic results do not estimate operating economics.
- Missing classes remain `INSUFFICIENT EVIDENCE`; examples are never manufactured or merged.
- The natural-distribution collection is capped at 40 consecutive findings or eight weeks.
- No output authorizes autonomous routing or production adoption.
- Action Cards remain a later 20-case usability smoke test, not adoption evidence.

If H1 passes, copy `laya-capability-ledger.example.csv` into the new Laya run. Every claimed Laya
advantage must be classified as `HARD/VALUABLE`, `SMALL CONVENIENCE`, or
`ALREADY BUILT BY BASELINE`. H2 cannot pass without a net-positive `HARD/VALUABLE` capability.
Use `action-card-observations.example.csv` only after H2 passes.

Run tests with `python3 -m unittest discover -s tests -v`.
