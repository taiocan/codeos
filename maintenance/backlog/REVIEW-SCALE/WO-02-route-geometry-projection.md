# WO-02 — Route-Geometry Projection

PROTECTED · Reviewer Tool Contract v22 → DBA-33 · direction approved by the human 2026-11-16

## Invariant

Bulk evidence may be projected only where CodeOS identifies the field semantically and the full
value stays deterministically bound and checked. Unknown fields are shown whole.

## Rule

In production evidence `schema: 1`, the exact route-coordinate sequence
`observations[].visible_result.route.geometry.coordinates` is mechanically verified evidence, not
review content. Stage 8 receives a deterministic projection (`mode: projected`, with its reason in
the manifest): count, endpoints, bounding box, 16 evenly spaced sample points, and the sha256 of the
full array. The whole file stays bound by its sha and checked by the workflow checker.

A future feature that needs human review of route geometry introduces its own reviewable artifact.
There is no claim-dependent exception and no general JSON or numeric-array projector.

## Acceptance criteria

1. The coordinates at that path and schema are projected with the deterministic sample.
2. A same-shaped array at any other path, or under an unknown schema, is shown whole.
3. The whole-file sha binding is unchanged.
4. `qualify-candidate.sh` replay passes; DBA-33 is issued on human approval.
5. A repeated JamFree F-0002 review is supporting evidence only, not a findings-parity gate.
