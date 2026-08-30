---
component_question: How should a Svelte browser interface be verified at its user-visible boundary?
out_of_scope: General Svelte design, routing, styling, component libraries, directory layout,
  deployment topology, and defining acceptance requirements.
---

# Pattern: Svelte GUI Verification

Consult this advisory pattern when approved architecture selects Svelte for a browser interface.
The approved Charter, Specification Package, and architecture remain authoritative. This pattern
adds verification techniques; it does not add requirements or decide what evidence is sufficient
for a claim.

## Choose the Smallest Boundary That Proves the Behavior

| Behavior under test | Default boundary | Tool |
|---|---|---|
| Pure TypeScript calculation or transformation with no browser behavior | Unit | Vitest in its Node environment |
| One Svelte component's rendered states and user interactions | Browser component | Vitest Browser Mode with the Playwright provider and `vitest-browser-svelte` |
| A bounded journey through the built application and its real application boundaries | End to end | Playwright Test |
| A requirement whose meaning is genuinely visual or spatial | Visual comparison at the smallest relevant region | Playwright screenshot assertion |

Do not move pure logic into a browser test merely because a component calls it. Conversely, a Node
or simulated-DOM test is not direct acceptance evidence when the claim names behavior in a rendered
browser. Keep end-to-end coverage to consequential journeys; component tests are cheaper for state
variants and interaction edges.

The Contract's operating context decides the browser set. Chromium is a lean starting point when a
requirement says only "current desktop browser." Add Firefox, WebKit, mobile viewports, or device
profiles only when an approved requirement or observed compatibility risk calls for them.

## Observe What the User Observes

Prefer locators based on role, accessible name, label, and visible text, followed by auto-retrying
browser assertions. Use a test id only when the behavior has no suitable user-facing locator. Avoid
component instances, rune values, private callbacks, CSS structure, and long selectors as evidence
for user-visible behavior.

Match the assertion to the claim:

```text
currency is present in a model     != currency is visible in the plot summary
criterion appears in a request     != visible matched plots changed
map-library state contains a plot  != the rendered map shows the required result
request completed                  != list and map reached the required settled state
```

For a required field, assert its rendered label and value within the relevant visible item. For a
filter or search, record the visible result identities before the user action, perform the action,
and assert the expected identities afterward. When list and map must update together, observe both:
assert the list identities and the map's browser-visible output. Library state alone is a proxy for
the latter. A screenshot is appropriate when the map requirement is visual or spatial; a
project-owned accessible representation is appropriate only when the Contract makes it part of the
observable surface.

## Exercise Visible States Deliberately

Cover the states the approved behavior distinguishes, such as loading, failure, empty, unknown, and
successful results. Assert the state through what is rendered, including the action or recovery the
user can take when one is required. Do not treat an absent result as proof of an explicit empty or
unknown state.

Use controlled inputs at the component boundary for cheap state coverage. Use an end-to-end test
only when the state depends on integration behavior that the isolated component cannot prove.

## Prove the Workload Before Timing It

Exploratory timing needs no ceremony. Before browser timing is accepted for a governed performance
requirement, first assert that the measured user action exercises the governed behavior. For a
filter, prove that the criterion change produces the expected changed list and map output, then time
that same action through the required settled output. Record the approved workload, environment,
sample count, and statistic with the measurement.

Do not use a completed request, changed query string, or animation frame as a substitute for the
stronger observable result named by the requirement.

## Minimal Current Setup

For a SvelteKit project, the official Svelte CLI can add the split without Codeos owning a project
scaffold:

```bash
npx sv add 'vitest=usages:unit,component' playwright --install npm
npx playwright install chromium
```

The generated Vitest configuration keeps ordinary TypeScript tests in Node and runs Svelte
component tests in Chromium Browser Mode with the Playwright provider. The generated Playwright
configuration builds and previews the application for end-to-end tests. Adapt the package manager
and existing configuration rather than replacing project decisions.

For a Vite/Svelte project that does not use SvelteKit, use the same underlying packages:
`vitest`, `@vitest/browser-playwright`, `vitest-browser-svelte`, and `@playwright/test`. Configure
the Svelte Vite plugin, a Browser Mode project with `provider: playwright()` and a Chromium
instance, and a separate Node project for pure logic. Always `await render(...)` before querying a
component rendered by `vitest-browser-svelte`.

Example component interaction:

```ts
import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import PlotSummary from './PlotSummary.svelte';

test('shows the currency at the decision point', async () => {
  const screen = await render(PlotSummary, { value: '120,000', currency: 'EUR' });

  await expect.element(screen.getByText('EUR')).toBeVisible();
});
```

## Sources and Revalidation

Setup and APIs were verified on 2026-08-30 against:

- Svelte testing and the official `sv` Vitest and Playwright add-ons:
  <https://svelte.dev/docs/svelte/testing>, <https://svelte.dev/docs/cli/vitest>, and
  <https://svelte.dev/docs/cli/playwright>
- Vitest Browser Mode, its Playwright provider, and the Svelte renderer:
  <https://vitest.dev/guide/browser/>, <https://vitest.dev/config/browser/playwright>, and
  <https://vitest.dev/api/browser/svelte>
- Playwright user-facing locators and retrying assertions:
  <https://playwright.dev/docs/locators> and <https://playwright.dev/docs/test-assertions>

Revalidate the real setup after a material revision to this recipe or a dependency change that
makes the documented interface doubtful. Routine Codeos verification checks this guidance and its
workflow links; it does not install npm packages or download browsers.
