import { Registry, collectDefaultMetrics, Counter, Histogram } from "prom-client";

export const metricsRegistry = new Registry();
collectDefaultMetrics({ register: metricsRegistry });

export const ssrRenderDuration = new Histogram({
  name: "nuxt_ssr_render_duration_seconds",
  help: "Duration of SSR page renders",
  labelNames: ["route"] as const,
  registers: [metricsRegistry],
});

export const pageRequests = new Counter({
  name: "nuxt_page_requests_total",
  help: "Total SSR page requests",
  labelNames: ["route", "status"] as const,
  registers: [metricsRegistry],
});
