<script setup lang="ts">
import { CheckIcon, ChevronUpDownIcon } from "@heroicons/vue/20/solid";
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from "@headlessui/vue";
import type { components } from "~/api_types";

type RoomfinderMapEntry = components["schemas"]["RoomfinderMapEntry"];

const props = defineProps<{
  available: readonly RoomfinderMapEntry[];
  defaultMapId: string;
}>();
const { t } = useI18n({ useScope: "local" });

const modalOpen = ref(false);
onBeforeMount(() => {
  for (let index = 0; index < props.available.length; index++) {
    if (props.available[index]?.id === props.defaultMapId) {
      selected_index.value = index;
      return;
    }
  }
});
const selected_index = ref(0);
const selectedMap = computed<RoomfinderMapEntry>(() => {
  return props.available[selected_index.value] as RoomfinderMapEntry;
});
</script>

<template>
  <template v-if="available">
    <div class="flex flex-col gap-2 pb-3">
      <Listbox v-model="selected_index">
        <div class="relative mt-1">
          <ListboxButton
            class="focusable text-zinc-900 bg-zinc-100 relative w-full cursor-pointer rounded-lg py-2 pl-3 pr-10 text-left shadow-md sm:text-sm"
          >
            <span class="block truncate">1:{{ selectedMap.scale }}, {{ selectedMap.name }}</span>
            <span class="absolute inset-y-0 right-0 flex items-center pr-2">
              <ChevronUpDownIcon class="text-zinc-400 h-5 w-5" />
            </span>
          </ListboxButton>

          <Transition
            leave-active-class="transition duration-100 ease-in"
            leave-from-class="opacity-100"
            leave-to-class="opacity-0"
          >
            <ListboxOptions
              class="ring-black/5 text-zinc-900 bg-zinc-100 absolute mt-1 max-h-60 w-full overflow-auto rounded-md py-1 text-base shadow-lg ring-1 focus:outline-none sm:text-sm"
            >
              <ListboxOption
                v-for="(map, i) in available"
                v-slot="{ active, selected }"
                :key="map.id"
                :value="i"
                as="template"
              >
                <li
                  class="relative cursor-pointer select-none py-2 pl-10 pr-4"
                  :class="[active ? 'text-blue-900 bg-blue-100' : 'text-zinc-900']"
                >
                  <span class="block truncate" :class="[selected ? 'font-medium' : 'font-normal']">
                    1:{{ map.scale }}, {{ map.name }}
                  </span>
                  <span v-if="selected" class="text-blue-600 absolute inset-y-0 left-0 flex items-center pl-3">
                    <CheckIcon class="h-5 w-5" />
                  </span>
                </li>
              </ListboxOption>
            </ListboxOptions>
          </Transition>
        </div>
      </Listbox>
      <button type="button" :aria-label="t('open_detailed_modal')" @click="modalOpen = true">
        <ClientOnly>
          <RoomfinderImageLocation id="rf_outer_image" :map="selectedMap" />
        </ClientOnly>
      </button>
    </div>

    <ClientOnly>
      <LazyModal v-model="modalOpen" :title="t('modal.header')" class="items-baseline">
        <LazyRoomfinderImageLocation id="rf_modal_image" :map="selectedMap" />
      </LazyModal>
    </ClientOnly>
  </template>
</template>

<i18n lang="yaml">
de:
  img_alt: Handgezeichnete Roofinder-Kartendarstellung
  open_detailed_modal: Show a larger popup of the map
  modal:
    header: Lageplan
en:
  img_alt: Hand-drawn location map optimised for printing
  open_detailed_modal: Ein größeres Popup der Karte anzeigen
  modal:
    header: Site Plan
</i18n>
