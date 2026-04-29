import { metricsRegistry } from "../utils/metrics";

export default defineEventHandler(async () => {
  return new Response(await metricsRegistry.metrics(), {
    headers: { "Content-Type": metricsRegistry.contentType },
  });
});
