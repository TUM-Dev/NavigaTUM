<script setup lang="ts">
import { Menu, MenuButton, MenuItem, MenuItems, Switch } from "@headlessui/vue";
import { AdjustmentsHorizontalIcon, MoonIcon, SunIcon } from "@heroicons/vue/24/outline";
import SelectionSwitch from "@/components/SelectionSwitch.vue";

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
const { locale, availableLocales } = useI18n({ inheritLocale: true });

function setLang(lang: string) {
  locale.value = lang;
  saveCooke("lang", lang, true);
}
</script>

<template>
  <Menu as="div" class="relative ml-4 flex-shrink-0 my-auto">
    <div>
      <MenuButton
        class="relative flex rounded-full bg-white text-sm ring-2 ring-white ring-opacity-20 focus:outline-none focus:ring-opacity-100"
      >
        <span class="absolute -inset-1.5" />
        <span class="sr-only">Open preferences menu</span>
        <AdjustmentsHorizontalIcon class="w-7 h-7" />
      </MenuButton>
    </div>
    <Transition
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <MenuItems
        class="absolute -right-2 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none"
      >
        <MenuItem as="div" class="block px-4 py-2 text-xs font-semibold text-slate-500">
          <SelectionSwitch v-model:selected="theme" label="Theme" :values="['dark', 'light']">
            <template #option1><MoonIcon class="h-3 w-3" /></template>
            <template #option2><SunIcon class="h-3 w-3" /></template>
          </SelectionSwitch>
          <SelectionSwitch v-model:selected="locale" label="Language" :values="['de', 'en']">
            <template #option1>de</template>
            <template #option2>en</template>
          </SelectionSwitch>
        </MenuItem>
      </MenuItems>
    </Transition>
  </Menu>
</template>

<i18n lang="yaml">
de:
  preferences: Präferenzen
en:
  preferences: Preferences
</i18n>
