import { describe, expect, it, vi } from "vitest";
import type { operations } from "../app/api_types";
import {
  googleMapsDirectionsUrl,
  googleMapsEndpoint,
  needsNavigationFallback,
} from "../app/utils/navigationFallback";

type NavigationResponse =
  operations["route_handler"]["responses"][200]["content"]["application/json"];
const emptyMotis: NavigationResponse = { router: "motis", direct: [], itineraries: [] };

describe("needsNavigationFallback", () => {
  it("covers routing errors, including an unresolved external address", () => {
    expect(needsNavigationFallback("osm_10698474", "mi", "error")).toBe(true);
  });

  it("covers successful transit responses without any connections", () => {
    expect(needsNavigationFallback("osm_10698474", "mi", "success", emptyMotis)).toBe(true);
  });

  it.each(["idle", "pending"])("does not show a fallback while %s", (status) => {
    expect(needsNavigationFallback("osm_10698474", "mi", status, emptyMotis)).toBe(false);
  });

  it.each([
    ["", "mi"],
    ["mi", ""],
    ["", ""],
  ])("does not show a fallback for incomplete endpoints (%s, %s)", (from, to) => {
    expect(needsNavigationFallback(from, to, "error")).toBe(false);
  });

  it.each(["direct", "itineraries"] as const)("preserves %s results", (field) => {
    const data = { ...emptyMotis, [field]: [{}] } as NavigationResponse;
    expect(needsNavigationFallback("a", "b", "success", data)).toBe(false);
  });

  it("preserves successful Valhalla results", () => {
    const data = { router: "valhalla" } as NavigationResponse;
    expect(needsNavigationFallback("a", "b", "success", data)).toBe(false);
  });
});

describe("googleMapsEndpoint", () => {
  it.each(["48.26,11.67", "coord:48.26,11.67", "coord:-48.26,-11.67"])(
    "uses coordinates directly for %s",
    async (id) => {
      const load = vi.fn();
      expect(await googleMapsEndpoint(id, "My location", load)).toBe(id.replace("coord:", ""));
      expect(load).not.toHaveBeenCalled();
    }
  );

  it("resolves a campus ID to coordinates rather than sending an opaque ID to Google", async () => {
    const load = vi.fn().mockResolvedValue({ lat: 48.262, lon: 11.668 });
    expect(await googleMapsEndpoint("mi", "Mathematik/Informatik", load)).toBe("48.262,11.668");
    expect(load).toHaveBeenCalledWith("mi");
  });

  it.each([
    ["coord:0,0", "0,0"],
    ["coord:1e-7,-.5", "1e-7,-0.5"],
    ["coord:90,-180", "90,-180"],
  ])("accepts valid coordinate boundaries and numeric formats (%s)", async (id, expected) => {
    const load = vi.fn();
    expect(await googleMapsEndpoint(id, "My location", load)).toBe(expected);
    expect(load).not.toHaveBeenCalled();
  });

  it.each(["coord:91,11", "coord:48,181", "coord:NaN,11", "coord:Infinity,11", "coord:,11"])(
    "does not send invalid coordinates to Google (%s)",
    async (id) => {
      const load = vi.fn().mockRejectedValue(new Error("Not found"));
      expect(await googleMapsEndpoint(id, "", load)).toBeUndefined();
    }
  );

  it("uses the selected address when its OSM ID cannot be resolved", async () => {
    const load = vi.fn().mockRejectedValue(new Error("Not found"));
    expect(await googleMapsEndpoint("osm_10698474", "Boltzmannstraße 3, Garching", load)).toBe(
      "Boltzmannstraße 3, Garching"
    );
  });

  it("does not substitute an opaque ID when no label is available", async () => {
    const load = vi.fn().mockRejectedValue(new Error("Not found"));
    expect(await googleMapsEndpoint("osm_10698474", " ", load)).toBeUndefined();
  });
});

describe("googleMapsDirectionsUrl", () => {
  it("creates directions from the selected address to the campus coordinates", () => {
    const url = new URL(
      googleMapsDirectionsUrl(
        "Boltzmannstraße 3, Garching & München",
        "48.262,11.668",
        undefined
      ) ?? ""
    );
    expect(url.origin + url.pathname).toBe("https://www.google.com/maps/dir/");
    expect(url.searchParams.get("api")).toBe("1");
    expect(url.searchParams.get("origin")).toBe("Boltzmannstraße 3, Garching & München");
    expect(url.searchParams.get("destination")).toBe("48.262,11.668");
    expect(url.searchParams.has("travelmode")).toBe(false);
  });

  it.each([
    ["pedestrian", "walking"],
    ["bicycle", "bicycling"],
    ["car", "driving"],
    ["motorcycle", "driving"],
    ["public_transit", "transit"],
  ] as const)("maps %s to Google's %s mode", (mode, expected) => {
    const url = new URL(googleMapsDirectionsUrl("48,11", "49,12", mode) ?? "");
    expect(url.searchParams.get("travelmode")).toBe(expected);
  });

  it("lets Google choose the origin when it cannot be resolved", () => {
    const url = new URL(googleMapsDirectionsUrl(undefined, "48,11", undefined) ?? "");
    expect(url.searchParams.has("origin")).toBe(false);
    expect(url.searchParams.get("destination")).toBe("48,11");
  });

  it("does not create a directions link without a destination", () => {
    expect(googleMapsDirectionsUrl("48,11", undefined, undefined)).toBeUndefined();
  });
});
