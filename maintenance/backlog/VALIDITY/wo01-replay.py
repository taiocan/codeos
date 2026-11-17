#!/usr/bin/env python3
"""VALIDITY WO-01: read-only replay of JamFree invalidations, 2026-11-13 .. 2026-11-17.

Usage: python3 -I wo01-replay.py <jamfree-repo> <commit> [--rows]

Everything is read with `git show <commit>:<path>`, so the working tree is never consulted.
For every re-decision of F4/F6/F8/F9 and every Stage 8 review in the window it reports which of
the old rule's bindings changed since the same subject's previous decision (or review), and, for a
changed Candidate Bundle, which bundle fields changed. A verification record that changed only in
its timestamp is reported as `record:same-content`; anything else is reported as it was bound.
"""

import collections
import hashlib
import json
import re
import subprocess
import sys

WINDOW = ("2026-11-13", "2026-11-18")
CHECKPOINTS = {
    "early_preview": "F4",
    "reconciliation": "F6",
    "final_ux_validation": "F8",
    "acceptance": "F9",
}
# Paths whose changes are workflow bookkeeping, not delivered product.
BOOKKEEPING = (".codeos/05-review/", ".codeos/06-workflow/", "events/", "backlog/", "docs/")


def git(repo, *args):
    return subprocess.run(
        ["git", "-C", repo, *args], check=True, capture_output=True, text=True
    ).stdout


def jsonl(repo, commit, path):
    return [l for l in git(repo, "show", f"{commit}:{path}").splitlines() if l.strip()]


