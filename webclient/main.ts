import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "./App.vue";
import { router } from "./router";
import { createI18n } from "vue-i18n";

type UserLocale = "de" | "en";
function defaultLocale(): UserLocale {
  const lang = localStorage.getItem("lang");
  if (lang && ["de", "en"].includes(lang)) return lang as UserLocale;
  const locales = [...navigator.languages, "de"];
  const relevantLocales = locales.filter((l) => ["de", "en"].includes(l));
  return relevantLocales[0] as UserLocale;
}
const i18n = createI18n<Record<string, never>, UserLocale, false>({
  legacy: false,
  locale: defaultLocale(),
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
