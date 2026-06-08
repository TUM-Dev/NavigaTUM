<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import { mdiCalendarStar, mdiDomain, mdiMapMarker, mdiSofa } from "@mdi/js";
import { useDebounceFn } from "@vueuse/core";
import type { components } from "~/api_types";
import { type AdditionFieldErrors, validateAddition } from "~/composables/additionSchema";
import { type AdditionKind, emptyAdditionDraft, useEditProposal } from "~/composables/editProposal";
import { wallTimeToRfc3339 } from "~/utils/datetime";
import { entityPath, isRoutableEntityType } from "~/utils/entityPath";

type FacetFilter = components["schemas"]["FacetFilter"];
type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

const FOUR_DIGIT_PREFIX = /^\d{4}$/;

const editProposal = useEditProposal();
const { t } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();

const localError = ref("");

const kindOptions: { value: AdditionKind; icon: string }[] = [
  { value: "room", icon: mdiSofa },
  { value: "building", icon: mdiDomain },
  { value: "poi", icon: mdiMapMarker },
  { value: "event", icon: mdiCalendarStar },
];
const kindIndex = computed(() => {
  const k = editProposal.value.pendingAddition.kind;
  return k ? kindOptions.findIndex((o) => o.value === k) : -1;
});
// Debounce + verify the id against /api/locations/{id}; 200 means collision, 404 means free.
const idCheckPending = ref(false);
const idCollidesOnServer = ref(false);
// Counter still needed to invalidate in-flight fetches when the input changes
// after the debounce already fired; `useDebounceFn` only cancels pending calls.
let idCheckCounter = 0;
const runIdCheck = useDebounceFn(async (id: string, ticket: number) => {
  try {
    const res = await fetch(
      `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(id)}`,
      { credentials: "omit" }
    );
    if (ticket !== idCheckCounter) return;
    idCollidesOnServer.value = res.ok;
  } catch {
    // Network failure: don't block. The server validates again on submit.
  } finally {
    if (ticket === idCheckCounter) idCheckPending.value = false;
  }
}, 350);
watch(
  () => editProposal.value.pendingAddition.id,
  (value) => {
    idCheckCounter++;
    idCollidesOnServer.value = false;
    const id = value.trim();
    if (!id || editProposal.value.pendingAddition.kind === "event") {
      idCheckPending.value = false;
      return;
    }
    idCheckPending.value = true;
    runIdCheck(id, idCheckCounter);
  }
);

const allowedParentTypes = computed<readonly FacetFilter[]>(() => {
  const kind = editProposal.value.pendingAddition.kind;
  if (kind === "room") return ["building"];
  // POIs may live inside a site/area or directly inside a building (e.g. a cafeteria);
  // buildings are parented under sites/areas only.
  if (kind === "poi") return ["site", "building"];
  return ["site"];
});

// When the user picks a parent, fetch its details so we can pre-fill the map centre + auto-mark
// coords as ready (saving a click; the user can still drag to refine).
const parentLookupUrl = computed(() => {
  const pid = editProposal.value.pendingAddition.parent_id;
  return pid ? `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(pid)}` : "";
});
interface ParentDetails {
  id: string;
  coords?: { lat: number; lon: number };
  aliases?: readonly string[];
  props?: {
    floors?: readonly {
      id: number;
      name: string;
      short_name: string;
      tumonline: string;
      type: string;
    }[];
  };
}
const { data: parentDetails } = useFetch<ParentDetails>(() => parentLookupUrl.value, {
  immediate: false,
  lazy: true,
  dedupe: "cancel",
  credentials: "omit",
  watch: [() => editProposal.value.pendingAddition.parent_id],
});

// The room-code prefix isn't always the entry id (joined buildings have textual ids like `mi`,
// while their TUMonline code is e.g. `5510`). Pick the first 4-digit alias as the prefix.
const roomParentPrefix = computed(() => {
  const parentId = editProposal.value.pendingAddition.parent_id.trim();
  if (!parentId) return "";
  if (FOUR_DIGIT_PREFIX.test(parentId)) return parentId;
  const aliases = parentDetails.value?.aliases ?? [];
  const numeric = aliases.find((a) => FOUR_DIGIT_PREFIX.test(a));
  return numeric ?? parentId;
});

