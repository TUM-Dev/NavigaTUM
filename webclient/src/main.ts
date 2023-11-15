import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "@/App.vue";
import { router } from "@/router";
import { createI18n } from "vue-i18n";

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
app.use(router);
app.use(i18n);

app.mount("#app");
