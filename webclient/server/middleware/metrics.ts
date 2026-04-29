import { ssrRenderDuration, pageRequests } from "../utils/metrics";

export default defineEventHandler((event) => {
  const start = performance.now();
  event.node.res.on("finish", () => {
    const route = getRequestURL(event).pathname;
    // Don't track the metrics endpoint itself
    if (route === "/metrics") return;

    const duration = (performance.now() - start) / 1000;
    // Normalize dynamic segments to avoid high-cardinality labels
    const normalizedRoute = route
      .replace(/\/(view|campus|site|building|room|poi)\/[^/]+/, "/$1/:id")
      .replace(/\/en\//, "/");
    ssrRenderDuration.labels(normalizedRoute).observe(duration);
    pageRequests.labels(normalizedRoute, String(event.node.res.statusCode)).inc();
  });
});
