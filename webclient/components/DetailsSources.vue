<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { components } from "../api_types";

type ImageInfo = components["schemas"]["ImageInfo"];
type Coordinate = components["schemas"]["Coordinate"];
type DataSources = components["schemas"]["DataSources"];

defineProps<{
  coords: Coordinate;
  sources: DataSources;
  shownImage?: ImageInfo;
}>();

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <section>
    <h2 class="text-md text-zinc-800 font-semibold">{{ t("title") }}</h2>
    <div class="text-zinc-600 text-sm">
      <p>
        {{ t("base.title") }}:
        <span v-for="(e, i) in sources.base" :key="e.name">
          <a v-if="e.url" :href="e.url">{{ e.name }}</a>
          <template v-else>{{ e.name }}</template>
          <template v-if="i < sources.base.length - 1"> • </template>
        </span>
        <span v-if="sources.patched">
          <br />
          ({{ t("base.patched") }})
        </span>
      </p>
      <p v-if="shownImage">
        {{ t("header_img") }}:
        <span>{{ shownImage.author.text }}</span>
        <span v-if="shownImage.source">
          •
          <a v-if="shownImage.source.url" :href="shownImage.source.url" target="_blank">
            {{ shownImage.source.text }}
          </a>
          <template v-else>{{ shownImage.source.text }}</template>
        </span>
        <span v-if="shownImage.license">
          •
          <a v-if="shownImage.license.url" :href="shownImage.license.url" target="_blank">
            {{ shownImage.license.text }}
          </a>
          <template v-else>{{ shownImage.license.text }}</template>
        </span>
      </p>
      <p>
        {{ t("coords.title") }}:
        <span v-if="coords.source === 'navigatum'"> {{ t("coords.navigatum") }}</span>
        <span v-if="coords.source === 'roomfinder'">
          {{ t("coords.roomfinder") }}
        </span>
        <span v-if="coords.source === 'inferred'">
          {{ t("coords.inferred") }}
        </span>
      </p>
    </div>
  </section>
</template>

<i18n lang="yaml">
de:
  base:
    patched: Bei diesem Eintrag wurden automatische Korrekturen zu externen Daten angewandt
    title: Basisdaten
  coords:
    inferred: Automatisch berechnet aus den zugehörigen Räumen oder Gebäuden
    navigatum: NavigaTUM Mitwirkende
    roomfinder: Roomfinder
    title: Koordinaten
  header_img: Bild
  title: Quellen
en:
  base:
    patched: For this entry automatic patches were applied to external data
    title: Base data
  coords:
    inferred: Automatically computed based on the associated rooms or buildings
    navigatum: NavigaTUM Contributors
    roomfinder: Roomfinder
    title: Coordinates
  header_img: Image
  title: Sources
</i18n>
