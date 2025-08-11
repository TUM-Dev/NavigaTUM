<template>
  <div v-if="show" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="emit('cancel')">
    <div class="bg-white rounded-lg p-6 m-4 max-w-lg w-full max-h-[90vh] overflow-y-auto">
      <h3 class="text-lg font-semibold mb-4 text-zinc-900">{{ t("image_metadata_title") }}</h3>

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
                <svg class="mx-auto h-16 w-16 text-zinc-400" stroke="currentColor" fill="none" viewBox="0 0 48 48">
                  <path
                    d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
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
                <svg class="mx-auto h-16 w-16 text-blue-500" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
              <div class="text-zinc-700">
                <p class="text-lg font-medium text-blue-700">{{ selectedFile.fileName }}</p>
                <p class="text-sm text-blue-600 mb-3">{{ t("file_selected_successfully") }}</p>
                <button
                  @click.stop="removeSelectedFile"
                  class="inline-flex items-center px-3 py-1 text-xs font-medium text-slate-500 bg-slate-50 border border-slate-200 rounded-full hover:bg-slate-100 hover:border-slate-300 transition-colors duration-200"
                >
                  <svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                    <path
                      fill-rule="evenodd"
                      d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                      clip-rule="evenodd"
                    />
                  </svg>
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
              <svg class="w-5 h-5 text-blue-500 mt-0.5 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fill-rule="evenodd"
                  d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                  clip-rule="evenodd"
                />
              </svg>
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
    </div>
  </div>
</template>

<script setup lang="ts">
import type { components } from "~/api_types";
import type { DeepWritable } from "ts-essentials";
type ImageMetadata = components["schemas"]["ImageMetadata"];

interface Props {
  show: boolean;
  metadata: DeepWritable<ImageMetadata>;
  selectedFile: { base64: string; fileName: string } | null;
}

interface Emits {
  (e: "confirm", metadata: ImageMetadata): void;
  (e: "cancel"): void;
  (e: "file-selected", file: { base64: string; fileName: string } | null): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { t } = useI18n();

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
  { immediate: true },
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

function handleDragOver(event: DragEvent) {
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
  invalid_file_type: Ungültiger Dateityp. Bitte wählen Sie ein Bild (JPG, PNG, GIF, WebP).
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
