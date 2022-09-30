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
app.use(router);
app.use(i18n);

app.mount("#app");
