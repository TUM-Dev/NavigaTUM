<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import { mdiClose } from "@mdi/js";
import type { Ref } from "vue";
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";

type BuildingKind = components["schemas"]["BuildingKind"];

const editProposal = useEditProposal();
const { t } = useI18n({ useScope: "local" });

// Provided by AddProposalModal so we can render the global id input inline with the other
// identifier fields without duplicating the validation logic.
const idValidation = inject<{ pending: Ref<boolean>; collides: Ref<boolean> }>(
  "addProposal:idValidation",
  { pending: ref(false), collides: ref(false) }
);
const editExistingEntry = inject<() => Promise<void> | void>(
  "addProposal:editExistingEntry",
  () => {}
);

// Prefix picker uses the same building search as the parent picker so the user can pick existing
// buildings instead of typing 4-digit codes by hand. The picker's ids feed into building_prefixes;
// we clear the picker after each selection so the user can keep adding.
const prefixPickerId = ref("");
const prefixPickerName = ref("");
watch(prefixPickerId, (id) => {
  const value = id.trim();
  if (!value) return;
  if (!editProposal.value.pendingAddition.building_prefixes.includes(value)) {
    editProposal.value.pendingAddition.building_prefixes.push(value);
  }
  prefixPickerId.value = "";
  prefixPickerName.value = "";
});

function removePrefix(idx: number) {
  // The id-prefix is auto-managed (always == the entry id); only allow removing the extras.
  if (idx === 0 && editProposal.value.pendingAddition.node_kind !== "building") {
    return;
  }
  editProposal.value.pendingAddition.building_prefixes.splice(idx, 1);
}

// Keep the first prefix in sync with the entry id. For `building` kind that's the entire list;
// for `joined_building`/`area` users add additional prefixes after it.
watch(
  [() => editProposal.value.pendingAddition.id, () => editProposal.value.pendingAddition.node_kind],
  ([id, kind]) => {
    const trimmed = id.trim();
    const prefixes = editProposal.value.pendingAddition.building_prefixes;
    if (kind === "building") {
      editProposal.value.pendingAddition.building_prefixes = trimmed ? [trimmed] : [];
      return;
    }
    if (!trimmed) return;
    if (prefixes.length === 0) {
      prefixes.push(trimmed);
    } else if (prefixes[0] !== trimmed) {
      prefixes[0] = trimmed;
    }
  },
  { immediate: true }
);

const kindOptions: { value: BuildingKind; label: string }[] = [
  { value: "building", label: "building" },
  { value: "joined_building", label: "joined_building" },
  { value: "area", label: "area" },
];
const nodeKindIndex = computed(() => {
  const k = editProposal.value.pendingAddition.node_kind;
  return k ? kindOptions.findIndex((o) => o.value === k) : -1;
});
</script>

