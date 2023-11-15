<script setup lang="ts">
import { useToggle } from "@vueuse/core";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
type BuildingsOverview = components["schemas"]["BuildingsOverview"];

const props = defineProps<{
  readonly buildings?: BuildingsOverview;
}>();

const [buildingsExpanded, toggleBuildingsExpanded] = useToggle(false);
const { t } = useI18n({ useScope: "local" });
const appURL = import.meta.env.VITE_APP_URL;
</script>

<template>
  <section v-if="props.buildings">
    <div class="columns">
      <div class="column">
        <h2>{{ t("title") }}</h2>
      </div>
      <!-- <div class="column col-auto">
          <a href="#">Übersichtskarte <i class="icon icon-forward" /></a>
        </div> -->
    </div>
    <div class="columns">
      <template v-for="(b, i) in props.buildings.entries" :key="b.id">
        <div v-if="i < props.buildings.n_visible || buildingsExpanded" class="col-4 col-md-12 column content">
          <RouterLink :to="'/view/' + b.id">
            <div class="tile tile-centered">
              <div class="tile-icon">
                <figure class="avatar avatar-lg">
                  <img v-if="b.thumb" :alt="t('thumbnail_preview')" :src="`${appURL}/cdn/thumb/${b.thumb}`" />
                  <img v-else :alt="t('default_thumbnail_preview')" src="@/assets/thumb-building.webp" />
                </figure>
              </div>
              <div class="tile-content">
                <p class="tile-title">{{ b.name }}</p>
                <small class="text-dark tile-subtitle">{{ b.subtext }}</small>
              </div>
              <div class="tile-action">
                <button
                  type="button"
                  class="btn btn-link"
                  :aria-label="`show the details for the building '${b.name}'`"
                >
                  <i class="icon icon-arrow-right" />
                </button>
              </div>
            </div>
          </RouterLink>
        </div>
      </template>
    </div>
    <div v-if="props.buildings.n_visible < props.buildings.entries.length">
      <button type="button" class="btn btn-link" @click="toggleBuildingsExpanded()">
        <template v-if="buildingsExpanded">
          <i class="icon icon-arrow-up" />
          {{ t("less") }}
        </template>
        <template v-else>
          <i class="icon icon-arrow-right" />
          {{ t("more") }}
        </template>
      </button>
    </div>
  </section>
</template>

<style lang="scss" scoped>
@import "@/assets/variables";

a {
  text-decoration: none !important;
}

.tile {
  border: 0.05rem solid $card-border;
  padding: 8px;
  border-radius: 0.1rem;
}

button {
  margin-top: 8px;
}
</style>

<i18n lang="yaml">
de:
  default_thumbnail_preview: Standard-Thumbnail, da kein Thumbnail verfügbar ist
  less: weniger
  more: mehr
  thumbnail_preview: Thumbnail, das eine Vorschau des Gebäudes zeigt
  title: Gebäude / Gebiete
en:
  default_thumbnail_preview: Default-thumbnail, as no thumbnail is available
  less: less
  more: more
  thumbnail_preview: Thumbnail, showing a preview of the building
  title: Buildings / Areas
</i18n>
