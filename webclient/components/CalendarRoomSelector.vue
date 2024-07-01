<script setup lang="ts">
import { PlusCircleIcon } from "@heroicons/vue/24/outline";
import { CalendarIcon } from "@heroicons/vue/16/solid";
import type { components } from "~/api_types";
import PreviewIcon from "~/components/PreviewIcon.vue";

type CalendarLocation = components["schemas"]["CalendarLocation"];
defineProps<{
  readonly data: Map<string, CalendarLocation>;
}>();
const calendar = useCalendar();
const { t } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();

type DetailsResponse = components["schemas"]["DetailsResponse"];
async function addLocation() {
  let location: string = "";
  while (!location) {
    location = window.prompt(t("prompt.initial")) || "";
    try {
      const result = await fetch(`${runtimeConfig.public.apiURL}/api/get/${location}`);
      if (!result.ok) {
        const userWantsToRetry = window.confirm(t("prompt.error_not_ok", [location]));
        if (!userWantsToRetry) return;
        location = "";
        continue;
      }
      const res = (await result.json()) as DetailsResponse;
      if (!res.props.calendar_url) {
        const userWantsToRetry = window.confirm(t("prompt.error_not_calendar", [location]));
        if (!userWantsToRetry) return;
        location = "";
        continue;
      }
    } catch (e) {
      window.alert("Failed because " + e);
      location = "";
      continue;
    }
    if (calendar.value.showing.find((k) => k == location)) {
      const userWantsToRetry = window.confirm(t("prompt.error_already_exists", [location]));
      if (!userWantsToRetry) return;
      location = "";
    }
  }
  calendar.value.showing.push(location);
}
</script>

<template>
  <ul v-if="calendar.showing.length" class="mb-6 flex gap-2 overflow-x-auto">
    <li v-if="data.size === 0" class="text-zinc-900 flex flex-col items-center gap-5 py-32">
      <Spinner class="h-8 w-8" />
      {{ t("Loading data...") }}
    </li>
    <li
      v-for="[key, location] in data.entries()"
      :key="key"
      class="flex min-w-64 gap-1 rounded-md px-2 md:gap-3 md:px-4"
    >
      <PreviewIcon
        :item="{
          type: location.type,
          parsed_id: 'parsed_id' in location ? (location.parsed_id as string) : undefined,
        }"
      />
      <div class="text-zinc-600 flex flex-col gap-1">
        <NuxtLink class="line-clamp-1 hover:underline" :to="'/view/' + key">{{ location.name }}</NuxtLink>
        <small>
          {{ location.type_common_name }}
        </small>
        <small>
          <Btn :to="location.calendar_url" variant="link" size="text-xs font-semibold rounded-md">
            <CalendarIcon class="mb-0.5 h-4 w-4" /> {{ t("view_in_tumonline") }}
          </Btn>
        </small>
      </div>
    </li>
    <li class="flex">
      <button
        type="button"
        class="focusable group bg-zinc-50 border-zinc-200 flex min-w-14 origin-center place-items-center content-center items-center justify-center justify-items-center justify-self-center rounded-md border object-center align-middle hover:bg-blue-100"
        :title="t('add_location')"
        :aria-label="t('add_location')"
        @click="addLocation"
      >
        <PlusCircleIcon class="group-hover:text-black h-5 w-5" />
      </button>
    </li>
  </ul>
</template>

<i18n lang="yaml">
de:
  add_location: Zusätzliche Location zum Kalendar hinzufügen
  view_in_tumonline: in TUMonline ansehen
  prompt:
    initial: |-
      Welcher Kalender soll hinzugefügt werden?
      Bitte wähle nur den letzten Teil der Url eines Raumes mit einem Kalender.

      Beispiel:
      https://nav.tum.de/room/5502.01.250 hat die ID 5502.01.250
    error_not_ok: |-
      Der Ort '{0}' existiert nicht.
      Wenn er existieren würde, gäbe es https://nav.tum.de/view/{0}.

      Möchtest du es mit einer anderen ID erneut versuchen?
    error_not_calendar: |-
      Der Ort hat keinen Kalender.
      Möchtest du es mit einer anderen ID erneut versuchen?
    error_already_exists: |-
      Der Ort ist bereits im Kalender eingetragen.
      Möchtest du es mit einer anderen ID erneut versuchen?
  Loading data...: Lädt daten...
en:
  add_location: add additional location to the calendar
  view_in_tumonline: View in TUMonline
  prompt:
    initial: |-
      Which calendar should be added?
      Please select just the last part of the url of a room with a calendar.

      Example:
      https://nav.tum.de/room/5502.01.250 has the id 5502.01.250
    error_not_ok: |-
      Location '{0}' does not exist.
      If it were to exist https://nav.tum.de/view/{0} would exist.
      Want to retry with a different id?
    error_not_calendar: |-
      Location does not have a calendar.
      Want to retry with a different id?
    error_already_exists: |-
      Location is already in the calendar.
      Want to retry with a different id?
  Loading data...: Loading data...
</i18n>
