<script lang="ts">
export interface IndoorRoomPopupProps {
  /** Canonical NavigaTUM room id (`ref:tum`); `null` for a bare toilet/shower node. */
  readonly refTum: string | null;
  readonly lat: number;
  readonly lng: number;
  readonly zoom: number;
  /** Active OSM floor level; defaults to 0 when no floor is selected. */
  readonly level: number;
  readonly isToilet: boolean;
  readonly isShower: boolean;
  readonly isMale: boolean;
  readonly isFemale: boolean;
  readonly isWheelchair: boolean;
}
</script>

<script setup lang="ts">
import { mdiOpenInNew } from "@mdi/js";
import type { components } from "~/api_types";

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

const props = defineProps<IndoorRoomPopupProps>();

const { t, locale } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();

const name = ref<string | null>(null);
const typeCommonName = ref<string | null>(null);
const loading = ref(false);

// Both actions work from the tile alone; the details fetch is pure header enrichment,
// so a 404 (data drift) or offline error degrades silently to the bare room code.
watch(
  () => props.refTum,
  async (refTum) => {
    name.value = null;
    typeCommonName.value = null;
    if (!refTum) {
      loading.value = false;
      return;
    }
    loading.value = true;
    try {
      const data = await $fetch<LocationDetailsResponse>(
        `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(refTum)}`,
        { query: { lang: locale.value }, credentials: "omit" }
      );
      name.value = data.name;
      typeCommonName.value = data.type_common_name;
    } catch {
      // Keep the code as the title and show no error banner.
    } finally {
      loading.value = false;
    }
  },
  { immediate: true }
);

// Only reachable with `refTum === null` when the feature is a toilet or shower node.
const kindLabel = computed(() => (props.isShower ? t("shower") : t("toilet")));
const title = computed(() => name.value ?? props.refTum ?? kindLabel.value);

const genders = computed(() => {
  // A toilet serving both is all-gender; show it as unisex rather than "male, female".
  if (props.isMale && props.isFemale) return [t("gender.unisex")];
  const out: string[] = [];
  if (props.isMale) out.push(t("gender.male"));
  if (props.isFemale) out.push(t("gender.female"));
  return out;
});

const osmUrl = computed(
  () =>
    `https://osminedit.pavie.info/#${Math.round(props.zoom)}/${props.lat.toFixed(7)}/${props.lng.toFixed(7)}/${props.level}`
);
const roomPath = computed(() => (props.refTum ? `/room/${props.refTum}` : null));
</script>

<template>
  <div class="flex min-w-[12rem] flex-col gap-2 text-sm">
    <div class="flex flex-col gap-1">
      <p class="font-semibold leading-tight">{{ title }}</p>
      <div v-if="refTum && loading" class="bg-zinc-200 h-3 w-2/3 animate-pulse rounded" />
      <p v-else-if="typeCommonName" class="opacity-60">{{ typeCommonName }}</p>
    </div>

    <div v-if="isToilet" class="flex flex-col gap-0.5 opacity-70">
      <p v-if="genders.length">{{ t("gender.label") }}: {{ genders.join(", ") }}</p>
      <p v-if="isWheelchair">{{ t("wheelchair_accessible") }}</p>
    </div>

    <div class="flex flex-col gap-1.5">
      <Btn v-if="roomPath" :to="roomPath" variant="primary" class="items-center justify-center">
        {{ t("open_room") }}
      </Btn>
      <Btn :to="osmUrl" variant="secondary" class="items-center justify-center">
        <MdiIcon :path="mdiOpenInNew" :size="16" />
        {{ t("edit_in_osm") }}
      </Btn>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  open_room: Raum öffnen
  edit_in_osm: In OpenStreetMap bearbeiten
  toilet: Toilette
  shower: Dusche
  wheelchair_accessible: Rollstuhlgerecht
  gender:
    label: Geschlecht
    male: Herren
    female: Damen
    unisex: Geschlechtsneutral
en:
  open_room: Open room
  edit_in_osm: Edit in OpenStreetMap
  toilet: Toilet
  shower: Shower
  wheelchair_accessible: Wheelchair accessible
  gender:
    label: Gender
    male: Male
    female: Female
    unisex: Unisex
</i18n>
