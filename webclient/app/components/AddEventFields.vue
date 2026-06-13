<script setup lang="ts">
import {
  Combobox,
  ComboboxButton,
  ComboboxInput,
  ComboboxOption,
  ComboboxOptions,
} from "@headlessui/vue";
import {
  mdiAlert,
  mdiCheck,
  mdiClose,
  mdiImage,
  mdiInformation,
  mdiUnfoldMoreHorizontal,
  mdiUpdate,
} from "@mdi/js";
import { useDebounceFn, useDropZone, useFileDialog, useObjectUrl } from "@vueuse/core";
import type { components } from "~/api_types";
import type { EventPreviewPopup } from "~/components/EventPreviewMap.vue";
import {
  type AdditionFieldErrors,
  additionRegistry,
  type EventDraft,
  eventDraftFromEntry,
  eventSourceImageUrl,
  validateAddition,
} from "~/composables/additionSchema";
import { useEditProposal } from "~/composables/editProposal";
import { type OrgOption, useKnownOrgs } from "~/composables/useKnownOrgs";
import {
  findDuplicateEventByName,
  formatEventDateRange,
  wallTimeToRfc3339,
} from "~/utils/datetime";
import { clampCropOffset, cropToBlob, HEADER_TARGET, THUMB_TARGET } from "~/utils/imageCrop";

type EventEntry = components["schemas"]["EventEntry"];
type SearchResponse = components["schemas"]["SearchResponse"];

const editProposal = useEditProposal();
const runtimeConfig = useRuntimeConfig();
const { t, locale } = useI18n({ useScope: "local" });

// The parent only mounts this component when `kind === "event"`, so the narrowing cast is safe
// and saves every binding from re-checking the discriminant.
const draft = computed(() => editProposal.value.pendingAddition as EventDraft);
const fieldErrors = computed<AdditionFieldErrors>(() => validateAddition(draft.value));
function errorFor(path: string): string | null {
  const key = fieldErrors.value[path];
  return key ? t(key) : null;
}

const knownOrgs = useKnownOrgs();
const { pending: orgsLoading, error: orgsError, refresh: reloadOrgs } = knownOrgs;
const orgQuery = ref("");
const filteredOrgs = computed<OrgOption[]>(() => knownOrgs.filter(orgQuery.value));
const orgTruncated = computed(() => filteredOrgs.value.length >= knownOrgs.maxResults);
const selectedOrg = computed<OrgOption | null>({
  get: () => knownOrgs.byId(draft.value.organising_org_id),
  set: (o) => {
    draft.value.organising_org_id = o?.org_id ?? null;
  },
});

const DEFAULT_LAT = 48.149;
const DEFAULT_LON = 11.568;
const mapLat = computed({
  get: () => draft.value.coords.lat || DEFAULT_LAT,
  set: (v: number) => {
    draft.value.coords.lat = v;
    draft.value.coords.picked = true;
  },
});
const mapLon = computed({
  get: () => draft.value.coords.lon || DEFAULT_LON,
  set: (v: number) => {
    draft.value.coords.lon = v;
    draft.value.coords.picked = true;
  },
});

const VALID_IMAGE_TYPES = ["image/jpeg", "image/png", "image/gif", "image/webp"];
const MAX_FILE_BYTES = 10 * 1024 * 1024;
const fileError = ref("");

// `useObjectUrl` owns the URL lifetime: it issues one when the source File/Blob changes and revokes
// the old one on the next change or on unmount, so the manual createObjectURL/revoke bookkeeping
// drops away. The bitmap is decoded once via `createImageBitmap` so canvas crops don't have to wait
// on a hidden <img> probe to fire `onload`.
const sourceFile = shallowRef<File | null>(null);
const sourceBitmap = shallowRef<ImageBitmap | null>(null);
const previewUrl = useObjectUrl(sourceFile);

// Thumb blob feeds the map-marker preview; useObjectUrl keeps its URL synced to the blob's lifetime.
// The header crop is offset-only - its preview lives inside ImageCropper, so no blob here.
const thumbBlob = shallowRef<Blob | null>(null);
const thumbUrl = useObjectUrl(thumbBlob);

