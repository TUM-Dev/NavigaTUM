<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import { mdiCalendarStar, mdiDomain, mdiMapMarker, mdiSofa } from "@mdi/js";
import { refDebounced } from "@vueuse/core";
import type { components } from "~/api_types";
import {
  type AdditionDraft,
  type AdditionFieldErrors,
  type AdditionKind,
  additionRegistry,
  buildAddition,
  emptyAdditionDraft,
  validateAddition,
} from "~/composables/additionSchema";
import { useEditProposal } from "~/composables/editProposal";
import { entityPath, isRoutableEntityType } from "~/utils/entityPath";

type FacetFilter = components["schemas"]["FacetFilter"];
type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

const props = withDefaults(
  defineProps<{
    initialDraft?: AdditionDraft;
    embedded?: boolean;
  }>(),
  { embedded: false }
);
const emit = defineEmits<{
  commit: [];
  "commit-with-image": [];
  cancel: [];
}>();

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

// Switching the tab swaps the whole draft for the new variant's empty seed.
// Coords and parent carry across so the user doesn't lose state they already picked.
function pickKind(k: AdditionKind) {
  const previous = editProposal.value.pendingAddition;
  if (previous.kind === k) return;
  const fresh = additionRegistry[k].empty();
  fresh.coords = { ...previous.coords };
  if (previous.kind !== null && previous.kind !== "event" && fresh.kind !== "event") {
    fresh.parent_id = previous.parent_id;
    fresh.parent_name = previous.parent_name;
  }
  editProposal.value.pendingAddition = fresh;
}

// Verify the id against /api/locations/{id}.
// 200 means collision, 404 means free.
// Event ids are content hashed locally and cannot collide, so the check is suppressed for them.
const trimmedId = computed(() => editProposal.value.pendingAddition.id.trim());
const debouncedId = refDebounced(trimmedId, 350);
const fetchingId = ref(false);
const idCollidesOnServer = ref(false);
watch(
  [debouncedId, () => editProposal.value.pendingAddition.kind],
  async ([id, kind], _old, onCleanup) => {
    idCollidesOnServer.value = false;
    if (!id || kind === "event") {
      fetchingId.value = false;
      return;
    }
    const controller = new AbortController();
    // `onCleanup` aborts the prior fetch the instant the watched inputs change again.
    // A slow response can never settle stale state.
    onCleanup(() => controller.abort());
    fetchingId.value = true;
    try {
      const res = await fetch(
        `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(id)}`,
        { credentials: "omit", signal: controller.signal }
      );
      idCollidesOnServer.value = res.ok;
    } catch {
      // Network failure or abort: don't block.
      // The server re-validates on submit.
    } finally {
      fetchingId.value = false;
    }
  }
);
const idCheckPending = computed(() => {
  const id = trimmedId.value;
  if (!id || editProposal.value.pendingAddition.kind === "event") return false;
  return debouncedId.value !== id || fetchingId.value;
});

const allowedParentTypes = computed<readonly FacetFilter[]>(() => {
  const kind = editProposal.value.pendingAddition.kind;
  if (kind === "room") return ["building"];
  // POIs may live inside a site, area, or directly inside a building like a cafeteria.
  // Buildings are parented under sites or areas only.
  if (kind === "poi") return ["site", "building"];
  return ["site"];
});

// Writable computeds let v-models bind to per-kind fields without narrowing in the template.
// Setters are inert on variants that don't carry the field.
// The matching UI block is hidden in that case.
const parentId = computed({
  get: () => {
    const a = editProposal.value.pendingAddition;
    return a.kind !== null && a.kind !== "event" ? a.parent_id : "";
  },
  set: (v: string) => {
    const a = editProposal.value.pendingAddition;
    if (a.kind !== null && a.kind !== "event") a.parent_id = v;
  },
});
const parentName = computed({
  get: () => {
    const a = editProposal.value.pendingAddition;
    return a.kind !== null && a.kind !== "event" ? a.parent_name : "";
  },
  set: (v: string) => {
    const a = editProposal.value.pendingAddition;
    if (a.kind !== null && a.kind !== "event") a.parent_name = v;
  },
});
const roomAltName = computed({
  get: () => {
    const a = editProposal.value.pendingAddition;
    return a.kind === "room" ? a.alt_name : "";
  },
  set: (v: string) => {
    const a = editProposal.value.pendingAddition;
    if (a.kind === "room") a.alt_name = v;
  },
});

