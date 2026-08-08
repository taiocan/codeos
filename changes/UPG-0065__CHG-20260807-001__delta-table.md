# Normative Delta Inventory — `dba-system.md` @ commit `77599e9`

Per `changes/UPG-0065__CHG-20260807-001__normative-delta-inventory.md`'s Change Intent and
Acceptance Criteria. Every normative rule currently in `dba-system.md` gets exactly one row.
`dba-system-lean.md` is comparison evidence only — its content informs `disposition` and
`proposed_rule` where it materially conflicts, but the inventory's completeness universe is
`dba-system.md` alone. All `source_anchor` line numbers are pinned to commit `77599e9`.

Disposition precedence: `RETIRE` (zero-semantic-loss duplicate) → `INTENTIONAL-BEHAVIOR-CHANGE`
(meaning changes, regardless of relocation) → `KEEP-IN-CORE` (meaning preserved, stays in
doctrine) → `MOVE` (meaning preserved, relocates to a named non-doctrine component).

Six `target_owner` values in use: `doctrine`, `review policy`, `architecture-synthesis policy`,
`implementation-profile policy`, `controlled-plain-english policy`, `reviewer tool contract`.
No new candidate component was found to earn its place under Invariant 4 during this pass.

Full `proposed_rule` / `requires_human_decision` text for every `INTENTIONAL-BEHAVIOR-CHANGE` row
is in Part 2, below the main table, keyed by `rule_id`.

---

## Part 1 — Disposition Table

### Section: Mode Declaration (L1-16)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| MODE-1 | Mode Declaration | L15: "Read this file fully at the start of every session before doing anything else." | Read `dba-system.md` fully at session start, before any other action. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Lean's "Before Work" proposes the opposite discipline (read only what the current task needs; don't load everything by default) — direct conflict on the same topic, not mere terseness. |

**Section coverage**: 1 normative rule found (MODE-1). L1 (title), L3-7 (blockquote describing what the file is/its relationship to `CLAUDE.md`), L11 ("You are operating in DBA mode..."), and L13 (toolkit location) are explanatory/positional context, not independent obligations — no rows.

---

### Section: Truth Authority and Conflict Resolution (L19-29)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| TRUTH-AUTHORITY-1 | Truth Authority and Conflict Resolution | L23: "Explicit human correction (at any stage gate) overrides all other sources." | Explicit human correction at any gate is the top-priority authority. | KEEP-IN-CORE | doctrine | Lean Authority #1 ("The human's current explicit decision") — same substance. |
| TRUTH-AUTHORITY-2 | Truth Authority and Conflict Resolution | L24: "Runtime behavior... overrides intent text when behavior is more specific." | Observed runtime behavior can override/amend intent text when more specific than the text. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Matches the "Authority" row of the brief's known-deltas table — lean treats approved artifacts as authoritative for required behavior; runtime shows only what the system *does*, it does not silently amend what the text says *should* happen. |
| TRUTH-AUTHORITY-3 | Truth Authority and Conflict Resolution | L25: "Safety, authorization, and invariant-enforcement logic always preserves intent primacy regardless of runtime behavior." | Runtime evidence never overrides safety/authorization/invariant-enforcement intent. | KEEP-IN-CORE | doctrine | Lean Authority: "Runtime evidence never overrides a safety, authorization, or data-integrity requirement." — same substance. |
| TRUTH-AUTHORITY-4 | Truth Authority and Conflict Resolution | L26: "Structural digest observations... do not override behavioral findings. They inform blast-radius estimates... only." | Structural/digest findings (fan-in, god functions, risk zones) never override behavioral findings; informational only. | KEEP-IN-CORE | doctrine | Lean doesn't mention structural-digest observations at all — silent, not contradicted. |
| TRUTH-AUTHORITY-5 | Truth Authority and Conflict Resolution | L27 (clause 1): "An approved Architecture Baseline and Cohort Logical Design... are authoritative only for project-level structural decisions not fixed by rules 1–4's behavioral artifacts." | Baseline/Logical Design authority is scoped to structural decisions not already fixed by rules 1-4. | KEEP-IN-CORE | doctrine | This is a truth-authority interaction rule (not gate mechanics), stays with core doctrine; lean's 4-level Authority list is consistent (approved artifacts rank above runtime/code). |
| TRUTH-AUTHORITY-6 | Truth Authority and Conflict Resolution | L27 (clause 2): "Neither ever overrides Intent, Contract, Event Schema, explicit human correction, or safety/authorization invariants." | Baseline/Logical Design never override Intent/Contract/Event Schema/human correction/safety invariants. | KEEP-IN-CORE | doctrine | Not contradicted; independently statable safeguard from TRUTH-AUTHORITY-5. |
| TRUTH-AUTHORITY-7 | Truth Authority and Conflict Resolution | L27 (clause 3): "Conflicts with runtime evidence are handled through rule 2 above, not a separate baseline- or logical-design-specific rule — runtime behavior does not silently amend either artifact, and neither silently overrides runtime-confirmed intent drift." | Baseline/Logical-Design vs. runtime-evidence conflicts route through TRUTH-AUTHORITY-2's rule; no separate resolution lane. | KEEP-IN-CORE | doctrine | Cross-reference/no-new-precedence-lane rule; independently meaningful from TRUTH-AUTHORITY-5/6. Note: since TRUTH-AUTHORITY-2 is itself flagged `INTENTIONAL-BEHAVIOR-CHANGE`, this row's cross-reference target moves together with it if adopted — noted, not a separate decision. |
| TRUTH-AUTHORITY-8 | Truth Authority and Conflict Resolution | L29: "When a conflict cannot be resolved by these rules: surface it clearly to the human rather than silently resolving it." | Unresolvable conflicts are surfaced to the human, never silently resolved. | KEEP-IN-CORE | doctrine | Lean: "stop and present the decision in one short question." — same substance, terser phrasing. |

**Section coverage**: 8 normative rules found (TRUTH-AUTHORITY-1 through 8). L21 (lead-in framing the list) is non-normative.

### Section: The Non-Negotiable Rules (L33-40)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| NN-1 | The Non-Negotiable Rules | L35: "Every stage transition requires explicit human approval. You NEVER advance... without a human 'APPROVED'..." | Every stage transition requires explicit human approval; never advance without it. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Matches "Stage transitions (Non-Negotiable Rule #1)" — the single most consequential flagged delta. Lean authorizes running Stages 4-8 as one uninterrupted delivery cycle after Event Schema approval. |
| NN-2 | The Non-Negotiable Rules | L36: "You NEVER implement before intent + contract + event schema are all approved." | Implementation is gated on all three (Intent, Contract, Event Schema) being approved. | KEEP-IN-CORE | doctrine | Lean Working Rule #1: "A human must approve Intent, Contract, and Event Schema." — same gate preserved. |
| NN-3 | The Non-Negotiable Rules | L37: "You NEVER add abstractions, patterns, or behaviors beyond what the current intent + contract + event schema specifies." | No abstractions/patterns/behaviors beyond what the three approved artifacts specify. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Matches "Stage 4 internal abstractions" — lean permits normal internal abstractions/error types/patterns provided observable behavior is unchanged. |
| NN-4 | The Non-Negotiable Rules | L38: "You NEVER emit events not listed in the approved event schema." | No events outside the approved event schema. | KEEP-IN-CORE | doctrine | Lean: "may emit only approved domain events" — same core prohibition preserved. Lean's added diagnostic-logs/metrics clarification is new content beyond this existing rule, not a change to it. |
| NN-5 | The Non-Negotiable Rules | L39: "You NEVER invent hidden behavior — all behavior must be traceable to an approved artifact." | No hidden/untraceable behavior; everything traces to an approved artifact. | KEEP-IN-CORE | doctrine | No direct lean restatement, but not contradicted; related to but distinct from lean's Working Rule #6 (missing-decision handling). |
| NN-6 | The Non-Negotiable Rules | L40: "After producing any stage output, you STOP and state: `AWAITING HUMAN APPROVAL`." | After any stage output, stop and state the literal phrase "AWAITING HUMAN APPROVAL". | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Coupled to NN-1: lean's batching means this no longer fires after *every* stage 4-8 output, and lean does not mandate this specific literal phrase — stops for approval at fewer, different points, described but not phrase-pinned. |

**Section coverage**: 6 normative rules found (NN-1 through 6), matching the six numbered items exactly — no lead-in prose to account for.

---

### Section: Default Advisory Review (L44-105)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| REVIEW-1 | Default Advisory Review | L46-49: "Advisory review runs by default at every reviewable gate across the whole workflow below..." | Independent review runs by default at every reviewable gate, project-wide. | INTENTIONAL-BEHAVIOR-CHANGE | review policy | Matches "Independent review" — lean makes review conditional on named trigger conditions, not default-everywhere. |
| REVIEWER-TOOL-1 | Default Advisory Review | L51-56: "How to run it... run `.codeos/scripts/codeos-review.sh review <feature_id> <stage>`..." | The exact invocation syntax and worked examples for running the reviewer before a gate decision. | MOVE | reviewer tool contract | Tool invocation mechanics, not review policy; lean doesn't specify tool syntax at all — silent. |
| REVIEW-2 | Default Advisory Review | L57-60: "The reviewer is independent, read-only, and non-gatekeeping — its verdict... informs the human's decision but never auto-blocks. The human decides at the gate..." | Reviewer is independent/read-only/advisory-only; verdict informs, never blocks; human decides. | MOVE | review policy | Lean: "A reviewer advises; it does not add requirements or control the gate." — same substance. Note: this row's closing clause ("Non-Negotiable Rule #1 is unchanged") is a cross-reference whose truth depends on NN-1's own disposition — flagged as an interaction, not treated as its own row. |
| REVIEWER-TOOL-2 | Default Advisory Review | L62-68: "The wrapper is the supported entry point... automatically resolve and inject this project's Controlled Plain English status... Invoking the compiled `codeos-reviewer` binary directly bypasses that injection..." | The `codeos-review.sh` wrapper is the sole supported entry point for CPE-status injection; the raw binary bypasses it. | MOVE | reviewer tool contract | Tool/wrapper mechanics; lean has no CPE section at all — silent, not contradicted. |
| REVIEW-3 | Default Advisory Review | L70-72: "Round 1 runs before the gate. Rounds 2-3 are allowed... After 3 rounds, stop and require a human decision..." | Default round budget: R1 + up to 2 more rounds (3 total), then escalate to human. | INTENTIONAL-BEHAVIOR-CHANGE | review policy | Lean Review Policy: "Use one review pass by default. Use one focused retry after a material fix." — 2 rounds total by default, not 3; genuine budget delta. |
| REVIEW-4 | Default Advisory Review | L74-79: "Solution Discovery is reviewed conditionally, not unconditionally... A Discovery session whose output nobody acts on is simply never reviewed." | Discovery is reviewed only if/when its output is actually carried into an approved artifact. | MOVE | review policy | Lean: "Review discovery material only when a claim from it is carried into an approved artifact." — same substance. |
| REVIEW-5a | Default Advisory Review | L81-83: "If reviewer tooling is unavailable or not configured for this project, the human records an explicit waiver with a reason and may continue..." | A waiver is allowed when reviewer tooling is unavailable or unconfigured. | MOVE | review policy | Independently changeable from the recording, silent-skip, and blocking facts. Lean silent — not contradicted. |
| REVIEW-5b | Default Advisory Review | L83-84: "...skipping the review silently is not allowed..." | Skipping a review silently is never allowed. | MOVE | review policy | Independently changeable from the eligibility and recording facts. Lean silent — not contradicted. |
| REVIEW-5c | Default Advisory Review | L84-85: "...and neither is blocking the whole project over missing reviewer setup." | Blocking the whole project over missing reviewer setup is never allowed. | MOVE | review policy | Independently changeable from the other three facts. Lean silent — not contradicted. |
| REVIEW-5d | Default Advisory Review | L85-86: "Record it as a plain entry in that feature's review log: 'Review waiver: reviewer not configured for this project; proceeding without advisory review at `<stage>`. Reason: `<text>`.'" | A waiver is recorded as a plain review-log entry in a specified format. | MOVE | review policy | Independently changeable from the eligibility, silent-skip, and blocking facts. Lean silent — not contradicted. |
| REVIEW-6 | Default Advisory Review | L86-88 (clause 4): "The waiver applies only to the advisory review run. It never waives Non-Negotiable Rule #1..." | A waiver never waives the human-approval gate itself (NN-1). | KEEP-IN-CORE | doctrine | Protects a core doctrine invariant (the approval gate), general enough to stay in doctrine even though the surrounding waiver mechanism MOVEs to review policy. |
| REVIEW-7 | Default Advisory Review | L90-98: "Verification round-trip... the acting agent may run `.codeos/prompts/verify-only.md`'s read-only verification pass... optional and judged by the acting agent... does not replace [human decision]." | Agent may optionally run a read-only verification pass targeting a reviewer's named uncertainty, as evidence for the next round; never mandatory, never a substitute for human decision. | MOVE | review policy | Lean has no equivalent "verification round-trip" mechanism — silent. |
| REVIEW-8 | Default Advisory Review | L101-105: "Relationship to the Reviewer Activation Package... optional, supplementary independent critical-assessor pass... does not replace the default review above." | `pipeline-reviewer.md` remains an optional supplementary second-opinion pass, not a replacement for the default review. | MOVE | review policy | Lean has no equivalent supplementary-package concept — silent. |

