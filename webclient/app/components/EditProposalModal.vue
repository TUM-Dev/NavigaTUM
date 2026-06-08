<script setup lang="ts">
import type { components } from "~/api_types";
import { emptyPropertyFields, emptyRoomEdit, useEditProposal } from "~/composables/editProposal";
import { useFeedback } from "~/composables/feedback";
import {
  buildDraftOpeningHours,
  draftHasInvalidRange,
  emptyOpeningHoursDraft,
  hasWeeklyHours,
  holidayHoursMissing,
  type OpeningHoursDraft,
} from "~/utils/openingHours";

const HTTP_URL_RE = /^https?:\/\//;

type PropertyEdit = components["schemas"]["PropertyEdit"];

const { t } = useI18n({ useScope: "local" });
const editProposal = useEditProposal();
const feedback = useFeedback();
const route = useRoute();

function switchToFeedback() {
  const id = editProposal.value.selected?.id ?? (route.params.id as string);
  editProposal.value.open = false;
  feedback.value.open = true;
  feedback.value.data = {
    category: "entry",
    subject: `[${id}]: `,
    body: "",
    deletion_requested: false,
  };
}

const propertiesModalOpen = ref(false);

// Opening-hours edit lives inside the properties modal; its draft is committed
// together with the other property edits on save.
const openingHoursDraft = ref<OpeningHoursDraft>(emptyOpeningHoursDraft());
const openingHoursError = ref<"" | "source" | "range" | "holiday">("");

function resetOpeningHoursDraft() {
  openingHoursDraft.value = emptyOpeningHoursDraft();
  openingHoursError.value = "";
}

// Returns false when a half-finished schedule must block the modal from closing.
function injectOpeningHours(): boolean {
  const roomId = editProposal.value.selected?.id;
  if (!roomId) return true;

  // A backwards range would be silently dropped by the assembler, so surface it instead.
  if (draftHasInvalidRange(openingHoursDraft.value)) {
    openingHoursError.value = "range";
    return false;
  }

  // Holidays alone (the default `PH off`) are not a schedule worth submitting -
  // only commit once regular weekly hours have actually been entered.
  if (!hasWeeklyHours(openingHoursDraft.value)) {
    openingHoursError.value = "";
    return true;
  }
  // "Open on holidays" without any hours is ambiguous - force a concrete range.
  if (holidayHoursMissing(openingHoursDraft.value)) {
    openingHoursError.value = "holiday";
    return false;
  }
  const osm = buildDraftOpeningHours(openingHoursDraft.value);

  const url = openingHoursDraft.value.sourceUrl;
  if (!HTTP_URL_RE.test(url) || !URL.canParse(url)) {
    openingHoursError.value = "source";
    return false;
  }

  openingHoursError.value = "";
  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = emptyRoomEdit();
  }
  editProposal.value.data.edits[roomId].opening_hours = { opening_hours: osm, source_url: url };
  return true;
}

function savePropertiesAndClose() {
  injectPropertyEdits();
  if (!injectOpeningHours()) return;
  propertiesModalOpen.value = false;
}

const osmEditUrl = computed(() => {
  const lat = editProposal.value.locationPicker.lat;
  const lon = editProposal.value.locationPicker.lon;
  return `https://www.openstreetmap.org/edit#map=19/${lat}/${lon}`;
});

// Known usages for category dropdown - cached across modal opens
const runtimeConfig = useRuntimeConfig();
const { data: knownUsages } = useAsyncData(
  "known_usages",
  () =>
    $fetch<
      { usage_id: number; occurrences: number; name_de: string; name_en: string; din_277: string }[]
    >(`${runtimeConfig.public.cdnURL}/cdn/known_usages.json`),
  { default: () => [] }
);

const categoryOptions = computed(() =>
  knownUsages.value.map((u) => ({
    label: `${u.name_de} / ${u.name_en}`,
    value: `${u.name_de}|${u.name_en}|${u.din_277}`,
    ...u,
  }))
);

