<script setup lang="ts">
import { useToggle } from "@vueuse/core";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
import { ChevronRightIcon, ChevronDownIcon, ChevronUpIcon } from "@heroicons/vue/24/outline";
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
          <a class="no-underline" href="#">Übersichtskarte <i class="icon icon-forward" /></a>
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
                  class="btn btn-link mt-2"
                  :aria-label="`show the details for the building '${b.name}'`"
                >
                  <ChevronRightIcon class="h-4 w-4" />
                </button>
              </div>
            </div>
          </RouterLink>
        </div>
      </template>
    </div>
    <div v-if="props.buildings.n_visible < props.buildings.entries.length">
      <button type="button" class="btn btn-link mt-2" @click="toggleBuildingsExpanded()">
        <div class="flex flex-row gap-2">
          <template v-if="buildingsExpanded">
            <ChevronUpIcon class="h-4 w-4" />
            {{ t("less") }}
          </template>
          <template v-else>
            <ChevronDownIcon class="h-4 w-4" />
            {{ t("more") }}
          </template>
        </div>
      </button>
    </div>
  </section>
</template>

<style lang="postcss" scoped>
.tile {
  @apply border-neutral-200 dark:border-neutral-700 border-solid border p-2 rounded-sm;
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
