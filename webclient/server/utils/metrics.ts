import { Counter, collectDefaultMetrics, Histogram, Registry } from "prom-client";

// Shared prefix so every metric this server emits - including the default
// nodejs_*/process_* ones - lives under one namespace instead of a mix of
// nuxt_*, nodejs_*, and process_*.
const METRIC_PREFIX = "navigatum_ssr_";

export const metricsRegistry = new Registry();
collectDefaultMetrics({ register: metricsRegistry, prefix: METRIC_PREFIX });

export const ssrRenderDuration = new Histogram({
  name: `${METRIC_PREFIX}render_duration_seconds`,
  help: "Duration of SSR page renders",
  labelNames: ["route"] as const,
  registers: [metricsRegistry],
});

export const pageRequests = new Counter({
  name: `${METRIC_PREFIX}page_requests_total`,
  help: "Total SSR page requests",
  labelNames: ["route", "status"] as const,
  registers: [metricsRegistry],
});
