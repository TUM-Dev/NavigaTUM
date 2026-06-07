<script setup lang="ts">
import { clampThumbOffset, thumbCropRect, thumbOffsetBounds } from "~/utils/imageCrop";

// Lets the user choose the `offsets.thumb` value (see `data/processors/images.py`): the 256² thumb
// is a `min(w, h)` square that slides along the longer axis. The frame shows what's kept; drag it
// or use the slider. A square source has nothing to choose, so the controls collapse to a note.
const offset = defineModel<number>({ required: true });
const props = defineProps<{
  imageUrl: string;
  width: number;
  height: number;
}>();

const { t } = useI18n({ useScope: "local" });

const maxOffset = computed(() => thumbOffsetBounds(props.width, props.height).max);
const isSquare = computed(() => maxOffset.value === 0);
const isLandscape = computed(() => props.width >= props.height);

const imageEl = ref<HTMLImageElement>();

// The crop square expressed as percentages of the displayed image, for the overlay frame.
const frame = computed(() => {
  const rect = thumbCropRect(props.width, props.height, offset.value);
  return {
    left: `${(rect.x / props.width) * 100}%`,
    top: `${(rect.y / props.height) * 100}%`,
    width: `${(rect.size / props.width) * 100}%`,
    height: `${(rect.size / props.height) * 100}%`,
  };
});

let dragStart: { pointer: number; offset: number; axisPx: number } | null = null;

function onPointerDown(event: PointerEvent): void {
  if (isSquare.value || !imageEl.value) return;
  const box = imageEl.value.getBoundingClientRect();
  dragStart = {
    pointer: isLandscape.value ? event.clientX : event.clientY,
    offset: offset.value,
    axisPx: isLandscape.value ? box.width : box.height,
  };
  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function onPointerMove(event: PointerEvent): void {
  if (!dragStart) return;
  const moved = (isLandscape.value ? event.clientX : event.clientY) - dragStart.pointer;
  // Convert the on-screen drag into source pixels via the displayed-axis length.
  const sourceAxis = isLandscape.value ? props.width : props.height;
  const delta = (moved / dragStart.axisPx) * sourceAxis;
  offset.value = clampThumbOffset(props.width, props.height, dragStart.offset + delta);
}

function onPointerUp(): void {
  dragStart = null;
  window.removeEventListener("pointermove", onPointerMove);
  window.removeEventListener("pointerup", onPointerUp);
}

onBeforeUnmount(onPointerUp);
</script>

<template>
  <div>
    <div class="border-zinc-300 dark:border-zinc-600 relative select-none overflow-hidden rounded border">
      <img
        ref="imageEl"
        :src="imageUrl"
        alt=""
        draggable="false"
        class="block w-full"
      />
      <!-- The frame's outward box-shadow dims everything outside the kept square. -->
      <div
        v-if="!isSquare"
        class="absolute touch-none rounded-sm ring-2 ring-white/90 shadow-[0_0_0_9999px_rgba(0,0,0,0.45)]"
        :class="isLandscape ? 'cursor-ew-resize' : 'cursor-ns-resize'"
        :style="frame"
        @pointerdown="onPointerDown"
      />
    </div>

    <template v-if="isSquare">
      <p class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("square") }}</p>
    </template>
    <template v-else>
      <input
        :value="offset"
        type="range"
        :min="-maxOffset"
        :max="maxOffset"
        step="1"
        :aria-label="t('aria_slider')"
        class="mt-2 w-full accent-blue-600 dark:accent-blue-400"
        @input="offset = clampThumbOffset(width, height, Number(($event.target as HTMLInputElement).value))"
      />
      <p class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("help") }}</p>
    </template>
  </div>
</template>

<i18n lang="yaml">
de:
  help: Ziehe den Ausschnitt oder nutze den Regler, um den quadratischen Karten-Marker zu wählen.
  square: Quadratisches Bild - der gesamte Ausschnitt wird verwendet.
  aria_slider: Bildausschnitt verschieben
en:
  help: Drag the frame or use the slider to choose the square shown on the map marker.
  square: Square image - the whole image is used.
  aria_slider: Move the image crop
</i18n>