**Section coverage**: 13 normative rules found (REVIEW-1 through 4, 5a-d, 6 through 8,
REVIEWER-TOOL-1 through 2). Corrected during Step 4 review (Codex R3): `REVIEW-5` bundled waiver
eligibility, the silent-skip prohibition, the project-blocking prohibition, and the recording
format — four independently-changeable facts, split into 5a-d.

### Section: The 9-Step DBA Development Loop (L109-162)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| LOOP-SEQ-1 | The 9-Step DBA Development Loop | L111: "Every feature follows this exact sequence. No skipping." | Every feature follows the 9-step sequence in order; no skipping steps. | KEEP-IN-CORE | doctrine | Lean's Nine Stages retain the same ordered sequence and Stage 9 routing-to-earliest-owning-stage discipline; only approval *cadence* changes (captured at NN-1), not step *order*. |
| LOOP-SEQ-2 | The 9-Step DBA Development Loop | L112: "Run the default advisory review... before each gate below." | Default advisory review applies to gates in the 9-step loop specifically. | RETIRE | — | Duplicate of REVIEW-1, which already states the general rule covers "not only the numbered Stage 1-9 loop"; this line adds no independent content. |
| STEP1-ACTIVITY | The 9-Step DBA Development Loop | L118-120: "STEP 1 — Intent: Human writes raw feature description. AI verifies, corrects format, flags missing information." | Human authors raw intent; AI verifies/corrects format/flags gaps. | KEEP-IN-CORE | doctrine | Lean Stage 1 elaborates content requirements without contradicting this division of labor. |
| STEP1-OUTPUT | The 9-Step DBA Development Loop | L121: "Output: intents/[feature_id].md" | Stage 1 output path. | KEEP-IN-CORE | doctrine | Identical path in lean Stage 1. |
| STEP1-GATE | The 9-Step DBA Development Loop | L122: "Gate: human approves intent before step 2." | Human approves Intent before Stage 2. | KEEP-IN-CORE | doctrine | Lean explicitly preserves the Stage 1→2 gate. |
| STEP2-ACTIVITY | The 9-Step DBA Development Loop | L124-125: "STEP 2 — Behavioral Contracts: AI derives BDD-style contracts from approved intent." | AI derives BDD-style contracts from approved Intent. | KEEP-IN-CORE | doctrine | Lean Stage 2 elaborates coverage detail without contradicting. |
| STEP2-OUTPUT | The 9-Step DBA Development Loop | L126: "Output: contracts/[feature_id]_contract.md" | Stage 2 output path. | KEEP-IN-CORE | doctrine | Identical path in lean. |
| STEP2-GATE | The 9-Step DBA Development Loop | L127: "Gate: human approves contracts before step 3." | Human approves Contract before Stage 3. | KEEP-IN-CORE | doctrine | Lean preserves the Stage 2→3 gate. |
| STEP3-ACTIVITY | The 9-Step DBA Development Loop | L129-131: "STEP 3 — Event Schema: AI defines the complete event spine from approved intent + contracts. This is the most constraining artifact — implementation is locked to it." | AI defines the complete event spine; it locks Stage 4 implementation. | KEEP-IN-CORE | doctrine | Lean Stage 3 elaborates without contradicting. |
| STEP3-OUTPUT | The 9-Step DBA Development Loop | L132: "Output: events/[feature_id]_schema.md (or events/event_schema.md)" | Stage 3 output path. | KEEP-IN-CORE | doctrine | Identical path in lean. |
| STEP3-GATE | The 9-Step DBA Development Loop | L133: "Gate: human approves schema before step 4." | Human approves Event Schema before Stage 4. | KEEP-IN-CORE | doctrine | Lean preserves this gate ("Approval of the Event Schema authorizes..." presupposes it exists). |
| STEP4-ACTIVITY | The 9-Step DBA Development Loop | L135-136: "STEP 4 — AI Implementation: AI implements ONLY what is specified by the three approved artifacts." | Implementation limited to exactly what the three approved artifacts specify. | RETIRE | — | Duplicate of NN-3; restates the same "no abstractions beyond spec" constraint with no independent content. |
| STEP4-OUTPUT | The 9-Step DBA Development Loop | L137: "Output: code in modules/" | Stage 4 output location. | KEEP-IN-CORE | doctrine | Lean: "Output: working code" — less specific but not contradictory. |
| STEP4-GATE | The 9-Step DBA Development Loop | L138: "Gate: human approves implementation before step 5." | Human approves implementation before Stage 5. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Collapsed by lean's batched Stage 3→8 cycle; see NN-1. |
| STEP5-ACTIVITY | The 9-Step DBA Development Loop | L140-141: "STEP 5 — Tests: AI writes behavioral tests and replay tests." | AI writes behavioral and replay tests. | KEEP-IN-CORE | doctrine | Lean Stage 5 elaborates without contradicting. |
| STEP5-OUTPUT | The 9-Step DBA Development Loop | L142: "Output: tests/behavioral/ and tests/replay/" | Stage 5 output paths. | KEEP-IN-CORE | doctrine | Identical paths in lean. |
| STEP5-GATE | The 9-Step DBA Development Loop | L143: "Gate: human approves tests before step 6." | Human approves tests before Stage 6. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Collapsed by the batched cycle; see NN-1. |
| STEP6-ACTIVITY | The 9-Step DBA Development Loop | L145-146: "STEP 6 — Runtime Execution: Human runs the implementation." | Human runs the implementation. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Matches "Stage 6 execution" — lean: the agent may run representative scenarios when the environment permits. |
| STEP6-EVENTS | The 9-Step DBA Development Loop | L147: "System emits events to events/runtime_events.jsonl (append-only)." | Runtime events are emitted to `events/runtime_events.jsonl`. | KEEP-IN-CORE | doctrine | Lean: "If the project uses `events/runtime_events.jsonl`, append new observations..." — same emission destination preserved. Not a duplicate of NEVER-DO-8: that row only states the modify-prohibition; this row is the only place the *emission destination itself* is stated, and retiring it as previously drafted would have silently dropped that fact — corrected during Step 3 review (Codex R1). |
| STEP7-ACTIVITY | The 9-Step DBA Development Loop | L149-151: "STEP 7 — AI Reconciliation Review: AI compares... Produces reconciliation table with ALIGNED / GAP / MISMATCH / MISSING status." | Reconciliation always produces a full table with status for every comparison, including aligned rows. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Newly identified (not in the original seed table): lean explicitly forbids restating aligned rows — "do not produce a large table of aligned rows," report only problems and supporting evidence. |
| STEP7-GATE | The 9-Step DBA Development Loop | L152: "Gate: human approves before step 8 or directs return to earlier step." | Human approves Stage 7's output before Stage 8, or directs return. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Collapsed into the batched Stage 3→8 cycle; see NN-1. |
| STEP8-ACTIVITY | The 9-Step DBA Development Loop | L154-155: "STEP 8 — Replay Verification: AI verifies runtime_events.jsonl conforms to schema and contract sequence." | Verify the runtime event log conforms to schema and contract sequence. | KEEP-IN-CORE | doctrine | Lean Stage 8 elaborates (correlation, determinism) without contradicting. |
| STEP8-GATE | The 9-Step DBA Development Loop | L156: "Gate: human approves before step 9 or directs return." | Human approves after Stage 8, before Stage 9. | KEEP-IN-CORE | doctrine | Preserved in lean — the batched cycle runs *through* Stage 8, after which "the human accepts the feature or requests refinement." This is the one post-batch gate that survives unchanged. |
| STEP9-ACTIVITY | The 9-Step DBA Development Loop | L158-160: "STEP 9 — Targeted Refinement: AI proposes the smallest effective change for each observed problem. Affected stages are re-run. No full rewrites." | Smallest effective change per problem; only affected stages re-run; no full rewrites. | KEEP-IN-CORE | doctrine | Lean Stage 9 is consistent, adding detail without contradiction. |
| STEP9-GATE | The 9-Step DBA Development Loop | L161: "Gate: human approves each refinement individually." | Every refinement requires individual human approval. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Matches "Stage 9 refinement" — lean allows a correction within already-approved behavior to proceed without a new product decision. |

**Section coverage**: 25 normative rules found (`LOOP-SEQ-1/2` plus 3 rows for each of Steps 1-3, 5, and 3 rows for Step 4, 2 rows each for Steps 6-9). L114 (the `[feature_id]` notation pointer) is explanatory, not a new obligation — no row.

