import { createPinia } from "pinia"; // or Vuex
import { createI18n } from "vue-i18n";
import { mount } from "cypress/vue";
import { createMemoryHistory, createRouter } from "vue-router";
import type { Router } from "vue-router";
import type { Component } from "vue";
import { routes } from "../../src/router";
import type { OptionsParam } from "../../cypress";

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

Cypress.Commands.add("mount", (component: Component, options: OptionsParam = {}) => {
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
  if (options.router !== undefined) {
    options.global.plugins.push({
      install(app) {
        app.use(options.router as Router);
      },
    });
  }

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  return mount(component, options);
});
