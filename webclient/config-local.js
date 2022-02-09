var config = {
    /* --- Site configuration --- */
    // Prefix for resource loading, e.g. "/app/" if the page is
    // running at "example.com/app/".
    // Setting it to "" makes paths relative. This only works for
    // hash-based navigation in development builds.
    app_prefix: "",
    // Prefix for 'cdn' content, e.g. images.
    cdn_prefix: "/cdn/",
    // Prefix for API requests
    api_prefix: "http://localhost:8080/api/",
};
module.exports = config;
