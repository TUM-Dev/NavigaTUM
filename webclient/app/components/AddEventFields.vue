<script setup lang="ts">
import {
  Combobox,
  ComboboxButton,
  ComboboxInput,
  ComboboxOption,
  ComboboxOptions,
} from "@headlessui/vue";
import { mdiCheck, mdiClose, mdiImage, mdiInformation, mdiUnfoldMoreHorizontal } from "@mdi/js";
import { type AdditionFieldErrors, validateAddition } from "~/composables/additionSchema";
import { useEditProposal } from "~/composables/editProposal";
import { type OrgOption, useKnownOrgs } from "~/composables/useKnownOrgs";
import { type CropTarget, cropToBlobUrl, HEADER_TARGET, THUMB_TARGET } from "~/utils/imageCrop";

const editProposal = useEditProposal();
const { t } = useI18n({ useScope: "local" });

const draft = computed(() => editProposal.value.pendingAddition);
const fieldErrors = computed<AdditionFieldErrors>(() => validateAddition(draft.value));
function errorFor(path: string): string | null {
  const key = fieldErrors.value[path];
  return key ? t(key) : null;
}

// --- Organising org combobox (client-filtered, like the room usage picker) ---
const knownOrgs = useKnownOrgs();
const orgQuery = ref("");
const filteredOrgs = computed<OrgOption[]>(() => knownOrgs.filter(orgQuery.value));
const orgTruncated = computed(() => filteredOrgs.value.length >= knownOrgs.maxResults);
const selectedOrg = computed<OrgOption | null>({
  get: () => knownOrgs.byId(draft.value.organising_org_id),
  set: (o) => {
    draft.value.organising_org_id = o?.org_id ?? null;
  },
});

// --- Coordinates. Events have no parent to centre on, so default to TUM main campus. ---
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

// --- Image upload ---
const VALID_IMAGE_TYPES = ["image/jpeg", "image/png", "image/gif", "image/webp"];
const MAX_FILE_BYTES = 10 * 1024 * 1024;
const fileInput = ref<HTMLInputElement>();
const isDragOver = ref(false);
const fileError = ref("");
// `blob:` URLs kept local (not in the persisted draft) and revoked on replace/unmount so they never
// outlive the session. `previewUrl` is the full image (the croppers' source); each crop preview
// renders the offset thumb/header exactly as the pipeline will.
const previewUrl = ref<string | null>(null);
const sourceImage = shallowRef<HTMLImageElement | null>(null);

// A rendered preview of one crop target, regenerated as its offset changes. A token guards against
// an earlier crop resolving after a later one and leaving a stale image.
function makeCropPreview(target: CropTarget) {
  const url = ref<string | null>(null);
  let token = 0;
  function set(next: string | null): void {
    if (url.value) URL.revokeObjectURL(url.value);
    url.value = next;
  }
  async function regenerate(offset: number): Promise<void> {
    const img = sourceImage.value;
    const w = draft.value.image_width;
    const h = draft.value.image_height;
    if (!img || !w || !h) {
      set(null);
      return;
    }
    const ticket = ++token;
    const next = await cropToBlobUrl(img, w, h, target, offset);
    if (ticket !== token) {
      if (next) URL.revokeObjectURL(next);
      return;
    }
    set(next);
  }
  return { url, set, regenerate };
}
const thumbPreview = makeCropPreview(THUMB_TARGET);
const headerPreview = makeCropPreview(HEADER_TARGET);
// Top-level refs so the template auto-unwraps them.
const thumbUrl = thumbPreview.url;
const headerUrl = headerPreview.url;

function bytesFromBase64(base64: string): Uint8Array<ArrayBuffer> {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

async function sha256Hex(bytes: Uint8Array<ArrayBuffer>): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const probe = new Image();
    probe.onload = () => resolve(probe);
    probe.onerror = () => reject(new Error("decode failed"));
    probe.src = url;
  });
}

function setPreviewUrl(url: string | null): void {
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value);
  previewUrl.value = url;
}

function regenerateCrops(): void {
  void thumbPreview.regenerate(draft.value.image_thumb_offset);
  void headerPreview.regenerate(draft.value.image_header_offset);
}

async function processFile(file: File): Promise<void> {
  fileError.value = "";
  if (!VALID_IMAGE_TYPES.includes(file.type)) {
    fileError.value = t("image_invalid_type");
    return;
  }
  if (file.size > MAX_FILE_BYTES) {
    fileError.value = t("image_too_large");
    return;
  }

  const base64 = await new Promise<string | null>((resolve) => {
    const reader = new FileReader();
    reader.onload = (e) => resolve(((e.target?.result as string) ?? "").split(",")[1] ?? null);
    reader.onerror = () => resolve(null);
    reader.readAsDataURL(file);
  });
  if (!base64) {
    fileError.value = t("image_read_error");
    return;
  }

  const url = URL.createObjectURL(file);
  let image: HTMLImageElement;
  try {
    image = await loadImage(url);
  } catch {
    URL.revokeObjectURL(url);
    fileError.value = t("image_read_error");
    return;
  }

  setPreviewUrl(url);
  sourceImage.value = image;
  draft.value.image = { base64, fileName: file.name || "event-image" };
  draft.value.image_width = image.naturalWidth;
  draft.value.image_height = image.naturalHeight;
  // A new image recentres both crops; their offset bounds depend on the new dimensions.
  draft.value.image_thumb_offset = 0;
  draft.value.image_header_offset = 0;
  // Content-addressed key, matching `event_{hash(img)}` consumed by the server (event.rs).
  draft.value.id = `event_${await sha256Hex(bytesFromBase64(base64))}`;
  regenerateCrops();
}

