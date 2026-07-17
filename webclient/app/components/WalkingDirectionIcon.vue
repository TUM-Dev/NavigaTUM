<script setup lang="ts">
import {
  mdiArrowDown,
  mdiArrowLeft,
  mdiArrowRight,
  mdiArrowTopLeft,
  mdiArrowTopRight,
  mdiArrowUp,
  mdiAxisXRotateClockwise,
  mdiAxisXRotateCounterclockwise,
  mdiElevator,
  mdiElevatorDown,
  mdiElevatorUp,
  mdiPlay,
  mdiStairs,
  mdiStairsDown,
  mdiStairsUp,
} from "@mdi/js";
import type { components } from "~/api_types";

type DirectionResponse = components["schemas"]["DirectionResponse"];
// `vertical` carries the sense of a floor change (derived from the step levels, which the
// direction enum alone does not encode), so stairs and elevators can point up or down.
const props = defineProps<{ direction: DirectionResponse; vertical?: "up" | "down" }>();

const stairsIcon = computed(() =>
  props.vertical === "up" ? mdiStairsUp : props.vertical === "down" ? mdiStairsDown : mdiStairs
);
const elevatorIcon = computed(() =>
  props.vertical === "up"
    ? mdiElevatorUp
    : props.vertical === "down"
      ? mdiElevatorDown
      : mdiElevator
);
</script>

<template>
  <div class="bg-blue-100 dark:bg-blue-800 text-blue-800 dark:text-blue-100 flex h-8 w-8 items-center justify-center rounded-full">
    <!-- Depart -->
    <MdiIcon v-if="direction === 'depart'" :path="mdiPlay" :size="16" />

    <!-- Straight movements -->
    <MdiIcon v-else-if="direction === 'continue'" :path="mdiArrowUp" :size="16" />

    <!-- Left turns -->
    <MdiIcon v-else-if="direction === 'hard_left'" :path="mdiArrowLeft" :size="16" />
    <MdiIcon v-else-if="direction === 'left'" :path="mdiArrowTopLeft" :size="16" />
    <MdiIcon v-else-if="direction === 'slightly_left'" :path="mdiArrowTopLeft" :size="16" class="opacity-60" />

    <!-- Right turns -->
    <MdiIcon v-else-if="direction === 'hard_right'" :path="mdiArrowRight" :size="16" />
    <MdiIcon v-else-if="direction === 'right'" :path="mdiArrowTopRight" :size="16" />
    <MdiIcon v-else-if="direction === 'slightly_right'" :path="mdiArrowTopRight" :size="16" class="opacity-60" />

    <!-- U-turns -->
    <MdiIcon v-else-if="direction === 'uturn_left'" :path="mdiArrowDown" :size="16" class="rotate-180" />
    <MdiIcon v-else-if="direction === 'uturn_right'" :path="mdiArrowDown" :size="16" class="rotate-180" />

    <!-- Circles/Roundabouts -->
    <MdiIcon v-else-if="direction === 'circle_clockwise'" :path="mdiAxisXRotateClockwise" :size="16" />
    <MdiIcon v-else-if="direction === 'circle_counterclockwise'" :path="mdiAxisXRotateCounterclockwise" :size="16" />

    <!-- Vertical movement -->
    <MdiIcon v-else-if="direction === 'stairs'" :path="stairsIcon" :size="16" />
    <MdiIcon v-else-if="direction === 'elevator'" :path="elevatorIcon" :size="16" />

    <!-- Fallback -->
    <span v-else class="text-xs font-bold">{{ (direction as string).slice(0, 2).toUpperCase() }}</span>
  </div>
</template>
