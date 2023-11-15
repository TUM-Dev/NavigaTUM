<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { saveCooke } from "@/composables/cookies";

const { locale } = useI18n({ inheritLocale: true });

function setLang(lang: string) {
  locale.value = lang;
  saveCooke("lang", lang, true);
}
</script>

<template>
  <div id="setting-lang" class="btn-group btn-group-block">
    <button
      v-for="lang in ['en', 'de']"
      :key="lang"
      type="button"
      :value="lang"
      class="btn btn-sm"
      :class="{ active: locale === lang }"
      v-bind="{ disabled: locale === lang }"
      @click="setLang(lang)"
    >
      {{ lang }}
    </button>
  </div>
</template>
