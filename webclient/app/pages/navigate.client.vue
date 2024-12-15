<script setup lang="ts">
import IndoorMap from "~/components/IndoorMap.vue";
import { ChevronLeftIcon } from "@heroicons/vue/16/solid";
import { firstOrDefault } from "~/composables/common";
import { useRouteQuery } from "@vueuse/router";
import Toast from "~/components/Toast.vue";

definePageMeta({
  layout: "navigation",
});

const route = useRoute();
const router = useRouter();
const { t } = useI18n({ useScope: "local" });
const coming_from = computed<string>(() => firstOrDefault(route.query.coming_from, ""));
const selected_from = computed<string>(() => firstOrDefault(route.query.from, ""));
const selected_to = computed<string>(() => firstOrDefault(route.query.to, ""));
const mode = useRouteQuery<"bike" | "transit" | "motorbike" | "car" | "pedestrian">("mode", "car", {
  mode: "replace",
  route,
  router,
});
const feedback = useFeedback();
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
          :variant="mode == 'bike' ? 'primary' : 'secondary'"
          :disabled="mode == 'bike'"
          size="md"
          :title="t('aria-bike')"
          :aria-label="t('aria-bike')"
          @click="mode = 'bike'"
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
          :variant="mode === 'motorbike' ? 'primary' : 'secondary'"
          :disabled="mode == 'motorbike'"
          size="md"
          :title="t('aria-motorbike')"
          :aria-label="t('aria-motorbike')"
          @click="mode = 'motorbike'"
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
      <p class="text-zinc-900">Navigating from '{{ selected_from }}' to '{{ selected_to }}' via '{{ mode }}'</p>
    </div>
    <div class="grow">
      <IndoorMap type="room" :coords="{ lat: 0, lon: 0, source: 'navigatum' }" />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  back: zurück
  aria-motorbike: Motorrad
  aria-bike: Fahrrad
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
  got_here_and_found_issues: "Ich habe die navigation via {0} gefunden und mir ist dieses Problem aufgefallen:"
en:
  back: back
  aria-motorbike: Motorbike
  aria-bike: Bike
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
</i18n>
