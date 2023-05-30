<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useDetailsStore } from "@/stores/details";
type RoomChild = { readonly id: string; readonly name: string };
const state = useDetailsStore();
const combined_list = computed(() => {
  if (!state.data) return [];

  const usages = state.data?.sections?.rooms_overview?.usages || [];
  const combinedList = [] as RoomChild[];
  usages.forEach((usage) => {
    combinedList.push(...usage.children);
  });
  return combinedList;
});

const display_list = ref([] as RoomChild[]);
const search = ref("");
const selected = ref(null as number | null);
const loading = ref(false);
const buildings_overview_expanded = ref(false);

watch(
  () => [search.value, selected.value],
  () => updateRoomsOverview()
);
function updateRoomsOverview() {
  console.log("updateRoomsOverview");
  const rooms_overview = state.data?.sections?.rooms_overview;
  if (!rooms_overview) return;

  if (selected.value === null) {
    display_list.value = [];
  } else {
    const rooms = selected.value === -1 ? combined_list.value : rooms_overview.usages[selected.value].children;
    if (search.value === "") {
      display_list.value = rooms;
    } else {
      const search_term = new RegExp(`.*${search.value}.*`, "i"); // i=>case insensitive
      const filtered = [] as RoomChild[];

      rooms.filter((f) => search_term.test(f.name)).forEach((f) => filtered.push(f));
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
  }
}
</script>

<template>
  <!-- Buildings overview -->
  <section v-if="state.data?.sections?.buildings_overview" id="building-overview">
    <div class="columns">
      <div class="column">
        <h2>{{ $t("view_view.buildings_overview.title") }}</h2>
      </div>
      <!--<div class="column col-auto">
          <a href="#">Übersichtskarte <i class="icon icon-forward" /></a>
        </div>-->
    </div>
    <div class="columns">
      <template v-for="(b, i) in state.data.sections.buildings_overview.entries" :key="b.id">
        <div
          class="column col-4 col-md-12 content"
          v-if="i < state.data.sections.buildings_overview.n_visible || buildings_overview_expanded"
        >
          <RouterLink :to="'/view/' + b.id">
            <div class="tile tile-centered">
              <div class="tile-icon">
                <figure class="avatar avatar-lg">
                  <img
                    v-if="b.thumb"
                    :alt="$t('view_view.buildings_overview.thumbnail_preview')"
                    :src="'/cdn/thumb/' + b.thumb"
                  />
                  <img
                    v-else
                    :alt="$t('view_view.buildings_overview.default_thumbnail_preview')"
                    src="../assets/thumb-building.webp"
                  />
                </figure>
              </div>
              <div class="tile-content">
                <p class="tile-title">{{ b.name }}</p>
                <small class="tile-subtitle text-dark">{{ b.subtext }}</small>
              </div>
              <div class="tile-action">
                <button class="btn btn-link" :aria-label="`show the details for the building '${b.name}'`">
                  <i class="icon icon-arrow-right" />
                </button>
              </div>
            </div>
          </RouterLink>
        </div>
      </template>
    </div>
    <div
      v-if="state.data.sections.buildings_overview.n_visible < state.data.sections.buildings_overview.entries.length"
    >
      <button class="btn btn-link" v-if="!buildings_overview_expanded" @click="buildings_overview_expanded = true">
        <i class="icon icon-arrow-right" />
        {{ $t("view_view.buildings_overview.more") }}
      </button>
      <button class="btn btn-link" v-if="buildings_overview_expanded" @click="buildings_overview_expanded = false">
        <i class="icon icon-arrow-up" />
        {{ $t("view_view.buildings_overview.less") }}
      </button>
    </div>
  </section>

  <!-- Rooms overview -->
  <section id="rooms-overview" v-if="state.data?.sections?.rooms_overview">
    <div class="columns">
      <div class="column">
        <h2>{{ $t("view_view.rooms_overview.title") }}</h2>
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

    <div class="columns content">
      <div class="column col-4 col-lg-5 col-md-6 col-sm-12" id="rooms-overview-select">
        <div class="panel">
          <div class="panel-header">
            <div class="panel-title h6">{{ $t("view_view.rooms_overview.by_usage") }}:</div>
          </div>
          <div class="panel-body">
            <ul class="menu">
              <li class="menu-item">
                <button
                  class="btn"
                  :class="{
                    active: selected === -1,
                  }"
                  @click="selected = -1"
                >
                  <i class="icon icon-arrow-right" />
                  <div class="menu-text">
                    {{ $t("view_view.rooms_overview.any") }}
                  </div>
                  <label class="label">{{ combined_list.length }}</label>
                </button>
              </li>
              <li class="divider" data-content="" />
              <li class="menu-item" v-for="(u, i) in state.data?.sections.rooms_overview.usages" :key="u.name">
                <button
                  class="btn"
                  :class="{
                    active: i === state.data.sections.rooms_overview.selected,
                  }"
                  @click="selected = i"
                >
                  <i class="icon icon-arrow-right" />
                  <div class="menu-text">{{ u.name }}</div>
                  <label class="label">{{ u.count }}</label>
                </button>
              </li>
            </ul>
          </div>
          <div class="panel-footer">
            <button class="btn btn-link btn-sm" @click="selected = null">
              {{ $t("view_view.rooms_overview.remove_selection") }}
            </button>
          </div>
        </div>
      </div>
      <div class="column col-8 col-lg-7 col-md-6 col-sm-12 hide-l" id="rooms-overview-list">
        <div class="show-sm" style="height: 15px" />
        <div class="panel">
          <div class="panel-header">
            <div class="input-group">
              <input v-model="search" :placeholder="$t('view_view.rooms_overview.filter')" class="form-input" />
              <button
                class="btn btn-primary input-group-btn"
                @click="search = ''"
                :aria-label="$t('view_view.rooms_overview.clear_filter')"
              >
                <i class="icon icon-cross" />
              </button>
            </div>
          </div>
          <div class="panel-body">
            <div :class="{ loading: loading }" />
            <ul class="menu" v-if="selected !== null">
              <li class="menu-item" v-for="r in display_list" :key="r.id">
                <RouterLink :to="'/view/' + r.id"><i class="icon icon-location" /> {{ r.name }}</RouterLink>
              </li>
            </ul>
          </div>
          <div class="panel-footer">
            <small>
              {{
                selected === null
                  ? $t("view_view.rooms_overview.choose_usage")
                  : display_list.length +
                    " " +
                    $t("view_view.rooms_overview.result") +
                    (display_list.length === 1 ? "" : $t("view_view.rooms_overview.results_suffix")) +
                    (search === "" ? "" : $t("view_view.rooms_overview.filtered"))
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
