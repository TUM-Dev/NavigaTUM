<script setup lang="ts">
import type { Component } from "vue";
import LectureResultContent from "~/components/LectureResultContent.vue";
import LocationResultContent from "~/components/LocationResultContent.vue";
import type { SearchResultEntry } from "~/utils/lectureRow";

defineProps<{
  item: SearchResultEntry;
  highlighted: boolean;
}>();

// `Record<kind, …>` (not `satisfies`); see SearchResultRow.vue.
const CONTENT_BY_KIND: Record<SearchResultEntry["kind"], Component> = {
  location: LocationResultContent,
  lecture: LectureResultContent,
};
</script>

<template>
  <component :is="CONTENT_BY_KIND[item.kind]" :item="item" :highlighted="highlighted" />
</template>
