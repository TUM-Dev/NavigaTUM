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
pnpm test           # Run all tests (auto-starts containers)
pnpm test:ui        # Interactive mode with UI
pnpm test:report    # View last test report
```

## Test Against Running Containers

```bash
SKIP_WEBSERVER=true pnpm test
```

## Test Against Production

```bash
SKIP_WEBSERVER=true BASE_URL=https://nav.tum.de pnpm test
```

## What's Tested

- `/cdn` static file serving (images, maps, sitemaps)
- Data files (JSON, Parquet)
- Caching headers (ETag, Last-Modified)
- CORS configuration
- Error handling

## Writing Tests

Add `*.api.spec.ts` files in `specs/`:

```typescript
import { expect, test } from "@playwright/test";

test("should work", async ({ request }) => {
  const response = await request.get("/api/endpoint");
  expect(response.status()).toBe(200);
});
```
