import { expect, test } from "@playwright/test";

test.describe("API Endpoints - Health Check", () => {
  test("should return healthy status", async ({ request }) => {
    const response = await request.get("/api/status");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/plain");

    const body = await response.text();
    expect(body).toContain("healthy");
  });
});

test.describe("API Endpoints - OpenAPI Documentation", () => {
  test("should serve OpenAPI JSON specification", async ({ request }) => {
    const response = await request.get("/api/openapi.json");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const spec = await response.json();
    expect(spec).toHaveProperty("openapi");
    expect(spec).toHaveProperty("info");
    expect(spec).toHaveProperty("paths");
    expect(spec.info.title).toBe("NavigaTUM");
  });
});

test.describe("API Endpoints - Search", () => {
  test("should search for locations with valid query", async ({ request }) => {
    const response = await request.get("/api/search?q=MI");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(data).toHaveProperty("sections");
    expect(Array.isArray(data.sections)).toBe(true);
    expect(data).toHaveProperty("time_ms");
    expect(typeof data.time_ms).toBe("number");
  });

  test("should return results for complex search query", async ({ request }) => {
    const response = await request.get("/api/search?q=Informatik&limit_all=5");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data.sections).toBeDefined();
    const section = data.sections[0];
    expect(section).toHaveProperty("facet");
    expect(section).toHaveProperty("entries");
    expect(Array.isArray(section.entries)).toBe(true);
  });

  test("should handle search with highlighting", async ({ request }) => {
    const response = await request.get("/api/search?q=Mensa&pre_highlight=<b>&post_highlight=</b>");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty("sections");
  });

  test("should return 200 for empty search query", async ({ request }) => {
    const response = await request.get("/api/search?q=");

    expect(response.status()).toBe(200);
  });

  test("should filter search by type", async ({ request }) => {
    const response = await request.get("/api/search?q=MI&type=room");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty("sections");
    expect(Array.isArray(data.sections)).toBe(true);
  });

  test("should filter search by parent with in parameter", async ({ request }) => {
    const response = await request.get("/api/search?q=hs&in=garching");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty("sections");
    expect(Array.isArray(data.sections)).toBe(true);
  });

  test("should filter search by usage", async ({ request }) => {
    const response = await request.get("/api/search?q=raum&usage=wc");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty("sections");
    expect(Array.isArray(data.sections)).toBe(true);
  });

  test("should sort search by near coordinate", async ({ request }) => {
    const response = await request.get("/api/search?q=mensa&near=48.2625,11.6681");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty("sections");
    expect(Array.isArray(data.sections)).toBe(true);
  });

  test("should support multiple filter values", async ({ request }) => {
    const response = await request.get("/api/search?q=hs&in=garching&in=5304");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty("sections");
    expect(Array.isArray(data.sections)).toBe(true);
  });

  test("should support combining multiple filter types", async ({ request }) => {
    const response = await request.get("/api/search?q=raum&type=room&in=garching");

    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty("sections");
    expect(Array.isArray(data.sections)).toBe(true);
  });
});

test.describe("API Endpoints - Location Details", () => {
  test("should get location details by valid ID", async ({ request }) => {
    const response = await request.get("/api/locations/mi");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(data).toHaveProperty("id");
    expect(data).toHaveProperty("name");
    expect(data).toHaveProperty("type");
    expect(data).toHaveProperty("type_common_name");
  });

  test("should return 404 for non-existent location", async ({ request }) => {
    const response = await request.get("/api/locations/nonexistent_location_12345");

    expect(response.status()).toBe(404);
  });

  test("should support language parameter", async ({ request }) => {
    const response = await request.get("/api/locations/mi?lang=en");

    expect(response.status()).toBe(200);
  });

  test("should return location with coordinates", async ({ request }) => {
    const response = await request.get("/api/locations/mi");

    expect(response.status()).toBe(200);
    const data = await response.json();
    expect(data.coords).toHaveProperty("lat");
    expect(data.coords).toHaveProperty("lon");
    expect(typeof data.coords.lat).toBe("number");
    expect(typeof data.coords.lon).toBe("number");
  });
});

test.describe("API Endpoints - Location Preview", () => {
  test("should generate preview image for valid location", async ({ request }) => {
    const response = await request.get("/api/locations/5602/preview?format=open_graph");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/png");

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });

  test("should return 400 for invalid ID length", async ({ request }) => {
    const longId = "a".repeat(256);
    const response = await request.get(`/api/locations/${longId}/preview?format=open_graph`);

    expect(response.status()).toBe(400);
  });

  test("should return 404 for non-existent location preview", async ({ request }) => {
    const response = await request.get(
      "/api/locations/nonexistent_location_12345/preview?format=open_graph"
    );

    expect(response.status()).toBe(404);
  });
});