### Section: Multi-Feature Architecture Synthesis Gate (L166-343)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| ARCH-GATE-1 | Multi-Feature Architecture Synthesis Gate | L168-171: "This is a conditional, project-level structural approval gate... not required for single-feature or loosely-coupled projects." | The gate is conditional; not required for single-feature/loosely-coupled projects. | MOVE | architecture-synthesis policy | Lean: "Use this gate when two or more features could constrain each other's..." — same conditionality. |
| ARCH-GATE-2 | Multi-Feature Architecture Synthesis Gate | L173-179: "A core architecture cohort is two or more features whose independent implementation choices could materially constrain each other's canonical ownership, dependency direction... Merely sharing a runtime or a database is evidence to inspect, not automatic inclusion." | The core-cohort test: material constraint on ownership/dependency/persistence/integration/infrastructure/topology; shared runtime/DB alone isn't automatic inclusion. | MOVE | architecture-synthesis policy | Lean: same test, condensed wording — "Sharing a runtime or database alone does not trigger the gate." |
| ARCH-GATE-3a | Multi-Feature Architecture Synthesis Gate | L181-182: "A human declares a core cohort by adding an `architecture_cohorts:` entry to `features/registry.yaml` (see the template's schema comments)..." | A cohort is declared by adding an `architecture_cohorts:` entry to `features/registry.yaml`. | MOVE | architecture-synthesis policy | Independently changeable from the uniqueness and registry-authority facts below. Lean doesn't specify a declaration mechanism — silent, not contradicted. |
| ARCH-GATE-3b | Multi-Feature Architecture Synthesis Gate | L183-184: "A feature belongs to at most one active cohort; a project may declare multiple cohorts, but their feature memberships must not overlap." | A feature belongs to at most one active cohort; declared cohorts' memberships must not overlap. | MOVE | architecture-synthesis policy | Independently changeable from the declaration mechanism and registry-authority facts. Lean silent — not contradicted. |
| ARCH-GATE-3c | Multi-Feature Architecture Synthesis Gate | L185-187: "This declaration is where cohort membership and gate status live — the registry remains an index (membership, status, baseline and logical design version *references*), never a second home for the structural decisions those artifacts themselves own." | The registry is an index only (membership, status, version references), never a second home for structural decisions. | MOVE | architecture-synthesis policy | Independently changeable from the declaration mechanism and uniqueness constraint. Lean silent — not contradicted. |
| ARCH-GATE-4 | Multi-Feature Architecture Synthesis Gate | L189-194: "Once a cohort is declared, every member feature reaching Stage 4 requires that cohort's Architecture Baseline and its Cohort Logical Design to both be `approved`... Intent, Contract, and Event Schema approval remain required exactly as before; the baseline and logical design are additional requirements..." | Cohort members need Baseline + Logical Design approved (current versions) before Stage 4, additive to Intent/Contract/Event Schema. | MOVE | architecture-synthesis policy | Lean gate step 4: "Only once `approved` may cohort members begin Stage 4" — same substance. |
| ARCH-GATE-5 | Multi-Feature Architecture Synthesis Gate | L196-208: "Wave Gates... the approval decision remains individual, per feature, per artifact version... A Wave Gate is never all-or-nothing... Checks... carry no decision authority of their own — the human still decides every approval." | Batched drafting/review is allowed, but the approval decision itself remains strictly individual per feature/artifact version — never a batch approval. | INTENTIONAL-BEHAVIOR-CHANGE | architecture-synthesis policy | Newly identified (not in the original seed table): lean explicitly allows "Approval may also be given as one explicit batch decision, provided the human can identify every included artifact" — a direct relaxation of the current "never all-or-nothing" rule. |
| ARCH-GATE-6 | Multi-Feature Architecture Synthesis Gate | L210-233: "The gate sequence. 1. Every cohort member completes Stage 1 (Intent)... The Intent Cohort Check..." — the full 6-step sequence (Intent Cohort Check → Contract Cohort Check → Event Cohort Check/Column Checks → Architecture Synthesis Steps 2-4), with per-stage Row/Column check apparatus. | The gate runs via a detailed 6-step sequence with per-stage Row Checks and a Column Check before each Wave Gate decision. | INTENTIONAL-BEHAVIOR-CHANGE | architecture-synthesis policy | Matches "Architecture governance" — lean condenses this to 4 steps (check cohort → draft Baseline → draft Logical Design → one human approval), producing the same two artifacts without the granular per-stage check apparatus. |
| ARCH-GATE-7a | Multi-Feature Architecture Synthesis Gate | L254-256: "The baseline may constrain implementation structure... integration style. It may never invent or alter behavior." | Baseline may constrain implementation structure but never invent or alter behavior. | MOVE | architecture-synthesis policy | Independently changeable from the gap-routing procedural fact below. Lean: "They must not repeat every feature artifact or invent behavior." — same constraint. |
| ARCH-GATE-7b | Multi-Feature Architecture Synthesis Gate | L256-258: "Any behavioral gap discovered during synthesis returns the affected feature to its owning Stage 1, 2, or 3 — it is never patched inside the baseline directly." | A behavioral gap found during synthesis returns to the feature's owning stage, never patched in the baseline. | MOVE | architecture-synthesis policy | Independently changeable from the never-invent-behavior constraint. Lean silent on this specific procedural consequence — not contradicted. |
| ARCH-GATE-8 | Multi-Feature Architecture Synthesis Gate | L260-272: "What the logical design may and may not do... may never invent or alter behavior either... does not restate or re-decide anything the baseline already settled..." | Logical Design may fix shared logical structure but never invent/alter behavior; never restates what the Baseline already settled. | MOVE | architecture-synthesis policy | Lean: "The Logical Design must not repeat the Baseline." — same constraint, plus the never-invent-behavior rule shared with ARCH-GATE-7a. |
| ARCH-GATE-9 | Multi-Feature Architecture Synthesis Gate | L274-278: "Authoritative decisions vs. derived views. The baseline distinguishes decisions a human manually approved... from matrices and inventories mechanically derived... regenerable, each carrying provenance... never a second canonical model that can silently drift..." | Human-approved decisions are distinguished from mechanically-derived, regenerable views; derived views never become a second silently-driftable canonical model. | MOVE | architecture-synthesis policy | Lean doesn't mention this distinction at all — silent, not contradicted; a genuinely independent safeguard, not folded into the ARCH-GATE-6 machinery-simplification delta. |
| ARCH-GATE-10 | Multi-Feature Architecture Synthesis Gate | L280-297: "Cohort, baseline, and logical design versioning... the superseded file moves to `architecture/history/core-baseline-v<version>.md`... before the new version is written... Historical files are a provenance record only..." | Approving a new baseline/logical-design version requires moving the superseded file to a `history/` path before writing the new one; registry fields updated in the same approval. | INTENTIONAL-BEHAVIOR-CHANGE | architecture-synthesis policy | Matches "Architecture governance" — lean: "Git history records superseded versions; duplicate history files and complex registry states are not required." |
| ARCH-GATE-11 | Multi-Feature Architecture Synthesis Gate | L299-304: "Targeted reassessment of Stage 1-3 approvals... a revision to one member's approved artifact identifies potentially affected already-approved artifacts... Only artifacts the assessment actually identifies are marked for reassessment..." | A revision triggers a targeted impact assessment; only artifacts actually identified are reassessed, never a full Wave Gate restart. | MOVE | architecture-synthesis policy | Lean doesn't mention this impact-assessment pattern — silent, not contradicted. |
| ARCH-GATE-12 | Multi-Feature Architecture Synthesis Gate | L306-314: "Compatibility rule for cohorts approved before this two-output model existed... treated... as `baseline-approved`... not `approved`, for the purpose of any *new* Stage 4 entry..." | A cohort `approved` under the old single-output rule is reinterpreted as `baseline-approved` (not `approved`) for new Stage 4 entry, until a Logical Design is also approved. | MOVE | architecture-synthesis policy | One-time transitional clause tied to `UPG-0058`'s own rollout; lean (a fresh proposal) doesn't address migration state. Not contradicted — may become `RETIRE`-eligible once confirmed discharged project-wide, which this inventory does not determine. |
| ARCH-GATE-13 | Multi-Feature Architecture Synthesis Gate | L316-328: "Verifying a `baseline_version` or `logical_design_version` reference... valid... only if it equals... the current `Baseline version` field exactly... A value that instead matches a file under `architecture/history/` is stale, not valid..." | Live Stage 4 eligibility requires the registry's pinned version to equal the artifact's *current* version exactly; a historical-file match is stale, not valid. | MOVE | architecture-synthesis policy | Lean doesn't specify this granular validation mechanic — silent, not contradicted. |
| ARCH-GATE-14 | Multi-Feature Architecture Synthesis Gate | L330-335: "Reviewer coverage. `codeos-reviewer` has a dedicated checklist for the `architecture-synthesis` stage id... run `.codeos/scripts/codeos-review.sh review <feature_id> architecture-synthesis`... does not weaken Non-Negotiable Rule #1." | The reviewer has a dedicated `architecture-synthesis` stage-id checklist, invoked the same way as any other stage. | MOVE | reviewer tool contract | Reviewer-checklist mechanics, not architecture policy per se; lean doesn't mention reviewer stage-id checklists — silent. |
| ARCH-GATE-15 | Multi-Feature Architecture Synthesis Gate | L337-343: "Naming. This is the Architecture Synthesis Gate... This gate is the opposite: conditional but, once triggered, mandatory, and it runs only after Stage 3 approval across the whole cohort..." | Once triggered by cohort declaration, the gate is mandatory; runs only after every current cohort member reaches Stage 3. | MOVE | architecture-synthesis policy | Lean's conditionality framing matches ("Use this gate when...", runs "after the cohort's Stage 3 artifacts are approved"). |

