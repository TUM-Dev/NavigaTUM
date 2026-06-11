<script setup lang="ts">
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from "@headlessui/vue";
import { mdiCheck, mdiFilter, mdiMagnify, mdiMapMarker, mdiUnfoldMoreHorizontal } from "@mdi/js";
import { useVirtualList } from "@vueuse/core";
import type { components } from "~/api_types";
import { entityPath } from "~/utils/entityPath";

type RoomsOverviewResponse = components["schemas"]["RoomsOverviewResponse"];
type RoomsOverviewUsageChildResponse = components["schemas"]["RoomsOverviewUsageChildResponse"];

const props = defineProps<{
  readonly rooms?: RoomsOverviewResponse | null;
  readonly browseMapUrl: string;
}>();

const { t } = useI18n({ useScope: "local" });
const selectedUsage = ref(-1);
const search = ref("");
const combined_list = computed(() => {
  const usages = props.rooms?.usages || [];
  const combinedList = [] as RoomsOverviewUsageChildResponse[];
  for (const usage of usages) {
    combinedList.push(...usage.children);
  }
  return combinedList;
});
interface SelectedRoomGroup {
  rooms: readonly RoomsOverviewUsageChildResponse[];
  label: string;
}
const selectedRooms = computed<SelectedRoomGroup>(() => {
  if (selectedUsage.value === -1) {
    return { rooms: combined_list.value, label: t("any_usage") };
  }
  const rooms_usgage = props.rooms?.usages || [];
  return {
    rooms: rooms_usgage[selectedUsage.value]?.children ?? [],
    label: rooms_usgage[selectedUsage.value]?.name ?? "usage-out-of-range",
  };
});
const filteredList = computed<RoomsOverviewUsageChildResponse[]>(() => {
  const search_term = new RegExp(`.*${search.value}.*`, "i"); // i ^= case-insensitive
  return selectedRooms.value.rooms.filter((f) => search_term.test(f.name));
});
const { list, containerProps, wrapperProps } = useVirtualList<RoomsOverviewUsageChildResponse>(
  filteredList,
  {
    itemHeight: 36,
  }
);
</script>