// Floors known on the parent - what the TUMonline room-code uses for the floor segment.
interface ParentFloorOption {
  tumonline: string;
  label: string;
}
const parentFloorOptions = computed<ParentFloorOption[]>(() => {
  const floors = parentDetails.value?.props?.floors ?? [];
  return floors
    .filter((f) => Boolean(f.tumonline))
    .map((f) => ({ tumonline: f.tumonline, label: `${f.tumonline} - ${f.short_name || f.name}` }));
});

// Room IDs follow PARENT.FLOOR.NUMBER. The parent segment is auto-filled and disabled so users
// can't desync it from the chosen parent; floor and number flow through the Zod schema like any
// other field once we compose the id.
const roomFloorSegment = ref("");
const roomNumberSegment = ref("");
const composedRoomId = computed(() => {
  const prefix = roomParentPrefix.value;
  const floor = roomFloorSegment.value.trim();
  const number = roomNumberSegment.value.trim();
  if (!prefix || !floor || !number) return "";
  return `${prefix}.${floor}.${number}`;
});
watch([composedRoomId, () => editProposal.value.pendingAddition.kind], ([id, kind]) => {
  if (kind !== "room") return;
  if (editProposal.value.pendingAddition.id === id) return;
  editProposal.value.pendingAddition.id = id;
});
// Reset the local segment refs when the kind changes or the draft id is cleared (commit/cancel
// replace `pendingAddition` with `emptyAdditionDraft()`).
watch(
  [() => editProposal.value.pendingAddition.kind, () => editProposal.value.pendingAddition.id],
  ([kind, id]) => {
    if (kind !== "room" || !id) {
      roomFloorSegment.value = "";
      roomNumberSegment.value = "";
    }
  }
);

const fieldErrors = computed<AdditionFieldErrors>(() =>
  validateAddition(editProposal.value.pendingAddition)
);
const roomIdFormatError = computed<string | null>(() => {
  const draft = editProposal.value.pendingAddition;
  if (draft.kind !== "room") return null;
  if (!roomFloorSegment.value.trim() && !roomNumberSegment.value.trim()) return null;
  return fieldErrors.value.id ?? null;
});
const draftIsReady = computed(() => {
  if (idCollidesOnServer.value || idCheckPending.value) return false;
  return Object.keys(fieldErrors.value).length === 0;
});

function buildAddition(): components["schemas"]["LimitedHashMap_String_Addition"][string] | null {
  const draft = editProposal.value.pendingAddition;
  const coords = { lat: draft.coords.lat, lon: draft.coords.lon };
  if (draft.kind === "room") {
    const seats =
      draft.seats.sitting !== null ||
      draft.seats.standing !== null ||
      draft.seats.wheelchair !== null
        ? {
            sitting: draft.seats.sitting,
            standing: draft.seats.standing,
            wheelchair: draft.seats.wheelchair,
          }
        : null;
    const links = draft.room_links.filter((l) => l.url.trim());
    return {
      kind: "room",
      parent_building_id: draft.parent_id,
      alt_name: draft.alt_name,
      arch_name: draft.arch_name,
      usage_id: draft.usage_id as number,
      coords,
      seats,
      floor_type: draft.floor_type || null,
      floor_level: draft.floor_level || null,
      // Address omitted on purpose: the server inherits it from the parent building.
      address: null,
      links: links.length > 0 ? links : undefined,
    } as components["schemas"]["LimitedHashMap_String_Addition"][string];
  }
  if (draft.kind === "building") {
    if (!draft.node_kind) return null;
    return {
      kind: "building",
      parent_id: draft.parent_id,
      name: draft.name,
      short_name: draft.short_name || null,
      node_kind: draft.node_kind,
      building_prefixes: [...draft.building_prefixes],
      internal_id: draft.internal_id || null,
      visible_id: draft.visible_id || null,
      coords,
    } as components["schemas"]["LimitedHashMap_String_Addition"][string];
  }
  if (draft.kind === "poi") {
    const links = draft.poi_links
      .filter((l) => l.url.trim())
      .map((l) => ({ url: l.url, text: { de: l.text_de, en: l.text_en } }));
    const generic_props = draft.generic_props
      .filter((p) => p.name_de.trim() || p.name_en.trim() || p.text.trim())
      .map((p) => ({ name: { de: p.name_de, en: p.name_en }, text: p.text }));
    const comment =
      draft.comment_de.trim() || draft.comment_en.trim()
        ? { de: draft.comment_de, en: draft.comment_en }
        : null;
    return {
      kind: "poi",
      parent: draft.parent_id,
      name: draft.name,
      usage_name: draft.usage_name,
      coords,
      comment,
      links: links.length > 0 ? links : undefined,
      generic_props: generic_props.length > 0 ? generic_props : undefined,
    } as components["schemas"]["LimitedHashMap_String_Addition"][string];
  }
  if (draft.kind === "event") {
    if (!draft.image) return null;
    return {
      kind: "event",
      name: draft.name,
      description: draft.description,
      starts_at: wallTimeToRfc3339(draft.starts_at) ?? "",
      ends_at: wallTimeToRfc3339(draft.ends_at) ?? "",
      coords,
      organising_org_id: draft.organising_org_id as number,
      image: {
        content: draft.image.base64,
        metadata: {
          author: draft.image_author,
          license: { text: "CC BY 4.0", url: "https://creativecommons.org/licenses/by/4.0/" },
          offsets:
            draft.image_thumb_offset === 0 && draft.image_header_offset === 0
              ? null
              : { thumb: draft.image_thumb_offset, header: draft.image_header_offset },
        },
      },
    } as components["schemas"]["LimitedHashMap_String_Addition"][string];
  }
  return null;
}

