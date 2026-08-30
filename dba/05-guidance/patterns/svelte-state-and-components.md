---
component_question: How is state modelled and component communication structured in Svelte 5?
out_of_scope: Cross-feature infrastructure sharing, snippet/slot mechanics, styling, build
  configuration, and general TypeScript composition.
---

# Pattern: Svelte State and Components

Consult this advisory pattern when approved architecture selects Svelte. An Implementation Profile
governs implementation language, not framework selection. Approved behavior, applicable
architecture, and the existing project structure remain authoritative.

The rune APIs discussed below are Svelte 5 APIs; recheck them against the project's selected Svelte
version when they change. This pattern addresses state and component communication. For observable
browser testing, consult `svelte-gui-verification.md` as well.

## Source State, Derived State

Use `$state` for source data and `$derived` for values computed from it. Do not keep two pieces of
state synchronized with `$effect` — that produces duplicated state that can disagree with its
source. Reserve `$effect` for real side effects: DOM APIs, analytics, external libraries, network
synchronization, and subscriptions.

```svelte
let items = $state([]);
let visible = $derived(items.filter(matchesFilter));
```

## Clear State Ownership

Each mutable state value has one clear owner — a component, a `.svelte.ts` model, or a shared state
module, not necessarily the component itself. Ownership flows down as props; a request to change it
flows up as a callback prop, the Svelte 5 mechanism (`createEventDispatcher` is its deprecated
predecessor). Runes-mode props are not bindable by default; use `bind:` only where bidirectional
ownership is genuinely intentional, not as the default way components communicate.

## Extract Reusable or Shared Reactive Models

Move reactive logic into a `.svelte.ts` module when it is reused across components, shared as state,
or benefits from an independently testable boundary. Keep component-local behavior local when
extraction would add no useful boundary — this is a reuse/sharing/testability judgment, not a size
threshold, and most components do not need a paired model file.

## What This Pattern Does Not Do

A component correctly split along these lines can still hold the wrong state owner or leak a domain
decision into a callback. This pattern addresses structure, not correctness of what is being
modelled.

## Verification

Look for concrete, checkable structures rather than subjective ones: `$effect` used to synchronize
derived state, the same mutable value owned in more than one place, `bind:` used where bidirectional
ownership is not actually intentional, and reusable or shared reactive state trapped inside one
component's `.svelte` file. Preserve behavior with proportional tests.
