<script setup lang="ts">
import {
  Combobox,
  ComboboxButton,
  ComboboxInput,
  ComboboxOption,
  ComboboxOptions,
} from "@headlessui/vue";
import { mdiCheck, mdiUnfoldMoreHorizontal } from "@mdi/js";
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";
import { type UsageOption, useKnownUsages } from "~/composables/knownUsages";

type FloorType = components["schemas"]["FloorType"];

// Order chosen so the dropdown reads bottom-up (roof first looks weird, basement first matches
// real-world thinking: where is the floor relative to the ground?).
const floorTypeOptions: readonly FloorType[] = [
  "basement",
  "semi_basement",
  "ground",
  "semi_upper",
  "upper",
  "roof",
];

const editProposal = useEditProposal();
const { t } = useI18n({ useScope: "local" });

const knownUsages = useKnownUsages();

const usageQuery = ref("");
const filteredUsages = computed<UsageOption[]>(() => knownUsages.filter(usageQuery.value));

const selectedUsage = computed<UsageOption | null>({
  get: () => knownUsages.byId(editProposal.value.pendingAddition.usage_id),
  set: (u) => {
    editProposal.value.pendingAddition.usage_id = u?.usage_id ?? null;
  },
});
</script>

<template>
  <div class="space-y-3">
    <div>
      <label class="text-zinc-600 mb-1 block text-xs font-medium" for="add-room-arch-name">
        {{ t("arch_name") }} <span class="text-red-700">*</span>
      </label>
      <input
        id="add-room-arch-name"
        v-model="editProposal.pendingAddition.arch_name"
        type="text"
        placeholder="003@5510"
        class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
      />
      <I18nT keypath="arch_name_help" tag="p" class="text-zinc-500 mt-1 text-xs">
        <template #ex1><code class="font-mono">MW2001</code></template>
        <template #ex2><code class="font-mono">003</code></template>
      </I18nT>
    </div>

    <div>
      <span class="text-zinc-600 mb-1 block text-xs font-medium">{{ t("usage_id") }} <span class="text-red-700">*</span></span>
      <Combobox v-model="selectedUsage" :nullable="true" by="usage_id">
        <div class="relative">
          <div
            class="bg-zinc-200 border-zinc-400 focus-within:border-blue-500 relative flex w-full items-center rounded-md border text-left text-sm"
          >
            <ComboboxInput
              class="text-zinc-900 w-full rounded-md border-none bg-transparent py-2 pl-3 pr-10 text-sm focus:outline-none"
              :display-value="(u: unknown) => (u as UsageOption | null)?.label ?? ''"
              :placeholder="t('usage_placeholder')"
              @change="usageQuery = ($event.target as HTMLInputElement).value"
            />
            <ComboboxButton class="absolute inset-y-0 right-0 flex items-center pr-2">
              <MdiIcon :path="mdiUnfoldMoreHorizontal" :size="20" class="text-zinc-600" aria-hidden="true" />
            </ComboboxButton>
          </div>
          <Transition leave-active-class="transition duration-100 ease-in" leave-from-class="opacity-100" leave-to-class="opacity-0">
            <ComboboxOptions
              class="ring-black/5 bg-zinc-50 absolute z-30 mt-1 max-h-72 w-full overflow-auto rounded-md py-1 shadow-lg ring-1 focus:outline-none"
            >
              <p v-if="filteredUsages.length === 0" class="text-zinc-500 px-3 py-2 text-sm">
                {{ t("usage_no_results") }}
              </p>
              <ComboboxOption
                v-for="u in filteredUsages"
                :key="u.usage_id"
                v-slot="{ active, selected }"
                :value="u"
                as="template"
              >
                <li
                  class="relative cursor-pointer select-none py-2 pl-3 pr-8"
                  :class="active ? 'bg-blue-100 text-blue-900' : 'text-zinc-900'"
                >
                  <div class="flex items-baseline gap-3">
                    <UsageOptionContent :usage="u" :emphasised="selected" />
                  </div>
                  <span v-if="selected" class="text-blue-600 absolute inset-y-0 right-0 flex items-center pr-2">
                    <MdiIcon :path="mdiCheck" :size="16" aria-hidden="true" />
                  </span>
                </li>
              </ComboboxOption>
            </ComboboxOptions>
          </Transition>
        </div>
      </Combobox>
    </div>

    <details class="border-zinc-300 rounded border px-3 py-2">
      <summary class="text-zinc-600 cursor-pointer text-xs font-medium">{{ t("more_options") }}</summary>
      <p class="text-zinc-500 mt-1 text-xs">{{ t("more_options_help") }}</p>
      <div class="mt-3 space-y-3">
        <div>
          <span class="text-zinc-600 mb-1 block text-xs font-medium">{{ t("floor.legend") }}</span>
          <p class="text-zinc-500 mb-2 text-xs">{{ t("floor.help") }}</p>
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="text-zinc-500 block text-xs" for="add-room-floor-type">{{ t("floor.type") }}</label>
              <select
                id="add-room-floor-type"
                v-model="editProposal.pendingAddition.floor_type"
                class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
              >
                <option value="">—</option>
                <option v-for="ft in floorTypeOptions" :key="ft" :value="ft">{{ t(`floor.type_options.${ft}`) }}</option>
              </select>
            </div>
            <div>
              <label class="text-zinc-500 block text-xs" for="add-room-floor-level">{{ t("floor.level") }}</label>
              <input
                id="add-room-floor-level"
                v-model="editProposal.pendingAddition.floor_level"
                type="text"
                placeholder="EG, 01, U1, …"
                class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
              />
            </div>
          </div>
        </div>

        <div>
          <span class="text-zinc-600 mb-1 block text-xs font-medium">{{ t("seats") }}</span>
          <div class="grid grid-cols-3 gap-2">
            <div>
              <label class="text-zinc-500 block text-xs" for="add-room-seats-sit">{{ t("seats_sitting") }}</label>
              <input
                id="add-room-seats-sit"
                v-model.number="editProposal.pendingAddition.seats.sitting"
                type="number"
                min="0"
                class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
              />
            </div>
            <div>
              <label class="text-zinc-500 block text-xs" for="add-room-seats-stand">{{ t("seats_standing") }}</label>
              <input
                id="add-room-seats-stand"
                v-model.number="editProposal.pendingAddition.seats.standing"
                type="number"
                min="0"
                class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
              />
            </div>
            <div>
              <label class="text-zinc-500 block text-xs" for="add-room-seats-wheel">{{ t("seats_wheelchair") }}</label>
              <input
                id="add-room-seats-wheel"
                v-model.number="editProposal.pendingAddition.seats.wheelchair"
                type="number"
                min="0"
                class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
              />
            </div>
          </div>
        </div>

        <div>
          <span class="text-zinc-600 mb-1 block text-xs font-medium">{{ t("links") }}</span>
          <LinkRowEditor v-model="editProposal.pendingAddition.room_links" />
        </div>
      </div>
    </details>
  </div>
