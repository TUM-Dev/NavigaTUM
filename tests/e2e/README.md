# End-to-End Tests

E2E tests for NavigaTUM using [Playwright](https://playwright.dev/), testing the full Docker stack.

## Quick Start

```bash
cd tests/e2e
pnpm install
pnpm test
```

This automatically starts Docker containers, runs tests, and generates a report.

## Commands

```bash
pnpm test              # Run all tests (API + UI, auto-starts containers)
pnpm test:api          # Run API tests only
pnpm test:ui           # Run UI tests in Chromium
pnpm test:ui:chromium  # Run UI tests in Chromium (alias)
pnpm test:ui:headed    # Run UI tests with browser visible
pnpm test:interactive  # Interactive mode with UI
pnpm test:report       # View last test report
```

## Test Against Running Containers

```bash
SKIP_WEBSERVER=true pnpm test
```

## Test Against Production

```bash
SKIP_WEBSERVER=true BASE_URL=https://nav.tum.de pnpm test
```
