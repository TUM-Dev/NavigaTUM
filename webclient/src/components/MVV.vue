<script setup lang="ts">
import type { components } from "@/api_types";
import { onMounted } from "vue";
import { ref } from "vue";
type Station = components["schemas"]["Station"];
//const props = defineProps<{
//  readonly stations? : Station[];
//}>();
onMounted(() => {
  const script = document.createElement("script");
  script.setAttribute(
    "src",
    "https://www.mvv-muenchen.de/typo3conf/ext/sn_mvv_efa/Resources/Public/mvv-monitor/mvv-monitor.min.js"
  );
  script.setAttribute("type", "text/javascript");
  document.head.appendChild(script);
  script.onload = () => document.dispatchEvent(new Event("DOMContentLoaded"));
});
const props = ref({
  stations: [
    {
      distance: 500,
      id: "de:09179:6300",
      lat: 48.2648076286257,
      lon: 11.1179991154529,
      name: "Vogach",
      sub_stations: [
        {
          id: "de:09179:6300:0:1",
          lat: 48.2648016486408,
          lon: 11.1180260649114,
          name: "Vogach",
        },
        {
          id: "de:09179:6300:0:2",
          lat: 48.2647956686552,
          lon: 11.1179901323001,
          name: "Vogach",
        },
        {
          id: "de:09179:6300:0:95",
          lat: 48.2647478287455,
          lon: 11.1181518290512,
          name: "Vogach",
        },
      ],
    },
    {
      distance: 700,
      id: "de:09184:460",
      lat: 48.264861448458,
      lon: 11.6712265832549,
      name: "Garch.,Forschungsz.",
      sub_stations: [
        {
          id: "de:09184:460:30:1",
          lat: 48.2647777286943,
          lon: 11.6712265832549,
          name: "Garch.,Forschungsz.",
        },
        {
          id: "de:09184:460:30:2",
          lat: 48.264998987772,
          lon: 11.671280482172,
          name: "Garch.,Forschungsz.",
        },
      ],
    },
    {
      distance: 100,
      id: "de:09177:3483",
      lat: 48.2648973283147,
      lon: 11.7594321610149,
      name: "Finsingerm.,Brennerm\u00fchlstr",
      sub_stations: [
        {
          id: "de:09177:3483:0:1",
          lat: 48.2649451680845,
          lon: 11.7594770767791,
          name: "Finsingerm.,Brennerm\u00fchlstr",
        },
        {
          id: "de:09177:3483:0:2",
          lat: 48.2648554684794,
          lon: 11.7593872452507,
          name: "Finsingerm.,Brennerm\u00fchlstr",
        },
      ],
    },
    {
      distance: 30,
      id: "de:09174:7081",
      lat: 48.2649092882614,
      lon: 11.4753579186782,
      name: "Dachau, Karl-Benz-Stra\u00dfe",
      sub_stations: [
        {
          id: "de:09174:7081:0:2",
          lat: 48.2649332081463,
          lon: 11.4754477502067,
          name: "Dachau, Karl-Benz-Stra\u00dfe",
        },
      ],
    },
    {
      distance: 1000,
      id: "de:09174:7095",
      lat: 48.2650647673131,
      lon: 11.4307565648155,
      name: "Dachau, Waldfriedhof",
      sub_stations: [
        {
          id: "de:09174:7095:0:1",
          lat: 48.2649810478825,
          lon: 11.4308284300382,
          name: "Dachau, Waldfriedhof",
        },
        {
          id: "de:09174:7095:0:2",
          lat: 48.2651544665509,
          lon: 11.4307116490513,
          name: "Dachau, Waldfriedhof",
        },
      ],
    },
    {
      distance: 600,
      id: "de:09174:7076",
      lat: 48.2651365267159,
      lon: 11.4708393927985,
      name: "Dachau, Robert-Bosch-Stra\u00dfe",
      sub_stations: [
        {
          id: "de:09174:7076:0:1",
          lat: 48.2653518043198,
          lon: 11.4708393927985,
          name: "Dachau, Robert-Bosch-Stra\u00dfe",
        },
      ],
    },
    {
      distance: 600,
      id: "de:09184:2050",
      lat: 48.2651724063795,
      lon: 11.5867849465359,
      name: "Lohhof, Haimhauser Stra\u00dfe",
      sub_stations: [
        {
          id: "de:09184:2050:0:1",
          lat: 48.2652441656312,
          lon: 11.5867759633831,
          name: "Lohhof, Haimhauser Stra\u00dfe",
        },
        {
          id: "de:09184:2050:0:2",
          lat: 48.2650707472672,
          lon: 11.5867849465359,
          name: "Lohhof, Haimhauser Stra\u00dfe",
        },
      ],
    },
    {
      distance: 4050,
      id: "de:09184:2084",
      lat: 48.2653817039154,
      lon: 11.7307220045304,
      name: "Ismaning, Peterhof",
      sub_stations: [
        {
          id: "de:09184:2084:0:1",
          lat: 48.2646222487695,
          lon: 11.7287007951409,
          name: "Ismaning, Peterhof",
        },
        {
          id: "de:09184:2084:0:2",
          lat: 48.2661351679478,
          lon: 11.7328330454484,
          name: "Ismaning, Peterhof",
        },
      ],
    },
    {
      distance: 350,
      id: "de:09174:6928",
      lat: 48.2658840145045,
      lon: 11.4530258007119,
      name: "Dachau, J.-F.-Kennedy-Platz",
      sub_stations: [
        {
          id: "de:09174:6928:0:1",
          lat: 48.2658301957489,
          lon: 11.4530707164761,
          name: "Dachau, J.-F.-Kennedy-Platz",
        },
        {
          id: "de:09174:6928:0:2",
          lat: 48.2659438130553,
          lon: 11.4530258007119,
          name: "Dachau, J.-F.-Kennedy-Platz",
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
    if (station.sub_stations.length > 1) {
      // if substations == 1 then sub is a duplicate of parent
      for (const sub of station.sub_stations) {
        y.push(sub);
      }
    }
    return y;
  });
}
</script>
<template>
  <div class="MVV">
    <div class="table">
      <div class="station-buttons table-column">
        <ul class="buttons">
          <h2>Nearby Public Transport</h2>
          <li v-for="station in props.stations.sort((s1, s2) => s1.distance - s2.distance)" :key="station.id">
            <button class="btn parent" @click="changeStation(station.id)">
              {{ station.distance }}m: {{ station.name }}
            </button>
            <ul v-if="station.sub_stations.length > 1">
              <!--only 1 substations means sub == parent -->
              <li class="substation" v-for="sub in station.sub_stations" :key="sub.id">
                <button class="btn sub" @click="changeStation(sub.id)">{{ sub.name }}</button>
              </li>
            </ul>
          </li>
        </ul>
      </div>
      <div class="monitor-container table-column">
        <ul id="monitor-container">
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
    </div>
  </div>
</template>

<style>
.MVV {
  display: inline-block;
  margin-right: auto;
  margin-left: auto;
}
.table {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  .table-column {
    flex: 1;
  }
}
ul {
  margin: 0 0;
}

.mvv-departure-monitor {
  width: 500px;
}
.btn {
  text-align: left;
  color: #0065bd;
  cursor: pointer;
  border: none;
  line-height: 85%;
  height: 85%;
}

.btn.sub {
  line-height: 70%;
  height: 70%;
  font-size: 70%;
}

img[alt="MVV Logo"] {
  display: none;
}
</style>