function commitDraft(): { id: string; displayName: string } | null {
  localError.value = "";
  const id = editProposal.value.pendingAddition.id.trim();
  if (!id) {
    localError.value = t("error.id_required");
    return null;
  }
  if (editProposal.value.data.additions[id] || editProposal.value.data.edits[id]) {
    localError.value = t("error.id_taken");
    return null;
  }
  if (idCollidesOnServer.value) {
    localError.value = t("error.id_exists_on_server");
    return null;
  }
  const addition = buildAddition();
  if (!addition) {
    localError.value = t("error.incomplete");
    return null;
  }
  const draft = editProposal.value.pendingAddition;
  // Best display name we have for the just-added entry, used by the image-upload flow.
  const displayName = (draft.kind === "room" ? draft.alt_name : draft.name) || id;
  // The OpenAPI types are readonly; round-trip through JSON to land on a mutable structural
  // clone matching the LimitedHashMap value type expected by `data.additions`.
  editProposal.value.data.additions[id] = JSON.parse(JSON.stringify(addition));
  editProposal.value.pendingAddition = emptyAdditionDraft();
  return { id, displayName };
}

function commitAddition() {
  if (!commitDraft()) return;
  // Hand back to the Propose Changes modal - submission/privacy/send live there.
  editProposal.value.addOpen = false;
  editProposal.value.open = true;
}

function commitAndAddImage() {
  const result = commitDraft();
  if (!result) return;
  // Point the existing image-upload flow at the just-added entry. The server applies additions
  // before edits in a single request, so an image edit keyed by this id resolves correctly.
  editProposal.value.selected = { id: result.id, name: result.displayName };
  editProposal.value.addOpen = false;
  editProposal.value.open = true;
  editProposal.value.imageUpload.open = true;
}

function cancelAddition() {
  editProposal.value.pendingAddition = emptyAdditionDraft();
  editProposal.value.addOpen = false;
  editProposal.value.open = true;
}

const localePath = useLocalePath();
async function editExistingEntry() {
  const id = editProposal.value.pendingAddition.id.trim();
  if (!id) return;
  editProposal.value.pendingAddition = emptyAdditionDraft();
  editProposal.value.addOpen = false;
  editProposal.value.selected = { id, name: null };
  // Open the edit modal once we land on the entry's detail page.
  editProposal.value.open = true;
  // Resolve the entity's type up front so we land on its canonical /{type}/{id} path directly
  // instead of bouncing through the /view/{id} redirect. On any failure (network, unknown type),
  // fall back to /view/{id}, which the server redirects to the canonical path.
  let target = `/view/${id}`;
  try {
    const res = await fetch(
      `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(id)}`,
      {
        credentials: "omit",
      }
    );
    if (res.ok) {
      const details = (await res.json()) as Pick<LocationDetailsResponse, "type">;
      if (isRoutableEntityType(details.type)) target = entityPath(id, details.type);
    }
  } catch {
    // Network failure: keep the /view/{id} fallback.
  }
  await navigateTo(localePath(target));
}
provide("addProposal:editExistingEntry", editExistingEntry);

