# WO-04 — DBA Impact-Aware Adoption

PROTECTED · after WO-02, reusing its dependency model

## Goal

Adopting a DBA re-runs only what its semantic change affects.

## Direction

- Compare the old and new configurations' semantic closures, and mechanically derive which bindings
  and checkpoints change. No manual "invalidates" declarations.
- Accepted features that are unaffected stay valid.
- Operating rule, not a mechanism: projects adopt DBA versions between feature cycles.

## Verification

Replay JamFree's DBA-32 → 33 → 34 adoptions, and show which re-runs were necessary.
