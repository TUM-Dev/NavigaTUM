<script setup lang="ts">
import { setTitle } from "@/composables/common";
import { useFetch } from "@/composables/fetch";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
import { MapPinIcon } from "@heroicons/vue/24/outline";
import { ArrowRightIcon, ChevronRightIcon, ChevronDownIcon, ChevronUpIcon } from "@heroicons/vue/24/solid";
import { ref } from "vue";
import Btn from "@/components/Btn.vue";
import Spinner from "@/components/Spinner.vue";
import Toast from "@/components/Toast.vue";
import { useGlobalStore } from "@/stores/global";
type RootResponse = components["schemas"]["RootResponse"];

const { t } = useI18n({ useScope: "local" });
const { data } = useFetch<RootResponse>(`/api/get/root`, (d) => setTitle(d.name));
const openPanels = ref<(boolean | undefined)[]>([]);
const global = useGlobalStore();
</script>

<template>
  <div class="flex flex-col justify-between gap-3 pt-8">
    <Toast level="info">
      {{ t("toast.released_many_changes") }}
      <Btn
        variant="link"
        size="ms-0 rounded-sm"
        :aria-label="t('feedback.open')"
        @click="global.openFeedback('general', t('toast.feedback_subject'), t('toast.feedback_body'))"
      >
        {{ t("toast.call_to_action") }}
      </Btn>
    </Toast>
    <div class="text-zinc-600 !text-lg font-semibold">{{ t("sites") }}</div>
    <!-- <a href="#" class="flex flex-row"><MapPinIcon class="h-4 w-4" /> {{ t("overview_map") }}</a> -->
  </div>
  <div v-if="data" class="mt-5">
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <div
        v-for="(site, siteIndex) in data.sites_overview"
        :key="site.id"
        class="border-zinc-200 flex flex-col gap-4 rounded-lg border-2 p-5"
      >
        <div>
          <RouterLink
            v-if="site.id"
            :to="'/view/' + site.id"
            :aria-label="t('show_details_for_campus', [site.name])"
            class="focusable text-zinc-700 flex grow-0 flex-row justify-between rounded !no-underline hover:text-tumBlue-500"
          >
            <span class="text-md font-semibold">{{ site.name }}</span>
            <ArrowRightIcon v-if="site.id" class="my-auto hidden h-6 w-6 md:block" />
          </RouterLink>
          <div v-else class="text-md text-zinc-700 font-semibold">{{ site.name }}</div>
        </div>
        <div class="flex flex-col gap-3">
          <RouterLink
            v-for="c in site.children.slice(0, openPanels[siteIndex] ? site.children.length : site.n_visible)"
            :key="c.id"
            :to="'/view/' + c.id"
            :aria-label="t('show_details_for_building', [c.name])"
            class="focusable text-tumBlue-600 flex flex-row justify-between rounded !no-underline hover:text-tumBlue-500"
          >
            <div class="flex flex-row gap-2">
              <MapPinIcon class="h-4 w-5" />
              <span>{{ c.name }}</span>
            </div>
            <ChevronRightIcon class="my-auto hidden h-4 w-4 sm:block" />
          </RouterLink>
          <div v-if="site.children.length > site.n_visible" class="mx-auto">
            <Btn
              v-if="openPanels[siteIndex]"
              variant="linkButton"
              :aria-label="t('less_aria')"
              @click="() => (openPanels[siteIndex] = false)"
            >
              <ChevronUpIcon class="h-4 w-4" />
              {{ t("less") }}
            </Btn>
            <Btn v-else variant="linkButton" :aria-label="t('more_aria')" @click="() => (openPanels[siteIndex] = true)">
              <ChevronDownIcon class="my-auto h-4 w-4" />
              {{ t("more") }}
            </Btn>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div v-else class="text-zinc-900 flex flex-col items-center gap-5 py-32">
    <Spinner class="h-8 w-8" />
    {{ t("Loading data...") }}
  </div>
</template>

<i18n lang="yaml">
de:
  less: weniger
  less_aria: weniger Gebäude anzeigen
  more: mehr
  more_aria: mehr Gebäude anzeigen
  overview_map: Übersichtskarte
  sites: Standorte
  "Loading data...": Lädt daten...
  show_details_for_campus: show the details for the campus '{0}'
  show_details_for_building: show the details for the building '{0}'
  toast:
    released_many_changes: Wir haben vor ein paar Tagen eine neue Version unseres Frontends mit einer Vielzahl von Änderungen veröffentlicht.
    feedback_subject: Feedback zum neuen Frontend
    feedback_body: |
      Es gefällt mir, dass:
      - Detail 1
      - Einzelheit 2

      Ich denke, das sollte verbessert werden:
      - Verbesserung 1
      - Verbesserung 2
    call_to_action: Gibt es etwas, das du nicht gut findest? Erzähle uns bitte davon!
en:
  less: less
  less_aria: show more buildings
  more: more
  more_aria: show more buildings
  overview_map: Overview Map
  sites: Sites
  "Loading data...": Loading data...
  show_details_for_campus: show the details for the campus '{0}'
  show_details_for_building: show the details for the building '{0}'
  toast:
    released_many_changes: We have recently released a new version of our frontend with a ton of changes.
    feedback_subject: Feedback about new Frontend
    feedback_body: |
      I like:
      - detail 1
      - detail 2

      I think this should be improved:
      - improvement 1
      - improvement 2
    call_to_action: Is there something you don't like? Please tell us about it!
</i18n>
