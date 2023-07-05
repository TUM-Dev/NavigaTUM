<script setup lang="ts">
import type { components } from "@/api_types";
import { ref } from "vue";
import "./mvv-monitor.min.js";
type Station = components["schemas"]["Station"];

//const props = defineProps<{
//  readonly stations? : Station[];
//}>();

const props = ref({
  stations: [
    {
      id: "de:09184:2051",
      name: "Oberschlei\u00dfh., Regattaanlage",
      lat: 48.2526907440601,
      lon: 11.5246215288662,
      distance: 400,
      sub_stations: [
        {
          id: "de:09184:2051:0:1",
          name: "Oberschlei\u00dfh., Regattaanlage",
          lat: 48.2526129857745,
          lon: 11.5251605180367,
          parent: "de:09184:2051",
        },
        {
          id: "de:09184:2051:0:2",
          name: "Oberschlei\u00dfh., Regattaanlage",
          lat: 48.2528223347355,
          lon: 11.5240016913201,
          parent: "de:09184:2051",
        },
      ],
    },
  ],
});

function B64String(station_id: string, station_name: string) {
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
}`);
}
function changeStation(station_id: string) {
  const children = document.getElementById("monitor-container")?.children;
  if (!children) {
    return;
  }
  for (let index = 0; index < children.length; index++) {
    const div = children[index];
    if (div.id === station_id) {
      div.setAttribute("style", "display:block");
    } else {
      div.setAttribute("style", "display:none");
    }
  }
}
function flat(stations: any) {
  return stations.flatMap((station) => {
    const y = [station];
    for (const sub of station.sub_stations) {
      y.push(sub);
    }
    return y;
  });
}
</script>

<template>
  <div class="grid-container">
    <div class="station-buttons grid-item">
      <ul>
        <li v-for="station in props.stations" :key="station.id">
          <button class="btn" @click="changeStation(station.id)">
            {{ station.name }} distance: {{ station.distance }}
          </button>
          <ul>
            <li class="substation" v-for="sub in station.sub_stations" :key="sub.id">
              <button class="btn" @click="changeStation(sub.id)">{{ sub.name }}</button>
            </li>
          </ul>
        </li>
      </ul>
    </div>
    <ul id="monitor-container" class="grid-item">
      <div
        :id="station.id"
        v-for="(station, index) in flat(props.stations)"
        :key="station.id"
        :style="index == 0 ? 'display:block' : 'display:none'"
      >
        <div
          id="mvv-departure-monitor"
          class="mvv-departure-monitor"
          :monitor-configuration="B64String(station.id, station.name)"
        />
      </div>
    </ul>
  </div>
</template>
<style>
li.substation {
  padding-left: 50px;
}
.grid-container {
  margin: auto;
  display: table;
}

.station-buttons {
  display: table-cell;
}

.monitor-container {
  display: table-cell;
}

img[alt="MVV Logo"] {
  display: none;
}
</style>
