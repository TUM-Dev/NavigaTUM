<script setup lang="ts">
import { computed, ref } from "vue";
import { useVirtualList } from "@vueuse/core";

type RoomChild = { readonly id: string; readonly name: string };
type Rooms = {
  readonly usages?: readonly {
    /**
     * @description Category Name
     * @example Büro
     */
    readonly name: string;
    /**
     * @description How many children this category has
     * @example 126
     */
    readonly count: number;
    readonly children: readonly RoomChild[];
  }[];
};

const props = defineProps<{
  readonly rooms?: Rooms;
}>();

const combined_list = computed(() => {
  const usages = props.rooms?.usages || [];
  const combinedList = [] as RoomChild[];
  usages.forEach((usage) => {
    combinedList.push(...usage.children);
  });
  return combinedList;
});

const search = ref("");
const selected = ref(-1);

const selectedRooms = computed<RoomChild[]>(() => {
  if (selected.value === -1) {
    return combined_list.value;
  }
  const rooms_usgage = props.rooms?.usages || [];
  return rooms_usgage[selected.value].children;
});
const filteredList = computed<RoomChild[]>(() => {
  const search_term = new RegExp(`.*${search.value}.*`, "i"); // i=>case insensitive
  return selectedRooms.value.filter((f) => search_term.test(f.name));
});
const { list, containerProps, wrapperProps } = useVirtualList(filteredList, { itemHeight: 32, overscan: 10 });
</script>

<template>
  <section v-if="props.rooms">
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
      <div class="column col-4 col-md-12 col-sm-12" id="category-select">
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
              <li class="menu-item" v-for="(u, i) in props.rooms.usages" :key="u.name">
                <button
                  class="btn"
                  :class="{
                    active: i === props.rooms.selected,
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
            <button class="btn btn-link btn-sm" @click="selected = -1">
              {{ $t("view_view.rooms_overview.remove_selection") }}
            </button>
          </div>
        </div>
      </div>
      <div class="column col-8 col-md-12 col-sm-12" id="rooms-list">
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
          <div v-bind="containerProps" class="panel-body">
            <ul v-bind="wrapperProps" class="menu">
              <li class="menu-item" v-for="item in list" :key="item.index">
                <RouterLink :to="'/view/' + item.data.id" class="text-ellipsis">
                  <i class="icon icon-location" /> {{ item.data.name }}
                </RouterLink>
              </li>
            </ul>
          </div>
          <div class="panel-footer">
            <small>
              {{ $t("view_view.rooms_overview.results", filteredList.length) }}
              {{ search === "" ? "" : `(${$t("view_view.rooms_overview.filtered")})` }}
            </small>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss" scoped>
@import "../assets/variables";
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
