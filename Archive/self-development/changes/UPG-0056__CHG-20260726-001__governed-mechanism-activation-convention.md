# Self-Development Change: UPG-0056__CHG-20260726-001 — governed-mechanism-activation-convention

<!-- TRACE HEADER (canonical) -->
```yaml
feature_id: UPG-0056
primary_feature_id: UPG-0056
change_id: CHG-20260726-001
slug: governed-mechanism-activation-convention
state: COMPLETE
current_step: 4-Reconcile
implements:
  - UPG-0056
related_features: []
review_series: RVS__UPG-0056__CHG-20260726-001__S4
review_profile: PROFILE-4
review_state: REVIEWED
review_history: reviews/review-log.md
fixes_findings: []
follow_up_of: null
```

<!-- SELF-REFERENCE BOUNDARY: this artifact is itself reviewed, so it must NOT embed the current
review round (which does not exist until after the packet is built). Reference the stable review
SERIES (review_series) + review_state; exact rounds live only in reviews/review-log.md and
reviews/codex/*. See prompts/codeos-self-dev.md → "Feature Thread & IDs" / "Self-Reference Boundary". -->


## Change Intent

<!-- REVISED after UPG-0057 planning review: the original design (below, preserved in git history)
was a versioned governed-mechanism framework — a Rust resolver, 25 result codes, activation modes,
provenance stamps — disproportionate for what every known consumer actually needs: a human-readable
on/off switch. This revision replaces that with a minimal one-line convention, per the human's
explicit direction and the plan at
/home/rimo/.claude/plans/calude-consider-this-inputs-steady-pnueli.md. -->

**Why (problem in the toolkit):**

No future Codeos feature that needs a human-controlled on/off switch for AI doctrine/generation
behavior — the first known case being a "Controlled Plain English" writing-style discipline, whose
own UPG (`UPG-0057`) is deliberately deferred and out of scope here — has one shared, documented
convention to reuse. Without one, each such feature would invent its own ad hoc status-file shape
and wording, and prompts consulting it would each phrase the same three-way check ("is it on, is it
off, is it broken?") slightly differently.

**What changes:**

- `templates/conventions.md` — new "Optional Mechanism Status Convention" section: the exact
  one-line file grammar (`status: enabled` / `status: disabled`, nothing else), the four-outcome
  table (absent → disabled; exact `disabled` → disabled; exact `enabled` → enabled; anything else →
  stop and report a configuration error), whitespace/line-ending pinning, the human-only-changes-it
  rule, and the `architecture/` downstream placement rule (self-dev placement is intentionally left
  to the first feature that needs it — see Scope boundary).
- `templates/optional-mechanism-status.yaml` (new) — a minimal illustrative example file containing
  exactly `status: disabled`, so a future feature has an exact copy-paste starting point.
- `dba-system.md` — a new, short `## Optional Mechanism Status Convention` section: what it is,
  where it is fully defined, the `architecture/` downstream placement rule, an explicit statement
  that no current doctrine rule uses it yet. One new File Layout line.
- `backlog/UPG-0056-governed-mechanism-activation-convention.md` — already created (this brief);
  updated to match this revision.

**No code of any kind** — no Rust crate, no shell script, no resolver binary. A one-line status file
is read directly by whichever prompt consults it; that check is a few words in each prompt's own
text, not shared tooling.

**Lifecycle bookkeeping (standard for every non-trivial self-dev change, not substantive scope):**
`backlog/features.md` and `status/roadmap.md` gain one new row for **this feature, `UPG-0056`**;
`status/self-development.md` gets one new/updated row per step, per `prompts/codeos-self-dev.md`'s
Step 1 instruction to "activate the row." **These same two dashboard files, in the current working
diff, also carry a separate row for `UPG-0057`** (Controlled Plain English) — a distinct, dependent
feature registered during the same planning/working session, not implemented by this CHG. That
row is `UPG-0057`'s own bookkeeping, filed when its own plan was approved; it is named here so the
scope boundary below is accurate about everything actually present in the diff, rather than
undercounting it.

**Scope boundary — what stays the same:**

Anything not named above is in scope for no change. In particular, this change does **not**: create
any concrete status file anywhere (downstream or self-dev — no `architecture/*.yaml`, no
`config/*.yaml`); touch `scripts/dba-init.sh`, `CLAUDE.md`, or `prompts/codeos-self-dev.md`; add any
Controlled Plain English content (all deferred to `UPG-0057`); add any resolver, shell script, Rust
crate, provenance stamp, version field, result-code table, or absence-policy option of any kind;
change any Non-Negotiable Rule; add any new Stage ID; add any new mandatory human-approval gate; or
touch anything resembling a runtime product feature flag (out of category — this governs AI
doctrine/generation behavior, never application runtime behavior). `dba-system.md`'s 9-stage
substance, stage table, and existing Artifact Classification/File Layout entries are otherwise
untouched — only additive.

**Class:** downstream-doctrine (revised — no executable tooling remains in scope, so the earlier
combined `downstream-doctrine + script-tooling` classification no longer applies).
**Profile:** PROFILE-4.
**Scope axis:** downstream doctrine only
**Backlog item:** `backlog/UPG-0056-governed-mechanism-activation-convention.md`

---

## Acceptance Criteria

| # | Criterion | How it will be verified |
|---|---|---|
| 1 | `templates/conventions.md`'s new section states the exact one-line grammar: after ignoring blank lines, the file contains exactly one line — `status: enabled` or `status: disabled`, verbatim. | Read-through of the new section against this exact wording. |
| 2 | The four-outcome table is stated exactly: absent → disabled; exact `status: disabled` → disabled; exact `status: enabled` → enabled; anything else (unreadable, extra content, any other value) → stop and report a configuration error. | Read-through against this exact table; no fifth outcome or ambiguous case introduced. |
| 3 | Whitespace/line-ending handling is pinned: leading/trailing blank lines allowed; LF and CRLF both accepted after line-ending normalization; internal whitespace, case, tabs, and inline comments are **not** normalized and make the file invalid. | Read-through confirming all five sub-rules are present and none contradicts the four-outcome table. |
| 4 | Only an explicit human instruction creates, edits, or deletes a status file; an agent never does so on its own initiative. | Read-through of the new section for this explicit statement. |
| 5 | Git history is the sole audit trail. No resolver, schema version, activation id, `state_revision`, `authorized_by`/`authorized_at`/`reason`, or result-code vocabulary is introduced **as an actual field, mechanism, or tool**. | No YAML field or documented mechanism by any of these names exists in `templates/conventions.md`'s new section, `templates/optional-mechanism-status.yaml`, or `dba-system.md`'s new section. The words themselves may still appear inside a negation sentence explicitly stating the convention does *not* have one (e.g. "No resolver... git history is the audit trail") — that is the intended, accurate way to document the absence, not a violation. A plain `grep` for the words alone is not the verification method; a read-through confirming no such field/mechanism is actually defined is. |
| 6 | No dedicated parser, resolver, or helper script is introduced. | `git diff`/`git status` for this change shows no new `.sh`, `.rs`, or other executable/code file — only `templates/conventions.md`, `templates/optional-mechanism-status.yaml`, `dba-system.md`, and the standard lifecycle-bookkeeping files. |
| 7 | `templates/optional-mechanism-status.yaml` contains exactly one line, `status: disabled`. | `wc -l` and a direct read of the file. |
| 8 | The convention explicitly tells future consumers they may implement the check locally but must not broaden the grammar or invent additional states/fields. | Read-through of the new section for this explicit constraint on consumers. |
| 9 | Downstream placement is documented as `architecture/`; the convention text **explicitly states** self-dev placement is left undecided, to be introduced by the first feature that needs it (not scaffolded by this change). | Read-through confirming the "Placement" paragraph contains an explicit sentence deferring self-dev placement, not merely silence on the topic; confirm no `config/` directory or self-dev file is created by this change. |
| 10 | `dba-system.md`'s new section states explicitly that no current doctrine rule uses this mechanism yet; no Stage ID, Non-Negotiable Rule, or mandatory approval gate is added. | Read-through for the explicit "not yet used" statement; `git diff` on `dba-system.md` shows only additions, no stage-table or Non-Negotiable Rule edits. |
| 11 | **Downstream-compatibility:** a downstream project's generated `CLAUDE.md` still loads `.codeos/dba-system.md` unchanged; the new `templates/conventions.md` section is additive only — no existing convention entry is renamed, removed, or restructured. | `git diff` on `templates/conventions.md` shows only an added section; no existing heading changes. |
| 12 | This change carries low behavioral risk **not merely because there is no code** (doctrine text is itself behavioral surface), but because — per AC10 — no current prompt, stage, or doctrine rule references or is required to consult the new section; nothing in any existing prompt's behavior changes as a result of this change. | `grep` across `prompts/*.md` and `dba-system.md`'s existing (pre-change) sections for a reference to "Optional Mechanism Status Convention" or `templates/conventions.md`'s new section finds none outside the new section itself; combined with AC10, this shows the addition is inert until a future feature consumes it. |

<!-- No script-tooling-specific criteria (I/O behavior, exit codes, idempotency) apply — this
change ships doctrine/template content only, per the revised lean scope. -->

---

## Implementation Notes

Three files touched, exactly as scoped in Step 1 — no code of any kind:

- `templates/conventions.md`: new "Optional Mechanism Status Convention" section (additive, after
  "Correlation IDs") — exact grammar, four-outcome table, whitespace/line-ending pinning, the
  missing-means-disabled trade-off statement, "no shared tool" statement, placement rule.
- `templates/optional-mechanism-status.yaml` (new): exactly one line, `status: disabled`.
- `dba-system.md`: new short "## Optional Mechanism Status Convention" section (additive, inserted
  after "Contract-to-Implementation Failure Boundary", before "What You Do at Each Stage" — matching
  the existing insertion pattern for `UPG-0051`/`UPG-0052`/`UPG-0054`'s own sections); one new File
  Layout line under `architecture/`.

All 12 acceptance criteria verified directly (see Reconciliation below) before writing this section.
No out-of-scope items discovered; nothing deferred beyond what Step 1 already deferred (self-dev
adoption, any concrete status file, Controlled Plain English content — all `UPG-0057`'s job).

---

## Reconciliation

**Acceptance verification:**

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Exact one-line grammar stated | PASS | `templates/conventions.md`, "Exact grammar" — exactly one non-blank line, `status: enabled`/`status: disabled` verbatim. |
| 2 | Four-outcome table stated exactly | PASS | `templates/conventions.md`, "Four outcomes only" table — absent/disabled/enabled/anything-else, matches Step 1/2 wording exactly. |
| 3 | Whitespace/line-ending pinning present | PASS | `templates/conventions.md`, "Whitespace and line endings" — blank lines, LF/CRLF, internal whitespace/case/tabs/comments all addressed. |
| 4 | Human-only authority stated | PASS | `templates/conventions.md`, "Missing means disabled" paragraph — explicit human-instruction requirement, agent never acts on its own initiative. |
| 5 | No resolver/schema/provenance field or mechanism actually introduced (negation prose permitted) | PASS | `grep -n "^schema_version:\|^activation_id:\|^state_revision:\|^authorized_by:\|^authorized_at:"` against both new files returns no match; the words "resolver"/"result codes" appear only inside negation sentences explicitly disclaiming them. |
| 6 | No new code file | PASS | `git status --porcelain \| grep -E "\.(sh\|rs)$"` returns nothing. |
| 7 | Example file is exactly one line, `status: disabled` | PASS | `wc -l templates/optional-mechanism-status.yaml` → 1; `cat -A` shows `status: disabled$`. |
| 8 | Consumer constraint (no broadening the grammar) stated | PASS | `templates/conventions.md`, "No shared tool" paragraph. |
| 9 | Self-dev placement explicitly deferred (not merely silent) | PASS | `templates/conventions.md`, "Placement" paragraph, final two sentences. |
| 10 | `dba-system.md` states no current doctrine rule uses it; no Stage ID/Non-Negotiable Rule/gate added | PASS | New section's explicit sentence; `git diff dba-system.md` shows only additions (0 removed lines). |
| 11 | Downstream-compatibility: `templates/conventions.md` change is additive only | PASS | `git diff templates/conventions.md \| grep "^-" \| grep -v "^---"` → 0 lines. |
| 12 | Low-risk claim correctly grounded in AC10, not "no code" alone | PASS | `dba-system.md`'s explicit "not yet used" statement + `grep -rl "Optional Mechanism Status Convention" prompts/` returns nothing — no existing prompt behavior changes. |

**Consistency sweep:** grep across the repo for stale references to the superseded Rust-resolver
design (`codeos-activation-resolver`, `activation-resolver` binary/crate name) returns no matches
outside this change's own filename/slug (`governed-mechanism-activation-convention`, which is an
identifier, not a reference to removed content) and historical `reviews/` artifacts (expected,
append-only record). `dba-system.md`'s stage table, Non-Negotiable Rules, and existing Artifact
Classification/File Layout entries are unchanged except the one new File Layout line. No orphaned
links introduced.

**Findings scope-triage:** all findings raised across Steps 1-3 (10 total, listed in the backlog
Feature Thread) were IN-SCOPE BLOCKER and fixed inline within this same CHG; none was OUT-OF-SCOPE
BACKLOG, REJECTED, or SELF-REFERENCE/REVIEW-BOOKKEEPING. No new findings surface at Reconcile.
