import { defineConfig, devices } from "@playwright/test";

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
