<script setup lang="ts">
import {
  mdiAirplane,
  mdiBike,
  mdiBus,
  mdiBusArticulatedFront,
  mdiCar,
  mdiFerry,
  mdiGondola,
  mdiParking,
  mdiPhoneClassic,
  mdiScooter,
  mdiSubway,
  mdiSubwayVariant,
  mdiTrain,
  mdiTrainCarPassenger,
  mdiTrainVariant,
  mdiTram,
  mdiTramSide,
  mdiTransitConnection,
  mdiVanPassenger,
  mdiVanUtility,
  mdiWalk,
} from "@mdi/js";
import type { components } from "~/api_types";

type ModeResponse = components["schemas"]["ModeResponse"];
const props = defineProps<{
  mode: ModeResponse;
  transparent?: boolean;
}>();

// Color encodes the mode so users can tell `mdiTrain` instances apart in
// the station-header strip (where no route badge is nearby to disambiguate).
const MODE_COLOR: Partial<Record<ModeResponse, string>> = {
  highspeed_rail: "text-red-700",
  long_distance: "text-red-900",
  night_rail: "text-blue-900",
  regional_fast_rail: "text-orange-700",
  regional_rail: "text-orange-500",
  rail: "text-zinc-700",
  tram: "text-red-500",
  subway: "text-blue-700",
  metro: "text-blue-400",
  suburban: "text-green-700",
  bus: "text-green-500",
  coach: "text-orange-800",
  ferry: "text-blue-500",
  airplane: "text-blue-800",
  cable_car: "text-green-300",
  funicular: "text-green-800",
  areal_lift: "text-blue-300",
  rental: "text-orange-300",
  flex: "text-orange-400",
  odm: "text-zinc-600",
  ride_sharing: "text-zinc-500",
};

const modeColorClass = computed(() => MODE_COLOR[props.mode] ?? "text-zinc-900");
</script>

<template>
  <div
    class="flex items-center justify-center text-xs font-medium"
    :class="
      transparent
        ? modeColorClass
        : 'bg-blue-100 dark:bg-blue-800 text-blue-800 dark:text-blue-100 h-8 w-8 rounded-full'
    "
  >
    <!-- Walking -->
    <MdiIcon v-if="mode === 'walk'" :path="mdiWalk" :size="18" />

    <!-- Cycling -->
    <MdiIcon v-else-if="mode === 'bike'" :path="mdiBike" :size="18" />

    <!-- Car and related -->
    <MdiIcon v-else-if="mode === 'car'" :path="mdiCar" :size="18" />
    <MdiIcon v-else-if="mode === 'car_parking'" :path="mdiParking" :size="18" />

    <!-- Public Transit - General -->
    <MdiIcon v-else-if="mode === 'transit'" :path="mdiTransitConnection" :size="18" />

    <!-- Rail transport -->
    <MdiIcon
      v-else-if="
        mode === 'rail' ||
        mode === 'highspeed_rail' ||
        mode === 'long_distance' ||
        mode === 'night_rail' ||
        mode === 'regional_fast_rail' ||
        mode === 'regional_rail'
      "
      :path="mdiTrain"
      :size="18"
    />
    <MdiIcon v-else-if="mode === 'tram'" :path="mdiTram" :size="18" />
    <MdiIcon v-else-if="mode === 'subway'" :path="mdiSubway" :size="18" />
    <MdiIcon v-else-if="mode === 'metro'" :path="mdiSubwayVariant" :size="18" />
    <MdiIcon v-else-if="mode === 'suburban'" :path="mdiTrainCarPassenger" :size="18" />

    <!-- Bus transport -->
    <MdiIcon v-else-if="mode === 'bus'" :path="mdiBus" :size="18" />
    <MdiIcon v-else-if="mode === 'coach'" :path="mdiBusArticulatedFront" :size="18" />

    <!-- Other transport -->
    <MdiIcon v-else-if="mode === 'ferry'" :path="mdiFerry" :size="18" />
    <MdiIcon v-else-if="mode === 'airplane'" :path="mdiAirplane" :size="18" />

    <!-- Rental and flexible -->
    <MdiIcon v-else-if="mode === 'rental'" :path="mdiScooter" :size="18" />
    <MdiIcon v-else-if="mode === 'flex'" :path="mdiPhoneClassic" :size="18" />
    <MdiIcon v-else-if="mode === 'odm'" :path="mdiVanUtility" :size="18" />
    <MdiIcon v-else-if="mode === 'ride_sharing'" :path="mdiVanPassenger" :size="18" />

    <!-- Cable / aerial -->
    <MdiIcon v-else-if="mode === 'cable_car'" :path="mdiTramSide" :size="18" />
    <MdiIcon v-else-if="mode === 'funicular'" :path="mdiTrainVariant" :size="18" />
    <MdiIcon v-else-if="mode === 'areal_lift'" :path="mdiGondola" :size="18" />

    <!-- Fallback -->
    <span v-else class="font-bold text-xs">{{ mode.slice(0, 2).toUpperCase() }}</span>
  </div>
</template>
