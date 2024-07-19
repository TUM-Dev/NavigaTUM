<script setup lang="ts">
import FullCalendar from "@fullcalendar/vue3";
import type { CalendarOptions, EventInput, EventSourceFuncArg } from "@fullcalendar/core";
import listPlugin from "@fullcalendar/list";
import timeGridPlugin from "@fullcalendar/timegrid";
import dayGridPlugin from "@fullcalendar/daygrid";
import type { components, operations } from "~/api_types";
import deLocale from "@fullcalendar/core/locales/de";
import enLocale from "@fullcalendar/core/locales/en-gb";

type CalendarResponse = components["schemas"]["CalendarResponse"];
type CalendarBody = operations["calendar"]["requestBody"]["content"]["application/json"];
type CalendarLocation = components["schemas"]["CalendarLocation"];

const props = defineProps<{ showing: readonly string[] }>();
const runtimeConfig = useRuntimeConfig();
const { locale } = useI18n({ useScope: "local" });

const earliest_last_sync = defineModel<string | null>("earliest_last_sync");
const locations = defineModel<Map<string, CalendarLocation>>("locations");

async function fetchEvents(arg: EventSourceFuncArg): Promise<EventInput[]> {
  const body: CalendarBody = {
    start_after: arg.startStr,
    end_before: arg.endStr,
    ids: props.showing,
  };
  const url = `${runtimeConfig.public.apiURL}/api/calendar`;
  const data = await $fetch<CalendarResponse>(url, {
    method: "POST",
    body: JSON.stringify(body),
    retry: 120,
    retryDelay: 5000,
  });
  extractInfos(data);

  const items = [];
  const show_room_names = Object.keys(data).length > 1;
  for (const [k, v] of Object.entries(data)) {
    items.push(
      ...v.events.map((e) => {
        const title = locale.value == "de" ? e.stp_title_de : e.stp_title_en;
        return {
          id: e.id.toString(),
          title: show_room_names ? `${k} ${title}` : title,
          start: new Date(e.start_at),
          end: new Date(e.end_at),
          classes: [e.entry_type],
        };
      }),
    );
  }
  return items;
}

function extractInfos(data: CalendarResponse): void {
  earliest_last_sync.value = Object.values(data)
    .map((d) => new Date(d.location.last_calendar_scrape_at))
    .reduce((d1, d2) => (d1 < d2 ? d1 : d2))
    .toLocaleString(locale.value, { timeStyle: "short", dateStyle: "short" });
  const tempLocationMap = new Map<string, CalendarLocation>();
  for (const [key, v] of Object.entries(data)) {
    tempLocationMap.set(key, v.location);
  }
  locations.value = tempLocationMap;
}

const calendarOptions: CalendarOptions = {
  plugins: [timeGridPlugin, dayGridPlugin, listPlugin],
  initialView: "timeGridWeek",
  weekends: false,
  events: fetchEvents,
  headerToolbar: {
    left: "prev,next",
    center: "title",
    right: "dayGridMonth,timeGridWeek,timeGridDay,list",
  },
  locale: locale.value == "de" ? deLocale : enLocale,
  height: 700,
  // like '14:30'
  displayEventEnd: false,
  eventTimeFormat: {
    hour: "2-digit",
    minute: "2-digit",
    meridiem: false,
  },
};
</script>

<template>
  <div class="flex max-h-[700px] min-h-[700px] grow flex-col">
    <FullCalendar :options="calendarOptions">
      <template #eventContent="arg">
        <NuxtLink
          :to="`https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/!wbTermin.wbEdit?pTerminNr=${arg.event.id}`"
          external
          class="flex overflow-x-auto gap-1 overflow-y-auto"
        >
          <span class="font-normal not-italic">{{ arg.timeText }}</span>
          <span class="font-medium not-italic">{{ arg.event.title }}</span>
        </NuxtLink>
      </template>
    </FullCalendar>
  </div>
</template>
