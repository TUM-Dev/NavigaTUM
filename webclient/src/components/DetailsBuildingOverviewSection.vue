<script setup lang="ts">
import { useToggle } from "@vueuse/core";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
import Button from "@/components/Button.vue";
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
    <h2>{{ t("title") }}</h2>
    <!--  <a class="no-underline" href="#">Übersichtskarte <i class="icon icon-forward" /> -->
    <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
      <template v-for="(b, i) in props.buildings.entries" :key="b.id">
        <RouterLink
          v-if="i < props.buildings.n_visible || buildingsExpanded"
          :to="'/view/' + b.id"
          class="!no-underline focusable flex flex-row justify-between border-neutral-200 dark:border-neutral-700 border-solid border p-3.5 rounded-sm hover:bg-zinc-50 dark:hover:bg-zinc-800 dark:bg-zinc-900"
          :aria-label="`show the details for the building '${b.name}'`"
        >
          <div class="flex flex-row gap-3">
            <figure class="h-12 w-12 flex justify-around">
              <img
                v-if="b.thumb"
                class="rounded-full aspect-square max-w-none"
                :alt="t('thumbnail_preview')"
                :src="`${appURL}/cdn/thumb/${b.thumb}`"
              />
              <img
                v-else
                class="rounded-full aspect-square max-w-none"
                :alt="t('default_thumbnail_preview')"
                src="@/assets/thumb-building.webp"
              />
            </figure>
            <div class="flex flex-col justify-evenly">
              <div>{{ b.name }}</div>
              <small class="text-zinc-600 dark:text-zinc-300">{{ b.subtext }}</small>
            </div>
          </div>
          <div class="flex-grow" />
          <div class="my-auto">
            <ChevronRightIcon class="h-4 w-4" />
          </div>
        </RouterLink>
      </template>
    </div>
    <div v-if="props.buildings.n_visible < props.buildings.entries.length" class="mt-2">
      <Button variant="link" @click="toggleBuildingsExpanded()">
        <template v-if="buildingsExpanded"> <ChevronUpIcon class="mt-0.5 h-4 w-4" /> {{ t("less") }} </template>
        <template v-else> <ChevronDownIcon class="h-4 w-4 mt-0.5" /> {{ t("more") }} </template>
      </Button>
    </div>
  </section>
</template>

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
