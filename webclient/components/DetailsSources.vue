<script setup lang="ts">
import { useDetailsStore } from "../stores/details";
import { useI18n } from "vue-i18n";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <section>
    <h2 class="text-md text-zinc-800 font-semibold">{{ t("title") }}</h2>
    <div class="text-zinc-600 text-sm">
      <p v-if="state.data?.sources.base">
        {{ t("base.title") }}:
        <span v-for="(e, i) in state.data.sources.base" :key="e.name">
          <a v-if="e.url" :href="e.url">{{ e.name }}</a>
          <template v-else>{{ e.name }}</template>
          <template v-if="i < state.data.sources.base.length - 1"> • </template>
        </span>
        <span v-if="state.data.sources.patched">
          <br />
          ({{ t("base.patched") }})
        </span>
      </p>
      <p v-if="state.image.shown_image">
        {{ t("header_img") }}:
        <span>{{ state.image.shown_image.author.text }}</span>
        <span v-if="state.image.shown_image.source">
          •
          <a v-if="state.image.shown_image.source.url" :href="state.image.shown_image.source.url" target="_blank">
            {{ state.image.shown_image.source.text }}
          </a>
          <template v-else>{{ state.image.shown_image.source.text }}</template>
        </span>
        <span v-if="state.image.shown_image.license">
          •
          <a v-if="state.image.shown_image.license.url" :href="state.image.shown_image.license.url" target="_blank">
            {{ state.image.shown_image.license.text }}
          </a>
          <template v-else>{{ state.image.shown_image.license.text }}</template>
        </span>
      </p>
      <p v-if="state.data?.coords">
        {{ t("coords.title") }}:
        <span v-if="state.data.coords.source === 'navigatum'"> {{ t("coords.navigatum") }}</span>
        <span v-if="state.data.coords.source === 'roomfinder'">
          {{ t("coords.roomfinder") }}
        </span>
        <span v-if="state.data.coords.source === 'inferred'">
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