**Section coverage**: 18 normative rules found (ARCH-GATE-1, 2, 3a-c, 4, 5, 6, 7a-b, 8-15).
Introductory/naming-distinction prose not itself stating a testable rule (e.g. "not 'Architecture
Discovery'") is folded into ARCH-GATE-15's rationale, not a separate row. Corrected during Step 4
review: Codex R1 found `ARCH-GATE-3` bundled the declaration mechanism, the one-cohort-per-feature
constraint, and the registry's index-only role, split into 3a-c; Codex R3 found `ARCH-GATE-7`
bundled the never-invent-behavior constraint with a separate procedural gap-routing fact, split
into 7a-b.

### Section: Implementation Profile (L346-421)

Lean has no equivalent section at all (confirmed at UPG-0065 filing time: "Implementation Profile:
Missing" in lean vs. current). Every row below is therefore `MOVE`/meaning-preserved unless noted
— lean's silence is not a contradiction under the Inventory Universe rule.

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| IMPL-PROFILE-1a | Implementation Profile | L348-350: "DBA artifacts through Stage 3 are language-neutral, and this stays true. A project may declare an Implementation Profile... that Stage 4 must consult once approved." | A project may declare an Implementation Profile; Stage 4 must consult it once approved. | MOVE | implementation-profile policy | Independently changeable from whether the mechanism is independent/optional and from its relationship to the Architecture Gate. Lean silent — not contradicted. |
| IMPL-PROFILE-1b | Implementation Profile | L350-351: "This is an independent, optional mechanism: it has no dependency on the Multi-Feature Architecture Synthesis Gate above..." | The Implementation Profile mechanism is optional and independent of the Architecture Synthesis Gate. | MOVE | implementation-profile policy | Independently changeable from the consultation trigger and the single-feature-project fact. Lean silent — not contradicted. |
| IMPL-PROFILE-1c | Implementation Profile | L351-352: "...and works identically for a single-feature project that never declares a core cohort." | The mechanism works identically for single-feature projects with no declared cohort. | MOVE | implementation-profile policy | Independently changeable from the other two facts — this is a scope-of-applicability statement, not the mechanism's existence or its Gate-independence. Lean silent — not contradicted. |
| IMPL-PROFILE-2 | Implementation Profile | L354-358: "Lifecycle. A profile has exactly one non-binding pre-approval state: `proposed → approved → superseded`. A profile becomes `approved` only following an explicit human approval decision; setting `status: approved`... does not constitute it..." | Lifecycle states `proposed → approved → superseded`; approval requires an explicit human decision, not merely writable fields. | MOVE | implementation-profile policy | Lean silent — not contradicted. |
| IMPL-PROFILE-3a | Implementation Profile | L360-362: "Once `approved`, a profile version is never edited in place. The current path, `architecture/implementation-profile.yaml`, always holds only the current approved version..." | An approved profile version is never edited in place. | MOVE | implementation-profile policy | Independently changeable from where proposals are drafted and how promotion is ordered. Lean silent — not contradicted. |
| IMPL-PROFILE-3b | Implementation Profile | L364-367: "A material change (language, applicability, exceptions) is drafted at `architecture/proposals/implementation-profile-v<N>.yaml`... while the current approved version remains binding, unaffected, at the current path." | A material change is drafted at `architecture/proposals/implementation-profile-v<N>.yaml`, not affecting the current binding version. | MOVE | implementation-profile policy | Independently changeable from the immutability rule and the promotion ordering. Lean silent — not contradicted. |
| IMPL-PROFILE-3c | Implementation Profile | L367-370: "On explicit human approval, in this order: (1) the old current file moves to `architecture/history/implementation-profile-v<old-N>.yaml` and its `status` becomes `superseded`; (2) the proposal is promoted to `architecture/implementation-profile.yaml` with `status: approved`." | Promotion on approval follows a strict order: old file moves to history first, then the proposal is promoted. | MOVE | implementation-profile policy | Independently changeable from the immutability rule and the proposal-drafting location. Lean silent — not contradicted. |
| IMPL-PROFILE-4a | Implementation Profile | L377-379: "Stage 4 must verify that the implementation is covered by an *approved* Implementation Profile or a recorded exception, within the profile's resolved scope." | Stage 4 must verify coverage by an approved Implementation Profile or a recorded exception. | MOVE | implementation-profile policy | Independently changeable from the tech-pattern's advisory status and the no-new-gate fact. Lean silent — not contradicted. |
| IMPL-PROFILE-4b | Implementation Profile | L379-381: "The corresponding technology pattern's recommendations (e.g. `.codeos/patterns/rust-project-structure.md` when `primary_language: rust`) remain advisory — consulted, never overriding an approved Architecture Baseline or another project-specific decision." | The matching technology pattern's recommendations remain advisory, never overriding the Baseline. | MOVE | implementation-profile policy | Independently changeable from the Stage 4 verification requirement and the no-new-gate fact. Lean silent — not contradicted. |
| IMPL-PROFILE-4c | Implementation Profile | L381-382: "No additional human gate is introduced beyond the existing Stage 4 approval, and no new Stage ID is needed." | No new human gate or Stage ID is introduced by this mechanism. | MOVE | implementation-profile policy | Independently changeable from the other two facts. Lean silent — not contradicted. |
| IMPL-PROFILE-5 | Implementation Profile | L384-389: "Resolvable scope — not free text. `applies_to.scope` is one of `all \| feature_ids \| cohort_ids`... A profile whose `scope` is `feature_ids` or `cohort_ids` must leave the other list empty..." | Scope is one of three enumerated kinds; the unused selector list must be empty. | MOVE | implementation-profile policy | Lean silent — not contradicted. |
| IMPL-PROFILE-6a | Implementation Profile | L391-394: "An exception uses the same selector model... to override the profile's `primary_language` for a specific feature or cohort... When a feature matches both a feature-level and a cohort-level exception, the feature-level exception wins (more specific)." | A feature-level exception wins over a cohort-level exception for the same feature. | MOVE | implementation-profile policy | Independently changeable from the same-specificity conflict rule. Lean silent — not contradicted. |
| IMPL-PROFILE-6b | Implementation Profile | L395-397: "Multiple matching exceptions at the *same* specificity that disagree make the profile invalid for that feature — Stage 4 treats it as ineligible rather than picking one arbitrarily." | Conflicting same-specificity exceptions make the profile invalid for that feature; Stage 4 treats it as ineligible. | MOVE | implementation-profile policy | Independently changeable from the specificity-precedence rule. Lean silent — not contradicted. |
| IMPL-PROFILE-7 | Implementation Profile | L399-404: "Profile–Architecture Baseline consistency... an unreconciled contradiction... Stage 4 must treat it as ineligible, not silently prefer either artifact..." | A profile/baseline language contradiction with no matching exception makes Stage 4 ineligible; never silently resolved. | MOVE | implementation-profile policy | Lean silent — not contradicted. |
| IMPL-PROFILE-8 | Implementation Profile | L406-409: "Provenance. When an approved profile applies to a feature, Stage 4's output records the `profile_id`, `profile_version`, the resolved language, and any matched exception..." | Stage 4 output must record profile provenance fields when a profile applies. | MOVE | implementation-profile policy | Lean silent — not contradicted. |
| IMPL-PROFILE-9a | Implementation Profile | L411-412: "Codeos's default policy, stated honestly about what exists today: Codeos recommends a rust-first profile as the default for new projects." | Codeos recommends a rust-first Implementation Profile as the default for new projects. | MOVE | implementation-profile policy | Independently changeable from the scaffold-behavior fact below (the recommendation could change language without changing how scaffolding works, or vice versa). Lean silent — not contradicted. |
| IMPL-PROFILE-9b | Implementation Profile | L412-415: "`dba-init.sh` scaffolds `architecture/implementation-profile.yaml` from `.codeos/templates/implementation-profile.yaml` automatically for every new project, always as `status: proposed` — never pre-approved." | `dba-init.sh` scaffolds the profile automatically, always `proposed`, never pre-approved. | MOVE | implementation-profile policy | Independently changeable from the rust-first recommendation. Lean silent — not contradicted. |
| IMPL-PROFILE-10 | Implementation Profile | L417-420: "Reviewer coverage. This mechanism introduces no new Stage ID. Consultation changes to Stage 4 are covered by the existing Stage ID `4`... no Review Waiver note is needed..." | No new Stage ID; existing Stage 4/onboarding IDs already cover consultation changes. | MOVE | implementation-profile policy | Lean silent — not contradicted. |

**Section coverage**: 18 normative rules found (IMPL-PROFILE-1a-c, 2, 3a-c, 4a-c, 5, 6a-b, 7, 8,
9a-b, 10). Corrected during Step 4 review: Codex R1 found `IMPL-PROFILE-9` bundled the rust-first
recommendation with `dba-init.sh`'s scaffold behavior — split into 9a-b; `IMPL-PROFILE-1` bundled
the mechanism's existence/consultation trigger, its Gate-independence, and its
single-feature-project applicability; `IMPL-PROFILE-4` bundled the Stage 4 verification
requirement, the tech-pattern's advisory status, and the no-new-gate fact — split into 1a-c and
4a-c. Codex R2 found `IMPL-PROFILE-3` bundled the immutability rule, the proposal-drafting
location, and the promotion ordering — split into 3a-c. Codex R3 found `IMPL-PROFILE-6` bundled
exception specificity-precedence with same-specificity conflict handling — split into 6a-b; R3
also found `FILE-LAYOUT-2a`'s "duplicate of `IMPL-PROFILE-3a`" claim was false (3a states only
immutability, not the current-version path) — `FILE-LAYOUT-2a` corrected to `KEEP-IN-CORE`.

### Section: Contract-to-Implementation Failure Boundary (L424-460)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| FAILURE-BOUNDARY-1 | Contract-to-Implementation Failure Boundary | L431-435: "Two boundaries, kept distinct. The behavioral boundary is... jointly by the Stage 2 Contract's Failure Classifications and the Stage 3 Event Schema. The technical API boundary is implementation-internal error propagation..." | Behavioral boundary (Contract + Event Schema) vs. technical API boundary (internal error types) are kept distinct. | KEEP-IN-CORE | doctrine | Section explicitly states it applies existing NN-2/NN-4, not a new rule; lean doesn't restate this detail but doesn't contradict it. |
| FAILURE-BOUNDARY-2 | Contract-to-Implementation Failure Boundary | L437-442: "The rule. Only failure classifications approved by the Stage 2 Contract may be exposed as classified behavioral outcomes. A failure event may be emitted only when that event is also present in the approved Stage 3 Event Schema — a Contract-approved classification alone does not authorize emitting it as an event." | Emitting a failure event requires both Contract approval AND Event Schema presence; neither alone suffices. | KEEP-IN-CORE | doctrine | Direct application of NN-2/NN-4 together; not contradicted. |
| FAILURE-BOUNDARY-3 | Contract-to-Implementation Failure Boundary | L444-447: "Internal and technical errors may propagate through richer implementation error types, but... must never be silently mapped [to a contractual outcome]. Every internal-to-contractual classification mapping is explicit and reviewable — Stage 4 records it in a Failure Mapping Table..." | Internal errors may use rich types but never silently map to contractual outcomes; every mapping is explicit and recorded in a Failure Mapping Table. | KEEP-IN-CORE | doctrine | Not contradicted; consistent with lean's Stage 4 permitting internal abstractions (NN-3's delta) provided observable behavior is unchanged. |
| FAILURE-BOUNDARY-4 | Contract-to-Implementation Failure Boundary | L449-453: "Stage 5 verifies all four directions" (contractual failures classify correctly; emitted events conform to schema; technical failures never masquerade as behavioral; no unapproved event is emitted). | Stage 5's verification protocol covers all four listed directions. | KEEP-IN-CORE | doctrine | One coordinated verification checklist serving one purpose; not contradicted. |
| FAILURE-BOUNDARY-5 | Contract-to-Implementation Failure Boundary | L455-457: "No universal error library or single canonical enum is prescribed. This stays language-neutral here; a Rust realization... is in `.codeos/patterns/rust-project-structure.md`..." | No canonical error library/enum prescribed at the doctrine level; language-neutral, with a Rust realization documented in a pattern file. | KEEP-IN-CORE | doctrine | Not contradicted. |

**Section coverage**: 5 normative rules found (FAILURE-BOUNDARY-1 through 5).

---

### Section: Optional Mechanism Status Convention (L462-476)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| OPT-MECH-1 | Optional Mechanism Status Convention | L464-470: "A future feature may need a human-controlled on/off switch... one minimal convention... a file containing exactly one line, `status: enabled` or `status: disabled`; missing means disabled; anything else is a configuration error. No resolver, no schema version..." | The convention: exactly one status line, four-outcome table, no resolver/versioning/provenance machinery. | KEEP-IN-CORE | doctrine | Shared infrastructure other policies (CPE, and this proposal's own component model) depend on — stays in the kernel rather than becoming its own conditional policy. Lean silent — not contradicted. |
| OPT-MECH-2a | Optional Mechanism Status Convention | L472-473: "This is documentation of a reusable shape, not a new Stage ID, not a new Non-Negotiable Rule, and not a new mandatory gate." | The convention introduces no new Stage ID, Non-Negotiable Rule, or mandatory gate. | KEEP-IN-CORE | doctrine | Independently changeable from the file-location and adopter facts below. Lean silent — not contradicted. |
| OPT-MECH-2b | Optional Mechanism Status Convention | L473-475: "A project-local status file for such a mechanism lives under `architecture/`, alongside the other project-level architecture artifacts already documented there." | A mechanism's status file lives under `architecture/`. | KEEP-IN-CORE | doctrine | Independently changeable from the Stage-ID fact and the adopter fact. Lean silent — not contradicted. |

