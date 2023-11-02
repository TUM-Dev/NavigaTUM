import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "@/App.vue";
import router from "@/router";
import { createI18n } from "vue-i18n";
import * as Sentry from "@sentry/vue";

const i18n = createI18n<Record<string, never>, "de" | "en", false>({
  legacy: false,
  locale: localStorage.getItem("lang") || "de",
  messages: { de: {}, en: {} },
  globalInjection: true,
  missingWarn: true,
  warnHtmlMessage: true,
});

const app = createApp(App);

app.use(createPinia());

if (import.meta.env.PROD) {
  Sentry.init({
    app,
    dsn: "https://4e10b1156a2f4320acaac22148c8a568@glitchtip.nav.tum.sexy/2",
    integrations: [
      new Sentry.Replay(),
      new Sentry.BrowserTracing({
        routingInstrumentation: Sentry.vueRouterInstrumentation(router),
      }),
    ],
    replaysSessionSampleRate: 0,
    replaysOnErrorSampleRate: 1.0,
    tracesSampleRate: 1.0,// 1.0 =>  capturing 100% of transactions
    tracePropagationTargets: ["nav.tum.de"],
  });
}

app.use(router);
app.use(i18n);

app.mount("#app");
