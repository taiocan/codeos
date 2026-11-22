# WO-02 — Platform Assurance Contract

PROTECTED · after WO-01

## Goal

Approve the public meaning of Platform Assurance before implementation.

## Work

- Fix the areas, statuses, evidence mapping, JSON, and terminal behavior.
- Keep Platform Assurance separate from Live Health and audit linkage.
- Add the accepted definition to the canonical glossary.

## Acceptance criteria

1. Human approval records unambiguous derivation and absence behavior.
2. Results do not change workflow authority, progression, or exit codes.
3. Compatibility and terminology checks pass.

## Approved contract (revised 2026-10-21)

Platform Assurance is project-wide and read-only. It includes started and historically accepted
features; planned candidates are excluded. Missing or invalid Feature Impact Accounting makes
every area `UNOBSERVED` for that subject.

Areas and required evidence are:

| Area | Applicability and evidence |
|---|---|
| `persistence` | Persistence changed; current `data_integrity` |
| `backend` | Backend changed; current smoke and behavior |
| `gui` | GUI changed or GUI-visible; current Playwright |
| `integrated_product` | Every included subject; smoke, active-contract product journey, and applicable tier checks |

Accepted features use exact Candidate Bundle record identities. Current in-progress features use
latest records evaluated against current bindings. Legacy accepted evidence without exact
identities is `UNOBSERVED`; no record is guessed. Current revalidation names its evidence chain.

Precedence is `BLOCKED`, `STALE`, `UNOBSERVED`, `PASS`; no applicable subject gives
`NOT_APPLICABLE`. `BLOCKED` requires a retained failed record. Basis lists each subject and exact
record identity deterministically.

JSON v2/v3 add `platform_assurance` without a version bump. Standard `status` summarizes
Persistence (PostgreSQL), Backend (Rust), GUI (Svelte), and Integrated product in that order.
`status --platform` remains the project-wide evidence drill-down and cannot combine with subject,
workflow, history, or JSON. Both retain the ordinary exit code.

Platform Assurance is not Engineering Health authority, Live Health, an audit chain, or a score.
The human approved this visibility revision on 2026-10-21; derivation semantics did not change.
