import { expect, test } from "@playwright/test";

test.describe("CDN Endpoints - Static Files", () => {
  test("should serve health check", async ({ request }) => {
    const response = await request.get("/cdn/health");
    expect(response.status()).toBe(200);
    const body = await response.text();
    expect(body).toBe("healthy");
  });

  test("should serve API data JSON", async ({ request }) => {
    const response = await request.get("/cdn/api_data.json");
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(Array.isArray(data)).toBe(true);
    expect(data.length).toBeGreaterThan(0);

    // Verify data structure
    const firstEntry = data[0];
    expect(firstEntry).toHaveProperty("id");
    expect(firstEntry).toHaveProperty("type");
  });

  test("should serve search data JSON", async ({ request }) => {
    const response = await request.get("/cdn/search_data.json");
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(Array.isArray(data)).toBe(true);
    expect(data.length).toBeGreaterThan(0);
  });

  test("should serve parquet files", async ({ request }) => {
    const files = ["alias_data.parquet", "status_data.parquet", "public_transport.parquet"];

    for (const file of files) {
      const response = await request.get(`/cdn/${file}`);

      // Accept both 200 (file exists) and 404 (file doesn't exist in test data)
      expect([200, 404]).toContain(response.status());

      if (response.status() === 200) {
        // Parquet files are binary, check content length
        const buffer = await response.body();
        expect(buffer.length).toBeGreaterThan(0);
      }
    }
  });

  test("should serve sitemap.xml", async ({ request }) => {
    const response = await request.get("/cdn/sitemap.xml");
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toMatch(/xml/);

    const body = await response.text();
    expect(body).toContain("<?xml");
    // Can be either a sitemapindex or urlset
    expect(body.includes("<sitemapindex") || body.includes("<urlset")).toBe(true);
  });
});

test.describe("CDN Endpoints - Images", () => {
  test("should serve large images", async ({ request }) => {
    const response = await request.get("/cdn/lg/0101_0.webp");
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/webp");

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });

  test("should serve thumbnail images", async ({ request }) => {
    const response = await request.get("/cdn/thumb/0101_0.webp");
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/webp");
  });
});

test.describe("CDN Endpoints - Caching Headers", () => {
  test("should have ETag and Last-Modified headers for static files", async ({ request }) => {
    const response = await request.get("/cdn/api_data.json");
    expect(response.status()).toBe(200);

    const headers = response.headers();
    expect(headers).toHaveProperty("etag");
    expect(headers).toHaveProperty("last-modified");
  });

  test("should support conditional requests with If-None-Match", async ({ request }) => {
    const firstResponse = await request.get("/cdn/api_data.json");
    expect(firstResponse.status()).toBe(200);

    const etag = firstResponse.headers()["etag"];
    expect(etag).toBeDefined();

    const secondResponse = await request.get("/cdn/api_data.json", {
      headers: {
        "If-None-Match": etag,
      },
    });

    expect(secondResponse.status()).toBe(304);
  });

  test("should have CORS headers for CDN endpoints", async ({ request }) => {
    const response = await request.get("/cdn/api_data.json");
    expect(response.status()).toBe(200);

    const headers = response.headers();
    expect(headers["access-control-allow-origin"]).toBe("*");
  });
});

test.describe("CDN Endpoints - Error Handling", () => {
  test("should return 404 for non-existent files", async ({ request }) => {
    const response = await request.get("/cdn/non_existent_file.json");
    expect(response.status()).toBe(404);
  });
});

test.describe("CDN Endpoints - Compression", () => {
  test("should serve files with compression support", async ({ request }) => {
    const response = await request.get("/cdn/api_data.json", {
      headers: {
        "Accept-Encoding": "gzip, deflate, br",
      },
    });

    expect(response.status()).toBe(200);

    const contentEncoding = response.headers()["content-encoding"];
    expect(contentEncoding).toBeDefined();
    expect(["gzip", "br", "deflate"]).toContain(contentEncoding);
  });
});
