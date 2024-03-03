<script setup lang="ts">
import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/vue";
import { AdjustmentsHorizontalIcon, MoonIcon, SunIcon } from "@heroicons/vue/24/outline";
import SelectionSwitch from "@/components/SelectionSwitch.vue";

import { ref, watchEffect } from "vue";

import { saveCooke } from "@/composables/cookies";
import { useI18n } from "vue-i18n";
type UserTheme = "light" | "dark";

const theme = ref<UserTheme>(initialUserTheme());
watchEffect(() => {
  console.log();
  document.documentElement.className = theme.value;
  saveCooke("theme", theme.value);
});

function initialUserTheme(): UserTheme {
  const storedPreference = localStorage.getItem("theme") as UserTheme;
  if (storedPreference) return storedPreference;

  const hasDarkPreference = window.matchMedia("(prefers-color-scheme: dark)").matches;
  return hasDarkPreference ? "dark" : "light";
}
const { locale } = useI18n({ useScope: "global" });
const { t } = useI18n({ useScope: "local" });
watchEffect(() => saveCooke("lang", locale.value));
</script>

<template>
  <Menu as="div">
    <div>
      <MenuButton
        class="focusable relative flex rounded-full bg-transparent p-2 text-sm ring-2 ring-white ring-opacity-0 hover:bg-slate-100/10 hover:ring-opacity-20 focus:outline-none focus:ring-opacity-100"
      >
        <span class="absolute -inset-1.5" />
        <span class="sr-only">Open preferences menu</span>
        <AdjustmentsHorizontalIcon class="text-black h-6 w-6" />
      </MenuButton>
    </div>
    <Transition
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <MenuItems
        class="bg-white absolute -right-1 top-20 z-10 w-48 origin-top-right rounded-md py-5 shadow-lg ring-1 ring-black ring-opacity-5 dark:bg-zinc-100 focus:outline-none"
      >
        <MenuItem as="div" class="text-zinc-400 block px-4 pb-2 text-xs font-semibold">
          {{ t("preferences") }}
        </MenuItem>
        <MenuItem as="div" class="text-md text-zinc-500 block px-4 py-1 font-semibold">
          <SelectionSwitch v-model="theme" label="Theme" :values="['dark', 'light']">
            <template #option1><MoonIcon class="h-3.5 w-3.5" /></template>
            <template #option2><SunIcon class="h-3.5 w-3.5" /></template>
          </SelectionSwitch>
        </MenuItem>
        <MenuItem as="div" class="text-md text-zinc-500 block px-4 py-1 font-semibold">
          <SelectionSwitch v-model="locale" :label="t('language')" :values="['de', 'en']">
            <template #option1><span class="text-xs">de</span></template>
            <template #option2><span class="text-xs">en</span></template>
          </SelectionSwitch>
        </MenuItem>
      </MenuItems>
    </Transition>
  </Menu>
</template>

<i18n lang="yaml">
de:
  preferences: Präferenzen
  language: Sprache
en:
  preferences: Preferences
  language: Language
</i18n>
