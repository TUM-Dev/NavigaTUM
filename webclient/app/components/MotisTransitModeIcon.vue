<script setup lang="ts">
import {
  mdiWalk,
  mdiBike,
  mdiCar,
  mdiParking,
  mdiTrain,
  mdiBus,
  mdiTram,
  mdiSubway,
  mdiFerry,
  mdiAirplane,
  mdiTransitConnection,
  mdiPhoneClassic,
  mdiVanUtility,
  mdiScooter,
} from "@mdi/js";
import type { components } from "~/api_types";

type ModeResponse = components["schemas"]["ModeResponse"];
defineProps<{ mode: ModeResponse }>();
</script>

<template>
  <div class="bg-blue-100 text-blue-800 flex h-8 w-8 items-center justify-center rounded-full text-xs font-medium">
    <!-- Walking -->
    <MdiIcon v-if="mode === 'Walk'" :path="mdiWalk" :size="16" />

    <!-- Cycling -->
    <MdiIcon v-else-if="mode === 'Bike'" :path="mdiBike" :size="16" />

    <!-- Car and related -->
    <MdiIcon v-else-if="mode === 'Car'" :path="mdiCar" :size="16" />
    <MdiIcon v-else-if="mode === 'CarParking'" :path="mdiParking" :size="16" />

    <!-- Public Transit - General -->
    <MdiIcon v-else-if="mode === 'Transit'" :path="mdiTransitConnection" :size="16" />

    <!-- Rail transport -->
    <MdiIcon v-else-if="mode === 'Rail'" :path="mdiTrain" :size="16" />
    <MdiIcon v-else-if="mode === 'Tram'" :path="mdiTram" :size="16" />
    <MdiIcon v-else-if="mode === 'Subway' || mode === 'Metro'" :path="mdiSubway" :size="16" />

    <!-- Bus transport -->
    <MdiIcon v-else-if="mode === 'Bus'" :path="mdiBus" :size="16" />
    <MdiIcon v-else-if="mode === 'Coach'" :path="mdiBus" :size="16" />

    <!-- Other transport -->
    <MdiIcon v-else-if="mode === 'Ferry'" :path="mdiFerry" :size="16" />
    <MdiIcon v-else-if="mode === 'Airplane'" :path="mdiAirplane" :size="16" />

    <!-- Rental and flexible -->
    <MdiIcon v-else-if="mode === 'Rental'" :path="mdiScooter" :size="16" />
    <MdiIcon v-else-if="mode === 'Flex'" :path="mdiPhoneClassic" :size="16" />
    <MdiIcon v-else-if="mode === 'Odm'" :path="mdiVanUtility" :size="16" />

    <!-- Fallback -->
    <span v-else class="font-bold text-xs">{{ mode.slice(0, 2).toUpperCase() }}</span>
  </div>
</template>
