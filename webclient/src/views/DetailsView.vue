<script lang="ts">
import {
  getLocalStorageWithExpiry,
  removeLocalStorage,
  setLocalStorageWithExpiry,
} from "@/utils/storage";
import { copyCurrentLink, setDescription, setTitle } from "@/utils/common";
import ShareButton from "@/components/ShareButton.vue";
import { selectedMap, useDetailsStore } from "@/stores/details";
import DetailsInteractiveMap from "@/components/DetailsInteractiveMap.vue";
import DetailsOverviewSections from "@/components/DetailsOverviewSections.vue";
import DetailsInfoSection from "@/components/DetailsInfoSection.vue";
import type { RoomfinderMapEntry } from "@/codegen";

function viewNavigateTo(to, from, next, component) {
  navigatum.getData(to.params.id).then((data) => {
    function finish() {
      if (component) {
        next();
        component.loadEntryData(data);
      } else {
        next((vm) => {
          vm.loadEntryData(data);
        });
      }
    }

    if (data === null) {
      finish();
    } else if (data.type === "root") {
      next("/");
    } else {
      // Redirect to the correct type if necessary. Technically the type information
      // is not required, but it makes nicer URLs.
      let urlTypeName = {
        campus: "campus",
        site: "site",
        area: "site", // Currently also "site", maybe "group"? TODO
        building: "building",
        joined_building: "building",
        room: "room",
        virtual_room: "room",
      }[data.type];
      if (urlTypeName === undefined) urlTypeName = "view";

      if (!to.path.slice(1).startsWith(urlTypeName)) {
        next(`/${urlTypeName}/${to.params.id}`);
      } else {
        finish();
      }
    }
  });
}

