Ta celostni arhitekturni model in razvojni krog se v sodobnem AI-native inženirstvu imenuje **Declarative Behavioral Architecture (DBA)** ali bolj specifično **Intent-Driven Architecture (IDA)**, včasih opisan tudi kot **Continuous Behavioral Verification Loop**.

To je model, ki ga v svojem jedru implementira **Codeos**. Za razliko od tradicionalnih arhitektur (ki se osredotočajo na strukturo kode in podatkovne baze) se ta model osredotoča izključno na **tok in preverjanje namena (intent)** skozi celotno življenjsko dobo programske opreme.

---

What you describe maps very well to a **human-supervised AI engineering loop** rather than a fully autonomous agent system.

For your stated preference — *more manual interventions/prompts than autonomy* — the simplest system is:

---

# Minimal Intent-Driven AI Dev System

## Core Principle

You do **not** let the AI freely generate software.

Instead:

1. Human defines:

   * intent


2. AI only (human reviews):

   * verifies and corrects intent format or specifies missing information
   * behavioral contracts
   * acceptance criteria
   * observable events
   * proposes implementations
   * proposes tests
   * proposes event schemas
   * proposes refinements

3. You manually approve every transition.

This gives you:

* stability
* predictability
* debuggability
* low cognitive overhead
* low infrastructure complexity

while still getting most AI productivity gains.

---

# Recommended Stack (Simple Version)

## Editor + AI

* [Visual Studio Code](https://code.visualstudio.com?utm_source=chatgpt.com)
* [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview?utm_source=chatgpt.com)
* [Gemini API Platform]()

Use:

* Claude Code for:

  * repository reasoning
  * large refactors
  * implementation
  * code navigation

Use Gemini model (2.5) for:

* architecture
* contract generation
* behavioral review
* refinement loops
* system design
* debugging reasoning
* event modeling

---

# Simplest Architecture

Do NOT start with:

* autonomous agents
* vector DBs
* orchestration frameworks
* multi-agent systems
* memory graphs
* LangChain-style complexity

You do not need them initially.

Start with only:

```text
/intents
/contracts
/events
/modules
/tests
/prompts
```

---

# Folder Structure

```text
project/
├── intents/
│   ├── feature_001.md
│   └── ...
│
├── contracts/
│   ├── feature_001_contract.md
│   └── ...
│
├── events/
│   ├── event_schema.md
│   └── runtime_events.jsonl
│
├── modules/
│   └── actual_code/
│
├── tests/
│   ├── behavioral/
│   └── replay/
│
└── prompts/
    ├── implement.md
    ├── refine.md
    ├── verify.md
    └── debug.md
    └── ...
```

This is enough.

---

# The Development Loop

## STEP 1 — Define Intent

Human writes:

```md
# Intent

User can upload CSV.

System validates:
- schema
- duplicates
- malformed rows

System emits events:
- upload_started
- row_validated
- row_rejected
- upload_completed
```

---

## STEP 2 — Define Behavioral Contract

BDD-style:

```gherkin
Given a valid CSV
When uploaded
Then all rows are validated
And upload_completed is emitted

Given malformed rows
When uploaded
Then row_rejected is emitted
```

---

# STEP 3 — Define Event Spine

Very important.

Example:

```json
{
  "event": "row_rejected",
  "correlation_id": "uuid",
  "row_number": 14,
  "reason": "missing_email",
  "timestamp": "..."
}
```

Now the AI is constrained.

It cannot invent hidden behavior.

---

# STEP 4 — Ask Claude Code to Implement

Example prompt:

```text
Implement ONLY the behavior described in:

/intents/feature_001.md
/contracts/feature_001_contract.md
/events/event_schema.md

Constraints:
- no additional abstractions
- emit all required events
- tests required
- implementation must be deterministic
```

This is the critical part:
you always constrain implementation with contracts + events.

---

# STEP 5 — Verification Loop

Use Gemini API for review:

```text
Compare:
- intent
- contracts
- emitted events
- tests
- implementation

Find behavioral mismatches.
Do not suggest rewrites.
Suggest only targeted refinements.
```

This creates your:

* refinement loop
* reconciliation loop
* behavioral verification loop

without complex infrastructure.

---

# STEP 6 — Replayability

Store runtime events:

```text
/events/runtime_events.jsonl
```

Example:

```json
{"event":"upload_started"}
{"event":"row_validated"}
{"event":"row_rejected"}
{"event":"upload_completed"}
```

Now you can:

* replay failures
* debug AI behavior
* reconstruct state
* verify contracts

This is your lightweight "Event Spine".

---

# Why This Works

You are separating:

| Layer          | Responsibility      |
| -------------- | ------------------- |
| Intent         | Human + AI          |
| Contract       | AI + Human          |
| Events         | AI + Human-defined truth |
| Implementation | AI                  |
| Verification   | AI + Human          |
| Approval       | Human               |

This is the stable boundary.

Most AI coding failures happen because:

* intent is vague
* behavior is implicit
* verification is absent
* runtime truth is missing

Your model fixes all four.

---

# Minimal Tooling Recommendations

## Keep It Simple Initially

Use only:

### Prompt files

```text
/prompts/*.md
```

### JSONL event logs

```text
/events/runtime_events.jsonl
```

### Behavioral tests

Prefer:

* pytest
* cargo test
* Playwright
* Vitest

depending on language.

---

# Best Language Choices

Since you already work heavily with Rust:

## Backend

* Rust

Excellent for:

* deterministic systems
* event pipelines
* replayability
* correctness

---

## Frontend

Keep extremely simple:

* SvelteKit

---

# Recommended Runtime Pattern

The simplest viable runtime pattern:

The Minimal Complete Loop

The truly complete minimal loop should be:

1. Intent
2. Contract
3. Event Schema and Constraints
4. AI Implementation
5. Runtime Execution
6. Event Capture
7. AI Reconciliation Review
8. Replay Verification
8. Targeted Refinement

THIS is the full loop.


Not:

```text
AI → giant autonomous system → chaos
```

---

# What NOT To Build Yet

Avoid:

* autonomous coding agents
* self-modifying systems
* auto-merging AI
* memory/vector infra
* agent swarms
* RAG pipelines
* LangGraph/CrewAI complexity
* automatic planning systems

These become useful later.

Initially they mostly create:

* unpredictability
* hallucinated abstractions
* hidden state
* debugging nightmares

---

# The Single Most Important Practice

Every feature should have:

1. Intent
2. Contract
3. Event schema
4. Behavioral test
5. Replayable event log

before implementation.

That is the essence of the architecture you described.

---

# The Simplest “DBA Starter Kit”

## Human Writes

```text
Intent
Contract
Events
```

## AI Writes

```text
Implementation
Tests
Refinements
```

## Runtime Produces

```text
Replayable truth
```

That is enough to build a very strong AI-native engineering workflow without overengineering it.


# alternative flow
# Model

Intent
  ↓
Behavior Contracts
  ↓
Verified Behavioral Modules
  ↓
Immutable Runtime Event Spine
  ↓
Observed Runtime Graph
  ↓
Replayable System Truth
  ↓
Targeted Behavioral Refinement
  ↓
Continuous Verification