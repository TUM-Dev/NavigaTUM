<script setup lang="ts">
import type { components } from "@/api_types";
import { useToggle } from "@vueuse/core";
type BuildingsOverview = components["schemas"]["BuildingsOverview"];

const props = defineProps<{
  readonly buildings?: BuildingsOverview;
}>();

import("https://www.mvv-muenchen.de/typo3conf/ext/sn_mvv_efa/Resources/Public/mvv-monitor/mvv-monitor.min.js");

function B64String(station_id: string,station_name: string){
  return btoa(`{
    "language": {
        "departure": "Abfahrt",
        "trainStops": "Haltestellen",
        "direction": "Richtung",
        "footerNote": " Copyright",
        "footerText": "Weitere Fahrplanausknfte unter www.mvv-auskunft.de oder mit der MVV-App",
        "headerText": "Abfahrten für heute, ",
        "language": "de",
        "line": "Linie",
        "live": "Live",
        "stop": "Haltestelle",
        "track": "Gleis"
    },
    "isFullscreen": false,
    "stations": [
        {
            "station": {
                "usage": "sf",
                "type": "any",
                "name": "${station_name}",
                "anyType": "stop",
                "sort": "2",
                "quality": "980",
                "best": "0",
                "modes": "0,1,2,4,6",
                
                "id": "${station_id}"
            },
            "lines": [
            ],
            "leadTimeMinutes": 0
        }
    ],
    "lines": [],
    "maxResults": 10,
    "fetchIntervalInMinutes": 3,
    "showNotification": true
}`)
};
function changeStation(station_id:string,station_name:string) {
  let b64=B64String(station_id,station_name);
  document.getElementById("mvv-departure-monitor").setAttribute("monitor-configuration",b64);
}

const [buildingsExpanded, toggleBuildingsExpanded] = useToggle(false);
</script>

<template>
  <section v-if="props.buildings">
    <div class="columns">
      <div class="column">
        <h2>{{ $t("view_view.buildings_overview.title") }}</h2>
      </div>
      <!--<div class="column col-auto">
          <a href="#">Übersichtskarte <i class="icon icon-forward" /></a>
        </div>-->
    </div>
    <!--for station in building.stations:
          <button v-on:click=(()=>{getdivbyclassname(mvv-departure-monitor).setattribute(monitor-configuration,B64String(station.id,station.name))})()>station.name</button>
    -->
    <li v-for="station in props.buildings.nearby_stations"> <!--parent station may give departures of all substations?-->
      <button class="btn" @click="changeStation(station.id,station.name)">{{station.name}}</button>
    </li>
    <div id="mvv-departure-monitor" class="mvv-departure-monitor" monitor-configuration=""></div>
    <div class="columns">
      <template v-for="(b, i) in props.buildings.entries" :key="b.id">
        <div class="column col-4 col-md-12 content" v-if="i < props.buildings.n_visible || buildingsExpanded">
          <RouterLink :to="'/view/' + b.id">
            <div class="tile tile-centered">
              <div class="tile-icon">
                <figure class="avatar avatar-lg">
                  <img
                    v-if="b.thumb"
                    :alt="$t('view_view.buildings_overview.thumbnail_preview')"
                    :src="'/cdn/thumb/' + b.thumb"
                  />
                  <img
                    v-else
                    :alt="$t('view_view.buildings_overview.default_thumbnail_preview')"
                    src="@/assets/thumb-building.webp"
                  />
                </figure>
              </div>
              <div class="tile-content">
                <p class="tile-title">{{ b.name }}</p>
                <small class="tile-subtitle text-dark">{{ b.subtext }}</small>
              </div>
              <div class="tile-action">
                <button class="btn btn-link" :aria-label="`show the details for the building '${b.name}'`">
                  <i class="icon icon-arrow-right" />
                </button>
              </div>
            </div>
          </RouterLink>
        </div>
      </template>
    </div>
    <div v-if="props.buildings.n_visible < props.buildings.entries.length">
      <button class="btn btn-link" @click="toggleBuildingsExpanded()">
        <template v-if="buildingsExpanded">
          <i class="icon icon-arrow-up" />
          {{ $t("view_view.buildings_overview.less") }}
        </template>
        <template v-else>
          <i class="icon icon-arrow-right" />
          {{ $t("view_view.buildings_overview.more") }}
        </template>
      </button>
    </div>
  </section>
</template>

<style lang="scss" scoped>
@import "@/assets/variables";

a {
  text-decoration: none !important;
}

.tile {
  border: 0.05rem solid $card-border;
  padding: 8px;
  border-radius: 0.1rem;
}

button {
  margin-top: 8px;
}
</style>
