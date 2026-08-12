#!/usr/bin/env python3
"""One-time conversion from legacy cohort architecture to one Architecture Scope."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

try:
    import yaml
except ModuleNotFoundError:
    print("migration stopped: PyYAML is required (python3 -m pip install PyYAML)", file=sys.stderr)
    raise SystemExit(2)


class MigrationError(Exception):
    pass


def fail(message: str) -> None:
    raise MigrationError(message)


def load_yaml(path: Path) -> dict:
    try:
        value = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        fail(f"cannot read valid YAML from {path}: {exc}")
    if not isinstance(value, dict):
        fail(f"expected a YAML mapping in {path}")
    return value


def field(text: str, key: str, table_name: str) -> str:
    matches = re.findall(rf"(?mi)^{re.escape(key)}:\s*([^\n#]+)", text)
    if matches:
        return matches[-1].strip()
    match = re.search(rf"(?mi)^\|\s*{re.escape(table_name)}\s*\|\s*([^|]+)\|", text)
    return match.group(1).strip() if match else ""


def membership(text: str) -> list[str]:
    marker = re.search(r"(?im)^\*\*Cohort membership set[^\n]*\*\*\s*$", text)
    if not marker:
        fail("legacy artifact has no cohort membership table")
    block = text[marker.end():]
    end = re.search(r"(?m)^(?:##\s|---\s*$)", block)
    if end:
        block = block[:end.start()]
    members = []
    for line in block.splitlines():
        match = re.match(r"^\|\s*([^|]+?)\s*\|", line)
        if not match:
            continue
        value = match.group(1).strip()
        if value.lower() not in {"feature id", "---"} and set(value) != {"-"}:
            members.append(value)
    if not members:
        fail("legacy artifact cohort membership table is empty")
    return members


def section(text: str, heading: str) -> str:
    match = re.search(rf"(?m)^## {re.escape(heading)}\s*$", text)
    if not match:
        return ""
    rest = text[match.end():]
    end = re.search(r"(?m)^##\s", rest)
    value = rest[:end.start()] if end else rest
    value = re.sub(r"<!--.*?-->", "", value, flags=re.S).strip()
    return value


def tracked(project: Path, path: Path) -> bool:
    result = subprocess.run(
        ["git", "-C", str(project), "ls-files", "--error-unmatch", str(path.relative_to(project))],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode == 0


def atomic_write(path: Path, content: str) -> None:
    temporary = path.with_name(f".{path.name}.migration-tmp")
    temporary.write_text(content, encoding="utf-8")
    temporary.replace(path)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project", nargs="?", default=".", help="downstream project root")
    parser.add_argument("--apply", action="store_true", help="perform conversion after validation")
    args = parser.parse_args()
    project = Path(args.project).resolve()

    if subprocess.run(
        ["git", "-C", str(project), "rev-parse", "--show-toplevel"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    ).returncode != 0:
        fail(f"not a Git repository: {project}")

    registry_path = project / "features/registry.yaml"
    baseline_path = project / "architecture/core-baseline.md"
    logical_path = project / "architecture/cohort-logical-design.md"
    for required in (registry_path, baseline_path, logical_path):
        if not required.is_file():
            fail(f"missing legacy input: {required.relative_to(project)}")
        if not tracked(project, required):
            fail(f"legacy input is not committed; commit it before migration: {required.relative_to(project)}")

    registry = load_yaml(registry_path)
    cohorts = registry.get("architecture_cohorts")
    if not isinstance(cohorts, list) or len(cohorts) != 1 or not isinstance(cohorts[0], dict):
        fail("migration requires exactly one legacy architecture_cohorts entry; singleton artifacts cannot be assigned safely otherwise")
    cohort = cohorts[0]
    scope_id = str(cohort.get("cohort_id", "")).strip()
    if not re.fullmatch(r"[a-z0-9][a-z0-9_-]*", scope_id):
        fail(f"invalid legacy cohort_id for a scope filename: {scope_id!r}")

    baseline = baseline_path.read_text(encoding="utf-8")
    logical = logical_path.read_text(encoding="utf-8")
    registry_members = cohort.get("member_features")
    if not isinstance(registry_members, list) or not registry_members:
        fail("legacy registry cohort has no member_features")
    registry_members = [str(item) for item in registry_members]

    feature_members = []
    for feature in registry.get("features", []):
        if not isinstance(feature, dict):
            fail("features registry contains a non-mapping entry")
        if feature.get("architecture_cohort") == scope_id:
            feature_members.append(str(feature.get("feature_id", "")))

    representations = {
        "registry cohort": registry_members,
        "feature entries": feature_members,
        "baseline": membership(baseline),
        "logical design": membership(logical),
    }
    expected = set(registry_members)
    for owner, members in representations.items():
        if len(members) != len(set(members)) or set(members) != expected:
            fail(f"ambiguous legacy membership in {owner}: {members!r}; expected {registry_members!r}")

    baseline_id = field(baseline, "cohort_id", "Cohort id")
    logical_id = field(logical, "cohort_id", "Cohort id")
    if baseline_id != scope_id or logical_id != scope_id:
        fail(f"cohort id conflict: registry={scope_id!r}, baseline={baseline_id!r}, logical={logical_id!r}")

    baseline_status = field(baseline, "status", "Status").lower()
    logical_status = field(logical, "status", "Status").lower()
    approved_pair = baseline_status == logical_status == "approved"
    if (baseline_status == "approved") != (logical_status == "approved"):
        fail("only one legacy architecture artifact is approved; human resolution is required")
    registry_status = str(cohort.get("status", "")).lower()
    if approved_pair != (registry_status == "approved"):
        fail(f"approval conflict: registry={registry_status!r}, baseline={baseline_status!r}, logical={logical_status!r}")

    approved_by = approved_at = ""
    if approved_pair:
        baseline_version = field(baseline, "baseline_version", "Baseline version")
        logical_version = field(logical, "logical_design_version", "Logical design version")
        if str(cohort.get("baseline_version", "")) != baseline_version:
            fail("registry baseline_version does not match the current legacy Baseline")
        if str(cohort.get("logical_design_version", "")) != logical_version:
            fail("registry logical_design_version does not match the current legacy Logical Design")
        baseline_by = field(baseline, "approved_by", "Approved by")
        logical_by = field(logical, "approved_by", "Approved by")
        baseline_at = field(baseline, "approved_at", "Approved at")
        logical_at = field(logical, "approved_at", "Approved at")
        if not baseline_by or not baseline_at or (baseline_by, baseline_at) != (logical_by, logical_at):
            fail("legacy artifacts do not establish one unambiguous joint approval")
        approved_by, approved_at = baseline_by, baseline_at

    decisions = section(baseline, "Authoritative Decisions")
    logical_decisions = section(logical, "Logical Design Decisions")
    if not decisions or not logical_decisions:
        fail("cannot find both legacy architecture decision sections")

    scope_path = project / f"architecture/scopes/{scope_id}.md"
    if scope_path.exists():
        fail(f"target already exists: {scope_path.relative_to(project)}")

    metadata = {
        "features": registry_members,
        "approval": {"by": approved_by, "at": approved_at} if approved_pair else None,
    }
    front_matter = yaml.safe_dump(metadata, sort_keys=False, default_flow_style=False).strip()
    output = (
        f"---\n{front_matter}\n---\n\n# Architecture Scope: {scope_id}\n\n"
        "Migrated without reinterpretation from the legacy Architecture Baseline and Cohort "
        "Logical Design. Git preserves the source revisions.\n\n"
        f"## Preserved project-level decisions\n\n{decisions}\n\n"
        f"## Preserved shared logical decisions\n\n{logical_decisions}\n"
    )

    profile_path = project / "architecture/implementation-profile.yaml"
    profile = load_yaml(profile_path) if profile_path.is_file() else None
    profile_changed = False
    if profile is not None:
        applies = profile.get("applies_to")
        if not isinstance(applies, dict):
            fail("Implementation Profile applies_to must be a mapping")
        unknown = set(applies) - {"scope", "feature_ids", "cohort_ids"}
        if unknown:
            fail(f"Implementation Profile applies_to has unsupported fields: {sorted(unknown)!r}")
        selector = applies.get("scope")
        cohort_ids = applies.get("cohort_ids", [])
        if not isinstance(cohort_ids, list):
            fail("Implementation Profile cohort_ids must be a list")
        if selector == "cohort_ids":
            if cohort_ids != [scope_id] or applies.get("feature_ids", []) not in ([], None):
                fail("Implementation Profile cohort selector does not resolve unambiguously to the migrated scope")
            applies["scope"] = "feature_ids"
            applies["feature_ids"] = list(registry_members)
            applies.pop("cohort_ids", None)
            profile_changed = True
        elif selector in {"all", "feature_ids"}:
            if cohort_ids:
                fail("Implementation Profile has populated unused cohort_ids")
            if "cohort_ids" in applies:
                applies.pop("cohort_ids")
                profile_changed = True
        else:
            fail(f"unsupported Implementation Profile selector during migration: {selector!r}")

        exceptions = profile.get("exceptions") or []
        if not isinstance(exceptions, list):
            fail("Implementation Profile exceptions must be a list")
        explicit = []
        cohort_exceptions = []
        for exception in exceptions:
            if not isinstance(exception, dict):
                fail("Implementation Profile contains a non-mapping exception")
            unknown = set(exception) - {"scope", "id", "language", "rationale"}
            if unknown:
                fail(f"Implementation Profile exception has unsupported fields: {sorted(unknown)!r}")
            if exception.get("scope") == "feature_id":
                explicit.append(exception)
            elif exception.get("scope") == "cohort_id":
                if exception.get("id") != scope_id:
                    fail("Implementation Profile exception names an unknown legacy cohort")
                cohort_exceptions.append(exception)
            else:
                fail(f"unsupported Implementation Profile exception selector: {exception.get('scope')!r}")

        migrated_exceptions = []
        resolved_languages = {}

        def add_exception(feature_id: str, source: dict) -> None:
            if not feature_id:
                fail("Implementation Profile exception has an empty feature id")
            language_value = source.get("language")
            rationale_value = source.get("rationale")
            if not isinstance(language_value, str) or not isinstance(rationale_value, str):
                fail("Implementation Profile exception language and rationale must be strings")
            language = language_value.strip()
            rationale = rationale_value.strip()
            if not language or not rationale:
                fail("Implementation Profile exception has an empty language or rationale")
            prior = resolved_languages.get(feature_id)
            if prior is not None:
                if prior != language:
                    fail(f"Implementation Profile exceptions disagree for feature {feature_id}")
                return
            resolved_languages[feature_id] = language
            migrated_exceptions.append({
                "scope": "feature_id",
                "id": feature_id,
                "language": language,
                "rationale": rationale,
            })

        for exception in explicit:
            feature_id = exception.get("id")
            if not isinstance(feature_id, str):
                fail("Implementation Profile exception feature id must be a string")
            add_exception(feature_id.strip(), exception)
        for exception in cohort_exceptions:
            for feature_id in registry_members:
                add_exception(feature_id, exception)
        if migrated_exceptions != exceptions:
            profile["exceptions"] = migrated_exceptions
            profile_changed = True
    if profile_changed and not tracked(project, profile_path):
        fail("Implementation Profile needs selector migration but is not committed; commit it first")

    legacy_history = sorted((project / "architecture/history").glob("core-baseline-v*.md"))
    legacy_history += sorted((project / "architecture/history").glob("cohort-logical-design-v*.md"))
    for path in legacy_history:
        if not tracked(project, path):
            fail(f"legacy history is not committed; commit it before migration: {path.relative_to(project)}")

    target_state = "approved" if approved_pair else "draft"
    print(f"validated legacy scope {scope_id}: {len(registry_members)} features, target state {target_state}")
    if not args.apply:
        print("dry run only; rerun with --apply to convert and remove legacy state")
        return 0

    scope_path.parent.mkdir(parents=True, exist_ok=True)
    atomic_write(scope_path, output)
    for feature in registry.get("features", []):
        if isinstance(feature, dict):
            feature.pop("architecture_cohort", None)
    registry.pop("architecture_cohorts", None)
    atomic_write(registry_path, yaml.safe_dump(registry, sort_keys=False))
    if profile_changed:
        atomic_write(profile_path, yaml.safe_dump(profile, sort_keys=False))
    for path in [baseline_path, logical_path, *legacy_history]:
        path.unlink()
    print(f"migrated to {scope_path.relative_to(project)}; legacy state removed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except MigrationError as exc:
        print(f"migration stopped: {exc}", file=sys.stderr)
        raise SystemExit(1)
