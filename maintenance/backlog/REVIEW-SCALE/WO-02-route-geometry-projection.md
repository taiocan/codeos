# WO-02 — Route-Geometry Projection

PROTECTED · Reviewer Tool Contract v22 → DBA-33 · direction approved by the human 2026-11-16

## Invariant

CodeOS may project the coordinates of a GeoJSON LineString only at the explicitly supported
production-evidence field and schema versions. The projection is identified by both its governed
location and its declared semantic type. All unsupported or ambiguous values are shown whole. The
complete evidence remains SHA-bound and mechanically checked.

## Rule

Correction approved by the human 2026-11-16: the real evidence is `schema: 2`, not `schema: 1` as
first stated, and `visible_result` is opaque to CodeOS, so the payload must declare its semantics.

At `observations[].visible_result.route.geometry` in production evidence with `schema: 1` or `2`,
an object declaring exactly `type: "LineString"` whose `coordinates` holds more than 16 positions of
two or three numbers is shown as `route-geometry/v1`: count, endpoints, bounding box, 16 evenly
spaced samples, and the SHA-256 of the array's exact bytes. The projection splices the file text,
so every other byte is the source. Any other type, a missing or inexact type, a duplicate key,
another location, another schema, or another shape is shown whole. There is no general GeoJSON or
numeric projector and no project-specific projection configuration.

The projection is a review presentation transformation only. It does not change workflow evidence
parsing, evidence currentness, claim satisfaction, or the underlying evidence identity. The 16
samples are descriptive reviewer context, never evidence that the route is geometrically correct;
the full array stays mechanically bound.

A future feature that needs human review of route geometry introduces its own reviewable artifact.

## Acceptance criteria

1. LineString coordinates at that location under schema 1 and 2 are projected with the
   deterministic sample. Polygon, a missing or inexact type, a duplicate key, another location, an
   unknown schema, and a malformed position are shown whole.
2. Only the input the manifest included as production evidence is projected; the whole-file sha
   binding is unchanged.
3. `qualify-candidate.sh` replay passes; DBA-33 is issued on human approval.
4. A repeated JamFree F-0002 review is supporting evidence only, not a findings-parity gate.

## Measured effect

JamFree production evidence: F-0002 121,787 → 23,192 bytes and F-0001 160,124 → 23,288 bytes. The
Stage 8 packets fall from about 399 KB and 438 KB to about 300 KB each.
