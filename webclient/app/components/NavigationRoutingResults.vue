<script setup lang="ts">
import type { operations } from "~/api_types";

type NavigationResponse = operations["route_handler"]["responses"][200]["content"]["application/json"];
defineProps<{ data: NavigationResponse }>();
const emit = defineEmits<{
  selectManeuver: [id: { begin_shape_index: number; end_shape_index: number }];
}>();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div>
    <div v-for="(l, i) in data.legs" :key="i" class="gap-1">
      <p class="text-zinc-500 mt-3 flex items-center gap-5 pb-4 font-semibold">
        <span>{{ t("meters", l.summary.length_meters) }}</span>
        <span class="border-zinc-500 flex-grow border-t" />
        <span>{{ t("minutes", Math.ceil(l.summary.time_seconds / 60)) }}</span>
      </p>
      <div
        v-for="(m, j) in l.maneuvers"
        :key="j"
        class="group cursor-pointer py-1"
        @click="emit('selectManeuver', { begin_shape_index: m.begin_shape_index, end_shape_index: m.end_shape_index })"
      >
        <div class="bg-zinc-200 flex flex-row items-center gap-3 rounded-md p-2 py-1 group-hover:bg-zinc-300">
          <NavigationRoutingManeuverIcon :type="m.type" />
          <div>
            <div class="text-zinc-900">{{ m.instruction }}</div>
          </div>
        </div>
        <small v-if="m.length_meters" class="text-zinc-500">{{ t("meters", m.length_meters) }}</small>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  minutes: "sofort | eine Minute | {count} Minuten"
  meters: "hier | einen Meter | {count} Meter"
en:
  minutes: "instant | one minutes | {count} minutes"
  meters: "here | one meter | {count} meters"
</i18n>
