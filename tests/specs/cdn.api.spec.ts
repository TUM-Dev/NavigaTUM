import { expect, test } from "@playwright/test";

test.describe("CDN Endpoints - Health Check", () => {
  test.skip("should serve health check endpoint with correct status and body", async ({ request }) => {
    const response = await request.get("/cdn/health");

    expect(response.status()).toBe(200);

    const body = await response.text();
    expect(body).toBe("healthy");
  });
});

test.describe("CDN Endpoints - JSON Data Files", () => {
  test.skip("should serve api_data.json with valid structure", async ({ request }) => {
    const response = await request.get("/cdn/api_data.json");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(Array.isArray(data)).toBe(true);
    expect(data.length).toBeGreaterThan(0);

    const firstEntry = data[0];
    expect(firstEntry).toHaveProperty("id");
    expect(firstEntry).toHaveProperty("type");
  });

  test.skip("should serve search_data.json with valid array structure", async ({ request }) => {
    const response = await request.get("/cdn/search_data.json");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(Array.isArray(data)).toBe(true);
    expect(data.length).toBeGreaterThan(0);
  });
});

test.describe("CDN Endpoints - Parquet Files", () => {
  test.skip("should serve alias_data.parquet with valid binary content", async ({ request }) => {
    const response = await request.get("/cdn/alias_data.parquet");

    expect(response.status()).toBe(200);

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });

  test.skip("should serve status_data.parquet with valid binary content", async ({ request }) => {
    const response = await request.get("/cdn/status_data.parquet");

    expect(response.status()).toBe(200);

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });

  test.skip("should serve public_transport.parquet with valid binary content", async ({ request }) => {
    const response = await request.get("/cdn/public_transport.parquet");

    expect(response.status()).toBe(200);

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });
});

test.describe("CDN Endpoints - XML Files", () => {
  test.skip("should serve sitemap.xml with valid XML structure", async ({ request }) => {
    const response = await request.get("/cdn/sitemap.xml");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toMatch(/xml/);

    const body = await response.text();
    expect(body).toContain("<?xml");
    expect(body.includes("<sitemapindex") || body.includes("<urlset")).toBe(true);
  });
});

test.describe("CDN Endpoints - Large Images", () => {
  test.skip("should serve large webp images with correct content type", async ({ request }) => {
    const response = await request.get("/cdn/lg/0101_0.webp");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/webp");

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });
});

test.describe("CDN Endpoints - Thumbnail Images", () => {
  test.skip("should serve thumbnail webp images with correct content type", async ({ request }) => {
    const response = await request.get("/cdn/thumb/0101_0.webp");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/webp");

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });
});

test.describe("CDN Endpoints - Cache Control with ETag Headers", () => {
  test.skip("should include ETag and Last-Modified headers for api_data.json", async ({ request }) => {
    const response = await request.get("/cdn/api_data.json");

    expect(response.status()).toBe(200);

    const headers = response.headers();
    expect(headers).toHaveProperty("etag");
    expect(headers).toHaveProperty("last-modified");
  });
});

test.describe("CDN Endpoints - Conditional Requests", () => {
  test.skip("should return 304 Not Modified when If-None-Match matches ETag for api_data.json", async ({
    request,
  }) => {
    const firstResponse = await request.get("/cdn/api_data.json");
    expect(firstResponse.status()).toBe(200);

    const etag = firstResponse.headers().etag;
    expect(etag).toBeDefined();

    const secondResponse = await request.get("/cdn/api_data.json", {
      headers: {
        "If-None-Match": etag,
      },
    });

    expect(secondResponse.status()).toBe(304);
  });
});

test.describe("CDN Endpoints - CORS Headers", () => {
  test.skip("should include CORS allow-origin wildcard header for api_data.json", async ({
    request,
  }) => {
    const response = await request.get("/cdn/api_data.json");

    expect(response.status()).toBe(200);

    const headers = response.headers();
    expect(headers["access-control-allow-origin"]).toBe("*");
  });
});

test.describe("CDN Endpoints - Compression Support", () => {
  test.skip("should serve api_data.json with compression when Accept-Encoding is provided", async ({
    request,
  }) => {
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

test.describe("CDN Endpoints - Error Handling", () => {
  test.skip("should return 404 for non-existent files", async ({ request }) => {
    const response = await request.get("/cdn/non_existent_file.json");

    expect(response.status()).toBe(404);
  });
});