// When the user picks a parent, fetch its details to pre-fill the map centre.
// We auto-mark coords as ready so the user saves a click and can still drag to refine.
const parentLookupUrl = computed(() => {
  const pid = parentId.value;
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
  watch: [parentId],
});

// The room code prefix isn't always the entry id.
// Joined buildings have textual ids like `mi`, while their TUMonline code is e.g. `5510`.
// Pick the first 4 digit alias as the prefix.
const roomParentPrefix = computed(() => {
  const a = editProposal.value.pendingAddition;
  if (a.kind !== "room") return "";
  const pid = a.parent_id.trim();
  if (!pid) return "";
  if (FOUR_DIGIT_PREFIX.test(pid)) return pid;
  const aliases = parentDetails.value?.aliases ?? [];
  const numeric = aliases.find((alias) => FOUR_DIGIT_PREFIX.test(alias));
  return numeric ?? pid;
});

// Floors known on the parent, used by the TUMonline room code as its floor segment.
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

// Room ids follow PARENT.FLOOR.NUMBER.
// The parent segment is auto filled and disabled so users can't desync it from the chosen parent.
// Floor and number flow through the Zod schema like any other field once we compose the id.
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
// Commit and cancel both replace `pendingAddition` with `emptyAdditionDraft()`, so the segment refs need to follow.
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

function displayNameOf(draft: AdditionDraft): string {
  if (draft.kind === null) return "";
  if (draft.kind === "room") return draft.alt_name;
  return draft.name;
}

type Addition = ReturnType<typeof buildAddition>;

function validateAndBuild():
  | { id: string; displayName: string; addition: NonNullable<Addition> }
  | null {
  localError.value = "";
  const draft = editProposal.value.pendingAddition;
  const id = draft.id.trim();
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
  const addition = buildAddition(draft);
  if (!addition) {
    localError.value = t("error.incomplete");
    return null;
  }
  return { id, addition, displayName: displayNameOf(draft) || id };
}

function commitDraft(): { id: string; displayName: string } | null {
  const built = validateAndBuild();
  if (!built) return null;
  // OpenAPI types are readonly; round-trip through JSON for a DeepWritable clone to match the LimitedHashMap value type.
  editProposal.value.data.additions[built.id] = JSON.parse(JSON.stringify(built.addition));
  editProposal.value.pendingAddition = emptyAdditionDraft();
  return { id: built.id, displayName: built.displayName };
}

function clearPending() {
  editProposal.value.pendingAddition = emptyAdditionDraft();
  localError.value = "";
}

defineExpose({
  validateAndBuild,
  clearPending,
  draftIsReady,
});

function commitAddition() {
  if (!commitDraft()) return;
  emit("commit");
}

function commitAndAddImage() {
  const result = commitDraft();
  if (!result) return;
  // Point the existing image upload flow at the freshly added entry.
  // The server applies additions before edits in a single request.
  // So an image edit keyed by this id resolves correctly.
  editProposal.value.selected = { id: result.id, name: result.displayName };
  emit("commit-with-image");
}

function cancelAddition() {
  editProposal.value.pendingAddition = emptyAdditionDraft();
  emit("cancel");
}

