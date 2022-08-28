<script lang="ts">
import { copyCurrentLink } from "@/utils/common";

export default {
  props: ["coords"],
  data() {
    return {
      browser_supports_share: "share" in navigator,
      copied:false,
    };
  },
  methods: {
    shareLink: function () {
      if (navigator.share) {
        navigator.share({
          title: this.view_data.name,
          text: document.title,
          url: window.location.href,
        });
      }
    },
    copyCurrentLink: copyCurrentLink,
  },
};
</script>

<style lang="scss" scoped></style>

<template>
  <div class="link-popover">
    <strong>{{ $t("view_view.header.external_link.open_in") }}</strong>
    <a
      class="btn"
      target="_blank"
      :href="
        'https://www.openstreetmap.org/?mlat=' +
        coords.lat +
        '&mlon=' +
        coords.lon +
        '#map=17/' +
        coords.lat +
        '/' +
        coords.lon +
        '&layers=T'
      "
      >OpenStreetMap</a
    ><br />
    <a
      class="btn"
      target="_blank"
      :href="
        'https://www.google.com/maps/search/?api=1&query=' +
        coords.lat +
        '%2C' +
        coords.lon
      "
      >Google Maps</a
    >
    <a class="btn" :href="'geo:' + coords.lat + ',' + coords.lon">{{
      $t("view_view.header.external_link.other_app ")
    }}</a>
    <strong>{{ $t("view_view.header.external_link.share") }}</strong>
    <button class="btn" @click="shareLink" v-if="browser_supports_share">
      {{ $t("view_view.header.external_link.shareLink") }}
    </button>
    <button
      class="btn"
      @click="copyCurrentLink(copied)"
      v-html="
        copied
          ? $t('view_view.header.external_link.copied')
          : $t('view_view.header.copy_link')
      "
    ></button>
  </div>
</template>
