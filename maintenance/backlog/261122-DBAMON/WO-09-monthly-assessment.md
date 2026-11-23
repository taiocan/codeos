# WO-DM-09 — Monthly Development Assessment

- **Class:** NORMAL, PowerBI repository (`projects/dba-monitor`).
- **Goal:** a succinct monthly assessment of development status, written as a professional
  programmer and a CodeOS developer would write it, shown in the report.

## Authority

- The assessment is advisory, dated narrative. It is never workflow, validity, release,
  remediation, or acceptance evidence.
- Claude drafts it only when the human invokes it ("monthly assessment"). The human sets the
  approval fields.
- `editorial_status: approved` means only that the human approved the narrative for display. It is
  not CodeOS acceptance and has no effect on any CodeOS authority.
- It may interpret snapshot facts and recommend actions. It never manufactures missing
  measurements and never turns UNOBSERVED into a qualitative judgement: an UNOBSERVED category says
  only that, plus at most an action to observe it.
- Narrative only: no ratings, scores, or targets (boundary 3 unchanged).

## Scope

- `assessments/README.md` is the one authority for the procedure and format, and states the
  authority rules above.
- Fixed categories:
  1. Delivery flow
  2. Rework and review
  3. Test health
  4. Code health and security
  5. Governance surface (CodeOS growth and complexity)
  6. Process and tooling reliability (reviewer availability, execution-invalid runs, unqualified
     releases)
  7. Risks and recommended actions (at most 3)
- Each category has `observation` (figures cited from the snapshots), `interpretation`, and
  `action` (a next step or "none"), one to two sentences each. The whole assessment fits on one
  page.
- One file per month, `assessments/YYYY-MM.json`, the only representation (no Markdown copy):
  `month`, `snapshot_ids` (project, static, toolkit), `drafted_by` (`claude`), `editorial_status`
  (`draft | approved`), `approved_by` (a fixed owner identifier such as `owner`, never an email),
  `approved_on`, `categories[]`.
- Report page "Assessment": a month slicer and a table of category, observation, interpretation,
  action, and editorial status; a draft is visibly marked. Power Query reads `assessments\*.json`
  through a helper expression (privacy firewall).
- The first assessment (2026-11) is drafted at implementation and stays `draft` until the human
  approves it.
- No signatures, schema tooling, or approval workflow beyond these fields.

## Verification

- The JSON follows the README format.
- The page shows the month and its editorial status.
- Every figure cited in the draft matches the snapshot it names.

## Acceptance

The verification passes and the human accepts.