const localePath = useLocalePath();
async function editExistingEntry() {
  const id = editProposal.value.pendingAddition.id.trim();
  if (!id) return;
  editProposal.value.pendingAddition = emptyAdditionDraft();
  editProposal.value.selected = { id, name: null };
  // Open the edit modal once we land on the entry's detail page.
  editProposal.value.addOpen = false;
  editProposal.value.open = true;
  // Resolve the entity's type up front so we land on its canonical /{type}/{id} path directly.
  // Otherwise we bounce through the /view/{id} redirect.
  // On any failure we fall back to /view/{id}, which the server redirects to the canonical path.
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

// Coordinate model for the inline picker.
// Centred on TUM main campus until the user picks a parent or moves the marker themselves.
// Picking a parent recenters the map on the parent's coords.
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

// Share id validation state with per kind sub components.
// AddBuildingFields, for example, renders the id input itself inside its Identifiers fieldset.
provide("addProposal:idValidation", {
  pending: idCheckPending,
  collides: idCollidesOnServer,
});

// Seed at setup so SSR and the initial client render see the right kind; doing
// this in onMounted flashes the default tab and kind-gated elements.
editProposal.value.pendingAddition = props.initialDraft ?? emptyAdditionDraft();
localError.value = "";
onBeforeUnmount(() => {
  editProposal.value.pendingAddition = emptyAdditionDraft();
  localError.value = "";
});
</script>

<template>
  <div class="flow-root">
    <Toast v-if="localError" id="add-proposal-local-error" class="mb-3" :msg="localError" level="error" />

    <div class="space-y-3">
      <TabGroup :selected-index="kindIndex < 0 ? 0 : kindIndex" :default-index="0">
        <TabList class="bg-zinc-100 dark:bg-zinc-800 flex space-x-1 rounded-lg p-1">
          <Tab v-for="opt in kindOptions" :key="opt.value" as="template">
            <SegmentedTab :selected="kindIndex === kindOptions.indexOf(opt)" class="w-full px-3 py-2.5" @click="pickKind(opt.value)">
              <div class="flex items-center justify-center gap-2">
                <span class="hidden md:inline-flex">
                  <MdiIcon :path="opt.icon" :size="16" />
                </span>
                {{ t(`kind.${opt.value}`) }}
              </div>
            </SegmentedTab>
          </Tab>
        </TabList>
      </TabGroup>

      <template v-if="editProposal.pendingAddition.kind">
        <div v-if="editProposal.pendingAddition.kind !== 'event'">
          <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">
            {{ t("parent_label") }} <span class="text-red-700 dark:text-red-200">*</span>
          </label>
          <EntryPicker
            v-model:selected-id="parentId"
            v-model:selected-name="parentName"
            :allowed-types="allowedParentTypes"
            :placeholder="t('parent_placeholder')"
          />
        </div>

        <!-- Room name sits between parent and id so the user works top down. -->
        <!-- The flow reads as where, then what's it called, then its id. -->
        <div v-if="editProposal.pendingAddition.kind === 'room'">
          <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-room-alt-name">
            {{ t("alt_name") }} <span class="text-red-700 dark:text-red-200">*</span>
          </label>
          <input
            id="add-room-alt-name"
            v-model="roomAltName"
            type="text"
            class="focusable input-field w-full rounded border px-2 py-1 text-sm"
          />
          <I18nT keypath="alt_name_help" tag="p" class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">
            <template #example>
              <code class="font-mono">{{ t("alt_name_help_example") }}</code>
            </template>
          </I18nT>
        </div>

        <!-- Buildings render the id input inside AddBuildingFields. -->
        <!-- Events derive their id from the image. -->
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

    <div v-if="!props.embedded" class="float-right mt-6 flex flex-row-reverse gap-2">
      <Btn variant="primary" size="md" :disabled="!draftIsReady" @click="commitAddition">{{ t("commit") }}</Btn>
      <Btn v-if="editProposal.pendingAddition.kind !== 'event'" variant="secondary" size="md" :disabled="!draftIsReady" @click="commitAndAddImage">{{ t("commit_with_image") }}</Btn>
      <Btn variant="linkButton" size="md" @click="cancelAddition">{{ t("cancel") }}</Btn>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  alt_name: Anzeigename
  alt_name_help: Wie der Raum auf der Detailseite angezeigt wird (z.B. {example})
  alt_name_help_example: Hörsaal 1
  kind:
    room: Raum
    building: Gebäude
    poi: POI
    event: Event
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
