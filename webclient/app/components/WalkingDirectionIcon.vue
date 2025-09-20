<script setup lang="ts">
import {
  mdiArrowUp,
  mdiArrowLeft,
  mdiArrowRight,
  mdiArrowTopLeft,
  mdiArrowTopRight,
  mdiArrowULeftTop,
  mdiArrowURightTop,
  mdiRotateLeft,
  mdiRotateRight,
  mdiStairs,
  mdiElevator,
  mdiPlay,
  mdiStraight,
} from "@mdi/js";
import type { components } from "~/api_types";

type DirectionResponse = components["schemas"]["DirectionResponse"];
defineProps<{ direction: DirectionResponse }>();
</script>

<template>
  <div class="bg-blue-100 text-blue-800 flex h-8 w-8 items-center justify-center rounded-full">
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
    <MdiIcon v-else-if="direction === 'uturn_left'" :path="mdiArrowULeftTop" :size="16" />
    <MdiIcon v-else-if="direction === 'uturn_right'" :path="mdiArrowURightTop" :size="16" />

    <!-- Circles/Roundabouts -->
    <MdiIcon v-else-if="direction === 'circle_clockwise'" :path="mdiRotateRight" :size="16" />
    <MdiIcon v-else-if="direction === 'circle_counterclockwise'" :path="mdiRotateLeft" :size="16" />

    <!-- Vertical movement -->
    <MdiIcon v-else-if="direction === 'stairs'" :path="mdiStairs" :size="16" />
    <MdiIcon v-else-if="direction === 'elevator'" :path="mdiElevator" :size="16" />

    <!-- Fallback -->
    <span v-else class="text-xs font-bold">{{ direction.slice(0, 2).toUpperCase() }}</span>
  </div>
</template>
