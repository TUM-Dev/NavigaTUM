<script setup lang="ts">
import { useI18n } from "vue-i18n";
const { locale } = useI18n({
  inheritLocale: true,
  useScope: "global",
});

function setLang(lang: string) {
  locale.value = lang;
  localStorage.setItem("lang", lang);
  const domain = import.meta.env.VITE_APP_URL;
  alert("domain:" + domain);
  document.cookie = `lang=${lang};Max-Age=31536000;SameSite=Lax;domain=${domain}`;
  alert("cookie:" + document.cookie);
  window.location.reload();
}
</script>

<template>
  <div class="btn-group btn-group-block" id="setting-lang">
    <button
      v-for="lang in ['en', 'de']"
      :key="lang"
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
