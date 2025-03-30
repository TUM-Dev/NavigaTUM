<script setup lang="ts">
import type { components } from "~/api_types";
import {
  ArrowLeftEndOnRectangleIcon,
  ArrowLeftStartOnRectangleIcon,
  QuestionMarkCircleIcon,
  ArrowTurnUpLeftIcon,
  ArrowTurnUpRightIcon,
  ArrowUpIcon,
  ArrowLeftIcon,
  ArrowRightIcon,
  MapPinIcon,
  ArrowsUpDownIcon,
  TicketIcon,
  ChevronDoubleLeftIcon,
  ChevronDoubleRightIcon,
  ChevronDoubleDownIcon,
  ArrowUpRightIcon,
  ArrowUpLeftIcon,
  ArrowPathIcon,
  ArrowUturnRightIcon,
  ArrowUturnLeftIcon,
  ArrowUpCircleIcon,
} from "@heroicons/vue/24/outline";

type ManeuverTypeResponse = components["schemas"]["ManeuverTypeResponse"];
defineProps<{ types: ManeuverTypeResponse[] }>();
const typeToIcon = {
  building_exit: ArrowLeftEndOnRectangleIcon,
  exit_left: ArrowLeftEndOnRectangleIcon,
  building_exit: ArrowLeftEndOnRectangleIcon,
  exit_left: ArrowLeftEndOnRectangleIcon,
  exit_right: ArrowLeftEndOnRectangleIcon,
  building_enter: ArrowLeftStartOnRectangleIcon,
  elevator_enter: ArrowsUpDownIcon,
  escalator_enter: ArrowsUpDownIcon,
  steps_enter: ArrowsUpDownIcon,
  transit: TicketIcon,
  transit_transfer: TicketIcon,
  transit_remain_on: TicketIcon,
  transit_connection_start: TicketIcon,
  transit_connection_transfer: TicketIcon,
  transit_connection_destination: TicketIcon,
  post_transit_connection_destination: TicketIcon,
  ferry_enter: TicketIcon,
  ferry_exit: TicketIcon,
  left: ArrowTurnUpLeftIcon,
  sharp_left: ArrowTurnUpLeftIcon,
  right: ArrowTurnUpRightIcon,
  sharp_right: ArrowTurnUpRightIcon,
  start: ArrowUpIcon,
  becomes: ArrowUpIcon,
  stay_straight: ArrowUpIcon,
  ramp_straight: ArrowUpIcon,
  merge_left: ChevronDoubleLeftIcon,
  merge_right: ChevronDoubleRightIcon,
  merge: ChevronDoubleDownIcon,
  slight_right: ArrowUpRightIcon,
  stay_right: ArrowUpRightIcon,
  ramp_right: ArrowUpRightIcon,
  slight_left: ArrowUpLeftIcon,
  stay_left: ArrowUpLeftIcon,
  ramp_left: ArrowUpLeftIcon,
  roundabout_enter: ArrowPathIcon,
  roundabout_exit: ArrowPathIcon,
  circle_clockwise: ArrowPathIcon,
  circle_counterclockwise: ArrowPathIcon,
  uturn_right: ArrowUturnRightIcon,
  uturn_left: ArrowUturnLeftIcon,
  continue: ArrowUpCircleIcon,
  start_left: ArrowLeftIcon,
  start_right: ArrowRightIcon,
  destination: MapPinIcon,
  destination_left: MapPinIcon,
  destination_right: MapPinIcon,
  none: QuestionMarkCircleIcon,
};
</script>

<template>
  <div
    class="text-zinc-900 min-w-11 rounded-full p-2 flex -space-x-1 overflow-hidden"
    v-for="type in types"
    :key="type"
  >
    <component :is="typeToIcon[type]" class="mx-auto h-7 w-7" />
  </div>
</template>