// `onCleanup` cancels the previous run so a slow encode never overwrites a newer offset - this is
// what the old `makeCropPreview` race-token bookkeeping was emulating.
watchEffect(async (onCleanup) => {
  const img = sourceBitmap.value;
  const w = draft.value.image_width;
  const h = draft.value.image_height;
  const offset = draft.value.image_thumb_offset;
  if (!img || !w || !h) {
    thumbBlob.value = null;
    return;
  }
  let cancelled = false;
  onCleanup(() => {
    cancelled = true;
  });
  const blob = await cropToBlob(img, w, h, THUMB_TARGET, offset);
  if (!cancelled) thumbBlob.value = blob;
});

function fileToBase64(file: File): Promise<string | null> {
  return new Promise((resolve) => {
    const reader = new FileReader();
    reader.onload = () => resolve(((reader.result as string) ?? "").split(",")[1] ?? null);
    reader.onerror = () => resolve(null);
    reader.readAsDataURL(file);
  });
}

// `adopt` is set only when re-fetching a based-on event's own image, so its saved crop survives.
async function processFile(file: File, adopt = false): Promise<void> {
  // Picking or unlinking a based-on event swaps the draft object while the image may
  // still be decoding; writes must land on the draft this file was selected for.
  const target = draft.value;
  fileError.value = "";
  if (!VALID_IMAGE_TYPES.includes(file.type)) {
    fileError.value = t("image_invalid_type");
    return;
  }
  if (file.size > MAX_FILE_BYTES) {
    fileError.value = t("image_too_large");
    return;
  }
  let bitmap: ImageBitmap;
  try {
    bitmap = await createImageBitmap(file);
  } catch {
    fileError.value = t("image_read_error");
    return;
  }
  const [base64, buffer] = await Promise.all([fileToBase64(file), file.arrayBuffer()]);
  if (!base64) {
    bitmap.close();
    fileError.value = t("image_read_error");
    return;
  }
  // Content-addressed key, matching `event_{hash(img)}` consumed by the server (event.rs). The full
  // sha256 is overkill for a filename; 16 hex chars (64 bits) keep collisions negligible yet tidy.
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", buffer));
  const hash = Array.from(digest, (b) => b.toString(16).padStart(2, "0"))
    .join("")
    .slice(0, 16);

  if (draft.value !== target) {
    bitmap.close();
    return;
  }
  sourceBitmap.value?.close();
  sourceBitmap.value = bitmap;
  sourceFile.value = file;
  draft.value.image = { base64, fileName: file.name || "event-image" };
  draft.value.image_width = bitmap.width;
  draft.value.image_height = bitmap.height;
  // A new image recentres both crops; adopting the based-on image restores its saved crop,
  // clamped to the loaded dimensions so a since-replaced source can't push it out of bounds.
  const basedOn = adopt ? draft.value.based_on : null;
  draft.value.image_thumb_offset = basedOn
    ? clampCropOffset(bitmap.width, bitmap.height, THUMB_TARGET, basedOn.thumb_offset)
    : 0;
  draft.value.image_header_offset = basedOn
    ? clampCropOffset(bitmap.width, bitmap.height, HEADER_TARGET, basedOn.header_offset)
    : 0;
  // In locked update mode the adopted key stays the identity, whatever the image.
  if (!draft.value.based_on) draft.value.id = `event_${hash}`;
}

const {
  open: openFileDialog,
  onChange: onFileDialogChange,
  reset: resetFileDialog,
} = useFileDialog({ accept: "image/*", multiple: false });
onFileDialogChange((files) => {
  const f = files?.[0];
  if (f) void processFile(f);
});

const dropArea = ref<HTMLElement>();
const { isOverDropZone } = useDropZone(dropArea, {
  onDrop: (files) => {
    const f = files?.[0];
    if (f) void processFile(f);
  },
});

function removeImage(): void {
  sourceBitmap.value?.close();
  sourceBitmap.value = null;
  sourceFile.value = null;
  draft.value.image = null;
  draft.value.image_width = null;
  draft.value.image_height = null;
  draft.value.image_thumb_offset = 0;
  draft.value.image_header_offset = 0;
  draft.value.id = draft.value.based_on?.id ?? "";
  fileError.value = "";
  resetFileDialog();
}

const prefillError = ref("");

function resetLocalImageState(): void {
  sourceBitmap.value?.close();
  sourceBitmap.value = null;
  sourceFile.value = null;
  fileError.value = "";
  prefillError.value = "";
  resetFileDialog();
}

// True once the user unlinks from an adopted event but keeps the copied fields: the
// proposal then creates a new event rather than updating the original, and the banner
// says so. Local to the component; `pendingAddition` outlives it in useEditProposal.
const proposingNewFromExisting = ref(false);

