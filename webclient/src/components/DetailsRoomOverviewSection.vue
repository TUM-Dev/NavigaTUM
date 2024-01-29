<script setup lang="ts">
import { computed, ref } from "vue";
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from "@headlessui/vue";
import { CheckIcon, ChevronUpDownIcon, MapPinIcon, FunnelIcon, MagnifyingGlassIcon } from "@heroicons/vue/24/outline";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
import Btn from "@/components/Btn.vue";

type RoomsOverview = components["schemas"]["RoomsOverview"];
type ChildEntry = components["schemas"]["ChildEntry"];

const props = defineProps<{
  readonly rooms?: RoomsOverview;
}>();

const { t } = useI18n({ useScope: "local" });
const selectedUsage = ref(-1);
const search = ref("");
const combined_list = computed(() => {
  const usages = props.rooms?.usages || [];
  const combinedList = [] as ChildEntry[];
  usages.forEach((usage) => {
    combinedList.push(...usage.children);
  });
  return combinedList;
});
type SelectedRoomGroup = {
  rooms: readonly ChildEntry[];
  label: string;
};
const selectedRooms = computed<SelectedRoomGroup>(() => {
  if (selectedUsage.value === -1) {
    return { rooms: combined_list.value, label: t("any_usage") };
  }
  const rooms_usgage = props.rooms?.usages || [];
  return { rooms: rooms_usgage[selectedUsage.value].children, label: rooms_usgage[selectedUsage.value].name };
});
const filteredList = computed<readonly ChildEntry[]>(() => {
  const search_term = new RegExp(`.*${search.value}.*`, "i"); // i=>case insensitive
  return selectedRooms.value.rooms.filter((f) => search_term.test(f.name));
});
</script>

<template>
  <div v-if="props.rooms?.usages" class="flex flex-col gap-1 rounded border p-4">
    <p class="text-lg font-semibold">{{ t("title") }}</p>
    <div class="flex flex-col gap-3">
      <Listbox v-model="selectedUsage" as="div" class="relative z-10">
        <ListboxButton
          class="focusable relative w-full rounded-sm border bg-white py-2 pr-10 text-left dark:border-gray-200 sm:text-sm"
        >
          <span class="absolute inset-y-0 left-0 flex items-center pl-2">
            <FunnelIcon class="h-4 w-4 text-zinc-400" aria-hidden="true" />
          </span>
          <span class="block truncate ps-8">{{ selectedRooms.label }}</span>
          <span class="absolute inset-y-0 right-0 flex items-center pr-2">
            <ChevronUpDownIcon class="h-5 w-5 text-zinc-400" aria-hidden="true" />
          </span>
        </ListboxButton>

        <Transition
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <ListboxOptions
            class="absolute !m-0 mt-1 max-h-60 w-full overflow-auto rounded-md bg-white text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
          >
            <ListboxOption v-slot="{ active, selected }" :key="-1" :value="-1" as="template">
              <li
                class="flex cursor-pointer select-none list-none flex-row justify-between py-2 pl-10 pr-4"
                :class="[active ? 'bg-tumBlue-100 text-tumBlue-900' : 'text-zinc-900']"
              >
                <span class="block truncate" :class="[selected ? 'font-medium' : 'font-normal']"
                  >{{ t("any_usage") }}
                </span>
                <span class="rounded-md bg-tumBlue-300 px-2 py-1 text-sm text-tumBlue-950"
                  >{{ t("rooms", combined_list.length) }}
                </span>

                <span v-if="selected" class="absolute inset-y-0 left-0 flex items-center pl-3 text-tumBlue-600">
                  <CheckIcon class="h-5 w-5" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
            <ListboxOption
              v-for="(usage, i) in props.rooms.usages"
              v-slot="{ active, selected }"
              :key="i"
              :value="i"
              as="template"
            >
              <li
                class="flex cursor-pointer select-none list-none flex-row justify-between py-2 pl-10 pr-4"
                :class="[active ? 'bg-tumBlue-100 text-tumBlue-900' : 'text-zinc-900']"
              >
                <span class="my-auto block truncate" :class="[selected ? 'font-medium' : 'font-normal']">
                  {{ usage.name }}
                </span>
                <span class="rounded-md bg-tumBlue-300 px-2 py-1 text-sm text-tumBlue-950"
                  >{{ t("rooms", usage.count) }}
                </span>
                <span v-if="selected" class="absolute inset-y-0 left-0 flex items-center pl-3 text-tumBlue-600">
                  <CheckIcon class="h-5 w-5" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
          </ListboxOptions>
        </Transition>
      </Listbox>
      <div class="relative z-0 w-full border dark:border-gray-200">
        <span class="absolute inset-y-0 left-0 flex items-center pl-2">
          <MagnifyingGlassIcon class="h-4 w-4 text-zinc-400" aria-hidden="true" />
        </span>
        <input
          id="search-input"
          v-model="search"
          class="focusable w-full flex-grow rounded-sm py-2 ps-8"
          :placeholder="t('search')"
        />
      </div>
    </div>
    <div>
      <ul v-if="filteredList.length > 0" class="max-h-96 list-none overflow-y-scroll pe-2.5">
        <RouterLink v-for="(room, index) in filteredList" :key="index" :to="`/view/${room.id}`" class="!no-underline">
          <li class="flex flex-row gap-2 p-1.5 px-3 hover:bg-tumBlue-500 hover:text-white">
            <MapPinIcon class="my-auto h-4 w-4" aria-hidden="true" />
            {{ room.name }}
          </li>
        </RouterLink>
      </ul>
      <div v-else class="flex flex-row items-baseline">
        {{ t("no_results_with_these_filters") }}
        <Btn
          size="sm"
          variant="link"
          @click="
            () => {
              search = '';
              selectedUsage = -1;
            }
          "
          >{{ t("clear_filter") }}
        </Btn>
      </div>
      <small>
        {{ t("results", filteredList.length) }}
      </small>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  any_usage: beliebige Nutzung
  filter_by_usage: nach Nutzung filtern
  no_results_with_these_filters: Keine Ergebnisse mit diesen Filtern gefunden.
  clear_filter: Filter löschen
  search: durchsuchen
  results: 1 Ergebnis | {count} Ergebnisse
  rooms: 1 Raum | {count} Räume
  title: Räume
en:
  any_usage: any usage
  filter_by_usage: filter by usage
  no_results_with_these_filters: No results found with these filters.
  clear_filter: Clear the filters
  search: search
  results: "{count} result | {count} results"
  title: Rooms
</i18n>
