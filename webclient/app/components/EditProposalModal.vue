<script setup lang="ts">
import { useEditProposal } from "~/composables/editProposal";
import ImageMetadataModal from "~/components/ImageMetadataModal.vue";

const { t } = useI18n({ useScope: "local" });
const editProposal = useEditProposal();

// State for managing edits
const showImageMetadataModal = ref(false);
const selectedImageFile = ref<{ base64: string; fileName: string } | null>(null);

// Image metadata state
const imageMetadata = ref({
  author: "",
  license: { text: "", url: "" },
  source: { text: "", url: "" },
  offsets: { header: null as number | null, thumb: null as number | null },
});

// Computed properties
const hasEdits = computed(() => Object.keys(editProposal.value.data.edits).length > 0);

// Methods
function removeEdit(roomId: string) {
  delete editProposal.value.data.edits[roomId];
}

function startAddEdit(editType: "image" | "location") {
  if (editType === "image") {
    // Handle image upload directly
    handleImageUpload();
  } else {
    // Start location editing directly
    startLocationEdit();
  }
}

function handleImageUpload() {
  if (process.client && typeof document !== "undefined") {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*";
    input.onchange = (event) => {
      const file = (event.target as HTMLInputElement)?.files?.[0];
      if (file) {
        const fileName = file.name || "uploaded-file";
        const reader = new FileReader();
        reader.onload = (e) => {
          const result = e.target?.result as string;
          if (result) {
            const base64 = result.split(",")[1];
            if (base64) {
              // Store the image data and initialize metadata with filename
              selectedImageFile.value = { base64, fileName };
              imageMetadata.value.source.text = fileName;

              // Always show metadata modal first for images
              showImageMetadataModal.value = true;
            }
          }
        };
        reader.readAsDataURL(file);
      }
    };
    input.click();
  }
}

function addImageEditForRoom(roomId: string, base64: string, metadata: any) {
  if (!editProposal.value.data.edits[roomId]) {
    editProposal.value.data.edits[roomId] = {
      coordinate: null,
      image: null,
    };
  }

  // Clean up metadata - remove empty URLs and null offsets
  const cleanMetadata = {
    author: metadata.author,
    license: {
      text: metadata.license.text,
      url: metadata.license.url || null,
    },
    source: {
      text: metadata.source.text,
      url: metadata.source.url || null,
    },
    ...(metadata.offsets.header !== null || metadata.offsets.thumb !== null
      ? {
          offsets: {
            header: metadata.offsets.header,
            thumb: metadata.offsets.thumb,
          },
        }
      : {}),
  };

  editProposal.value.data.edits[roomId].image = {
    content: base64,
    metadata: cleanMetadata,
  };
}

