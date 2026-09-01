---
component_question: How should Codeos AI agents structure human-readable output and use canonical terminology?
out_of_scope: Governed meaning, artifact syntax, approval decisions, prose scoring, and terminology ownership.
---

# Reader-Oriented LLM Output

Apply this guidance to every human-readable output produced by a Codeos-owned AI path. Canonical
terminology applies even to a one-sentence response. The progression rules below apply when the
output contains enough explanatory structure for them to be useful.

This guidance controls presentation only. The user's instruction, governed artifacts, and
artifact-specific syntax remain authoritative. Preserve technical meaning and logical precision
over stylistic variety.

## Open With the Result

For a simple response, lead with the main result, recommendation, or decision.

For a complex response with several important findings, begin with a short summary. Preview the
important findings, then explain them in the same order. Omit a summary that would only repeat the
answer.

## Use Canonical Terminology

Before producing project-specific prose, consult the applicable canonical terminology:

- `dba/05-guidance/terminology.md` owns Codeos and DBA concepts.
- `.codeos/00-project/terminology.md`, when it exists, owns project-domain concepts.

A project glossary must not silently redefine a Codeos or DBA term. Surface a conflict between the
two sources instead of guessing which meaning applies.

Use this deterministic behavior:

```text
canonical term exists
-> reuse it exactly and consistently

missing ordinary engineering term
-> use the standard term consistently; define it briefly when needed

missing recurring project-specific term
-> establish it through the existing terminology mechanism when authorized
-> otherwise present the proposed term and definition explicitly

conflicting definitions
-> surface the conflict; do not guess or create competing synonyms
```

Do not substitute synonyms merely for variety. Repeat a precise term when repetition prevents
ambiguity. Distinct concepts should keep intentionally distinct names. Do not add stylistic
vocabulary or every newly used word to a terminology glossary.

## Maintain Sentence-to-Sentence Continuity

Within explanatory prose, connect each sentence to information the reader can already identify
from the preceding context before introducing new information.

Prefer:

> The reviewer receives a frozen packet. That packet contains the evidence available for
> assessment. The assessment is then bound to the packet hash.

Avoid:

> The reviewer receives a frozen packet. Assessment integrity depends on hashing. Evidence
> selection occurs during packet construction.

The second version may be factually correct, but it makes the reader reconstruct the relationship
between sentences. Do not force grammatical repetition when it makes a sentence unnatural or less
precise. The requirement is a recognizable information connection, not an identical sentence
opening.

## Choose a Reader-Oriented Progression

Use the simplest progression that fits the explanation. Do not name the pattern in the output
unless doing so is itself useful to the reader.

Patterns may be combined when useful. For example, Preview Then Traverse may introduce three
architecture areas, Whole Before Parts may structure one area, and Known-to-New may explain a
workflow within it. Do not combine patterns mechanically.

### Stable Topic

Use several sentences with a recognizable shared topic when explaining properties of one concept:

```text
Topic -> fact A
Topic -> fact B
Topic -> fact C
```

Do not repeatedly change the subject when each sentence still describes the same thing.

### Known-to-New Progression

Use a deliberate chain when one newly introduced concept becomes the subject of the next reasoning
step, especially for workflows, causality, transformations, and logical derivations:

```text
A -> B
B -> C
C -> D
```

Begin each step from information the reader can already identify, then introduce the next concept.

### Whole Before Parts

Name the whole, relationship, or category before explaining its parts. Use this for architecture,
decomposition, responsibilities, and sibling concepts whose relationship matters.

Example:

> The reviewer protects assessment integrity through three boundaries: packet construction,
> provider isolation, and assessment recording.
>
> Packet construction freezes the evidence. Provider isolation limits access to that evidence.
> Assessment recording binds the result to the packet.

### Preview Then Traverse

For several important findings, changes, risks, decisions, or recommendations, state the important
parts first and explain them in that same order.

## Keep Paragraphs Coherent

Give each paragraph one primary topic, reasoning step, finding, decision, or purpose. Keep closely
related information together and start a new paragraph when the primary logical subject changes.
Make the relationship between adjacent paragraphs visible when a complex explanation would
otherwise require the reader to infer it.

## Preserve Semantic and Formal Structure

Reader-oriented restructuring must not:

- change governed meaning, canonical terminology, exact identifiers, names, or literals;
- weaken a condition, merge distinct requirements, or remove a necessary qualification;
- introduce an unsupported conclusion or convert uncertainty into certainty;
- convert advisory language into mandatory language;
- invent missing technical facts.

Formal artifact rules take precedence over this guidance. Do not mechanically transform source
code, schemas, event payloads, identifiers, literal values, commands, logs, generated
machine-readable data, compact tables, or formal syntax whose structure is already defined.

## Delivery Invariant

Every Codeos-owned AI entry point that produces human-readable prose must receive the canonical
reader-oriented guidance and applicable terminology. Exclusively machine-structured outputs are
exempt. Integration tests must demonstrate the applicable route.

This invariant requires actual delivery. A normal-agent instruction must direct the agent to read
and apply this file. An isolated provider must receive the exact guidance and terminology bytes in
its effective input. Terminology supplied to a reviewer or implementer is language and
interpretation context, never acceptance evidence or a substitute for requirements.