const selectedCategory = computed({
  get() {
    const de = editProposal.value.propertyFields.categoryDe;
    const en = editProposal.value.propertyFields.categoryEn;
    const din = editProposal.value.propertyFields.categoryDin277;
    if (!de && !en) return "";
    return `${de}|${en}|${din}`;
  },
  set(val: string) {
    if (!val) {
      editProposal.value.propertyFields.categoryDe = "";
      editProposal.value.propertyFields.categoryEn = "";
      editProposal.value.propertyFields.categoryDin277 = "";
      editProposal.value.propertyFields.categoryDin277Desc = "";
      return;
    }
    const parts = val.split("|");
    editProposal.value.propertyFields.categoryDe = parts[0] ?? "";
    editProposal.value.propertyFields.categoryEn = parts[1] ?? "";
    editProposal.value.propertyFields.categoryDin277 = parts[2] ?? "";
    editProposal.value.propertyFields.categoryDin277Desc = "";
  },
});

// Build property edits from changed fields
function buildPropertyEdits(): PropertyEdit[] {
  const fields = editProposal.value.propertyFields;
  const original = editProposal.value.originalPropertyFields;
  const edits: PropertyEdit[] = [];

  // Name changed?
  if (
    (fields.name !== original.name || fields.shortName !== original.shortName) &&
    (fields.name || fields.shortName)
  ) {
    edits.push({
      type: "name",
      name: fields.name || null,
      short_name: fields.shortName || null,
    });
  }

  // Category changed?
  if (
    (fields.categoryDe !== original.categoryDe || fields.categoryEn !== original.categoryEn) &&
    fields.categoryDe
  ) {
    edits.push({
      type: "usage",
      name_de: fields.categoryDe,
      name_en: fields.categoryEn || fields.categoryDe,
      din_277: fields.categoryDin277 || null,
      din_277_desc: fields.categoryDin277Desc || null,
    });
  }

  // Link added?
  if (
    fields.linkUrl &&
    (fields.linkUrl.startsWith("http://") || fields.linkUrl.startsWith("https://")) &&
    (fields.linkUrl.startsWith("http://") || fields.linkUrl.startsWith("https://")) &&
    (fields.linkTextDe || fields.linkTextEn)
  ) {
    edits.push({
      type: "link",
      text_de: fields.linkTextDe || fields.linkTextEn,
      text_en: fields.linkTextEn || fields.linkTextDe,
      url: fields.linkUrl,
    });
  }

  return edits;
}

// Inject property edits into the edit data before submission
function injectPropertyEdits() {
  const roomId = editProposal.value.selected?.id;
  if (!roomId) return;

  const propertyEdits = buildPropertyEdits();
  if (propertyEdits.length === 0) return;

  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = emptyRoomEdit();
  }
  editProposal.value.data.edits[roomId].properties = propertyEdits;
}

// Watch for submission - inject property edits when the modal data changes
watch(
  () => editProposal.value.open,
  (isOpen) => {
    if (!isOpen) {
      propertiesModalOpen.value = false;
      editProposal.value.propertyFields = emptyPropertyFields();
      editProposal.value.originalPropertyFields = emptyPropertyFields();
      resetOpeningHoursDraft();
    }
  }
);

function switchToAddProposal() {
  editProposal.value.open = false;
  editProposal.value.addOpen = true;
}

// Methods
function addImageEditForRoom(
  roomId: string,
  base64: string,
  metadata: typeof editProposal.value.imageUpload.metadata
) {
  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = emptyRoomEdit();
  }

  if (!metadata.license.url) {
    metadata.license.url = null;
  }

  editProposal.value.data.edits[roomId].image = {
    content: base64,
    metadata: metadata,
  };
}

function startLocationEdit() {
  const roomId = editProposal.value?.selected?.id;
  if (!roomId || !editProposal.value?.data?.edits) {
    console.error("No room context available for location edit");
    return;
  }

  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = emptyRoomEdit();
  }
  editProposal.value.locationPicker.open = true;
}

