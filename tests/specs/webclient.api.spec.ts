import { expect, test } from "@playwright/test";

const WEBCLIENT_BASE_URL = process.env.WEBCLIENT_BASE_URL || "http://localhost:3000";

test.describe("Webclient - Prometheus Metrics", () => {
  test("should expose prometheus metrics at /metrics", async ({ request }) => {
    const response = await request.get(`${WEBCLIENT_BASE_URL}/metrics`);

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/plain");

    const body = await response.text();

    // Extract only HELP and TYPE lines for the snapshot - these are stable
    // across environments, while metric values, active handles, and resource
    // types vary between dev mode, production, and different Node.js versions.
    const structuralLines = `${body
      .split("\n")
      .filter((line) => line.startsWith("# HELP") || line.startsWith("# TYPE"))
      .join("\n")}\n`;

    expect(structuralLines).toMatchSnapshot("prometheus-metrics.txt");
  });

  test("should include default nodejs metrics", async ({ request }) => {
    const response = await request.get(`${WEBCLIENT_BASE_URL}/metrics`);
    const body = await response.text();

    expect(body).toContain("navigatum_ssr_process_cpu_user_seconds_total");
    expect(body).toContain("navigatum_ssr_process_resident_memory_bytes");
    expect(body).toContain("navigatum_ssr_nodejs_heap_size_total_bytes");
  });

  test("should include custom SSR metrics", async ({ request }) => {
    const response = await request.get(`${WEBCLIENT_BASE_URL}/metrics`);
    const body = await response.text();

    expect(body).toContain("navigatum_ssr_render_duration_seconds");
    expect(body).toContain("navigatum_ssr_page_requests_total");
  });
});
