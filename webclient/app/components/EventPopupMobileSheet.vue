<script setup lang="ts">
import { mdiClose } from "@mdi/js";
import type { EventPopupProps } from "~/composables/useEventMarkers";

const props = defineProps<{
  readonly event: EventPopupProps | null;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n({ useScope: "local" });

const open = computed<boolean>({
  get: () => props.event !== null,
  set: (value) => {
    if (!value) emit("close");
  },
});
</script>

<template>
  <Modal v-model="open" :title="''" chromeless>
    <div v-if="props.event" class="relative">
      <button
        type="button"
        :aria-label="t('close')"
        class="focusable bg-zinc-900/70 hover:bg-zinc-900/90 text-white absolute end-1.5 top-1.5 z-10 flex h-8 w-8 items-center justify-center rounded-full backdrop-blur-sm"
        @click="emit('close')"
      >
        <MdiIcon :path="mdiClose" :size="16" />
      </button>
      <EventPopupCard
        :name="props.event.name"
        :description="props.event.description"
        :image-path="props.event.imagePath"
        :image-author="props.event.imageAuthor"
        :starts-at="props.event.startsAt"
        :ends-at="props.event.endsAt"
        :org-code="props.event.orgCode"
        :org-name-de="props.event.orgNameDe"
        :org-name-en="props.event.orgNameEn"
      />
    </div>
  </Modal>
</template>

<i18n lang="yaml">
de:
  close: Veranstaltungsdetails schließen
en:
  close: Close event details
</i18n>
