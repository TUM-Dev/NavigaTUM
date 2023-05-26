<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { saveCooke } from "@/composables/cookies";
const { locale } = useI18n({
  inheritLocale: true,
  useScope: "global",
});

function setLang(lang: string) {
  locale.value = lang;
  saveCooke("lang", lang, true);
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