// Coordinate model for the inline picker. Centred on TUM main campus until the user picks a parent
// (then the map recenters on the parent) or moves the marker themselves.
const mapInitialLat = ref(48.149);
const mapInitialLon = ref(11.568);

watch(parentDetails, (details) => {
  const lat = details?.coords?.lat;
  const lon = details?.coords?.lon;
  if (typeof lat !== "number" || typeof lon !== "number") return;
  mapInitialLat.value = lat;
  mapInitialLon.value = lon;
  if (!editProposal.value.pendingAddition.coords.picked) {
    editProposal.value.pendingAddition.coords = { lat, lon, picked: true };
  }
});

const mapLat = computed({
  get: () => editProposal.value.pendingAddition.coords.lat || mapInitialLat.value,
  set: (v: number) => {
    editProposal.value.pendingAddition.coords.lat = v;
    editProposal.value.pendingAddition.coords.picked = true;
  },
});
const mapLon = computed({
  get: () => editProposal.value.pendingAddition.coords.lon || mapInitialLon.value,
  set: (v: number) => {
    editProposal.value.pendingAddition.coords.lon = v;
    editProposal.value.pendingAddition.coords.picked = true;
  },
});

// Share id-validation state with per-kind sub-components (e.g. AddBuildingFields renders the id
// input itself inside its Identifiers fieldset).
provide("addProposal:idValidation", {
  pending: idCheckPending,
  collides: idCollidesOnServer,
});

watch(
  () => editProposal.value.addOpen,
  (isOpen) => {
    if (!isOpen) {
      editProposal.value.pendingAddition = emptyAdditionDraft();
      localError.value = "";
    }
  }
);
</script>

