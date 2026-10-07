# CodeOS × Jev Capability and Upgrade Analysis

> **Status:** Non-authoritative architecture analysis. This report recommends no governance or
> implementation change by itself.
>
> **Repository snapshot:** `4bc96c7e90ceab997dea082800b54ad657ef6c47` on 2026-09-24, with
> uncommitted DBA-16/17 worktree capability assessed separately as candidate capability.

## Executive conclusion

CodeOS is strongest where decisions can be reduced to deterministic evidence and weakest where
semantic classification occurs before those deterministic checks. Its mature pattern is:

```text
human-approved meaning
→ LLM-assisted production and review
→ deterministic identity/evidence enforcement
→ explicit human authority
```

Jev has a meaningful but narrow role between the LLM and deterministic enforcement. It should
classify or prioritize bounded semantic objects; it should not synthesize architecture, judge final
conformance, advance workflow state, or approve anything.

The top opportunities are:

1. **Review-finding triage** — best first pilot, in shadow mode.
2. **Change-impact and architecture-reassessment triggers** — valuable after triage establishes
   calibration and operating economics.
3. **Supplemental review-context prioritization** — potentially valuable for tokens and attention,
   but only after preserving a deterministic mandatory-evidence floor.

The earlier “implementation → architecture conformance” hypothesis does not survive scrutiny as a
direct Jev use. That decision is multi-hop, context-heavy, consequential, and explanation-dependent.
Jev may flag reasons to request deeper architecture review; it should not decide conformance.

Do not build a generic Typed Semantic Decision Layer now. Run one isolated triage pilot first. Build
a shared adapter only if a second pilot independently succeeds and requires the same model-version,
audit, calibration, privacy, and fallback lifecycle.

The first pilot should not replace Codex classification. It should independently classify existing
findings, expose disagreements and confidence, and measure whether this improves consistency or
human attention enough to justify another API dependency. Initially it avoids no frontier call; its
value must come from decision quality or reduced human triage.

### Repository capability states

- **Active default:** [`dba-system.md`](../../dba-system.md) selects DBA-9 for new projects.
- **Committed selectable capability:** immutable compositions through DBA-15 exist and can be
  explicitly adopted.
- **Candidate capability:** DBA-16/17 and their Terminal Review Closure implementation are
  uncommitted worktree changes. They receive no maturity credit.
- **Historical evidence:** archived CodeOS experiments inform economics and failure modes but are
  not current authority.

## Current capability and maturity

Maturity meanings:

- **M0:** absent
- **M1:** experimental or instruction-only
- **M2:** functional
- **M3:** committed, governed, enforced, and regression-tested
- **M4:** repeatedly demonstrated on real downstream work

Nothing qualifies as M4. JamFree supplies one valuable real downstream case, but M4 requires
repeated operational exercise.

