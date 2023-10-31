<script setup lang="ts">
import { onMounted, ref } from "vue";
import { saveCooke } from "@/composables/cookies";
import { useI18n } from "vue-i18n";

type UserTheme = "light" | "dark";

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
const theme = ref<UserTheme>(getTheme() || getMediaPreference());
const { t } = useI18n({ useScope: "local" });
onMounted(() => setTheme(theme.value, false));
</script>

<template>
  <div class="btn-group btn-group-block" id="setting-theme">
    <button
      class="btn btn-sm"
      @click="setTheme('light')"
      :class="{ active: theme === 'light' }"
      v-bind="{ disabled: theme === 'light' }"
    >
      {{ t("light") }}
    </button>
    <button
      class="btn btn-sm"
      @click="setTheme('dark')"
      :class="{ active: theme === 'dark' }"
      v-bind="{ disabled: theme === 'dark' }"
    >
      {{ t("dark") }}
    </button>
  </div>
</template>

<i18n lang="yaml">
de:
  dark: Dunkel
  light: Hell
en:
  dark: Dark
  light: Light
</i18n>