test.describe("API Endpoints - QR Code", () => {
  test("should generate QR code for valid location", async ({ request }) => {
    const response = await request.get("/api/locations/mi/qr-code");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/png");

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });

  test("should generate QR code for any location ID", async ({ request }) => {
    const response = await request.get("/api/locations/nonexistent_location_12345/qr-code");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/png");
  });
});

test.describe("API Endpoints - Nearby Locations", () => {
  test("should get nearby locations for valid ID", async ({ request }) => {
    const response = await request.get("/api/locations/mi/nearby");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(data).toHaveProperty("public_transport");
    expect(Array.isArray(data.public_transport)).toBe(true);
  });

  test("should return empty nearby list for non-existent location", async ({ request }) => {
    const response = await request.get("/api/locations/nonexistent_location_12345/nearby");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(data).toHaveProperty("public_transport");
    expect(Array.isArray(data.public_transport)).toBe(true);
  });

  // The MI building sits ~150m from the Garching U6 endpoint
  // "Garching, Forschungszentrum". Motis returns several platform rows for that
  // station (subway tracks expose `parentId`, bus tracks don't), and the scraper
  // has to fold them into a single row with both modes. If it regresses, this
  // station shows up multiple times and/or with only one mode — that's the
  // single most likely regression on this endpoint.
  test("folds motis platforms into one station with all served modes", async ({ request }) => {
    const response = await request.get("/api/locations/mi/nearby");
    expect(response.status()).toBe(200);

    const data = await response.json();
    const garching = data.public_transport.filter(
      (s: { name: string }) => s.name === "Garching, Forschungszentrum"
    );
    expect(garching).toHaveLength(1);
    expect(garching[0].modes).toEqual(expect.arrayContaining(["subway", "bus"]));
    expect(garching[0].distance_meters).toBeLessThan(500);
  });
});

test.describe("API Endpoints - Calendar", () => {
  test("should return 404 or 405 for GET request to calendar endpoint", async ({ request }) => {
    const response = await request.get("/api/calendar");

    expect(response.status()).toBeGreaterThanOrEqual(404);
  });

  test("should return 400 for invalid calendar request", async ({ request }) => {
    const response = await request.post("/api/calendar", {
      data: {},
    });

    expect(response.status()).toBe(400);
  });

  test("should return 400 for too many IDs in calendar request", async ({ request }) => {
    const manyIds = Array.from({ length: 200 }, (_, i) => `id_${i}`);

    const response = await request.post("/api/calendar", {
      data: {
        ids: manyIds,
        start_after: "2024-01-01T00:00:00Z",
        end_before: "2024-12-31T23:59:59Z",
      },
    });

    expect(response.status()).toBe(400);
  });
});

test.describe("API Endpoints - Feedback Token", () => {
  test("should generate feedback token", async ({ request }) => {
    const response = await request.post("/api/feedback/get_token");

    expect(response.status()).toBe(201);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(data).toHaveProperty("token");
    expect(data).toHaveProperty("created_at");
    expect(typeof data.token).toBe("string");
    expect(typeof data.created_at).toBe("number");
  });
});

test.describe("API Endpoints - Routing", () => {
  // Routing is fairly expensive, so we skip this test by default
  test.skip("should calculate pedestrian route between coordinates", async ({ request }) => {
    const response = await request.get(
      "/api/maps/route?lang=de&from=chemie-nebengebaeude&to=48.265795,11.669106&route_costing=public_transit"
    );

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const data = await response.json();
    expect(data).toHaveProperty("router");
  });

  // Routing is fairly expensive, so we skip this test by default
  test.skip("should calculate route between location IDs", async ({ request }) => {
    const response = await request.get(
      "/api/maps/route?from=mi&to=chemie&route_costing=public_transit"
    );

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");
  });

  test("should return 404 for invalid location IDs", async ({ request }) => {
    const response = await request.get(
      "/api/maps/route?from=invalid_id_123&to=invalid_id_456&route_costing=pedestrian"
    );

    expect(response.status()).toBe(404);
  });
});

test.describe("API Endpoints - Response Format", () => {
  test("should return JSON for search endpoint", async ({ request }) => {
    const response = await request.get("/api/search?q=test");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");
  });

  test("should return JSON for location details endpoint", async ({ request }) => {
    const response = await request.get("/api/locations/mi");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");
  });
});

test.describe("API Endpoints - Error Handling", () => {
  test("should return 404 for non-existent API endpoint", async ({ request }) => {
    const response = await request.get("/api/nonexistent_endpoint");

    expect(response.status()).toBe(404);
  });
});

test.describe("API Endpoints - Content Negotiation", () => {
  test("should return JSON for search with Accept header", async ({ request }) => {
    const response = await request.get("/api/search?q=MI", {
      headers: {
        Accept: "application/json",
      },
    });

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");
  });

  test("should return plain text for health check", async ({ request }) => {
    const response = await request.get("/api/status", {
      headers: {
        Accept: "text/plain",
      },
    });

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/plain");
  });
});
