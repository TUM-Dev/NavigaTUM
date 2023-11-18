import { createPinia } from "pinia"; // or Vuex
import { createI18n } from "vue-i18n";
import { mount } from "cypress/vue";
import { createMemoryHistory, createRouter } from "vue-router";
import { routes } from "../../src/router";

// We recommend that you pull this out
// into a constants file that you share with
// your main.js file.
const i18nOptions = {
  legacy: false,
  locale: localStorage.getItem("lang") || "de",
  messages: { de: {}, en: {} },
  globalInjection: true,
  missingWarn: true,
  warnHtmlMessage: true,
};

Cypress.Commands.add("mount", (component, options = {}) => {
  options.global = options.global || {};
  options.global.plugins = options.global.plugins || [];
  options.global.plugins = options.global.plugins || [];
  options.global.plugins.push(createPinia());
  options.global.plugins.push(createI18n(i18nOptions));

  // create router if one is not provided
  if (!options.router) {
    options.router = createRouter({
      routes: routes,
      history: createMemoryHistory(),
    });
  }

  // Add router plugin
  options.global.plugins.push({
    install(app) {
      app.use(options.router);
    },
  });

  return mount(component, options);
});
