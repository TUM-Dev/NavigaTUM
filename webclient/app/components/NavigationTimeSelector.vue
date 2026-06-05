<script setup lang="ts">
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from "@headlessui/vue";
import { mdiCheck, mdiUnfoldMoreHorizontal } from "@mdi/js";
import { computed } from "vue";
import type { TimeSelection } from "~/types/navigation";

const timeSelection = defineModel<TimeSelection | undefined>("timeSelection");

const { t } = useI18n({ useScope: "local" });

const options = [
  { value: "now", label: computed(() => t("start_now")) },
  { value: "depart_at", label: computed(() => t("depart_at")) },
  { value: "arrive_by", label: computed(() => t("arrive_by")) },
];

const selectedMode = computed({
  get: () => {
    if (!timeSelection.value) return "now";
    return timeSelection.value.type;
  },
  set: (value: "now" | "depart_at" | "arrive_by") => {
    if (value === "now") {
      timeSelection.value = undefined;
    } else {
      // Initialize with current time if switching from "now"
      const now = new Date();
      timeSelection.value = {
        type: value,
        time: timeSelection.value?.time || now,
      };
    }
  },
});

const selectedOption = computed(() => options.find((opt) => opt.value === selectedMode.value));
</script>

<template>
  <Listbox v-model="selectedMode" as="div" class="relative">
    <ListboxButton
      class="relative w-full cursor-pointer rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-black py-1.5 pl-3 pr-10 text-left text-zinc-900 dark:text-zinc-50 shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 sm:text-sm"
    >
      <span class="block truncate">{{ selectedOption?.label }}</span>
      <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
        <MdiIcon :path="mdiUnfoldMoreHorizontal" :size="20" class="text-zinc-400 dark:text-zinc-500" aria-hidden="true" />
      </span>
    </ListboxButton>

    <transition
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <ListboxOptions
        class="absolute z-10 mt-1 max-h-60 w-auto min-w-full overflow-auto rounded-md bg-white dark:bg-black py-1 text-base shadow-lg ring-1 ring-black/5 dark:ring-white/5 focus:outline-none sm:text-sm"
      >
        <ListboxOption
          v-for="option in options"
          :key="option.value"
          :value="option.value"
          as="template"
          v-slot="{ active, selected }"
        >
          <li
            :class="[
              active ? 'bg-blue-600 dark:bg-blue-300 text-white dark:text-black' : 'text-zinc-900 dark:text-zinc-50',
              'relative cursor-default select-none py-2 pl-10 pr-4',
            ]"
          >
            <span :class="[selected ? 'font-semibold' : 'font-normal', 'block truncate whitespace-nowrap']">
              {{ option.label }}
            </span>
            <span
              v-if="selected"
              :class="[active ? 'text-white dark:text-black' : 'text-blue-600 dark:text-blue-300', 'absolute inset-y-0 left-0 flex items-center pl-3']"
            >
              <MdiIcon :path="mdiCheck" :size="20" aria-hidden="true" />
            </span>
          </li>
        </ListboxOption>
      </ListboxOptions>
    </transition>
  </Listbox>
</template>

<i18n lang="yaml">
de:
  start_now: Jetzt starten
  depart_at: Abfahrt ab
  arrive_by: Ankunft bis
en:
  start_now: Start now
  depart_at: Depart at
  arrive_by: Arrive by
</i18n>