| Capability | Current mechanism and owner | Maturity | Main weakness | Frequency | Wrong-decision consequence | Upgrade potential |
|---|---|---:|---|---|---|---|
| Intent and requirement governance | LLM drafts; human approves; reviewer checks | M3 | Semantic quality remains dependent on human/LLM judgment | Per feature | High | Low for Jev |
| Contract/specification formation | Claude/Codex-assisted formation under fixed artifact roles | M3 | Completeness and correct abstraction are not mechanically decidable | Per feature/revision | High | Low |
| Architecture synthesis | Frontier LLM plus explicit human approval | M3 | Significance and sufficiency are semantic | Per architectural scope | Very high | Low |
| Architecture conformance | Metadata inspector and journey checks plus reviewer judgment | M2 | No general semantic code-to-architecture conformance engine | Per implementation/review | High | Medium as trigger only |
| Change-impact reasoning | Feature Impact Accounting, prompts, reviewer, human | M2 | Mostly declared/judged rather than independently inferred | Every material change | High | High |
| Implementation | Claude/Codex/human; disabled delegated implementer experiment | M2 | Alternative-model delegation was not net-positive | Every feature | High | Low |
| Test generation and verification | LLM creates tests; deterministic runners execute them | M3 | Generated test quality still needs semantic review | Every feature | High | Medium for routing, not generation |
| Evidence collection/classification | Hash-bound records, run-bounded events, deterministic validation; reviewer judges adequacy | M3 | Relevance and sufficiency remain semantic | Every feature | High | Medium–high |
| Reconciliation | Structured prompt and human/LLM comparison; receipt records completion | M2 | Reconciliation content is inline and semantically produced; completion does not mean gaps closed | Every feature | High | Medium |
| Review | Isolated Codex packet, strict parser, immutable records, human decisions | M3 | Expensive semantic pass; classifications can be inconsistent | Multiple times per feature | High | High |
| Review-finding triage | Codex emits five governed labels; parser validates form | M2 | Format is enforced, classification correctness is not independently evaluated | Every finding | Medium–high | Very high |
| Workflow/progression | Deterministic checkpoint engine with current bindings and fail-closed ordering | M3 | Complexity has grown around exceptional recovery paths | Every status/check | Very high | Very low |
| Model/tool/task routing | Fixed Codex reviewer, manual/configured effort; alternative paths are external or disabled | M1 | No calibrated router or measured selection policy | Every AI task | Medium | Medium |
| Context selection/management | Caller selects artifacts; packet builder supports full/delta/hash-only modes and budget refusal | M2 | Semantic relevance selection remains manual | Every review | Medium–high | High for supplemental context |
| Failure/retry/escalation | Deterministic exit paths, bounded retries, review budgets, explicit human escalation | M3 | Different mechanisms handle different failure classes | Frequent | High | Low–medium |
| Cross-artifact consistency | Prompts/reviewer plus deterministic approval, revision, ID, and selected structural checks | M2 | Semantic consistency still requires wide-context reasoning | Per package/revision | High | Medium as anomaly trigger |
| Runtime/production evidence | Verification records, bounded event evidence, production-source evidence in selectable DBA-15 | M2 | Recent and only narrowly exercised downstream | Per verification | High | Low for currency; medium for relevance |
| Replay and recovery | Deterministic semantic event hashes, replay rules, review/evidence recovery mechanisms | M3 committed through DBA-15 | Lifecycle is sophisticated and has exposed edge cases | Per acceptance/failure | High | Low |
| Human decision boundaries | Doctrine, explicit approvals, receipts, append-only decisions | M3 | Correct human attention still depends on presentation and triage | At consequential gates | Very high | Medium for prioritization only |
| Cost/token optimization | Delta/hash-only packets, packet budgets, token logging, provider experiments | M2 | No automated routing; measured alternative-model delegation was negative | Every review/agent call | Medium | High if measurable |

### Where CodeOS deliberately avoids probabilistic judgment

These should remain deterministic:

- configuration resolution and composition compatibility;
- IDs, hashes, content identities, append ordering, and staleness;
- schema parsing and prohibited-field checks;
- checkpoint ordering and binding currency;
- review-round and recovery cardinality;
- required artifact presence and approval metadata;
- event-set boundaries and schema validation;
- authorization existence and human-decision provenance;
- secret redaction and filesystem/isolation rules.

The relevant implementations are concentrated in the
[`workflow engine`](../../dba/04-tools/workflow/engine/src/checker.rs),
[`event evidence validator`](../../dba/04-tools/workflow/engine/src/event_evidence.rs), and
[`reviewer engine`](../../dba/04-tools/reviewer/engine/src/main.rs).

### Candidate DBA-16/17 capability

DBA-16/17 introduce package-bound re-entry and Terminal Review Closure. Their current worktree
implementation is tested, but uncommitted capability does not raise any rating above. It does show
a future architectural concern: as recovery semantics multiply, CodeOS benefits more from clearer
deterministic state models—not from probabilistic progression decisions.

## Decision inventory

