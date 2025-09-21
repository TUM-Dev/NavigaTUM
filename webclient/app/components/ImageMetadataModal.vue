<script setup lang="ts">
import { mdiClose, mdiFileCheck, mdiImage, mdiInformation } from "@mdi/js";
import type { DeepWritable } from "ts-essentials";
import type { components } from "~/api_types";

type ImageMetadata = components["schemas"]["ImageMetadata"];

interface Props {
  metadata: DeepWritable<ImageMetadata>;
  selectedFile: { base64: string; fileName: string } | null;
}
const modalOpen = defineModel<boolean>("open", {
  required: true,
});

interface Emits {
  (e: "confirm", metadata: ImageMetadata): void;
  (e: "cancel"): void;
  (e: "file-selected", file: { base64: string; fileName: string } | null): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { t } = useI18n({ useScope: "local" });

// Create a local copy of metadata to avoid mutating props
const localMetadata = ref<ImageMetadata>({
  author: "",
  license: { text: "CC BY 4.0", url: "https://creativecommons.org/licenses/by/4.0/" },
});

// Computed property for selected file
const selectedFile = computed(() => props.selectedFile);

// Drag and drop state
const isDragOver = ref(false);
const fileInput = ref<HTMLInputElement>();
const dropZone = ref<HTMLElement>();
const fileError = ref<string>("");

// Watch for changes in props.metadata to update local copy
watch(
  () => props.metadata,
  (newMetadata: DeepWritable<ImageMetadata>) => {
    localMetadata.value = {
      author: newMetadata.author,
      license: { text: "CC BY 4.0", url: "https://creativecommons.org/licenses/by/4.0/" },
    };
  },
  { immediate: true }
);

// File upload handlers
function handleFileSelect(event: Event) {
  const file = (event.target as HTMLInputElement)?.files?.[0];
  if (file) {
    processFile(file);
  }
}

function handleDrop(event: DragEvent) {
  isDragOver.value = false;
  const files = event.dataTransfer?.files;
  if (files && files.length > 0 && files[0]) {
    processFile(files[0]);
  }
}

function handleDragOver() {
  isDragOver.value = true;
}

function handleDragLeave(event: DragEvent) {
  // Only set to false if we're leaving the drop zone completely
  const rect = dropZone.value?.getBoundingClientRect();
  if (rect) {
    const x = event.clientX;
    const y = event.clientY;
    if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
      isDragOver.value = false;
    }
  }
}

function triggerFileInput() {
  fileInput.value?.click();
}

function removeSelectedFile() {
  emit("file-selected", null);
  // Reset the file input
  if (fileInput.value) {
    fileInput.value.value = "";
  }
}

function processFile(file: File) {
  // Reset any previous errors
  fileError.value = "";

  // Validate file type
  const validTypes = ["image/jpeg", "image/jpg", "image/png", "image/gif", "image/webp"];
  if (!validTypes.includes(file.type)) {
    fileError.value = t("invalid_file_type");
    return;
  }

  // Validate file size (max 10MB)
  const maxSize = 10 * 1024 * 1024; // 10MB in bytes
  if (file.size > maxSize) {
    fileError.value = t("file_too_large");
    return;
  }

  const fileName = file.name || "uploaded-file";
  const reader = new FileReader();
  reader.onload = (e) => {
    const result = e.target?.result as string;
    if (result) {
      const base64 = result.split(",")[1];
      if (base64) {
        emit("file-selected", { base64, fileName });
      }
    }
  };
  reader.onerror = () => {
    fileError.value = t("file_read_error");
  };
  reader.readAsDataURL(file);
}
</script>