async function adoptEvent(entry: EventEntry): Promise<void> {
  // The picked event becomes the new basis; a half-typed draft is intentionally discarded.
  proposingNewFromExisting.value = false;
  resetLocalImageState();
  editProposal.value.pendingAddition = eventDraftFromEntry(entry, Date.now());
  try {
    const res = await fetch(eventSourceImageUrl(entry.id), {
      credentials: "omit",
    });
    if (!res.ok) throw new Error(`unexpected status ${res.status}`);
    const blob = await res.blob();
    // The user may have unlinked or re-picked while the image was downloading.
    if (draft.value.based_on?.id !== entry.id) return;
    await processFile(
      new File([blob], `${entry.id}_0.webp`, { type: blob.type || "image/webp" }),
      true
    );
  } catch {
    if (draft.value.based_on?.id === entry.id) prefillError.value = t("prefill_image_failed");
  }
}

// The copied fields stay so the user can tweak them into a new event; only the upsert
// lock drops. draft.id still equals the content hash of the prefilled image, so a new
// image auto-recomputes a fresh key (processFile's `!based_on` branch).
function unlinkBasedOn(): void {
  draft.value.based_on = null;
  proposingNewFromExisting.value = true;
}

function clearAllFields(): void {
  proposingNewFromExisting.value = false;
  resetLocalImageState();
  editProposal.value.pendingAddition = additionRegistry.event.empty();
}

// The form's top region is a three-state switch: updating an adopted event, proposing a
// new one seeded from a former adoption, or searching for an event to adopt. `based_on`
// is the real data (id/name/dates, submit-label); the flag only splits the two null cases.
const eventPickerState = computed<"updating" | "proposing-new" | "searching">(() => {
  if (draft.value.based_on) return "updating";
  if (proposingNewFromExisting.value) return "proposing-new";
  return "searching";
});

const basedOnName = computed(() => draft.value.based_on?.name ?? "");

// "Zuletzt" only fits a finished edition; an upcoming or running one needs its own framing.
const basedOnDateLabel = computed(() => {
  const basedOn = draft.value.based_on;
  if (!basedOn) return "";
  const range = formatEventDateRange(
    basedOn.starts_at,
    basedOn.ends_at,
    locale.value === "de" ? "de" : "en"
  );
  const now = Date.now();
  const start = Date.parse(basedOn.starts_at);
  const end = Date.parse(basedOn.ends_at);
  if (Number.isNaN(start) || Number.isNaN(end) || end < now)
    return t("based_on_dates_ended", [range]);
  if (start <= now) return t("based_on_dates_ongoing", [range]);
  return t("based_on_dates_upcoming", [range]);
});

// Safety net for the freehand path: the adopt-an-event combobox is optional, so a user can type a
// name that already exists and unknowingly create a duplicate. This runs a second, independent
// event search keyed on the typed name (the combobox has its own query, so the two can't be shared)
// and offers a one-click switch into locked update mode. Only while searching - once an event is
// adopted or explicitly unlinked, the warning would be noise.
const duplicateQuery = ref("");
const applyDuplicateQuery = useDebounceFn((value: string) => {
  duplicateQuery.value = value.trim();
}, 200);
watch(() => (eventPickerState.value === "searching" ? draft.value.name : ""), applyDuplicateQuery);

const duplicateSearchUrl = computed(() => {
  if (duplicateQuery.value.length < 2) return null;
  const params = new URLSearchParams();
  params.set("q", duplicateQuery.value);
  // `type=event` enables the default-off events facet; mirror AddEventSearch's plain-text request.
  params.append("type", "event");
  params.set("limit_events", "8");
  params.set("pre_highlight", "");
  params.set("post_highlight", "");
  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});

const duplicateMatches = ref<readonly EventEntry[]>([]);
let duplicateSearchCounter = 0;
watch(duplicateSearchUrl, async (url) => {
  if (!url) {
    duplicateMatches.value = [];
    return;
  }
  const ticket = ++duplicateSearchCounter;
  try {
    const res = await $fetch<SearchResponse>(url, { credentials: "omit" });
    if (ticket !== duplicateSearchCounter) return;
    duplicateMatches.value = res.sections.flatMap((s) => (s.facet === "events" ? s.entries : []));
  } catch {
    if (ticket === duplicateSearchCounter) duplicateMatches.value = [];
  }
});

