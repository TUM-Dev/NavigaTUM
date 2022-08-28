<script setup lang="ts">
import {
  getLocalStorageWithExpiry,
  removeLocalStorage,
  setLocalStorageWithExpiry,
} from "@/utils/storage";
import { copyCurrentLink, setDescription, setTitle } from "@/utils/common";
import ShareButton from "@/components/ShareButton.vue";
import { selectedMap, useDetailsStore } from "@/stores/details";

/* global mapboxgl */
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
  components: [ShareButton],
  data: function () {
    return {
      view_data: null,
      image: {
        shown_image: null,
        shown_image_id: null,
        slideshow_open: false,
      },
      map: {
        interactive: {
          map: null,
          component: null,
          marker: null,
          marker2: null,
        },
      },
      sections: {
        rooms_overview: {
          combined_count: 0,
          combined_list: [],
          display_list: [],
          _filter_index: {
            selected: null,
            list: [],
          },
          loading: false,
        },
      },
      // State is preserved when navigating in history.
      // May only contain serializable objects!
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
        // the set coordinate button in the feedback form. If we didn't
        // made a backup then, this would be lost after clicking confirm there.
        backup_id: null,
        subject_backup: null,
        body_backup: null,
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
    showImageShowcase: function (i, openSlideshow = true) {
      if (this.view_data && this.view_data.imgs && this.view_data.imgs[i]) {
        this.image.slideshow_open = openSlideshow;
        this.image.shown_image_id = i;
        this.image.shown_image = this.view_data.imgs[i];
      } else {
        this.image.slideshow_open = false;
        this.image.shown_image_id = null;
        this.image.shown_image = null;
      }
    },
    hideImageShowcase: function () {
      this.image.slideshow_open = false;
    },
    // This is called
    // - on initial page load
    // - when the view is loaded for the first time
    // - when the view is navigated to from a different view
    // - when the view is navigated to from the same view, but with a different entry
    loadEntryData: function (data) {
      this.view_data = data;

      this.showImageShowcase(0, false);

      if (data === null) return;

      // --- Maps ---
      // We need to reset state to default here, else it is preserved from the previous page
      this.state.reset();

      this.state.map.selected = data.maps.default;
      // Interactive has to be always available, but roomfinder may be unavailable
      if ("roomfinder" in data.maps) {
        // Find default map
        data.maps.roomfinder.available.forEach((availableMap, index) => {
          if (availableMap.id === data.maps.roomfinder.default) {
            this.state.map.roomfinder.selected_index = index;
            this.state.map.roomfinder.selected_id = availableMap.id;
          }
        });
      }

      // Maps can only be loaded after first mount because then the elements are
      // created and can be referenced by id.
      if (this.is_mounted) this.loadMap();

      // --- Additional data ---
      setTitle(data.name);
      setDescription(this.genDescription(data));

      // --- Sections ---
      if (this.view_data.sections && this.view_data.sections.rooms_overview) {
        const { usages } = this.view_data.sections.rooms_overview;
        const combinedList = [];
        usages.forEach((usage) => {
          combinedList.push(...usage.children);
        });
        this.sections.rooms_overview.combined_list = combinedList;
        this.sections.rooms_overview.combined_count = combinedList.length;
        this.updateRoomsOverview();
      }
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
    addLocationPicker: function () {
      // If this is called from the feedback form using the edit coordinate
      // button, we temporarily save the current subject and body, so it is
      // not lost when being reopened
      if (
        window.feedback &&
        document.getElementById("feedback-modal").classList.contains("active")
      ) {
        this.coord_picker.backup_id = this.view_data.id;
        this.coord_picker.subject_backup =
          document.getElementById("feedback-subject").value;
        this.coord_picker.body_backup =
          document.getElementById("feedback-body").value;
        this.coord_picker.force_reopen = true; // reopen after confirm

        window.feedback.closeForm();
      }

      this.state.map.selected = selectedMap.interactive;

      // Verify that there isn't already a marker (could happen if you click 'assign
      // a location' multiple times from the 'missing accurate location' toast)
      if (this.map.interactive.marker2 === null) {
        // Coordinates are either taken from the entry, or if there are already
        // some in the localStorage use them
        const currentEdits = getLocalStorageWithExpiry(
          "coordinate-feedback",
          {}
        );

        const { coords } = currentEdits[this.view_data.id] || this.view_data;
        const marker2 = new mapboxgl.Marker({
          draggable: true,
          color: "#ff0000",
        });
        marker2
          .setLngLat([coords.lon, coords.lat])
          .addTo(this.map.interactive.map);
        this.map.interactive.marker2 = marker2;
      }
    },
    _getFeedbackSubject: function (currentEdits) {
      if (Object.keys(currentEdits).length > 1) {
        return (
          `[${this.view_data.id} et.al.]: ` +
          $t("feedback.coordinatepicker.edit_coordinates_subject")
        );
      }

      const subjectPrefix = `[${this.view_data.id}]: `;
      const subjectMsg =
        Object.keys(currentEdits).length === 0
          ? ""
          : $t("feedback.coordinatepicker.edit_coordinate_subject");

      // The subject backup is only loaded (and supported) when a single
      // entry is being edited
      if (
        this.coord_picker.subject_backup &&
        this.coord_picker.backup_id === this.view_data.id &&
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
        this.coord_picker.backup_id === this.view_data.id
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
        this.view_data.coords.accuracy === "building"
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
      currentEdits[this.view_data.id] = {
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
    loadInteractiveMap: function (fromUi = false) {
      const _this = this;
      const fromMap = this.state.map.selected;

      this.state.map.selected = selectedMap.interactive;

      const doMapUpdate = function () {
        getModule("interactive-map").then((c) => {
          _this.map.interactive.component = c;

          let { map } = _this.map.interactive;
          let { marker } = _this.map.interactive;
          // The map might or might not be initialized depending on the type
          // of navigation.
          if (document.getElementById("interactive-map")) {
            if (
              document
                .getElementById("interactive-map")
                .classList.contains("mapboxgl-map")
            ) {
              marker.remove();
            } else {
              map = c.initMap("interactive-map");
              _this.map.interactive.map = map;

              document
                .getElementById("interactive-map")
                .classList.remove("loading");
            }
          }
          marker = new mapboxgl.Marker({ element: c.createMarker() });
          _this.map.interactive.marker = marker;
          const coords = _this.view_data.coords;
          marker.setLngLat([coords.lon, coords.lat]).addTo(map);

          if (_this.view_data.maps && _this.view_data.maps.overlays) {
            c.setFloorOverlays(
              _this.view_data.maps.overlays.available,
              _this.view_data.maps.overlays.default
            );
          } else {
            c.setFloorOverlays(null);
          }

          const defaultZooms = {
            joined_building: 16,
            building: 17,
            room: 18,
          };
          if (fromMap === selectedMap.interactive) {
            map.flyTo({
              center: [coords.lon, coords.lat],
              zoom: defaultZooms[_this.view_data.type]
                ? defaultZooms[_this.view_data.type]
                : 16,
              speed: 1,
              maxDuration: 2000,
            });
          } else {
            map.setZoom(16);
            map.setCenter([coords.lon, coords.lat]);
          }
        });
      };

      // The map element should be visible when initializing
      if (!document.querySelector("#interactive-map .mapboxgl-canvas"))
        this.$nextTick(doMapUpdate());
      else doMapUpdate();

      // To have an animation when the roomfinder is opened some time later,
      // the cursor is set to 'zero' while the interactive map is displayed.
      this.state.map.roomfinder.x = -1023 - 10;
      this.state.map.roomfinder.y = -1023 - 10;

      if (fromUi) {
        window.scrollTo(0, 0);
      }
    },
    loadRoomfinderMap: function (mapIndex, fromUi) {
      const map = this.view_data.maps.roomfinder.available[mapIndex];
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
    updateRoomsOverview: function (setSelected = undefined) {
      const state = this.state.rooms_overview;
      const data = this.view_data.sections.rooms_overview;
      const local = this.sections.rooms_overview;

      if (setSelected !== undefined) state.selected = setSelected;

      if (state.selected === null) {
        local.display_list = [];
      } else {
        const baseList =
          state.selected === -1
            ? local.combined_list
            : data.usages[state.selected].children;
        if (state.filter === "") {
          local.display_list = baseList;
        } else {
          // Update filter index if required
          if (state.selected !== local._filter_index.selected) {
            const rooms = baseList;
            local._filter_index.list = [];

            rooms.forEach((room) => {
              room._lower = room.name.toLowerCase();
              local._filter_index.list.push(room);
            });
            local._filter_index.selected = state.selected;
          }

          const filter = state.filter.toLowerCase();
          const filtered = [];

          local._filter_index.list.forEach((f) => {
            if (f._lower.indexOf(filter) >= 0) filtered.push(f);
          });
          local.display_list = filtered;
        }
      }

      // If there are a lot of rooms, updating the DOM takes a while.
      // In this case we first reset the list, show a loading indicator and
      // set the long list a short time later (So DOM can update and the indicator
      // is visible).
      if (local.display_list.length > 150) {
        local.loading = true;
        const tmp = local.display_list;
        local.display_list = [];
        // this.$nextTick doesn't work for some reason, the view freezes
        // before the loading indicator is visible.
        window.setTimeout(() => {
          local.display_list = tmp;
          local.loading = false;
        }, 20);
      }
    },
    copyCurrentLink: copyCurrentLink(),
  },
  watch: {
    "state.rooms_overview.filter": function () {
      this.updateRoomsOverview();
    },
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
  <div id="view-view" v-if="view_data">
    <!-- Header image (on mobile) -->
    <a
      class="show-sm header-image-mobile c-hand"
      @click="showImageShowcase(image.shown_image_id)"
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
            <span class="default">{{
              $t("view_view.msg.coordinate-counter.delete")
            }}</span>
            <span class="confirm">{{
              $t("view_view.msg.coordinate-counter.delete-confirm")
            }}</span>
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
        v-for="(p, i) in view_data.parent_names"
        property="itemListElement"
        typeof="ListItem"
      >
        <RouterLink
          v-bind="{ to: '/view/' + view_data.parents[i] }"
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
          {{ view_data.name
          }}<!-- <small class="label">Exaktes Ergebnis</small>-->
        </h1>
      </div>
      <div class="columns subtitle">
        <div class="column col-auto">
          <span>{{ view_data.type_common_name }}</span>
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
          <ShareButton v-bind:coords="view_data.coords"></ShareButton>
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
              view_data.coords.accuracy &&
              view_data.coords.accuracy === 'building'
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
              view_data.type == 'room' &&
              view_data.maps &&
              view_data.maps.overlays &&
              view_data.maps.overlays.default === null
            "
          >
            {{ $t("view_view.msg.no_floor_overlay") }}
          </div>
          <div class="toast" v-if="view_data.props && view_data.props.comment">
            {{ view_data.props.comment }}
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

        <div
          id="interactive-map-container"
          v-bind:class="{ 'd-none': state.map.selected !== 'interactive' }"
        >
          <div>
            <div id="interactive-map" class="loading"></div>
          </div>
        </div>
        <div
          class="roomfinder-map-container"
          v-bind:class="{ 'd-none': state.map.selected !== 'roomfinder' }"
          v-if="
            view_data.maps.roomfinder && view_data.maps.roomfinder.available
          "
        >
          <img
            alt="Cross showing where the room is located on the hand-drawn roomfinder map image"
            src="@/assets/roomfinder_cross-v2.webp"
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
              view_data.maps.roomfinder.available[
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
              view_data.maps.roomfinder.available[
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
            view_data.maps.roomfinder && view_data.maps.roomfinder.available
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
              view_data.maps.roomfinder.available[
                state.map.roomfinder.selected_index
              ].scale
            }},
            {{
              view_data.maps.roomfinder.available[
                state.map.roomfinder.selected_index
              ].name
            }}
            <i class="icon icon-caret"></i>
          </label>
          <div
            class="accordion-body"
            v-if="view_data.maps && view_data.maps.roomfinder"
          >
            <ul class="menu menu-nav">
              <li
                class="menu-item"
                v-for="(m, i) in view_data.maps.roomfinder.available"
              >
                <button
                  class="btn btn-sm"
                  v-bind:aria-label="
                    `show the map '` + m.name + `' at the scale 1:` + m.scale
                  "
                  v-bind:class="{
                    selected: m.id == state.map.roomfinder.selected_id,
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
                view_data.maps.roomfinder && view_data.maps.roomfinder.available
              )
            "
          >
            {{ $t("view_view.map.roomfinder") }}
          </button>
        </div>
        <div class="divider" style="margin-top: 10px"></div>
      </div>

      <!-- Information section (on mobile) -->
      <div
        class="column col-5 col-sm-12 show-sm mobile-info-section"
        v-if="view_data.props && view_data.props.computed"
      >
        <h2>Informationen</h2>
        <table class="info-table">
          <tbody>
            <tr v-for="prop in view_data.props.computed">
              <td>
                <strong>{{ prop.name }}</strong>
              </td>
              <td>{{ prop.text }}</td>
            </tr>
            <tr v-if="view_data.props.links">
              <td>
                <strong>{{ $t("view_view.info_table.links") }}</strong>
              </td>
              <td>
                <ul>
                  <li v-for="link in view_data.props.links">
                    <a v-bind:href="link.url">
                      {{ link.text }}
                    </a>
                  </li>
                </ul>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Informationen card (desktop) -->
      <!-- Some elements are currently duplicate, which is not optimal but should be okay
           as long as only little information is there -->
      <div class="column col-5 col-md-12 hide-sm">
        <div class="card">
          <a
            class="card-image c-hand"
            @click="showImageShowcase(image.shown_image_id)"
            v-if="image.shown_image"
          >
            <img
              alt="Header-Image, showing the building"
              v-bind:src="'/cdn/header/' + image.shown_image.name"
              class="img-responsive"
              width="100%"
            />
          </a>
          <div class="card-header">
            <div class="card-title h5">{{ $t("view_view.info_title") }}</div>
          </div>
          <div class="card-body">
            <table
              class="info-table"
              v-if="view_data.props && view_data.props.computed"
            >
              <tbody>
                <tr v-for="prop in view_data.props.computed">
                  <td>
                    <strong>{{ prop.name }}</strong>
                  </td>
                  <td>{{ prop.text }}</td>
                </tr>
                <tr v-if="view_data.props.links">
                  <td>
                    <strong>{{ $t("view_view.info_table.links") }}</strong>
                  </td>
                  <td>
                    <ul>
                      <li v-for="link in view_data.props.links">
                        <a v-bind:href="link.url">
                          {{ link.text }}
                        </a>
                      </li>
                    </ul>
                  </td>
                </tr>
              </tbody>
            </table>
            <span v-else>-</span>
            <div
              class="toast toast-warning"
              v-if="
                view_data.coords.accuracy &&
                view_data.coords.accuracy === 'building'
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
                view_data.type == 'room' &&
                view_data.maps &&
                view_data.maps.overlays &&
                view_data.maps.overlays.default === null
              "
            >
              {{ $t("view_view.msg.no_floor_overlay") }}
            </div>
            <div
              class="toast"
              v-if="view_data.props && view_data.props.comment"
            >
              {{ view_data.props.comment }}
            </div>
          </div>
          <!--<div class="card-footer">
              <button class="btn btn-link">Mehr Infos</button>
          </div>-->
        </div>
      </div>
      <div
        class="modal modal-lg active"
        id="modal-slideshow"
        v-if="image.slideshow_open"
      >
        <a
          class="modal-overlay"
          aria-label="Close"
          @click="hideImageShowcase"
        ></a>
        <div class="modal-container modal-fullheight">
          <div class="modal-header">
            <button
              class="btn btn-clear float-right"
              v-bind:aria-label="$t('view_view.slideshow.close')"
              @click="hideImageShowcase"
            ></button>
            <h5 class="modal-title">{{ $t("view_view.slideshow.header") }}</h5>
          </div>
          <div class="modal-body">
            <div class="content">
              <div class="carousel">
                <template v-for="(_, i) in view_data.imgs">
                  <input
                    v-if="i == image.shown_image_id"
                    v-bind:id="'slide-' + (i + 1)"
                    class="carousel-locator"
                    type="radio"
                    name="carousel-radio"
                    hidden=""
                    checked="checked"
                  />
                  <input
                    v-else
                    v-bind:id="'slide-' + (i + 1)"
                    class="carousel-locator"
                    type="radio"
                    name="carousel-radio"
                    hidden=""
                    @click="showImageShowcase(i)"
                  />
                </template>

                <div class="carousel-container">
                  <figure
                    v-for="(img, i) in view_data.imgs"
                    class="carousel-item"
                  >
                    <label
                      v-if="i != 0"
                      class="item-prev btn btn-action btn-lg"
                      v-bind:for="'slide-' + i"
                      @click="showImageShowcase(i - 1)"
                    >
                      <i class="icon icon-arrow-left"></i>
                    </label>
                    <label
                      v-if="i != view_data.imgs.length - 1"
                      class="item-next btn btn-action btn-lg"
                      v-bind:for="'slide-' + (i + 2)"
                      @click="showImageShowcase(i + 1)"
                    >
                      <i class="icon icon-arrow-right"></i>
                    </label>
                    <div itemscope itemtype="http://schema.org/ImageObject">
                      <img
                        itemprop="contentUrl"
                        v-bind:alt="$t('view_view.slideshow.image_alt')"
                        loading="lazy"
                        v-bind:src="'/cdn/lg/' + img.name"
                        v-bind:srcset="
                          '/cdn/sm/' +
                          img.name +
                          ' 1024w,' +
                          '/cdn/md/' +
                          img.name +
                          ' 1920w,' +
                          '/cdn/lg/' +
                          img.name +
                          ' 3860w'
                        "
                        sizes="100vw"
                        class="img-responsive rounded"
                      />
                      <span
                        class="d-none"
                        v-if="img.license.url"
                        itemprop="license"
                      >
                        {{ img.license.url }}</span
                      >
                      <span class="d-none" v-else itemprop="license">
                        img.license.text</span
                      >
                      <span
                        class="d-none"
                        v-if="img.license.url"
                        itemprop="author"
                      >
                        {{ img.author.url }}</span
                      >
                      <span class="d-none" v-else itemprop="author">
                        img.author.text</span
                      >
                    </div>
                  </figure>
                </div>
                <div class="carousel-nav">
                  <label
                    v-for="(_, i) in view_data.imgs"
                    class="nav-item text-hide c-hand"
                    v-bind:for="'slide-' + (i + 1)"
                    >{{ i + 1 }}</label
                  >
                </div>
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <div class="columns">
              <div class="column col-4 col-sm-6 col-md-6 text-left">
                <h6>{{ $t("view_view.slideshow.source") }}</h6>
                <a
                  v-if="image.shown_image.source.url"
                  v-bind:href="image.shown_image.source.url"
                  >{{ image.shown_image.source.text }}</a
                >
                <template v-else>{{ image.shown_image.source.text }}</template>
              </div>
              <div
                class="column col-4 col-sm-6 col-md-6 text-center text-md-right"
              >
                <h6>{{ $t("view_view.slideshow.author") }}</h6>
                <a
                  v-if="image.shown_image.author.url"
                  v-bind:href="image.shown_image.author.url"
                  >{{ image.shown_image.author.text }}</a
                >
                <template v-else>{{ image.shown_image.author.text }}</template>
              </div>
              <div
                class="column col-4 col-sm-12 col-md-12 text-md-center mt-md-3"
              >
                <h6>{{ $t("view_view.slideshow.license") }}</h6>
                <a
                  v-if="image.shown_image.license.url"
                  v-bind:href="image.shown_image.license.url"
                  >{{ image.shown_image.license.text }}</a
                >
                <template v-else>{{ image.shown_image.license.text }}</template>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- TMP
  <div v-if="view_data.sections && view_data.sections.featured">
  <div class="columns" style="margin-top: 40px">
      <div class="column"><h2>Featured</h2></div>
  </div>
  <div style="position: relative;overflow: hidden;white-space: nowrap;">
      <div style="position: absolute;height: 100%;display: flex;background: linear-gradient(-90deg, #fff0 0%, #fffd 100%);">
          <button class="btn btn-action s-circle" style="margin: auto 0;/*! position: absolute; */"><i class="icon icon-arrow-left"></i></button>
      </div>
      <div class="card" style="width: 250px;display: inline-flex;margin-right: 8px;">
          <div class="card-image">
              <img alt="Header-Image, showing the building"
                   src="/cdn/header/mi_0.webp"
                   class="img-responsive"/>
          </div>
          <div class="card-header">
              <div class="card-title h6" style="text-overflow: ellipsis;overflow: hidden;white-space: nowrap;">Teilbibliothek Stammgelände</div>
              <small class="card-subtitle text-gray">Teilbibliothek</small>-->
    <!--<div class="divider"></div>
</div>
<div class="card-body">

</div>
<div class="card-footer">
    <button class="btn btn-primary">Mehr Infos</button>
</div>
</div>
<div class="card" style="width: 250px;display: inline-flex;margin-right: 8px;height: 200px;vertical-align: top;">
<div class="card-image" style="display: none;">
    <img alt="Header-Image, showing the building"
         src="/cdn/header/mi_0.webp"
         class="img-responsive">
</div>
<div class="card-header">
    <div class="card-title h6" style="text-overflow: ellipsis; overflow: hidden; white-space: nowrap;">Validierungsautomaten</div>
    <small class="card-subtitle text-gray" style="display: none;">Teilbibliothek</small>
</div>
<div class="card-body" style="overflow-y: auto;/*! position: relative; *//*! overflow-x: hidden; */">
    <div class="tile tile-centered">
        <div class="tile-icon">
            <div class="example-tile-icon">
                <i class="icon icon-location centered"></i>
            </div>
        </div>
        <div class="tile-content">
            <div class="tile-title">
                <a href="#/view/mi" class="btn btn-link">Süd-Seite</a>
            </div>
        </div>
        <div class="tile-action">
            <button class="btn btn-link"><i class="icon icon-arrow-right"></i></button>
        </div>
    </div>
</div>
</div>
<div style="position: absolute;height: 100%;display: flex;background: linear-gradient(90deg, #fff0 0%, #fffd 100%);right: 0;top: 0;">
<button class="btn btn-action s-circle" style="margin: auto 0;/*! position: absolute; */"><i class="icon icon-arrow-right"></i></button>
</div>
</div>
</div>-->

    <!-- Buildings overview -->
    <section
      v-if="view_data.sections && view_data.sections.buildings_overview"
      id="building-overview"
    >
      <div class="columns">
        <div class="column">
          <h2>{{ $t("view_view.buildings_overview.title") }}</h2>
        </div>
        <!--<div class="column col-auto">
          <a href="#">Übersichtskarte <i class="icon icon-forward"></i></a>
        </div>-->
      </div>
      <div class="columns">
        <div
          class="column col-4 col-md-12 content"
          v-for="(b, i) in view_data.sections.buildings_overview.entries"
          v-if="
            i < view_data.sections.buildings_overview.n_visible ||
            state.buildings_overview.expanded
          "
        >
          <RouterLink v-bind:to="'/view/' + b.id">
            <div class="tile tile-centered">
              <div class="tile-icon">
                <figure class="avatar avatar-lg">
                  <img
                    v-bind:alt="
                      b.thumb
                        ? 'Thumbnail, showing a preview of the building.'
                        : 'Default-thumbnail, as no thumbnail is available'
                    "
                    v-bind:src="
                      b.thumb
                        ? '/cdn/thumb/' + b.thumb
                        : '@/assets/thumb-building.webp'
                    "
                  />
                </figure>
              </div>
              <div class="tile-content">
                <p class="tile-title">{{ b.name }}</p>
                <small class="tile-subtitle text-dark">{{ b.subtext }}</small>
              </div>
              <div class="tile-action">
                <button
                  class="btn btn-link"
                  v-bind:aria-label="
                    `show the details for the building '` + b.name + `'`
                  "
                >
                  <i class="icon icon-arrow-right"></i>
                </button>
              </div>
            </div>
          </RouterLink>
        </div>
      </div>
      <div
        v-if="
          view_data.sections.buildings_overview.n_visible <
          view_data.sections.buildings_overview.entries.length
        "
      >
        <button
          class="btn btn-link"
          v-if="!state.buildings_overview.expanded"
          v-on:click="state.buildings_overview.expanded = true"
        >
          <i class="icon icon-arrow-right"></i>
          {{ $t("view_view.buildings_overview.more") }}
        </button>
        <button
          class="btn btn-link"
          v-if="state.buildings_overview.expanded"
          v-on:click="state.buildings_overview.expanded = false"
        >
          <i class="icon icon-arrow-up"></i>
          {{ $t("view_view.buildings_overview.less") }}
        </button>
      </div>
    </section>

    <!-- Rooms overview -->
    <section
      id="rooms-overview"
      v-if="view_data.sections && view_data.sections.rooms_overview"
    >
      <div class="columns">
        <div class="column">
          <h2>{{ $t("view_view.rooms_overview.title") }}</h2>
        </div>
        <!--<div class="column col-auto">
          <div class="dropdown"><a class="btn btn-link dropdown-toggle" tabindex="0">{{ $t("view_view.rooms_overview.by_usage") }} <i class="icon icon-caret"></i></a>
            <ul class="menu">
                    <li class="menu-item"><a href="#dropdowns">nach Nutzung</a></li>
                    <li class="menu-item"><a href="#dropdowns">nach ...</a></li>
            </ul>
          </div>
        </div>-->
      </div>

      <div class="columns content">
        <div
          class="column col-4 col-lg-5 col-md-6 col-sm-12"
          id="rooms-overview-select"
        >
          <div class="panel">
            <div class="panel-header">
              <div class="panel-title h6">
                {{ $t("view_view.rooms_overview.by_usage") }}:
              </div>
            </div>
            <div class="panel-body">
              <ul class="menu">
                <li class="menu-item">
                  <button
                    class="btn"
                    v-bind:class="{
                      active: state.rooms_overview.selected === -1,
                    }"
                    v-on:click="updateRoomsOverview(-1)"
                  >
                    <i class="icon icon-arrow-right"></i>
                    <div class="menu-text">
                      {{ $t("view_view.rooms_overview.any") }}
                    </div>
                    <label class="label">{{
                      sections.rooms_overview.combined_count
                    }}</label>
                  </button>
                </li>
                <li class="divider" data-content=""></li>
                <li
                  class="menu-item"
                  v-for="(u, i) in view_data.sections.rooms_overview.usages"
                >
                  <button
                    class="btn"
                    v-bind:class="{
                      active: i === state.rooms_overview.selected,
                    }"
                    v-on:click="updateRoomsOverview(i)"
                  >
                    <i class="icon icon-arrow-right"></i>
                    <div class="menu-text">{{ u.name }}</div>
                    <label class="label">{{ u.count }}</label>
                  </button>
                </li>
              </ul>
            </div>
            <div class="panel-footer">
              <button
                class="btn btn-link btn-sm"
                v-on:click="updateRoomsOverview(null)"
              >
                {{ $t("view_view.rooms_overview.remove_selection") }}
              </button>
            </div>
          </div>
        </div>
        <div
          class="column col-8 col-lg-7 col-md-6 col-sm-12 hide-l"
          id="rooms-overview-list"
        >
          <div class="show-sm" style="height: 15px"></div>
          <div class="panel">
            <div class="panel-header">
              <div class="input-group">
                <input
                  v-model="state.rooms_overview.filter"
                  v-bind:placeholder="$t('view_view.rooms_overview.filter')"
                  class="form-input"
                />
                <button
                  class="btn btn-primary input-group-btn"
                  @click="state.rooms_overview.filter = ''"
                  aria-label="Clear the filter"
                >
                  <i class="icon icon-cross"></i>
                </button>
              </div>
            </div>
            <div class="panel-body">
              <div
                v-bind:class="{ loading: sections.rooms_overview.loading }"
              ></div>
              <ul class="menu" v-if="state.rooms_overview.selected !== null">
                <li
                  class="menu-item"
                  v-for="r in sections.rooms_overview.display_list"
                >
                  <RouterLink v-bind:to="'/view/' + r.id">
                    <i class="icon icon-location"></i> {{ r.name }}
                  </RouterLink>
                </li>
              </ul>
            </div>
            <div class="panel-footer">
              <small>
                {{
                  state.rooms_overview.selected === null
                    ? $t("view_view.rooms_overview.choose_usage")
                    : sections.rooms_overview.display_list.length +
                      $t("view_view.rooms_overview.result") +
                      (sections.rooms_overview.display_list.length === 1
                        ? ""
                        : $t("view_view.rooms_overview.results_suffix")) +
                      (state.rooms_overview.filter === ""
                        ? ""
                        : "(" + $t("view_view.rooms_overview.filtered") + ")")
                }}
              </small>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section id="entry-sources">
      <div class="columns">
        <div class="column">
          <h2>{{ $t("view_view.sources.title") }}</h2>
        </div>
      </div>
      <p v-if="">
        {{ $t("view_view.sources.base.title") }}:
        <span v-for="(e, i) in view_data.sources.base">
          <a v-if="e.url" v-bind:href="e.url">{{ e.name }}</a>
          <template v-else>{{ e.name }}</template>
          <template v-if="i < view_data.sources.base.length - 1"
            >&#32;•&#32;</template
          >
        </span>
        <span v-if="view_data.sources.patched">
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
      <p v-if="view_data.coords">
        {{ $t("view_view.sources.coords.title") }}:
        <span v-if="view_data.coords.source == 'navigatum'">{{
          $t("view_view.sources.coords.navigatum")
        }}</span>
        <span v-if="view_data.coords.source == 'roomfinder'">{{
          $t("view_view.sources.coords.roomfinder")
        }}</span>
        <span v-if="view_data.coords.source == 'inferred'">{{
          $t("view_view.sources.coords.inferred")
        }}</span>
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

  /* --- Interactive map display --- */
  #interactive-map-container {
    margin-bottom: 10px;
    aspect-ratio: 4 / 3; // Not yet supported by all browsers

    > div {
      padding-bottom: 75%; // 4:3 aspect ratio
      border: 1px solid $border-light;
      background-color: $container-loading-bg;
      position: relative;
    }

    &.maximize {
      position: absolute;
      top: -10px;
      left: 0;
      width: 100%;
      height: calc(100vh - 60px);
      z-index: 1000;

      > div {
        padding-bottom: 0;
        height: 100%;
      }
    }
  }

  #interactive-map {
    position: absolute;
    height: 100%;
    width: 100%;
  }

  .marker {
    position: absolute;
    pointer-events: none;
    padding: 0;
  }

  .mapboxgl-ctrl-group.floor-ctrl {
    max-width: 100%;
    display: none;
    overflow: hidden;

    &.visible {
      display: block;
    }

    &.closed #floor-list {
      display: none !important;
    }

    & button {
      &.active {
        background: #ececec;
      }

      & .arrow {
        font-weight: normal;
        font-size: 0.3rem;
        line-height: 0.9rem;
        vertical-align: top;
      }
    }

    &.reduced > .vertical-oc,
    &.reduced > .horizontal-oc {
      display: none !important;
    }

    & > .vertical-oc,
    & > .horizontal-oc {
      font-weight: bold;
      background: #ececec;
    }

    &.closed {
      & > .vertical-oc,
      & > .horizontal-oc {
        background: #fff;
      }

      &:hover > .vertical-oc,
      &:hover > .horizontal-oc {
        background: #f2f2f2;
      }
    }

    // vertical is default layout
    & > .horizontal-oc {
      display: none;
    }

    &.horizontal {
      & > .horizontal-oc {
        display: inline-block;
      }

      & > .vertical-oc {
        display: none;
      }

      & #floor-list {
        display: inline-block;
        width: calc(100% - 29px);
      }

      & button {
        display: inline-block;
        border-top: 0;
        border-left: 1px solid #ddd;

        &.arrow {
          font-size: 0.4rem;
          vertical-align: bottom;
          line-height: 1.1rem;
        }

        & + button {
          border-top: 0;
        }
      }
    }

    // mapbox logo
    & + .mapboxgl-ctrl {
      opacity: 0.4;
      pointer-events: none;
      z-index: -1;
    }
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

  /* --- Information Section (mobile) --- */
  .mobile-info-section {
    margin-top: 15px;

    & > .info-table {
      margin-top: 16px;
    }
  }

  /* --- Information Card (desktop) --- */
  .card-body .toast {
    margin-top: 12px;
  }

  #map-container .toast {
    // Mobile
    margin-bottom: 9px;
    font-size: 0.7rem;
  }

  /* --- Info table --- */
  .info-table {
    width: 100%;
    border-collapse: collapse;

    td {
      vertical-align: top;
      padding: 4px 0;

      &:last-child {
        padding-left: 10px;
      }
    }

    tr {
      border-bottom: 1px solid $border-light;

      &:last-child {
        border-bottom: 0;
      }
    }

    ul {
      list-style-type: none;
      margin: 0;
    }

    li {
      margin: 0 0 0.4rem;

      &:last-child {
        margin: 0;
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
  #building-overview {
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
  }

  .menu {
    padding: 0;
    box-shadow: none;

    .menu-item button {
      text-align: left !important;
      border: 0 transparent !important;
      width: 100%;
    }

    .menu-item a,
    .menu-item label,
    .menu-item button {
      cursor: pointer;
      user-select: none;
    }
  }

  #rooms-overview {
    #rooms-overview-select .menu-item {
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
      padding-bottom: 4px;

      .divider {
        margin: 6px 0;
      }
    }

    .panel-footer {
      color: $text-gray;
    }
  }

  #rooms-overview-select .panel-body {
    max-height: 500px + 8px;
  }

  #rooms-overview-list .panel-body {
    max-height: 500px;
  }

  #entry-sources {
    h2 {
      margin-bottom: 16px;
    }

    p {
      margin-bottom: 6px;
    }
  }

  /* --- Image slideshow / showcase --- */
  #modal-slideshow {
    align-items: baseline;

    & .modal-container {
      position: relative;
      top: 5em;

      & .carousel-item {
        // Disable the animation of Spectre, because it appears a bit irritating.
        // It always run if we open the image slideshow and is wrong if we go back
        // in the slideshow.
        animation: none;
        transform: translateX(0);
      }
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
    // The mapbox logo is taking away space from the layer
    // selection on the bottom left on mobile, so we move
    // it a bit
    .floor-ctrl.visible + .mapboxgl-ctrl {
      position: absolute;
      bottom: 2px;
      left: 42px;
    }

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