<template>
  <div class="space-y-3">
    <div>
      <label class="text-zinc-600 mb-1 block text-xs font-medium" for="add-building-name">
        {{ t("name") }} <span class="text-red-700">*</span>
      </label>
      <input
        id="add-building-name"
        v-model="editProposal.pendingAddition.name"
        type="text"
        class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
      />
    </div>

    <div>
      <label class="text-zinc-600 mb-1 block text-xs font-medium" for="add-building-short-name">{{ t("short_name") }}</label>
      <input
        id="add-building-short-name"
        v-model="editProposal.pendingAddition.short_name"
        type="text"
        class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
      />
    </div>

    <div>
      <span class="text-zinc-600 mb-1 block text-xs font-medium">{{ t("node_kind") }} <span class="text-red-700">*</span></span>
      <TabGroup :selected-index="nodeKindIndex < 0 ? 0 : nodeKindIndex" :default-index="0">
        <TabList class="bg-zinc-100 flex space-x-1 rounded-lg p-1">
          <Tab v-for="opt in kindOptions" :key="opt.value" as="template">
            <button
              :class="[
                'w-full rounded-md py-2 px-3 text-sm font-medium leading-5',
                'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                'focus:outline-none focus:ring-2 transition-all',
                nodeKindIndex === kindOptions.indexOf(opt) ? 'bg-white text-zinc-700 shadow' : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
              ]"
              @click="editProposal.pendingAddition.node_kind = opt.value"
            >
              {{ t(`kind.${opt.label}`) }}
            </button>
          </Tab>
        </TabList>
      </TabGroup>
      <p v-if="editProposal.pendingAddition.node_kind" class="text-zinc-500 mt-1 text-xs">
        {{ t(`node_kind_help.${editProposal.pendingAddition.node_kind}`) }}
      </p>
    </div>

    <fieldset class="border-zinc-300 rounded border px-3 pb-3 pt-2">
      <legend class="text-zinc-700 px-1 text-xs font-semibold">{{ t("identifiers.legend") }}</legend>
      <p class="text-zinc-500 mb-2 text-xs">{{ t("identifiers.intro") }}</p>

      <!-- Visual: shows a sample row with the four fields labelled. -->
      <div class="bg-zinc-50 border-zinc-200 mb-3 overflow-x-auto rounded border p-2 font-mono text-xs">
        <div class="text-zinc-700 mb-1 grid grid-cols-[110px_1fr] gap-x-3">
          <span class="text-zinc-500">{{ t("identifiers.diagram.id_label") }}</span>
          <span class="text-blue-700">5510</span>
        </div>
        <div class="text-zinc-700 mb-1 grid grid-cols-[110px_1fr] gap-x-3">
          <span class="text-zinc-500">{{ t("identifiers.diagram.prefixes_label") }}</span>
          <span class="text-emerald-700">5510, 5512</span>
        </div>
        <div class="text-zinc-700 mb-1 grid grid-cols-[110px_1fr] gap-x-3">
          <span class="text-zinc-500">{{ t("identifiers.diagram.internal_label") }}</span>
          <span class="text-amber-700">G05</span>
        </div>
        <div class="text-zinc-700 grid grid-cols-[110px_1fr] gap-x-3">
          <span class="text-zinc-500">{{ t("identifiers.diagram.visible_label") }}</span>
          <span class="text-purple-700">MW</span>
        </div>
      </div>

      <div class="space-y-3">
        <div>
          <label class="text-blue-700 mb-1 block text-xs font-medium" for="add-building-id">
            {{ t("identifiers.id") }} <span class="text-red-700">*</span>
          </label>
          <I18nT keypath="identifiers.id_help" tag="p" class="text-zinc-500 mb-1 text-xs">
            <template #example>
              <code class="font-mono">5510</code>
            </template>
          </I18nT>
          <input
            id="add-building-id"
            v-model="editProposal.pendingAddition.id"
            type="text"
            placeholder="5510"
            class="focusable bg-zinc-200 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
            :class="idValidation.collides.value ? 'border-red-500' : 'border-zinc-400'"
          />
          <p v-if="idValidation.pending.value" class="text-zinc-500 mt-1 text-xs">{{ t("identifiers.id_checking") }}</p>
          <template v-else-if="idValidation.collides.value">
            <p class="text-red-700 mt-1 text-xs">{{ t("identifiers.id_exists_on_server") }}</p>
            <button type="button" class="text-blue-600 hover:underline mt-1 text-xs" @click="editExistingEntry">
              {{ t("identifiers.edit_existing_instead") }}
            </button>
          </template>
        </div>

        <div
          v-if="
            editProposal.pendingAddition.node_kind === 'joined_building' ||
            editProposal.pendingAddition.node_kind === 'area'
          "
        >
          <label class="text-emerald-700 mb-1 block text-xs font-medium" for="add-building-prefixes">
            {{ t("identifiers.prefixes") }} <span class="text-red-700">*</span>
          </label>
          <p class="text-zinc-500 mb-1 text-xs">
            {{ t(`identifiers.prefixes_help.${editProposal.pendingAddition.node_kind}`) }}
          </p>
          <EntryPicker
            v-model:selected-id="prefixPickerId"
            v-model:selected-name="prefixPickerName"
            :allowed-types="['building']"
            :placeholder="t('identifiers.prefix_picker_placeholder')"
          />
          <div v-if="editProposal.pendingAddition.building_prefixes.length" class="mt-2 flex flex-wrap gap-1">
            <span
              v-for="(prefix, idx) in editProposal.pendingAddition.building_prefixes"
              :key="prefix"
              class="bg-emerald-100 text-emerald-900 inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs"
              :class="idx === 0 ? 'opacity-70' : ''"
              :title="idx === 0 ? t('identifiers.prefix_id_locked') : ''"
            >
              {{ prefix }}
              <button
                v-if="idx !== 0"
                type="button"
                class="focusable rounded-sm"
                @click="removePrefix(idx)"
              >
                <MdiIcon :path="mdiClose" :size="12" />
              </button>
            </span>
          </div>
        </div>

        <details>
          <summary class="text-zinc-600 cursor-pointer text-xs font-medium">{{ t("identifiers.optional") }}</summary>
          <div class="mt-2 space-y-3">
            <div>
              <label class="text-amber-700 mb-1 block text-xs font-medium" for="add-building-internal-id">
                {{ t("identifiers.internal_id") }}
              </label>
              <p class="text-zinc-500 mb-1 text-xs">{{ t("identifiers.internal_id_help") }}</p>
              <input
                id="add-building-internal-id"
                v-model="editProposal.pendingAddition.internal_id"
                type="text"
                class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
              />
            </div>

            <div>
              <label class="text-purple-700 mb-1 block text-xs font-medium" for="add-building-visible-id">
                {{ t("identifiers.visible_id") }}
              </label>
              <I18nT keypath="identifiers.visible_id_help" tag="p" class="text-zinc-500 mb-1 text-xs">
                <template #example><code class="font-mono">MW</code></template>
              </I18nT>
              <input
                id="add-building-visible-id"
                v-model="editProposal.pendingAddition.visible_id"
                type="text"
                class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 w-full rounded border px-2 py-1 text-sm"
              />
            </div>
          </div>
        </details>
      </div>
    </fieldset>

  </div>
