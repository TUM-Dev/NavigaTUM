<script setup lang="ts">
import {
  mdiAccountGroup,
  mdiCheck,
  mdiChevronDown,
  mdiChevronRight,
  mdiChevronUp,
  mdiHelpCircle,
  mdiLock,
  mdiOpenInNew,
} from "@mdi/js";
import { useToggle } from "@vueuse/core";
import { useIrisAvailability } from "~/composables/irisAvailability";
import {
  bookedUntilTime,
  IRIS_SITE_URL,
  type IrisRoomRow,
  occupancyPercent,
} from "~/utils/iris";

const props = defineProps<{
  // The NavigaTUM building id whose Iris learning rooms to show. Only passed for entries whose
  // build-time `has_iris_coverage` signal is true, so the parent gates the browser-side fetch.
  readonly buildingId: string;
}>();

const { t } = useI18n({ useScope: "local" });

const cardEl = ref<HTMLElement | null>(null);
const { rooms, loading } = useIrisAvailability(() => props.buildingId, cardEl);

// Hide once the first snapshot settled with nothing to show (a failed first load, or no room that
// resolved to a NavigaTUM page) - that is the silent-degradation path. The card stays mounted while
// loading so its element can be observed and the fetch can start when it scrolls into view.
const visible = computed(() => loading.value || rooms.value.length > 0);

// The default view shows rooms a student could plausibly grab now: fully free (`frei`) and
// sensor-based (`WAAS`) rooms whose avatar colour already encodes how full they are. Booked
// (`belegt`) and unknown-status rooms hide behind the expand toggle so the sidebar stays scannable.
function isPartiallyFree(room: IrisRoomRow): boolean {
  return room.status === "frei" || room.status === "WAAS";
}

const partiallyFreeRooms = computed(() => rooms.value.filter(isPartiallyFree));
const otherRooms = computed(() => rooms.value.filter((room) => !isPartiallyFree(room)));
const [expanded, toggleExpanded] = useToggle(false);
// If every room is booked/unknown, defaulting to the partial-free filter would render an empty
// list. In that case there is no useful "show all" to expand to, so just show every room.
const canCollapse = computed(
  () => partiallyFreeRooms.value.length > 0 && otherRooms.value.length > 0
);
const visibleRooms = computed<readonly IrisRoomRow[]>(() => {
  if (!canCollapse.value) return rooms.value;
  // Keep the partially-free rooms on top once expanded so the useful ones stay above the fold.
  return expanded.value ? [...partiallyFreeRooms.value, ...otherRooms.value] : partiallyFreeRooms.value;
});

function avatarIcon(status: string): string {
  switch (status) {
    case "frei":
      return mdiCheck;
    case "belegt":
      return mdiLock;
    case "WAAS":
      return mdiAccountGroup;
    default:
      return mdiHelpCircle;
  }
}

// Fixed semantic colours per status. WAAS rooms override this from the Iris-computed occupancy band
// via `avatarStyle`, so the colour itself reads as how full the room is.
function avatarClass(status: string): string {
  switch (status) {
    case "frei":
      return "bg-green-500 dark:bg-green-400";
    case "belegt":
      return "bg-red-500 dark:bg-red-400";
    case "WAAS":
      return "bg-sky-500 dark:bg-sky-400";
    default:
      return "bg-zinc-400 dark:bg-zinc-500";
  }
}

function avatarStyle(room: IrisRoomRow): Record<string, string> | undefined {
  if (room.status === "WAAS" && room.occupancy) {
    return { backgroundColor: room.occupancy.color };
  }
  return undefined;
}

// One short context line under the room name. The avatar colour carries the headline, so this
// only needs to add the detail (a time, a percentage, or - for unknown values - the raw status).
function detailLine(room: IrisRoomRow): string {
  switch (room.status) {
    case "frei":
      return t("status.frei");
    case "belegt": {
      const until = room.bookedUntil ? bookedUntilTime(room.bookedUntil) : null;
      if (until && room.bookedBy) return t("until_by", { time: until, by: room.bookedBy });
      if (until) return t("until", { time: until });
      if (room.bookedBy) return room.bookedBy;
      return t("status.belegt");
    }
    case "WAAS":
      return room.occupancy
        ? t("occupancy", { percent: occupancyPercent(room.occupancy) })
        : t("status.WAAS");
    case "unbekannt":
      return t("status.unbekannt");
    default:
      // Iris may add new status codes; pass through verbatim rather than mislabel them.
      return room.status;
  }
}

