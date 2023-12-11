<script setup lang="ts">
import { setTitle } from "@/composables/common";
import { useFetch } from "@/composables/fetch";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
import { ArrowRightIcon, ChevronRightIcon, ChevronDownIcon, ChevronUpIcon } from "@heroicons/vue/24/outline";
type RootResponse = components["schemas"]["RootResponse"];

const { t } = useI18n({ useScope: "local" });
const { data } = useFetch<RootResponse>(`/api/get/root`, (d) => setTitle(d.name));

function more(id: string) {
  document.getElementById(`panel-${id}`)?.classList.add("open");
}
function less(id: string) {
  document.getElementById(`panel-${id}`)?.classList.remove("open");
}
</script>

<template>
  <div v-if="data" id="view-main">
    <div class="columns" style="margin-top: 25px">
      <div class="column">
        <h5>{{ t("sites") }}</h5>
      </div>
      <!-- <div class="column col-auto"><a href="#"><i class="icon icon-location" /> {{ t("overview_map") }}</a></div> -->
    </div>
    <div class="columns">
      <div v-for="site in data.sites_overview" :key="site.id" class="col-6 col-xs-12 column">
        <div class="panel" v-bind="{ id: `panel-${site.id}` }">
          <div class="panel-header">
            <RouterLink v-if="site.id" :to="'/view/' + site.id">
              <div class="columns">
                <div class="column">
                  <div class="h6 panel-title">{{ site.name }}</div>
                </div>
                <div class="column col-auto">
                  <button
                    type="button"
                    class="btn btn-link"
                    :style="{ visibility: site.id ? undefined : 'hidden' }"
                    :aria-label="`show the details for the campus '${site.name}'`"
                  >
                    <ArrowRightIcon class="h-4 w-4" />
                  </button>
                </div>
              </div>
            </RouterLink>
            <div v-else class="columns">
              <div class="column">
                <div class="h6 panel-title">{{ site.name }}</div>
              </div>
            </div>
          </div>
          <div class="panel-body">
            <RouterLink
              v-for="(c, i) in site.children"
              :key="c.id"
              :to="'/view/' + c.id"
              :class="{ 'link-more': i >= site.n_visible }"
              :aria-label="`show the details for the building '${c.name}'`"
            >
              <div class="tile tile-centered">
                <div class="tile-icon">
                  <div class="example-tile-icon">
                    <i class="centered icon icon-location" />
                  </div>
                </div>
                <div class="tile-content">
                  <div class="tile-title">{{ c.name }}</div>
                </div>
                <div class="tile-action">
                  <button
                    type="button"
                    class="btn btn-link"
                    :aria-label="`show the details for the building '${c.name}'`"
                  >
                    <ChevronRightIcon class="h-4 w-4" />
                  </button>
                </div>
              </div>
            </RouterLink>
            <button
              v-if="site.children.length > site.n_visible"
              type="button"
              class="btn btn-link btn-more"
              :aria-label="t('more_aria')"
              @click="more(site.id)"
            >
              <div class="flex flex-row gap-2">
                <ChevronDownIcon class="h-4 w-4" />
                {{ t("more") }}
              </div>
            </button>
            <button type="button" class="btn btn-less btn-link" :aria-label="t('less_aria')" @click="less(site.id)">
              <div class="flex flex-row gap-2">
                <ChevronUpIcon class="h-4 w-4" />
                {{ t("less") }}
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
@import "@/assets/variables";

#view-main {
  .panel {
    border: 1px solid $card-border;
    border-radius: 10px;
    overflow: hidden;
    box-shadow: $card-shadow-dark;
    margin: 10px 0;
    padding-bottom: 12px;

    .panel-header {
      width: 100%;
      margin-bottom: 8px;

      & > a {
        text-decoration: none;

        .h6 {
          text-align: left;
          color: $body-font-color;
          transition: color 0.1s;

          &:hover,
          &:active {
            color: $primary-color;
          }
        }

        button {
          margin-top: -7px;
          margin-bottom: -7px;
        }
      }

      a.btn {
        margin: -8px 0;
      }

      .h6 {
        font-weight: bold;
      }
    }

    .panel-body {
      & > a {
        text-decoration: none;
      }

      .link-more {
        opacity: 0.5;
        transition: opacity 0.1s;

        .tile {
          display: none;
        }
      }

      .tile-icon {
        color: $body-font-color;
        margin-top: -4px;
      }

      .tile-title {
        padding-left: 8px;
      }
    }

    .btn-more,
    .btn-less {
      margin-top: 5px;
      padding-bottom: 0;
      padding-left: 0;
    }

    .btn-less {
      display: none;
    }
  }

  .panel.open {
    .panel-body .link-more {
      opacity: 1;

      .tile {
        display: flex;
      }
    }

    .btn-more {
      display: none;
    }

    .btn-less {
      display: inline-block;
    }
  }
}
</style>

<i18n lang="yaml">
de:
  less: weniger
  less_aria: weniger Gebäude anzeigen
  more: mehr
  more_aria: mehr Gebäude anzeigen
  overview_map: Übersichtskarte
  sites: Standorte
en:
  less: less
  less_aria: show more buildings
  more: more
  more_aria: show more buildings
  overview_map: Overview Map
  sites: Sites
</i18n>
