<script setup lang="ts">
import type { components } from "@/api_types";
import { onMounted } from "vue";
import { ref } from "vue";
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

const props: components["schemas"]["DetailsResponse"]["poi"] = {
  mvg: [
    {
      distance: 100,
      station_id: "de:09179:6386",
      lat: 48.1403085455476,
      lon: 11.2624841457708,
      name: "Germannsberg",
      sub_stations: [
        {
          station_id: "de:09179:6386:0:1",
          lat: 48.1402965564732,
          lon: 11.2624482131594,
          name: "Germannsberg",
        },
        {
          station_id: "de:09179:6386:0:2",
          lat: 48.1403025510107,
          lon: 11.2625021120765,
          name: "Germannsberg",
        },
        {
          station_id: "de:09179:6386:0:95",
          lat: 48.1403085455476,
          lon: 11.2624841457708,
          name: "Germannsberg",
        },
      ],
    },
    {
      distance: 200,
      station_id: "de:09162:6",
      lat: 48.1403114185532,
      lon: 11.5611056053238,
      name: "Hauptbahnhof (S, U, Bus, Tram)",
      sub_stations: [],
    },
    {
      distance: 130,
      station_id: "de:09162:1627",
      lat: 48.1403664682758,
      lon: 11.4617937800982,
      name: "Weinbergerstra\u00dfe",
      sub_stations: [],
    },
    {
      distance: 2400,
      station_id: "de:09162:1727",
      lat: 48.1404077419819,
      lon: 11.4687401833924,
      name: "Benedikterstra\u00dfe",
      sub_stations: [],
    },
    {
      distance: 140,
      station_id: "de:09162:811",
      lat: 48.140488617811,
      lon: 11.6804229399256,
      name: "Graf-Lehndorff-Stra\u00dfe",
      sub_stations: [],
    },
    {
      distance: 104,
      station_id: "de:09162:1668",
      lat: 48.1405363374306,
      lon: 11.4480940498014,
      name: "Wehnerstra\u00dfe",
      sub_stations: [
        {
          station_id: "de:09162:1668:3:WEN 2",
          lat: 48.1406262550009,
          lon: 11.4481479487185,
          name: "Wehnerstra\u00dfe",
        },
      ],
    },
    {
      distance: 1005,
      station_id: "de:09162:1716",
      lat: 48.1405423319402,
      lon: 11.4251331111361,
      name: "Veldensteinstra\u00dfe",
      sub_stations: [
        {
          station_id: "de:09162:1716:1:1",
          lat: 48.1405962824949,
          lon: 11.425195993206,
          name: "Veldensteinstra\u00dfe",
        },
        {
          station_id: "de:09162:1716:1:2",
          lat: 48.1404943758439,
          lon: 11.4250702290662,
          name: "Veldensteinstra\u00dfe",
        },
      ],
    },
    {
      distance: 180,
      station_id: "de:09162:731",
      lat: 48.1405978994335,
      lon: 11.6620003731536,
      name: "Rennbahnstra\u00dfe",
      sub_stations: [],
    },
    {
      distance: 40,
      station_id: "de:09162:805",
      lat: 48.1406811218803,
      lon: 11.68173855848,
      name: "Martin-Empl-Ring",
      sub_stations: [],
    },
    {
      distance: 400,
      station_id: "de:09162:73",
      lat: 48.1407321820681,
      lon: 11.6000689672441,
      name: "Friedensengel/Villa Stuck",
      sub_stations: [],
    },
    {
      distance: 900,
      station_id: "de:09162:1206",
      lat: 48.1407404161784,
      lon: 11.524217700077,
      name: "Am Lokschuppen",
      sub_stations: [],
    },
    {
      distance: 750,
      station_id: "de:09179:6384",
      lat: 48.1407461448496,
      lon: 11.2998810110539,
      name: "Alling, Griesstra\u00dfe",
      sub_stations: [
        {
          station_id: "de:09179:6384:0:1",
          lat: 48.140776117268,
          lon: 11.3000606741107,
          name: "Alling, Griesstra\u00dfe",
        },
      ],
    },
  ],
};
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

