import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "./App.vue";
import router from "./router";
import { createI18n } from "vue-i18n";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import de from "./locales/de.yaml";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import en from "./locales/en.yaml";
import * as Sentry from "@sentry/vue";
import { BrowserTracing } from "@sentry/tracing";

const i18n = createI18n({
  locale: localStorage.getItem("lang") || "de",
  fallbackLocale: "en",
  messages: { en, de },
  legacy: false,
  missingWarning: true,
  include: "yaml",
});

const app = createApp(App);

app.use(createPinia());

if (import.meta.env.PROD) {
  Sentry.init({
    app,
    dsn: "https://e7192dffa92c4f4cbfb8cf8967c83583@sentry.mm.rbg.tum.de/6",
    integrations: [
      new BrowserTracing({
        routingInstrumentation: Sentry.vueRouterInstrumentation(router),
        tracePropagationTargets: ["nav.tum.de"],
      }),
    ],
    // Set tracesSampleRate to 1.0 to capture 100%
    // of transactions for performance monitoring.
    // We recommend adjusting this value in production
    tracesSampleRate: 1.0,
  });
}

app.use(router);
app.use(i18n);

app.mount("#app");
