<script setup lang="ts">
import type { Component } from "vue";
import AddressResultRow from "~/components/AddressResultRow.vue";
import LectureSearchResultRow from "~/components/LectureSearchResultRow.vue";
import LocationResultRow from "~/components/LocationResultRow.vue";
import type { SearchResultEntry } from "~/utils/lectureRow";

defineProps<{
  item: SearchResultEntry;
  highlighted: boolean;
}>();
defineEmits<{
  (e: "click"): void;
  (e: "mouseover"): void;
}>();

// Annotation, not `satisfies`: it both requires a row per kind (a new kind is a
// compile error) and widens the values to `Component`, so `<component :is>` does
// not intersect the variants' incompatible prop types.
const ROW_BY_KIND: Record<SearchResultEntry["kind"], Component> = {
  location: LocationResultRow,
  address: AddressResultRow,
  lecture: LectureSearchResultRow,
};
</script>

<template>
  <component
    :is="ROW_BY_KIND[item.kind]"
    :item="item"
    :highlighted="highlighted"
    @click="$emit('click')"
    @mouseover="$emit('mouseover')"
  />
</template>