</template>

<i18n lang="yaml">
de:
  arch_name: Architekturname
  arch_name_help: Der Name, den der Architekt diesem Raum gegeben hat, z.B. {ex1}, {ex2}, …
  usage_id: Nutzungsart
  usage_placeholder: Suchen…
  usage_no_results: Keine passende Nutzungsart
  more_options: Weitere Felder
  more_options_help: Optional. Stockwerk, Sitzplätze und Links können später ergänzt werden.
  floor:
    legend: Stockwerk
    help: Wo liegt der Raum im Gebäude und wie ist die Etage beschriftet?
    type: Lage
    level: Beschriftung
    type_options:
      basement: Untergeschoss
      semi_basement: Teil-Untergeschoss
      ground: Erdgeschoss
      semi_upper: Zwischengeschoss
      upper: Obergeschoss
      roof: Dachgeschoss
  seats: Sitzplätze
  seats_sitting: Sitzend
  seats_standing: Stehend
  seats_wheelchair: Rollstuhl
  links: Links
en:
  arch_name: Architectural name
  arch_name_help: The name the architect gave this room, e.g. {ex1}, {ex2}, …
  usage_id: Usage type
  usage_placeholder: Search…
  usage_no_results: No matching usage type
  more_options: More fields
  more_options_help: Optional. Floor, seats and links can be added later if needed.
  floor:
    legend: Floor
    help: Where the room sits in the building and how the floor is labelled.
    type: Position
    level: Label
    type_options:
      basement: Basement
      semi_basement: Half-basement
      ground: Ground floor
      semi_upper: Mezzanine
      upper: Upper floor
      roof: Roof

  seats: Seats
  seats_sitting: Sitting
  seats_standing: Standing
  seats_wheelchair: Wheelchair
  links: Links
</i18n>
