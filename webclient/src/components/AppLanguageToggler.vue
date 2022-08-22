<script setup lang="ts">
import { useI18n } from "vue-i18n";
const { t, locale } = useI18n({
  inheritLocale: true,
  useScope: "global",
});

function setLang(lang: string) {
  locale.value = lang;
  localStorage.setItem("lang", lang);
  const path = import.meta.env.VITE_APP_URL;
  document.cookie = `lang=${lang};Max-Age=31536000;SameSite=Lax;Path=${path}`;
}
</script>

<template>
  <div class="btn-group btn-group-block" id="setting-lang">
    <button
      v-for="lang in ['en', 'de']"
      v-bind:value="lang"
      class="btn btn-sm"
      v-bind:class="{ active: locale === lang }"
      v-bind="{ disabled: locale === lang }"
      @click="setLang(lang)"
    >
      {{ lang }}
    </button>
  </div>
</template>
