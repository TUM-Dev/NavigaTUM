<script setup lang="ts">
import { onMounted, ref } from "vue";
import { saveCooke } from "@/composables/cookies";
import { useI18n } from "vue-i18n";

type UserTheme = "light" | "dark";

const theme = ref<UserTheme>(getTheme() || getMediaPreference());
const { t } = useI18n({ useScope: "local" });
function setTheme(newTheme: UserTheme, reload = true) {
  theme.value = newTheme;
  document.documentElement.className = newTheme;
  saveCooke("theme", newTheme, reload);
}

function getTheme(): UserTheme {
  return localStorage.getItem("theme") as UserTheme;
}

function getMediaPreference(): UserTheme {
  const hasDarkPreference = window.matchMedia("(prefers-color-scheme: dark)").matches;
  return hasDarkPreference ? "dark" : "light";
}
onMounted(() => setTheme(theme.value, false));
</script>

<template>
  <div id="setting-theme" class="btn-group btn-group-block">
    <button
      type="button"
      class="btn btn-sm"
      :class="{ active: theme === 'light' }"
      v-bind="{ disabled: theme === 'light' }"
      @click="setTheme('light')"
    >
      {{ t("light") }}
    </button>
    <button
      type="button"
      class="btn btn-sm"
      :class="{ active: theme === 'dark' }"
      v-bind="{ disabled: theme === 'dark' }"
      @click="setTheme('dark')"
    >
      {{ t("dark") }}
    </button>
  </div>
</template>

<i18n lang="yaml">
de:
  dark: dunkel
  light: hell
en:
  dark: dark
  light: light
</i18n>