function onLocationSelected() {
  const roomId = editProposal.value.selected?.id;
  if (!roomId) return;

  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = emptyRoomEdit();
  }

  editProposal.value.data.edits[roomId].coordinate = {
    lat: editProposal.value.locationPicker.lat,
    lon: editProposal.value.locationPicker.lon,
  };
  editProposal.value.locationPicker.open = false;
}

function confirmImageMetadata(metadata: typeof editProposal.value.imageUpload.metadata) {
  editProposal.value.imageUpload.open = false;

  const roomId = editProposal.value.selected?.id;
  if (roomId && editProposal.value.imageUpload.selectedFile) {
    addImageEditForRoom(roomId, editProposal.value.imageUpload.selectedFile.base64, metadata);
    // Reset for next use
    editProposal.value.imageUpload.selectedFile = null;
    editProposal.value.imageUpload.metadata = {
      author: "",
      license: { text: "", url: "" },
    };
  } else {
    console.error("No room context or file available for image edit");
  }
}

function cancelImageMetadata() {
  editProposal.value.imageUpload.open = false;
  editProposal.value.imageUpload.selectedFile = null;
  // Reset metadata
  editProposal.value.imageUpload.metadata = {
    author: "",
    license: { text: "", url: "" },
  };
}

function handleFileSelected(file: { base64: string; fileName: string } | null) {
  editProposal.value.imageUpload.selectedFile = file;
}

function getEditTypeDisplay(roomId: string): string {
  const edit = editProposal.value.data.edits[roomId];
  if (!edit) return t("room_edits");

  const types: string[] = [];
  if (edit.coordinate) types.push(t("coordinate"));
  if (edit.image) types.push(t("image"));
  if (edit.properties?.length) types.push(t("property"));
  if (edit.opening_hours) types.push(t("opening_hours"));

  return types.length > 0 ? types.join(", ") : t("room_edits");
}
</script>