<template>
  <Modal v-model="modalOpen" :title="t('image_metadata_title')" class="!min-w-[90vw]" @close="emit('cancel')">
    <div class="space-y-4">
      <!-- File Upload Drop Zone -->
      <div>
        <label class="block text-sm font-medium text-zinc-700 mb-2"> {{ t("select_image") }} <span class="text-red-500">*</span> </label>
        <div
          ref="dropZone"
          :class="[
            'border-2 border-dashed rounded-xl p-12 text-center transition-all duration-300 cursor-pointer relative overflow-hidden',
            isDragOver
              ? 'border-blue-500 bg-blue-50 scale-105 shadow-lg'
              : selectedFile
                ? 'border-blue-400 bg-blue-50 hover:border-blue-500 hover:bg-blue-100'
                : 'border-zinc-300 bg-gradient-to-br from-zinc-50 to-zinc-100 hover:border-blue-400 hover:bg-gradient-to-br hover:from-blue-50 hover:to-zinc-100 hover:shadow-md',
          ]"
          @click="triggerFileInput"
          @dragover.prevent="handleDragOver"
          @dragleave.prevent="handleDragLeave"
          @drop.prevent="handleDrop"
        >
          <input ref="fileInput" type="file" accept="image/*" @change="handleFileSelect" class="hidden" />

          <div v-if="!selectedFile" class="space-y-3">
            <div :class="['transition-transform duration-300', isDragOver ? 'scale-110' : 'scale-100']">
              <MdiIcon :path="mdiImage" :size="64" class="mx-auto text-zinc-400" />
            </div>
            <div :class="['transition-all duration-300', isDragOver ? 'text-blue-600' : 'text-zinc-600']">
              <p class="text-lg font-medium">
                {{ isDragOver ? t("release_to_upload") : t("drop_image_here") }}
              </p>
              <p class="text-sm mt-1">{{ t("or_click_to_browse") }}</p>
              <p class="text-xs text-zinc-500 mt-2">{{ t("supported_formats", ["JPG, PNG, GIF, WebP (max 10MB)"]) }}</p>
            </div>
          </div>

          <div v-else class="space-y-3">
            <div>
              <MdiIcon :path="mdiFileCheck" :size="64" class="mx-auto text-blue-500" />
            </div>
            <div class="text-zinc-700">
              <p class="text-lg font-medium text-blue-700">{{ selectedFile.fileName }}</p>
              <p class="text-sm text-blue-600 mb-3">{{ t("file_selected_successfully") }}</p>
              <button
                @click.stop="removeSelectedFile"
                class="inline-flex items-center px-3 py-1 text-xs font-medium text-slate-500 bg-slate-50 border border-slate-200 rounded-full hover:bg-slate-100 hover:border-slate-300 transition-colors duration-200"
              >
                <MdiIcon :path="mdiClose" :size="12" class="mr-1" />
                {{ t("remove_file") }}
              </button>
            </div>
          </div>
          <!-- Error message -->
          <p v-if="fileError" class="text-sm text-red-600 mt-2 font-medium">{{ fileError }}</p>
        </div>
      </div>

      <!-- Author -->
      <div>
        <label class="block text-sm font-medium text-zinc-700 mb-1"> {{ t("image_author") }} <span class="text-red-500">*</span> </label>
        <input
          v-model="localMetadata.author"
          type="text"
          :placeholder="t('image_author_placeholder')"
          class="w-full px-3 py-2 border border-zinc-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          required
        />
      </div>

      <!-- License Info -->
      <div>
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div class="flex items-start">
            <MdiIcon :path="mdiInformation" :size="20" class="text-blue-500 mt-0.5 mr-2 flex-shrink-0" />
            <div>
              <p class="text-sm font-medium text-blue-800">{{ t("license_info_title") }}</p>
              <p class="text-sm text-blue-700 mt-1">{{ t("license_info_description") }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="flex justify-end gap-2 mt-6">
      <Btn variant="secondary" @click="emit('cancel')">
        {{ t("cancel") }}
      </Btn>
      <Btn variant="primary" @click="emit('confirm', localMetadata)" :disabled="!localMetadata.author || !selectedFile">
        {{ t("confirm") }}
      </Btn>
    </div>
  </Modal>
</template>

<i18n lang="yaml">
de:
  image_metadata_title: Ein neues Bild vorschlagen
  select_image: Bild auswählen
  drop_image_here: Bild hier ablegen
  release_to_upload: Loslassen zum Hochladen
  or_click_to_browse: oder zum Durchsuchen klicken
  supported_formats: Unterstützt {0}
  file_selected_successfully: Datei erfolgreich ausgewählt
  remove_file: Datei entfernen
  invalid_file_type: Ungültiger Dateityp. Bitte wähle ein Bild (JPG, PNG, GIF, WebP).
  file_too_large: Datei zu groß (max 10MB).
  file_read_error: Fehler beim Lesen der Datei.
  selected_file: Ausgewählte Datei
  image_author: Autor
  image_author_placeholder: Wer hat dieses Bild erstellt?
  license_info_title: CC BY 4.0 - Frei zu verwenden mit Namensnennung
  license_info_description: Alle hochgeladenen Bilder werden unter der CC BY 4.0 Lizenz veröffentlicht. Das bedeutet, jeder kann das Bild verwenden, solange du als Autor genannt wirst.
  confirm: Bild hinzufügen
  cancel: Abbrechen
en:
  image_metadata_title: Suggest a new Image
  select_image: Select Image
  drop_image_here: Drop image here
  release_to_upload: Release to upload
  or_click_to_browse: or click to browse
  supported_formats: Supports {0}
  file_selected_successfully: File selected successfully
  remove_file: Remove file
  invalid_file_type: Invalid file type. Please select an image (JPG, PNG, GIF, WebP).
  file_too_large: File too large (max 10MB).
  file_read_error: Error reading file.
  selected_file: Selected file
  image_author: Author
  image_author_placeholder: Who created this image?
  license_info_title: CC BY 4.0 - Free to use with attribution
  license_info_description: All uploaded images are published under the CC BY 4.0 license. This means anyone can use the image as long as they credit you as the author.
  confirm: Add image
  cancel: Cancel
</i18n>
