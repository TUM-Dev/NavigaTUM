<script setup lang="ts">
import { mdiAlert, mdiChevronDown, mdiElevator, mdiStairs, mdiSwapVertical } from "@mdi/js";
import type { components } from "~/api_types";
import { formatDistance, formatDuration, legFloorSpan } from "~/utils/motis";

type MotisLegResponse = components["schemas"]["MotisLegResponse"];
type DirectionResponse = components["schemas"]["DirectionResponse"];

const props = defineProps<{
  leg: MotisLegResponse;
  open: boolean;
}>();
const emit = defineEmits<{ toggle: []; selectStep: [stepIndex: number] }>();

const { t } = useI18n({ useScope: "local" });

// Maps each turn to a base-instruction i18n key; vertical moves are narrated separately below.
const TURN_KEY: Record<DirectionResponse, string> = {
  depart: "dir_depart",
  continue: "dir_continue",
  slightly_left: "dir_bear_left",
  left: "dir_turn_left",
  hard_left: "dir_sharp_left",
  slightly_right: "dir_bear_right",
  right: "dir_turn_right",
  hard_right: "dir_sharp_right",
  uturn_left: "dir_uturn",
  uturn_right: "dir_uturn",
  circle_clockwise: "dir_roundabout",
  circle_counterclockwise: "dir_roundabout",
  stairs: "dir_continue",
  elevator: "dir_continue",
};

const steps = computed(() => props.leg.steps ?? []);
const hasRestrictedStep = computed(() => steps.value.some((step) => step.access_restriction));

// The floors this walk touches, so the collapsed leg can advertise vertical movement.
const floorSpan = computed(() => legFloorSpan(props.leg));
const floorBadge = computed<string | null>(() => {
  const span = floorSpan.value;
  if (span.length === 0) return null;
  return `${formatLevel(span[0] ?? 0)} – ${formatLevel(span[span.length - 1] ?? 0)}`;
});
const verticalIcon = computed(() => {
  if (steps.value.some((step) => step.relative_direction === "elevator")) return mdiElevator;
  if (steps.value.some((step) => step.relative_direction === "stairs")) return mdiStairs;
  return mdiSwapVertical;
});

// Motis reports the level per step (and, inconsistently, within a step) rather than as explicit
// transitions, so a floor change shows up as the level a step lands on differing from the one
// before it. We surface the reached level as-is — half-levels and all — without smoothing.
const levelChanges = computed(() =>
  steps.value.map((step, index) => {
    const before = index > 0 ? (steps.value[index - 1]?.to_level ?? 0) : props.leg.from.level;
    if (step.to_level === before) return null;
    return { level: step.to_level, up: step.to_level > before };
  })
);

// Whether a step heads up or down, so stairs and elevators can point the right way. Prefer a
// within-step level change, then the change relative to the previous step.
const verticalDirections = computed<(("up" | "down") | undefined)[]>(() =>
  steps.value.map((step, index) => {
    if (step.to_level !== step.from_level) return step.to_level > step.from_level ? "up" : "down";
    const change = levelChanges.value[index];
    return change ? (change.up ? "up" : "down") : undefined;
  })
);

function formatLevel(level: number): string {
  return String(level);
}

// Motis hands us a maneuver enum and (rarely, indoors) a street name, but no instruction text, so
// we compose one — narrating stairs, elevators, and bare level changes with the floor they reach.
function stepInstruction(index: number): string {
  const step = steps.value[index];
  if (!step) return t("dir_continue");
  const direction = step.relative_direction;
  const change = levelChanges.value[index];
  const vertical = verticalDirections.value[index];

  if (direction === "stairs" || direction === "elevator") {
    const mode = direction === "stairs" ? "stairs" : "elevator";
    if (change && vertical)
      return t(`take_${mode}_${vertical}`, { level: formatLevel(change.level) });
    return t(`take_${mode}`);
  }
  if (change && vertical && (direction === "continue" || direction === "depart")) {
    return t(`go_${vertical}`, { level: formatLevel(change.level) });
  }

  const base = t(TURN_KEY[direction]);
  if (!step.street_name) return base;
  const connector =
    direction === "continue" || direction === "depart" ? "on_street" : "onto_street";
  return t(connector, { maneuver: base, street: step.street_name });
}
</script>

