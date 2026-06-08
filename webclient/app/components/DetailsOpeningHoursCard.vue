<script setup lang="ts">
import { mdiChevronDown, mdiOpenInNew } from "@mdi/js";
import { useToggle } from "@vueuse/core";
import type { components } from "~/api_types";
import {
  type OpeningHoursDay,
  type OpeningHoursRange,
  parseOpeningHoursWeek,
} from "~/utils/openingHours";

type OpeningHoursResponse = components["schemas"]["OpeningHoursResponse"];

const props = defineProps<{
  readonly openingHours: OpeningHoursResponse;
}>();

const { t, locale } = useI18n({ useScope: "local" });

// `undefined` while parsing, `null` once it has failed.
const week = ref<readonly OpeningHoursDay[] | null | undefined>(undefined);
const parseFailed = computed(() => week.value === null);

async function refreshWeek(): Promise<void> {
  week.value = await parseOpeningHoursWeek(props.openingHours.osm, new Date());
}

onMounted(refreshWeek);
watch(() => props.openingHours.osm, refreshWeek);

const today = computed<OpeningHoursDay | null>(
  () => week.value?.find((day) => day.isToday) ?? null
);

const [expanded, toggleExpanded] = useToggle(false);

function formatRanges(ranges: readonly OpeningHoursRange[]): string {
  return ranges.map((range) => `${range.from}–${range.to}`).join(", ");
}

const lastUpdated = computed(() => formatIsoDate(props.openingHours.last_update));
const validUntil = computed(() =>
  props.openingHours.valid_until ? formatIsoDate(props.openingHours.valid_until) : null
);

// Parse the parts explicitly; `new Date("YYYY-MM-DD")` is UTC and shifts the day in negative
// offsets.
function formatIsoDate(iso: string): string {
  const [year, month, day] = iso.split("-").map(Number);
  if (!year || !month || !day) return iso;
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString(locale.value === "de" ? "de-DE" : "en-GB", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}
</script>

<template>
  <section v-if="week || parseFailed" class="flex flex-col gap-3 print:!hidden">
    <div class="flex flex-row items-baseline justify-between gap-2">
      <p class="text-zinc-800 dark:text-zinc-100 text-lg font-semibold">{{ t("title") }}</p>
      <Btn :to="openingHours.source_url" variant="link" size="text-xs gap-1 rounded">
        {{ t("source") }}
        <MdiIcon :path="mdiOpenInNew" :size="14" class="my-auto" aria-hidden="true" />
      </Btn>
    </div>

    <p
      v-if="parseFailed"
      class="bg-zinc-100 dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700 text-zinc-600 dark:text-zinc-300 rounded-sm border p-3 font-mono text-sm break-words"
    >
      {{ openingHours.osm }}
    </p>
    <div
      v-else
      class="bg-zinc-100 dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700 rounded-sm border"
    >
      <button
        type="button"
        class="focusable flex w-full items-center gap-3 p-3 text-left"
        :aria-expanded="expanded"
        @click="toggleExpanded()"
      >
        <span class="text-zinc-800 dark:text-zinc-100 font-medium">{{ t("today") }}</span>
        <span v-if="today?.ranges.length" class="text-zinc-600 dark:text-zinc-300 min-w-0 flex-1 truncate">
          {{ formatRanges(today.ranges) }}
        </span>
        <span v-else class="text-zinc-500 dark:text-zinc-400 min-w-0 flex-1">{{ t("closed") }}</span>
        <MdiIcon
          :path="mdiChevronDown"
          :size="18"
          class="text-zinc-500 dark:text-zinc-400 shrink-0 transition-transform"
          :class="{ 'rotate-180': expanded }"
          aria-hidden="true"
        />
      </button>
      <div
        v-if="expanded"
        class="border-zinc-200 dark:border-zinc-700 grid grid-cols-[auto_minmax(0,1fr)] gap-x-6 gap-y-1.5 border-t p-3 text-sm"
      >
        <template v-for="day in week" :key="day.key">
          <span
            :class="
              day.isToday
                ? 'text-zinc-800 dark:text-zinc-100 font-medium'
                : 'text-zinc-500 dark:text-zinc-400'
            "
          >
            {{ t(`day.${day.key}`) }}
          </span>
          <span class="text-zinc-600 dark:text-zinc-300 text-end" :class="{ 'font-medium': day.isToday }">
            <template v-if="day.ranges.length">
              <span v-for="(range, index) in day.ranges" :key="index" class="block" :title="range.comment">
                {{ range.from }}&#8211;{{ range.to }}
              </span>
            </template>
            <span v-else class="text-zinc-500 dark:text-zinc-400">{{ t("closed") }}</span>
          </span>
        </template>
      </div>
    </div>

    <small class="text-zinc-500 dark:text-zinc-400">
      <span v-if="openingHours.service">{{ openingHours.service }} · </span>
      <span v-if="validUntil">{{ t("valid_until", { date: validUntil }) }} · </span>
      {{ t("last_updated", { date: lastUpdated }) }}
    </small>
  </section>
</template>

<i18n lang="yaml">
de:
  title: Öffnungszeiten
  source: Quelle
  today: Heute
  closed: geschlossen
  last_updated: "zuletzt aktualisiert am {date}"
  valid_until: "gültig bis {date}"
  day:
    mo: Montag
    tu: Dienstag
    we: Mittwoch
    th: Donnerstag
    fr: Freitag
    sa: Samstag
    su: Sonntag
en:
  title: Opening hours
  source: Source
  today: Today
  closed: closed
  last_updated: "last updated on {date}"
  valid_until: "valid until {date}"
  day:
    mo: Monday
    tu: Tuesday
    we: Wednesday
    th: Thursday
    fr: Friday
    sa: Saturday
    su: Sunday
</i18n>