export default {
  components: [
    ShareButton,
    DetailsInteractiveMap,
    DetailsInfoSection,
    DetailsOverviewSections,
  ],
  data: function () {
    return {
      state: useDetailsStore(),
      copied: false,
      // Coordinate picker states
      coord_counter: {
        counter: null,
        to_confirm_delete: false,
      },
      coord_picker: {
        // The coordinate picker keeps backups of the subject and body
        // in case someone writes a text and then after that clicks
        // the set coordinate button in the feedback form. If we wouldn't
        // make a backup, this would be lost after clicking confirm there.
        backup_id: null as string | null,
        subject_backup: null as string | null,
        body_backup: null as string | null,
        force_reopen: false,
      },
    };
  },
  beforeRouteEnter: function (to, from, next) {
    viewNavigateTo(to, from, next, null);
  },
  beforeRouteUpdate: function (to, from, next) {
    viewNavigateTo(to, from, next, this);
  },
  methods: {
    // This is called
    // - on initial page load
    // - when the view is loaded for the first time
    // - when the view is navigated to from a different view
    // - when the view is navigated to from the same view, but with a different entry
    loadEntryData: function (data) {
      this.state.data = data;

      this.state.showImageShowcase(0, false);

      if (data === null) return;

      // --- Maps ---
      // We need to reset state to default here, else it is preserved from the previous page
      this.state.$reset();

      this.state.map.selected = data.maps.default;
      // Interactive has to be always available, but roomfinder may be unavailable
      if ("roomfinder" in data.maps) {
        // Find default map
        data.maps.roomfinder.available.forEach(
          (availableMap: RoomfinderMapEntry, index: number) => {
            if (availableMap.id === data.maps.roomfinder.default) {
              this.state.map.roomfinder.selected_index = index;
              this.state.map.roomfinder.selected_id = availableMap.id;
            }
          }
        );
      }

      // Maps can only be loaded after first mount because then the elements are
      // created and can be referenced by id.
      if (this.is_mounted) this.loadMap();

      // --- Additional data ---
      setTitle(data.name);
      setDescription(this.genDescription(data));
    },
    genDescription: function (data) {
      const detailsFor = $t("view_view.meta.details_for");
      let description = `${detailsFor} ${data.type_common_name} ${data.name}`;
      if (data.props.computed) {
        description += ":";
        data.props.computed.forEach((prop) => {
          description += `\n- ${prop.name}: ${prop.text}`;
        });
      }
      return description;
    },
    // --- Loading components ---
    // When these methods are called, the view has already been mounted,
    // so we can find elements by id.
    loadMap: function () {
      if (navigator.userAgent === "Rendertron") {
        return;
      }
      if (this.state.map.selected === selectedMap.interactive)
        this.loadInteractiveMap();
      else if (this.state.map.selected === selectedMap.roomfinder)
        this.loadRoomfinderMap(this.state.map.roomfinder.selected_index);
    },
    _getFeedbackSubject: function (currentEdits) {
      if (Object.keys(currentEdits).length > 1) {
        return (
          `[${this.state.data.id} et.al.]: ` +
          $t("feedback.coordinatepicker.edit_coordinates_subject")
        );
      }

      const subjectPrefix = `[${this.state.data.id}]: `;
      const subjectMsg =
        Object.keys(currentEdits).length === 0
          ? ""
          : $t("feedback.coordinatepicker.edit_coordinate_subject");

      // The subject backup is only loaded (and supported) when a single
      // entry is being edited
      if (
        this.coord_picker.subject_backup &&
        this.coord_picker.backup_id === this.state.data.id &&
        this.coord_picker.subject_backup !== subjectPrefix
      ) {
        const backup = this.coord_picker.subject_backup;
        this.coord_picker.subject_backup = null;
        return backup;
      }
      return subjectPrefix + subjectMsg;
    },
    _getFeedbackBody: function (currentEdits) {
      // Look up whether there is a backup of the body and extract the section
      // that is not the coordinate
      let actionMsg = "";
      if (
        this.coord_picker.body_backup &&
        this.coord_picker.backup_id === this.state.data.id
      ) {
        const parts = this.coord_picker.body_backup.split("\n```");
        if (parts.length === 1) {
          actionMsg = parts[0];
        } else {
          actionMsg = parts[0] + parts[1].split("```").slice(1).join("\n");
        }

        this.coord_picker.body_backup = null;
      }

      if (Object.keys(currentEdits).length === 0) {
        // For no edits, don't show a badly formatted message
        // (This is "" if there was no backup)
        return actionMsg;
      }

      const defaultActionMsg =
        this.state.data.coords.accuracy === "building"
          ? $t("feedback.coordinatepicker.add_coordinate")
          : $t("feedback.coordinatepicker.correct_coordinate");
      actionMsg = actionMsg || defaultActionMsg;

      if (Object.keys(currentEdits).length > 1) {
        // The body backup is discarded if more than a single entry
        // is being edited (because then it is not supported).
        actionMsg = $t("feedback.coordinatepicker.edit_multiple_coordinates");
      }

      let editStr = "";
      Object.entries(currentEdits).forEach(([key, value]) => {
        editStr += `"${key}": {coords: {lat: ${value.coords.lat}, lon: ${value.coords.lon}}},`;
      });

      return `${actionMsg}\n\`\`\`\n${editStr}\`\`\``;
    },
    openFeedbackForm: function () {
      // The feedback form is opened. This may be prefilled with previously corrected coordinates.
      // Maybe get the old coordinates from localstorage
      const currentEdits = getLocalStorageWithExpiry("coordinate-feedback", {});
      const body = this._getFeedbackBody(currentEdits);
      const subject = this._getFeedbackSubject(currentEdits);

      document
        .getElementById("feedback-coordinate-picker")
        .addEventListener("click", this.addLocationPicker);

      /* global openFeedback */
      openFeedback("entry", subject, body);
    },
    confirmLocationPicker: function () {
      // add the current edits to the feedback
      const currentEdits = getLocalStorageWithExpiry("coordinate-feedback", {});
      const location = this.map.interactive.marker2.getLngLat();
      currentEdits[this.state.data.id] = {
        coords: { lat: location.lat, lon: location.lng },
      };
      // save to local storage with ttl of 12h (garbage-collected on next read)
      setLocalStorageWithExpiry("coordinate-feedback", currentEdits, 12);

      this.map.interactive.marker2.remove();
      this.map.interactive.marker2 = null;

      // A feedback form is only opened when this is the only (and therefore
      // first coordinate). If there are more coordinates we can assume
      // someone is doing batch edits. They can then use the send button in
      // the coordinate counter at the top of the page.
      if (
        Object.keys(currentEdits).length === 1 ||
        this.coord_picker.force_reopen
      ) {
        this.coord_picker.force_reopen = false;
        this.openFeedbackForm();
      }

      // The helptext (which says thet you can edit multiple coordinates in bulk)
      // is also only shown if there is one edit.
      if (Object.keys(currentEdits).length === 1) {
        document
          .getElementById("feedback-coordinate-picker-helptext")
          .classList.remove("d-none");
      }
    },
    cancelLocationPicker: function () {
      this.map.interactive.marker2.remove();
      this.map.interactive.marker2 = null;

      if (this.coord_picker.force_reopen) {
        this.coord_picker.force_reopen = false;
        this.openFeedbackForm();
      }
    },
    deletePendingCoordinates: function () {
      if (this.coord_counter.to_confirm_delete) {
        removeLocalStorage("coordinate-feedback");
        this.coord_counter.to_confirm_delete = false;
        this.coord_picker.body_backup = null;
        this.coord_picker.subject_backup = null;
        this.coord_picker.backup_id = null;
      } else {
        this.coord_counter.to_confirm_delete = true;
      }
    },
    loadRoomfinderMap: function (mapIndex, fromUi) {
      const map = this.state.data.maps.roomfinder.available[mapIndex];
      this.state.map.selected = selectedMap.roomfinder;
      this.state.map.roomfinder.selected_id = map.id;
      this.state.map.roomfinder.selected_index = mapIndex;

      // Using the #map-container since the bounding rect is still all zero
      // if we switched here from interactive map
      const rect = document
        .getElementById("map-container")
        .getBoundingClientRect();
      // -1023px, -1023px is top left corner, 16px = 2*8px is element padding
      this.state.map.roomfinder.x =
        -1023 + (map.x / map.width) * (rect.width - 16);

      // We cannot use "height" here as it might be still zero before layouting
      // finished, so we use the aspect ratio here.
      this.state.map.roomfinder.y =
        -1023 +
        (map.y / map.height) * (rect.width - 16) * (map.height / map.width);

      this.state.map.roomfinder.width = map.width;
      this.state.map.roomfinder.height = map.height;

      if (fromUi) {
        document.getElementById("map-accordion").checked = false;
        /* window.setTimeout(() => {
                    document.getElementById("roomfinder-map-img").scrollIntoView(false);
                }, 50); */
        window.scrollTo(
          0,
          rect.top + this.state.map.roomfinder.y + 1023 - window.innerHeight / 2
        );
      }
    },
    copyCurrentLink: copyCurrentLink(),
  },
  mounted: function () {
    this.is_mounted = true;
    if (navigator.userAgent === "Rendertron") {
      return;
    }

    // Update pending coordinate counter on localStorage changes
    const _this = this;
    const updateCoordinateCounter = function () {
      const coords = getLocalStorageWithExpiry("coordinate-feedback", {});
      _this.coord_counter.counter = Object.keys(coords).length;
    };
    window.addEventListener("storage", updateCoordinateCounter);
    updateCoordinateCounter();

    this.$nextTick(() => {
      // Even though 'mounted' is called there is no guarantee apparently,
      // that it really is mounted now. For this reason we try to poll now.
      // (Not the best solution probably)
      let timeoutInMs = 5;
      const __this = this;

      function pollMap() {
        if (document.getElementById("interactive-map") !== null) {
          __this.loadMap();
        } else {
          console.warn(
            `'mounted' called, but page doesn't appear to be mounted yet. Retrying to load the map in ${timeoutInMs}ms`
          );
          window.setTimeout(pollMap, timeoutInMs);
          timeoutInMs *= 1.5;
        }
      }

      pollMap();
    });
  },
};
</script>

