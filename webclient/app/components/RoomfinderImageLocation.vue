<script setup lang="ts">
import { useInterval } from "@vueuse/core";
import type { components } from "~/api_types";

type RoomfinderMapEntryResponse = components["schemas"]["RoomfinderMapEntryResponse"];

const props = defineProps<{
  map: RoomfinderMapEntryResponse;
  id: string;
}>();

const { t } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();

// count will increase every 150ms
const counter = useInterval(150);
const animationColors = [
  "#ff6666",
  "#f19f9f",
  "#f8cfcf",
  "#ffffff",
  "#e4efff",
  "#aecdff",
  "#70a5ff",
  "#3984ff",
  "#0062ff",
  "#3984ff",
  "#70a5ff",
  "#aecdff",
  "#e4efff",
  "#ffffff",
  "#f8cfcf",
  "#f19f9f",
  "#ff6666",
  "#ff5151",
];
watch(counter, () => {
  const ctx = getContext();
  if (ctx == null) return;

  const size = 10;
  const outerBorder = 2;
  //outer
  ctx.fillStyle = "#fff";
  ctx?.fillRect(
    props.map.x - size / 2 - outerBorder,
    props.map.y - size / 2 - outerBorder,
    size + outerBorder * 2,
    size + outerBorder * 2
  );
  // inner
  ctx.fillStyle = animationColors[counter.value % animationColors.length] ?? "#ffffff";
  ctx?.fillRect(props.map.x - size / 2, props.map.y - size / 2, size, size);
  // stripes
  ctx.setLineDash([1, 2]);
  ctx.beginPath();

  ctx.moveTo(props.map.x, 0);
  ctx.lineTo(props.map.x, props.map.y - size);
  ctx.moveTo(props.map.x, props.map.y + size);
  ctx.lineTo(props.map.x, props.map.height);

  ctx.moveTo(0, props.map.y);
  ctx.lineTo(props.map.x - size, props.map.y);
  ctx.moveTo(props.map.x + size, props.map.y);
  ctx.lineTo(props.map.width, props.map.y);

  ctx.stroke();
});

function getContext(): CanvasRenderingContext2D | null {
  const canvas = document.getElementById(props.id) as HTMLCanvasElement;
  return canvas?.getContext("2d");
}

function draw() {
  const ctx = getContext();
  if (ctx == null) return;
  const mapURL = new URL(
    `${runtimeConfig.public.cdnURL}/cdn/maps/site_plans/${props.map.file}`,
    import.meta.url
  );
  const mapSprite = new Image();
  mapSprite.src = mapURL.href;

  mapSprite.addEventListener("load", () => {
    ctx?.drawImage(mapSprite, 0, 0);
    ctx.textAlign = "end";
    ctx.font = "12px sans-serif";
    const txt = `${t("img_source")}: ${props.map.source}`;
    const measurement = ctx.measureText(txt);
    ctx.fillStyle = "#fafafa";
    ctx?.fillRect(
      props.map.width - measurement.width - 10,
      props.map.height - 20,
      measurement.width + 10,
      20
    );
    ctx.fillStyle = "#374151";
    ctx.fillText(txt, props.map.width - 5, props.map.height - 5);
  });
}

watch(props, draw);
onMounted(draw);
</script>

<template>
  <div
    class="mx-auto print:!max-w-40"
    :class="{
      'max-w-sm': map.height > map.width,
      'max-w-2xl': map.height <= map.width,
    }"
  >
    <canvas :id="props.id" class="w-full" :width="map.width" :height="map.height" />
  </div>
</template>

<i18n lang="yaml">
de:
  img_alt: Bild des Lageplans
  img_source: Bildquelle
en:
  img_alt: Image showing the Site Plan
  img_source: Image source
</i18n>
