<script setup lang="ts">
import { ref, reactive, computed, watch } from "vue";
import { useDetailsStore } from "@/stores/details";

const state = useDetailsStore();
const combined_list = computed(() => {
  if (!state.data) {
    loading.value = true;
    return [];
  }
  const usages = state.data.sections.rooms_overview.usages;
  const combinedList = [];
  usages.forEach((usage) => {
    combinedList.push(...usage.children);
  });
  return combinedList;
});
const display_list = ref([]);
const filter = reactive({
  search: "",
  selected: null,
  list: [],
});
const selected = ref(null);
const loading = ref(true);
const buildings_overview_expanded = ref(false);

watch(
  () => filter.search,
  (_) => updateRoomsOverview
);
function updateRoomsOverview(setSelected = undefined) {
  const rooms_overview = state.data?.sections.rooms_overview;

  if (setSelected !== undefined) selected.value = setSelected;

  if (selected.value === null) {
    display_list.value = [];
  } else {
    const baseList =
      selected.value === -1
        ? combined_list
        : rooms_overview.usages[selected.value].children;
    if (filter.search === "") {
      display_list.value = baseList;
    } else {
      // Update filter index if required
      if (selected.value !== filter.selected) {
        const rooms = baseList;
        filter.list = [];

        rooms.forEach((room) => {
          room._lower = room.name.toLowerCase();
          filter.list.push(room);
        });
        filter.selected = selected.value;
      }

      const search_term = filter.search.toLowerCase();
      const filtered = [];

      filter.list.forEach((f) => {
        if (f._lower.indexOf(search_term) >= 0) filtered.push(f);
      });
      display_list.value = filtered;
    }
  }

  // If there are a lot of rooms, updating the DOM takes a while.
  // In this case we first reset the list, show a loading indicator and
  // set the long list a short time later (So DOM can update and the indicator
  // is visible).
  if (display_list.value.length > 150) {
    loading.value = true;
    const tmp = display_list.value;
    display_list.value = [];
    // this.$nextTick doesn't work for some reason, the view freezes
    // before the loading indicator is visible.
    window.setTimeout(() => {
      display_list.value = tmp;
      loading.value = false;
    }, 20);
  } else loading.value = false;
}
</script>

<template>
  <!-- Buildings overview -->
  <section
    v-if="state.data.sections?.buildings_overview"
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
        v-for="(b, i) in state.data.sections.buildings_overview.entries"
        v-if="
          i < state.data.sections.buildings_overview.n_visible ||
          buildings_overview_expanded
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
                      : '/thumb-building.webp'
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
        state.data.sections.buildings_overview.n_visible <
        state.data.sections.buildings_overview.entries.length
      "
    >
      <button
        class="btn btn-link"
        v-if="!buildings_overview_expanded"
        v-on:click="buildings_overview_expanded = true"
      >
        <i class="icon icon-arrow-right"></i>
        {{ $t("view_view.buildings_overview.more") }}
      </button>
      <button
        class="btn btn-link"
        v-if="buildings_overview_expanded"
        v-on:click="buildings_overview_expanded = false"
      >
        <i class="icon icon-arrow-up"></i>
        {{ $t("view_view.buildings_overview.less") }}
      </button>
    </div>
  </section>

  <!-- Rooms overview -->
  <section
    id="rooms-overview"
    v-if="state.data.sections && state.data.sections.rooms_overview"
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
                    active: selected === -1,
                  }"
                  v-on:click="updateRoomsOverview(-1)"
                >
                  <i class="icon icon-arrow-right"></i>
                  <div class="menu-text">
                    {{ $t("view_view.rooms_overview.any") }}
                  </div>
                  <label class="label">{{ combined_list.length }}</label>
                </button>
              </li>
              <li class="divider" data-content=""></li>
              <li
                class="menu-item"
                v-for="(u, i) in state.data.sections.rooms_overview.usages"
              >
                <button
                  class="btn"
                  v-bind:class="{
                    active: i === state.data.sections.rooms_overview.selected,
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
                v-model="filter.search"
                v-bind:placeholder="$t('view_view.rooms_overview.filter')"
                class="form-input"
              />
              <button
                class="btn btn-primary input-group-btn"
                @click="filter.search = ''"
                aria-label="Clear the filter"
              >
                <i class="icon icon-cross"></i>
              </button>
            </div>
          </div>
          <div class="panel-body">
            <div v-bind:class="{ loading: loading }"></div>
            <ul class="menu" v-if="selected !== null">
              <li class="menu-item" v-for="r in display_list">
                <RouterLink v-bind:to="'/view/' + r.id">
                  <i class="icon icon-location"></i> {{ r.name }}
                </RouterLink>
              </li>
            </ul>
          </div>
          <div class="panel-footer">
            <small>
              {{
                selected === null
                  ? $t("view_view.rooms_overview.choose_usage")
                  : display_list.length +
                    $t("view_view.rooms_overview.result") +
                    (display_list.length === 1
                      ? ""
                      : $t("view_view.rooms_overview.results_suffix")) +
                    (filter.search === ""
                      ? ""
                      : "(" + $t("view_view.rooms_overview.filtered") + ")")
              }}
            </small>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss">
@import "../assets/variables";
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
</style>
