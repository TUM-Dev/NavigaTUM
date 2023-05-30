<script setup lang="ts">
import { ref, onMounted } from "vue";
import { saveCooke } from "@/composables/cookies";

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
      {{ $t("footer.theme_light") }}
    </button>
    <button
      class="btn btn-sm"
      @click="setTheme('dark')"
      :class="{ active: theme === 'dark' }"
      v-bind="{ disabled: theme === 'dark' }"
    >
      {{ $t("footer.theme_dark") }}
    </button>
  </div>
</template>
