<script setup lang="ts">
import {
  mdiAirplane,
  mdiBike,
  mdiBus,
  mdiCar,
  mdiFerry,
  mdiParking,
  mdiPhoneClassic,
  mdiScooter,
  mdiSubway,
  mdiTrain,
  mdiTram,
  mdiTransitConnection,
  mdiVanUtility,
  mdiWalk,
} from "@mdi/js";
import type { components } from "~/api_types";

type ModeResponse = components["schemas"]["ModeResponse"];
defineProps<{
  mode: ModeResponse;
  transparent?: boolean;
}>();
</script>

<template>
  <div
    class="flex items-center justify-center text-xs font-medium"
    :class="transparent ? 'text-current' : 'bg-blue-100 text-blue-800 h-8 w-8 rounded-full'"
  >
    <!-- Walking -->
    <MdiIcon v-if="mode === 'walk'" :path="mdiWalk" :size="16" />

    <!-- Cycling -->
    <MdiIcon v-else-if="mode === 'bike'" :path="mdiBike" :size="16" />

    <!-- Car and related -->
    <MdiIcon v-else-if="mode === 'car'" :path="mdiCar" :size="16" />
    <MdiIcon v-else-if="mode === 'car_parking'" :path="mdiParking" :size="16" />

    <!-- Public Transit - General -->
    <MdiIcon v-else-if="mode === 'transit'" :path="mdiTransitConnection" :size="16" />

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
      :size="16"
    />
    <MdiIcon v-else-if="mode === 'tram'" :path="mdiTram" :size="16" />
    <MdiIcon v-else-if="mode === 'subway' || mode === 'metro'" :path="mdiSubway" :size="16" />

    <!-- Bus transport -->
    <MdiIcon v-else-if="mode === 'bus'" :path="mdiBus" :size="16" />
    <MdiIcon v-else-if="mode === 'coach'" :path="mdiBus" :size="16" />

    <!-- Other transport -->
    <MdiIcon v-else-if="mode === 'ferry'" :path="mdiFerry" :size="16" />
    <MdiIcon v-else-if="mode === 'airplane'" :path="mdiAirplane" :size="16" />

    <!-- Rental and flexible -->
    <MdiIcon v-else-if="mode === 'rental'" :path="mdiScooter" :size="16" />
    <MdiIcon v-else-if="mode === 'flex'" :path="mdiPhoneClassic" :size="16" />
    <MdiIcon v-else-if="mode === 'odm'" :path="mdiVanUtility" :size="16" />
    <MdiIcon
      v-else-if="mode === 'cable_car' || mode === 'funicular' || mode === 'areal_lift'"
      :path="mdiTrain"
      :size="16"
    />

    <!-- Fallback -->
    <span v-else class="font-bold text-xs">{{ mode.slice(0, 2).toUpperCase() }}</span>
  </div>
</template>
