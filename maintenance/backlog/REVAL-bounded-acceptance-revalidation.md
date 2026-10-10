# REVAL — Bounded Acceptance Revalidation

Classification: decided at planning; likely PROTECTED (touches acceptance validity) · Source:
ECP WO-06 gap E

## Concrete gap

`status` reports accepted features stale by `REVALIDATION_REQUIRED` with action
`revalidate_acceptance` (AUTO) and no command. On JamFree, F-0002–F-0004 show "revalidate only
the affected accepted surface", but no workflow mechanism performs that bounded revalidation.

## Goal

A workflow/revalidation capability, owned outside ECP, that revalidates only the changed accepted
surface under existing acceptance authority. Once it exists, ECP-5 may give
`revalidate_acceptance` a sufficient command. ECP must not manufacture one before then.
