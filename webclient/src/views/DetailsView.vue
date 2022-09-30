<script setup lang="ts">
import ShareButton from "@/components/ShareButton.vue";
import DetailsInteractiveMap from "@/components/DetailsInteractiveMap.vue";
import DetailsOverviewSections from "@/components/DetailsOverviewSections.vue";
import DetailsInfoSection from "@/components/DetailsInfoSection.vue";
import DetailsSources from "@/components/DetailsSources.vue";
import DetailsFeedbackButton from "@/components/DetailsFeedbackButton.vue";
import DetailsRoomfinderMap from "@/components/DetailsRoomfinderMap.vue";

import { useI18n } from "vue-i18n";
const { t } = useI18n({
  inheritLocale: true,
  useScope: "global",
});
//import DetailsFeaturedSection from "@/components/DetailsFeaturedSection.vue";
import {
  getLocalStorageWithExpiry,
  removeLocalStorage,
  setLocalStorageWithExpiry,
} from "@/utils/storage";
import { copyCurrentLink, setDescription, setTitle } from "@/utils/common";
import { selectedMap, useDetailsStore } from "@/stores/details";
import type {
  DetailsResponse,
  RoomfinderMapEntry,
} from "@/codegen";
import { nextTick, ref } from "vue";
import { useFetch } from "@/utils/fetch";
import router from "@/router";

function copyLink(copied) {
  copyCurrentLink(copied);
}

const path = location.pathname.substring(1).split("/");
useFetch<DetailsResponse>(`/api/get/${path[1]}`, (d) => {
  if (d.type === "root") {
    router.replace({ path: "/" });
    return;
  }
  // Redirect to the correct type if necessary. Technically the type information
  // is not required, but it makes nicer URLs.
  const urlTypeName =
    {
      campus: "campus",
      site: "site",
      area: "site", // Currently also "site", maybe "group"? TODO
      building: "building",
      joined_building: "building",
      room: "room",
      virtual_room: "room",
    }[d.type] || "view";
  if (path[0] !== urlTypeName) {
    router.replace({path: `/${urlTypeName}/${path[1]}`});
  }
  state.data = d;
});

const state = useDetailsStore();
const copied = ref(false);
// Coordinate picker states
const coord_counter = ref({
  counter: null as number | null,
  to_confirm_delete: false,
});
// This is called
// - on initial page load
// - when the view is loaded for the first time
// - when the view is navigated to from a different view
// - when the view is navigated to from the same view, but with a different entry
function loadEntryData() {
  state.showImageSlideshow(0, false);

  if (state.data === null) return;

  // --- Maps ---
  // We need to reset state to default here, else it is preserved from the previous page
  state.$reset();

  state.map.selected =
    state.data.maps.default === "interactive"
      ? selectedMap.interactive
      : selectedMap.roomfinder;
  // Interactive has to be always available, but roomfinder may be unavailable
  if (state.data.maps.roomfinder !== undefined) {
    // Find default map
    state.data.maps.roomfinder.available.forEach(
      (availableMap: RoomfinderMapEntry, index: number) => {
        if (availableMap.id === state.data?.maps.roomfinder?.default) {
          state.map.roomfinder.selected_index = index;
          state.map.roomfinder.selected_id = availableMap.id;
        }
      }
    );
  }
  // --- Additional data ---
  setTitle(state.data.name);
  setDescription(genDescription());
}

function genDescription() {
  const detailsFor = t("view_view.meta.details_for");
  let description = `${detailsFor} ${state.data?.type_common_name} ${state.data?.name}`;
  if (state.data?.props.computed) {
    description += ":";
    state.data?.props.computed.forEach((prop) => {
      description += `\n- ${prop.name}: ${prop.text}`;
    });
  }
  return description;
}
// --- Loading components ---
function deletePendingCoordinates() {
  if (coord_counter.value.to_confirm_delete) {
    removeLocalStorage("coordinate-feedback");
    coord_counter.value.to_confirm_delete = false;
    state.coord_picker.body_backup = null;
    state.coord_picker.subject_backup = null;
    state.coord_picker.backup_id = null;
  } else {
    coord_counter.value.to_confirm_delete = true;
  }
}
function mounted() {
  if (navigator.userAgent === "Rendertron") return;

  // Update pending coordinate counter on localStorage changes
  const updateCoordinateCounter = function () {
    const coords = getLocalStorageWithExpiry("coordinate-feedback", {});
    coord_counter.value.counter = Object.keys(coords).length;
  };
  window.addEventListener("storage", updateCoordinateCounter);
  updateCoordinateCounter();

  nextTick(() => {
    // Even though 'mounted' is called there is no guarantee apparently,
    // that it really is mounted now. For this reason we try to poll now.
    // (Not the best solution probably)
    let timeoutInMs = 5;

    function pollMap() {
      if (document.getElementById("interactive-map") !== null) {
        if (state.map.selected === selectedMap.interactive) loadInteractiveMap();
        else loadRoomfinderMap(state.map.roomfinder.selected_index);
      } else {
        console.warn(
          `'mounted' called, but page doesn't appear to be mounted yet. Retrying to load the map in ${timeoutInMs}ms`
        );
        window.setTimeout(pollMap, timeoutInMs);
        timeoutInMs *= 1.5;
      }
    }

  if (navigator.userAgent !== "Rendertron") pollMap();
  });
}
</script>

