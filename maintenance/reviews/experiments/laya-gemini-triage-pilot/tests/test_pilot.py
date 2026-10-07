import csv
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

HERE = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("triage_pilot", HERE / "pilot.py")
pilot = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pilot)


class PilotTests(unittest.TestCase):
    def test_validate_response_rejects_contradictory_blocker_kind(self):
        protocol = pilot.load_json(HERE / "protocol.json")
        value = {
            "classification": "REJECTED",
            "blocker_kind": "substantive",
            "abstain": False,
            "confidence": 0.9,
            "reason": "x",
        }
        self.assertIn("non-blocker has an applicable blocker_kind", pilot.validate_response(value, protocol))

    def test_budget_is_fail_closed_at_limit(self):
        with self.assertRaises(pilot.PilotError):
            pilot.ensure_budget(
                {"requests": 150, "input_tokens": 0, "output_tokens": 0, "estimated_usd": 0},
                {"requests": 150, "input_tokens": 600000, "output_tokens": 75000, "estimated_usd": 4},
            )

    def test_phase0_selects_narrow_h1(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            census = root / "census.json"
            workload = root / "workload.csv"
            output = root / "decision.json"
            pilot.write_json(census, {
                "canonical_findings": 29,
                "counts": {"IN-SCOPE BLOCKER": 20, "IN-SCOPE NON-BLOCKER": 9},
            })
            with workload.open("w", newline="", encoding="utf-8") as stream:
                writer = csv.DictWriter(stream, fieldnames=[
                    "decision_id", "observed_at", "classification_checked", "classification_minutes",
                    "classification_corrected", "monthly_weight", "notes",
                ])
                writer.writeheader()
                for index in range(10):
                    writer.writerow({"decision_id": str(index), "classification_checked": "true",
                                     "classification_minutes": "15", "classification_corrected": "false",
                                     "monthly_weight": "1"})
            rc = pilot.command_phase0(SimpleNamespace(census=census, workload=workload, output=output))
            self.assertEqual(0, rc)
            self.assertEqual("NARROW_H1", pilot.load_json(output)["decision"])

    def test_freeze_refuses_disagreement(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            phase0 = root / "phase0.json"
            cases = root / "cases.jsonl"
            first = root / "a.jsonl"
            second = root / "b.jsonl"
            pilot.write_json(phase0, {"decision": "NARROW_H1"})
            cases.write_text('{"case_id":"C1","summary":"x"}\n')
            first.write_text('{"case_id":"C1","gold_label":"IN-SCOPE BLOCKER"}\n')
            second.write_text('{"case_id":"C1","gold_label":"REJECTED"}\n')
            with self.assertRaises(pilot.PilotError):
                pilot.command_freeze(SimpleNamespace(
                    phase0=phase0, cases=cases, labels_a=first, labels_b=second, output=root / "frozen"
                ))

    def test_census_reads_only_frontmatter_and_deduplicates(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reviews = root / "reviews"
            reviews.mkdir()
            reviews.joinpath("one.md").write_text(
                "---\nreview_id: R1\nfindings:\n  - finding_id: F1\n    classification: IN-SCOPE BLOCKER\n"
                "    summary: Same issue\nreviewed:\n  feature: U1\n---\nClassification: REJECTED\n"
            )
            reviews.joinpath("two.md").write_text(
                "---\nreview_id: R2\nfindings:\n  - finding_id: F2\n    classification: IN-SCOPE BLOCKER\n"
                "    summary: same   issue\nreviewed:\n  feature: U1\n---\n"
            )
            output = root / "out"
            pilot.command_census(SimpleNamespace(reviews=reviews, output=output))
            report = pilot.load_json(output / "census.json")
            self.assertEqual(1, report["canonical_findings"])
            self.assertEqual(1, report["exact_duplicates_removed"])


if __name__ == "__main__":
    unittest.main()