| Decision | Current owner | Current weakness | Best owner | Why change? | Jev? |
|---|---|---|---|---|---|
| Approve purpose or product meaning | Human | Human effort | Human | Normative authority cannot be delegated | No |
| Draft Intent/Contract/Event Schema | Frontier LLM + human | Expensive and variable | Frontier LLM + human | Requires creation and explanation | No |
| Validate artifact shape and required fields | Deterministic tool | Limited to expressible rules | Deterministic tool | Already exact and cheap | No |
| Decide whether architecture synthesis applies | Claude/human | Can be inconsistent | Jev trigger → frontier/human decision | Bounded warning could focus attention | Pilot |
| Synthesize architecture | Frontier LLM + human | Expensive | Frontier LLM + human | Deep multi-step synthesis | No |
| Validate architecture metadata/membership | Deterministic inspector | None material | Deterministic inspector | Exact rule | No |
| Judge implementation conformance to architecture | Reviewer/human | Expensive, broad context | Frontier reviewer/human | Requires reasoning and explanation | No direct Jev |
| Classify change impact | Claude/reviewer/human | Repeated and inconsistent | Jev advisory trigger + existing owner | Naturally multi-label and frequent | Pilot |
| Decide whether a semantic revision is legitimate | Human approval plus deterministic eligibility | Materiality is semantic | Human | Review-budget consequences are high | No |
| Generate implementation | Frontier LLM/human | Token-heavy | Frontier LLM/human | Prior cheaper-model trials were negative | No |
| Generate tests | Frontier LLM/human | Quality varies | Frontier LLM/human | Requires construction | No |
| Select likely test area | Agent/human | May over-run suites | Jev or rules as routing | Closed routing decision | Later |
| Execute and evaluate deterministic tests | Tools | None material | Deterministic tools | Verifiable | No |
| Classify CI failure family | Human/agent | Repetitive | Jev routing with rule-first handling | Bounded taxonomy; low authority | Later |
| Estimate flaky-test likelihood | Human/history | Weak capability | Statistical model or Jev advisory | Needs longitudinal data CodeOS lacks | Later |
| Establish evidence freshness | Hash/binding logic | None material | Deterministic tool | Exact identity question | No |
| Judge evidence relevance | Reviewer/human | Expensive and variable | Jev advisory + reviewer | Bounded semantic relation | Later |
| Judge evidence sufficiency | Reviewer/human | Expensive | Frontier reviewer/human | Nuanced claim-bound reasoning | No direct Jev |
| Reconcile requirement layers | Claude/reviewer | Large tables and repeated classifications | Frontier LLM with possible Jev pre-triage | Cross-layer explanation required | Later |
| Generate reviewer findings | Codex | Expensive | Codex/frontier reviewer | Requires open-ended defect discovery | No |
| Classify an existing finding | Codex, then deterministic parser | No independent correctness check | Jev advisory cross-check | Closed five-label decision | Pilot |
| Determine blocker kind | Codex | Inconsistent boundary cases | Jev advisory, reviewer/human controls | Two-way bounded classification | Pilot within triage |
| Detect duplicate findings | Human/reviewer | Repeated findings consume attention | Jev advisory cluster | Pairwise semantic similarity | Later |
| Select mandatory review evidence | Human/caller plus rules | Manual | Deterministic floor + human | Omitting primary evidence is consequential | No Jev authority |
| Prioritize supplemental context | Human/caller | Token-heavy and inconsistent | Jev advisory ranking | Recoverable if mandatory floor remains | Pilot |
| Select reviewer provider | Fixed policy | No dynamic economics | Fixed policy until stronger evidence | Existing alternatives failed quality tests | Later |
| Select reasoning effort | Configured manually | Always-high may be costly | Jev advisory router after calibration | Bounded routing but quality-sensitive | Later |
| Retry after transport failure | Deterministic bounded policy | None material | Deterministic policy | HTTP status/time budget is exact | No |
| Continue or stop a review loop | Human within bounded rules | Attention cost | Deterministic signals + human | Consequential and history-sensitive | No direct Jev |
| Advance workflow checkpoint | Deterministic engine/human receipt | None justifying probability | Existing owner | Must remain reproducible and fail-closed | No |
| Prioritize human attention | Presentation and human judgment | High volume | Jev advisory ranking | Useful if it never suppresses blockers | Later |

