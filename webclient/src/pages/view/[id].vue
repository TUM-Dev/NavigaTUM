<script setup lang="ts">
import ShareButton from "@/components/ShareButton.vue";
import DetailsInteractiveMap from "@/components/DetailsInteractiveMap.vue";
import DetailsRoomOverviewSection from "@/components/DetailsRoomOverviewSection.vue";
import DetailsBuildingOverviewSection from "@/components/DetailsBuildingOverviewSection.vue";
import DetailsInfoSection from "@/components/DetailsInfoSection.vue";
import DetailsSources from "@/components/DetailsSources.vue";
import DetailsFeedbackButton from "@/components/DetailsFeedbackButton.vue";
import DetailsRoomfinderMap from "@/components/DetailsRoomfinderMap.vue";
import { useI18n } from "vue-i18n";
import { setDescription, setTitle } from "@/composables/common";
import { useClipboard } from "@vueuse/core";
import { selectedMap, useDetailsStore } from "@/stores/details";
import { computed, nextTick, onMounted, ref, watchEffect } from "vue";
import { useFetch } from "@/composables/fetch";
import { useRoute } from "vue-router";
import router from "@/router";
import type { components } from "@/api_types";
type DetailsResponse = components["schemas"]["DetailsResponse"];

const { t } = useI18n({ useScope: "local" });

const route = useRoute();

function loadData(data: DetailsResponse) {
  if (route.fullPath !== data.redirect_url) router.replace({ path: data.redirect_url });
  // --- Additional data ---
  setTitle(data.name);
  setDescription(genDescription(data));
  state.$reset();
  state.loadData(data);
  tryToLoadMap();
}
watchEffect(() => {
  if (route.params.id === "root") {
    router.replace({ path: "/" });
    return;
  }
  useFetch<DetailsResponse>(`/api/get/${route.params.id}`, loadData, () => {
    router.push({
      name: "404",
      // preserve current path and remove the first char to avoid the target URL starting with `//`
      params: { catchAll: route.path.substring(1).split("/") },
      query: route.query,
      hash: route.hash,
    });
  });
});

const state = useDetailsStore();
const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const { copy, copied, isSupported: clipboardIsSupported } = useClipboard({ source: clipboardSource });
const appURL = import.meta.env.VITE_APP_URL;

function genDescription(d: DetailsResponse) {
  const detailsFor = t("details_for");
  let description = `${detailsFor} ${d.type_common_name} ${d.name}`;
  if (d.props.computed) {
    description += ":";
    d.props.computed.forEach((prop) => {
      description += `\n- ${prop.name}: ${prop.text}`;
    });
  }
  return description;
}
// --- Loading components ---
function tryToLoadMap() {
  /**
   * Try to load the entry map (interactive or roomfinder). It requires the map container
   * element to be loaded in DOM.
   * @return {boolean} Whether the loading was successful
   */
  if (document.getElementById("interactive-map") !== null) {
    if (state.map.selected === selectedMap.interactive) interactiveMap.value?.loadInteractiveMap();
    else roomfinderMap.value?.loadRoomfinderMap(state.map.roomfinder.selected_index);
    return true;
  }
  return false;
}

// following variables are bound to ref objects
const feedbackButton = ref<InstanceType<typeof DetailsFeedbackButton> | null>(null);
const interactiveMap = ref<InstanceType<typeof DetailsInteractiveMap> | null>(null);
const roomfinderMap = ref<InstanceType<typeof DetailsRoomfinderMap> | null>(null);
onMounted(() => {
  window.addEventListener("resize", () => {
    if (state.map.selected === selectedMap.roomfinder) {
      roomfinderMap.value?.loadRoomfinderMap(state.map.roomfinder.selected_index);
      roomfinderMap.value?.loadRoomfinderModalMap();
    }
  });

  nextTick(() => {
    // Even though 'mounted' is called there is no guarantee apparently,
    // that we can reference the map by ID in the DOM yet. For this reason we
    // try to poll now (Not the best solution probably)
    let timeoutInMs = 25;

    function pollMap() {
      if (!tryToLoadMap()) {
        console.warn(
          `'mounted' called, but page doesn't appear to be mounted yet. Retrying to load the map in ${timeoutInMs}ms`,
        );
        window.setTimeout(pollMap, timeoutInMs);
        timeoutInMs *= 1.5;
      }
    }

    pollMap();
  });
});
</script>

