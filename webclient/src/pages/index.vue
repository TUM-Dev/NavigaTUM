<script setup lang="ts">
import { setTitle } from "@/composables/common";
import { useFetch } from "@/composables/fetch";
import type { components } from "@/api_types";
type RootResponse = components["schemas"]["RootResponse"];

const { data } = useFetch<RootResponse>(`/api/get/root`, (d) => setTitle(d.name));
function more(id: string) {
  document.getElementById(`panel-${id}`)?.classList.add("open");
}
function less(id: string) {
  document.getElementById(`panel-${id}`)?.classList.remove("open");
}
</script>

<template>
  <div id="view-main" v-if="data">
    <div class="columns" style="margin-top: 25px">
      <div class="column">
        <h5>{{ $t("view_main.sites") }}</h5>
      </div>
      <!--<div class="column col-auto"><a href="#"><i class="icon icon-location" /> {{ $t("view_main.overview_map") }}</a></div>-->
    </div>
    <div class="columns">
      <div class="column col-6 col-xs-12" v-for="site in data.sites_overview" :key="site.id">
        <div class="panel" v-bind="{ id: `panel-${site.id}` }">
          <div class="panel-header">
            <RouterLink :to="'/view/' + site.id" v-if="site.id">
              <div class="columns">
                <div class="column">
                  <div class="panel-title h6">{{ site.name }}</div>
                </div>
                <div class="column col-auto">
                  <button
                    class="btn btn-link"
                    :style="{ visibility: site.id ? undefined : 'hidden' }"
                    :aria-label="`show the details for the campus '${site.name}'`"
                  >
                    <i class="icon icon-forward" />
                  </button>
                </div>
              </div>
            </RouterLink>
            <div class="columns" v-else>
              <div class="column">
                <div class="panel-title h6">{{ site.name }}</div>
              </div>
            </div>
          </div>
          <div class="panel-body">
            <RouterLink
              :to="'/view/' + c.id"
              v-for="(c, i) in site.children"
              :class="{ 'link-more': i >= site.n_visible }"
              :key="c.id"
              :aria-label="`show the details for the building '${c.name}'`"
            >
              <div class="tile tile-centered">
                <div class="tile-icon">
                  <div class="example-tile-icon">
                    <i class="icon icon-location centered" />
                  </div>
                </div>
                <div class="tile-content">
                  <div class="tile-title">{{ c.name }}</div>
                </div>
                <div class="tile-action">
                  <button class="btn btn-link" :aria-label="`show the details for the building '${c.name}'`">
                    <i class="icon icon-arrow-right" />
                  </button>
                </div>
              </div>
            </RouterLink>
            <button
              class="btn btn-link btn-more"
              :aria-label="$t('view_main.more_aria')"
              @click="more(site.id)"
              v-if="site.children.length > site.n_visible"
            >
              <i class="icon icon-arrow-right" />
              {{ $t("view_main.more") }}
            </button>
            <button class="btn btn-link btn-less" :aria-label="$t('view_main.less_aria')" @click="less(site.id)">
              <i class="icon icon-arrow-up" />
              {{ $t("view_main.less") }}
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
