<script setup lang="ts">
// Alread pre-request root data:
navigatum.getData("root");

navigatum.registerView("main", {
  name: "view-main",
  template: { gulp_inject: "view-main.inc" },
  data: function () {
    return {
      root_data: null,
    };
  },
  beforeRouteEnter: function (to, from, next) {
    navigatum.getData("root").then((data) => next((vm) => vm.setData(data)));
  },
  beforeRouteUpdate: function (to, from, next) {
    // beforeRouteUpdate not used for now since data rarely changes
    next();
  },
  methods: {
    setData: function (data) {
      this.root_data = data;
      if (data !== null) navigatum.setTitle(data.name);
    },
    more: function (id) {
      document.getElementById(`panel-${id}`).classList.add("open");
    },
    less: function (id) {
      document.getElementById(`panel-${id}`).classList.remove("open");
    },
  },
});
</script>

<style lang="scss">
@import "@assets/variables";

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
                    transition: color .1s;

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
                opacity: .5;
                transition: opacity .1s;

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

<template>
  <div id="view-main" v-if="root_data">
  <div class="columns" style="margin-top: 25px">
    <div class="column"><h5>{{ $t("view_main.sites") }}</h5></div>
    <!--<div class="column col-auto"><a href="#"><i class="icon icon-location"></i> {{ $t("view_main.overview_map") }}</a></div>-->
  </div>
  <div class="columns">
    <div
      class="column col-6 col-xs-12"
      v-for="site in root_data.sites_overview"
    >
      <div class="panel" v-bind="{'id': 'panel-' + site.id}">
        <div class="panel-header">
          <router-link v-bind:to="'/view/' + site.id" v-if="site.id">
            <div class="columns">
              <div class="column">
                <div class="panel-title h6">{{ site.name }}</div>
              </div>
              <div class="column col-auto">
                <button
                  class="btn btn-link"
                  v-bind:style="{visibility: site.id ? '' : 'hidden'}"
                >
                  <i class="icon icon-forward"></i>
                </button>
              </div>
            </div>
          </router-link>
          <div class="columns" v-else>
            <div class="column">
              <div class="panel-title h6">{{ site.name }}</div>
            </div>
          </div>
        </div>
        <div class="panel-body">
          <router-link
            v-bind:to="'/view/' + c.id"
            v-for="(c, i) in site.children"
            v-bind:class="{'link-more': i >= site.n_visible}"
            v-bind:key="c.id"
            v-bind:aria-label="`show the details for the building '` + c.name + `'`"
          >
            <div class="tile tile-centered">
              <div class="tile-icon">
                <div class="example-tile-icon">
                  <i class="icon icon-location centered"></i>
                </div>
              </div>
              <div class="tile-content">
                <div class="tile-title">{{ c.name }}</div>
              </div>
              <div class="tile-action">
                <button
                  class="btn btn-link"
                  v-bind:aria-label="`show the details for the building '` + c.name + `'`"
                >
                  <i class="icon icon-arrow-right"></i>
                </button>
              </div>
            </div>
          </router-link>
          <button
            class="btn btn-link btn-more"
            aria-label="show more buildings"
            v-on:click="more(site.id)"
            v-if="site.children.length > site.n_visible"
          >
            <i class="icon icon-arrow-right"></i>
            {{ $t("view_main.more") }}
          </button>
          <button
            class="btn btn-link btn-less"
            aria-label="show less buildings"
            v-on:click="less(site.id)"
          >
            <i class="icon icon-arrow-up"></i>
            {{ $t("view_main.less") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</div>
</template>