<template>
  <Modal v-if="editProposal" v-model="editProposal.addOpen" :title="t('title')" @close="cancelAddition">
    <Toast v-if="localError" id="add-proposal-local-error" class="mb-3" :msg="localError" level="error" />

    <div class="space-y-3">
      <TabGroup :selected-index="kindIndex < 0 ? 0 : kindIndex" :default-index="0">
        <TabList class="bg-zinc-100 dark:bg-zinc-800 flex space-x-1 rounded-lg p-1">
          <Tab v-for="opt in kindOptions" :key="opt.value" as="template">
            <button
              :class="[
                'w-full rounded-md py-2.5 px-3 text-sm font-medium leading-5',
                'ring-white/60 dark:ring-black/60 ring-offset-2 ring-offset-blue-400 dark:ring-offset-blue-500',
                'focus:outline-none focus:ring-2 transition-all',
                kindIndex === kindOptions.indexOf(opt) ? 'bg-white dark:bg-black text-zinc-700 dark:text-zinc-200 shadow' : 'text-zinc-500 dark:text-zinc-400 hover:bg-white/[0.12] dark:hover:bg-black/[0.12] hover:text-zinc-700 dark:hover:text-zinc-200',
              ]"
              @click="editProposal.pendingAddition.kind = opt.value"
            >
              <div class="flex items-center justify-center gap-2">
                <MdiIcon :path="opt.icon" :size="16" />
                {{ t(`kind.${opt.value}`) }}
              </div>
            </button>
          </Tab>
        </TabList>
      </TabGroup>

      <template v-if="editProposal.pendingAddition.kind">
        <div v-if="editProposal.pendingAddition.kind !== 'event'">
          <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">
            {{ t("parent_label") }} <span class="text-red-700 dark:text-red-200">*</span>
          </label>
          <EntryPicker
            v-model:selected-id="editProposal.pendingAddition.parent_id"
            v-model:selected-name="editProposal.pendingAddition.parent_name"
            :allowed-types="allowedParentTypes"
            :placeholder="t('parent_placeholder')"
          />
        </div>

        <!-- Room name comes between parent and id so the user works top-down: where → what's it called → its id. -->
        <div v-if="editProposal.pendingAddition.kind === 'room'">
          <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-room-alt-name">
            {{ t("alt_name") }} <span class="text-red-700 dark:text-red-200">*</span>
          </label>
          <input
            id="add-room-alt-name"
            v-model="editProposal.pendingAddition.alt_name"
            type="text"
            class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
          />
          <I18nT keypath="alt_name_help" tag="p" class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">
            <template #example>
              <code class="font-mono">{{ t("alt_name_help_example") }}</code>
            </template>
          </I18nT>
        </div>

        <!-- Buildings render the id input inside AddBuildingFields; events derive it from the image. -->
        <div v-if="editProposal.pendingAddition.kind !== 'building' && editProposal.pendingAddition.kind !== 'event'">
          <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-id">
            {{ t("id_label") }} <span class="text-red-700 dark:text-red-200">*</span>
          </label>
          <div v-if="editProposal.pendingAddition.kind === 'room'" class="flex items-center gap-1">
            <input
              id="add-id"
              :value="roomParentPrefix"
              type="text"
              disabled
              :placeholder="t('room_id_parent_placeholder')"
              class="bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-500 dark:text-zinc-400 w-24 cursor-not-allowed rounded border px-2 py-1 text-sm"
            />
            <span class="text-zinc-500 dark:text-zinc-400 select-none">.</span>
            <select
              v-if="parentFloorOptions.length"
              v-model="roomFloorSegment"
              class="focusable bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-50 w-32 rounded border px-2 py-1 text-sm"
              :class="roomIdFormatError ? 'border-red-500 dark:border-red-400' : 'border-zinc-400 dark:border-zinc-500'"
            >
              <option value="">-</option>
              <option v-for="f in parentFloorOptions" :key="f.tumonline" :value="f.tumonline">{{ f.label }}</option>
            </select>
            <input
              v-else
              v-model="roomFloorSegment"
              type="text"
              :placeholder="t('room_id_floor_placeholder')"
              class="focusable bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-50 w-20 rounded border px-2 py-1 text-sm"
              :class="roomIdFormatError ? 'border-red-500 dark:border-red-400' : 'border-zinc-400 dark:border-zinc-500'"
            />
            <span class="text-zinc-500 dark:text-zinc-400 select-none">.</span>
            <input
              v-model="roomNumberSegment"
              type="text"
              :placeholder="t('room_id_number_placeholder')"
              class="focusable bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-50 flex-grow rounded border px-2 py-1 text-sm"
              :class="roomIdFormatError ? 'border-red-500 dark:border-red-400' : 'border-zinc-400 dark:border-zinc-500'"
            />
          </div>
          <input
            v-else
            id="add-id"
            v-model="editProposal.pendingAddition.id"
            type="text"
            class="focusable bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
            :class="idCollidesOnServer ? 'border-red-500 dark:border-red-400' : 'border-zinc-400 dark:border-zinc-500'"
          />
          <p v-if="idCheckPending" class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("id_checking") }}</p>
          <template v-else-if="idCollidesOnServer">
            <p class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ t("error.id_exists_on_server") }}</p>
            <button
              type="button"
              class="text-blue-600 dark:text-blue-300 hover:underline mt-1 text-xs"
              @click="editExistingEntry"
            >
              {{ t("error.edit_existing_instead") }}
            </button>
          </template>
          <p v-else-if="roomIdFormatError" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ t(roomIdFormatError) }}</p>
          <p v-else-if="editProposal.pendingAddition.kind === 'room'" class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">
            {{ t("id_hint.room_segments") }}
          </p>
          <p v-else class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("id_hint.poi") }}</p>
        </div>

        <AddRoomFields v-if="editProposal.pendingAddition.kind === 'room'" />
        <AddBuildingFields v-if="editProposal.pendingAddition.kind === 'building'" />
        <AddPoiFields v-if="editProposal.pendingAddition.kind === 'poi'" />
        <AddEventFields v-if="editProposal.pendingAddition.kind === 'event'" />

        <div v-if="editProposal.pendingAddition.kind !== 'event'">
          <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">
            {{ t("coords_label") }} <span class="text-red-700 dark:text-red-200">*</span>
          </label>
          <LocationPickerInline
            v-model:lat="mapLat"
            v-model:lon="mapLon"
            :initial-lat="mapInitialLat"
            :initial-lon="mapInitialLon"
            :awaiting-selection="!editProposal.pendingAddition.coords.picked"
          />
          <p v-if="editProposal.pendingAddition.coords.picked" class="text-zinc-600 dark:text-zinc-300 mt-1 text-xs">
            {{ editProposal.pendingAddition.coords.lat.toFixed(5) }},
            {{ editProposal.pendingAddition.coords.lon.toFixed(5) }}
          </p>
        </div>
      </template>
    </div>

    <div class="float-right mt-6 flex flex-row-reverse gap-2">
      <Btn variant="primary" size="md" :disabled="!draftIsReady" @click="commitAddition">{{ t("commit") }}</Btn>
      <Btn v-if="editProposal.pendingAddition.kind !== 'event'" variant="secondary" size="md" :disabled="!draftIsReady" @click="commitAndAddImage">{{ t("commit_with_image") }}</Btn>
      <Btn variant="linkButton" size="md" @click="cancelAddition">{{ t("cancel") }}</Btn>
    </div>
  </Modal>
