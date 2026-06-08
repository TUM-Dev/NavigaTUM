<script setup lang="ts">
import { mdiChevronDown, mdiClockCheckOutline, mdiClockRemoveOutline, mdiOpenInNew } from "@mdi/js";
import { useToggle } from "@vueuse/core";
import type { components } from "~/api_types";
import {
  computeOpeningHoursState,
  type OpeningHoursDay,
  type OpeningHoursLiveState,
  type OpeningHoursRange,
  parseOpeningHoursWeek,
  WEEKDAY_KEYS,
} from "~/utils/openingHours";

type OpeningHoursResponse = components["schemas"]["OpeningHoursResponse"];

const props = defineProps<{
  readonly openingHours: OpeningHoursResponse;
}>();

const { t, locale } = useI18n({ useScope: "local" });

// `undefined` while parsing, `null` once it has failed.
const week = ref<readonly OpeningHoursDay[] | null | undefined>(undefined);
const parseFailed = computed(() => week.value === null);
const liveState = ref<OpeningHoursLiveState | null | undefined>(undefined);
// The instant the week and live state were evaluated at; the live hint is relative to it.
const evaluatedAt = ref<Date | null>(null);

async function refresh(): Promise<void> {
  const now = new Date();
  const [parsedWeek, state] = await Promise.all([
    parseOpeningHoursWeek(props.openingHours.osm, now),
    computeOpeningHoursState(props.openingHours.osm, now),
  ]);
  week.value = parsedWeek;
  liveState.value = state;
  evaluatedAt.value = now;
}

onMounted(refresh);
watch(() => props.openingHours.osm, refresh);

const today = computed<OpeningHoursDay | null>(
  () => week.value?.find((day) => day.isToday) ?? null
);

// Below this many minutes to closing, surface a "closes in X minutes" hint instead of the time.
const CLOSES_SOON_MINUTES = 60;

// The live indicator's `open` flag and localized detail line, or `null` while loading or unparsable.
const liveStatus = computed<{ open: boolean; detail: string | null } | null>(() => {
  const state = liveState.value;
  const now = evaluatedAt.value;
  if (!state || !now) return null;
  if (state.open) {
    if (!state.nextChange) return { open: true, detail: null };
    const minutes = Math.ceil((state.nextChange.getTime() - now.getTime()) / 60_000);
    if (minutes <= CLOSES_SOON_MINUTES) return { open: true, detail: t("closes_in", minutes) };
    return { open: true, detail: t("closes_at", { time: formatTime(state.nextChange) }) };
  }
  if (!state.nextChange) return { open: false, detail: null };
  if (isSameDay(state.nextChange, now))
    return { open: false, detail: t("opens_at", { time: formatTime(state.nextChange) }) };
  const day = t(`day.${WEEKDAY_KEYS[(state.nextChange.getDay() + 6) % 7]}`);
  return { open: false, detail: t("opens_on", { day, time: formatTime(state.nextChange) }) };
});

const [expanded, toggleExpanded] = useToggle(false);

function formatRanges(ranges: readonly OpeningHoursRange[]): string {
  return ranges.map((range) => `${range.from}-${range.to}`).join(", ");
}

function formatTime(date: Date): string {
  return `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
}

function isSameDay(a: Date, b: Date): boolean {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
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

    <p v-if="liveStatus" class="flex items-center gap-1.5 text-sm">
      <MdiIcon
        :path="liveStatus.open ? mdiClockCheckOutline : mdiClockRemoveOutline"
        :size="18"
        class="shrink-0"
        :class="liveStatus.open ? 'text-green-700 dark:text-green-300' : 'text-red-700 dark:text-red-300'"
        aria-hidden="true"
      />
      <span class="text-zinc-800 dark:text-zinc-100 font-medium">
        {{ liveStatus.open ? t("open_now") : t("closed_now") }}
      </span>
      <span v-if="liveStatus.detail" class="text-zinc-500 dark:text-zinc-400">{{ liveStatus.detail }}</span>
    </p>

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
  open_now: Geöffnet
  closed_now: Geschlossen
  closes_in: "schließt in einer Minute | schließt in {count} Minuten"
  closes_at: "schließt um {time}"
  opens_at: "öffnet um {time}"
  opens_on: "öffnet {day} um {time}"
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
  open_now: Open
  closed_now: Closed
  closes_in: "closes in one minute | closes in {count} minutes"
  closes_at: "closes at {time}"
  opens_at: "opens at {time}"
  opens_on: "opens {day} at {time}"
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