class Replay:
    def __init__(self, repo, commit):
        self.repo, self.commit = repo, commit
        self.decisions = [json.loads(l) for l in jsonl(repo, commit, ".codeos/06-workflow/decisions.jsonl")]
        self.records = {}
        for line in jsonl(repo, commit, ".codeos/06-workflow/verifications.jsonl"):
            self.records[hashlib.sha256(line.encode()).hexdigest()] = json.loads(line)
        self.bundles = {}
        self.declared = collections.defaultdict(set)  # feature -> declared dependency additions
        self.recorded_in = self._recorded_in()

    def bundle(self, bundle_id):
        if bundle_id not in self.bundles:
            path = f".codeos/06-workflow/candidates/{bundle_id}.json"
            self.bundles[bundle_id] = json.loads(git(self.repo, "show", f"{self.commit}:{path}"))
            b = self.bundles[bundle_id]
            self.declared[b["feature"]].update(b.get("dependency_additions") or [])
        return self.bundles[bundle_id]

    def _recorded_in(self):
        """Decision index -> the first commit whose decisions.jsonl contains it."""
        path = ".codeos/06-workflow/decisions.jsonl"
        commits = git(self.repo, "log", "--reverse", "--format=%H", self.commit, "--", path).split()
        owner, seen = {}, 0
        for c in commits:
            n = len(jsonl(self.repo, c, path))
            for i in range(seen, n):
                owner[i] = c
            seen = max(seen, n)
        return owner

    def product_delta(self, older, newer):
        """Delivered (non-bookkeeping) files changed between two recording commits."""
        if older is None or newer is None or older == newer:
            return []
        names = git(self.repo, "diff", "--name-only", older, newer).split()
        return [n for n in names if not n.startswith(BOOKKEEPING)]

    def code_commits(self, older, newer):
        """Commits between two recording commits that touch delivered (non-bookkeeping) paths."""
        if older is None or newer is None or older == newer:
            return []
        excludes = [f":(exclude){p}" for p in BOOKKEEPING]
        out = git(self.repo, "log", "--format=%h", f"--since={WINDOW[0]}T00:00:00Z", f"{older}..{newer}",
                  "--", ".", *excludes).split()
        if not out and self.files(older):
            # The change was committed together with the previous decision (marked `*`).
            out = [git(self.repo, "rev-parse", "--short", older).strip() + "*"]
        return out

    def files(self, commit):
        """Delivered (non-bookkeeping) files a commit changed."""
        names = git(self.repo, "show", "--format=", "--name-only", commit.rstrip("*")).split()
        return [n for n in names if not n.startswith(BOOKKEEPING)]

    def categories(self, row):
        """Path categories of the delivered files changed in a case's interval."""
        own = (f"/{row[1]}.md", f"/{row[1]}_contract.md", f"/{row[1]}_schema.md")
        found = set()
        for c in row[4]:
            for f in self.files(c):
                if f in self.declared[row[1]]:
                    found.add("declared-dependency")
                elif f.startswith(".codeos/01-specification/"):
                    found.add("own-spec" if f.endswith(own) else "other-spec")
                elif f == ".codeos/00-project/codeos.yaml":
                    found.add("configuration")
                elif f.startswith((".codeos/00-project/", ".codeos/02-architecture/")):
                    found.add("governed")
                elif re.search(r"(\.e2e\.ts|\.spec\.ts|\.test\.)", f) or f.startswith(
                        ("scripts/", "evidence/", "provider-boundary/")):
                    found.add("test-evidence-script")
                elif f.startswith(("backend/src/", "web/src/", "backend/migrations/")):
                    found.add("product-source")
                else:
                    found.add("runtime-build")
        return sorted(found)

    def same_content(self, a, b):
        ra, rb = self.records.get(a), self.records.get(b)
        if ra is None or rb is None:
            return None
        strip = lambda r: {k: v for k, v in r.items() if k != "timestamp"}
        return strip(ra) == strip(rb)

    def bundle_causes(self, old_id, new_id):
        """Bundle fields that changed; verification records split by content."""
        old, new = self.bundle(old_id), self.bundle(new_id)
        causes = []
        for key in sorted(set(old) | set(new)):
            if key in ("predecessor_bundle",) or old.get(key) == new.get(key):
                continue
            if key == "verification_records":
                o, n = old.get(key, {}), new.get(key, {})
                for rec in sorted(set(o) | set(n)):
                    if o.get(rec) == n.get(rec):
                        continue
                    same = self.same_content(o.get(rec), n.get(rec))
                    if same:
                        causes.append(f"record:same-content:{rec}")
                    else:
                        changed = self.record_binding_delta(o.get(rec), n.get(rec))
                        causes.append(f"record:changed:{rec}" + (f"[{changed}]" if changed else ""))
            elif key in ("implementation_inputs", "governed_inputs") and isinstance(new.get(key), dict):
                o, n = old.get(key) or {}, new.get(key) or {}
                paths = sorted(p for p in set(o) | set(n) if o.get(p) != n.get(p))
                causes.append(f"{key}:{len(paths)}:" + ",".join(paths[:4]))
            else:
                causes.append(key)
        return causes

    def record_binding_delta(self, a, b):
        ra, rb = self.records.get(a), self.records.get(b)
        if ra is None or rb is None:
            return "unresolved"
        keys = sorted(k for k in set(ra["bindings"]) | set(rb["bindings"])
                      if ra["bindings"].get(k) != rb["bindings"].get(k))
        if ra.get("result") != rb.get("result"):
            keys.append("result")
        if not keys and ra.get("details") != rb.get("details"):
            keys.append("details")
        return ",".join(keys)

    def decision_rows(self):
        previous = {}
        rows = []
        for i, d in enumerate(self.decisions):
            key = (d["subject"], d["checkpoint"])
            prior = previous.get(key)
            previous[key] = (i, d)
            if d["checkpoint"] not in CHECKPOINTS or not (WINDOW[0] <= d["timestamp"] < WINDOW[1]):
                continue
            if prior is None:
                rows.append((CHECKPOINTS[d["checkpoint"]], d["subject"], d["timestamp"], ["first"], [], d["result"], (None, None)))
                continue
            pi, p = prior
            changed = sorted(k for k in set(p["bindings"]) | set(d["bindings"])
                             if p["bindings"].get(k) != d["bindings"].get(k))
            causes = []
            for k in changed:
                if k == "candidate_bundle" and p["bindings"].get(k) and d["bindings"].get(k):
                    causes += [f"bundle>{c}" for c in self.bundle_causes(p["bindings"][k], d["bindings"][k])]
                else:
                    causes.append(k)
            delta = self.code_commits(self.recorded_in.get(pi), self.recorded_in.get(i))
            span = (self.recorded_in.get(pi), self.recorded_in.get(i))
            rows.append((CHECKPOINTS[d["checkpoint"]], d["subject"], d["timestamp"], causes or ["none"], delta, d["result"], span))
        return rows

    def review_rows(self):
        base = ".codeos/05-review/reviews/codex/"
        names = sorted(n for n in git(self.repo, "ls-tree", "--name-only", self.commit, base).split()
                       if re.match(r".*/\d{4}-\d\d-\d\dT\d{6}Z-F-\d{4}-stage-8-", n))
        previous, previous_commit, rows = {}, {}, []
        for name in names:
            text = git(self.repo, "show", f"{self.commit}:{name}")
            stamp = name.rsplit("/", 1)[1][:17]
            feature = re.search(r"\n  feature: (F-\d{4})", text).group(1)
            bundle = re.search(r"\n  candidate_bundle: ([0-9a-f]{64})", text)
            concern = re.search(r"\n  effective_concern: (.+)", text)
            bundle = bundle.group(1) if bundle else None
            concern = concern.group(1).strip() if concern else "unknown"
            when = stamp[:10]
            prior = previous.get(feature)
            previous[feature] = bundle
            if not (WINDOW[0] <= when < WINDOW[1]):
                commit = re.search(r"\n  review_commit: ([0-9a-f]{40})", text)
                previous_commit[feature] = commit.group(1) if commit else None
                continue
            if prior is None or bundle is None:
                causes = ["first" if prior is None else "no-bundle"]
            elif prior == bundle:
                causes = ["same-bundle"]
            else:
                causes = [f"bundle>{c}" for c in self.bundle_causes(prior, bundle)]
            commit = re.search(r"\n  review_commit: ([0-9a-f]{40})", text)
            commit = commit.group(1) if commit else None
            span = (previous_commit.get(feature), commit)
            delta = self.code_commits(*span)
            previous_commit[feature] = commit
            rows.append(("Stage8", feature, stamp, causes, delta, concern, span))
        return rows


    def field(self, commit, path, name):
        """A `name: value` line from a file at a commit, or None."""
        if commit is None:
            return None
        try:
            text = git(self.repo, "show", f"{commit}:{path}")
        except subprocess.CalledProcessError:
            return None
        m = re.search(rf"^{name}: *(.+)$", text, re.M)
        return m.group(1).strip() if m else None

    def relevance(self, row):
        """Whether the old rule's changed inputs were semantically relevant to the decision."""
        cls = self.refine(row)
        if cls in ("record-timestamp-only", "spec:same-revision", "other:none", "execution-identity-only"):
            return "not-relevant"
        if cls in ("spec:semantic", "observed_surface", "first"):
            return "relevant"
        cats = set(self.categories(row))
        if cats & {"product-source", "governed", "own-spec", "declared-dependency"}:
            return "relevant"
        if "runtime-build" in cats:
            return "runtime-only"
        if cls.startswith("configuration"):
            return "per-DBA"
        if not cats:
            return "unresolved"
        return "not-relevant"

    def refine(self, row):
        """Mechanical relevance tests: a spec change counts as semantic only when the feature's
        semantic_revision changed; a configuration change names its DBA transition."""
        cls = trigger(row[3])
        older, newer = row[6]
        if cls == "spec":
            path = f".codeos/01-specification/intents/{row[1]}.md"
            a, b = self.field(older, path, "semantic_revision"), self.field(newer, path, "semantic_revision")
            if a is None or b is None:
                return "spec:unresolved"
            return "spec:semantic" if a != b else "spec:same-revision"
        if cls == "configuration":
            path = ".codeos/00-project/codeos.yaml"
            a, b = self.field(older, path, "dba_configuration"), self.field(newer, path, "dba_configuration")
            return f"configuration:{a}>{b}" if a != b else f"configuration:{b}-unchanged"
        return cls


