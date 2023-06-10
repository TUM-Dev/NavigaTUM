<script setup lang="ts">
import { useToggle } from "@vueuse/core";
import type { components } from "@/api_types";
type BuildingsOverview = components["schemas"]["BuildingsOverview"];

const props = defineProps<{
  readonly buildings?: BuildingsOverview;
}>();

const [buildingsExpanded, toggleBuildingsExpanded] = useToggle(false);
</script>

<template>
  <section v-if="props.buildings">
    <div class="columns">
      <div class="column">
        <h2>{{ $t("view_view.buildings_overview.title") }}</h2>
      </div>
      <!--<div class="column col-auto">
          <a href="#">Übersichtskarte <i class="icon icon-forward" /></a>
        </div>-->
    </div>
    <div class="columns">
      <template v-for="(b, i) in props.buildings.entries" :key="b.id">
        <div class="column col-4 col-md-12 content" v-if="i < props.buildings.n_visible || buildingsExpanded">
          <RouterLink :to="'/view/' + b.id">
            <div class="tile tile-centered">
              <div class="tile-icon">
                <figure class="avatar avatar-lg">
                  <img
                    v-if="b.thumb"
                    :alt="$t('view_view.buildings_overview.thumbnail_preview')"
                    :src="'/cdn/thumb/' + b.thumb"
                  />
                  <img
                    v-else
                    :alt="$t('view_view.buildings_overview.default_thumbnail_preview')"
                    src="../assets/thumb-building.webp"
                  />
                </figure>
              </div>
              <div class="tile-content">
                <p class="tile-title">{{ b.name }}</p>
                <small class="tile-subtitle text-dark">{{ b.subtext }}</small>
              </div>
              <div class="tile-action">
                <button class="btn btn-link" :aria-label="`show the details for the building '${b.name}'`">
                  <i class="icon icon-arrow-right" />
                </button>
              </div>
            </div>
          </RouterLink>
        </div>
      </template>
    </div>
    <div v-if="props.buildings.n_visible < props.buildings.entries.length">
      <button class="btn btn-link" @click="toggleBuildingsExpanded()">
        <template v-if="buildingsExpanded">
          <i class="icon icon-arrow-up" />
          {{ $t("view_view.buildings_overview.less") }}
        </template>
        <template v-else>
          <i class="icon icon-arrow-right" />
          {{ $t("view_view.buildings_overview.more") }}
        </template>
      </button>
    </div>
  </section>
</template>

<style lang="scss" scoped>
@import "../assets/variables";

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
