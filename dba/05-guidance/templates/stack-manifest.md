# Stack Manifest

> **Trigger rule.** If any of the following file types change — `Cargo.toml`, `Cargo.lock`,
> `pyproject.toml`, `poetry.lock`, `requirements.txt`, `package.json`, `package-lock.json`,
> `pnpm-lock.yaml`, `Dockerfile`, `docker-compose.yml`, `.env.example`, `config/*.toml`,
> `config/*.yaml`, `settings.*` — fill in `dba/05-guidance/templates/stack-reconciliation-report.md` before
> merge or release. This record is updated when the reconciliation report says YES.
>
> This manifest has two layers: (1) stable decisions that change rarely; (2) policy that
> governs how the stack evolves. Keep it short — one line per field is the norm.

---

## Stable Stack Decisions

Language/runtime:
Package manager:
Test framework:
Event log format:
Replay test location:
Database/persistence:
External services:
Deployment target:
Allowed dependency categories:
Forbidden dependency categories:

---

## Dependency Policy

When a new dependency may be added:
Who approves:
Required justification:
Where it must be documented:
Required tests before merge:

---

## Configuration Policy

Where config lives:
How config schema is validated:
Secret vs non-secret config:
Environment-specific config:
Defaulting policy:

---

## Last reconciled

Date:
Commit:
Triggered by:
