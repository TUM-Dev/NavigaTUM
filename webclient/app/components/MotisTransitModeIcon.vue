<script setup lang="ts">
import {
  mdiAirplane,
  mdiBike,
  mdiBusArticulatedFront,
  mdiCar,
  mdiFerry,
  mdiGondola,
  mdiParking,
  mdiPhoneClassic,
  mdiScooter,
  mdiSubwayVariant,
  mdiTrain,
  mdiTrainVariant,
  mdiTramSide,
  mdiTransitConnection,
  mdiVanPassenger,
  mdiVanUtility,
  mdiWalk,
} from "@mdi/js";
import type { components } from "~/api_types";

type ModeResponse = components["schemas"]["ModeResponse"];
const props = withDefaults(
  defineProps<{
    mode: ModeResponse;
    // `pill`: blue rounded badge. `inherit`: transparent, takes the surrounding
    // contrast color via `currentColor`. `mode-colored`: transparent, tinted per mode.
    variant?: "pill" | "inherit" | "mode-colored";
  }>(),
  { variant: "pill" }
);

// Color encodes the mode so users can tell `mdiTrain` instances apart in
// the station-header strip (where no route badge is nearby to disambiguate).
const MODE_COLOR: Partial<Record<ModeResponse, string>> = {
  highspeed_rail: "text-red-700 dark:text-red-300",
  long_distance: "text-red-900 dark:text-red-200",
  night_rail: "text-blue-900 dark:text-blue-200",
  regional_fast_rail: "text-orange-700 dark:text-orange-300",
  regional_rail: "text-orange-500 dark:text-orange-400",
  rail: "text-zinc-700 dark:text-zinc-300",
  metro: "text-blue-400 dark:text-blue-300",
  coach: "text-orange-800 dark:text-orange-200",
  ferry: "text-blue-500 dark:text-blue-400",
  airplane: "text-blue-800 dark:text-blue-300",
  cable_car: "text-green-300 dark:text-green-300",
  funicular: "text-green-800 dark:text-green-300",
  areal_lift: "text-blue-300 dark:text-blue-300",
  rental: "text-orange-300 dark:text-orange-300",
  flex: "text-orange-400 dark:text-orange-300",
  odm: "text-zinc-600 dark:text-zinc-300",
  ride_sharing: "text-zinc-500 dark:text-zinc-300",
};

const modeColorClass = computed(() => MODE_COLOR[props.mode] ?? "text-zinc-900 dark:text-zinc-50");

// Modes rendered with an official, self-colored Munich pictogram (see MotisMvvPictogram).
function isMunichMode(mode: ModeResponse): boolean {
  return mode === "subway" || mode === "suburban" || mode === "bus" || mode === "tram";
}
const isMunichPictogram = computed(() => isMunichMode(props.mode));

const variantClass = computed(() => {
  // The Munich pictogram carries its own color, so it gets no tint or pill background -
  // only `pill`'s sizing box is kept so standalone usages stay 8×8.
  if (isMunichPictogram.value) return props.variant === "pill" ? "h-8 w-8" : "";
  if (props.variant === "pill")
    return "bg-blue-100 dark:bg-blue-800 text-blue-800 dark:text-blue-100 h-8 w-8 rounded-full";
  if (props.variant === "mode-colored") return modeColorClass.value;
  // "inherit": no text-* class, so the icon takes the surrounding contrast color.
  return "";
});
</script>

<template>
  <div class="flex items-center justify-center text-xs font-medium" :class="variantClass">
    <!-- Munich U-Bahn / S-Bahn / Bus / Tram: official MVV pictograms in their official colors. -->
    <MotisMvvPictogram v-if="isMunichPictogram" :mode="mode" :monochrome="variant === 'inherit'" />

    <!-- Walking -->
    <MdiIcon v-else-if="mode === 'walk'" :path="mdiWalk" :size="18" />

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
    <MdiIcon v-else-if="mode === 'metro'" :path="mdiSubwayVariant" :size="18" />

    <!-- Bus transport (coach = long-distance; city `bus` uses the Munich pictogram above) -->
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
