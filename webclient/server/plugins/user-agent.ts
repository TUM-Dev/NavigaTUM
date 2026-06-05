// Identifies outbound server-side HTTP requests (SSR `useFetch`, Nitro routes,
// any other `$fetch` from the server). Browser-side `fetch()` cannot set
// User-Agent - it is a forbidden request header per the Fetch spec, so this
// plugin has no effect on requests issued directly from the user's browser.
export default defineNitroPlugin(() => {
  const version = process.env.GIT_COMMIT_SHA?.slice(0, 7) || "dev";
  const userAgent = `NavigaTUM-webclient/${version} (+https://nav.tum.de; mailto:navigatum@tum.de)`;
  globalThis.$fetch = globalThis.$fetch.create({
    headers: { "User-Agent": userAgent },
  });
});
