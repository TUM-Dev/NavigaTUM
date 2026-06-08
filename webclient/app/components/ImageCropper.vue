<script setup lang="ts">
import { useEventListener } from "@vueuse/core";
import {
  type CropTarget,
  clampCropOffset,
  cropAxis,
  cropOffsetBounds,
  cropRect,
} from "~/utils/imageCrop";

const offset = defineModel<number>({ required: true });
const props = defineProps<{
  // Undefined upstream is guarded by `v-if="previewUrl"` at the parent's call site; widening the
  // type here just keeps that template strict-typeable.
  imageUrl: string | undefined;
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

const frame = computed(() => {
  const rect = cropRect(props.width, props.height, props.target, offset.value);
  return {
    left: `${(rect.x / props.width) * 100}%`,
    top: `${(rect.y / props.height) * 100}%`,
    width: `${(rect.width / props.width) * 100}%`,
    height: `${(rect.height / props.height) * 100}%`,
  };
});

interface DragStart {
  pointer: number;
  offset: number;
  axisPx: number;
}
// shallowRef rather than a `let`: an outside-component cancellation (e.g. parent unmount mid-drag)
// only needs to null this for the global listeners below to short-circuit.
const dragStart = shallowRef<DragStart | null>(null);

function onPointerDown(event: PointerEvent): void {
  if (fixed.value || !imageEl.value) return;
  const box = imageEl.value.getBoundingClientRect();
  dragStart.value = {
    pointer: isHorizontal.value ? event.clientX : event.clientY,
    offset: offset.value,
    axisPx: isHorizontal.value ? box.width : box.height,
  };
}

// Window-scoped listeners ride the component's effect scope, so `onBeforeUnmount` cleanup is
// implicit; gating on `dragStart.value` makes the always-bound move handler a no-op when idle.
useEventListener(window, "pointermove", (event: PointerEvent) => {
  const start = dragStart.value;
  if (!start) return;
  const moved = (isHorizontal.value ? event.clientX : event.clientY) - start.pointer;
  const sourceAxis = isHorizontal.value ? props.width : props.height;
  const delta = (moved / start.axisPx) * sourceAxis;
  offset.value = clampCropOffset(props.width, props.height, props.target, start.offset + delta);
});
useEventListener(window, "pointerup", () => {
  dragStart.value = null;
});
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
