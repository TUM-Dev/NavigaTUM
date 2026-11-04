# End-to-End Tests

E2E tests for NavigaTUM using [Playwright](https://playwright.dev/), testing the full Docker stack.

In CI, the local Compose server and webclient images are built once from the tested
revision and shared with the API and UI jobs. Each job loads those images and pulls
only the local database and Meilisearch dependencies before starting the stack.
The image artifact is retained for one day.

## Quick Start

```bash
cd tests/e2e
pnpm install
pnpm test
```

This automatically starts Docker containers, runs tests, and generates a report.

## Commands

```bash
pnpm test                  # Run all tests (API + UI, auto-starts containers)
pnpm test:api              # Run API tests only
pnpm test:ui               # Run UI tests in Chromium
pnpm test:ui:chromium      # Run UI tests in Chromium (alias)
pnpm test:ui:headed        # Run UI tests with browser visible
pnpm test:interactive      # Interactive mode with UI
pnpm test:report           # View last test report
pnpm update-screenshots    # Update all screenshot baselines
```

## Test Against Running Containers

```bash
SKIP_WEBSERVER=true pnpm test
```

## Test Against Production

```bash
SKIP_WEBSERVER=true BASE_URL=https://nav.tum.de pnpm test
```
