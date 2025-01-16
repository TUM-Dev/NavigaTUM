<script setup lang="ts">
import IndoorMap from "~/components/IndoorMap.vue";
import { ChevronLeftIcon } from "@heroicons/vue/16/solid";
import { firstOrDefault } from "~/composables/common";
import { useRouteQuery } from "@vueuse/router";
import Toast from "~/components/Toast.vue";
import { useTemplateRef } from "vue";
import type { operations } from "~/api_types";
import type { Ref } from "vue";

definePageMeta({
  layout: "navigation",
});

const indoorMap = useTemplateRef("indoorMap");
const route = useRoute();
const router = useRouter();
const { t, locale } = useI18n({ useScope: "local" });
const coming_from = computed<string>(() => firstOrDefault(route.query.coming_from, ""));
const selected_from = computed<string>(() => firstOrDefault(route.query.from, ""));
const selected_to = computed<string>(() => firstOrDefault(route.query.to, ""));
const mode = useRouteQuery<"bicycle" | "transit" | "motorcycle" | "car" | "pedestrian">("mode", "car", {
  mode: "replace",
  route,
  router,
});
const feedback = useFeedback();
type RequestQuery = operations["route_handler"]["parameters"]["query"];
type NavigationResponse = operations["route_handler"]["responses"][200]["content"]["application/json"];
const { data, status, error } = await useFetch<NavigationResponse>("https://nav.tum.de/api/maps/route", {
  query: {
    lang: locale as Ref<RequestQuery["lang"]>,
    from: selected_from as Ref<RequestQuery["from"]>,
    to: selected_to as Ref<RequestQuery["to"]>,
    route_costing: mode as Ref<RequestQuery["route_costing"]>,
    pedestrian_type: undefined as RequestQuery["pedestrian_type"],
    ptw_type: undefined as RequestQuery["ptw_type"],
    bicycle_type: undefined as RequestQuery["bicycle_type"],
  },
});
effect(() => {
  if (!data.value || !indoorMap.value) return;

  indoorMap.value.drawRoute(data.value.legs[0].shape);
});

function setBoundingBoxFromIndex(from_shape_index: number, to_shape_index: number) {
  if (!data.value) return;

  const coords = data.value.legs[0].shape.slice(from_shape_index, to_shape_index);
  const latitudes = coords.map((c) => c.lat);
  const longitudes = coords.map((c) => c.lon);
  indoorMap.value?.fitBounds(
    [Math.min(...longitudes), Math.max(...longitudes)],
    [Math.min(...latitudes), Math.max(...latitudes)],
  );
}
</script>

