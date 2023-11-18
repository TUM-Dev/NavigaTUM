<script setup lang="ts">
import { computed, ref } from "vue";
import { useVirtualList } from "@vueuse/core";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
type RoomsOverview = components["schemas"]["RoomsOverview"];
type ChildEntry = components["schemas"]["ChildEntry"];

const props = defineProps<{
  readonly rooms?: RoomsOverview;
}>();

const { t } = useI18n({ useScope: "local" });
const search = ref("");
const selected = ref(-1);

const combined_list = computed(() => {
  const usages = props.rooms?.usages || [];
  const combinedList = [] as ChildEntry[];
  usages.forEach((usage) => {
    combinedList.push(...usage.children);
  });
  return combinedList;
});
const selectedRooms = computed<readonly ChildEntry[]>(() => {
  if (selected.value === -1) {
    return combined_list.value;
  }
  const rooms_usgage = props.rooms?.usages || [];
  return rooms_usgage[selected.value].children;
});
const filteredList = computed<readonly ChildEntry[]>(() => {
  const search_term = new RegExp(`.*${search.value}.*`, "i"); // i=>case insensitive
  return selectedRooms.value.filter((f) => search_term.test(f.name));
});
// useVirtualList does not work with readonly arrays
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore-next-line
const { list, containerProps, wrapperProps } = useVirtualList<ChildEntry>(filteredList, {
  itemHeight: 32,
  overscan: 10,
});
</script>

<template>
  <section v-if="props.rooms">
    <div class="columns">
      <div class="column">
        <h2>{{ t("title") }}</h2>
      </div>
      <!-- <div class="column col-auto">
          <div class="dropdown"><a class="btn btn-link dropdown-toggle" tabindex="0">{{ t("by_usage") }} <i class="icon icon-caret" /></a>
            <ul class="menu">
                    <li class="menu-item"><a href="#dropdowns">nach Nutzung</a></li>
                    <li class="menu-item"><a href="#dropdowns">nach ...</a></li>
            </ul>
          </div>
        </div> -->
    </div>

    <div class="columns content">
      <div id="category-select" class="col-4 col-md-12 col-sm-12 column">
        <div class="panel">
          <div class="panel-header">
            <div class="h6 panel-title">{{ t("by_usage") }}:</div>
          </div>
          <div class="panel-body">
            <ul class="menu">
              <li class="menu-item">
                <button
                  type="button"
                  class="btn"
                  :class="{
                    active: selected === -1,
                  }"
                  @click="selected = -1"
                >
                  <i class="icon icon-arrow-right" />
                  <div class="menu-text">
                    {{ t("any") }}
                  </div>
                  <label class="label">{{ combined_list.length }}</label>
                </button>
              </li>
              <li class="divider" data-content="" />
              <li v-for="(u, i) in props.rooms.usages" :key="u.name" class="menu-item">
                <button
                  type="button"
                  class="btn"
                  :class="{
                    active: i === selected,
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
            <button type="button" class="btn btn-link btn-sm" @click="selected = -1">
              {{ t("remove_selection") }}
            </button>
          </div>
        </div>
      </div>
      <div id="rooms-list" class="col-8 col-md-12 col-sm-12 column">
        <div class="show-sm" style="height: 15px" />
        <div class="panel">
          <div class="panel-header">
            <div class="input-group">
              <input v-model="search" :placeholder="t('filter')" class="form-input" />
              <button
                type="button"
                class="btn btn-primary input-group-btn"
                :aria-label="t('clear_filter')"
                @click="search = ''"
              >
                <i class="icon icon-cross" />
              </button>
            </div>
          </div>
          <div v-bind="containerProps" class="panel-body">
            <ul v-bind="wrapperProps" class="menu">
              <li v-for="item in list" :key="item.index" class="menu-item">
                <RouterLink :to="'/view/' + item.data.id" class="text-ellipsis">
                  <i class="icon icon-location" /> {{ item.data.name }}
                </RouterLink>
              </li>
            </ul>
          </div>
          <div class="panel-footer">
            <small>
              {{ t("results", filteredList.length) }}
              {{ search === "" ? "" : `(${t("filtered")})` }}
            </small>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss" scoped>
@import "@/assets/variables";
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

<i18n lang="yaml">
de:
  any: beliebig
  by_usage: nach Nutzung
  clear_filter: Filter löschen
  filter: Filter
  filtered: gefiltert
  remove_selection: Auswahl löschen
  results: 1 Ergebnis | {count} Ergebnisse
  title: Räume
en:
  any: any
  by_usage: by usage
  clear_filter: Clear the filter
  filter: Filter
  filtered: filtered
  remove_selection: Remove selection
  results: "{count} result | {count} results"
  title: Rooms
</i18n>
