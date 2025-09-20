<script setup lang="ts">
import {
  mdiArrowLeft,
  mdiArrowLeftTop,
  mdiArrowRight,
  mdiArrowTopLeft,
  mdiArrowTopRight,
  mdiArrowULeftTop,
  mdiArrowUp,
  mdiArrowUpCircle,
  mdiArrowUpDown,
  mdiArrowUpLeft,
  mdiArrowUpRight,
  mdiArrowURightTop,
  mdiChevronDoubleDown,
  mdiChevronDoubleLeft,
  mdiChevronDoubleRight,
  mdiExitToApp,
  mdiLogin,
  mdiMapMarker,
  mdiRefresh,
  mdiTicket,
} from "@mdi/js";
import type { components } from "~/api_types";

type ManeuverTypeResponse = components["schemas"]["ManeuverTypeResponse"];
defineProps<{ type: ManeuverTypeResponse }>();
</script>

<template>
  <div class="text-zinc-900 min-w-11 rounded-full p-2">
    <MdiIcon :path="mdiExitToApp" :size="28" v-if="type === 'building_exit' || type === 'exit_left'" class="mx-auto" />
    <MdiIcon :path="mdiExitToApp" :size="28" v-else-if="type === 'exit_right'" class="mx-auto scale-x-[-1]" />
    <MdiIcon :path="mdiLogin" :size="28" v-else-if="type === 'building_enter'" class="mx-auto" />
    <MdiIcon
      :path="mdiArrowUpDown"
      :size="28"
      v-else-if="type === 'elevator_enter' || type === 'escalator_enter' || type === 'steps_enter'"
      class="mx-auto"
    />

    <MdiIcon
      :path="mdiTicket"
      :size="28"
      v-else-if="
        type === 'transit' ||
        type === 'transit_transfer' ||
        type === 'transit_remain_on' ||
        type === 'transit_connection_start' ||
        type === 'transit_connection_transfer' ||
        type === 'transit_connection_destination' ||
        type === 'post_transit_connection_destination' ||
        type === 'ferry_enter' ||
        type === 'ferry_exit'
      "
      class="mx-auto"
    />

    <MdiIcon :path="mdiArrowTopLeft" :size="28" v-else-if="type === 'left' || type === 'sharp_left'" class="mx-auto" />
    <MdiIcon
      :path="mdiArrowTopRight"
      :size="28"
      v-else-if="type === 'right' || type === 'sharp_right'"
      class="mx-auto"
    />
    <MdiIcon
      :path="mdiArrowUp"
      :size="28"
      v-else-if="type === 'start' || type === 'becomes' || type === 'stay_straight' || type === 'ramp_straight'"
      class="mx-auto"
    />
    <MdiIcon :path="mdiChevronDoubleLeft" :size="28" v-else-if="type === 'merge_left'" class="mx-auto" />
    <MdiIcon :path="mdiChevronDoubleRight" :size="28" v-else-if="type === 'merge_right'" class="mx-auto" />
    <MdiIcon :path="mdiChevronDoubleDown" :size="28" v-else-if="type === 'merge'" class="mx-auto" />
    <MdiIcon
      :path="mdiArrowUpRight"
      :size="28"
      v-else-if="type === 'slight_right' || type === 'stay_right' || type === 'ramp_right'"
      class="mx-auto"
    />
    <MdiIcon
      :path="mdiArrowUpLeft"
      :size="28"
      v-else-if="type === 'slight_left' || type === 'stay_left' || type === 'ramp_left'"
      class="mx-auto"
    />
    <MdiIcon
      :path="mdiRefresh"
      :size="28"
      v-else-if="type === 'roundabout_enter' || type === 'roundabout_exit'"
      class="mx-auto"
    />
    <MdiIcon :path="mdiArrowURightTop" :size="28" v-else-if="type === 'uturn_right'" class="mx-auto" />
    <MdiIcon :path="mdiArrowULeftTop" :size="28" v-else-if="type === 'uturn_left'" class="mx-auto" />
    <MdiIcon :path="mdiArrowUpCircle" :size="28" v-else-if="type === 'continue'" class="mx-auto" />
    <MdiIcon :path="mdiArrowLeft" :size="28" v-else-if="type === 'start_left'" class="mx-auto" />
    <MdiIcon :path="mdiArrowRight" :size="28" v-else-if="type === 'start_right'" class="mx-auto" />
    <MdiIcon
      :path="mdiMapMarker"
      :size="28"
      v-else-if="type === 'destination' || type === 'destination_left' || type === 'destination_right'"
      class="mx-auto"
    />
    <div v-else>{{ type }}</div>
  </div>
</template>
