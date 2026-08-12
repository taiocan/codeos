# Codeos Toolkit Stack Manifest

> **Historical snapshot.** This status and reconciliation mechanism was retired from Codeos
> self-development. The content below records the former mechanism and creates no current
> documentation, approval, or reconciliation requirement.
>
> This file records the current observed stack and dependency-policy status for Codeos
> self-development. It is not an independent approval authority. If it conflicts with the
> self-dev workflow, `CLAUDE.md` and the approved change record govern.
>
> Mutable status file — like `status/self-development.md` and `status/roadmap.md`, this is
> evidence/status, not doctrine. See `CLAUDE.md`'s "Stack / Dependency Reconciliation" section
> for the trigger rule this file supports.

---

## Stable Stack Decisions

Runtime/build stack: Rust / Cargo (the `tools/reviewer` crate, binary `codeos-reviewer`)
Primary verification: `cargo test` (smoke tests in `tools/reviewer/tests/smoke.rs`)
Event log / replay: not applicable — self-dev has no runtime events or replay to reconcile
(see `CLAUDE.md`'s Mode Declaration)
Dependency approval point: Step 1 ("What changes" names the dependency) and Step 2
(acceptance criteria) of the 4-step self-development loop — see "Dependency Policy" below.

---

## Watched Files (this repo)

`check-drift` (UPG-0020) hardcodes a generic watched-file list shared with downstream
projects that adopt this toolkit. Of that list, only these two currently exist and apply in
Codeos's own repo:

- `Cargo.toml`
- `Cargo.lock`

The tool's other patterns — `pyproject.toml`, `poetry.lock`, `requirements.txt`,
`package.json`, `package-lock.json`, `pnpm-lock.yaml`, `Dockerfile`, `docker-compose.yml`,
`.env.example`, `config/*.toml`, `config/*.yaml`, `settings.*` — exist for downstream repos
using this shared binary and currently match nothing in this repo.

---

## Dependency Policy

When a new dependency may be added: any self-dev change whose implementation needs a new
`tools/reviewer` Cargo dependency.
Who approves: the human, at the Step 1 and Step 2 gates of the 4-step self-development loop
(see `CLAUDE.md`).
Required justification: stated in the change record's Step 1 "Design intent" — why this
dependency, and why not a hand-rolled alternative (the pattern already established by
`generate-report`/`generate-adr-candidates`, which avoided new dependencies for simpler
Markdown-structured inputs).
Where it must be documented: the change record itself, plus a
`status/stack-reconciliation/<CHG-id>-stack-reconciliation-report.md` instance (see below).
Required tests before merge: `cargo build` and `cargo test --test smoke` both clean.

---

## Reconciliation

Trigger: any self-dev change whose Step 1 "What changes" touches `Cargo.toml` or `Cargo.lock`
must include a `status/stack-reconciliation/<CHG-id>-stack-reconciliation-report.md` instance
in the same change, verified at that change's Step 4 — see `CLAUDE.md`'s "Stack / Dependency
Reconciliation" rule. `check-drift` is run against the repo at that change's Step 4 as a
functional check, not just described.

## History

| Change | Dependency event | Reconciliation report |
|---|---|---|
| UPG-0032 / `CHG-20260702-001` | Original 9 dependencies added (`anyhow`, `chrono`, `clap`, `hex`, `regex`, `serde`, `sha2`, `tempfile`, `toml`). Retroactively backfilled by UPG-0036 (2026-07-05) — this reconciliation process did not exist at the time of the original commit. | `status/stack-reconciliation/CHG-20260702-001-stack-reconciliation-report.md` |
