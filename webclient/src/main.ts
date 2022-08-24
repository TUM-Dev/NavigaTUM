import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "./App.vue";
import router from "./router";
import { createI18n } from "vue-i18n";
import en from "./locales/en.yaml";
import de from "./locales/de.yaml";

const i18n = createI18n({
  locale: "de",
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
