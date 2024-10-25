<script>
import { FerrostarMap } from "@stadiamaps/ferrostar-webcomponents";

const runtimeConfig = useRuntimeConfig();
</script>

<template>
  <ferrostar-map
    id="ferrostar"
    :valhalla-endpoint-url="runtimeConfig.public.valhallaUrl"
    :style-url="`${runtimeConfig.public.mapsURL}/maps/styles/navigatum-basemap/style.json`"
    profile="bicycle"
    :center="{ lng: -122.42, lat: 37.81 }"
    :zoom="18"
    :use-voice-guidance="true"
    :geolocate-on-load="true"
  />
</template>

<style scoped>
ferrostar-map {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
