<script setup lang="ts">
import { ref, onMounted } from "vue";

type UserTheme = "light" | "dark";

function setTheme(newTheme: UserTheme) {
  localStorage.setItem("theme", newTheme);
  theme.value = newTheme;
  document.documentElement.className = newTheme;
  const path = import.meta.env.VITE_APP_URL;
  document.cookie = `theme=${theme.value};Max-Age=31536000;SameSite=Lax;Path=${path}`;
}

function getTheme(): UserTheme {
  return localStorage.getItem("theme") as UserTheme;
}

function getMediaPreference(): UserTheme {
  const hasDarkPreference = window.matchMedia(
    "(prefers-color-scheme: dark)"
  ).matches;
  return hasDarkPreference ? "dark" : "light";
}
const theme = ref<UserTheme>(getTheme() || getMediaPreference());

onMounted(() => setTheme(theme.value));
</script>

<template>
  <div class="btn-group btn-group-block" id="setting-theme">
    <button
      class="btn btn-sm"
      @click="setTheme('light')"
      v-bind:class="{ active: theme === 'light' }"
      v-bind="{ disabled: theme === 'light' }"
    >
      {{ $t("footer.theme_light") }}
    </button>
    <button
      class="btn btn-sm"
      @click="setTheme('dark')"
      v-bind:class="{ active: theme === 'dark' }"
      v-bind="{ disabled: theme === 'dark' }"
    >
      {{ $t("footer.theme_dark") }}
    </button>
  </div>
</template>