<template>
  <div class="flex items-stretch gap-3">
    <div class="flex w-4 flex-shrink-0 justify-center">
      <span class="my-0.5 w-0 border-l-2 border-dashed border-zinc-300 dark:border-zinc-600" />
    </div>

    <div class="min-w-0 flex-grow py-1">
      <button
        type="button"
        class="focusable flex w-full items-center gap-2 rounded-sm text-left text-sm text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200"
        :aria-expanded="open"
        @click="emit('toggle')"
      >
        <MotisTransitModeIcon :mode="leg.mode" variant="inherit" class="h-4 w-4 flex-shrink-0" />
        <span v-if="leg.distance">{{ formatDistance(leg.distance) }}</span>
        <span
          v-if="floorBadge"
          :title="t('touches_floors')"
          class="text-indigo-700 dark:text-indigo-300 flex items-center gap-0.5 text-xs font-medium"
        >
          <MdiIcon :path="verticalIcon" :size="14" />
          {{ floorBadge }}
        </span>
        <span v-if="hasRestrictedStep" class="text-amber-700 dark:text-amber-300 flex items-center gap-1 text-xs">
          <MdiIcon :path="mdiAlert" :size="14" />
          {{ t("access_restriction_hint") }}
        </span>
        <span class="ml-auto flex-shrink-0">{{ formatDuration(leg.duration) }}</span>
        <MdiIcon
          v-if="steps.length"
          :path="mdiChevronDown"
          :size="14"
          class="flex-shrink-0 transition-transform"
          :class="{ 'rotate-180': open }"
        />
      </button>

      <Collapsible :open="open">
        <ol class="mt-1 space-y-2 pl-1">
          <li
            v-for="(step, s) in steps"
            :key="`step-${s}`"
            class="focusable -mx-2 flex cursor-pointer items-start gap-3 rounded px-2 py-1 hover:bg-zinc-100 dark:hover:bg-zinc-800"
            @click="emit('selectStep', s)"
          >
            <WalkingDirectionIcon :direction="step.relative_direction" :vertical="verticalDirections[s]" />
            <div class="min-w-0 flex-grow text-sm">
              <div class="text-zinc-900 dark:text-zinc-50">
                {{ stepInstruction(s) }}
              </div>
              <div class="text-zinc-600 dark:text-zinc-300 flex flex-wrap items-center gap-2 text-xs">
                {{ formatDistance(step.distance) }}
                <span v-if="step.elevation_up" class="text-zinc-500 dark:text-zinc-400">↗ {{ step.elevation_up }}m</span>
                <span v-if="step.elevation_down" class="text-zinc-500 dark:text-zinc-400">↘ {{ step.elevation_down }}m</span>
                <span v-if="step.toll" class="text-orange-600 dark:text-orange-300">{{ t("toll") }}</span>
              </div>
              <div
                v-if="step.access_restriction"
                class="text-amber-700 dark:text-amber-300 mt-0.5 flex items-start gap-1 text-xs"
              >
                <MdiIcon :path="mdiAlert" :size="14" class="mt-0.5 flex-shrink-0" />
                <span>{{ t("access_restriction") }}: {{ step.access_restriction }}</span>
              </div>
            </div>
          </li>
        </ol>
      </Collapsible>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  touches_floors: berührte Ebenen
  dir_depart: Losgehen
  dir_continue: Weiter
  dir_bear_left: Leicht links halten
  dir_turn_left: Links abbiegen
  dir_sharp_left: Scharf links abbiegen
  dir_bear_right: Leicht rechts halten
  dir_turn_right: Rechts abbiegen
  dir_sharp_right: Scharf rechts abbiegen
  dir_uturn: Wenden
  dir_roundabout: In den Kreisverkehr
  on_street: "{maneuver} auf {street}"
  onto_street: "{maneuver} auf {street}"
  take_stairs: Treppe nehmen
  take_stairs_up: Treppe hoch auf Ebene {level}
  take_stairs_down: Treppe runter auf Ebene {level}
  take_elevator: Aufzug nehmen
  take_elevator_up: Aufzug hoch auf Ebene {level}
  take_elevator_down: Aufzug runter auf Ebene {level}
  go_up: Hoch auf Ebene {level}
  go_down: Runter auf Ebene {level}
  toll: Maut
  access_restriction: Eingeschränkter Zugang
  access_restriction_hint: enthält Zugangsbeschränkungen
en:
  touches_floors: floors touched
  dir_depart: Start walking
  dir_continue: Continue
  dir_bear_left: Bear left
  dir_turn_left: Turn left
  dir_sharp_left: Turn sharply left
  dir_bear_right: Bear right
  dir_turn_right: Turn right
  dir_sharp_right: Turn sharply right
  dir_uturn: Turn around
  dir_roundabout: Take the roundabout
  on_street: "{maneuver} on {street}"
  onto_street: "{maneuver} onto {street}"
  take_stairs: Take the stairs
  take_stairs_up: Take the stairs up to level {level}
  take_stairs_down: Take the stairs down to level {level}
  take_elevator: Take the elevator
  take_elevator_up: Take the elevator up to level {level}
  take_elevator_down: Take the elevator down to level {level}
  go_up: Go up to level {level}
  go_down: Go down to level {level}
  toll: Toll
  access_restriction: Restricted access
  access_restriction_hint: contains access restrictions
</i18n>