function statusAriaLabel(status: string): string {
  switch (status) {
    case "frei":
    case "belegt":
    case "WAAS":
    case "unbekannt":
      return t(`status.${status}`);
    default:
      return status;
  }
}
</script>

<template>
  <section v-if="visible" ref="cardEl" class="flex flex-col gap-3 print:!hidden">
    <div class="flex flex-row items-baseline justify-between gap-2">
      <p class="text-zinc-800 dark:text-zinc-100 text-lg font-semibold">{{ t("title") }}</p>
      <Btn :to="IRIS_SITE_URL" variant="link" size="text-xs gap-1 rounded">
        <span class="sm:hidden">{{ t("source_short") }}</span>
        <span class="hidden sm:inline">{{ t("source") }}</span>
        <MdiIcon :path="mdiOpenInNew" :size="14" class="my-auto" aria-hidden="true" />
      </Btn>
    </div>

    <div
      v-if="loading && !rooms.length"
      class="text-zinc-500 dark:text-zinc-400 text-sm"
    >
      {{ t("loading") }}
    </div>
    <div v-else class="text-zinc-600 dark:text-zinc-300 grid grid-cols-1 sm:grid-cols-2 gap-3">
      <NuxtLinkLocale
        v-for="room in visibleRooms"
        :key="room.archName"
        :to="room.path"
        class="focusable border-zinc-200 dark:border-zinc-700 flex flex-row items-center justify-between gap-3 rounded-sm border border-solid p-3.5 !no-underline hover:bg-zinc-100 dark:hover:bg-zinc-800"
      >
        <div class="flex flex-row items-center gap-3 min-w-0">
          <div
            class="text-white dark:text-black min-w-11 h-11 w-11 rounded-full p-2 flex items-center justify-center shrink-0"
            :class="avatarClass(room.status)"
            :style="avatarStyle(room)"
            :aria-label="statusAriaLabel(room.status)"
          >
            <MdiIcon :path="avatarIcon(room.status)" :size="24" aria-hidden="true" />
          </div>
          <div class="flex flex-col justify-evenly min-w-0">
            <div class="line-clamp-2 text-balance text-zinc-800 dark:text-zinc-100">{{ room.name }}</div>
            <small class="text-zinc-600 dark:text-zinc-300 truncate">{{ detailLine(room) }}</small>
          </div>
        </div>
        <MdiIcon :path="mdiChevronRight" :size="16" class="shrink-0" aria-hidden="true" />
      </NuxtLinkLocale>
    </div>
    <Btn
      v-if="canCollapse"
      variant="linkButton"
      :aria-label="expanded ? t('show_less_aria') : t('show_more_aria', otherRooms.length)"
      @click="toggleExpanded()"
    >
      <template v-if="expanded">
        <MdiIcon :path="mdiChevronUp" :size="16" class="mt-0.5" />
        {{ t("show_less") }}
      </template>
      <template v-else>
        <MdiIcon :path="mdiChevronDown" :size="16" class="mt-0.5" />
        {{ t("show_more", otherRooms.length) }}
      </template>
    </Btn>
  </section>
</template>

<i18n lang="yaml">
de:
  title: Lernräume
  source: via Studentische Vertretung IRIS
  source_short: via SV IRIS
  loading: Live-Verfügbarkeit wird geladen…
  occupancy: "{percent}% belegt"
  until: bis {time}
  until_by: "bis {time} · {by}"
  show_more: "alle anzeigen (1 weiteren) | alle anzeigen ({count} weitere)"
  show_less: weniger anzeigen
  show_more_aria: "alle Räume anzeigen, einschließlich 1 weiteren belegten oder unbekannten Raumes | alle Räume anzeigen, einschließlich {count} weiterer belegter oder unbekannter Räume"
  show_less_aria: nur (teilweise) freie Räume anzeigen
  status:
    frei: frei
    belegt: belegt
    unbekannt: Status unbekannt
    WAAS: sensorbasiert
en:
  title: Learning rooms
  source: via Studentische Vertretung IRIS
  source_short: via SV IRIS
  loading: Loading live availability…
  occupancy: "{percent}% in use"
  until: until {time}
  until_by: "until {time} · {by}"
  show_more: "show all (1 more) | show all ({count} more)"
  show_less: show less
  show_more_aria: "show all rooms, including 1 more booked or unknown room | show all rooms, including {count} more booked or unknown rooms"
  show_less_aria: show only (partially) free rooms
  status:
    frei: free
    belegt: booked
    unbekannt: status unknown
    WAAS: sensor-based
</i18n>