<template>
  <div class="flex min-h-[calc(100vh-150px)] flex-row">
    <div class="bg-zinc-100 flex min-w-96 max-w-96 flex-col gap-3 p-4">
      <NuxtLinkLocale
        v-if="coming_from"
        :to="'/view/' + coming_from"
        property="item"
        class="focusable text-blue-400 rounded-md pb-2 hover:text-blue-500 hover:underline"
      >
        <div class="my-auto flex flex-row gap-2">
          <ChevronLeftIcon class="h-4 w-4" />
          <span class="text-xs font-semibold uppercase">{{ t("back") }}</span>
        </div>
      </NuxtLinkLocale>
      <div class="end-0 flex flex-row justify-evenly">
        <Btn
          :variant="mode === 'pedestrian' ? 'primary' : 'secondary'"
          :disabled="mode == 'pedestrian'"
          size="md"
          :title="t('aria-pedestrian')"
          :aria-label="t('aria-pedestrian')"
          @click="mode = 'pedestrian'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
            <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
              <path d="M12 4a1 1 0 1 0 2 0a1 1 0 1 0-2 0M7 21l3-4m6 4l-2-4l-3-3l1-6" />
              <path d="m6 12l2-3l4-1l3 3l3 1" />
            </g>
          </svg>
        </Btn>
        <Btn
          :variant="mode == 'bicycle' ? 'primary' : 'secondary'"
          :disabled="mode == 'bicycle'"
          size="md"
          :title="t('aria-bicycle')"
          :aria-label="t('aria-bicycle')"
          @click="mode = 'bicycle'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
            <path
              fill="none"
              stroke="currentColor"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M2 18a3 3 0 1 0 6 0a3 3 0 1 0-6 0m14 0a3 3 0 1 0 6 0a3 3 0 1 0-6 0m-4 1v-4l-3-3l5-4l2 3h3m-3-6a1 1 0 1 0 2 0a1 1 0 1 0-2 0"
            />
          </svg>
        </Btn>
        <Btn
          :variant="mode === 'transit' ? 'primary' : 'secondary'"
          :disabled="true"
          size="md"
          :title="t('aria-transit')"
          :aria-label="t('aria-transit')"
          @click="mode = 'transit'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
            <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
              <path d="M21 13c0-3.87-3.37-7-10-7H3m0 9h16a2 2 0 0 0 2-2" />
              <path d="M3 6v5h17.5M3 10v4m5-3V6m5 5V6.5M3 19h18" />
            </g>
          </svg>
        </Btn>
        <Btn
          :variant="mode === 'motorcycle' ? 'primary' : 'secondary'"
          :disabled="mode == 'motorcycle'"
          size="md"
          :title="t('aria-motorcycle')"
          :aria-label="t('aria-motorcycle')"
          @click="mode = 'motorcycle'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
            <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
              <path d="M2 16a3 3 0 1 0 6 0a3 3 0 1 0-6 0m14 0a3 3 0 1 0 6 0a3 3 0 1 0-6 0m-8.5-2h5l4-4H6m1.5 4l4-4" />
              <path d="M13 6h2l1.5 3l2 4" />
            </g>
          </svg>
        </Btn>
        <Btn
          :variant="mode === 'car' ? 'primary' : 'secondary'"
          :disabled="mode == 'car'"
          size="md"
          :title="t('aria-car')"
          :aria-label="t('aria-car')"
          @click="mode = 'car'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
            <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
              <path d="M5 17a2 2 0 1 0 4 0a2 2 0 1 0-4 0m10 0a2 2 0 1 0 4 0a2 2 0 1 0-4 0" />
              <path d="M5 17H3v-6l2-5h9l4 5h1a2 2 0 0 1 2 2v4h-2m-4 0H9m-6-6h15m-6 0V6" />
            </g>
          </svg>
        </Btn>
      </div>
      <form action="/navigate" autocomplete="off" method="GET" role="search" class="flex flex-col gap-2">
        <NavigationSearchBar query-id="from" />
        <NavigationSearchBar query-id="to" />
      </form>
      <div class="overflow-auto">
        <div v-if="status === 'success' && !!data">
          <div v-for="(l, i) in data.legs" :key="i" class="gap-1">
            <p class="text-zinc-500 mt-3 flex items-center gap-5 pb-4 font-semibold">
              <span>{{ t("meters", l.summary.length_meters) }}</span>
              <span class="border-zinc-500 flex-grow border-t" />
              <span>{{ t("minutes", Math.ceil(l.summary.time_seconds / 60)) }}</span>
            </p>
            <div
              v-for="(m, j) in l.maneuvers"
              :key="j"
              class="group cursor-pointer py-1"
              @click="setBoundingBoxFromIndex(m.begin_shape_index, m.end_shape_index)"
            >
              <div class="bg-zinc-200 flex flex-row items-center gap-3 rounded-md p-2 py-1 group-hover:bg-zinc-300">
                <RoutingManeuverIcon :type="m.type" />
                <div>
                  <div class="text-zinc-900">{{ m.instruction }}</div>
                </div>
              </div>
              <small v-if="m.length_meters" class="text-zinc-500">{{ t("meters", m.length_meters) }}</small>
            </div>
          </div>
        </div>
        <Toast v-else id="nav-error" level="error">
          <p class="text-zinc-900">status:{{ status }}</p>
          <p class="text-zinc-900">error:{{ error }}</p>
        </Toast>
      </div>
      <Toast id="nav-disclaimer" level="warning">
        {{ t("disclaimer_0") }}:
        <ul class="ms-5 list-outside list-disc">
          <I18nT tag="li" keypath="disclaimer_1">
            <template #cta>
              <b class="font-bold">{{ t("disclaimer_1_cta") }}</b>
            </template>
          </I18nT>
          <I18nT tag="li" keypath="disclaimer_2">
            <template #cta>
              <b class="font-bold">{{ t("disclaimer_2_cta") }}</b>
            </template>
          </I18nT>
          <I18nT tag="li" keypath="disclaimer_3">
            <template #cta>
              <b class="font-bold">{{ t("disclaimer_3_cta") }}</b>
            </template>
          </I18nT>
        </ul>
        <Btn
          variant="link"
          :aria-label="t('open-feedback-form')"
          :title="t('open-feedback-form')"
          @click="
            () => {
              feedback.open = true;
              feedback.data = {
                category: 'navigation',
                subject: `navigation from \`${selected_from}\` to \`${selected_to}\``,
                body: !!coming_from ? t('got_here_and_found_issues', coming_from) : t('found_issues'),
                deletion_requested: false,
              };
            }
          "
        >
          {{ t("disclaimer_cta") }}
        </Btn>
      </Toast>
    </div>
    <div class="grow">
      <IndoorMap ref="indoorMap" type="room" :coords="{ lat: 0, lon: 0, source: 'navigatum' }" />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  back: zurück
  aria-motorcycle: Motorrad
  aria-bicycle: Fahrrad
  aria-transit: Transit
  aria-car: Auto
  aria-pedestrian: Fußgänger
  disclaimer_0: Dies ist derzeit in einer Beta-Phase. Die folgenden Punkte sind noch nicht implementiert
  disclaimer_1_cta: Transit-Routing
  disclaimer_1: "{cta}, da wir noch keine Möglichkeit gefunden haben, die Daten der MVVs einzubeziehen"
  disclaimer_2_cta: Routenplanung für Innenräume
  disclaimer_2: "{cta}, da der Import der CAD-Daten und die Implementierung des barrierefreien Routings noch nicht abgeschlossen sind"
  disclaimer_3_cta: Abkürzungen im Innenbereich
  disclaimer_3: Wegen der Nichtberücksichtigung von {0} könnten die Routen suboptimal sein
  disclaimer_cta: Wir würden wir uns trotzdem über feedback freuen
  open-feedback-form: Öffnet das Feedback-Formular
  found_issues: "Ich habe diese Probleme gefunden:"
  got_here_and_found_issues: "Ich habe die Navigation via {0} gefunden und mir ist dieses Problem aufgefallen:"
  minutes: "sofort | eine Minute | {count} Minuten"
  meters: "hier | einen Meter | {count} Meter"
en:
  back: back
  aria-motorcycle: Motorcycle
  aria-bicycle: Bicycle
  aria-transit: Transit
  aria-car: Car
  aria-pedestrian: Pedestrian
  disclaimer_0: This is currently in a beta stage. These are the issues that are currently not implemented
  disclaimer_1_cta: Transit routing
  disclaimer_1: "{cta} as we have not found a way to incorporate the MVVs data yet"
  disclaimer_2_cta: Indoor routing
  disclaimer_2: "{cta} as the CAD-data import and accessible routing implementation is not yet done"
  disclaimer_3_cta: indoor shortcuts
  disclaimer_3: Because of not considering {cta}, routes might be suboptimal
  disclaimer_cta: We would still appreciate feedback
  open-feedback-form: Open the feedback form
  found_issues: "I have found these problems:"
  got_here_and_found_issues: "I found the navigation via {0} and I noticed these problems:"
  minutes: "instant | one minutes | {count} minutes"
  meters: "here | one meter | {count} meters"
</i18n>
