<template>
  <div v-if="show" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="cancel">
    <div class="bg-white rounded-lg p-6 m-4 max-w-lg w-full max-h-[90vh] overflow-y-auto">
      <h3 class="text-lg font-semibold mb-4 text-zinc-900">{{ t("image_metadata_title") }}</h3>

      <div class="space-y-4">
        <!-- Author -->
        <div>
          <label class="block text-sm font-medium text-zinc-700 mb-1">
            {{ t("image_author") }} <span class="text-red-500">*</span>
          </label>
          <input
            v-model="localMetadata.author"
            type="text"
            :placeholder="t('image_author_placeholder')"
            class="w-full px-3 py-2 border border-zinc-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            required
          />
        </div>

        <!-- License -->
        <div>
          <label class="block text-sm font-medium text-zinc-700 mb-1">
            {{ t("image_license") }} <span class="text-red-500">*</span>
          </label>
          <input
            v-model="localMetadata.license.text"
            type="text"
            :placeholder="t('image_license_placeholder')"
            class="w-full px-3 py-2 border border-zinc-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 mb-2"
            required
          />
          <input
            v-model="localMetadata.license.url"
            type="url"
            :placeholder="t('image_license_url_placeholder')"
            class="w-full px-3 py-2 border border-zinc-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <!-- Source -->
        <div>
          <label class="block text-sm font-medium text-zinc-700 mb-1">
            {{ t("image_source") }} <span class="text-red-500">*</span>
          </label>
          <input
            v-model="localMetadata.source.text"
            type="text"
            :placeholder="t('image_source_placeholder')"
            class="w-full px-3 py-2 border border-zinc-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 mb-2"
            required
          />
          <input
            v-model="localMetadata.source.url"
            type="url"
            :placeholder="t('image_source_url_placeholder')"
            class="w-full px-3 py-2 border border-zinc-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      <div class="flex gap-2 mt-6">
        <Btn
          variant="primary"
          @click="confirm"
          :disabled="!localMetadata.author || !localMetadata.license.text || !localMetadata.source.text"
        >
          {{ t("confirm_metadata") }}
        </Btn>
        <Btn variant="secondary" @click="cancel">
          {{ t("cancel") }}
        </Btn>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface ImageMetadata {
  author: string;
  license: { text: string; url: string };
  source: { text: string; url: string };
  offsets: { header: number | null; thumb: number | null };
}

interface Props {
  show: boolean;
  metadata: ImageMetadata;
}

interface Emits {
  (e: 'confirm', metadata: ImageMetadata): void;
  (e: 'cancel'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { t } = useI18n();

// Create a local copy of metadata to avoid mutating props
const localMetadata = ref<ImageMetadata>({
  author: "",
  license: { text: "", url: "" },
  source: { text: "", url: "" },
  offsets: { header: null, thumb: null },
});

// Watch for changes in props.metadata to update local copy
watch(() => props.metadata, (newMetadata) => {
  localMetadata.value = {
    author: newMetadata.author,
    license: { ...newMetadata.license },
    source: { ...newMetadata.source },
    offsets: { ...newMetadata.offsets },
  };
}, { immediate: true });

function confirm() {
  emit('confirm', localMetadata.value);
}

function cancel() {
  emit('cancel');
}
</script>

<i18n lang="yaml">
de:
  image_metadata_title: Bild-Metadaten
  image_author: Autor
  image_author_placeholder: Wer hat dieses Bild erstellt?
  image_license: Lizenz
  image_license_placeholder: z.B. CC BY 4.0, Eigenes Werk, etc.
  image_license_url_placeholder: Link zur Lizenz (optional)
  image_source: Quelle
  image_source_placeholder: Woher stammt dieses Bild?
  image_source_url_placeholder: Link zur Quelle (optional)
  confirm_metadata: Metadaten bestätigen
  cancel: Abbrechen
en:
  image_metadata_title: Image Metadata
  image_author: Author
  image_author_placeholder: Who created this image?
  image_license: License
  image_license_placeholder: e.g. CC BY 4.0, Own work, etc.
  image_license_url_placeholder: Link to license (optional)
  image_source: Source
  image_source_placeholder: Where does this image come from?
  image_source_url_placeholder: Link to source (optional)
  confirm_metadata: Confirm Metadata
  cancel: Cancel
</i18n>
