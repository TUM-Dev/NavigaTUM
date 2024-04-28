<script setup lang="ts">
// @ts-expect-error
import { CalendarView, CalendarViewHeader } from "vue-simple-calendar";
// @ts-expect-error
import type { ICalendarItem } from "vue-simple-calendar/dist/src/ICalendarItem.d.ts";

import type { components } from "@/api_types";
import "/node_modules/vue-simple-calendar/dist/style.css";
import "/node_modules/vue-simple-calendar/dist/css/gcal.css";

type CalendarResponse = components["schemas"]["CalendarResponse"];

const showDate = ref(new Date());

const calendar = useCalendar();
const { t, locale } = useI18n({ useScope: "local" });

const start_after = computed(() => {
  const start = new Date(showDate.value);
  start.setDate(start.getDate() - 60);
  return start.toISOString();
});
const end_before = computed(() => {
  const start = new Date(showDate.value);
  start.setDate(start.getDate() + 60);
  return start.toISOString();
});

const runtimeConfig = useRuntimeConfig();
const url = computed(() => {
  const params = new URLSearchParams();
  params.append("start_after", start_after.value);
  params.append("end_before", end_before.value);

  return `${runtimeConfig.public.apiURL}/api/calendar/${calendar.value.showing[0]}?${params.toString()}`;
});
const { data, status, error } = useFetch<CalendarResponse>(url, { key: "calendar", dedupe: "defer", deep: false });
const earliest_last_sync = computed<string | null>(() => {
  if (!data.value) return null;
  return Object.values(data.value)
    .map((d) => new Date(d.location.last_calendar_scrape_at))
    .reduce((d1, d2) => (d1 < d2 ? d1 : d2))
    .toLocaleString(locale.value, { timeStyle: "short", dateStyle: "short" });
});
const events = computed<ICalendarItem[]>(() => {
  if (!data.value) return [];
  return data.value.events.map((e) => ({
    id: e.id.toString(),
    title: e.title,
    startDate: new Date(e.start),
    endDate: new Date(e.end),
    classes: [e.entry_type],
  }));
});

function setShowDate(d: Date) {
  showDate.value = d;
}
</script>

<template>
  <LazyModal v-model="calendar.open" :title="t('title')">
    <Toast v-if="error" level="error">
      <p class="text-md font-bold">{{ t("error.header") }}</p>
      <p class="text-sm">
        {{ t("error.reason") }}:<br />
        <code
          class="text-red-900 bg-red-200 mb-1 mt-2 inline-flex max-w-full items-center space-x-2 overflow-auto rounded-md px-4 py-3 text-left font-mono text-xs dark:bg-red-50/20"
        >
          {{ error }}
        </code>
      </p>
      <p class="text-sm">
        {{ t("error.call_to_action") }}
      </p>
    </Toast>
    <div v-else>
      <CalendarView
        v-if="data"
        :items="events"
        :show-date="showDate"
        :show-times="true"
        :time-format-options="{ hour: '2-digit', minute: '2-digit', hour12: false }"
        :starting-day-of-week="1"
        item-top="2.5em"
        class="theme-gcal flex grow flex-col"
      >
        <template #header="{ headerProps }">
          <CalendarViewHeader :header-props="headerProps" @input="setShowDate" />
        </template>
      </CalendarView>
      <div v-else-if="status === 'pending'" class="text-zinc-900 flex flex-col items-center gap-5 py-32">
        <Spinner class="h-8 w-8" />
        {{ t("Loading data...") }}
      </div>

      <div class="pt-2 text-xs">
        {{ t("footer.disclaimer") }} <br />
        {{ t("footer.please_check") }}
        <template v-if="earliest_last_sync !== null">
          <br />
          {{ t("footer.last_sync", [earliest_last_sync]) }}
        </template>
      </div>
    </div>
  </LazyModal>
</template>

<style lang="postcss" scoped>
.startTime,
.endTime {
  @apply text-white text-[0.5rem];
}

.cv-day.today .cv-day-number {
  @apply text-white mt-[0.1em] bg-[#0065bd];
}
.dark .cv-day.today .cv-day-number {
  @apply bg-[#59b2ff] text-[#17181a];
}
.cv-day-number,
.periodLabel,
.currentPeriod,
.cv-header-day {
  @apply text-zinc-900;
}
.currentPeriod {
  @apply bg-transparent;
}
.cv-item {
  @apply pb-4 pt-[0.1em];
}
.past.cv-item {
  @apply brightness-[1.3] grayscale-[0.55];
}

/* colors */
.barred {
  @apply text-red-900 bg-red-100 border-red-300;
}

.lecture {
  @apply text-blue-900 bg-blue-100 border-blue-300;
}

.exercise {
  @apply text-orange-900 bg-orange-100 border-orange-300;
}
.exam {
  @apply text-fuchsia-pink-900 bg-fuchsia-pink-100 border-fuchsia-pink-300;
}

.other {
  @apply text-zinc-900 bg-zinc-100 border-zinc-300;
}
</style>

<i18n lang="yaml">
de:
  title: Kalendar
  close: Schließen
  Loading data...: Lädt daten...
  error:
    header: Beim Versuch, den Kalender anzuzeigen, ist ein Fehler aufgetreten
    reason: Der Grund für diesen Fehler ist
    call_to_action: Wenn dieses Problem weiterhin besteht, kontaktieren Sie uns bitte über das Feedback-Formular am Ende der Seite.
  footer:
    disclaimer: Stündlich aktualisiert und identische Termine zusammengefasst.
    please_check: Im Zweifelsfall prüfe bitte den offiziellen TUMonline-Kalender.
    last_sync: Stand {0}
en:
  title: Calendar
  close: Close
  Loading data...: Loading data...
  error:
    header: Got an error trying to display calendar
    reason: Reason for this error is
    call_to_action: If this issue persists, please contact us via the feedback form at the bottom of the page
  footer:
    disclaimer: Updated hourly and identical events are merged.
    please_check: If in doubt, please check the official calendar on TUMonline
    last_sync: Updated {0}
</i18n>
