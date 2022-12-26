export const configRelease = {
  /* --- Site configuration --- */
  // Prefix for resource loading, e.g. "/app/" if the page is
  // running at "example.com/app/".
  // Setting it to "" makes paths relative. This only works for
  // hash-based navigation in development builds.
  app_prefix: "/",
  // Prefix for 'cdn' content, e.g. images.
  cdn_prefix: "https://nav.tum.de/cdn/",
  // Prefix for API requests
  api_prefix: "https://nav.tum.de/api/",
};

export const configLocal = {
  app_prefix: "",
  cdn_prefix: "https://nav.tum.de/cdn/",
  api_prefix: "https://nav.tum.de/api/",
};