const duplicateEvent = computed<EventEntry | null>(() => {
  if (eventPickerState.value !== "searching") return null;
  return findDuplicateEventByName(duplicateMatches.value, draft.value.name, Date.now());
});

// Release the bitmap's GPU/CPU buffers promptly; useObjectUrl handles the URL revokes itself.
onScopeDispose(() => {
  sourceBitmap.value?.close();
});

const showPreview = computed(() => Boolean(thumbUrl.value) && draft.value.coords.picked);

const previewEvent = computed<EventPreviewPopup | null>(() => {
  const org = selectedOrg.value;
  const startsAt = wallTimeToRfc3339(draft.value.starts_at);
  const endsAt = wallTimeToRfc3339(draft.value.ends_at);
  if (!org || !startsAt || !endsAt || !draft.value.name || !previewUrl.value) return null;
  return {
    name: draft.value.name,
    description: draft.value.description,
    startsAt,
    endsAt,
    orgCode: org.code,
    orgNameDe: org.nameDe,
    orgNameEn: org.nameEn,
    imageSrc: previewUrl.value,
    imageAuthor: draft.value.image_author,
  };
});
</script>

<template>
  <div class="space-y-3">
    <div
      v-if="eventPickerState === 'updating'"
      class="bg-amber-50 dark:bg-amber-900 border-amber-300 dark:border-amber-600 flex items-start gap-2 rounded border p-3"
      data-cy="event-update-banner"
    >
      <MdiIcon :path="mdiUpdate" :size="18" class="text-amber-600 dark:text-amber-300 mt-0.5 flex-shrink-0" />
      <div class="flex-grow">
        <p class="text-amber-900 dark:text-amber-50 text-sm font-medium">{{ t("based_on_title", [basedOnName]) }}</p>
        <p class="text-amber-800 dark:text-amber-100 mt-0.5 text-xs">{{ basedOnDateLabel }}</p>
        <p class="text-amber-800 dark:text-amber-100 mt-0.5 text-xs">{{ t("based_on_help") }}</p>
        <p v-if="!draft.starts_at && !draft.ends_at" class="text-amber-800 dark:text-amber-100 mt-0.5 text-xs font-medium">
          {{ t("based_on_dates_cleared") }}
        </p>
        <button
          type="button"
          class="text-amber-900 dark:text-amber-50 hover:no-underline mt-1 cursor-pointer text-xs underline"
          @click="unlinkBasedOn"
        >
          {{ t("based_on_unlink") }}
        </button>
      </div>
    </div>
    <div
      v-else-if="eventPickerState === 'proposing-new'"
      class="bg-blue-50 dark:bg-blue-900 border-blue-200 dark:border-blue-700 flex items-start gap-2 rounded border p-3"
      data-cy="event-new-banner"
    >
      <MdiIcon :path="mdiInformation" :size="18" class="text-blue-500 dark:text-blue-400 mt-0.5 flex-shrink-0" />
      <div class="flex-grow">
        <p class="text-blue-800 dark:text-blue-100 text-sm font-medium">{{ t("new_event_title") }}</p>
        <p class="text-blue-700 dark:text-blue-200 mt-0.5 text-xs">{{ t("new_event_help") }}</p>
        <button
          type="button"
          class="text-blue-800 dark:text-blue-100 hover:no-underline mt-1 cursor-pointer text-xs underline"
          @click="clearAllFields"
        >
          {{ t("new_event_clear") }}
        </button>
      </div>
    </div>
    <AddEventSearch v-else @pick="adoptEvent" />
    <p v-if="prefillError" class="text-red-700 dark:text-red-200 text-xs">{{ prefillError }}</p>

    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-name">
        {{ t("name") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <input
        id="add-event-name"
        v-model="draft.name"
        type="text"
        :placeholder="t('name_placeholder')"
        class="focusable input-field w-full rounded border px-2 py-1 text-sm"
        :class="errorFor('name') ? '!border-red-500 dark:!border-red-400' : ''"
      />
      <p v-if="errorFor('name')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("name") }}</p>
      <div
        v-if="duplicateEvent"
        class="bg-amber-50 dark:bg-amber-900 border-amber-300 dark:border-amber-600 mt-1 flex items-start gap-2 rounded border p-3"
        data-cy="event-duplicate-warning"
      >
        <MdiIcon :path="mdiAlert" :size="18" class="text-amber-600 dark:text-amber-300 mt-0.5 flex-shrink-0" />
        <div class="flex-grow">
          <p class="text-amber-900 dark:text-amber-50 text-sm font-medium">{{ t("duplicate_warning", [duplicateEvent.name]) }}</p>
          <button
            type="button"
            class="text-amber-900 dark:text-amber-50 hover:no-underline mt-1 cursor-pointer text-xs underline"
            @click="adoptEvent(duplicateEvent)"
          >
            {{ t("duplicate_update") }}
          </button>
        </div>
      </div>
    </div>

    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-description">
        {{ t("description") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <textarea
        id="add-event-description"
        v-model="draft.description"
        rows="3"
        :placeholder="t('description_placeholder')"
        class="focusable input-field w-full resize-y rounded border px-2 py-1 text-sm"
        :class="errorFor('description') ? '!border-red-500 dark:!border-red-400' : ''"
      />
      <p v-if="errorFor('description')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("description") }}</p>
      <p class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("description_help") }}</p>
    </div>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <div>
        <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-start">
          {{ t("starts_at") }} <span class="text-red-700 dark:text-red-200">*</span>
        </label>
        <input
          id="add-event-start"
          v-model="draft.starts_at"
          type="datetime-local"
          class="focusable bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
          :class="errorFor('starts_at') ? 'border-red-500 dark:border-red-400' : 'border-zinc-400 dark:border-zinc-500'"
        />
        <p v-if="errorFor('starts_at')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("starts_at") }}</p>
      </div>
      <div>
        <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-end">
          {{ t("ends_at") }} <span class="text-red-700 dark:text-red-200">*</span>
        </label>
        <input
          id="add-event-end"
          v-model="draft.ends_at"
          type="datetime-local"
          class="focusable bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
          :class="errorFor('ends_at') ? 'border-red-500 dark:border-red-400' : 'border-zinc-400 dark:border-zinc-500'"
        />
        <p v-if="errorFor('ends_at')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("ends_at") }}</p>
      </div>
    </div>
    <p class="text-zinc-500 dark:text-zinc-400 -mt-1 text-xs">{{ t("timezone_help") }}</p>

    <div>
      <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("organising_org") }} <span class="text-red-700 dark:text-red-200">*</span></span>
      <Combobox v-model="selectedOrg" :nullable="true" by="org_id">
        <div class="relative">
          <div
            class="bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 focus-within:border-blue-500 dark:focus-within:border-blue-400 relative flex w-full items-center rounded-md border text-left text-sm"
          >
            <ComboboxInput
              class="text-zinc-900 dark:text-zinc-50 w-full rounded-md border-none bg-transparent py-2 pl-3 pr-10 text-sm focus:outline-none"
              :display-value="(o: unknown) => (o as OrgOption | null)?.label ?? ''"
              :placeholder="t('organising_org_placeholder')"
              @change="orgQuery = ($event.target as HTMLInputElement).value"
            />
            <ComboboxButton class="absolute inset-y-0 right-0 flex items-center pr-2">
              <MdiIcon :path="mdiUnfoldMoreHorizontal" :size="20" class="text-zinc-600 dark:text-zinc-300" aria-hidden="true" />
            </ComboboxButton>
          </div>
          <Transition leave-active-class="transition duration-100 ease-in" leave-from-class="opacity-100" leave-to-class="opacity-0">
            <ComboboxOptions
              class="ring-black/5 dark:ring-white/5 bg-zinc-50 dark:bg-zinc-900 absolute z-30 mt-1 max-h-72 w-full overflow-auto rounded-md py-1 shadow-lg ring-1 focus:outline-none"
            >
              <p v-if="orgsLoading" class="text-zinc-500 dark:text-zinc-400 px-3 py-2 text-sm">
                {{ t("organising_org_loading") }}
              </p>
              <div v-else-if="orgsError" class="px-3 py-2 text-sm">
                <p class="text-red-700 dark:text-red-200">{{ t("organising_org_load_error") }}</p>
                <button
                  type="button"
                  class="text-blue-600 dark:text-blue-300 mt-1 cursor-pointer underline"
                  @mousedown.prevent="reloadOrgs()"
                >
                  {{ t("organising_org_retry") }}
                </button>
              </div>
              <p v-else-if="filteredOrgs.length === 0" class="text-zinc-500 dark:text-zinc-400 px-3 py-2 text-sm">
                {{ t("organising_org_no_results") }}
              </p>
              <ComboboxOption
                v-for="o in filteredOrgs"
                :key="o.org_id"
                v-slot="{ active, selected }"
                :value="o"
                as="template"
              >
                <li
                  class="relative cursor-pointer select-none py-2 pl-3 pr-8"
                  :class="active ? 'bg-blue-100 dark:bg-blue-800 text-blue-900 dark:text-blue-50' : 'text-zinc-900 dark:text-zinc-50'"
                >
                  <div class="flex items-baseline gap-3">
                    <OrgOptionContent :org="o" :emphasised="selected" />
                  </div>
                  <span v-if="selected" class="text-blue-600 dark:text-blue-300 absolute inset-y-0 right-0 flex items-center pr-2">
                    <MdiIcon :path="mdiCheck" :size="16" aria-hidden="true" />
                  </span>
                </li>
              </ComboboxOption>
              <p v-if="orgTruncated" class="text-zinc-400 dark:text-zinc-500 border-zinc-200 dark:border-zinc-700 mt-1 border-t px-3 py-2 text-xs">
                {{ t("organising_org_truncated", [knownOrgs.maxResults]) }}
              </p>
            </ComboboxOptions>
          </Transition>
        </div>
      </Combobox>
      <p v-if="errorFor('organising_org_id')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("organising_org_id") }}</p>
    </div>

    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">
        {{ t("coords") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <LocationPickerInline
        v-model:lat="mapLat"
        v-model:lon="mapLon"
        :initial-lat="mapLat"
        :initial-lon="mapLon"
        :awaiting-selection="!draft.coords.picked"
        container-class="h-44"
      />
      <p v-if="draft.coords.picked" class="text-zinc-600 dark:text-zinc-300 mt-1 text-xs">
        {{ draft.coords.lat.toFixed(5) }}, {{ draft.coords.lon.toFixed(5) }}
      </p>
      <p v-else-if="errorFor('coords.picked')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("coords.picked") }}</p>
    </div>

    <div>
      <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("image") }} <span class="text-red-700 dark:text-red-200">*</span></span>
      <div
        ref="dropArea"
        class="cursor-pointer rounded border-2 border-dashed p-4 text-center text-sm transition-colors"
        :class="[
          isOverDropZone
            ? 'border-blue-500 dark:border-blue-400 bg-blue-50 dark:bg-blue-900'
            : 'border-zinc-300 dark:border-zinc-600 hover:border-blue-400 dark:hover:border-blue-500',
          errorFor('image') ? 'border-red-500 dark:border-red-400' : '',
        ]"
        @click="openFileDialog()"
      >
        <template v-if="draft.image">
          <p class="text-blue-700 dark:text-blue-200 font-medium">{{ draft.image.fileName }}</p>
          <p v-if="draft.image_width && draft.image_height" class="text-zinc-500 dark:text-zinc-400 text-xs">
            {{ draft.image_width }}×{{ draft.image_height }}px
          </p>
          <button type="button" class="text-blue-600 dark:text-blue-300 hover:underline mt-1 inline-flex cursor-pointer items-center gap-1 text-xs" @click.stop="removeImage">
            <MdiIcon :path="mdiClose" :size="12" /> {{ t("image_remove") }}
          </button>
        </template>
        <template v-else>
          <MdiIcon :path="mdiImage" :size="32" class="text-zinc-400 dark:text-zinc-500 mx-auto" />
          <p class="text-zinc-600 dark:text-zinc-300 mt-1">{{ t("image_drop") }}</p>
          <p class="text-zinc-500 dark:text-zinc-400 text-xs">{{ t("image_hint") }}</p>
        </template>
      </div>
      <p v-if="fileError" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ fileError }}</p>
      <p v-else-if="errorFor('image')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("image") }}</p>
    </div>

    <div
      v-if="previewUrl && draft.image_width && draft.image_height"
      class="grid grid-cols-1 gap-3 sm:grid-cols-2 sm:items-start"
    >
      <div>
        <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("crop_thumb") }}</span>
        <p class="text-zinc-500 dark:text-zinc-400 mb-1 text-xs">{{ t("crop_thumb_help") }}</p>
        <ImageCropper
          v-model="draft.image_thumb_offset"
          :image-url="previewUrl"
          :width="draft.image_width"
          :height="draft.image_height"
          :target="THUMB_TARGET"
        />
      </div>

      <div>
        <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("crop_header") }}</span>
        <p class="text-zinc-500 dark:text-zinc-400 mb-1 text-xs">{{ t("crop_header_help") }}</p>
        <ImageCropper
          v-model="draft.image_header_offset"
          :image-url="previewUrl"
          :width="draft.image_width"
          :height="draft.image_height"
          :target="HEADER_TARGET"
        />
      </div>
    </div>

    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-image-author">
        {{ t("image_author") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <input
        id="add-event-image-author"
        v-model="draft.image_author"
        type="text"
        :placeholder="t('image_author_placeholder')"
        class="focusable input-field w-full rounded border px-2 py-1 text-sm"
        :class="errorFor('image_author') ? '!border-red-500 dark:!border-red-400' : ''"
      />
      <p v-if="errorFor('image_author')" class="text-red-700 dark:text-red-200 mt-1 text-xs">{{ errorFor("image_author") }}</p>
    </div>

    <div class="bg-blue-50 dark:bg-blue-900 border-blue-200 dark:border-blue-700 flex items-start gap-2 rounded border p-3">
      <MdiIcon :path="mdiInformation" :size="18" class="text-blue-500 dark:text-blue-400 mt-0.5 flex-shrink-0" />
      <div>
        <p class="text-blue-800 dark:text-blue-100 text-sm font-medium">{{ t("license_info_title") }}</p>
        <p class="text-blue-700 dark:text-blue-200 mt-0.5 text-xs">{{ t("license_info_description") }}</p>
      </div>
    </div>

    <div v-if="showPreview">
      <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("preview") }}</span>
      <p class="text-zinc-500 dark:text-zinc-400 mb-1 text-xs">{{ t("preview_help") }}</p>
      <EventPreviewMap
        :lat="draft.coords.lat"
        :lon="draft.coords.lon"
        :image-url="thumbUrl"
        :popup="previewEvent"
      />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  based_on_title: Du aktualisierst "{0}"
  based_on_dates_ended: "Zuletzt: {0}"
  based_on_dates_ongoing: "Läuft gerade: {0}"
  based_on_dates_upcoming: "Nächster Termin: {0}"
  based_on_help: Dein Vorschlag ersetzt die bestehende Veranstaltung. Alle Felder können angepasst werden.
  based_on_dates_cleared: Die letzte Ausgabe ist vorbei - bitte trage die neuen Termine ein.
  based_on_unlink: Stattdessen eine neue Veranstaltung vorschlagen
  new_event_title: Du schlägst eine neue Veranstaltung vor.
  new_event_help: Die ursprüngliche Veranstaltung bleibt unverändert.
  new_event_clear: Alle Felder leeren
  duplicate_warning: Eine Veranstaltung namens "{0}" existiert bereits.
  duplicate_update: Stattdessen diese aktualisieren
  prefill_image_failed: Das bisherige Bild konnte nicht geladen werden - bitte lade ein neues hoch.
  name: Name
  name_placeholder: z.B. GARNIX Festival
  description: Beschreibung
  description_placeholder: Was passiert bei dieser Veranstaltung?
  description_help: Wird unverändert angezeigt - bitte in der Sprache der Veranstaltung.
  starts_at: Beginn
  ends_at: Ende
  timezone_help: Zeiten in der Zeitzone Europe/Berlin.
  organising_org: Veranstaltende Organisation
  organising_org_placeholder: Organisation suchen…
  organising_org_no_results: Keine passende Organisation gefunden
  organising_org_loading: Organisationen werden geladen…
  organising_org_load_error: Die Organisationen konnten nicht geladen werden.
  organising_org_retry: Erneut versuchen
  organising_org_truncated: "Nur die ersten {0} Treffer werden angezeigt - bitte weiter eingrenzen."
  coords: Ort
  image: Bild
  image_drop: Bild hierher ziehen oder klicken
  image_hint: JPG, PNG, GIF oder WebP, mind. 256px, max. 10MB
  image_remove: Entfernen
  image_invalid_type: Nicht unterstütztes Dateiformat.
  image_too_large: Die Datei ist größer als 10MB.
  image_read_error: Das Bild konnte nicht gelesen werden.
  image_author: Urheber:in des Bildes
  image_author_placeholder: z.B. Studentische Vertretung TUM
  license_info_title: CC BY 4.0 - Frei zu verwenden mit Namensnennung
  license_info_description: Alle hochgeladenen Bilder werden unter der CC BY 4.0 Lizenz veröffentlicht. Das bedeutet, jeder kann das Bild verwenden, solange du als Urheber:in genannt wirst.
  crop_thumb: Marker-Ausschnitt
  crop_thumb_help: Quadratischer Ausschnitt für den Foto-Marker auf der Karte.
  crop_header: Header-Ausschnitt
  crop_header_help: Breiter Ausschnitt (512×210) für die Kopfzeile.
  preview: Vorschau auf der Karte
  preview_help: So erscheint die Veranstaltung auf der Karte - als Foto-Marker mit Popup.
  error:
    name_required: Bitte gib einen Namen an.
    name_too_long: Der Name ist zu lang (max. 200 Zeichen).
    description_required: Bitte gib eine Beschreibung an.
    starts_at_required: Bitte gib einen gültigen Beginn an.
    ends_at_required: Bitte gib ein gültiges Ende an.
    event_ends_before_start: Das Ende muss nach dem Beginn liegen.
    event_ended: Die Veranstaltung liegt bereits in der Vergangenheit.
    event_too_far_out: Der Beginn liegt mehr als ein Jahr in der Zukunft.
    event_too_long: Die Veranstaltung darf höchstens 30 Tage dauern.
    org_required: Bitte wähle eine Organisation aus.
    coords_required: Bitte wähle einen Ort auf der Karte aus.
    image_required: Bitte lade ein Bild hoch.
    image_too_small: Das Bild muss mindestens 256px auf der kürzeren Seite haben.
    image_author_required: Bitte gib die Urheber:in des Bildes an.
