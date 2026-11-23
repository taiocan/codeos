# WO-DM-09 — Monthly Development Assessment

- **Class:** NORMAL, PowerBI repository (`projects/dba-monitor`).
- **Goal:** a succinct monthly assessment of development status, written as a professional
  programmer and systems engineer would write it: where each area stands, whether it is improving,
  stable or degrading, and the best improvement opportunity. It is shown in the report.

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
- Evaluation is allowed (revised by the human 2026-11-23, replacing "narrative only"). Each
  category carries a quality assessment, a trend label and an improvement opportunity. There are
  still no scores, no targets and no overall rating: the summary is prose, and the report never
  aggregates or ranks the trend labels.
- Every trend label states its basis: the figures compared and their windows. Without an earlier
  comparable figure, the label is `no_baseline`, never a guess. Rates are compared instead of raw
  counts when volume changes.

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
- A two- or three-sentence `summary` opens the assessment.
- Each category has:
  - `observation`: figures cited from the snapshots;
  - `assessment`: where the area stands, in professional judgement;
  - `trend`: `improving | stable | degrading | no_baseline`;
  - `trend_basis`: the compared figures and their windows;
  - `opportunity`: the most valuable improvement and its next step, or "none".

  Each field is one or two sentences, and the whole assessment fits on one page.
- One file per month, `assessments/YYYY-MM.json`, the only representation (no Markdown copy):
  `month`, `snapshot_ids` (project, static, toolkit), `drafted_by` (`claude`), `editorial_status`
  (`draft | approved`), `approved_by` (a fixed owner identifier such as `owner`, never an email),
  `approved_on`, `summary`, `categories[]`.
- Report page "Assessment": a month slicer and a summary, and a table of category, observation, assessment,
  trend, basis and opportunity, plus the editorial status; a draft is visibly marked. Power Query reads `assessments\*.json`
  through a helper expression (privacy firewall).
- The first assessment (2026-11) is drafted at implementation and stays `draft` until the human
  approves it.
- No signatures, schema tooling, or approval workflow beyond these fields.

## Verification

- The JSON follows the README format.
- The page shows the month and its editorial status.
- Every figure cited in the draft matches the snapshot it names.
- Every trend other than `no_baseline` has a basis comparing at least two windows.

## Acceptance

The verification passes and the human accepts.
