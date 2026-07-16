<script setup lang="ts">
import { mdiAlert, mdiChevronDown } from "@mdi/js";
import type { components } from "~/api_types";
import { formatDistance, formatDuration } from "~/utils/motis";

type MotisLegResponse = components["schemas"]["MotisLegResponse"];

const props = defineProps<{
  leg: MotisLegResponse;
  open: boolean;
}>();
const emit = defineEmits<{ toggle: []; selectStep: [stepIndex: number] }>();

const { t } = useI18n({ useScope: "local" });

const steps = computed(() => props.leg.steps ?? []);
const hasRestrictedStep = computed(() => steps.value.some((step) => step.access_restriction));
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
        <ol class="mt-1 space-y-2 border-l border-zinc-200 dark:border-zinc-700 pl-3">
          <li
            v-for="(step, s) in steps"
            :key="`step-${s}`"
            class="focusable -mx-2 flex cursor-pointer items-start gap-3 rounded px-2 py-1 hover:bg-zinc-100 dark:hover:bg-zinc-800"
            @click="emit('selectStep', s)"
          >
            <WalkingDirectionIcon :direction="step.relative_direction" />
            <div class="min-w-0 flex-grow text-sm">
              <div class="text-zinc-900 dark:text-zinc-50">
                {{ step.street_name || t("continue") }}
              </div>
              <div class="text-zinc-600 dark:text-zinc-300 flex flex-wrap items-center gap-2 text-xs">
                {{ formatDistance(step.distance) }}
                <span v-if="step.from_level !== step.to_level" class="text-zinc-500 dark:text-zinc-400">
                  {{ t(step.to_level > step.from_level ? "level_up" : "level_down", { level: step.to_level }) }}
                </span>
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
  level_up: hoch auf Ebene {level}
  level_down: runter auf Ebene {level}
  continue: Weiter
  toll: Maut
  access_restriction: Eingeschränkter Zugang
  access_restriction_hint: enthält Zugangsbeschränkungen
en:
  level_up: up to level {level}
  level_down: down to level {level}
  continue: Continue
  toll: Toll
  access_restriction: Restricted access
  access_restriction_hint: contains access restrictions
</i18n>