## What Jev actually provides

Jev is not a coding or reviewing LLM. It does not generate code, prose, explanations, or tool
calls. It evaluates one text/JSON state against typed questions and returns:

- `Choice`: one predefined option, option probabilities, and confidence;
- `Score`: a position across 2–10 ordered levels, probabilities, and confidence;
- `Noul`: probability of a yes/true answer.

Questions sharing a state are evaluated independently and in parallel. Choice supports up to 255
options. Sources: [TypeSafe primitives](https://docs.typesafe.ai/primitives) and
[API reference](https://docs.typesafe.ai/api).

Documented Jev 1.13 constraints include:

- 64k total context and 32k for state plus the longest question;
- text/JSON only;
- English is the strongest language;
- `$0.042` per million input tokens and unmetered output;
- 250,000 tokens/second and 1,200 requests/minute, currently subject to change;
- `jev-latest` aliases can move, while the response reports the concrete version;
- customer requests are not used for training; zero-data-retention is an enterprise option.

Source: [TypeSafe models](https://docs.typesafe.ai/models).

The vendor reports 70–500 ms latency and calibrated probabilities. These are vendor claims, not
CodeOS measurements. Confidence indicates distribution concentration, not correctness of an
individual answer; thresholds must be calibrated on CodeOS data. Source:
[TypeSafe confidence](https://docs.typesafe.ai/confidence).

Important documented weaknesses match several CodeOS risks:

- literal interpretation;
- weak numeric precision and counting;
- reduced accuracy with indirection;
- distraction from large irrelevant states;
- sensitivity to adversarial content;
- no guaranteed arithmetic consistency between related questions;
- no text generation.

Source: [Jev 1.13 jaggedness](https://docs.typesafe.ai/model-jaggedness/jev-1.13).

Type safety therefore means “the result is one of the declared values,” not “the selected value is
correct.” Official documentation describes Jev as consistent, but does not promise deterministic
replay. CodeOS would need to preserve each request, concrete model version, response, and local
threshold result to explain a historical branch.

Failure modes include authentication and validation errors, rate limits, overload, timeouts, and
connection failures. SDKs support bounded retry/backoff, but CodeOS should fall back to the existing
owner rather than convert provider failure into a negative semantic result.

## Critical assessment of the supplied hypotheses

| Candidate | Current maturity | Jev suitability | Key conclusion |
|---|---:|---|---|
| Implementation → architecture conformance | M2 | Low | Too broad, multi-hop, and consequential. Use Jev only as a possible reassessment trigger. |
| Change-impact classification | M2 | High for advisory multi-label questions | Strong second pilot. Never let a negative result suppress existing review until calibrated. |
| Architecture-decision decomposition | M3 synthesis process | Low | Requires creating and relating architectural meaning. Keep with frontier LLM and human. |
| Review-finding triage | M2 | High | Best first pilot because the output taxonomy already exists and historical examples are plentiful. |
| Test/evidence coverage classification | M2–M3 | Medium | Relevance is suitable; sufficiency is not. Separate those decisions. |
| Task/model/tool routing | M1 | Medium | Natural typed output, but CodeOS lacks trustworthy outcome labels and viable interchangeable providers. |
| Artifact consistency scanning | M2 | Medium | Deterministic comparisons first; Jev only for semantic anomaly flags. |
| General code-quality judgment | M2 | Low | Open-ended, explanation-heavy, and not safely reduced to one closed label. |

### Additional candidates

- **Context compaction:** useful only as supplemental relevance ranking. Jev should not rewrite or
  summarize authoritative context.
- **LLM invocation avoidance:** plausible for isolated classification calls, but most current
  semantic decisions happen inside an already-running coding-agent session. Savings are therefore
  unknown.
- **Finding clustering:** technically suitable but lower value than correct initial triage.
- **Review-depth selection:** attractive but dangerous; a false “shallow review” decision can hide
  defects.
- **CI routing:** suitable in principle, but CodeOS has no current labeled failure corpus.
- **Runtime anomaly triage:** outside current core capability and requires operational data.
- **Delegated implementation suitability:** unsuitable as an execution gate; prior delegation
  failures show that superficially bounded specifications still demand deep invariant reasoning.

## Ranked opportunity matrix

| Rank | Opportunity | Current maturity | Jev fit | Expected value | Complexity | Risk | Recommendation / owner |
|---:|---|---:|---|---|---|---|---|
| 1 | Review-finding triage cross-check | M2 | High | Consistency and human-attention savings; token saving initially zero | Low–medium | Low in shadow mode | **PILOT — Jev advisory** |
| 2 | Change-impact and architecture-reassessment trigger | M2 | High | Earlier escalation and fewer unnecessary deep analyses | Medium | Medium–high false-negative cost | **PILOT — Jev escalation trigger** |
| 3 | Supplemental review-context prioritization | M2 | High | Potential packet/token reduction and better focus | Medium | High if allowed to omit primary evidence | **PILOT — advisory only** |
| 4 | Duplicate-finding detection | M1 | Medium–high | Less repeated human work | Medium | Hidden distinction if merged incorrectly | **LATER — advisory cluster** |
| 5 | Evidence relevance triage | M2 | Medium–high | Faster reconciliation/review | Medium | Relevance confused with sufficiency | **LATER — advisory** |
| 6 | CI/test-failure routing | M0 in CodeOS | High abstract fit | Could save agent attention | Medium | Low if routing only | **LATER — insufficient corpus** |
| 7 | Human-attention prioritization | M1 | Medium | Better ordering of review work | Medium | Delayed blocker attention | **LATER — advisory** |
| 8 | Model/reasoning-effort routing | M1 | Medium | Possible token and latency savings | High | Missed defects; poor labels | **LATER** |
| 9 | Scope-drift semantic warning | M2 | Medium | Earlier detection | Medium | False reassurance | **LATER — positive trigger only** |
| 10 | Full architecture conformance | M2 | Low | Potentially large but unsafe | High | Very high | **DO NOT USE JEV** |
| 11 | Architecture synthesis/decomposition | M3 | Low | Little | High | Very high | **DO NOT USE JEV** |
| 12 | General code-quality review | M2 | Low | Unclear | High | High | **DO NOT USE JEV** |
| 13 | Evidence freshness/staleness | M3 | None | None | Adds needless dependency | High relative to value | **DO NOT USE JEV** |
| 14 | Workflow progression/acceptance | M3 | None | None | Harmful | Very high | **DO NOT USE JEV** |
| 15 | Retry/cardinality/authorization rules | M3 | None | None | Harmful | Very high | **DO NOT USE JEV** |

## First pilot: review-finding triage

### Decision

For an already-generated reviewer finding:

> Which existing CodeOS triage category best describes this finding?

Jev does not decide whether the finding is true, close it, change its severity, or advance workflow
state.

### Typed output

Primary `Choice`:

```yaml
classification:
  in_scope_blocker
  in_scope_non_blocker
  out_of_scope_backlog
  rejected
  self_reference_review_bookkeeping
```

For findings classified as `in_scope_blocker`, ask a second independent `Choice` in the same
request:

```yaml
blocker_kind:
  substantive
  evidence_insufficient
  not_applicable
```

Code ignores `blocker_kind` unless the first result is `in_scope_blocker`. Low-confidence or
structurally contradictory results become `ESCALATE`; they are never coerced into a governance
label.

### Input

Send only:

- finding summary;
- evidence, why, required action, and scope reason;
- feature/stage identifier;
- reviewed artifact role;
- concise approved change scope;
- canonical definitions of the five labels and two blocker kinds.

Do not send the entire packet, repository, secrets, raw runtime logs, or unrelated findings.

### Dataset

The repository contains approximately:

- 1,116 recorded `IN-SCOPE BLOCKER` occurrences;
- 72 `IN-SCOPE NON-BLOCKER`;
- 2 `OUT-OF-SCOPE BACKLOG`;
- 16 `SELF-REFERENCE / REVIEW-BOOKKEEPING`;
- no reliably discovered `REJECTED` examples in the same scan.

These are not gold labels. They are highly imbalanced and may contain duplicated historical text or
model mistakes.

Build the pilot corpus by:

1. extracting complete finding blocks structurally;
2. deduplicating exact and near-exact historical copies;
3. stratifying by stage, artifact role, and existing label;
4. having two humans independently label a held-out sample;
5. adjudicating disagreements without seeing Jev’s answer;
6. treating absent categories as a dataset limitation, not filling them with invented examples.

If a meaningful evaluation set cannot be established for every required category, narrow or
abandon the pilot rather than claiming five-class performance.

### Decision boundary

```text
Codex finding
→ Jev shadow classification
→ compare with Codex label
→ disagreement/low confidence highlighted to human
```

During the pilot:

- the Codex label remains the recorded reviewer output;
- the parser and all deterministic safeguards remain unchanged;
- Jev creates no review record or decision receipt;
- no workflow state consumes Jev output.

### Calibration

Split adjudicated examples into calibration and locked evaluation sets.

Use the calibration set to choose:

- minimum Choice confidence;
- acceptable margin between first and second options;
- any class-specific escalation rules.

Do not select a universal threshold in advance. Because false negatives are more costly than false
positives, calibration must prioritize recall of true blockers. Report coverage—the fraction routed
automatically—alongside accuracy.

Pin the concrete Jev model version during evaluation. A model upgrade requires replaying the locked
set and explicitly accepting the new operating point.

### Measurements

Measure:

- macro and per-class accuracy;
- blocker false-negative rate;
- non-blocker false-positive rate;
- blocker-kind accuracy;
- calibration by confidence band;
- disagreement rate with Codex;
- human adjudication time with and without Jev;
- request latency and TypeSafe input tokens;
- any reduction in Codex prompt/output tokens;
- provider failure/escalation rate;
- total integration and maintenance effort.

Initial frontier-call savings are expected to be **zero** because Codex still generates the finding.
This pilot succeeds only if independent triage materially improves correctness or human attention.

### Abandon criteria

Do not adopt if any of these holds:

- blocker false negatives remain unacceptable at useful coverage;
- confidence does not separate correct from incorrect classifications on CodeOS data;
- human adjudication time is not reduced;
- Jev mostly repeats Codex without catching meaningful errors;
- maintaining the API, privacy controls, model pin, calibration set, logs, and regressions costs more
  than the benefit;
- required source or review material cannot be sent under acceptable data-handling terms;
- model upgrades require disproportionate requalification.

## Roadmap and explicit non-recommendations

### Suggested roadmap

```text
Shadow triage pilot
→ measure quality, calibration, human time, and total complexity
→ stop if not clearly beneficial
→ change-impact trigger pilot
→ supplemental-context pilot
→ build a shared adapter only if at least two pilots succeed
```

If a shared adapter becomes justified, its minimum durable record should include:

- decision adapter and question-spec version;
- pinned provider/model version;
- request-state hash;
- typed answers and probabilities;
- local threshold/configuration version;
- derived route;
- latency and token usage;
- fallback or escalation result.

It should preserve the response that caused a branch; it should not claim probabilistic replay will
reproduce the same response.

### Explicit non-recommendations

Do not use Jev for:

- product or specification approval;
- architecture synthesis or architectural conformance verdicts;
- semantic-revision legitimacy;
- final evidence sufficiency;
- general code review;
- implementation or test generation;
- workflow checkpoint derivation;
- review-round cardinality or recovery eligibility;
- hash, schema, freshness, provenance, or configuration checks;
- autonomous retry, acceptance, or artifact mutation.

Those decisions either already have exact deterministic solutions or require deep reasoning,
explanation, or human authority.

## Final recommendation

Jev is potentially useful to CodeOS as a **cheap independent classifier and attention router**, not
as a new governance layer. Review-finding triage remains the best first pilot because it is
frequent, typed, non-authoritative, easy to run in shadow mode, and supported by a substantial—though
not yet gold-standard—historical corpus.
