<script setup lang="ts">
import ImageMetadataModal from "~/components/ImageMetadataModal.vue";
import { useEditProposal } from "~/composables/editProposal";

const { t } = useI18n({ useScope: "local" });
const editProposal = useEditProposal();

const osmEditUrl = computed(() => {
  const lat = editProposal.value.locationPicker.lat;
  const lon = editProposal.value.locationPicker.lon;
  return `https://www.openstreetmap.org/edit#map=19/${lat}/${lon}`;
});

// Methods
function addImageEditForRoom(
  roomId: string,
  base64: string,
  metadata: typeof editProposal.value.imageUpload.metadata
) {
  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = {
      coordinate: null,
      image: null,
    };
  }

  // Clean up metadata - remove empty URLs
  if (!metadata.license.url) {
    metadata.license.url = null;
  }

  if (editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId].image = {
      content: base64,
      metadata: metadata,
    };
  }
}

function startLocationEdit() {
  const roomId = editProposal.value?.selected?.id;
  if (!roomId || !editProposal.value?.data?.edits) {
    console.error("No room context available for location edit");
    return;
  }

  // Initialize edit for room if it doesn't exist
  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = {
      coordinate: null,
      image: null,
    };
  }
  editProposal.value.locationPicker.open = true;
}

function onLocationSelected() {
  const roomId = editProposal.value.selected.id;
  if (roomId && editProposal.value.data.edits) {
    if (!editProposal.value.data.edits[roomId]) {
      editProposal.value.data.edits[roomId] = {
        coordinate: null,
        image: null,
      };
    }

    editProposal.value.data.edits[roomId].coordinate = {
      lat: editProposal.value.locationPicker.lat,
      lon: editProposal.value.locationPicker.lon,
    };
  }
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

  const types = [];
  if (edit.coordinate) types.push(t("coordinate"));
  if (edit.image) types.push(t("image"));

  return types.length > 0 ? types.join(", ") : t("room_edits");
}
</script>

<template>
  <TokenBasedEditProposalModal v-if="editProposal" :data="editProposal.data">
    <template #modal>
      <!-- Additional Context -->
      <div class="flex flex-col">
        <label class="text-zinc-600 text-sm font-semibold" for="edit-context">
          {{ t("additional_context") }}
        </label>
        <textarea
          id="edit-context"
          v-model="editProposal.data.additional_context"
          class="focusable bg-zinc-200 border-zinc-400 text-zinc-900 resize-y rounded border px-2 py-1"
          :placeholder="t('additional_context_placeholder')"
          rows="6"
        />
        <p class="text-zinc-500 text-xs">{{ t("additional_context_help") }}</p>
      </div>

      <!-- Add New Edit Actions -->
      <div class="pt-4">
        <label class="text-zinc-600 text-sm font-semibold mb-3 block">{{ t("suggest_changes") }}</label>

        <div class="space-y-2">
          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="() => (editProposal.imageUpload.open = true)">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("suggest_image_title") }}</span>
              <span class="text-xs text-zinc-200 font-normal">{{ t("suggest_image_desc") }}</span>
            </div>
          </Btn>

          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="startLocationEdit">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("room_position_wrong_title") }}</span>
              <span class="text-xs text-zinc-200 font-normal">{{ t("room_position_wrong_desc") }}</span>
            </div>
          </Btn>

          <Btn variant="secondary" size="md" class="w-full justify-start text-left" :to="osmEditUrl" target="_blank">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("map_missing_roads_title") }}</span>
              <span class="text-xs text-zinc-200 font-normal">{{ t("map_missing_roads_desc") }}</span>
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
          @coordinates-changed="
            (lat: number, lon: number) => {
              editProposal.locationPicker.lat = lat;
              editProposal.locationPicker.lon = lon;
            }
          "
          @confirm="onLocationSelected"
          @cancel="() => (editProposal.locationPicker.open = false)"
        />
      </div>

      <!-- Current Edits -->
      <div class="pt-4 pb-8" v-if="Object.keys(editProposal.data.edits).length">
        <label class="text-zinc-600 text-sm font-semibold">{{ t("current_edits") }}</label>
        <div class="space-y-2 mt-2">
          <div v-for="(edit, roomId) in editProposal.data.edits" :key="roomId" class="bg-zinc-100 border-zinc-300 rounded p-3 border">
            <div class="flex justify-between items-start">
              <div class="flex-grow">
                <p class="font-medium text-sm text-zinc-900">{{ editProposal.selected?.name }}</p>
                <div class="text-xs text-zinc-600 mt-1">
                  <p>{{ t("edit_type", [getEditTypeDisplay(String(roomId))]) }}</p>
                </div>
              </div>
              <button @click="() => delete editProposal.data.edits[roomId]" class="text-red-600 hover:text-red-800 text-sm">
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
  additional_context: Zusätzlicher Kontext
  additional_context_placeholder: "Beschreibe was falsch ist oder verbessert werden sollte:\n- Falsche Rauminformationen (Name, Beschreibung, Öffnungszeiten)\n- Fehlende oder veraltete Details\n- Andere Korrekturen oder Verbesserungen"
  additional_context_help: Beschreibe hier alle Probleme oder Verbesserungsvorschläge.
  current_edits: Aktuelle Änderungen
  suggest_changes: Was möchtest du ändern?
  suggest_image_title: Neues Bild vorschlagen
  suggest_image_desc: Ein Foto vom Raum, Gebäude oder Standort hinzufügen
  room_position_wrong_title: Raum ist falsch positioniert
  room_position_wrong_desc: Position dieses Raums in Navigatum korrigieren
  map_missing_roads_title: Wege/Gebäude fehlen auf der Karte
  map_missing_roads_desc: Fehlende Wege oder Gebäude direkt in OpenStreetMap hinzufügen
  edit_type: Änderungstyp {0}
  room_edits: Raum-Änderungen
  image_attached: Bild angehängt
  coordinate: Koordinaten
  image: Bild
  remove: Entfernen
  success_thank_you: Vielen Dank für deinen Verbesserungsvorschlag! Wir werden ihn schnellstmöglich bearbeiten.
  success_response_at: Du findest unsere Antwort auf {this_pr}
  success_this_pr: diesem GitHub Pull Request
en:
  additional_context: Additional Context
  additional_context_placeholder: "Describe what's wrong or needs improvement:\n- Incorrect room information (name, description, hours)\n- Missing or outdated details\n- Other corrections or improvements"
  additional_context_help: Describe any issues or improvement suggestions here.
  current_edits: Current Edits
  suggest_changes: What would you like to change?
  suggest_image_title: Suggest a new image
  suggest_image_desc: Add a photo of the room, building, or location
  room_position_wrong_title: Room is positioned incorrectly
  room_position_wrong_desc: Correct this room's position in Navigatum
  map_missing_roads_title: Other details (paths, vegetation) missing from map
  map_missing_roads_desc: Add missing paths or buildings directly in OpenStreetMap
  edit_type: Edit Type {0}
  room_edits: Room Edits
  image_attached: Image attached
  coordinate: Coordinate
  image: Image
  remove: Remove
  success_thank_you: Thank you for your edit proposal! We will process it as soon as possible.
  success_response_at: You can see our response at {this_pr}
  success_this_pr: this GitHub pull request
</i18n>
