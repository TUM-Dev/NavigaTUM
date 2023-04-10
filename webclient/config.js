export const configRelease = {
  /* --- Site configuration --- */
  // Prefix for resource loading, e.g. "/app/" if the page is
  // running at "example.com/app/".
  // Setting it to "" makes paths relative. This only works for
  // hash-based navigation in development builds.
  app_prefix: "/",
  // Prefix for 'cdn' content, e.g. images.
  // can be changed to "https://nav.tum.de/cdn" if testing against production is desired.
  // not configured by default to make staging easier
  cdn_prefix: "/cdn/",
  // Prefix for API requests
  // can be changed to "https://nav.tum.de/api" if testing against production is desired.
  // not configured by default to make staging easier
  api_prefix: "/api/",
};

export const configLocal = {
  app_prefix: "",
  cdn_prefix: "/cdn/",
  api_prefix: "http://localhost:8080/api/",
};
