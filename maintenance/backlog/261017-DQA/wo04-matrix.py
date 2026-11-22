#!/usr/bin/env python3
"""DQA WO-04: the repetition matrix, recomputed from a JamFree `.codeos/06-workflow` directory.

Read-only. Usage: python3 -I wo04-matrix.py <JamFree clone>/.codeos/06-workflow
"""
import collections
import glob
import json
import os
import re
import sys

d = sys.argv[1]
load = lambda name: [json.loads(l) for l in open(f"{d}/{name}") if l.strip()]
V = [v for v in load("verifications.jsonl") if v["workflow"] == "feature"]
D = load("decisions.jsonl")
EVIDENCE = {os.path.basename(f)[:-5]: json.load(open(f)) for f in glob.glob(f"{d}/reality-evidence/*.json")}

# A command differs between runs only by its per-run report path.
norm = lambda c: re.sub(r"/tmp/codeos-[a-z-]+-[0-9]+-[0-9]+\.json", "<tmp>", c)
state = lambda v: v["bindings"].get("implementation_state") or v["bindings"].get("evidence_scope_state")


def leg(v):
    claims = EVIDENCE[v["details"]["evidence_sha256"]]["claims"]
    return "return" if v["subject"] == "F-0002" or "return-summary" in claims else "outbound"


def execution(v):
    """What the run consumed: the command and implementation state; production work is one
    forecast request per leg and date, whatever claims it is checked against."""
    if v["verification"] == "production_source_evidence":
        ev = EVIDENCE[v["details"]["evidence_sha256"]]
        return (v["verification"], leg(v), ev["observed_at"][:10], state(v))
    return (v["verification"], norm(v["command"]), state(v), v["checkpoint"] == "candidate")


print("## Executions (records -> distinct executions; repeats by cause)")
groups = collections.defaultdict(list)
for v in V:
    if v["verification"] != "runtime_event_schema":
        groups[execution(v)].append(v)
causes = collections.defaultdict(collections.Counter)
records = collections.Counter()
distinct = collections.Counter()
for key, vs in groups.items():
    m = key[0]
    records[m] += len(vs)
    distinct[m] += 1
    for i, v in enumerate(vs[1:], 1):
        prior = [p for p in vs[:i] if p["subject"] == v["subject"]]
        if not prior:
            cause = "other feature, same execution"
        else:
            p = prior[-1]
            changed = sorted(k for k in set(p["bindings"]) | set(v["bindings"])
                             if p["bindings"].get(k) != v["bindings"].get(k))
            cause = f"same feature, changed {','.join(changed)}" if changed else "same feature, nothing changed"
        causes[m][cause] += 1
for m in sorted(records):
    print(f"{m}: {records[m]} -> {distinct[m]}; {dict(causes[m])}")

print("\n## Production provider work (receipted successful calls)")
P = [v for v in V if v["verification"] == "production_source_evidence"]
receipts = lambda v: len({json.dumps(r, sort_keys=True) for o in EVIDENCE[v["details"]["evidence_sha256"]]["observations"]
                          for r in o["provider_request_response_receipts"]})
legs = collections.defaultdict(list)
for v in P:
    legs[execution(v)].append((v["subject"], receipts(v)))
print(f"runs {len(P)}, calls {sum(receipts(v) for v in P)}; "
      f"distinct legs {len(legs)}, calls {sum(max(n for _, n in xs) for xs in legs.values())}")
for key, xs in sorted(legs.items(), key=lambda kv: kv[0][2]):
    if len(xs) > 1:
        print(f"  one leg, several features: {key[1]} {key[2]} state {key[3][:8]} {xs}")

print("\n## Journey observations shared across features (state, frontier -> observed surfaces)")
journeys = collections.defaultdict(list)
for v in V:
    if v["verification"] == "product_journey":
        surface = (v.get("details") or {}).get("observed_surface", "-")[:8]
        journeys[(state(v)[:8], v["command"].split("@")[-1])].append((v["subject"], surface))
for key, xs in sorted(journeys.items()):
    if len({s for s, _ in xs}) > 1:
        print(f"  {key[0]} {key[1]}: {xs}")

print("\n## Human decisions by Observed Surface (one observation, several judgments)")
surfaces = collections.defaultdict(list)
for r in D:
    if r["checkpoint"] in ("early_preview", "final_ux_validation") and "observed_surface" in r["bindings"]:
        surfaces[r["bindings"]["observed_surface"][:8]].append((r["subject"], r["checkpoint"], r["timestamp"]))
judgments = sum(len(xs) for xs in surfaces.values())
print(f"receipts {judgments}, distinct surfaces {len(surfaces)}")
for s, xs in surfaces.items():
    if len(xs) > 1:
        print(f"  {s}: {xs}")