<template>
  <div id="view-view" v-if="state.data">
    <!-- Header image (on mobile) -->
    <a
      class="show-sm header-image-mobile c-hand"
      @click="state.showImageSlideshow(state.image.shown_image_id)"
      v-if="state.image.shown_image"
    >
      <img
        alt="Header-Image, showing the building"
        v-bind:src="'/cdn/header/' + state.image.shown_image.name"
        class="img-responsive"
      />
    </a>

    <!-- Pending coordinates counter (if any) -->
    <div class="panel coord-counter" v-if="coord_counter.counter">
      <div class="panel-body columns">
        <div class="column col col-sm-12 msg">
          {{ $t("view_view.msg.coordinate-counter.msg-1") }}
          <em>{{ coord_counter.counter }} </em>
          <span v-if="coord_counter.counter === 1">
            {{ $t("view_view.msg.coordinate-counter.msg-2") }}
          </span>
          <span v-else>
            {{ $t("view_view.msg.coordinate-counter.msg-2-plural") }}
          </span>
          <button
            class="btn btn-action btn-sm btn-link tooltip tooltip-left"
            v-bind:data-tooltip="$t('view_view.msg.coordinate-counter.info')"
          >
            &#x1f6c8;
          </button>
        </div>
        <div class="column col-auto col-sm-12 btns">
          <button
            class="btn btn-link btn-sm delete"
            v-bind:class="{ 'to-confirm': coord_counter.to_confirm_delete }"
            @click="deletePendingCoordinates"
          >
            <i class="icon icon-cross"></i>
            <span class="default">
              {{ $t("view_view.msg.coordinate-counter.delete") }}
            </span>
            <span class="confirm">
              {{ $t("view_view.msg.coordinate-counter.delete-confirm") }}
            </span>
          </button>
          <button
            class="btn btn-primary btn-sm"
            @click="$refs.feedbackButton.openFeedbackForm"
          >
            <i class="icon icon-check"></i>
            {{ $t("view_view.msg.coordinate-counter.send") }}
          </button>
        </div>
      </div>
    </div>

    <!-- Breadcrumbs -->
    <ol class="breadcrumb" vocab="https://schema.org/" typeof="BreadcrumbList">
      <li
        class="breadcrumb-item"
        v-for="(p, i) in state.data.parent_names"
        property="itemListElement"
        typeof="ListItem"
      >
        <RouterLink
          v-bind="{ to: '/view/' + state.data.parents[i] }"
          property="item"
          typeof="WebPage"
        >
          <span property="name">{{ p }}</span>
        </RouterLink>
        <meta property="position" v-bind:content="i + 1" />
      </li>
    </ol>

    <!-- Entry header / title -->
    <div class="entry-header">
      <div class="title">
        <div class="hide-sm">
          <button
            class="btn btn-link btn-action btn-sm"
            v-bind:title="$t('view_view.header.copy_link')"
            @click="copyLink(copied)"
          >
            <i class="icon icon-check" v-if="copied"></i>
            <i class="icon icon-link" v-else></i>
          </button>
        </div>
        <h1>
          {{ state.data.name
          }}<!-- <small class="label">Exaktes Ergebnis</small>-->
        </h1>
      </div>
      <div class="columns subtitle">
        <div class="column col-auto">
          <span>{{ state.data.type_common_name }}</span>
        </div>
        <div class="column col-auto col-ml-auto">
          <button
            class="btn btn-link btn-action btn-sm"
            v-bind:title="$t('view_view.header.external_link.tooltip')"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 3.704 3.704"
              fill="none"
              stroke="#0065bd"
              stroke-width=".529"
              stroke-linecap="round"
            >
              <path
                d="M2.912 2.179v1.26H.267V.794h1.197"
                stroke-linejoin="round"
              />
              <path d="M1.407 2.297l2.03-2.03" />
              <path d="M2.352.268h1.085v1.085" stroke-linejoin="round" />
            </svg>
          </button>
          <ShareButton v-bind:coords="state.data.coords"/>
          <DetailsFeedbackButton ref="feedbackButton" />
          <!--<button class="btn btn-link btn-action btn-sm"
                  v-bind:title="$t('view_view.header.favorites')">
            <i class="icon icon-bookmark"></i>
          </button>-->
        </div>
      </div>
      <div class="divider"></div>
    </div>

    <!-- First info section (map + infocard) -->
    <div class="columns">
      <!-- Map container -->
      <div class="column col-7 col-md-12" id="map-container">
        <div class="show-sm">
          <div
            class="toast toast-warning"
            v-if="
              state.data.coords.accuracy &&
              state.data.coords.accuracy === 'building'
            "
          >
            {{ $t("view_view.msg.inaccurate_only_building.msg") }}
            <button class="btn btn-sm" @click="addLocationPicker">
              {{ $t("view_view.msg.inaccurate_only_building.btn") }}
            </button>
          </div>
          <div
            class="toast toast-warning"
            v-if="
              state.data.type === 'room' &&
              state.data.maps &&
              state.data.maps.overlays &&
              state.data.maps.overlays.default === null
            "
          >
            {{ $t("view_view.msg.no_floor_overlay") }}
          </div>
          <div
            class="toast"
            v-if="state.data.props && state.data.props.comment"
          >
            {{ state.data.props.comment }}
          </div>
        </div>

        <DetailsInteractiveMap />
        <DetailsRoomfinderMap />
        <div class="btn-group btn-group-block">
          <button
            class="btn btn-sm"
            v-on:click="loadInteractiveMap(true)"
            v-bind:class="{ active: state.map.selected === 'interactive' }"
          >
            {{ $t("view_view.map.interactive") }}
          </button>
          <button
            class="btn btn-sm"
            v-on:click="
              loadRoomfinderMap(state.map.roomfinder.selected_index, true)
            "
            v-bind:class="{
              active: state.map.selected === selectedMap.roomfinder,
            }"
            v-bind:disabled="!state.data.maps.roomfinder?.available"
          >
            {{ $t("view_view.map.roomfinder") }}
          </button>
        </div>
        <div class="divider" style="margin-top: 10px"></div>
      </div>

      <DetailsInfoSection></DetailsInfoSection>
    </div>

    <!-- <DetailsFeaturedSection></DetailsFeaturedSection> -->
    <DetailsOverviewSections></DetailsOverviewSections>
    <DetailsSources></DetailsSources>
  </div>
