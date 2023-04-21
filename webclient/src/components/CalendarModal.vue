<script setup lang="ts">
import { CalendarView, CalendarViewHeader } from "vue-simple-calendar";
import { ref, computed, watch } from "vue";
import type { ICalendarItem } from "vue-simple-calendar/dist/src/ICalendarItem";

import { useGlobalStore } from "@/stores/global";
import { useRoute } from "vue-router";
import type { components } from "@/api_types";
type CalendarResponse = components["schemas"]["CalendarResponse"];
import "/node_modules/vue-simple-calendar/dist/style.css";
import "/node_modules/vue-simple-calendar/dist/css/gcal.css";
import { useFetch } from "@/composables/fetch";

const global = useGlobalStore();
const showDate = ref(new Date());

const tumonlineCalendarUrl = ref("https://campus.tum.de/tumonline");
const last_sync = ref("xx.xx.xxxx xx:xx");
const events = ref<ICalendarItem[]>([]);

const route = useRoute();

const start = computed(() => {
  const start = new Date(showDate.value);
  start.setDate(start.getDate() - 60);
  return start.toISOString().replace("Z", "");
});

const end = computed(() => {
  const start = new Date(showDate.value);
  start.setDate(start.getDate() + 60);
  return start.toISOString().replace("Z", "");
});
// called when the view is loaded
update();
// called when the view navigates to another view, but not when its initially loaded
watch(() => showDate.value, update);
watch(() => route.params.id, update);

function update() {
  useFetch<CalendarResponse>(
    `https://nav.tum.de/api/calendar/${route.params.id}?start=${start.value}&end=${end.value}`,
    (d) => {
      tumonlineCalendarUrl.value = d.calendar_url;
      last_sync.value = new Date(d.last_sync).toLocaleString("de-DE", { timeStyle: "short", dateStyle: "short" });
      events.value = d.events.map((e) => ({
        id: e.id.toString(),
        title: e.title,
        startDate: new Date(e.start),
        endDate: new Date(e.end),
        classes: [e.entry_type],
      }));
    }
  );
}
function setShowDate(d: Date) {
  showDate.value = d;
}
</script>
<template>
  <div class="modal modal-lg active" id="calendar-modal">
    <a @click="global.calendar.open = false" class="modal-overlay" aria-label="Close"></a>
    <div class="modal-container">
      <div class="modal-header">
        <a
          @click="global.calendar.open = false"
          class="btn btn-clear float-right"
          v-bind:aria-label="$t('calendar.modal.close')"
        ></a>
        <div class="modal-title h5">{{ $t("calendar.modal.title") }}</div>
      </div>
      <div class="modal-body">
        <div class="modal-body">
          <CalendarView
            id="calendar-view"
            :items="events"
            :show-date="showDate"
            :showTimes="true"
            :timeFormatOptions="{ hour: '2-digit', minute: '2-digit', hour12: false }"
            :startingDayOfWeek="1"
            :itemTop="'2.5em'"
            class="theme-gcal"
          >
            <template #header="{ headerProps }">
              <CalendarViewHeader :header-props="headerProps" @input="setShowDate" />
            </template>
          </CalendarView>
        </div>
      </div>
      <div class="modal-footer">
        {{ $t("calendar.modal.footer.disclaimer") }} <br />
        {{ $t("calendar.modal.footer.please_check") }}
        <a v-bind:href="tumonlineCalendarUrl">{{ $t("calendar.modal.footer.official_calendar") }}</a
        >. {{ $t("calendar.modal.footer.last_sync") }}: {{ last_sync }}
      </div>
    </div>
  </div>
</template>
<style lang="scss">
@import "../assets/variables";

#calendar-modal {
  .modal-container {
    position: relative;
    height: auto;
    max-width: 97.5vw;
    max-height: 90vh;
    .modal-body {
      padding: 0;
      height: 40rem;
      #calendar-view {
        display: flex;
        flex-direction: column;
        flex-grow: 1;
      }
    }
  }
  .startTime,
  .endTime {
    color: white;
    font-size: 0.5rem;
  }

  .cv-day.today .cv-day-number {
    margin-top: 0.1em !important;
    background-color: $primary-color !important;
    color: $light-color !important;
  }

  .cv-day-number,
  .periodLabel {
    color: $body-font-color !important;
  }
  .currentPeriod {
    color: $body-font-color !important;
    background-color: transparent !important;
  }
  .cv-header-day {
    color: $body-font-color !important;
  }

  .cv-item {
    padding-bottom: 1rem !important;
  }
  .cv-item {
    padding-bottom: 0.1em !important;
    padding-top: 0.1em !important;
  }
  .past.cv-item {
    filter: brightness(1.3) grayscale(0.55);
  }

  // colors
  .barred {
    background-color: $error-color;
  }
  .lecture {
    background-color: $secondary-color;
  }
  .exercise {
    background-color: #d99208;
  }
  .exam {
    background-color: #b55ca5;
  }
  .other {
    background-color: var(--event-color-graphite);
  }
}
</style>
