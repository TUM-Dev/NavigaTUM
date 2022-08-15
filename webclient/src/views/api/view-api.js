function apiNavigateTo(to, from, next) {
  navigatum.beforeNavigate(to, from);
  navigatum.setTitle("${{_.view_api.title}}$");
  next();

  const head = document.getElementsByTagName("head")[0];
  // Add CSS first (required by swagger-ui)
  const elCSS = document.createElement("link");
  elCSS.rel = "stylesheet";
  elCSS.href = "/* @echo app_prefix */css/swagger-ui.min.css";
  head.appendChild(elCSS);

  // JS should trigger init on load
  const elJS = document.createElement("script");
  elJS.src = "/* @echo app_prefix */js/swagger-ui.min.js";
  elJS.onload = () => {
    window.setTimeout(() => {
      // we need to make sure, that swagger-ui exists, otherwise the following command will fail
      // therefore waiting is effective
      /* global SwaggerUIBundle */
      SwaggerUIBundle({
        url: "https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml",
        dom_id: "#swagger-ui",
        presets: [
          SwaggerUIBundle.presets.apis,
          // SwaggerUIStandalonePreset
        ],
        // layout: "StandaloneLayout",
      });
      navigatum.afterNavigate(to, from);
    }, 10);
  };
  head.appendChild(elJS);
}

navigatum.registerView("api", {
  name: "view-api",
  template: { gulp_inject: "view-api.inc" },
  data: function () {
    return {};
  },
  beforeRouteEnter: function (to, from, next) {
    apiNavigateTo(to, from, next);
  },
  beforeRouteUpdate: function (to, from, next) {
    apiNavigateTo(to, from, next);
  },
});