<template>
  <div
    v-if="props.rooms?.usages"
    class="flex flex-col gap-3 print:!hidden"
  >
    <div class="flex flex-row items-baseline justify-between gap-2">
      <p class="text-zinc-800 dark:text-zinc-100 text-lg font-semibold">{{ t("title") }}</p>
      <MapBridgeLink :to="props.browseMapUrl">{{ t("view_on_map") }}</MapBridgeLink>
    </div>
    <div class="flex flex-col gap-2">
      <Listbox v-model="selectedUsage" as="div" class="relative z-10">
        <ListboxButton
          class="focusable text-zinc-600 dark:text-zinc-300 bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 relative w-full rounded-sm border py-2 pr-10 text-left sm:text-sm"
        >
          <span class="absolute inset-y-0 left-0 flex items-center pl-2">
            <MdiIcon :path="mdiFilter" :size="16" />
          </span>
          <span class="block truncate ps-8">{{ selectedRooms.label }}</span>
          <span class="absolute inset-y-0 right-0 flex items-center pr-2">
            <MdiIcon :path="mdiUnfoldMoreHorizontal" :size="20" />
          </span>
        </ListboxButton>

        <Transition
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <ListboxOptions
            class="ring-black/5 dark:ring-white/5 bg-zinc-200 dark:bg-zinc-700 absolute !m-0 mt-1 max-h-60 w-full overflow-auto rounded-md text-base shadow-lg ring-1 focus:outline-none sm:text-sm"
          >
            <ListboxOption
              v-slot="{ active, selected }"
              :key="-1"
              :value="-1"
              as="li"
              class="cursor-pointer select-none list-none"
            >
              <div
                class="flex flex-row justify-start gap-3 px-3 py-2"
                :class="[active ? 'text-blue-900 dark:text-blue-50 bg-blue-100 dark:bg-blue-800' : 'text-zinc-900 dark:text-zinc-50']"
              >
                <span v-if="selected" class="text-blue-600 dark:text-blue-300 my-auto">
                  <MdiIcon :path="mdiCheck" :size="20" />
                </span>
                <div class="flex flex-grow flex-row justify-between gap-3">
                  <span
                    class="text-zinc-600 dark:text-zinc-300 my-auto block truncate"
                    :class="[selected ? 'font-medium' : 'ms-10 font-normal']"
                  >
                    {{ t("any_usage") }}
                  </span>
                  <span class="bg-blue-300 dark:bg-blue-600 rounded-md px-2 py-1 text-sm text-blue-950"
                    >{{ t("rooms", combined_list.length) }}
                  </span>
                </div>
              </div>
            </ListboxOption>
            <ListboxOption
              v-for="(usage, i) in props.rooms.usages"
              v-slot="{ active, selected }"
              :key="i"
              :value="i"
              as="li"
              class="cursor-pointer select-none list-none"
            >
              <div
                class="flex flex-row justify-start gap-3 px-3 py-2"
                :class="[active ? 'text-blue-900 dark:text-blue-50 bg-blue-100 dark:bg-blue-800' : 'text-zinc-900 dark:text-zinc-50']"
              >
                <span v-if="selected" class="text-blue-600 dark:text-blue-300 my-auto">
                  <MdiIcon :path="mdiCheck" :size="20" />
                </span>
                <div class="flex flex-grow flex-row justify-between gap-3">
                  <span
                    class="text-zinc-600 dark:text-zinc-300 my-auto block truncate"
                    :class="[selected ? 'font-medium' : 'ms-10 font-normal']"
                  >
                    {{ usage.name }}
                  </span>
                  <span class="bg-blue-300 dark:bg-blue-600 rounded-md px-2 py-1 text-sm text-blue-950"
                    >{{ t("rooms", usage.count) }}
                  </span>
                </div>
              </div>
            </ListboxOption>
          </ListboxOptions>
        </Transition>
      </Listbox>
      <div class="bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 z-0 flex w-full shrink items-center border">
        <MdiIcon :path="mdiMagnify" :size="24" class="text-zinc-600 dark:text-zinc-300 pl-2" aria-hidden="true" />
        <textarea
          id="search-input"
          v-model="search"
          :title="t('search')"
          :aria-label="t('search_rooms_of_building')"
          rows="1"
          aria-autocomplete="both"
          aria-haspopup="false"
          autocomplete="off"
          autocapitalize="off"
          spellcheck="false"
          maxlength="2048"
          type="text"
          class="focusable text-zinc-800 dark:text-zinc-100 bg-zinc-200 dark:bg-zinc-700 w-full flex-grow resize-none rounded-sm py-2 ps-6 font-semibold placeholder:text-zinc-800 dark:placeholder:text-zinc-100 focus-within:placeholder:text-zinc-500 dark:focus-within:placeholder:text-zinc-400 placeholder:font-normal"
          :placeholder="t('search')"
        />
      </div>
    </div>
    <div class="text-zinc-600 dark:text-zinc-300">
      <div
        v-if="filteredList.length > 0"
        v-bind="containerProps"
        class="bg-zinc-100 border-zinc-400 dark:border-zinc-500 max-h-96 overflow-y-scroll border p-2 dark:bg-zinc-700"
      >
        <ul v-bind="wrapperProps">
          <li>
            <NuxtLinkLocale
              v-for="(room, index) in list"
              :key="index"
              :to="entityPath(room.data.id, 'room')"
              class="flex h-[36px] max-h-[36px] min-h-[36px] flex-row gap-2 p-1.5 px-3 hover:text-white dark:hover:text-black hover:bg-blue-500 dark:hover:bg-blue-400"
            >
              <MdiIcon :path="mdiMapMarker" :size="16" class="my-auto" aria-hidden="true" />
              {{ room.data.name }}
            </NuxtLinkLocale>
          </li>
        </ul>
      </div>
      <div v-else class="flex flex-row items-baseline">
        {{ t("no_results_with_these_filters") }}
        <Btn
          size="sm"
          variant="linkButton"
          @click="
            () => {
              search = '';
              selectedUsage = -1;
            }
          "
          >{{ t("clear_filter") }}
        </Btn>
      </div>
      <small class="p-4">
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
  search_rooms_of_building: durchsucht Räume des Gebäudes
  view_on_map: Auf der Karte anzeigen
en:
  any_usage: any usage
  filter_by_usage: filter by usage
  no_results_with_these_filters: No results found with these filters.
  clear_filter: Clear the filters
  search: search
  results: "{count} result | {count} results"
  rooms: 1 room | {count} rooms
  title: Rooms
  search_rooms_of_building: searches rooms of the building
  view_on_map: View on map
</i18n>
