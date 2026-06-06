<script setup lang="ts">
import { mdiBookOpenPageVariantOutline, mdiOpenInNew } from "@mdi/js";

const props = defineProps<{
  // Absent (rather than `false`) for entries without coverage, so an undefined value
  // must render nothing rather than an empty state.
  readonly hasCoverage?: boolean;
}>();

const { t } = useI18n({ useScope: "local" });

const IRIS_URL = "https://iris.asta.tum.de/";
</script>

<template>
  <div
    v-if="props.hasCoverage"
    class="text-blue-900 dark:text-blue-50 bg-blue-50 dark:bg-blue-900 border border-blue-200 dark:border-blue-700 rounded p-3 text-sm flex flex-row gap-3 print:!hidden"
  >
    <MdiIcon :path="mdiBookOpenPageVariantOutline" :size="20" class="mt-0.5 shrink-0" aria-hidden="true" />
    <div class="flex flex-col items-start gap-2">
      <div class="flex flex-col gap-0.5">
        <span class="font-semibold">{{ t("title") }}</span>
        <span>{{ t("description") }}</span>
      </div>
      <Btn
        :to="IRIS_URL"
        variant="link"
        size="text-sm gap-1.5 rounded font-semibold"
      >
        {{ t("cta") }}
        <MdiIcon :path="mdiOpenInNew" :size="16" class="my-auto" aria-hidden="true" />
      </Btn>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  title: Auf der Suche nach einem Lernplatz?
  description: AStA Iris zeigt, welche Lernräume in diesem Gebäude gerade frei sind.
  cta: Freien Lernraum finden
en:
  title: Looking for a place to study?
  description: AStA Iris shows which learning rooms in this building are currently free.
  cta: Find a free learning room
</i18n>
