<script setup lang="ts">
import { useI18n } from "vue-i18n";
const { t, locale } = useI18n({
  inheritLocale: true,
  useScope: "global",
});

function setLang(lang: string) {
  localStorage.setItem("lang", lang);
  document.cookie = `lang=${lang};Max-Age=31536000;SameSite=Lax;Path=${import.meta.env.VITE_APP_URL}`;
}
</script>

<template>
  <select class="btn-group btn-group-block" id="setting-lang" v-model="locale">
    <option
      v-for="lang in ['en', 'de']"
      v-bind:value="lang"
      class="btn btn-sm"
      v-bind:class="{ active: locale.value === lang }"
      @click="setLang(lang)"
    >
      {{ lang }}
    </option>
  </select>
</template>
