import { pageRequests, ssrRenderDuration } from "../utils/metrics";

// Normalize dynamic segments to avoid high-cardinality labels.
const DYNAMIC_SEGMENT_RE = /\/(view|campus|site|building|room|poi)\/[^/]+/;
const LOCALE_PREFIX_RE = /\/en\//;

export default defineEventHandler((event) => {
  const start = performance.now();
  event.node.res.on("finish", () => {
    const route = getRequestURL(event).pathname;
    // Don't track the metrics endpoint itself
    if (route === "/metrics") return;

    const duration = (performance.now() - start) / 1000;
    const normalizedRoute = route
      .replace(DYNAMIC_SEGMENT_RE, "/$1/:id")
      .replace(LOCALE_PREFIX_RE, "/");
    ssrRenderDuration.labels(normalizedRoute).observe(duration);
    pageRequests.labels(normalizedRoute, String(event.node.res.statusCode)).inc();
  });
});