SPEC = ("contract", "intent", "event_schema", "semantic_revision", "charter")
CONFIG = ("codeos_yaml", "configuration_sha256")
IMPLEMENTATION = ("implementation_state", "delivered_state", "implementation_inputs",
                  "evidence_scope_state", "dependency_addition", "qualification_script")


def trigger(causes):
    """The old rule's bindings that changed, as one class, upstream first. `session` is ignored:
    every human decision opens a new session."""
    text = " ".join(causes)
    names = set(re.findall(r"[a-z_]+", text))
    if "first" in names:
        return "first"
    if names & set(SPEC):
        return "spec"
    if names & set(CONFIG):
        return "configuration"
    if names & set(IMPLEMENTATION):
        return "implementation"
    if "observed_surface" in names:
        return "observed_surface"
    if all(c.startswith("bundle>record:same-content") or c == "session" for c in causes):
        return "record-timestamp-only"
    records = [c for c in causes if c.startswith("bundle>record:")]
    if records and all(c.endswith("[execution_identity]") for c in records) and all(
            c.startswith("bundle>record:") or c == "session" for c in causes):
        return "execution-identity-only"
    if names & {"production_evidence", "runtime_evidence", "f6_receipt_sha256", "execution_identity"}:
        return "evidence"
    if causes == ["same-bundle"]:
        return "same-bundle"
    return "other:" + ",".join(sorted(names))


