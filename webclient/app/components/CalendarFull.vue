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
defineExpose({ refetchEvents });
const runtimeConfig = useRuntimeConfig();
const { locale } = useI18n({ useScope: "local" });

const earliest_last_sync = defineModel<string | null>("earliest_last_sync");
const locations = defineModel<Map<string, CalendarLocation>>("locations");

interface Color {
  backgroundColor: string;
  borderColor: string;
  textColor: string;
}

function colorForType(entry_type: "lecture" | "exercise" | "exam" | "barred" | "other"): Color | Record<string, never> {
  switch (entry_type) {
    case "lecture":
      return { backgroundColor: "#93bae6", borderColor: "#3070b3", textColor: "#13243e" };
    case "exam":
      return { backgroundColor: "#e6bbe2", borderColor: "#c56fb9", textColor: "#3f1837" };
    case "exercise":
      return { backgroundColor: "#fdba74", borderColor: "#f97316", textColor: "#431407" };
    case "other":
      return { backgroundColor: "#d4d4d8", borderColor: "#71717a", textColor: "#09090b" };
    case "barred":
      return { backgroundColor: "#fca5a5", borderColor: "#ef4444", textColor: "#450a0a" };
    default:
      return {};
  }
}

async function fetchEvents(arg: EventSourceFuncArg): Promise<EventInput[]> {
  const body: CalendarBody = {
    start_after: arg.startStr,
    end_before: arg.endStr,
    ids: props.showing,
  };
  const url = `${runtimeConfig.public.apiURL}/api/calendar`;
  const data = await $fetch<CalendarResponse>(url, {
    method: "POST",
    body: body,
    retry: 120,
    retryDelay: 5000,
    credentials: "omit",
    headers: { "Content-Type": "application/json" },
  });
  extractInfos(data);

  const items = [];
  const show_room_names = Object.keys(data).length > 1;
  for (const [k, v] of Object.entries(data)) {
    items.push(
      ...v.events.map((e) => {
        const title = locale.value == "de" ? e.stp_title_de : e.stp_title_en;
        const color = colorForType(e.entry_type);
        return {
          id: e.id.toString(),
          title: show_room_names ? `${k} ${title}` : title,
          start: new Date(e.start_at),
          end: new Date(e.end_at),
          ...color,
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
    .toLocaleString(locale.value, {
      timeStyle: "short",
      dateStyle: "short",
    });
  const tempLocationMap = new Map<string, CalendarLocation>();
  for (const [key, v] of Object.entries(data)) {
    tempLocationMap.set(key, v.location);
  }
  locations.value = tempLocationMap;
}

const calendarOptions: CalendarOptions = {
  plugins: [timeGridPlugin, dayGridPlugin, listPlugin],
  initialView: "timeGridWeek",
  weekends: true,
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

const fullCalendarRef = ref<InstanceType<typeof FullCalendar> | null>(null);

function refetchEvents() {
  const api = fullCalendarRef.value?.getApi();
  if (api) {
    console.debug("Re-Fetching events");
    api.refetchEvents();
  } else {
    nextTick(refetchEvents);
  }
}
</script>

<template>
  <div class="flex max-h-[700px] min-h-[700px] grow flex-col">
    <FullCalendar :options="calendarOptions">
      <template #eventContent="arg">
        <NuxtLink
          :to="`https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/!wbTermin.wbEdit?pTerminNr=${arg.event.id}`"
          external
          class="flex gap-1 overflow-x-auto overflow-y-auto"
          :class="arg.view.type == 'timeGridWeek' ? 'flex-col' : 'flex-row'"
        >
          <template v-if="arg.view.type == 'timeGridWeek'">
            <span class="font-medium">{{ arg.event.title }}</span>
            <span class="font-normal opacity-70"
              >{{ arg.timeText }} - {{ arg.event.end.toLocaleTimeString(locale, { timeStyle: "short" }) }}</span
            >
          </template>
          <template v-else>
            <span class="font-normal">{{ arg.timeText }} ({{ arg.event.start - arg.event.end }})</span>
            <span class="font-medium">{{ arg.event.title }}</span>
          </template>
        </NuxtLink>
      </template>
    </FullCalendar>
  </div>
</template>

<style lang="postcss">
.fc-event-main {
  @apply flex;
}
</style>
