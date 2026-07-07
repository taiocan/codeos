# DBA Session Start

<!--
HOW TO USE THIS FILE:
Paste the filled-in version of this prompt at the start of every Claude Code session.
Fill in the [BRACKETS] sections before pasting.
Claude will read .codeos/dba-system.md and orient itself for the session.
-->

---

You are operating in **Declarative Behavioral Architecture (DBA)** mode for this project.

**Step 1:** Read `.codeos/dba-system.md` now (the authoritative DBA doctrine). Confirm you have read it by stating the 3 non-negotiable rules.

**Step 2:** Read the project `CLAUDE.md` (in the project root). Note the Active Features table.

**Step 2b:** If `docs/codebase-digest.md` exists in the project root, read it now. Confirm you have read it by stating: "Structural digest read — [N] critical hubs, [N] god functions noted." If it does not exist, state: "No structural digest found — proceeding without structural orientation."

**Step 3:** Generate the Current Verified State block before proceeding. Run each command below and report the output.

**3a — Repository state:**
```bash
git branch --show-current
git rev-parse --short HEAD
git status --short
```
Report:
- Branch: `<output>`
- Commit: `<output>`
- Working tree: clean / dirty — list every modified and untracked file if any

**3b — Active feature state:**

If `features/registry.yaml` exists, read it. For each feature whose status is not COMPLETE:
- Stage ≥ 1 → check that `intents/<feature_id>*.md` exists.
- Stage ≥ 2 → check that `contracts/<feature_id>*.md` exists.
- Stage ≥ 3 → check that `events/<feature_id>*.md` exists.

If any expected artifact file is absent, or if an artifact is present but the registry reports a stage that does not require it yet: **STOP immediately** and report the specific mismatch to the human before continuing. Do not silently resolve the disagreement.

If `features/registry.yaml` does not exist: read the Active Features table from `CLAUDE.md` and report it as-is.

Report:
- Active feature: `<feature_id>` or `none`
- Current stage: `<N>` or `unknown`
- Registry/filesystem: `match` or `MISMATCH — <describe>`

**3c — Artifact directories:**
```bash
ls intents/   2>/dev/null || echo "(none)"
ls contracts/ 2>/dev/null || echo "(none)"
ls events/    2>/dev/null || echo "(none)"
ls tests/     2>/dev/null || echo "(none)"
```
Report each directory as a one-line list of filenames.

After completing 3a–3c, state:
`CURRENT STATE VERIFIED — Branch: <branch>, Commit: <sha>, Tree: <clean|dirty>, Feature: <id|none>, Stage: <N|unknown>`

**Repair-Before-Next-Feature check:** Before proceeding to Step 4, check whether the
current feature (if any) has any of the following unresolved:

- Unresolved Stage 7 GAP / MISMATCH / MISSING
- Stage 8 replay failure
- Required Stage 9 refinement not yet done
- Stage 10 structural blocker
- Failing CI
- Unresolved reviewer BLOCK
- Unresolved pre-release blocker

If any of the above are unresolved and the intent is to start a **new behavioral feature**,
surface this to the human before proceeding. Routing:
- Behavioral issue (Stage 7/8/9) → Stage 9 targeted refinement, or rerun the affected stage.
- Structural issue → Stage 10 architectural refinement.
- Release / package issue → Readiness checklist / resolve the release blocker.

**Human override:** The human may suspend the current feature and start another. If so, the
suspended feature must be marked blocked / incomplete — its evidence chain is not abandoned.

Then proceed to Step 4.

---

**Step 4:** Determine the session type from the options below and confirm it:

---

### Session Type

**A — Feature Brief (new feature discovery)**
Use when: starting a new feature, uncertain about scope, or the feature has not been written as a DBA intent yet.
Prompt to load: `.codeos/prompts/00b-feature-brief.md`
Output: a completed Feature Brief in `backlog/[feature_id].md`, ready to become Stage 1 input.

**B — Feature Stage Work (continuing or starting a feature in the DBA loop)**
Use when: a feature has an approved intent (or brief) and you are advancing through Stages 1–9.
Prompt to load: the appropriate stage prompt for the feature's current stage.

**C — Architectural Refinement**
Use when: the change is structural, not behavioral — workspace restructuring, shared library extraction, dependency consolidation, test infrastructure, naming normalization.
No behavioral contract or event schema is required.
Prompt to load: `.codeos/prompts/10-arch-refine.md`

**D — Existing Codebase Onboarding**
Use when: working code exists in `modules/` but has no approved DBA artifacts.
Goal: produce draft Feature Briefs and Intents for existing modules so they can enter the Stage 1 review queue.
Prompt to load: `.codeos/prompts/00c-onboarding.md`
Output: `HYPOTHESIZED_INTENT` draft briefs in `backlog/` and draft intents in `intents/` + registry entries. None are APPROVED — all require Stage 1 review before advancing.

**E — Solution Discovery** *(optional, advisory)*
Use when: exploring a new problem domain before writing individual Feature Briefs — to map candidate feature families, shared vocabulary, event hypotheses, configuration needs, and architectural risks.
This session type is **optional and non-gating**. Features may enter Stage 1 (Intent Capture) without a prior Solution Discovery session. The standard DBA path — Intent → Contract → Schema → Implement → Tests → Runtime → Reconcile → Replay → Refinement — is unchanged.
Prompt to load: `.codeos/prompts/00a-solution-discovery.md`
Output: non-authoritative planning material only. No output from Session Type E is an approved DBA artifact. Findings outside the session's stated scope are recorded as backlog candidates, not automatically incorporated.

---

**Step 5:** Check the feature registry. If `features/registry.yaml` exists, read it and report the current status of all features. State any features that are blocked on approval.

**Step 6:** Use the following session context:

## Session Context

**Today's goal:**
[Human fills in: e.g., "Complete Stage 2 contract for user_login feature"]

**Session type:** [A / B / C / D / E — from Step 4]

**Current feature or refinement:**

| ID | Type | Current Stage | Status |
|---|---|---|---|
| [feature_id] | [Feature / Arch Refinement] | [Stage N or step name] | [DRAFT/APPROVED/IN_PROGRESS] |

**This session's scope:**
[Human fills in: e.g., "Only work on user_login. Do not touch payment features."]

**Any session-specific forbidden actions:**
[Human fills in, or "none"]

**Structural context** (if `docs/codebase-digest.md` exists):
[Human fills in, or "see codebase-digest.md" — e.g., "We will be modifying cmd_export (god function, 30 fan-out). Contract clause required before touching it."]

---

**Library documentation:** When working on implementation stages (4, 5) or architectural refinements that involve external libraries, use the **Context7 MCP tool** to fetch current library documentation rather than relying on training data. This applies especially to Rust crates (Tokio, Serde, Reqwest, Clap) and any dependency where API surface or version compatibility matters. Invoke with the library name — Context7 returns current, version-accurate docs.

---

**Forbidden action (structural):** Behavioral modification of a function listed in `docs/codebase-digest.md` as a Critical Hub or God Function requires explicit contract coverage for that change. Non-behavioral modifications (performance, extraction of helpers, testability work) that do not change observable outputs are exempt.

**Step 7:** After reading and confirming, state:
- You are in DBA mode
- The session type (A, B, C, D, or E)
- Which feature(s) or refinement you'll work on
- Which stage or step you are currently at
- What you will produce this session

Then **STOP** and wait for the human to begin.

Do not start producing artifacts, writing code, or analyzing anything until the human explicitly says to proceed.
