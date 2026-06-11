<script setup lang="ts">
import { mdiClose } from "@mdi/js";
import { onKeyStroke } from "@vueuse/core";
import type { EventPopupProps, ScreenPos } from "~/composables/useEventMarkers";
import { useIsMobile } from "~/composables/useIsMobile";

const props = defineProps<{
  readonly event: EventPopupProps | null;
  /** The marker's projected position within the map container the overlay is anchored to. */
  readonly screenPos: ScreenPos | null;
}>();
const emit = defineEmits<{ close: [] }>();

const isMobile = useIsMobile();
const { t } = useI18n({ useScope: "local" });

onKeyStroke("Escape", () => {
  if (props.event) emit("close");
});
</script>

<template>
  <div
    v-if="!isMobile && event && screenPos"
    class="pointer-events-none absolute z-20"
    :style="{ left: `${screenPos.x}px`, top: `${screenPos.y}px` }"
  >
    <div class="pointer-events-auto relative -mt-3 -translate-x-1/2 -translate-y-full">
      <button
        type="button"
        :aria-label="t('close')"
        class="focusable bg-zinc-900/70 hover:bg-zinc-900/90 text-white absolute end-1.5 top-1.5 z-10 flex h-7 w-7 items-center justify-center rounded-full backdrop-blur-sm"
        @click="emit('close')"
      >
        <MdiIcon :path="mdiClose" :size="14" />
      </button>
      <EventPopupCard
        :name="event.name"
        :description="event.description"
        :image-path="event.imagePath"
        :image-author="event.imageAuthor"
        :starts-at="event.startsAt"
        :ends-at="event.endsAt"
        :org-code="event.orgCode"
        :org-name-de="event.orgNameDe"
        :org-name-en="event.orgNameEn"
      />
    </div>
  </div>

  <EventPopupMobileSheet :event="isMobile ? event : null" @close="emit('close')" />
</template>

<i18n lang="yaml">
de:
  close: Veranstaltungsdetails schließen
en:
  close: Close event details
</i18n>
