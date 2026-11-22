# WO-UL-01 — Default Guided User Learning

- **Class:** PROTECTED — semantics §1 and the three terms approved by the human 2026-11-15
- **Baseline:** DBA-31
- **Goal:** CodeOS includes learning during development and in user-facing solutions by default,
  but it can be switched off without affecting core functionality or mandatory control mechanisms.

Planning context only. Git and the governing policies stay authoritative. Translated from the
human's Slovenian original (2026-11-15). Two decisions taken while planning are applied here:
"Learning Checkpoint" is renamed **Learning Pause**, because "Checkpoint" is already a governed
term, and the **learning journal** is added (§2).

## 1. Accepted semantics

### Learning during development

For a significant new concept, CodeOS uses this loop:

1. Explain: briefly explain the concept and its purpose.
2. Predict: the user predicts the result of a change.
3. Observe: the result is actually executed and observed.
4. Reproduce: the user performs a related step without click-by-click instructions.

The following rules apply:

- The learning mode is guided by default.
- A check is a soft pause. CodeOS waits for an answer, but a wrong answer is not a formal block on
  development.
- CodeOS corrects a misunderstanding and, where needed, offers an easier example.
- Routine commands and concepts already shown to be understood do not trigger new questions.
- At most one significant new concept is handled at a time.
- A learning answer never replaces an approval, a test, runtime evidence or UX validation.
- Switching learning off does not remove safety warnings, explanations of the consequences of
  decisions, or the information needed for informed approval.

### Learning in delivered solutions

For solutions with a human user interface, guided learning is on by default:

- the solution may explain important concepts, results and the consequences of the user's actions;
- where it makes sense, it uses prediction, observation and an independent repeat step;
- every user has a Guided learning setting, on by default;
- switching it off removes learning explanations, questions and exercises, not business
  functionality;
- solutions without a human user may record not-applicable with a reason.

Removing runtime learning from an already approved solution is a change in product behavior. It
requires a revision of the Charter and of the affected Specification Packages. A CodeOS setting
must not remove it directly.

### Terms

- User Learning Mode: the mode that determines whether CodeOS uses learning interactions.
- Learning Pause: a soft pause for a prediction or an independent repeat.
- Learning Progress: a minimal local record of the stage reached for each concept.
- The existing Learning Register remains a register of operational findings about the system and
  is not used to track the user's knowledge.

## 2. Implementation

### Policy and activation

- Add a versioned user_learning_policy as a standalone, selectable CodeOS policy.
- Add the policy selection to the configuration resolver and to the project instructions for
  interactive CodeOS sessions.
- Prepare DBA-32 as DBA-31 with the learning policy added.
- DBA-32 becomes the default configuration for new projects only after qualification and human
  approval of the protected semantics.
- Existing projects stay unchanged and get the functionality only through an explicit
  configuration upgrade.

### Local development setting

Add a small command, codeos-learning:

```text
codeos-learning status
codeos-learning set-mode guided|off
codeos-learning record <concept-id> introduced|predicted|observed|reproduced
codeos-learning journal <concept-id> --question <text> --answer <text> [--correction <text>]
codeos-learning reset-progress
```

State is kept in .codeos-state/user-learning.json:

- it is not included in Git;
- it is limited to the current project;
- it contains no user answers, personal data or grades;
- it does not use a mastered label;
- corrupt or missing state does not block technical development.

Learning journal: after each Learning Pause, unless the mode is off, CodeOS appends one entry to
.codeos-state/learning-journal.md in the simplest form: date, concept, question, the user's
answer, and the correction if any. The journal:

- is not included in Git and is limited to the current project;
- is append-only plain text, with no grades, scores or mastered label;
- is the only place user answers are recorded, so user-learning.json itself stays free of answers;
- is deleted by reset-progress.

A temporary request to "switch learning off for this session" overrides the local mode only until
the end of the session.

### Solution runtime contract

For projects that select the policy, the Charter adds this decision:

```yaml
user_learning:
  mode: guided | off | not-applicable
  audience: <primary user>
  opt_out: per-user | n/a
```

The default proposal for a human interface is guided.

The Contract of every feature that introduces a significant new user concept specifies:

- the concept and the expected learning outcome;
- the learning element the user actually notices;
- behavior when the learning mode is off;
- the invariant that core results are the same in guided and off modes;
- how runtime verification is done and how the user setting persists.

If the solution already has user profiles, the setting is stored per user. Otherwise a local device
or session setting is used. There is no global knowledge profile across projects.

## 3. Verification and acceptance scenarios

1. A new project on DBA-32 starts in guided mode.
2. A significant new concept triggers the full explain → predict → observe → reproduce loop.
3. A wrong prediction triggers a correction, not a formal rejection of the implementation.
4. A routine step does not trigger an unnecessary learning question.
5. set-mode off removes learning questions but keeps mandatory warnings and approvals.
6. Switching back on uses the existing local progress.
7. Local state is not tracked in Git. user-learning.json contains no answer text. The learning
   journal records each Learning Pause's question and answer, is not tracked in Git, and is deleted
   by reset-progress.
8. A new GUI/CLI solution proposes runtime mode guided and per-user opt-out.
9. Switching it off in a running solution hides learning elements, while the core result and the
   business flow stay the same.
10. A headless project may use not-applicable with a reason.
11. A change of the runtime mode after approval is routed through the Charter and the affected
    specifications.
12. A project on DBA-31 or an older configuration stays behaviorally unchanged.
13. A qualification replay confirms that learning gains no approval or verification authority.
14. A pilot project with a real human user confirms understandability, switching off and switching
    back on.

## 4. Excluded from this WO

- a global knowledge profile across projects;
- user assessment, points, certificates or an LMS;
- central telemetry of learning answers;
- strict knowledge conditions for technical progression;
- automatic changes to already approved solutions;
- using learnings.md for the user's learning record.

Protected invariant: learning increases the user's understanding, but never becomes a substitute
for human decision-making, approval, testing or runtime evidence. Switching learning elements off
must not change the core behavior of the developed solution.
