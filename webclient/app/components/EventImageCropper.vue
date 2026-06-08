<script setup lang="ts">
import {
  type CropTarget,
  clampCropOffset,
  cropAxis,
  cropOffsetBounds,
  cropRect,
} from "~/utils/imageCrop";

// Lets the user choose an `offsets.*` value (see `data/processors/images.py`): a crop of the
// target's aspect ratio slides along the over-long axis. The draggable frame shows what's kept; a
// source that already matches the target aspect has nothing to choose.
const offset = defineModel<number>({ required: true });
const props = defineProps<{
  imageUrl: string;
  width: number;
  height: number;
  target: CropTarget;
}>();

const { t } = useI18n({ useScope: "local" });

const maxOffset = computed(() => cropOffsetBounds(props.width, props.height, props.target).max);
const fixed = computed(() => maxOffset.value === 0);
const axis = computed(() => cropAxis(props.width, props.height, props.target));
const isHorizontal = computed(() => axis.value === "horizontal");

const imageEl = ref<HTMLImageElement>();

// The crop rectangle expressed as percentages of the displayed image, for the overlay frame.
const frame = computed(() => {
  const rect = cropRect(props.width, props.height, props.target, offset.value);
  return {
    left: `${(rect.x / props.width) * 100}%`,
    top: `${(rect.y / props.height) * 100}%`,
    width: `${(rect.width / props.width) * 100}%`,
    height: `${(rect.height / props.height) * 100}%`,
  };
});

let dragStart: { pointer: number; offset: number; axisPx: number } | null = null;

function onPointerDown(event: PointerEvent): void {
  if (fixed.value || !imageEl.value) return;
  const box = imageEl.value.getBoundingClientRect();
  dragStart = {
    pointer: isHorizontal.value ? event.clientX : event.clientY,
    offset: offset.value,
    axisPx: isHorizontal.value ? box.width : box.height,
  };
  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function onPointerMove(event: PointerEvent): void {
  if (!dragStart) return;
  const moved = (isHorizontal.value ? event.clientX : event.clientY) - dragStart.pointer;
  // Convert the on-screen drag into source pixels via the displayed-axis length.
  const sourceAxis = isHorizontal.value ? props.width : props.height;
  const delta = (moved / dragStart.axisPx) * sourceAxis;
  offset.value = clampCropOffset(props.width, props.height, props.target, dragStart.offset + delta);
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
      <!-- The frame's outward box-shadow dims everything outside the kept region. -->
      <div
        v-if="!fixed"
        class="absolute touch-none rounded-sm ring-2 ring-white/90 shadow-[0_0_0_9999px_rgba(0,0,0,0.45)]"
        :class="isHorizontal ? 'cursor-ew-resize' : 'cursor-ns-resize'"
        :style="frame"
        @pointerdown="onPointerDown"
      />
    </div>

    <template v-if="fixed">
      <p class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("fixed") }}</p>
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
        @input="offset = clampCropOffset(width, height, target, Number(($event.target as HTMLInputElement).value))"
      />
    </template>
  </div>
</template>

<i18n lang="yaml">
de:
  fixed: Das Seitenverhältnis passt bereits - das ganze Bild wird verwendet.
  aria_slider: Bildausschnitt verschieben
en:
  fixed: The aspect ratio already matches - the whole image is used.
  aria_slider: Move the image crop
</i18n>