function onFileChange(event: Event): void {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (file) void processFile(file);
}
function onDrop(event: DragEvent): void {
  isDragOver.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file) void processFile(file);
}
function removeImage(): void {
  setPreviewUrl(null);
  thumbPreview.set(null);
  headerPreview.set(null);
  sourceImage.value = null;
  draft.value.image = null;
  draft.value.image_width = null;
  draft.value.image_height = null;
  draft.value.image_thumb_offset = 0;
  draft.value.image_header_offset = 0;
  draft.value.id = "";
  fileError.value = "";
  if (fileInput.value) fileInput.value.value = "";
}

watch(
  () => draft.value.image_thumb_offset,
  (o) => thumbPreview.regenerate(o)
);
watch(
  () => draft.value.image_header_offset,
  (o) => headerPreview.regenerate(o)
);

onBeforeUnmount(() => {
  setPreviewUrl(null);
  thumbPreview.set(null);
  headerPreview.set(null);
});

const showPreview = computed(() => Boolean(thumbPreview.url.value) && draft.value.coords.picked);
</script>

<template>
  <div class="space-y-3">
    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-name">
        {{ t("name") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <input
        id="add-event-name"
        v-model="draft.name"
        type="text"
        :placeholder="t('name_placeholder')"
        class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
      />
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
        class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full resize-y rounded border px-2 py-1 text-sm"
      />
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
              <p v-if="filteredOrgs.length === 0" class="text-zinc-500 dark:text-zinc-400 px-3 py-2 text-sm">
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
        container-class="h-44"
      />
      <p v-if="draft.coords.picked" class="text-zinc-600 dark:text-zinc-300 mt-1 text-xs">
        {{ draft.coords.lat.toFixed(5) }}, {{ draft.coords.lon.toFixed(5) }}
      </p>
    </div>

    <div>
      <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("image") }} <span class="text-red-700 dark:text-red-200">*</span></span>
      <div
        class="cursor-pointer rounded-lg border-2 border-dashed p-4 text-center text-sm transition-colors"
        :class="[
          isDragOver
            ? 'border-blue-500 dark:border-blue-400 bg-blue-50 dark:bg-blue-900'
            : 'border-zinc-300 dark:border-zinc-600 hover:border-blue-400 dark:hover:border-blue-500',
          errorFor('image') ? 'border-red-500 dark:border-red-400' : '',
        ]"
        @click="fileInput?.click()"
        @dragover.prevent="isDragOver = true"
        @dragleave.prevent="isDragOver = false"
        @drop.prevent="onDrop"
      >
        <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="onFileChange" />
        <template v-if="draft.image">
          <p class="text-blue-700 dark:text-blue-200 font-medium">{{ draft.image.fileName }}</p>
          <p v-if="draft.image_width && draft.image_height" class="text-zinc-500 dark:text-zinc-400 text-xs">
            {{ draft.image_width }}×{{ draft.image_height }}px
          </p>
          <button type="button" class="text-blue-600 dark:text-blue-300 hover:underline mt-1 inline-flex items-center gap-1 text-xs" @click.stop="removeImage">
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

    <template v-if="previewUrl && draft.image_width && draft.image_height">
      <div>
        <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("crop_thumb") }}</span>
        <p class="text-zinc-500 dark:text-zinc-400 mb-1 text-xs">{{ t("crop_thumb_help") }}</p>
        <EventImageCropper
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
        <EventImageCropper
          v-model="draft.image_header_offset"
          :image-url="previewUrl"
          :width="draft.image_width"
          :height="draft.image_height"
          :target="HEADER_TARGET"
        />
        <img v-if="headerUrl" :src="headerUrl" alt="" class="border-zinc-300 dark:border-zinc-600 mt-2 w-full rounded border" />
      </div>
    </template>

    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-image-author">
        {{ t("image_author") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <input
        id="add-event-image-author"
        v-model="draft.image_author"
        type="text"
        :placeholder="t('image_author_placeholder')"
        class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
      />
    </div>

    <div class="bg-blue-50 dark:bg-blue-900 border-blue-200 dark:border-blue-700 flex items-start gap-2 rounded-lg border p-3">
      <MdiIcon :path="mdiInformation" :size="18" class="text-blue-500 dark:text-blue-400 mt-0.5 flex-shrink-0" />
      <div>
        <p class="text-blue-800 dark:text-blue-100 text-sm font-medium">{{ t("license_info_title") }}</p>
        <p class="text-blue-700 dark:text-blue-200 mt-0.5 text-xs">{{ t("license_info_description") }}</p>
      </div>
    </div>

    <div v-if="showPreview">
      <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("preview") }}</span>
      <p class="text-zinc-500 dark:text-zinc-400 mb-1 text-xs">{{ t("preview_help") }}</p>
      <EventPreviewMap :lat="draft.coords.lat" :lon="draft.coords.lon" :image-url="thumbUrl" />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
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
  preview_help: So erscheint die Veranstaltung als Foto-Marker.
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
    image_required: Bitte lade ein Bild hoch.
    image_too_small: Das Bild muss mindestens 256px auf der kürzeren Seite haben.
    image_author_required: Bitte gib die Urheber:in des Bildes an.
en:
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
  preview_help: This is how the event will appear as a photo marker.
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
    image_required: Please upload an image.
    image_too_small: The image must be at least 256px on its shorter edge.
    image_author_required: Please enter the image author.
</i18n>