function startLocationEdit() {
  const roomId = editProposal.selected?.id;
  if (!roomId) {
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

function onLocationSelected(lat: number, lon: number) {
  const roomId = editProposal.selected?.id;
  if (roomId) {
    if (!editProposal.value.data.edits[roomId]) {
      editProposal.value.data.edits[roomId] = {
        coordinate: null,
        image: null,
      };
    }

    editProposal.value.data.edits[roomId]!.coordinate = { lat, lon };
  }
  editProposal.value.locationPicker.open = false;
}

function confirmImageMetadata(metadata: typeof imageMetadata.value) {
  showImageMetadataModal.value = false;

  const roomId = editProposal.selected?.id;
  if (roomId && selectedImageFile.value) {
    addImageEditForRoom(roomId, selectedImageFile.value.base64, metadata);
    selectedImageFile.value = null;
    // Reset metadata for next use
    imageMetadata.value = {
      author: "",
      license: { text: "", url: "" },
      source: { text: "", url: "" },
      offsets: { header: null, thumb: null },
    };
  } else {
    console.error("No room context available for image edit");
  }
}

function cancelImageMetadata() {
  showImageMetadataModal.value = false;
  selectedImageFile.value = null;
  // Reset metadata
  imageMetadata.value = {
    author: "",
    license: { text: "", url: "" },
    source: { text: "", url: "" },
    offsets: { header: null, thumb: null },
  };
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
  <TokenBasedEditProposalModal :data="editProposal.data">
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
          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="startAddEdit('image')">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("suggest_image_title") }}</span>
              <span class="text-xs text-zinc-200 font-normal">{{ t("suggest_image_desc") }}</span>
            </div>
          </Btn>

          <Btn variant="secondary" size="md" class="w-full justify-start text-left" @click="startAddEdit('location')">
            <div class="flex flex-col items-start">
              <span class="font-medium">{{ t("location_wrong_title") }}</span>
              <span class="text-xs text-zinc-200 font-normal">{{ t("location_wrong_desc") }}</span>
            </div>
          </Btn>
        </div>

        <!-- Image Metadata Modal -->
        <ImageMetadataModal :show="showImageMetadataModal" :metadata="imageMetadata" @confirm="confirmImageMetadata" @cancel="cancelImageMetadata" />

        <!-- Location Picker Modal -->
        <div v-if="editProposal.value.locationPicker.open" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="()=>editProposal.value.locationPicker.open = falses">
          <div class="bg-white rounded-lg p-6 m-4 max-w-lg w-full max-h-[90vh] overflow-y-auto">
            <h3 class="text-lg font-semibold mb-4 text-zinc-900">{{ t("select_location") }}</h3>
            <LocationPicker
              :initial-lat="editProposal.value.locationPicker.lat"
              :initial-lon="editProposal.value.locationPicker.lon"
              @coordinates-changed="
                (lat: number, lon: number) => {
                  editProposal.value.locationPicker.lat = lat;
                  editProposal.value.locationPicker.lon = lon;
                }
              "
            />
            <div class="flex gap-2 mt-4">
              <Btn variant="primary" @click="onLocationSelected(locationPickerCoords.lat, locationPickerCoords.lon)">
                {{ t("confirm_location") }}
              </Btn>
              <Btn variant="secondary" @click="cancelLocationPicker">
                {{ t("cancel") }}
              </Btn>
            </div>
          </div>
        </div>
      </div>

      <!-- Current Edits -->
      <div class="pt-4 pb-8" v-if="hasEdits">
        <label class="text-zinc-600 text-sm font-semibold">{{ t("current_edits") }}</label>
        <div class="space-y-2 mt-2">
          <div v-for="(edit, roomId) in editProposal.data.edits" :key="roomId" class="bg-zinc-100 border-zinc-300 rounded p-3 border">
            <div class="flex justify-between items-start">
              <div class="flex-grow">
                <p class="font-medium text-sm text-zinc-900">{{ editProposal.value.selected?.name }}</p>
                <div class="text-xs text-zinc-600 mt-1">
                  <p>{{ t("edit_type") }}: {{ getEditTypeDisplay(String(roomId)) }}</p>
                </div>
              </div>
              <button @click="removeEdit(String(roomId))" class="text-red-600 hover:text-red-800 text-sm">
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
  location_wrong_title: Der Standort ist falsch
  location_wrong_desc: Position auf der Karte korrigieren
  select_location: Standort auswählen
  confirm_location: Standort bestätigen
  cancel: Abbrechen
  edit_type: Änderungstyp
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
  location_wrong_title: The location is wrong
  location_wrong_desc: Correct position on the map
  select_location: Select Location
  confirm_location: Confirm Location
  cancel: Cancel
  edit_type: Edit Type
  room_edits: Room Edits
  image_attached: Image attached
  coordinate: Coordinate
  image: Image
  remove: Remove
  success_thank_you: Thank you for your edit proposal! We will process it as soon as possible.
  success_response_at: You can see our response at {this_pr}
  success_this_pr: this GitHub pull request
</i18n>