</template>

<i18n lang="yaml">
de:
  title: Neuen Eintrag vorschlagen
  alt_name: Anzeigename
  alt_name_help: Wie der Raum auf der Detailseite angezeigt wird (z.B. {example})
  alt_name_help_example: Hörsaal 1
  kind:
    room: Raum
    building: Gebäude
    poi: POI
    event: Veranstaltung
  id_label: ID
  id_hint:
    room_segments: "Setzt sich aus übergeordnetem Gebäude, Stockwerk und Raumnummer zusammen."
    building: "4-stellige numerische Gebäude-ID (z.B. {example})"
    poi: "Kleinbuchstaben, Ziffern, Bindestrich oder Unterstrich, max. 64 Zeichen"
  room_id_parent_placeholder: Gebäude
  room_id_floor_placeholder: ETAGE
  room_id_number_placeholder: NUMMER
  parent_label: Übergeordneter Eintrag
  parent_placeholder: Suchen…
  coords_label: Koordinaten
  commit: Eintrag hinzufügen
  commit_with_image: …und Bild ergänzen
  cancel: Abbrechen
  id_checking: Prüfe Verfügbarkeit…
  error:
    id_required: Bitte gib eine ID an.
    id_taken: Diese ID ist bereits in dieser Anfrage enthalten.
    id_exists_on_server: Diese ID existiert bereits in Navigatum. Bitte wähle eine andere.
    edit_existing_instead: Stattdessen den vorhandenen Eintrag bearbeiten →
    id_room_format: Etage und Nummer dürfen nur Buchstaben, Zahlen und Bindestriche enthalten.
    id_room_incomplete: Bitte fülle Etage und Nummer aus.
    incomplete: Bitte fülle alle Pflichtfelder aus.
en:
  title: Propose a new entry
  alt_name: Display name
  alt_name_help: How the room is shown on its detail page (e.g. {example})
  alt_name_help_example: Lecture Hall 1
  kind:
    room: Room
    building: Building
    poi: POI
    event: Event
  id_label: ID
  id_hint:
    room_segments: "Composed from the parent building, floor and room number."
    building: "4-digit numeric building id (e.g. {example})"
    poi: "lowercase letters, digits, hyphen or underscore, max 64 chars"
  room_id_parent_placeholder: Building
  room_id_floor_placeholder: FLOOR
  room_id_number_placeholder: NUMBER
  parent_label: Parent entry
  parent_placeholder: Search…
  coords_label: Coordinates
  commit: Add entry
  commit_with_image: …and add an image
  cancel: Cancel
  id_checking: Checking availability…
  error:
    id_required: Please supply an id.
    id_taken: This id is already in this request.
    id_exists_on_server: This id already exists in Navigatum. Please pick a different one.
    edit_existing_instead: Edit the existing entry instead →
    id_room_format: Floor and number may only contain letters, digits and dashes.
    id_room_incomplete: Please fill in both the floor and the number.
    incomplete: Please fill in all required fields.
</i18n>
