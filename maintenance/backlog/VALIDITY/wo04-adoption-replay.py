#!/usr/bin/env python3
"""VALIDITY WO-04: replay JamFree's DBA adoptions under Rule W4, read-only.

Usage: python3 -I wo04-adoption-replay.py <jamfree-repo> <commit>

For every commit that changed `dba_configuration`, it reports whether a consumer's W4 dependency
changed: the `codeos.yaml` sections it reads (compared without comments) and the contract
versions the selected configurations name. Today every consumer binds the whole file, so every
adoption invalidated all of them.
"""
import subprocess, sys

TOOLKIT = __file__.rsplit("/maintenance/", 1)[0]
CONSUMERS = {
    "B3": (["platform", "runtime_verification"], ["workflow_bootstrap_policy", "codeos_mechanics_policy"]),
    "B4": (["platform"], ["workflow_bootstrap_policy"]),
    "execution identity": (["platform", "runtime_verification", "reality_verification"], ["codeos_mechanics_policy"]),
    "project_qualification": ([], []),  # protocol schema 1 throughout
    "F6": ([], []),
}


def git(repo, *a):
    return subprocess.run(["git", "-C", repo, *a], check=True, capture_output=True, text=True).stdout


def section(text, name):
    """The section's lines without comments: an approximation of the engine's parsed comparison."""
    out, inside = [], False
    for line in text.splitlines():
        if not line.startswith((" ", "#")) and line.strip():
            inside = line.split(":")[0] == name
        if inside and not line.lstrip().startswith("#") and line.strip():
            out.append(line.split(" #")[0].rstrip())
    return "\n".join(out)


def contracts(dba):
    text = open(f"{TOOLKIT}/dba/00-entry/configurations/{dba}.yaml").read()
    return dict(l.split(": ", 1) for l in text.splitlines() if ": " in l and not l.startswith("#"))


def main():
    repo, commit = sys.argv[1], sys.argv[2]
    path = ".codeos/00-project/codeos.yaml"
    for c in git(repo, "log", "--reverse", "--format=%h", commit, "--", path).split():
        try:
            old = git(repo, "show", f"{c}^:{path}")
        except subprocess.CalledProcessError:
            continue  # the file's first commit
        new = git(repo, "show", f"{c}:{path}")
        dba = lambda t: next((l.split(": ")[1] for l in t.splitlines() if l.startswith("dba_configuration:")), None)
        if dba(old) == dba(new) or dba(old) is None:
            continue
        a, b = contracts(dba(old)), contracts(dba(new))
        changed = []
        for name, (sections, keys) in CONSUMERS.items():
            if any(section(old, s) != section(new, s) for s in sections) or any(a.get(k) != b.get(k) for k in keys):
                changed.append(name)
        print(f"{c} {dba(old)} -> {dba(new)}: invalidated under W4: {', '.join(changed) or 'none'}")


main()
