<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";

const state = useDetailsStore();
</script>

<template>
  <section id="entry-sources">
    <div class="columns">
      <div class="column">
        <h2>{{ $t("view_view.sources.title") }}</h2>
      </div>
    </div>
    <p v-if="state.data?.sources.base">
      {{ $t("view_view.sources.base.title") }}:
      <span v-for="(e, i) in state.data.sources.base" :key="e.name">
        <a v-if="e.url" :href="e.url">{{ e.name }}</a>
        <template v-else>{{ e.name }}</template>
        <template v-if="i < state.data.sources.base.length - 1">&#32;•&#32;</template>
      </span>
      <span v-if="state.data.sources.patched">
        <br />
        ({{ $t("view_view.sources.base.patched") }})
      </span>
    </p>
    <p v-if="state.image.shown_image">
      {{ $t("view_view.sources.header_img") }}:
      <span>{{ state.image.shown_image.author.text }}</span>
      <span v-if="state.image.shown_image.source"
        >•
        <a v-if="state.image.shown_image.source.url" :href="state.image.shown_image.source.url" target="_blank">
          {{ state.image.shown_image.source.text }}
        </a>
        <template v-else>{{ state.image.shown_image.source.text }}</template>
      </span>
      <span v-if="state.image.shown_image.license"
        >&#32;•
        <a v-if="state.image.shown_image.license.url" :href="state.image.shown_image.license.url" target="_blank">
          {{ state.image.shown_image.license.text }}
        </a>
        <template v-else>{{ state.image.shown_image.license.text }}</template>
      </span>
    </p>
    <p v-if="state.data?.coords">
      {{ $t("view_view.sources.coords.title") }}:
      <span v-if="state.data.coords.source === 'navigatum'"> {{ $t("view_view.sources.coords.navigatum") }}</span>
      <span v-if="state.data.coords.source === 'roomfinder'">
        {{ $t("view_view.sources.coords.roomfinder") }}
      </span>
      <span v-if="state.data.coords.source === 'inferred'">
        {{ $t("view_view.sources.coords.inferred") }}
      </span>
    </p>
  </section>
</template>

<style lang="scss">
@import "@/assets/variables";

#entry-sources {
  h2 {
    margin-bottom: 16px;
  }

  p {
    margin-bottom: 6px;
  }
}
</style>
