import { defineConfig, devices } from "@playwright/test";

/* One-time, dismissable promo popups (e.g. the hiring popup) auto-open on a visitor's first page
   and overlay the app with a full-screen backdrop. Pre-seed their dismissal so the UI suite tests
   the app, not the promo. The value mirrors what the client writes via `useCookie` (URI-encoded
   JSON); the domain is host-only ("localhost") so it matches both the :3003 and :3000 CI targets. */
const dismissedNotices = {
  cookies: [
    {
      name: "dismissedNotices",
      value: encodeURIComponent(JSON.stringify(["hiring-werkstudent-2026"])),
      domain: "localhost",
      path: "/",
      expires: -1,
      httpOnly: false,
      secure: false,
      sameSite: "Lax" as const,
    },
  ],
  origins: [],
};

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./specs",

  /* Run tests in files in parallel */
  fullyParallel: true,

  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,

  /* Retry on CI only */
  retries: process.env.CI ? 6 : 0,
  workers: process.env.CI ? 2 : undefined,

  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [["html"], ["list"]],

  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: process.env.BASE_URL || "http://localhost:3003",

    /* German is the default Nuxt locale; visiting `/` with an English browser
       trips detectBrowserLanguage's `redirectOn: "root"` and bounces to `/en`.
       Pin the browser locale so tests get the German UI they assert on. */
    locale: "de-DE",

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: "on-first-retry",

    /* Screenshot on failure */
    screenshot: "only-on-failure",
  },

  /* Expect configuration */
  expect: {
    timeout: 10 * 1000,

    toHaveScreenshot: {
      stylePath: "./screenshot.css",
    },
  },

  /* Configure projects for different test types */
  projects: [
    {
      name: "api-tests",
      testMatch: /.*\.api\.spec\.ts/,
      use: {
        // API tests don't need a browser context
      },
    },
    {
      name: "ui-tests-chromium",
      testMatch: /.*\.ui\.spec\.ts/,
      use: {
        ...devices["Desktop Chrome"],
        storageState: dismissedNotices,
      },
    },
    {
      // Reproduces Safari/iOS focus races that Chromium can't (#3324); not yet in CI.
      name: "ui-tests-webkit",
      testMatch: /.*\.ui\.spec\.ts/,
      use: {
        ...devices["Desktop Safari"],
        storageState: dismissedNotices,
      },
    },
  ],

  /* Run your local dev server before starting the tests */
  webServer: process.env.SKIP_WEBSERVER
    ? undefined
    : {
        command: "docker compose -f ../compose.local.yml up",
        url: "http://localhost:3003/api/status",
        reuseExistingServer: !process.env.CI,
        timeout: 120 * 1000, // 2 minutes to start all containers
        stdout: "pipe",
        stderr: "pipe",
      },

  /* Global timeout for each test */
  timeout: 60 * 1000,
});