<template>
  <TokenBasedEditProposalModal v-if="editProposal" v-model:open="editProposal.open" :data="editProposal.data" :title="t('title')">
    <template #modal>
      <!-- What would you like to change? -->
      <div class="flex flex-col">
        <label class="text-zinc-600 dark:text-zinc-300 text-sm font-semibold" for="edit-context">
          {{ t("additional_context") }}
        </label>
        <textarea
          id="edit-context"
          v-model="editProposal.data.additional_context"
          class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 resize-y rounded border px-2 py-1"
          :placeholder="t('additional_context_placeholder')"
          rows="3"
        />
        <p class="text-zinc-500 dark:text-zinc-400 text-xs">{{ t("additional_context_help") }}</p>
        <button
          type="button"
          class="focusable text-zinc-500 dark:text-zinc-400 hover:text-blue-700 dark:hover:text-blue-300 mt-1 self-start rounded-sm text-xs underline"
          @click="switchToFeedback"
        >
          {{ t("report_problem_instead") }}
        </button>
      </div>

      <!-- Other Changes Section -->
      <div class="pt-4">
        <label class="text-zinc-600 dark:text-zinc-300 text-sm font-semibold mb-3 block">{{ t("other_changes") }}</label>

        <div class="space-y-2">
          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="() => (editProposal.imageUpload.open = true)">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("suggest_image_title") }}</span>
              <span class="text-xs text-zinc-200 dark:text-zinc-700 font-normal">{{ t("suggest_image_desc") }}</span>
            </div>
          </Btn>

          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="startLocationEdit">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("room_position_wrong_title") }}</span>
              <span class="text-xs text-zinc-200 dark:text-zinc-700 font-normal">{{ t("room_position_wrong_desc") }}</span>
            </div>
          </Btn>

          <Btn variant="secondary" size="md" class="w-full justify-start text-left" :to="osmEditUrl" target="_blank">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("map_missing_roads_title") }}</span>
              <span class="text-xs text-zinc-200 dark:text-zinc-700 font-normal">{{ t("map_missing_roads_desc") }}</span>
            </div>
          </Btn>

          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="() => (propertiesModalOpen = true)">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("properties_title") }}</span>
              <span class="text-xs text-zinc-200 dark:text-zinc-700 font-normal">{{ t("properties_desc") }}</span>
            </div>
          </Btn>

          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="switchToAddProposal">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("propose_addition_title") }}</span>
              <span class="text-xs text-zinc-200 dark:text-zinc-700 font-normal">{{ t("propose_addition_desc") }}</span>
            </div>
          </Btn>
        </div>

        <!-- Image Metadata Modal -->
        <ImageMetadataModal
          v-model:open="editProposal.imageUpload.open"
          :metadata="editProposal.imageUpload.metadata"
          :selected-file="editProposal.imageUpload.selectedFile"
          @confirm="confirmImageMetadata"
          @cancel="cancelImageMetadata"
          @file-selected="handleFileSelected"
        />

        <!-- Location Picker Modal -->
        <LocationPickerModal
          v-model:open="editProposal.locationPicker.open"
          :initial-lat="editProposal.locationPicker.lat"
          :initial-lon="editProposal.locationPicker.lon"
          :floors="editProposal.locationPicker.floors"
          :initial-floor="editProposal.locationPicker.floor"
          @coordinates-changed="
            (lat: number, lon: number) => {
              editProposal.locationPicker.lat = lat;
              editProposal.locationPicker.lon = lon;
            }
          "
          @confirm="onLocationSelected"
          @cancel="() => (editProposal.locationPicker.open = false)"
        />

        <!-- Properties Modal -->
        <Modal v-model="propertiesModalOpen" :title="t('properties')">
          <div class="space-y-3">
            <!-- Name -->
            <div>
              <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-1" for="edit-name">{{ t("field_name") }}</label>
              <input
                id="edit-name"
                v-model="editProposal.propertyFields.name"
                type="text"
                class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 w-full text-sm"
              />
              <p class="text-zinc-500 dark:text-zinc-400 text-xs mt-1">{{ t("field_name_help") }}</p>
            </div>

            <!-- Short Name -->
            <div>
              <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-1" for="edit-short-name">{{ t("field_short_name") }}</label>
              <input
                id="edit-short-name"
                v-model="editProposal.propertyFields.shortName"
                type="text"
                class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 w-full text-sm"
              />
              <p class="text-zinc-500 dark:text-zinc-400 text-xs mt-1">{{ t("field_short_name_help") }}</p>
            </div>

            <!-- Category -->
            <div>
              <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-1" for="edit-category">{{ t("field_category") }}</label>
              <select
                id="edit-category"
                v-model="selectedCategory"
                class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 w-full text-sm"
              >
                <option value="">-</option>
                <option v-for="opt in categoryOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>

            <!-- Add a Link -->
            <div class="border-t border-zinc-200 dark:border-zinc-700 pt-3">
              <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-1">{{ t("field_add_link") }}</label>
              <div class="space-y-2">
                <div class="flex items-center gap-2">
                  <span class="text-zinc-400 dark:text-zinc-500 text-xs w-8">URL</span>
                  <input
                    v-model="editProposal.propertyFields.linkUrl"
                    type="url"
                    placeholder="https://"
                    class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 flex-1 text-sm"
                  />
                </div>
                <div class="flex items-center gap-2">
                  <span class="text-zinc-400 dark:text-zinc-500 text-xs w-8">DE</span>
                  <input
                    v-model="editProposal.propertyFields.linkTextDe"
                    type="text"
                    class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 flex-1 text-sm"
                  />
                </div>
                <div class="flex items-center gap-2">
                  <span class="text-zinc-400 dark:text-zinc-500 text-xs w-8">EN</span>
                  <input
                    v-model="editProposal.propertyFields.linkTextEn"
                    type="text"
                    class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 flex-1 text-sm"
                  />
                </div>
              </div>
            </div>

            <!-- Opening hours -->
            <div class="border-t border-zinc-200 dark:border-zinc-700 pt-3">
              <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-2">{{ t("opening_hours_title") }}</label>
              <OpeningHoursEditor v-model="openingHoursDraft" />
              <p v-if="openingHoursError === 'source'" class="text-red-600 dark:text-red-300 text-xs mt-2">
                {{ t("opening_hours_source_required") }}
              </p>
              <p v-else-if="openingHoursError === 'range'" class="text-red-600 dark:text-red-300 text-xs mt-2">
                {{ t("opening_hours_invalid_range") }}
              </p>
              <p v-else-if="openingHoursError === 'holiday'" class="text-red-600 dark:text-red-300 text-xs mt-2">
                {{ t("opening_hours_holiday_hours_required") }}
              </p>
            </div>
          </div>
          <div class="flex justify-end pt-4">
            <Btn variant="primary" size="md" @click="savePropertiesAndClose">{{ t("save") }}</Btn>
          </div>
        </Modal>
      </div>

      <!-- Current Edits -->
      <div class="pt-4 pb-2" v-if="Object.keys(editProposal.data.edits).length">
        <label class="text-zinc-600 dark:text-zinc-300 text-sm font-semibold">{{ t("current_edits") }}</label>
        <div class="space-y-2 mt-2">
          <div v-for="roomId in Object.keys(editProposal.data.edits)" :key="roomId" class="bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 rounded p-3 border">
            <div class="flex justify-between items-start">
              <div class="flex-grow">
                <p class="font-medium text-sm text-zinc-900 dark:text-zinc-50">{{ editProposal.selected?.name }}</p>
                <div class="text-xs text-zinc-600 dark:text-zinc-300 mt-1">
                  <p>{{ getEditTypeDisplay(String(roomId)) }}</p>
                </div>
              </div>
              <button @click="() => delete editProposal.data.edits[roomId]" class="text-red-600 dark:text-red-300 hover:text-red-800 dark:hover:text-red-100 text-sm">
                {{ t("remove") }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Pending Additions -->
      <div class="pt-4 pb-8" v-if="Object.keys(editProposal.data.additions).length">
        <label class="text-zinc-600 dark:text-zinc-300 text-sm font-semibold">{{ t("pending_additions") }}</label>
        <div class="space-y-2 mt-2">
          <div
            v-for="(addition, addId) in editProposal.data.additions"
            :key="addId"
            class="bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 rounded p-3 border"
          >
            <div class="flex justify-between items-start">
              <div class="flex-grow">
                <p class="font-medium text-sm text-zinc-900 dark:text-zinc-50">{{ addId }}</p>
                <p class="text-xs text-zinc-600 dark:text-zinc-300 mt-1">{{ t(`kind.${addition.kind}`) }}</p>
              </div>
              <button @click="() => delete editProposal.data.additions[addId]" class="text-red-600 dark:text-red-300 hover:text-red-800 dark:hover:text-red-100 text-sm">
                {{ t("remove") }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template #success="{ successUrl }">
      <p>{{ t("success_thank_you") }}</p>
      <I18nT tag="p" keypath="success_response_at">
        <template #this_pr>
          <Btn variant="link" :to="successUrl">{{ t("success_this_pr") }}</Btn>
        </template>
      </I18nT>
    </template>
  </TokenBasedEditProposalModal>
</template>

<i18n lang="yaml">
de:
  title: Änderungen vorschlagen
  additional_context: Was möchtest du ändern?
  additional_context_placeholder: "Beschreibe was falsch ist oder verbessert werden sollte:\n- Falsche Rauminformationen (Name, Beschreibung, Öffnungszeiten)\n- Fehlende oder veraltete Details\n- Andere Korrekturen oder Verbesserungen"
  additional_context_help: Beschreibe hier alle Probleme oder Verbesserungsvorschläge.
  report_problem_instead: Du kennst die Lösung nicht? Melde einfach ein Problem.
  other_changes: Weitere Änderungen
  properties: Eigenschaften
  properties_title: Eigenschaften bearbeiten
  properties_desc: Name, Kategorie oder Links dieses Raums ändern
  opening_hours_title: Öffnungszeiten
  opening_hours_source_required: Bitte gib eine Quelle (URL) für die Öffnungszeiten an.
  opening_hours_invalid_range: Bitte korrigiere die ungültigen Zeiträume (Ende muss nach dem Anfang liegen).
  opening_hours_holiday_hours_required: Bitte gib die Öffnungszeiten an Feiertagen an oder wähle „Geschlossen“.
  field_name: Name
  field_name_help: Der vollständige Name, wie er auf der Detailseite angezeigt wird (z.B. „Hörsaal 1 Friedrich L. Bauer")
  field_short_name: Kurzname
  field_short_name_help: Der Kurzname wird in Suchergebnissen angezeigt (z.B. „Hörsaal 1" oder „5602.EG.001")
  field_category: Kategorie
  field_add_link: Link hinzufügen
  current_edits: Aktuelle Änderungen
  suggest_image_title: Bild hinzufügen
  suggest_image_desc: Ein Foto vom Raum, Gebäude oder Standort hinzufügen
  room_position_wrong_title: Raum ist falsch positioniert
  room_position_wrong_desc: Position dieses Raums in Navigatum korrigieren
  map_missing_roads_title: Wege/Gebäude fehlen auf der Karte
  map_missing_roads_desc: Fehlende Wege oder Gebäude direkt in OpenStreetMap hinzufügen
  propose_addition_title: Raum, Gebäude oder POI fehlt
  propose_addition_desc: Einen neuen Eintrag strukturiert vorschlagen
  pending_additions: Neue Einträge in dieser Anfrage
  kind:
    room: Raum
    building: Gebäude
    poi: POI
    event: Veranstaltung
  room_edits: Raum-Änderungen
  coordinate: Koordinaten
  image: Bild
  property: Eigenschaft
  opening_hours: Öffnungszeiten
  save: Speichern
  remove: Entfernen
  success_thank_you: Vielen Dank für deinen Verbesserungsvorschlag! Wir werden ihn schnellstmöglich bearbeiten.
  success_response_at: Du findest unsere Antwort auf {this_pr}
  success_this_pr: diesem GitHub Pull Request
en:
  title: Propose Changes
  additional_context: What would you like to change?
  additional_context_placeholder: "Describe what's wrong or needs improvement:\n- Incorrect room information (name, description, hours)\n- Missing or outdated details\n- Other corrections or improvements"
  additional_context_help: Describe any issues or improvement suggestions here.
  report_problem_instead: Don't know the fix? Just report a problem.
  other_changes: Other changes
  properties: Properties
  properties_title: Edit properties
  properties_desc: Change the name, category, or links of this room
  opening_hours_title: Opening hours
  opening_hours_source_required: Please provide a source (URL) for the opening hours.
  opening_hours_invalid_range: Please fix the invalid time ranges (the end must be after the start).
  opening_hours_holiday_hours_required: Please add the opening hours for public holidays, or choose “Closed”.
  field_name: Name
  field_name_help: The full name shown on the detail page (e.g. "Lecture Hall 1 Friedrich L. Bauer")
  field_short_name: Short name
  field_short_name_help: The short name is shown in search results (e.g. "Lecture Hall 1" or "5602.EG.001")
  field_category: Category
  field_add_link: Add a link
  current_edits: Current Edits
  suggest_image_title: Add image
  suggest_image_desc: Add a photo of the room, building, or location
  room_position_wrong_title: Room is positioned incorrectly
  room_position_wrong_desc: Correct this room's position in Navigatum
  map_missing_roads_title: Other details (paths, vegetation) missing from map
  map_missing_roads_desc: Add missing paths or buildings directly in OpenStreetMap
  propose_addition_title: Missing a room, building, or POI?
  propose_addition_desc: Propose a new entry in a structured form
  pending_additions: New entries in this request
  kind:
    room: Room
    building: Building
    poi: POI
    event: Event
  room_edits: Room Edits
  coordinate: Coordinate
  image: Image
  property: Property
  opening_hours: Opening hours
  save: Save
  remove: Remove
  success_thank_you: Thank you for your edit proposal! We will process it as soon as possible.
  success_response_at: You can see our response at {this_pr}
  success_this_pr: this GitHub pull request
</i18n>
