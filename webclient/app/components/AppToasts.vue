<script setup lang="ts">
const { t } = useI18n({ useScope: "local" });
const month = computed<number>(() => new Date().getMonth() + 1);
</script>

<template>
  <Toast
    v-if="[2, 3, 4, 8, 9].includes(month)"
    :id="'osm_sc_link-' + new Date().getFullYear()"
    level="info"
    :dismissable="true"
  >
    <I18nT class="text-sm" tag="span" keypath="street_complete">
      <template #osm_sc_link>
        <NuxtLink
          to="https://streetcomplete.app/"
          class="text-blue-600 underline visited:text-blue-600 hover:text-blue-700"
          target="_blank"
        >
          {{ t("osm_sc_link") }}
        </NuxtLink>
      </template>
    </I18nT>
  </Toast>
</template>

<i18n lang="yaml">
de:
  street_complete: Wenn du während der Lernpausen für Prüfungen spazieren gehst, macht es Spaß, {osm_sc_link}. Genauere Karten helfen uns, bessere Navigationserlebnisse zu entwickeln.
  osm_sc_link: OpenStreetMap mit StreetComplete zu verbessern
en:
  street_complete: If you go for walks during exam-study breaks, helping us by {osm_sc_link} is fun. More accurate maps assist us in building better navigation experiences.
  osm_sc_link: improving OpenStreetMap with StreetComplete
</i18n>