en:
  based_on_title: You are updating "{0}"
  based_on_dates_ended: "Last held: {0}"
  based_on_dates_ongoing: "Currently running: {0}"
  based_on_dates_upcoming: "Next dates: {0}"
  based_on_help: Your proposal replaces the existing event. Every field can still be adjusted.
  based_on_dates_cleared: The previous edition has ended - please enter the new dates.
  based_on_unlink: Start a new event instead
  new_event_title: You are proposing a new event.
  new_event_help: The original event stays unchanged.
  new_event_clear: Clear all fields
  duplicate_warning: An event named "{0}" already exists.
  duplicate_update: Update it instead
  prefill_image_failed: The existing image could not be loaded - please upload a new one.
  name: Name
  name_placeholder: e.g. GARNIX Festival
  description: Description
  description_placeholder: What happens at this event?
  description_help: Shown verbatim - please use the event's own language.
  starts_at: Starts
  ends_at: Ends
  timezone_help: Times are in the Europe/Berlin timezone.
  organising_org: Organising organisation
  organising_org_placeholder: Search for an organisation…
  organising_org_no_results: No matching organisation found
  organising_org_loading: Loading organisations…
  organising_org_load_error: The organisations couldn't be loaded.
  organising_org_retry: Try again
  organising_org_truncated: "Showing the first {0} matches only - keep typing to narrow it down."
  coords: Location
  image: Image
  image_drop: Drop an image here or click to browse
  image_hint: JPG, PNG, GIF or WebP, at least 256px, max 10MB
  image_remove: Remove
  image_invalid_type: Unsupported file format.
  image_too_large: The file is larger than 10MB.
  image_read_error: The image could not be read.
  image_author: Image author
  image_author_placeholder: e.g. Studentische Vertretung TUM
  license_info_title: CC BY 4.0 - Free to use with attribution
  license_info_description: All uploaded images are published under the CC BY 4.0 license. This means anyone can use the image as long as they credit you as the author.
  crop_thumb: Marker crop
  crop_thumb_help: Square crop used for the photo marker on the map.
  crop_header: Header crop
  crop_header_help: Wide crop (512×210) used for the page header.
  preview: Map preview
  preview_help: This is how the event will appear on the map - as a photo marker with its popup.
  error:
    name_required: Please enter a name.
    name_too_long: The name is too long (max 200 characters).
    description_required: Please enter a description.
    starts_at_required: Please enter a valid start time.
    ends_at_required: Please enter a valid end time.
    event_ends_before_start: The end must be after the start.
    event_ended: The event is already in the past.
    event_too_far_out: The start is more than a year in the future.
    event_too_long: The event may last at most 30 days.
    org_required: Please choose an organisation.
    coords_required: Please pick a location on the map.
    image_required: Please upload an image.
    image_too_small: The image must be at least 256px on its shorter edge.
    image_author_required: Please enter the image author.
</i18n>
