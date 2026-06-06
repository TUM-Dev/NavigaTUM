<script setup lang="ts">
import { mdiBookOpenPageVariantOutline, mdiOpenInNew } from "@mdi/js";
import { useIrisAvailability } from "~/composables/irisAvailability";
import {
  bookedUntilTime,
  IRIS_SITE_URL,
  type IrisRoomRow,
  isKnownStatus,
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

function statusLabel(status: string): string {
  // German status words are passed through; the four known statuses are localized.
  return isKnownStatus(status) ? t(`status.${status}`) : status;
}

const STATUS_BADGE: Record<string, string> = {
  frei: "text-green-900 dark:text-green-50 bg-green-100 dark:bg-green-800",
  belegt: "text-red-900 dark:text-red-50 bg-red-100 dark:bg-red-800",
  WAAS: "text-sky-900 dark:text-sky-50 bg-sky-100 dark:bg-sky-800",
};
const STATUS_BADGE_FALLBACK = "text-zinc-700 dark:text-zinc-200 bg-zinc-200 dark:bg-zinc-600";

function badgeClass(status: string): string {
  return STATUS_BADGE[status] ?? STATUS_BADGE_FALLBACK;
}

// The secondary line under a room: booker/until for booked rooms, occupancy for sensor rooms.
function detailLine(room: IrisRoomRow): string {
  if (room.status === "belegt") {
    const until = room.bookedUntil ? bookedUntilTime(room.bookedUntil) : null;
    if (room.bookedBy && until) return t("booked_by_until", { by: room.bookedBy, time: until });
    if (until) return t("until", { time: until });
    if (room.bookedBy) return t("booked_by", { by: room.bookedBy });
  }
  if (room.occupancy) return t("occupancy", { percent: occupancyPercent(room.occupancy) });
  return "";
}
</script>

<template>
  <section v-if="visible" ref="cardEl" class="flex flex-col gap-3 print:!hidden">
    <div class="flex flex-row items-baseline justify-between gap-2">
      <p class="text-zinc-800 dark:text-zinc-100 text-lg font-semibold flex flex-row items-center gap-2">
        <MdiIcon :path="mdiBookOpenPageVariantOutline" :size="20" class="text-blue-600 dark:text-blue-300" aria-hidden="true" />
        {{ t("title") }}
      </p>
      <Btn :to="IRIS_SITE_URL" variant="link" size="text-xs gap-1 rounded font-semibold">
        {{ t("source") }}
        <MdiIcon :path="mdiOpenInNew" :size="14" class="my-auto" aria-hidden="true" />
      </Btn>
    </div>

    <div
      v-if="loading && !rooms.length"
      class="text-zinc-500 dark:text-zinc-400 text-sm"
    >
      {{ t("loading") }}
    </div>
    <ul
      v-else
      class="bg-zinc-100 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 flex flex-col gap-1 rounded-sm border p-2"
    >
      <li v-for="room in rooms" :key="room.archName">
        <NuxtLinkLocale
          :to="room.path"
          class="group text-zinc-700 dark:text-zinc-200 hover:bg-blue-500 dark:hover:bg-blue-400 hover:text-white dark:hover:text-black flex flex-col gap-0.5 rounded-sm px-2 py-1.5"
        >
          <span class="flex flex-row items-center justify-between gap-2">
            <span class="truncate font-medium">{{ room.name }}</span>
            <span class="flex shrink-0 flex-row items-center gap-1.5">
              <span
                v-if="room.occupancy"
                class="inline-block h-2.5 w-2.5 rounded-full ring-1 ring-black/10 dark:ring-white/20"
                :style="{ backgroundColor: room.occupancy.color }"
                aria-hidden="true"
              />
              <span class="rounded-sm px-1.5 py-0.5 text-xs font-semibold" :class="badgeClass(room.status)">
                {{ statusLabel(room.status) }}
              </span>
            </span>
          </span>
          <span
            v-if="detailLine(room)"
            class="text-zinc-500 dark:text-zinc-300 group-hover:text-white dark:group-hover:text-black truncate text-xs"
          >
            {{ detailLine(room) }}
          </span>
        </NuxtLinkLocale>
      </li>
    </ul>
  </section>
</template>

<i18n lang="yaml">
de:
  title: Lernräume
  source: via AStA Iris
  loading: Live-Verfügbarkeit wird geladen…
  occupancy: "{percent}% belegt"
  until: bis {time}
  booked_by: "{by}"
  booked_by_until: "{by} · bis {time}"
  status:
    frei: frei
    belegt: belegt
    unbekannt: unbekannt
    WAAS: sensorbasiert
en:
  title: Learning rooms
  source: via AStA Iris
  loading: Loading live availability…
  occupancy: "{percent}% full"
  until: until {time}
  booked_by: "{by}"
  booked_by_until: "{by} · until {time}"
  status:
    frei: free
    belegt: booked
    unbekannt: unknown
    WAAS: sensor-based
</i18n>