</template>

<i18n lang="yaml">
de:
  name: Name
  short_name: Kurzname
  node_kind: Eintragsart
  node_kind_help:
    building: Ein einzelnes Gebäude.
    joined_building: Ein zusammengefasster Gebäudekomplex.
    area: Ein Areal.
  identifiers:
    legend: Kennungen
    intro: "Ein Gebäude hat mehrere IDs für unterschiedliche Zwecke. Hier ein Beispiel:"
    optional: Optionale IDs (interne / sichtbare ID)
    diagram:
      id_label: ID
      prefixes_label: Präfixe
      internal_label: Interne ID
      visible_label: Sichtbare ID
    id: ID
    id_help: Primäre ID. Wird in der URL und intern verwendet (z.B. {example}).
    id_checking: Prüfe Verfügbarkeit…
    id_exists_on_server: Diese ID existiert bereits in Navigatum. Bitte wähle eine andere.
    edit_existing_instead: Stattdessen das vorhandene Gebäude bearbeiten →
    prefixes: Zugehörige Gebäude
    prefixes_help:
      joined_building: Wähle alle Gebäude aus, die zu diesem Gebäudekomplex gehören.
      area: Wähle alle Gebäude aus, die zu diesem Areal gehören.
    prefix_picker_placeholder: Gebäude suchen…
    prefix_id_locked: Wird automatisch aus der ID gesetzt
    internal_id: Interne ID
    internal_id_help: "Optional. Bezeichnung in TUM-internen Systemen (z.B. Gebäude-Liste der Bauverwaltung)."
    visible_id: Sichtbare ID
    visible_id_help: Optional. Kürzel, das in einigen Oberflächen statt der ID angezeigt wird (z.B. {example}).
  kind:
    building: Gebäude
    joined_building: Gebäudeverbund
    area: Areal
en:
  name: Name
  short_name: Short name
  node_kind: Entry kind
  node_kind_help:
    building: A single building.
    joined_building: A complex of joined buildings.
    area: A campus or area.
  identifiers:
    legend: Identifiers
    intro: "A building has several ids serving different purposes. Example:"
    optional: Optional ids (internal / visible)
    diagram:
      id_label: ID
      prefixes_label: Prefixes
      internal_label: Internal id
      visible_label: Visible id
    id: ID
    id_help: Primary id. Shown in the URL and used internally (e.g. {example}).
    id_checking: Checking availability…
    id_exists_on_server: This id already exists in Navigatum. Please pick a different one.
    edit_existing_instead: Edit the existing building instead →
    prefixes: Member buildings
    prefixes_help:
      joined_building: Pick every building that belongs to this complex.
      area: Pick every building that belongs to this area.
    prefix_picker_placeholder: Search for a building…
    prefix_id_locked: Auto-derived from the id
    internal_id: Internal id
    internal_id_help: "Optional. Identifier in TUM-internal systems (e.g. the construction office's building list)."
    visible_id: Visible id
    visible_id_help: Optional. Short label shown in some UIs instead of the id (e.g. {example}).
  kind:
    building: Building
    joined_building: Joined building
    area: Area
</i18n>
