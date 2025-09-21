<script setup lang="ts">
const mode = defineModel<"bicycle" | "public_transit" | "motorcycle" | "car" | "pedestrian">(
  "mode",
  {
    required: true,
  }
);
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div class="end-0 flex flex-row justify-evenly">
    <Btn
      :variant="mode === 'pedestrian' ? 'primary' : 'secondary'"
      :disabled="mode == 'pedestrian'"
      size="md"
      :title="t('aria-pedestrian')"
      :aria-label="t('aria-pedestrian')"
      @click="mode = 'pedestrian'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
        <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
          <path d="M12 4a1 1 0 1 0 2 0a1 1 0 1 0-2 0M7 21l3-4m6 4l-2-4l-3-3l1-6" />
          <path d="m6 12l2-3l4-1l3 3l3 1" />
        </g>
      </svg>
    </Btn>
    <Btn
      :variant="mode == 'bicycle' ? 'primary' : 'secondary'"
      :disabled="mode == 'bicycle'"
      size="md"
      :title="t('aria-bicycle')"
      :aria-label="t('aria-bicycle')"
      @click="mode = 'bicycle'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
        <path
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M2 18a3 3 0 1 0 6 0a3 3 0 1 0-6 0m14 0a3 3 0 1 0 6 0a3 3 0 1 0-6 0m-4 1v-4l-3-3l5-4l2 3h3m-3-6a1 1 0 1 0 2 0a1 1 0 1 0-2 0"
        />
      </svg>
    </Btn>
    <Btn
      :variant="mode === 'public_transit' ? 'primary' : 'secondary'"
      :disabled="mode == 'public_transit'"
      size="md"
      :title="t('aria-public_transit')"
      :aria-label="t('aria-public_transit')"
      @click="mode = 'public_transit'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
        <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
          <path d="M21 13c0-3.87-3.37-7-10-7H3m0 9h16a2 2 0 0 0 2-2" />
          <path d="M3 6v5h17.5M3 10v4m5-3V6m5 5V6.5M3 19h18" />
        </g>
      </svg>
    </Btn>
    <Btn
      :variant="mode === 'motorcycle' ? 'primary' : 'secondary'"
      :disabled="mode == 'motorcycle'"
      size="md"
      :title="t('aria-motorcycle')"
      :aria-label="t('aria-motorcycle')"
      @click="mode = 'motorcycle'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
        <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
          <path d="M2 16a3 3 0 1 0 6 0a3 3 0 1 0-6 0m14 0a3 3 0 1 0 6 0a3 3 0 1 0-6 0m-8.5-2h5l4-4H6m1.5 4l4-4" />
          <path d="M13 6h2l1.5 3l2 4" />
        </g>
      </svg>
    </Btn>
    <Btn
      :variant="mode === 'car' ? 'primary' : 'secondary'"
      :disabled="mode == 'car'"
      size="md"
      :title="t('aria-car')"
      :aria-label="t('aria-car')"
      @click="mode = 'car'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
        <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
          <path d="M5 17a2 2 0 1 0 4 0a2 2 0 1 0-4 0m10 0a2 2 0 1 0 4 0a2 2 0 1 0-4 0" />
          <path d="M5 17H3v-6l2-5h9l4 5h1a2 2 0 0 1 2 2v4h-2m-4 0H9m-6-6h15m-6 0V6" />
        </g>
      </svg>
    </Btn>
  </div>
</template>

<i18n lang="yaml">
de:
  back: zurück
  aria-motorcycle: Motorrad
  aria-bicycle: Fahrrad
  aria-public_transit: Transit
  aria-car: Auto
  aria-pedestrian: Fußgänger
en:
  back: back
  aria-motorcycle: Motorcycle
  aria-bicycle: Bicycle
  aria-public_transit: Transit
  aria-car: Car
  aria-pedestrian: Pedestrian
</i18n>