def main():
    repo, commit = sys.argv[1], git(sys.argv[1], "rev-parse", sys.argv[2]).strip()
    show_rows = "--rows" in sys.argv
    r = Replay(repo, commit)
    rows = r.decision_rows() + r.review_rows()
    order = ("F4", "F6", "Stage8", "F8", "F9")
    print(f"JamFree {commit[:12]}, window {WINDOW[0]} .. {WINDOW[1]} (exclusive)\n")
    by_cp = collections.Counter(row[0] for row in rows)
    print("Counts: " + ", ".join(f"{k} {by_cp[k]}" for k in order))

    print("\nTrigger class per case (old rule's changed bindings, upstream first):")
    classes = sorted({r.refine(row) for row in rows})
    print("  " + "class".ljust(34) + "".join(k.rjust(8) for k in order))
    for c in classes:
        n = [sum(1 for row in rows if row[0] == k and r.refine(row) == c) for k in order]
        print("  " + c.ljust(34) + "".join(str(x).rjust(8) for x in n))

    print("\nRelevance of the changed inputs (WO-01 step 2):")
    verdicts = ("relevant", "runtime-only", "not-relevant", "per-DBA", "unresolved")
    print("  " + "verdict".ljust(34) + "".join(k.rjust(8) for k in order))
    for v in verdicts:
        n = [sum(1 for row in rows if row[0] == k and r.relevance(row) == v) for k in order]
        print("  " + v.ljust(34) + "".join(str(x).rjust(8) for x in n))
    print("\n  Stage 8 verdict by relevance:")
    tab = collections.Counter((r.relevance(row), row[5]) for row in rows if row[0] == "Stage8")
    for (v, concern), n in sorted(tab.items()):
        print(f"    {v:13} {concern:18} {n}")
    print("\n  implementation cases by changed-path categories:")
    combos = collections.Counter((row[0], ",".join(r.categories(row)) or "-") for row in rows
                                 if r.refine(row) == "implementation")
    for (cp, cats), n in sorted(combos.items()):
        print(f"    {cp:6} {n:3}  {cats}")

    print("\nCode commits contained in re-decision intervals (cases per checkpoint):")
    per_commit = collections.defaultdict(collections.Counter)
    for row in rows:
        for c in row[4]:
            per_commit[c][row[0]] += 1
    for c in sorted(per_commit, key=lambda c: git(repo, "show", "-s", "--format=%cI", c.rstrip("*"))):
        subject = git(repo, "show", "-s", "--format=%cs %s", c.rstrip("*")).strip()[:70]
        files = r.files(c)
        counts = " ".join(f"{k}:{per_commit[c][k]}" for k in order if per_commit[c][k])
        print(f"  {c} {subject}\n      files: {','.join(files[:5])}{' …' if len(files) > 5 else ''}\n      {counts}")

    if show_rows:
        print("\nRows:")
        for row in rows:
            cp, subject, when, causes, commits, verdict, _ = row
            print(f"  {cp:6} {subject:8} {when}  {r.refine(row):30} {r.relevance(row):13} {verdict:20} code:{','.join(commits) or '-'}")
            print(f"         {' | '.join(causes)[:400]}")


if __name__ == "__main__":
    main()