**Section coverage**: 3 normative rules found (OPT-MECH-1, 2a-b). Corrected during Step 4 review:
Codex R1 found `OPT-MECH-2` bundled the no-new-Stage-ID fact, the file-location fact, and the
first-adopter fact, split into 2a-c; Codex R2 then found the third of those (`OPT-MECH-2c`, "CPE
is the first feature to adopt it") was not itself a normative rule at all — a historical fact
with no obligation, permission, prohibition, authority, gate, or lifecycle content — so it was
removed as a row rather than kept, not merely relocated.

### Section: Controlled Plain English Writing Discipline (L479-514)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| CPE-1 | Controlled Plain English Writing Discipline | L481-486: "An optional writing discipline... adopting the Optional Mechanism Status Convention... Full content... lives in `.codeos/patterns/controlled-plain-english.md`. This section documents only the activation mechanics..." | CPE is an optional, OMSC-governed discipline; this section is activation mechanics only, full content elsewhere. | MOVE | controlled-plain-english policy | Not directly contradicted by lean's "Writing Rules" (a shorter, different-style section) — but whether lean's Writing Rules constitutes a `v2` candidate for *this* policy, or a distinct parallel policy, is one of the brief's own explicitly still-open Step-1 questions, not resolved by this row. |
| CPE-2a | Controlled Plain English Writing Discipline | L488: "A downstream project's status file is `architecture/controlled-plain-english.yaml` (project-local — not reached through the `.codeos` symlink...)" | CPE's status file lives at `architecture/controlled-plain-english.yaml`, project-local, not through the symlink. | MOVE | controlled-plain-english policy | Independently changeable from the scaffold-default and symlink-reach facts below; lean silent — not contradicted. Split from the previous single CPE-2 row during Step 3 review (Codex R1). |
| CPE-2b | Controlled Plain English Writing Discipline | L489: "`scripts/dba-init.sh` scaffolds it at `status: enabled` by default — a human sets it to `status: disabled` to turn the discipline off." | `dba-init.sh` scaffolds the status file at `status: enabled` by default. | MOVE | controlled-plain-english policy | Independently changeable from the path and symlink facts; lean silent — not contradicted. |
| CPE-2c | Controlled Plain English Writing Discipline | L490-492: "The pattern file itself *is* reached through the symlink, at `.codeos/patterns/controlled-plain-english.md`." | The pattern file (not the status file) is reached through the `.codeos` symlink. | MOVE | controlled-plain-english policy | Independently changeable from the status-file path and default; lean silent — not contradicted. |
| CPE-2d | Controlled Plain English Writing Discipline | L492-494: "A missing file still means disabled (the Optional Mechanism Status Convention's own fallback, unchanged); the exact grammar and four-outcome table are that convention's, unchanged." | Missing status file still means disabled, per OMSC's own fallback. | RETIRE | — | Duplicate of OPT-MECH-1's four-outcome table (missing = disabled); the source text itself says this is "the Optional Mechanism Status Convention's own fallback, unchanged" — zero independent content. |
| CPE-3a | Controlled Plain English Writing Discipline | L500-501: "Stage 1-10 prompts (`.codeos/prompts/01-intent.md`...`10-arch-refine.md`) \| `architecture/controlled-plain-english.yaml`" and the matching `pipeline-reviewer.md` row. | Named prompt call-sites read the project's CPE status file directly. | MOVE | controlled-plain-english policy | Lean silent — not contradicted. |
| CPE-3b | Controlled Plain English Writing Discipline | L502: "`codeos-reviewer-task.md`... Reads neither file — configuration-neutral. `codeos-review.sh` resolves this project's status automatically and appends a synthetic status artifact... before invoking the reviewer... `tools/reviewer` is unchanged — it still embeds whatever artifact paths it is given; the wrapper is what supplies this one automatically now." | The reviewer wrapper (not the shared task template) resolves and injects CPE status automatically as a synthetic artifact. | MOVE | reviewer tool contract | This is wrapper/tool invocation mechanics specifically, distinct from CPE's own policy content — split from CPE-3a's call-site table for target-owner precision. |
| CPE-4 | Controlled Plain English Writing Discipline | L504-508: "What is and isn't toggle-gated. Layer A... and Layer C1/D1... are not new mandatory rules and are never toggle-gated... Layer B, C2, and D2 are the only parts this switch actually gates. Disabling... leaves generation and review behavior exactly as it is today." | Layers A/C1/D1 always apply regardless of the toggle; only B/C2/D2 are gated; disabling changes nothing else. | MOVE | controlled-plain-english policy | Lean silent — not contradicted. |
| CPE-5 | Controlled Plain English Writing Discipline | L510-513: "No new Stage ID, no new Non-Negotiable Rule, no new mandatory human-approval gate. This is a writing discipline consulted by existing stages and the existing reviewer, not a new stage or gate of its own." | CPE introduces no new Stage ID, Non-Negotiable Rule, or mandatory gate. | MOVE | controlled-plain-english policy | Lean silent — not contradicted. |

**Section coverage**: 9 normative rules found (CPE-1, CPE-2a-d, CPE-3a-b, CPE-4, CPE-5) — CPE-3's
source table split into two rows by target owner; CPE-2's activation paragraph split into three
independently-changeable facts plus one duplicate `RETIRE`, corrected during Step 3 review.

### Section: What You Do at Each Stage (L516-556)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| STAGE-TABLE-1a | What You Do at Each Stage | L525: "Session start \| — \| `.codeos/prompts/00-session-start.md`" | Session start uses `.codeos/prompts/00-session-start.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other stage/file mapping; needed regardless of doctrine version. |
| STAGE-TABLE-1b | What You Do at Each Stage | L526: "Session end (handoff) \| — \| `.codeos/prompts/00-session-end.md`" | Session end uses `.codeos/prompts/00-session-end.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1c | What You Do at Each Stage | L527: "Solution Discovery... \| `discovery` \| `.codeos/prompts/00a-solution-discovery.md`" | Solution Discovery's Stage ID is `discovery`, using `.codeos/prompts/00a-solution-discovery.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1d | What You Do at Each Stage | L528: "Feature Brief... \| `brief` \| `.codeos/prompts/00b-feature-brief.md`" | Feature Brief's Stage ID is `brief`, using `.codeos/prompts/00b-feature-brief.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1e | What You Do at Each Stage | L529: "Existing Codebase Onboarding... \| `onboarding` \| `.codeos/prompts/00c-onboarding.md`" | Onboarding's Stage ID is `onboarding`, using `.codeos/prompts/00c-onboarding.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1f | What You Do at Each Stage | L530: "Stage 1: Intent \| `1` \| `.codeos/prompts/01-intent.md`" | Stage 1's ID is `1`, using `.codeos/prompts/01-intent.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1g | What You Do at Each Stage | L531: "Stage 2: Contracts \| `2` \| `.codeos/prompts/02-contract.md`" | Stage 2's ID is `2`, using `.codeos/prompts/02-contract.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1h | What You Do at Each Stage | L532: "Stage 3: Event Schema \| `3` \| `.codeos/prompts/03-event-schema.md`" | Stage 3's ID is `3`, using `.codeos/prompts/03-event-schema.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1i | What You Do at Each Stage | L533: "Architecture Synthesis Gate... \| `architecture-synthesis` \| `.codeos/prompts/03b-architecture-synthesis.md`" | The Architecture Synthesis Gate's ID is `architecture-synthesis`, using `.codeos/prompts/03b-architecture-synthesis.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1j | What You Do at Each Stage | L534: "Stage 4: Implementation \| `4` \| `.codeos/prompts/04-implement.md`" | Stage 4's ID is `4`, using `.codeos/prompts/04-implement.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1k | What You Do at Each Stage | L535: "Stage 5: Tests \| `5` \| `.codeos/prompts/05-tests.md`" | Stage 5's ID is `5`, using `.codeos/prompts/05-tests.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1l | What You Do at Each Stage | L536: "Stage 6: Observation \| `6` \| `.codeos/prompts/06-observe.md`" | Stage 6's ID is `6`, using `.codeos/prompts/06-observe.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1m | What You Do at Each Stage | L537: "Stage 7: Reconcile \| `7` \| `.codeos/prompts/07-reconcile.md`" | Stage 7's ID is `7`, using `.codeos/prompts/07-reconcile.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1n | What You Do at Each Stage | L538: "Stage 8: Replay \| `8` \| `.codeos/prompts/08-replay.md`" | Stage 8's ID is `8`, using `.codeos/prompts/08-replay.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1o | What You Do at Each Stage | L539: "Stage 9: Refine \| `9` \| `.codeos/prompts/09-refine.md`" | Stage 9's ID is `9`, using `.codeos/prompts/09-refine.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1p | What You Do at Each Stage | L540: "Architectural Refinement... \| `10` \| `.codeos/prompts/10-arch-refine.md`" | Architectural Refinement's ID is `10`, using `.codeos/prompts/10-arch-refine.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1q | What You Do at Each Stage | L541: "Reviewer Activation Package... \| — \| `.codeos/prompts/pipeline-reviewer.md`" | The Reviewer Activation Package uses `.codeos/prompts/pipeline-reviewer.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other mapping; not contradicted. |
| STAGE-TABLE-1r | What You Do at Each Stage | L562: "Feature brief \| `.codeos/templates/feature-brief.md`" | The Feature Brief artifact uses template `.codeos/templates/feature-brief.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1s | What You Do at Each Stage | L563: "Feature intent \| `.codeos/templates/intent.md`" | The Intent artifact uses template `.codeos/templates/intent.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1t | What You Do at Each Stage | L564: "Behavioral contract \| `.codeos/templates/contract.md`" | The Contract artifact uses template `.codeos/templates/contract.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1u | What You Do at Each Stage | L565: "Event schema \| `.codeos/templates/event-schema.md`" | The Event Schema artifact uses template `.codeos/templates/event-schema.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1v | What You Do at Each Stage | L566: "Feature specification \| `.codeos/templates/feature-spec.md`" | The Feature Specification artifact uses template `.codeos/templates/feature-spec.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1w | What You Do at Each Stage | L567: "Refinement log \| `.codeos/templates/refinement.md`" | The Refinement Log artifact uses template `.codeos/templates/refinement.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1x | What You Do at Each Stage | L568: "Architectural refinement \| `.codeos/templates/arch-refinement.md`" | The Architectural Refinement artifact uses template `.codeos/templates/arch-refinement.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1y | What You Do at Each Stage | L569: "Architecture Baseline... \| `.codeos/templates/architecture-baseline.md`" | The Architecture Baseline artifact uses template `.codeos/templates/architecture-baseline.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1z | What You Do at Each Stage | L570: "Cohort Logical Design... \| `.codeos/templates/cohort-logical-design.md`" | The Cohort Logical Design artifact uses template `.codeos/templates/cohort-logical-design.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1aa | What You Do at Each Stage | L571: "Implementation Profile... \| `.codeos/templates/implementation-profile.yaml`" | The Implementation Profile artifact uses template `.codeos/templates/implementation-profile.yaml`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1ab | What You Do at Each Stage | L572: "Codebase digest \| `.codeos/templates/codebase-digest.md`" | The Codebase Digest artifact uses template `.codeos/templates/codebase-digest.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1ac | What You Do at Each Stage | L573: "Session handoff \| `.codeos/templates/handoff.md`" | The Session Handoff artifact uses template `.codeos/templates/handoff.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1ad | What You Do at Each Stage | L574: "Review Package \| `.codeos/templates/review-package.md`" | The Review Package artifact uses template `.codeos/templates/review-package.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-1ae | What You Do at Each Stage | L575: "Per-feature review file \| `.codeos/templates/review-file.md`" | The per-feature review file artifact uses template `.codeos/templates/review-file.md`. | KEEP-IN-CORE | doctrine | Independently changeable from every other template mapping; not contradicted. |
| STAGE-TABLE-2 | What You Do at Each Stage | L543-547: "On `onboarding`'s position... not a step every feature passes through after `brief`. It is an alternate entry point, used instead of `discovery`/`brief` only when bootstrapping an existing codebase..." | Onboarding is an alternate entry point, not a mandatory step in every feature's path. | KEEP-IN-CORE | doctrine | Lean's own onboarding-adjacent guidance ("For an existing codebase, inspect the current structure... before Stage 1") is consistent, not contradictory. |
| STAGE-TABLE-3 | What You Do at Each Stage | L549-554: "On `architecture-synthesis`'s position... a project-level gate that applies only when a human has declared a core architecture cohort... most features, and most projects, never trigger it." | Architecture-synthesis is a project-level, cohort-conditional gate, not a per-feature step. | MOVE | architecture-synthesis policy | Lean: same conditionality — "Use this gate when two or more features could constrain..." |
| STAGE-TABLE-4 | What You Do at Each Stage | L556: "The Architectural Refinement workflow is a 5-step alternative loop (Scope → Impact → Implement → Verify → Reconcile)... Use the 9-step loop for any change that would alter a contract or schema... Stage-10-eligible only when it does not change any feature's behavior..." | Structural-only changes use a named 5-step Stage-10 loop instead of the 9-step loop; behavioral changes always use the 9-step loop. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Newly identified: lean's "Structural-Only Changes" preserves the structural-vs-behavioral distinction and the same core discipline (state goal, get approval, implement, verify, reconcile) but drops the formal named 5-step Scope→Impact→Implement→Verify→Reconcile structure and the `refinements/arch/`/`arch-refinement.md` artifact convention in favor of unstructured prose. Part 2's `proposed_rule` states this replacement completely; `requires_human_decision: yes` is the human's approve/reject of that one stated replacement, not a choice between multiple forms. |

**Section coverage**: 34 normative rules found (STAGE-TABLE-1a-q [17 Stage/Stage-ID/File
mappings], STAGE-TABLE-1r-ae [14 Artifact/Template mappings], STAGE-TABLE-2, 3, 4). Corrected
during Step 3 review (Codex R3): `STAGE-TABLE-1` bundled 31 independently-changeable
Stage-to-file and Artifact-to-template mappings into one row — split fully, one row per mapping,
matching the same rigor already applied to `Artifact Classification`.

---

### Section: What You NEVER Do (L579-589)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| NEVER-DO-1 | What You NEVER Do | L581: "Implement before intent + contract + event schema are all APPROVED" | Duplicate prohibition. | RETIRE | — | Duplicate of NN-2; zero independent content. |
| NEVER-DO-2 | What You NEVER Do | L582: "Add abstractions not demanded by the contracts" | Duplicate prohibition. | RETIRE | — | Duplicate of NN-3 (whose disposition, `INTENTIONAL-BEHAVIOR-CHANGE`, is recorded once at NN-3 — not re-flagged here). |
| NEVER-DO-3 | What You NEVER Do | L583: "Add 'just in case' error handling not listed in the contract's failure modes" | Duplicate prohibition. | RETIRE | — | Substantively covered by NN-3's already-flagged delta (internal/defensive handling that doesn't change observable behavior is now permitted); no independent content beyond NN-3. |
| NEVER-DO-4 | What You NEVER Do | L584: "Emit events not in the approved event schema" | Duplicate prohibition. | RETIRE | — | Duplicate of NN-4; zero independent content. |
| NEVER-DO-5 | What You NEVER Do | L585: "Move to the next stage without explicit human approval" | Duplicate prohibition. | RETIRE | — | Duplicate of NN-1 (whose disposition, `INTENTIONAL-BEHAVIOR-CHANGE`, is recorded once at NN-1). |
| NEVER-DO-6 | What You NEVER Do | L586: "Suggest full rewrites — only targeted, localized changes" | No full rewrites; only targeted, localized changes. | KEEP-IN-CORE | doctrine | Lean Working Rule #7: "Make the smallest change that satisfies the approved behavior." — same substance. |
| NEVER-DO-7 | What You NEVER Do | L587: "Add autonomous planning, self-direction, or multi-step autonomous execution" | No autonomous planning, self-direction, or multi-step autonomous execution. | INTENTIONAL-BEHAVIOR-CHANGE | doctrine | Newly identified, and significant: NN-1's own batched Stage 4-8 execution (running multiple stages continuously between two human checkpoints) is a form of extended multi-step execution this literal prohibition may read as forbidding outright. Lean does not explicitly address or repeal this prohibition. Whether artifact-bounded, checkpoint-delimited batched execution is a narrower permitted case or is barred by this rule as written is a genuine open question this inventory surfaces but does not resolve. |
| NEVER-DO-8 | What You NEVER Do | L588: "Modify `events/runtime_events.jsonl` — it is append-only" | Never modify the runtime event log; append-only. | KEEP-IN-CORE | doctrine | Lean: "never edit old observations." — same invariant. Distinct from `STEP6-EVENTS` (also `KEEP-IN-CORE`): that row states the emission *destination* fact (events are written there); this row states the modify *prohibition* (they're never edited once written) — two independent facts, not duplicates of each other. |

**Section coverage**: 8 normative rules found (NEVER-DO-1 through 8), matching the 8 bullets exactly.

### Section: Naming Conventions (L592-596)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| NAMING-1 | Naming Conventions | L594: "See `.codeos/templates/conventions.md` for the authoritative naming convention reference." | `templates/conventions.md` is the authoritative source for naming conventions. | KEEP-IN-CORE | doctrine | Not contradicted — lean's differing overall document structure doesn't invalidate this pointer. |

**Section coverage**: 1 normative rule found (NAMING-1).

---

### Section: Artifact Classification (L598-624)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| ARTIFACT-CLASS-1 | Artifact Classification | L600-604: "Not all artifacts are required... Required artifacts block stage advancement. Optional and recommended artifacts improve decision quality but are never prerequisites for stage transitions." | Required-vs-optional-vs-recommended is the governing distinction for stage-advancement blocking. | KEEP-IN-CORE | doctrine | Lean's "Lean Artifacts": "Required feature artifacts are Intent, Contract, Event Schema, code, tests, runtime evidence, reconciliation, and replay evidence... Everything else is optional." — same governing distinction. |
| ARTIFACT-CLASS-2 | Artifact Classification | L611: "Feature Brief (`backlog/[id].md`) \| Optional \| Pre-Stage-1 discovery; not required to start Stage 1" | Feature Brief is optional, pre-Stage-1 discovery; not required to start Stage 1. | KEEP-IN-CORE | doctrine | Lean: "A Feature Brief is also optional. Use one only when it helps decide..." — same substance. Corrected during Step 3 review (Codex R1): this classification has no surviving row anywhere else in the inventory. |
| ARTIFACT-CLASS-3 | Artifact Classification | L612: "Intent (`intents/[id].md`) \| **Required** \| Any behavioral work — must be APPROVED before Stage 2" | Intent is required for any behavioral work, approved before Stage 2. | KEEP-IN-CORE | doctrine | Not a duplicate of STEP1-GATE, corrected at Step 4 R3: STEP1-GATE states only "human approves intent before step 2" — it does not state the scope condition "required for *any behavioral work*" (as distinct from structural-only work, see STAGE-TABLE-4). That scope qualifier has no other surviving row. |
| ARTIFACT-CLASS-4 | Artifact Classification | L613: "Contract (`contracts/[id]_contract.md`) \| **Required** \| Any behavioral work — must be APPROVED before Stage 3" | Contract is required for any behavioral work, approved before Stage 3. | KEEP-IN-CORE | doctrine | Not a duplicate of STEP2-GATE, same reasoning as ARTIFACT-CLASS-3 — the "any behavioral work" scope qualifier is not preserved by the gate row alone. |
| ARTIFACT-CLASS-5 | Artifact Classification | L614: "Event Schema (`events/[id]_schema.md`) \| **Required** \| Any behavioral work — must be APPROVED before Stage 4" | Event Schema is required for any behavioral work, approved before Stage 4. | KEEP-IN-CORE | doctrine | Not a duplicate of STEP3-GATE, same reasoning as ARTIFACT-CLASS-3/4. |
| ARTIFACT-CLASS-6 | Artifact Classification | L615: "Feature Registry (`features/registry.yaml`) \| Recommended \| Multi-feature projects; not required for single-feature work" | Feature Registry is recommended, not required, for multi-feature projects. | KEEP-IN-CORE | doctrine | Genuinely new fact, stated nowhere else; independently changeable from every other row on this table. Lean doesn't classify it the same way but doesn't contradict. |
| ARTIFACT-CLASS-7 | Artifact Classification | L616: "Codebase Digest (`docs/codebase-digest.md`) \| Optional \| Existing codebases and mature projects; absent digest is never a blocker" | Codebase Digest is optional; its absence is never a blocker. | KEEP-IN-CORE | doctrine | Independently changeable from every other row; not contradicted. |
| ARTIFACT-CLASS-8 | Artifact Classification | L617: "Structural Alignment (Stage 7 output section) \| Optional output \| Produced at Stage 7 only when architectural observations exist" | Structural Alignment is an optional Stage 7 output, produced only when architectural observations exist. | KEEP-IN-CORE | doctrine | Independently changeable from every other row; not contradicted. |
| ARTIFACT-CLASS-9 | Artifact Classification | L618: "Architectural Refinement (`refinements/arch/[id].md`) \| Optional \| Non-behavioral structural changes; uses the Stage 10 workflow" | Architectural Refinement is optional, used for non-behavioral structural changes via the Stage 10 workflow. | KEEP-IN-CORE | doctrine | Independently changeable from every other row; not contradicted. Also the source of the `refinements/arch/[id].md` path (see FILE-LAYOUT-7). |
| ARTIFACT-CLASS-10 | Artifact Classification | L619: "Architecture Baseline (`architecture/core-baseline.md`) \| **Required for cohort members' Stage 4** \| Only when a core architecture cohort is declared..." | Architecture Baseline is required for cohort members' Stage 4, only when a cohort is declared. | RETIRE | — | Duplicate of ARCH-GATE-4. |
| ARTIFACT-CLASS-11 | Artifact Classification | L620: "Cohort Logical Design (`architecture/cohort-logical-design.md`) \| **Required for cohort members' Stage 4** \| Only when a core architecture cohort is declared, approved together with the Baseline..." | Cohort Logical Design is required for cohort members' Stage 4, approved together with the Baseline. | RETIRE | — | Duplicate of ARCH-GATE-4. |
| ARTIFACT-CLASS-12 | Artifact Classification | L621: "Implementation Profile (`architecture/implementation-profile.yaml`) \| Optional \| Governs Stage 4 language/pattern consultation only once `approved`..." | Implementation Profile is optional, governs Stage 4 consultation only once approved. | RETIRE | — | Duplicate of IMPL-PROFILE-1a/1b. |
| ARTIFACT-CLASS-13 | Artifact Classification | L622: "Controlled Plain English status (`architecture/controlled-plain-english.yaml`) \| Optional \| Governs Layer B/C2/D2 writing-discipline consultation only when `status: enabled`..." | CPE status is optional, governs Layer B/C2/D2 consultation only when enabled. | RETIRE | — | Duplicate of CPE-1. |
| ARTIFACT-CLASS-14 | Artifact Classification | L623: "Onboarding artifacts (`HYPOTHESIZED_INTENT`) \| Onboarding only \| Produced by Session Type D; must pass Stage 1 review before advancing" | Onboarding artifacts apply only to Session Type D and must pass Stage 1 review before advancing. | KEEP-IN-CORE | doctrine | Independently changeable from every other row; not contradicted. |

**Section coverage**: 14 normative rules found (ARTIFACT-CLASS-1 through 14) — one row per table
entry, corrected during Step 3 review (Codex R2): a shared `RETIRE` disposition does not license
bundling independently-changeable facts into one row; the semantic-independence test applies
regardless of whether rows end up with the same disposition. Corrected further at Step 4 R3:
`ARTIFACT-CLASS-3`/`4`/`5` (Intent/Contract/Event-Schema "required for any behavioral work")
were wrongly `RETIRE`d as duplicates of `STEP1/2/3-GATE` — those gate rows never actually state
the "any behavioral work" scope qualifier, so the classification rows are `KEEP-IN-CORE` instead.

---

### Section: File Layout (L627-678)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| FILE-LAYOUT-1 | File Layout | L636-637: "`features/` └── `registry.yaml` ← authoritative feature status index (human-maintained)" | Feature status is indexed at `features/registry.yaml`, human-maintained. | KEEP-IN-CORE | doctrine | Independently changeable from the other paths in the tree; not stated elsewhere, not contradicted. |
| FILE-LAYOUT-2 | File Layout | L638: "`architecture/` ← project-level architecture artifacts, each independently optional" | The `architecture/` directory exists; each entry under it is independently optional. | KEEP-IN-CORE | doctrine | Genuinely new general framing fact, stated nowhere else; independently changeable from any specific path beneath it. Not contradicted. |
| FILE-LAYOUT-2a | File Layout | L639: "`implementation-profile.yaml` ← current Implementation Profile (proposed or approved)" | Current Implementation Profile lives at `architecture/implementation-profile.yaml`. | KEEP-IN-CORE | doctrine | Not a duplicate of IMPL-PROFILE-3a, corrected at Step 4 R3: `IMPL-PROFILE-3a` states only the immutability rule, not the current-version path. No other row states this path. |
| FILE-LAYOUT-2b | File Layout | L640-641: "`proposals/` └── `implementation-profile-v[N].yaml` ← pending replacement, never binding" | Pending profile replacements live at `architecture/proposals/implementation-profile-v[N].yaml`. | RETIRE | — | Duplicate of IMPL-PROFILE-3b, which already states this path. |
| FILE-LAYOUT-2c | File Layout | L642-643: "`core-baseline.md` ← current approved Architecture Baseline — only when a core architecture cohort is declared (current version only)" | Current Architecture Baseline lives at `architecture/core-baseline.md`. | KEEP-IN-CORE | doctrine | Not a duplicate of ARCH-GATE-10, corrected at Step 4 R3: `ARCH-GATE-10` states only the history-move/versioning mechanic, not the current-version path. No other row states this path. |
| FILE-LAYOUT-2d | File Layout | L644-645: "`cohort-logical-design.md` ← current approved Cohort Logical Design..." | Current Cohort Logical Design lives at `architecture/cohort-logical-design.md`. | KEEP-IN-CORE | doctrine | Not a duplicate of ARCH-GATE-10, same reasoning as FILE-LAYOUT-2c. |
| FILE-LAYOUT-2e | File Layout | L646-648: "`[mechanism-name].yaml` ← optional: an enabled/disabled status file for an optional AI-doctrine mechanism..." | An optional mechanism's status file lives at `architecture/[mechanism-name].yaml`. | RETIRE | — | Duplicate of OPT-MECH-2b, which already states this path pattern. |
| FILE-LAYOUT-2f | File Layout | L649-652: "`controlled-plain-english.yaml` ← status file for the Controlled Plain English Writing Discipline; scaffolded by dba-init.sh at status: enabled..." | CPE's status file lives at `architecture/controlled-plain-english.yaml`. | RETIRE | — | Duplicate of CPE-2a, which already states this path. |
| FILE-LAYOUT-2g1 | File Layout | L654: "`core-baseline-v[N].md` ← superseded baseline versions" | Superseded baseline versions live at `architecture/history/core-baseline-v[N].md`. | RETIRE | — | Duplicate of ARCH-GATE-10, which already states this path. |
| FILE-LAYOUT-2g2 | File Layout | L655: "`cohort-logical-design-v[N].md` ← superseded logical design versions" | Superseded logical-design versions live at `architecture/history/cohort-logical-design-v[N].md`. | RETIRE | — | Duplicate of ARCH-GATE-10, which already states this path. |
| FILE-LAYOUT-2g3 | File Layout | L656: "`implementation-profile-v[N].yaml` ← superseded profile versions" | Superseded profile versions live at `architecture/history/implementation-profile-v[N].yaml`. | RETIRE | — | Duplicate of IMPL-PROFILE-3c, which already states this path. |
| FILE-LAYOUT-3a | File Layout | L657-658: "`intents/` └── `[feature_id].md` ← one per feature" | Intent files live at `intents/[feature_id].md`. | RETIRE | — | Duplicate of STEP1-OUTPUT. |
| FILE-LAYOUT-3b | File Layout | L659-660: "`contracts/` └── `[feature_id]_contract.md` ← one per feature" | Contract files live at `contracts/[feature_id]_contract.md`. | RETIRE | — | Duplicate of STEP2-OUTPUT. |
| FILE-LAYOUT-3c | File Layout | L661-662: "`events/` ├── `[feature_id]_schema.md` ← event schema per feature (or shared event_schema.md)" | Event Schema files live at `events/[feature_id]_schema.md`. | RETIRE | — | Duplicate of STEP3-OUTPUT. |
| FILE-LAYOUT-3d | File Layout | L663: "`runtime_events.jsonl` ← append-only runtime log" | The runtime event log lives at `events/runtime_events.jsonl`. | RETIRE | — | Duplicate of STEP6-EVENTS. |
| FILE-LAYOUT-4 | File Layout | L664-665: "`backlog/` └── `[feature_id].md` ← feature briefs (pre-Stage-1 discovery)" | Feature briefs live at `backlog/[feature_id].md`. | KEEP-IN-CORE | doctrine | Independently changeable from the other paths; not stated elsewhere (ARTIFACT-CLASS-2 classifies Feature Brief as optional but doesn't state its path) — not contradicted. |
| FILE-LAYOUT-5a | File Layout | L666-667: "`handoffs/` └── `[YYYY-MM-DD]-[desc].md`" | Session handoffs live at `handoffs/[date]-[desc].md`. | KEEP-IN-CORE | doctrine | Independently changeable from whether handoffs are optional or count as DBA artifacts. Not stated elsewhere, not contradicted. |
| FILE-LAYOUT-5b | File Layout | L667: "← session handoffs (optional, not DBA artifacts)" | Session handoffs are optional. | KEEP-IN-CORE | doctrine | Independently changeable from the path and the DBA-artifact classification. Not stated elsewhere, not contradicted. |
| FILE-LAYOUT-5c | File Layout | L667: "← session handoffs (optional, not DBA artifacts)" | Session handoffs are not DBA artifacts. | KEEP-IN-CORE | doctrine | Independently changeable from the path and the optionality fact. Not stated elsewhere, not contradicted. |
| FILE-LAYOUT-6a | File Layout | L669: "`[feature_id].md` ← per-feature: Decision Log + Decision Rationale (traceability)" | Per-feature Decision Log/Rationale files live at `reviews/[feature_id].md`. | RETIRE | — | Duplicate of REVIEW-LOG-1b, which already states this path. |
| FILE-LAYOUT-6b | File Layout | L670: "`architecture-journal.md` ← cross-feature institutional memory (AJ-NNN entries)" | The architecture journal lives at `reviews/architecture-journal.md`. | RETIRE | — | Duplicate of REVIEW-LOG-1d, which already states this path. |
| FILE-LAYOUT-7 | File Layout | L671-673: "`refinements/` └── `arch/` └── `[refine_id].md` ← architectural refinement records". | Architectural refinement records live at `refinements/arch/[refine_id].md`. | RETIRE | — | Duplicate of ARTIFACT-CLASS-9, which already states this exact path for Architectural Refinement. |
| FILE-LAYOUT-8 | File Layout | L674: "`modules/` ← actual implementation code" | Implementation code lives under `modules/`. | RETIRE | — | Duplicate of STEP4-OUTPUT ("Output: code in modules/"). |
| FILE-LAYOUT-9a | File Layout | L676: "`behavioral/` ← behavioral outcome tests" | Behavioral tests live at `tests/behavioral/`. | RETIRE | — | Duplicate of STEP5-OUTPUT, which already states this path. |
| FILE-LAYOUT-9b | File Layout | L677: "`replay/` ← replay verification tests" | Replay tests live at `tests/replay/`. | RETIRE | — | Duplicate of STEP5-OUTPUT, which already states this path. |

**Section coverage**: 25 normative rules found (FILE-LAYOUT-1, 2, 2a-f, 2g1-3, 3a-d, 4, 5a-c,
6a-b, 7, 8, 9a-b). This note itself was found stale at Step 4 R1 — it still said 16 after the R3
fix that split `FILE-LAYOUT-2g`/`6`/`9` further (16→23) updated the top-level summary table and
Implementation Notes but missed this in-section note. Fixed at Step 4 R1; see the note after this
table about not trusting restated counts over the actual row listing.
Corrected across Step 3 and Step 4 review: Step 3 R1 first split the single bundled tree into 9
top-level directory rows; Step 3 R2 found two of those (`FILE-LAYOUT-2` and `FILE-LAYOUT-3`)
still bundled several independently-changeable sub-paths under one disposition and split them
further; Step 3 R3 split `FILE-LAYOUT-2g`/`6`/`9`; Step 4 R2 found `FILE-LAYOUT-5` still bundled
its path, optionality, and DBA-artifact classification under one disposition — split into 5a-c
(23→25). Step 4 R3 found `FILE-LAYOUT-2a`/`2c`/`2d`'s "duplicate of X" claims were false — their
named target rows (`IMPL-PROFILE-3a`, `ARCH-GATE-10`) state only immutability/versioning
mechanics, never the actual current-version file path — so all three are now `KEEP-IN-CORE`
instead of `RETIRE`, preserving the path facts that would otherwise have been silently dropped.
Each specific fact is now its own row, `RETIRE`d individually only against an owning row that
actually restates it, `KEEP-IN-CORE` where no such row exists. A shared disposition never
licenses bundling, and a named "duplicate" claim is only as good as whether the target row
actually contains the claimed content — both lessons learned the hard way across five rounds.

---

### Section: DBA Vocabulary (L682-697)

**Section coverage**: `NO NORMATIVE RULES`. The glossary (Intent, Behavioral Contract, Event Spine, Observational/Behavioral/Failure Event, Reconciliation Review, Replay Verification, Targeted Refinement, Correlation ID, Shared Infrastructure Module, Vertical Drift) defines vocabulary other rules depend on, but a definition alone is not itself an obligation, prohibition, permission, authority rule, gate, or lifecycle requirement per the granularity test — no rows.

### Section: Human Navigation (L701-727)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| HUMAN-NAV-1 | Human Navigation | L703-712: "Intent files are precision artifacts... not for fast reading. When you need a quick plain-language explanation... Ask Claude directly... Claude will read `intents/[feature_id].md` and produce a jargon-free explanation on demand. No file is saved. No approval gate. No DBA lifecycle." | On-demand plain-English explanation of an intent file is permitted, never persisted, never a DBA artifact. | KEEP-IN-CORE | doctrine | Lean doesn't mention this on-demand pattern — silent, not contradicted. |
| HUMAN-NAV-2a | Human Navigation | L715-719: "If a stored summary is needed... Generate it on request, include provenance metadata (see below), and treat it as generated output..." plus the provenance YAML block (`generated_from_intent`/`generated_at`/`generated_by`). | A stored summary, if ever generated, carries provenance metadata naming its source intent, generation time, and generator. | KEEP-IN-CORE | doctrine | Independently changeable from the regeneration rule and the artifact-classification fact. Lean silent — not contradicted. |
| HUMAN-NAV-2b | Human Navigation | L718: "never manually edit, regenerate from intent when the intent changes." | A stored summary is never manually edited; it is regenerated when the source intent changes. | KEEP-IN-CORE | doctrine | Independently changeable from the provenance-metadata requirement and the artifact-classification fact. Lean silent — not contradicted. |
| HUMAN-NAV-2c | Human Navigation | L726-727: "Stored generated summaries are not DBA artifacts. They do not feed into any stage. They do not carry `status`, `approved_by`, or `derived_contracts` fields." | Stored generated summaries are not DBA artifacts, don't feed into any stage, and carry none of the DBA-artifact fields. | KEEP-IN-CORE | doctrine | Independently changeable from the provenance and regeneration rules. Lean silent — not contradicted. |

**Section coverage**: 4 normative rules found (HUMAN-NAV-1, 2a-c). Corrected during Step 4 review
(Codex R2): `HUMAN-NAV-2` bundled the provenance-metadata requirement, the regeneration rule, and
the not-a-DBA-artifact classification — three independently-changeable facts, split into 2a-c.

---

### Section: Review Logging (L731-784)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| REVIEW-LOG-1a | Review Logging | L733-734: "When the human provides a reviewer's assessment and their decision on it, before writing any review artifacts, Claude shows a brief preview of what it will write." | Before writing any review artifact, show a brief preview first. | MOVE | review policy | Independently changeable from the write-trigger and format rules below. Lean silent on a preview step — not contradicted. |
| REVIEW-LOG-1b | Review Logging | L735-738: "Then Claude writes — do this before proceeding to any other work: 1. One row to `reviews/[feature_id].md` Decision Log." | Every reviewed decision gets a mandatory Decision Log row — always, for every decision. | INTENTIONAL-BEHAVIOR-CHANGE | review policy | Matches "Review persistence" — lean: "Save a review only when its decision changes behavior, architecture, or an accepted risk. For ordinary corrections, the artifact diff and git history are enough." The write-trigger becomes conditional, not mandatory-by-default. |
| REVIEW-LOG-1c | Review Logging | L739-741: "2. A Decision Rationale section... only when the decision would be difficult to reconstruct from artifact history alone... Most stages do not get a section." | A Decision Rationale section is added only when the decision is hard to reconstruct from artifact history alone. | MOVE | review policy | Independently changeable from the Decision Log write-trigger and the Journal rule. Lean doesn't organize around a distinct "Rationale section" concept, but doesn't forbid adding rationale when something *is* written — not contradicted. |
| REVIEW-LOG-1d | Review Logging | L742-745: "3. One entry to `reviews/architecture-journal.md` — only if the insight is likely to remain useful six months from now to someone who has forgotten this change. When unsure, journal only if future usefulness is clear." | An Architecture Journal entry is added only when the insight will likely still matter six months from now. | MOVE | review policy | Independently changeable from the Decision Log and Rationale rules. Distinct from REVIEW-LOG-5 (which governs journal-vs-feature-file *placement*, not the journal write-trigger itself). Lean has no journal concept — silent, not contradicted. |
| REVIEW-LOG-2 | Review Logging | L747-749: "Human overrides... 'do not log this review'... 'journal this'... 'do not journal this'" | Named human-override phrases control logging/journaling on a per-cycle basis. | MOVE | review policy | Lean silent on override phrases specifically, but compatible with either logging model as an override mechanism — not contradicted. |
| REVIEW-LOG-3 | Review Logging | L752-753: "Log fidelity: Preserve the reviewer's core insight as close to verbatim as the format allows. Compress explanation and context — never compress the insight itself." | Preserve the core insight verbatim; compress only surrounding context. | MOVE | review policy | Lean silent — not contradicted; applies whenever logging does happen, regardless of how often. |
| REVIEW-LOG-4 | Review Logging | L755-756: "Log quality: Record conclusions and rationale, not conversation history. Capture what was learned, not what happened. Review artifacts must never become meeting minutes." | Log conclusions/rationale, never conversation history or meeting-minutes-style transcription. | MOVE | review policy | Lean silent — not contradicted. |
| REVIEW-LOG-5 | Review Logging | L758-763: "Architecture Journal: the journal is the long-term institutional knowledge artifact; per-feature review files are primarily traceability artifacts. When an architectural finding belongs equally in both, put it in the journal and keep the feature file entry brief with a reference..." | Journal-vs-feature-file placement rule, including the reader-context test for journal entries. | MOVE | review policy | Lean silent — not contradicted. |
| REVIEW-LOG-6 | Review Logging | L765-768: "Decision Log rows are append-only. Original findings and decisions are never rewritten. Superseded decisions are addressed by adding a new row..." | Decision Log is append-only; supersession is a new row, never a rewrite. | MOVE | review policy | Not contradicted by lean's conditional-logging model — append-only-ness survives regardless of how often logging happens. |
| REVIEW-LOG-7 | Review Logging | L770-772: "Architecture Journal entry format... ## AJ-NNN — [topic] Date: YYYY-MM-DD" — the full template (L770-782: Date/Status/Context/Finding/Decision/Action/Supersedes/Related fields). | Architecture Journal entries follow this fixed field format. | MOVE | review policy | Lean specifies no journal-entry format at all — silent, not contradicted. |

**Section coverage**: 10 normative rules found (REVIEW-LOG-1a-d, 2 through 7). Corrected during
Step 3 review (Codex R3): `REVIEW-LOG-1` bundled four independently-changeable facts (preview
step, Decision Log write-trigger, Rationale conditionality, Journal conditionality) under one
disposition; split into 1a-1d, with only 1b (the write-trigger) carrying the actual
`INTENTIONAL-BEHAVIOR-CHANGE`.

---

### Section: How to Use the Toolkit in a New Project (L786-793)

| rule_id | source_section | source_anchor | current_rule | disposition | target_owner | rationale |
|---|---|---|---|---|---|---|
| TOOLKIT-USE-1 | How to Use the Toolkit in a New Project | L788-793: "1. Run... `dba-init.sh`. 2. This creates `.codeos` symlink, all required directories, and a project `CLAUDE.md`. 3. Start Claude Code... 4. Claude reads the project `CLAUDE.md` which directs it to read this file. 5. Human pastes `.codeos/prompts/00-session-start.md`..." | The 5-step bootstrap sequence: `dba-init.sh` → scaffold → start session → `CLAUDE.md` directs to this file → paste session-start prompt. | KEEP-IN-CORE | doctrine | Lean has no equivalent onboarding section — silent, not contradicted. Step 4's "directs it to read this file" stays accurate under the brief's recommended (not-yet-decided) choice of keeping `dba-system.md` at its current path as the manifest; if the alternative `dba/dba.md` path were chosen instead, this step would need a textual update — a downstream consequence of that still-open question, not resolved here. |

**Section coverage**: 1 normative rule found (TOOLKIT-USE-1).

---

## Section-by-Section Rule Count

| Section | Rules found | KEEP-IN-CORE | MOVE | RETIRE | INTENTIONAL-BEHAVIOR-CHANGE |
|---|---|---|---|---|---|
| Mode Declaration | 1 | 0 | 0 | 0 | 1 |
| Truth Authority and Conflict Resolution | 8 | 7 | 0 | 0 | 1 |
| The Non-Negotiable Rules | 6 | 3 | 0 | 0 | 3 |
| Default Advisory Review | 13 | 1 | 10 | 0 | 2 |
| The 9-Step DBA Development Loop | 25 | 17 | 0 | 2 | 6 |
| Multi-Feature Architecture Synthesis Gate | 18 | 0 | 15 | 0 | 3 |
| Implementation Profile | 18 | 0 | 18 | 0 | 0 |
| Contract-to-Implementation Failure Boundary | 5 | 5 | 0 | 0 | 0 |
| Optional Mechanism Status Convention | 3 | 3 | 0 | 0 | 0 |
| Controlled Plain English Writing Discipline | 9 | 0 | 8 | 1 | 0 |
| What You Do at Each Stage | 34 | 32 | 1 | 0 | 1 |
| What You NEVER Do | 8 | 2 | 0 | 5 | 1 |
| Naming Conventions | 1 | 1 | 0 | 0 | 0 |
| Artifact Classification | 14 | 10 | 0 | 4 | 0 |
| File Layout | 25 | 9 | 0 | 16 | 0 |
| DBA Vocabulary | 0 | — | — | — | — |
| Human Navigation | 4 | 4 | 0 | 0 | 0 |
| Review Logging | 10 | 0 | 9 | 0 | 1 |
| How to Use the Toolkit in a New Project | 1 | 1 | 0 | 0 | 0 |
| **Total** | **203** | **95** | **61** | **28** | **19** |

`MOVE` totals combine all five non-doctrine target owners (review policy, architecture-synthesis
policy, implementation-profile policy, controlled-plain-english policy, reviewer tool contract) —
see each section's table for the specific owner per row.

Row count rose 115 → 131 → 147 → 184 → 193 → 198 → 203 across three Step 3 review rounds plus
three Step 4 rounds. **Step 3 R1-R3**, **Step 4 R1**, and **Step 4 R2** are summarized above each
section's own coverage note. **Step 4 R3** found two distinct classes of defect: (1) further
bundling in `REVIEW-5` (Default Advisory Review 10→13 — waiver eligibility, silent-skip
prohibition, project-blocking prohibition, and recording format bundled as one), `ARCH-GATE-7`
(Architecture Synthesis Gate 17→18 — a behavior constraint bundled with an unrelated procedural
gap-routing fact), and `IMPL-PROFILE-6` (Implementation Profile split, net Profile section
17→18 — specificity precedence bundled with conflict handling); (2) three `RETIRE` rows whose
named "duplicate" target did not actually contain the claimed content —
`ARTIFACT-CLASS-3`/`4`/`5` (Artifact Classification, 3 rows corrected `RETIRE`→`KEEP-IN-CORE`)
and `FILE-LAYOUT-2a`/`2c`/`2d` (File Layout, 3 rows corrected `RETIRE`→`KEEP-IN-CORE`), each of
which would have silently dropped a real fact (a scope qualifier, or a current-version file path)
that no other row actually preserved.

## Part 2 — `INTENTIONAL-BEHAVIOR-CHANGE` Detail

19 rows total. Every row below carries the literal `requires_human_decision: yes` marker; none
proposes deletion (`proposed_rule: REMOVED` / `target_owner: NONE`) — every flagged change has
exactly one stated `proposed_rule`, per AC5.

**MODE-1** (target_owner: doctrine)
> proposed_rule: At session start, read only the context needed for the current task — project
> instructions, current feature status, the current stage artifact and its direct approved
> inputs, applicable architecture decisions, and relevant code/tests/diff/runtime evidence. Do
> not load every project document, old review, or full history by default.
> requires_human_decision: yes

**TRUTH-AUTHORITY-2** (target_owner: doctrine)
> proposed_rule: Runtime evidence and tests establish what the system currently does; they inform
> claims about current behavior but do not silently amend what the intent/contract/event schema
> says should happen. A discrepancy is surfaced as intent-text drift requiring a human decision,
> not auto-resolved by treating runtime as authoritative.
> requires_human_decision: yes

**NN-1** (target_owner: doctrine)
> proposed_rule: A human must approve Intent, Contract, and Event Schema. Approval of the Event
> Schema authorizes the agent to run Stages 4 through 8 as one delivery cycle, stopping only for a
> controlled action, a material blocked decision, or an explicit human request for an extra
> checkpoint.
> requires_human_decision: yes

**NN-3** (target_owner: doctrine)
> proposed_rule: Implementation may use normal internal abstractions, technical error types,
> logging, and established project patterns to deliver the approved contract, provided these
> choices do not add or change externally observable feature behavior.
> requires_human_decision: yes

**NN-6** (target_owner: doctrine)
> proposed_rule: After producing a stage output that requires human approval under the batched
> model (Stage 1, 2, or 3 output; the Stage 8 delivery-cycle result; or a controlled
> action/blocked decision), stop and clearly indicate that human approval is needed — no longer
> after every individual Stage 4-8 output, and no longer pinned to the exact literal phrase
> "AWAITING HUMAN APPROVAL."
> requires_human_decision: yes

**REVIEW-1** (target_owner: review policy)
> proposed_rule: The acting agent performs a direct self-check before every gate. Independent
> review is used only when at least one condition is true: the human asks for it; the
> Multi-Feature Architecture Gate is ready for approval; the change crosses a
> security/authorization/privacy/financial/migration/irreversible-data boundary; the agent has a
> material unresolved concern after self-check; or Stage 7/8 finds a material mismatch.
> requires_human_decision: yes

**REVIEW-3** (target_owner: review policy)
> proposed_rule: Use one review pass by default. Use one focused retry after a material fix. The
> human then decides whether to approve, revise, accept a known risk, or stop.
> requires_human_decision: yes

**STEP4-GATE** (target_owner: doctrine)
> proposed_rule: No individual gate after Stage 4 — collapsed into NN-1's single Stage 3→8 batch
> cycle.
> requires_human_decision: yes

**STEP5-GATE** (target_owner: doctrine)
> proposed_rule: No individual gate after Stage 5 — collapsed into NN-1's single Stage 3→8 batch
> cycle.
> requires_human_decision: yes

**STEP6-ACTIVITY** (target_owner: doctrine)
> proposed_rule: The agent may run representative scenarios when the environment permits. Never
> fabricate runtime evidence.
> requires_human_decision: yes

**STEP7-ACTIVITY** (target_owner: doctrine)
> proposed_rule: Compare Intent, Contract, Event Schema, code, tests, and runtime evidence. Report
> only gaps, mismatches, missing evidence, and the evidence supporting the conclusion. If all
> layers align, a short conclusion is enough — do not produce a full table of aligned rows.
> requires_human_decision: yes

**STEP7-GATE** (target_owner: doctrine)
> proposed_rule: No individual gate after Stage 7 — collapsed into NN-1's single Stage 3→8 batch
> cycle.
> requires_human_decision: yes

**STEP9-GATE** (target_owner: doctrine)
> proposed_rule: A correction within already-approved behavior may proceed without a new product
> decision. A change to approved behavior must return to Stage 1, 2, or 3 and receive approval
> there.
> requires_human_decision: yes

**ARCH-GATE-5** (target_owner: architecture-synthesis policy)
> proposed_rule: Features in a cohort may be drafted and reviewed in same-stage batches. Approval
> may be given as one explicit batch decision, provided the human can identify every included
> artifact — not required to be strictly individual-only.
> requires_human_decision: yes

**ARCH-GATE-6** (target_owner: architecture-synthesis policy)
> proposed_rule: 1. Check the cohort for material contradictions and unclear ownership. 2. Draft a
> Core Architecture Baseline for project-level decisions. 3. Draft a Cohort Logical Design for
> shared identity, interfaces, transactions, event ownership, persistence, and migration decisions
> implementation needs now. 4. Present both drafts for one human approval.
> requires_human_decision: yes

**ARCH-GATE-10** (target_owner: architecture-synthesis policy)
> proposed_rule: Git history records superseded baseline/logical-design versions. Duplicate
> history files and complex registry states are not required.
> requires_human_decision: yes

**STAGE-TABLE-4** (target_owner: doctrine)
> proposed_rule: A change that cannot alter product behavior does not need the nine stages. State
> the structural goal and constraints, get approval to implement, make the change, run relevant
> checks, and reconcile the result. If the change can alter an observable outcome, use the
> nine-stage workflow and return to the earliest affected stage. This replaces the named 5-step
> Scope→Impact→Implement→Verify→Reconcile structure and the `refinements/arch/`/
> `arch-refinement.md` artifact convention with the functional description above — adopting this
> proposed rule means the formal Stage-10 identity and artifact convention no longer exist as
> separate concepts, superseded by this text.
> requires_human_decision: yes

**NEVER-DO-7** (target_owner: doctrine)
> proposed_rule: Do not add autonomous planning, self-direction, or multi-step execution that is
> not bounded by already-approved artifacts and explicit human checkpoints. Running Stages 4
> through 8 continuously under NN-1's batched delivery cycle is not an exception to this
> prohibition — it is execution bounded on both ends (Stage 3 approval in, Stage 8 human decision
> out) and constrained throughout by already-approved Intent/Contract/Event-Schema; it is not
> open-ended goal-setting or self-directed planning, which this prohibition continues to bar.
> requires_human_decision: yes

**REVIEW-LOG-1b** (target_owner: review policy)
> proposed_rule: Save a review only when its decision changes behavior, architecture, or an
> accepted risk. For ordinary corrections, the artifact diff and git history are enough.
> requires_human_decision: yes