const selected = ref([...props.mvg].sort((s1, s2) => s1.distance - s2.distance)[0].station_id);
</script>
<template>
  <div v-if="props.mvg">
    <div class="columns">
      <div class="column">
        <h2>{{ $t("view_view.rooms_overview.public_transport") }}</h2>
      </div>
      <!--<div class="column col-auto">
        <div class="dropdown"><a class="btn btn-link dropdown-toggle" tabindex="0">{{ $t("view_view.rooms_overview.by_usage") }} <i class="icon icon-caret" /></a>
          <ul class="menu">
                  <li class="menu-item"><a href="#dropdowns">nach Nutzung</a></li>
                  <li class="menu-item"><a href="#dropdowns">nach ...</a></li>
          </ul>
        </div>
      </div>-->
    </div>
    <div class="columns content" v-if="props.mvg">
      <div class="column col-4 col-md-12 col-sm-12" id="button-list">
        <div class="panel">
          <div class="panel-header">
            <div class="panel-title h6">{{ $t("view_view.rooms_overview.by_distance") }}:</div>
          </div>
          <div class="panel-body">
            <ul class="menu">
              <li
                class="menu-item"
                v-for="s in [...props.mvg].sort((s1, s2) => s1.distance - s2.distance)"
                :key="s.station_id"
              >
                <button
                  class="btn"
                  :class="{
                    active: s.station_id === selected,
                  }"
                  @click="selected = s.station_id"
                >
                  <i class="icon icon-arrow-right" />
                  <div class="menu-text">{{ s.name }}</div>
                  <label class="label">{{ Math.round(s.distance) }}m</label>
                </button>
              </li>
            </ul>
          </div>
          <div class="panel-footer">
            <button
              class="btn btn-link btn-sm"
              @click="selected = [...props.mvg].sort((s1, s2) => s1.distance - s2.distance)[0].station_id"
            >
              {{ $t("view_view.rooms_overview.remove_selection") }}
            </button>
          </div>
        </div>
      </div>
      <div class="column col-8 col-md-12 col-sm-12" id="rooms-list">
        <ul id="monitor-container">
          <div
            :id="station.station_id"
            v-for="station in props.mvg"
            :key="station.station_id"
            :style="selected == station.station_id ? 'display:block' : 'display:none'"
          >
            <div
              id="mvv-departure-monitor"
              class="mvv-departure-monitor"
              :monitor-configuration="B64String(station.station_id, station.name)"
            />
          </div>
        </ul>
      </div>
    </div>
  </div>
</template>

<style>
ul#monitor-container {
  margin: 0 0;
  position: relative;
}

.mvv-departure-monitor {
  width: 500px;
  position: absolute;
}
</style>
<style lang="scss" scoped>
@import "@/assets/variables";
.panel {
  .menu {
    padding: 0;
    box-shadow: none;

    .menu-item button {
      text-align: left !important;
      border: 0 transparent !important;
      width: 100%;
    }
    .menu-item {
      height: 32px;
    }

    .menu-item a,
    .menu-item label,
    .menu-item button {
      cursor: pointer;
      user-select: none;
    }
  }

  #category-select .menu-item {
    padding: 0;

    & .icon-arrow-right {
      margin-right: 4px;
    }
  }

  .menu-item button {
    display: flex;
    flex-direction: row;
    box-sizing: border-box;
    width: 100%;

    .menu-text {
      flex-grow: 1;
      flex-shrink: 1;
      text-overflow: ellipsis;
      overflow: hidden;
    }

    .icon,
    label {
      flex-grow: 0;
      flex-shrink: 0;
    }

    .icon {
      top: 5px;
    }
  }

  .panel-title {
    font-weight: bold;
  }

  .panel-body {
    height: 500px;
    padding-bottom: 4px;

    .divider {
      margin: 6px 0;
    }
  }

  .panel-footer {
    color: $text-gray;
  }
}

// 'sm' (mobile)
@media (max-width: 600px) {
  #category-select .panel-body {
    height: 260px;
  }

  #rooms-list .panel-body {
    height: 275px;
  }
}
</style>