</template>

<style lang="scss">
@import "../assets/variables";

#view-view {
  /* --- General --- */
  h1 {
    font-size: 1.2rem;
    font-weight: 500;
  }

  h2 {
    font-size: 1rem;
    font-weight: 500;
  }

  /* --- Header --- */
  .header-image-mobile {
    margin: -10px -23px 10px;

    > img {
      min-width: 100%;
      min-height: 100px;
      max-height: 200px;
      object-fit: cover;
    }
  }

  .breadcrumb {
    margin-top: 0;
    font-size: 12px;
  }

  .entry-header {
    .title {
      position: relative;

      & > div {
        position: absolute;
        left: -32px;
        opacity: 0;
        transition: opacity 0.2s;
      }

      &:hover > div {
        opacity: 1;
      }
    }

    .subtitle {
      span {
        color: $text-gray;
      }

      button svg {
        margin-top: 4px;
        stroke: $primary-color;
      }

      .column:last-child {
        position: relative;
      }

      .link-popover {
        position: absolute;
        z-index: 1000;
        padding: 6px 10px;
        width: 200px;
        right: 36px;
        background: $light-color;
        box-shadow: $card-shadow-dark;
        border-radius: 2px;
        border: 1px solid $card-border;
        visibility: hidden;
        opacity: 0;
        transform: translateY(-5px);
        transition: opacity 0.05s, transform 0.05s;

        a,
        button {
          width: 100%;
          margin: 4px 0;
        }

        strong {
          margin-top: 2px;
          display: block;

          & + a,
          & + button {
            margin-top: 2px;
          }
        }
      }

      button:focus + .link-popover,
      .link-popover:hover {
        visibility: visible;
        opacity: 1;
        transform: translateY(0);
      }
    }

    .divider {
      margin-bottom: 22px;
    }
  }

  /* --- Pending coordinates counter --- */
  .coord-counter {
    margin: 8px 0;
    box-shadow: $card-shadow;
    border: 1px solid $card-border;
    border-radius: 4px;
    background: $card-highlighted-bg;

    & .panel-body {
      overflow-y: visible;

      & .msg {
        margin: 15px 0;

        & em {
          color: $theme-accent;
          font-style: normal;
          font-weight: bold;
        }

        & .btn {
          height: 1.3rem;
          width: 1.3rem;

          &::after {
            z-index: 2000;
          }
        }
      }

      & .btns {
        margin: auto 0 12px;

        .delete .default {
          display: inline-block;
        }

        .delete .confirm {
          display: none;
        }

        .delete.to-confirm {
          animation: delay-btn 0.3s steps(1);
          animation-fill-mode: both;
        }

        .delete.to-confirm .default {
          display: none;
        }

        .delete.to-confirm .confirm {
          display: inline-block;
        }
      }
    }
  }

  /* --- Sections general --- */
  section {
    margin-top: 40px;

    .content {
      margin-top: 15px;
    }
  }
}

// 'md' (
@media (max-width: 840px) {
  #view-view {
    .text-md-right {
      text-align: right !important;
    }

    .text-md-center {
      text-align: center !important;
    }

    .mt-md-3 {
      margin-top: 1rem !important;
    }
  }
}

// Animations
@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

// 'sm' (mobile)
@media (max-width: 600px) {
  #view-view {
    #rooms-overview-select .panel-body {
      max-height: 260px;
    }

    #rooms-overview-list .panel-body {
      max-height: 275px;
    }
  }
}

@keyframes delay-btn {
  from {
    pointer-events: none;
    color: $text-gray;
  }

  to {
    pointer-events: all;
    color: $error-color;
  }
}
</style>
