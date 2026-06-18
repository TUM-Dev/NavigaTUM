<script setup lang="ts">
import { Popover, PopoverButton, PopoverPanel } from "@headlessui/vue";
import { mdiCheck, mdiChevronDown, mdiCog, mdiFoodApple } from "@mdi/js";
import { MENSA_PRICE_ROLES } from "~/utils/mensaMenu";

const { t } = useI18n({ useScope: "local" });
// Shared via `useLocalStorage`, so picking a role here also updates the prices on every dish and
// the matching control in the settings popup, without this component owning the state.
const { priceRole, allergenWarnings } = useMensaPreferences();
const { open: openPreferences } = usePreferencesPopup();
</script>

<template>
  <Popover class="relative">
    <PopoverButton
      class="focusable text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 inline-flex items-center gap-0.5 rounded text-xs font-normal"
      :aria-label="t('price_for', { role: t(`role.${priceRole}`) })"
    >
      <span>{{ t(`role.${priceRole}`) }}</span>
      <MdiIcon :path="mdiChevronDown" :size="14" class="shrink-0" aria-hidden="true" />
    </PopoverButton>
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-1"
    >
      <PopoverPanel
        v-slot="{ close }"
        class="ring-black/5 dark:ring-white/5 absolute right-0 z-20 mt-1 w-44 rounded-sm bg-white p-1 shadow-lg ring-1 dark:bg-zinc-800"
      >
        <div class="text-zinc-500 dark:text-zinc-400 px-2 py-1 text-xs font-semibold">
          {{ t("price_for_label") }}
        </div>
        <button
          v-for="role in MENSA_PRICE_ROLES"
          :key="role"
          type="button"
          class="focusable flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-start text-sm hover:bg-zinc-100 dark:hover:bg-zinc-700"
          :class="
            priceRole === role
              ? 'text-blue-800 dark:text-blue-100'
              : 'text-zinc-800 dark:text-zinc-100'
          "
          @click="
            priceRole = role;
            close();
          "
        >
          <span class="flex-grow">{{ t(`role.${role}`) }}</span>
          <MdiIcon
            v-if="priceRole === role"
            :path="mdiCheck"
            :size="16"
            class="text-blue-600 dark:text-blue-300"
          />
        </button>
        <div class="border-zinc-200 dark:border-zinc-700 my-1 border-t" />
        <button
          type="button"
          class="focusable flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-start text-sm text-zinc-800 hover:bg-zinc-100 dark:text-zinc-100 dark:hover:bg-zinc-700"
          @click="
            openPreferences('allergens');
            close();
          "
        >
          <MdiIcon
            :path="mdiFoodApple"
            :size="16"
            class="text-zinc-500 dark:text-zinc-400 shrink-0"
            aria-hidden="true"
          />
          <span class="flex-grow">{{ t("allergy_warnings") }}</span>
          <span
            v-if="allergenWarnings.length"
            class="bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200 rounded-full px-1.5 text-xs font-semibold tabular-nums"
          >
            {{ allergenWarnings.length }}
          </span>
          <MdiIcon
            v-else
            :path="mdiCog"
            :size="15"
            class="text-zinc-400 dark:text-zinc-500 shrink-0"
            aria-hidden="true"
          />
        </button>
      </PopoverPanel>
    </Transition>
  </Popover>
</template>

<i18n lang="yaml">
de:
  price_for: "Preise für {role}"
  price_for_label: "Preise für"
  allergy_warnings: Allergiewarnungen …
  role:
    students: Studierende
    staff: Bedienstete
    guests: Gäste
en:
  price_for: "Prices for {role}"
  price_for_label: "Prices for"
  allergy_warnings: Allergy warnings …
  role:
    students: Students
    staff: Staff
    guests: Guests
</i18n>
