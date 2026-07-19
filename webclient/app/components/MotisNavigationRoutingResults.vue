<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];

const pageCursor = defineModel<string | undefined>("pageCursor", { required: true });

const props = defineProps<{
  data: MotisRoutingResponse;
}>();
const emit = defineEmits<{
  selectItinerary: [itineraryIndex: number];
  selectLeg: [itineraryIndex: number, legIndex: number];
  selectStep: [itineraryIndex: number, legIndex: number, stepIndex: number];
}>();

const { t } = useI18n({ useScope: "local" });

const hasResults = computed(
  () => props.data.direct.length > 0 || props.data.itineraries.length > 0
);

// `0..` are transit itineraries, `-1 - i` the i-th direct connection. `null` collapses all.
const expandedIndex = ref<number | null>(null);
watch(
  () => props.data,
  () => {
    expandedIndex.value = null;
  }
);

function toggleConnection(itineraryIndex: number) {
  if (expandedIndex.value === itineraryIndex) {
    expandedIndex.value = null;
    return;
  }
  expandedIndex.value = itineraryIndex;
  emit("selectItinerary", itineraryIndex);
}
</script>

<template>
  <div class="w-full max-w-full">
    <MotisNoRoutesFound v-if="!hasResults" />
    <template v-else>
      <div v-if="data.direct.length > 0" class="mb-6">
        <h3 class="text-zinc-700 dark:text-zinc-200 mb-3 text-lg font-semibold">
          {{ t("direct_connections") }}
        </h3>
        <div class="space-y-2">
          <MotisConnectionCard
            v-for="(connection, i) in data.direct"
            :key="`direct-${i}`"
            :itinerary="connection"
            :itinerary-index="-1 - i"
            :expanded="expandedIndex === -1 - i"
            @toggle="toggleConnection(-1 - i)"
            @select-leg="(legIndex) => emit('selectLeg', -1 - i, legIndex)"
            @select-step="(legIndex, stepIndex) => emit('selectStep', -1 - i, legIndex, stepIndex)"
          />
        </div>
      </div>

      <div v-if="data.itineraries.length > 0">
        <div class="space-y-2">
          <MotisConnectionCard
            v-for="(itinerary, i) in data.itineraries"
            :key="`itinerary-${i}`"
            :itinerary="itinerary"
            :itinerary-index="i"
            :expanded="expandedIndex === i"
            @toggle="toggleConnection(i)"
            @select-leg="(legIndex) => emit('selectLeg', i, legIndex)"
            @select-step="(legIndex, stepIndex) => emit('selectStep', i, legIndex, stepIndex)"
          />
        </div>

        <div class="mt-4">
          <MotisPaginationControls
            :previous-page-cursor="data.previous_page_cursor"
            :next-page-cursor="data.next_page_cursor"
            size="lg"
            v-model:page-cursor="pageCursor"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<i18n lang="yaml">
de:
  direct_connections: Direkte Verbindungen
en:
  direct_connections: Direct connections
</i18n>
