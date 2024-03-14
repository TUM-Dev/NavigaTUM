<script setup lang="ts">
import { MapSelections, useDetailsStore } from "../stores/details";
import { useI18n } from "vue-i18n";
import Modal from "../components/Modal.vue";
import RoomfinderImageLocation from "../components/RoomfinderImageLocation.vue";
import { computed, ref } from "vue";
import { CheckIcon, ChevronUpDownIcon } from "@heroicons/vue/20/solid";
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from "@headlessui/vue";
import type { components } from "../api_types";

type RoomfinderMapEntry = components["schemas"]["RoomfinderMapEntry"];

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });

defineExpose({
  loadRoomfinderMap,
});

function loadRoomfinderMap(mapIndex: number, fromUi = false) {
  state.map.selected = MapSelections.roomfinder;
  state.map.roomfinder.selected_index = mapIndex;

  if (fromUi) {
    window.scrollTo(0, 0);
  }
}

const modalOpen = ref(false);
const selectedMap = computed<RoomfinderMapEntry>(() => {
  return state.data?.maps.roomfinder?.available[state.map.roomfinder.selected_index] as RoomfinderMapEntry;
});
</script>

<template>
  <template v-if="state.data?.maps.roomfinder?.available">
    <div>
      <Listbox v-model="state.map.roomfinder.selected_index">
        <div class="relative mt-1">
          <ListboxButton
            class="focusable bg-white relative w-full cursor-pointer rounded-lg py-2 pl-3 pr-10 text-left shadow-md sm:text-sm"
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
              class="bg-white absolute mt-1 max-h-60 w-full overflow-auto rounded-md py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
            >
              <ListboxOption
                v-for="(map, i) in state.data?.maps.roomfinder?.available"
                v-slot="{ active, selected }"
                :key="map.id"
                :value="i"
                as="template"
              >
                <li
                  class="relative cursor-pointer select-none py-2 pl-10 pr-4"
                  :class="[active ? 'text-tumBlue-900 bg-tumBlue-100' : 'text-zinc-900']"
                >
                  <span class="block truncate" :class="[selected ? 'font-medium' : 'font-normal']">
                    1:{{ map.scale }}, {{ map.name }}
                  </span>
                  <span v-if="selected" class="text-tumBlue-600 absolute inset-y-0 left-0 flex items-center pl-3">
                    <CheckIcon class="h-5 w-5" />
                  </span>
                </li>
              </ListboxOption>
            </ListboxOptions>
          </Transition>
        </div>
      </Listbox>
      <button type="button" :aria-label="t('open_detailed_modal')" @click="modalOpen = true">
        <RoomfinderImageLocation id="rf_outer_image" :map="state.selectedRoomfinderMap()" />
      </button>
    </div>

    <Modal v-model="modalOpen" :title="t('modal.header')" :classes="{ modal: 'items-baseline' }">
      <RoomfinderImageLocation id="rf_modal_image" :map="state.selectedRoomfinderMap()" />
    </Modal>
  </template>
</template>

<i18n lang="yaml">
de:
  img_alt: Handgezeichnete Roofinder-Kartendarstellung
  open_detailed_modal: Show a larger popup of the map
  modal:
    header: Lageplan
en:
  img_alt: Hand-drawn roomfinder map image
  open_detailed_modal: Ein größeres Popup der Karte anzeigen
  modal:
    header: Site Plan
</i18n>