<template>
  <div id="view-view" v-if="state.data">
    <!-- Header image (on mobile) -->
    <a
      class="show-sm header-image-mobile c-hand"
      @click="state.showImageSlideshow(state.image.shown_image_id || 0)"
      v-if="state.image.shown_image"
    >
      <img
        :alt="t('image_alt')"
        :src="`${appURL}/cdn/header/${state.image.shown_image.name}`"
        class="block h-auto max-w-full bg-zinc-100"
      />
    </a>

    <!-- Breadcrumbs -->
    <ol class="breadcrumb" vocab="https://schema.org/" typeof="BreadcrumbList">
      <li
        class="breadcrumb-item"
        v-for="(p, i) in state.data.parent_names"
        :key="p"
        property="itemListElement"
        typeof="ListItem"
      >
        <RouterLink v-bind="{ to: '/view/' + state.data.parents[i] }" property="item" typeof="WebPage">
          <span property="name">{{ p }}</span>
        </RouterLink>
        <meta property="position" :content="`${i + 1}`" />
      </li>
    </ol>

    <!-- Entry header / title -->
    <div class="entry-header">
      <div class="title">
        <div class="hide-sm" v-if="clipboardIsSupported">
          <button
            class="btn btn-link btn-action btn-sm"
            :title="t('header.copy_link')"
            @click="copy(`https://nav.tum.de${route.fullPath}`)"
          >
            <i class="icon icon-check" v-if="copied" />
            <i class="icon icon-link" v-else />
          </button>
        </div>
        <h1>
          {{ state.data.name }}
          <!-- <small class="label">Exaktes Ergebnis</small>-->
        </h1>
      </div>
      <div class="columns subtitle">
        <div class="column col-auto">
          <span>{{ state.data.type_common_name }}</span>
        </div>
        <div class="column col-auto col-ml-auto">
          <template v-if="state.data?.props?.calendar_url">
            <a
              class="btn btn-link btn-action btn-sm"
              :href="state.data.props.calendar_url"
              target="_blank"
              :title="t('header.calendar')"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 16 16"
                fill="currentColor"
                style="margin-bottom: -2px"
              >
                <path
                  d="M4.571 0A1.143 1.143 0 0 0 3.43 1.143H2.286A2.306 2.306 0 0 0 0 3.429v10.285A2.306 2.306 0 0 0 2.286 16h11.428A2.306 2.306 0 0 0 16 13.714V3.43a2.306 2.306 0 0 0-2.286-2.286h-1.143A1.143 1.143 0 0 0 11.43 0a1.143 1.143 0 0 0-1.143 1.143H5.714A1.143 1.143 0 0 0 4.571 0zM2.286 5.714h11.428v8H2.286v-8z"
                />
                <path
                  d="M6.857 6.857v2.286h2.286V6.857H6.857zm3.429 0v2.286h2.285V6.857h-2.285zm-6.857 3.429v2.285h2.285v-2.285H3.43zm3.428 0v2.285h2.286v-2.285H6.857z"
                />
              </svg>
            </a>
          </template>
          <button class="btn btn-link btn-action btn-sm" :title="t('header.external_link')" onclick="this.focus()">
            <!-- The onclick handler is a fix for Safari -->
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
              <path d="M2.912 2.179v1.26H.267V.794h1.197" stroke-linejoin="round" />
              <path d="M1.407 2.297l2.03-2.03" />
              <path d="M2.352.268h1.085v1.085" stroke-linejoin="round" />
            </svg>
          </button>
          <ShareButton :coords="state.data.coords" :name="state.data.name" />
          <DetailsFeedbackButton ref="feedbackButton" />
          <!--<button class="btn btn-link btn-action btn-sm"
                  :title="t('header.favorites')">
            <i class="icon icon-bookmark" />
          </button>-->
        </div>
      </div>
      <div class="divider" />
    </div>

    <!-- First info section (map + infocard) -->
    <div class="columns">
      <!-- Map container -->
      <div class="column col-7 col-md-12" id="map-container">
        <div class="show-sm">
          <div
            class="toast toast-warning"
            v-if="state.data?.type === 'room' && state.data?.maps?.overlays?.default === null"
          >
            {{ t("no_floor_overlay") }}
          </div>
          <div class="toast" v-if="state.data?.props?.comment">
            {{ state.data.props.comment }}
          </div>
        </div>

        <DetailsInteractiveMap ref="interactiveMap" />
        <DetailsRoomfinderMap ref="roomfinderMap" />
        <div class="btn-group btn-group-block">
          <button
            class="btn btn-sm"
            @click="interactiveMap?.loadInteractiveMap(true)"
            :class="{
              active: state.map.selected === selectedMap.interactive,
            }"
          >
            {{ t("map.interactive") }}
          </button>
          <button
            class="btn btn-sm"
            @click="roomfinderMap?.loadRoomfinderMap(state.map.roomfinder.selected_index, true)"
            :class="{
              active: state.map.selected === selectedMap.roomfinder,
            }"
            :disabled="!state.data.maps.roomfinder?.available"
          >
            {{ t("map.roomfinder") }}
          </button>
        </div>
        <div class="divider" style="margin-top: 10px" />
      </div>

      <DetailsInfoSection />
    </div>

    <DetailsBuildingOverviewSection :buildings="state.data?.sections?.buildings_overview" />
    <DetailsRoomOverviewSection :rooms="state.data?.sections?.rooms_overview" />
    <DetailsSources />
  </div>
</template>

<style lang="scss">
@import "@/assets/variables";

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
        color: text-gray;
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
        transition:
          opacity 0.05s,
          transform 0.05s;

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

@keyframes delay-btn {
  from {
    pointer-events: none;
    color: text-gray;
  }

  to {
    pointer-events: all;
    color: $error-color;
  }
}
</style>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  details_for: Details für
  map:
    interactive: Interaktive Karte
    roomfinder: Lagepläne
  no_floor_overlay: Für den angezeigten Raum gibt es leider keine Indoor Karte.
  header:
    calendar: Kalender öffnen
    copy_link: Link kopieren
    external_link: Externe Links
    favorites: Zu Favoriten hinzufügen
en:
  image_alt: Header image, showing the building
  details_for: Details for
  map:
    interactive: Interactive Map
    roomfinder: Site Plans
  no_floor_overlay: There is unfortunately no indoor map for the displayed room.
  header:
    calendar: Open calendar
    copy_link: Copy link
    external_link: External links
    favorites: Add to favorites
</i18n>