<template>
  <div id="view-view" v-if="state.data">
    <!-- Header image (on mobile) -->
    <a
      class="show-sm header-image-mobile c-hand"
      @click="state.showImageShowcase(image.shown_image_id)"
      v-if="image.shown_image"
    >
      <img
        alt="Header-Image, showing the building"
        v-bind:src="'/cdn/header/' + image.shown_image.name"
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
          <button class="btn btn-primary btn-sm" @click="openFeedbackForm">
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
            @click="copyCurrentLink(copied)"
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
          <ShareButton v-bind:coords="state.data.coords"></ShareButton>
          <button
            class="btn btn-link btn-action btn-sm"
            v-bind:title="$t('view_view.header.feedback')"
            @click="openFeedbackForm"
          >
            <i class="icon icon-flag"></i>
          </button>
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

        <div
          class="toast toast-primary mb-2 location-picker"
          v-if="map.interactive.marker2"
        >
          <div class="columns">
            <div class="column col col-sm-12">
              {{ $t("view_view.msg.correct_location.msg") }}
            </div>
            <div class="column col-auto col-sm-12 btns">
              <button class="btn btn-sm" @click="cancelLocationPicker">
                {{ $t("view_view.msg.correct_location.btn-cancel") }}
              </button>
              <button class="btn btn-sm" @click="confirmLocationPicker">
                <i class="icon icon-check"></i>
                {{ $t("view_view.msg.correct_location.btn-done") }}
              </button>
            </div>
          </div>
        </div>

        <DetailsInteractiveMap></DetailsInteractiveMap>
        <div
          class="roomfinder-map-container"
          v-bind:class="{ 'd-none': state.map.selected !== 'roomfinder' }"
          v-if="
            state.data.maps.roomfinder && state.data.maps.roomfinder.available
          "
        >
          <img
            alt="Cross showing where the room is located on the hand-drawn roomfinder map image"
            src="@/assets/map/roomfinder_cross-v2.webp"
            v-bind:style="{
              transform:
                'translate(' +
                state.map.roomfinder.x +
                'px, ' +
                state.map.roomfinder.y +
                'px)',
            }"
            id="roomfinder-map-cross"
          />
          <img
            alt="Hand-drawn roomfinder map image"
            v-bind:src="
              '/cdn/maps/roomfinder/' +
              state.data.maps.roomfinder.available[
                state.map.roomfinder.selected_index
              ].file
            "
            class="img-responsive"
            v-bind:width="state.map.roomfinder.width"
            v-bind:height="state.map.roomfinder.height"
            id="roomfinder-map-img"
          />
          <div>
            {{ $t("view_view.map.img_source") }}:
            {{
              state.data.maps.roomfinder.available[
                state.map.roomfinder.selected_index
              ].source
            }}
          </div>
        </div>
        <div
          class="accordion"
          id="roomfinder-map-select"
          v-bind:class="{ 'd-none': state.map.selected !== 'roomfinder' }"
          v-if="
            state.data.maps.roomfinder && state.data.maps.roomfinder.available
          "
        >
          <input
            id="map-accordion"
            type="checkbox"
            name="accordion-checkbox"
            hidden
          />
          <label
            for="map-accordion"
            class="btn btn-sm btn-block accordion-header"
          >
            1:{{
              state.data.maps.roomfinder.available[
                state.map.roomfinder.selected_index
              ].scale
            }},
            {{
              state.data.maps.roomfinder.available[
                state.map.roomfinder.selected_index
              ].name
            }}
            <i class="icon icon-caret"></i>
          </label>
          <div
            class="accordion-body"
            v-if="state.data.maps && state.data.maps.roomfinder"
          >
            <ul class="menu menu-nav">
              <li
                class="menu-item"
                v-for="(m, i) in state.data.maps.roomfinder.available"
              >
                <button
                  class="btn btn-sm"
                  v-bind:aria-label="
                    `show the map '` + m.name + `' at the scale 1:` + m.scale
                  "
                  v-bind:class="{
                    selected: m.id === state.map.roomfinder.selected_id,
                  }"
                  v-on:click="loadRoomfinderMap(i, true)"
                >
                  1:{{ m.scale }}, {{ m.name }}
                </button>
              </li>
            </ul>
          </div>
        </div>
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
            v-bind:class="{ active: state.map.selected === 'roomfinder' }"
            v-bind:disabled="
              !(
                state.data.maps.roomfinder &&
                state.data.maps.roomfinder.available
              )
            "
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
    <section id="entry-sources">
      <div class="columns">
        <div class="column">
          <h2>{{ $t("view_view.sources.title") }}</h2>
        </div>
      </div>
      <p v-if="">
        {{ $t("view_view.sources.base.title") }}:
        <span v-for="(e, i) in state.data.sources.base">
          <a v-if="e.url" v-bind:href="e.url">{{ e.name }}</a>
          <template v-else>{{ e.name }}</template>
          <template v-if="i < state.data.sources.base.length - 1"
            >&#32;•&#32;</template
          >
        </span>
        <span v-if="state.data.sources.patched">
          <br />{{ $t("view_view.sources.base.patched") }}
        </span>
      </p>
      <p v-if="image.shown_image">
        {{ $t("view_view.sources.header_img") }}:
        <span>{{ image.shown_image.author.text }}</span>
        <span v-if="image.shown_image.source"
          >•
          <a
            v-if="image.shown_image.source.url"
            v-bind:href="image.shown_image.source.url"
            target="_blank"
          >
            {{ image.shown_image.source.text }}
          </a>
          <template v-else>{{ image.shown_image.source.text }}</template>
        </span>
        <span v-if="image.shown_image.license"
          >&#32;•
          <a
            v-if="image.shown_image.license.url"
            v-bind:href="image.shown_image.license.url"
            target="_blank"
          >
            {{ image.shown_image.license.text }}
          </a>
          <template v-else>{{ image.shown_image.license.text }}</template>
        </span>
      </p>
      <p v-if="state.data.coords">
        {{ $t("view_view.sources.coords.title") }}:
        <span v-if="state.data.coords.source === 'navigatum'">
          {{ $t("view_view.sources.coords.navigatum") }}</span
        >
        <span v-if="state.data.coords.source === 'roomfinder'">
          {{ $t("view_view.sources.coords.roomfinder") }}
        </span>
        <span v-if="state.data.coords.source === 'inferred'">
          {{ $t("view_view.sources.coords.inferred") }}
        </span>
      </p>
    </section>
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

  /* --- Map container --- */
  #map-container {
    // This does not change anything (except using px instead of rem),
    // but ensures that roomfinder position calculations are predictable.
    padding: 0 8px;

    // The marker2 (draggable)
    .mapboxgl-marker + .mapboxgl-marker {
      animation: fade-in 0.1s linear 0.05s;
      animation-fill-mode: both;
    }
  }

  .toast.location-picker {
    animation: fade-in 0.1s linear 0.05s;
    animation-fill-mode: both;

    & .btns {
      margin: auto 0;
    }
  }

  /* --- Maps --- */
  #map-container {
    .toast {
      // Mobile
      margin-bottom: 9px;
      font-size: 0.7rem;
    }
    /* --- Roomfinder display --- */
    .roomfinder-map-container {
      overflow: hidden;
      position: relative;
      margin-bottom: 6px;

      > div {
        // Image source label
        position: absolute;
        bottom: 1px;
        right: 1px;
        padding: 1px 5px;
        color: $body-font-color;
        background-color: $container-loading-bg;
        font-size: 10px;
      }
    }

    #roomfinder-map-cross {
      position: absolute;
      transition: transform 0.3s;
      pointer-events: none;
    }

    #roomfinder-map-img {
      width: 100%;
      display: block;
    }

    #roomfinder-map-select > label {
      padding: 0.05rem 0.3rem;
    }

    .accordion-body {
      ul,
      button,
      li {
        font-size: 12px;
      }

      .selected {
        background: $roomfinder-selected-bg;
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

  /* --- Sections --- */
  #entry-sources {
    h2 {
      margin-bottom: 16px;
    }

    p {
      margin-bottom: 6px;
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
