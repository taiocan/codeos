#!/usr/bin/env python3
"""Fail-closed tooling for the CodeOS Gemini/Laya triage pilot."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable

import yaml

ROOT = Path(__file__).resolve().parent
PROTOCOL = ROOT / "protocol.json"
PROMPT = ROOT / "prompt.txt"
SCHEMA = ROOT / "response-schema.json"
BLOCKER = "IN-SCOPE BLOCKER"
TERMINAL_PHASE0 = {
    "FULL_H1",
    "NARROW_H1",
    "NO_MATERIAL_CLASSIFICATION_BOTTLENECK",
    "INSUFFICIENT_EVALUATION_DATA",
}
MODEL_CASE_FIELDS = {
    "case_id", "summary", "evidence", "why", "required_action", "scope_reason",
    "feature", "stage", "artifact_role", "approved_scope",
}
FORBIDDEN_CASE_FIELDS = {"gold_label", "classification", "historical_codex_label", "confidence"}


class PilotError(RuntimeError):
    pass


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def digest_file(path: Path) -> str:
    return digest_bytes(path.read_bytes())


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    rows = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as exc:
            raise PilotError(f"{path}:{number}: invalid JSON: {exc}") from exc
        if not isinstance(row, dict):
            raise PilotError(f"{path}:{number}: expected an object")
        rows.append(row)
    return rows


def append_jsonl(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as stream:
        stream.write(canonical(value).decode() + "\n")
        stream.flush()
        os.fsync(stream.fileno())


def frontmatter(path: Path) -> dict[str, Any] | None:
    text = path.read_text(encoding="utf-8", errors="replace")
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    try:
        value = yaml.safe_load(text[4:end])
    except yaml.YAMLError:
        return None
    return value if isinstance(value, dict) else None


def normalized_summary(value: str) -> str:
    return " ".join(value.casefold().split())


def command_census(args: argparse.Namespace) -> int:
    protocol = load_json(PROTOCOL)
    allowed = set(protocol["labels"])
    rows: list[dict[str, Any]] = []
    invalid = []
    duplicates = []
    seen: dict[str, str] = {}
    files = sorted(args.reviews.glob("*.md"))
    for path in files:
        metadata = frontmatter(path)
        if not metadata:
            continue
        reviewed = metadata.get("reviewed") or {}
        feature = reviewed.get("feature") or metadata.get("review_id") or path.stem
        stage = reviewed.get("stage")
        findings = metadata.get("findings") or []
        if not isinstance(findings, list):
            invalid.append({"path": str(path), "reason": "findings is not a list"})
            continue
        for finding in findings:
            if not isinstance(finding, dict):
                continue
            label = finding.get("classification")
            summary = finding.get("summary")
            finding_id = finding.get("finding_id")
            if label not in allowed or not isinstance(summary, str) or not finding_id:
                invalid.append({"path": str(path), "finding": finding})
                continue
            exact_key = digest_bytes(normalized_summary(summary).encode())
            if exact_key in seen:
                duplicates.append({"finding_id": finding_id, "duplicate_of": seen[exact_key]})
                continue
            seen[exact_key] = finding_id
            rows.append({
                "case_id": finding_id,
                "group_id": str(feature),
                "stage": stage,
                "historical_codex_label": label,
                "summary": summary,
                "severity": finding.get("severity"),
                "required_action": finding.get("required_action"),
                "source_path": str(path),
                "source_sha256": digest_file(path),
            })
    counts = Counter(row["historical_codex_label"] for row in rows)
    output = args.output
    output.mkdir(parents=True, exist_ok=True)
    with (output / "corpus.jsonl").open("w", encoding="utf-8") as stream:
        for row in rows:
            stream.write(canonical(row).decode() + "\n")
    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source": str(args.reviews.resolve()),
        "source_files": len(files),
        "canonical_findings": len(rows),
        "counts": {label: counts.get(label, 0) for label in protocol["labels"]},
        "exact_duplicates_removed": len(duplicates),
        "invalid_records": len(invalid),
        "diagnostic_capacity_is_not_gold_truth": True,
        "corpus_sha256": digest_file(output / "corpus.jsonl"),
    }
    write_json(output / "census.json", report)
    write_json(output / "excluded.json", {"duplicates": duplicates, "invalid": invalid})
    print(json.dumps(report, indent=2))
    return 0


def truth(value: str) -> bool:
    return value.strip().casefold() in {"true", "1", "yes"}


def command_phase0(args: argparse.Namespace) -> int:
    protocol = load_json(PROTOCOL)
    rules = protocol["phase0"]
    census = load_json(args.census)
    with args.workload.open(newline="", encoding="utf-8") as stream:
        observations = list(csv.DictReader(stream))
    if any(row.get("decision_id", "").startswith("EXAMPLE-") for row in observations):
        raise PilotError("remove all example workload rows before evaluating Phase 0")
    if not observations:
        raise PilotError("at least one workload observation is required")
    checked = sum(truth(row.get("classification_checked", "")) for row in observations)
    corrections = sum(truth(row.get("classification_corrected", "")) for row in observations)
    minutes = sum(float(row.get("classification_minutes") or 0) for row in observations)
    weights = sum(float(row.get("monthly_weight") or 0) for row in observations)
    weighted_hours = sum(
        float(row.get("classification_minutes") or 0) * float(row.get("monthly_weight") or 0)
        for row in observations
    ) / 60
    weighted_corrections = sum(
        truth(row.get("classification_corrected", "")) * float(row.get("monthly_weight") or 0)
        for row in observations
    )
    occurrence_rate = checked / len(observations)
    enough_observations = len(observations) >= rules["minimum_observed_decisions"]
    material = (
        enough_observations
        and occurrence_rate >= rules["minimum_occurrence_rate"]
        and (
            weighted_hours >= rules["minimum_projected_hours_per_month"]
            or weighted_corrections >= rules["minimum_corrections_per_month"]
        )
    )
    counts = census["counts"]
    blockers = counts.get(BLOCKER, 0)
    non_blockers = census["canonical_findings"] - blockers
    if not enough_observations:
        decision = "INSUFFICIENT_EVALUATION_DATA"
        reason = "fewer than 10 workload observations"
    elif not material:
        decision = "NO_MATERIAL_CLASSIFICATION_BOTTLENECK"
        reason = "measured workload did not pass the occurrence and value gate"
    elif blockers >= rules["minimum_diagnostic_blockers"] and non_blockers >= rules["minimum_diagnostic_non_blockers"]:
        decision = "FULL_H1"
        reason = "workload and binary diagnostic corpus gates passed"
    elif blockers >= rules["minimum_diagnostic_blockers"]:
        decision = "NARROW_H1"
        reason = "workload passed, but only obvious-blocker confirmation is supportable"
    else:
        decision = "INSUFFICIENT_EVALUATION_DATA"
        reason = "too few real canonical findings for either H1 form"
    result = {
        "decision": decision,
        "reason": reason,
        "observations": len(observations),
        "classification_checked": checked,
        "occurrence_rate": occurrence_rate,
        "observed_minutes": minutes,
        "projected_hours_per_month": weighted_hours,
        "projected_corrections_per_month": weighted_corrections,
        "monthly_weight_total": weights,
        "canonical_blockers": blockers,
        "canonical_non_blockers": non_blockers,
        "census_sha256": digest_file(args.census),
        "workload_sha256": digest_file(args.workload),
    }
    write_json(args.output, result)
    print(json.dumps(result, indent=2))
    return 0


def labels_by_case(path: Path) -> dict[str, str]:
    result = {}
    for row in read_jsonl(path):
        case_id, label = row.get("case_id"), row.get("gold_label")
        if not case_id or not label:
            raise PilotError(f"{path}: each row needs case_id and gold_label")
        if case_id in result:
            raise PilotError(f"{path}: duplicate case_id {case_id}")
        result[case_id] = label
    return result


def command_freeze(args: argparse.Namespace) -> int:
    phase0 = load_json(args.phase0)
    if phase0.get("decision") not in {"FULL_H1", "NARROW_H1"}:
        raise PilotError(f"Phase 0 does not permit inference: {phase0.get('decision')}")
    cases = read_jsonl(args.cases)
    case_ids = [row.get("case_id") for row in cases]
    if not case_ids or None in case_ids or len(set(case_ids)) != len(case_ids):
        raise PilotError("cases must have unique, non-empty case_id values")
    if any(not isinstance(case_id, str) or not re.fullmatch(r"[A-Za-z0-9_.-]+", case_id) for case_id in case_ids):
        raise PilotError("case_id may contain only letters, digits, underscore, period, and hyphen")
    leaked = sorted({key for row in cases for key in row if key in FORBIDDEN_CASE_FIELDS})
    if leaked:
        raise PilotError(f"cases contain label leakage fields: {leaked}")
    if any(not isinstance(row.get("summary"), str) or not row["summary"].strip() for row in cases):
        raise PilotError("each case needs a non-empty summary")
    labels_a, labels_b = labels_by_case(args.labels_a), labels_by_case(args.labels_b)
    if set(case_ids) != set(labels_a) or set(case_ids) != set(labels_b):
        raise PilotError("both label files must cover exactly the frozen cases")
    disagreements = [case_id for case_id in case_ids if labels_a[case_id] != labels_b[case_id]]
    if disagreements:
        raise PilotError(f"unadjudicated label disagreements: {', '.join(disagreements[:10])}")
    protocol = load_json(PROTOCOL)
    if protocol["calibrated_operating_point"]["status"].startswith("placeholder"):
        raise PilotError("calibrate and freeze the confidence operating point before inference")
    unknown = sorted(set(labels_a.values()) - set(protocol["labels"]))
    if unknown:
        raise PilotError(f"unknown gold labels: {unknown}")
    args.output.mkdir(parents=True, exist_ok=False)
    files = {
        "phase0.json": args.phase0,
        "cases.jsonl": args.cases,
        "labels-a.jsonl": args.labels_a,
        "labels-b.jsonl": args.labels_b,
        "protocol.json": PROTOCOL,
        "prompt.txt": PROMPT,
        "response-schema.json": SCHEMA,
    }
    for name, source in files.items():
        (args.output / name).write_bytes(source.read_bytes())
    manifest = {
        "frozen_at": datetime.now(timezone.utc).isoformat(),
        "phase0_decision": phase0["decision"],
        "case_count": len(cases),
        "files": {name: digest_file(args.output / name) for name in files},
    }
    write_json(args.output / "manifest.json", manifest)
    print(json.dumps(manifest, indent=2))
    return 0


def verify_frozen(directory: Path) -> dict[str, Any]:
    manifest = load_json(directory / "manifest.json")
    for name, expected in manifest["files"].items():
        actual = digest_file(directory / name)
        if actual != expected:
            raise PilotError(f"frozen file changed: {name}")
    return manifest


def validate_response(value: Any, protocol: dict[str, Any]) -> list[str]:
    errors = []
    required = {"classification", "blocker_kind", "abstain", "confidence", "reason"}
    if not isinstance(value, dict):
        return ["response is not an object"]
    if set(value) != required:
        errors.append("response fields do not exactly match the schema")
    if value.get("classification") not in protocol["labels"]:
        errors.append("invalid classification")
    if value.get("blocker_kind") not in protocol["blocker_kinds"]:
        errors.append("invalid blocker_kind")
    if not isinstance(value.get("abstain"), bool):
        errors.append("abstain is not boolean")
    confidence = value.get("confidence")
    if isinstance(confidence, bool) or not isinstance(confidence, (int, float)) or not 0 <= confidence <= 1:
        errors.append("confidence is outside [0,1]")
    if not isinstance(value.get("reason"), str) or len(value.get("reason", "")) > 500:
        errors.append("reason is invalid")
    if value.get("classification") != BLOCKER and value.get("blocker_kind") != "not_applicable":
        errors.append("non-blocker has an applicable blocker_kind")
    return errors


def ledger_totals(path: Path) -> dict[str, float]:
    totals = {"requests": 0, "input_tokens": 0, "output_tokens": 0, "estimated_usd": 0.0}
    if not path.exists():
        return totals
    for row in read_jsonl(path):
        for key in totals:
            totals[key] += row.get(key, 0)
    return totals


def ensure_budget(totals: dict[str, float], limits: dict[str, Any]) -> None:
    exceeded = [key for key in totals if totals[key] >= limits[key]]
    if exceeded:
        raise PilotError(f"budget exhausted: {', '.join(exceeded)}")


def gemini_request(api_key: str, model: str, prompt: str, case: dict[str, Any], schema: dict[str, Any]) -> tuple[dict[str, Any], bytes, float]:
    body = {
        "contents": [{"role": "user", "parts": [{"text": prompt + "\n\nCASE:\n" + canonical(case).decode()}]}],
        "generationConfig": {"responseMimeType": "application/json", "responseJsonSchema": schema},
    }
    encoded = canonical(body)
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
    request = urllib.request.Request(url, data=encoded, headers={"Content-Type": "application/json", "x-goog-api-key": api_key})
    started = time.monotonic()
    with urllib.request.urlopen(request, timeout=60) as response:
        raw = response.read()
    return json.loads(raw), encoded, (time.monotonic() - started) * 1000


def parse_gemini(envelope: dict[str, Any]) -> tuple[dict[str, Any], int, int, str]:
    text = envelope["candidates"][0]["content"]["parts"][0]["text"]
    usage = envelope.get("usageMetadata", {})
    return (
        json.loads(text),
        int(usage.get("promptTokenCount", 0)),
        int(usage.get("candidatesTokenCount", 0)) + int(usage.get("thoughtsTokenCount", 0)),
        envelope.get("modelVersion", "unknown"),
    )


def command_run_direct(args: argparse.Namespace) -> int:
    verify_frozen(args.frozen)
    protocol = load_json(args.frozen / "protocol.json")
    cases = read_jsonl(args.frozen / "cases.jsonl")
    prompt = (args.frozen / "prompt.txt").read_text(encoding="utf-8")
    schema = load_json(args.frozen / "response-schema.json")
    api_key = os.environ.get("GEMINI_API_KEY")
    if not api_key:
        raise PilotError("GEMINI_API_KEY is not set")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    ledger = args.output.with_name("budget-ledger.jsonl")
    completed = Counter(row["case_id"] for row in read_jsonl(args.output)) if args.output.exists() else Counter()
    prices = protocol["pricing_usd_per_million_tokens"]
    for case in cases:
        case_id = case["case_id"]
        model_case = {key: value for key, value in case.items() if key in MODEL_CASE_FIELDS}
        while completed[case_id] < args.repetitions:
            final_record = None
            run_index = completed[case_id] + 1
            for attempt in range(protocol["limits"]["retries_per_case"] + 1):
                totals = ledger_totals(ledger)
                ensure_budget(totals, protocol["limits"])
                status, errors, envelope, request_bytes, latency = "failed", [], None, b"", 0.0
                input_tokens = output_tokens = 0
                model_version = "unknown"
                prediction = None
                try:
                    envelope, request_bytes, latency = gemini_request(
                        api_key, protocol["model"], prompt, model_case, schema
                    )
                    prediction, input_tokens, output_tokens, model_version = parse_gemini(envelope)
                    errors = validate_response(prediction, protocol)
                    status = "valid" if not errors else "invalid"
                except (urllib.error.URLError, TimeoutError, KeyError, IndexError, json.JSONDecodeError) as exc:
                    errors = [f"{type(exc).__name__}: {exc}"]
                estimated = (input_tokens * prices["input"] + output_tokens * prices["output"]) / 1_000_000
                after = {
                    **totals,
                    "requests": totals["requests"] + 1,
                    "input_tokens": totals["input_tokens"] + input_tokens,
                    "output_tokens": totals["output_tokens"] + output_tokens,
                    "estimated_usd": totals["estimated_usd"] + estimated,
                }
                append_jsonl(ledger, {
                    "case_id": case_id, "attempt": attempt + 1, "requests": 1,
                    "input_tokens": input_tokens, "output_tokens": output_tokens,
                    "estimated_usd": estimated,
                })
                envelope_path = None
                if envelope is not None:
                    envelope_path = args.output.parent / "raw" / f"{case_id}-run-{run_index}-attempt-{attempt + 1}.json"
                    write_json(envelope_path, envelope)
                if any(after[key] > protocol["limits"][key] for key in after):
                    raise PilotError(
                        "response crossed a hard inference budget; usage was recorded and result was not accepted"
                    )
                final_record = {
                    "case_id": case_id,
                    "run_index": run_index,
                    "arm": "direct-gemini",
                    "attempts": attempt + 1,
                    "status": status,
                    "prediction": prediction if status == "valid" else None,
                    "errors": errors,
                    "model_requested": protocol["model"],
                    "model_version": model_version,
                    "latency_ms": round(latency, 3),
                    "input_tokens": input_tokens,
                    "output_tokens": output_tokens,
                    "estimated_usd": estimated,
                    "case_sha256": digest_bytes(canonical(model_case)),
                    "request_sha256": digest_bytes(request_bytes) if request_bytes else None,
                    "response_sha256": digest_bytes(canonical(envelope)) if envelope else None,
                    "response_envelope": str(envelope_path.relative_to(args.output.parent)) if envelope_path else None,
                    "recorded_at": datetime.now(timezone.utc).isoformat(),
                }
                if status == "valid":
                    break
            append_jsonl(args.output, final_record)
            completed[case_id] += 1
    return 0


def severity(gold: str, predicted: str, automated: bool) -> str:
    if automated and gold == BLOCKER and predicted != BLOCKER:
        return "Critical"
    if gold == BLOCKER and predicted != BLOCKER:
        return "High"
    if gold != predicted:
        return "Medium"
    return "None"


def command_evaluate(args: argparse.Namespace) -> int:
    verify_frozen(args.frozen)
    protocol = load_json(args.frozen / "protocol.json")
    gold = labels_by_case(args.frozen / "labels-a.jsonl")
    results = read_jsonl(args.results)
    by_case: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in results:
        if row.get("case_id") in gold:
            by_case[row["case_id"]].append(row)
    policies = ("reported_confidence", "deterministic_uncertainty", "class_restricted")
    policy_reports = {}
    for name in policies:
        assessed = Counter()
        severity_counts = Counter()
        for case_id, runs in by_case.items():
            row = next((run for run in runs if run.get("status") == "valid"), None)
            if row is None:
                assessed["operational_failures"] += 1
                continue
            prediction = row["prediction"]
            valid_labels = [
                run["prediction"]["classification"]
                for run in runs if run.get("status") == "valid"
            ]
            if name == "reported_confidence":
                threshold = protocol["calibrated_operating_point"]["reported_confidence_threshold"]
                automatic = not prediction["abstain"] and prediction["confidence"] >= threshold
            elif name == "deterministic_uncertainty":
                automatic = (
                    not prediction["abstain"] and row["attempts"] == 1
                    and len(valid_labels) >= 2 and len(set(valid_labels)) == 1
                )
            else:
                automatic = not prediction["abstain"] and prediction["classification"] == BLOCKER
            correct = prediction["classification"] == gold[case_id]
            assessed["automatic" if automatic else "escalated"] += 1
            assessed["correct" if correct else "incorrect"] += 1
            severity_counts[severity(gold[case_id], prediction["classification"], automatic)] += 1
        total = assessed["automatic"] + assessed["escalated"]
        policy_reports[name] = {
            **assessed,
            "automatic_coverage": assessed["automatic"] / total if total else None,
            "errors_by_severity": dict(severity_counts),
        }
    class_counts = Counter(gold.values())
    insufficient = [label for label in protocol["labels"] if class_counts[label] < 20]
    first_valid = {
        case_id: next((run for run in runs if run.get("status") == "valid"), None)
        for case_id, runs in by_case.items()
    }
    binary = Counter()
    confidence_bands: dict[str, Counter] = defaultdict(Counter)
    stability = {}
    for case_id, row in first_valid.items():
        if not row:
            continue
        prediction = row["prediction"]
        actual_blocker = gold[case_id] == BLOCKER
        predicted_blocker = prediction["classification"] == BLOCKER
        binary[("actual_blocker" if actual_blocker else "actual_non_blocker") + ("_correct" if actual_blocker == predicted_blocker else "_wrong")] += 1
        confidence = prediction["confidence"]
        band = "0.0-0.59" if confidence < 0.6 else "0.6-0.79" if confidence < 0.8 else "0.8-1.0"
        confidence_bands[band]["total"] += 1
        confidence_bands[band]["correct" if prediction["classification"] == gold[case_id] else "incorrect"] += 1
        labels = [run["prediction"]["classification"] for run in by_case[case_id] if run.get("status") == "valid"]
        if len(labels) > 1:
            stability[case_id] = {"runs": len(labels), "stable": len(set(labels)) == 1, "labels": labels}
    blocker_total = binary["actual_blocker_correct"] + binary["actual_blocker_wrong"]
    non_blocker_total = binary["actual_non_blocker_correct"] + binary["actual_non_blocker_wrong"]
    blocker_recall = binary["actual_blocker_correct"] / blocker_total if blocker_total else None
    specificity = binary["actual_non_blocker_correct"] / non_blocker_total if non_blocker_total else None
    balanced_accuracy = (blocker_recall + specificity) / 2 if blocker_recall is not None and specificity is not None and non_blocker_total >= 20 else None
    critical = sum(report["errors_by_severity"].get("Critical", 0) for report in policy_reports.values())
    conclusion = (
        "No critical errors were observed in this bounded pilot; results may justify continued shadow evaluation, not autonomous production routing."
        if critical == 0
        else "Critical errors were observed; no automation is justified."
    )
    report = {
        "cases_with_results": len(by_case),
        "gold_class_counts": dict(class_counts),
        "insufficient_evidence_classes": insufficient,
        "policies": policy_reports,
        "binary_metrics": {
            "blocker_recall": blocker_recall,
            "non_blocker_specificity": specificity,
            "balanced_accuracy": balanced_accuracy,
            "balanced_accuracy_status": "MEASURED" if balanced_accuracy is not None else "INSUFFICIENT EVIDENCE",
        },
        "confidence_bands": {band: dict(counts) for band, counts in sorted(confidence_bands.items())},
        "repeatability": stability,
        "conclusion": conclusion,
        "historical_codex_is_external_only": True,
    }
    write_json(args.output, report)
    print(json.dumps(report, indent=2))
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    sub = result.add_subparsers(dest="command", required=True)
    census = sub.add_parser("census")
    census.add_argument("--reviews", type=Path, required=True)
    census.add_argument("--output", type=Path, required=True)
    census.set_defaults(func=command_census)
    phase0 = sub.add_parser("phase0")
    phase0.add_argument("--census", type=Path, required=True)
    phase0.add_argument("--workload", type=Path, required=True)
    phase0.add_argument("--output", type=Path, required=True)
    phase0.set_defaults(func=command_phase0)
    freeze = sub.add_parser("freeze")
    freeze.add_argument("--phase0", type=Path, required=True)
    freeze.add_argument("--cases", type=Path, required=True)
    freeze.add_argument("--labels-a", type=Path, required=True)
    freeze.add_argument("--labels-b", type=Path, required=True)
    freeze.add_argument("--output", type=Path, required=True)
    freeze.set_defaults(func=command_freeze)
    direct = sub.add_parser("run-direct")
    direct.add_argument("--frozen", type=Path, required=True)
    direct.add_argument("--output", type=Path, required=True)
    direct.add_argument("--repetitions", type=int, choices=range(1, 5), default=1,
                        help="total identical calls per case, including an existing result (max 4)")
    direct.set_defaults(func=command_run_direct)
    evaluate = sub.add_parser("evaluate")
    evaluate.add_argument("--frozen", type=Path, required=True)
    evaluate.add_argument("--results", type=Path, required=True)
    evaluate.add_argument("--output", type=Path, required=True)
    evaluate.set_defaults(func=command_evaluate)
    return result


def main() -> int:
    try:
        args = parser().parse_args()
        return args.func(args)
    except (PilotError, OSError, ValueError) as exc:
        print(f"pilot stopped: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
