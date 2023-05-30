<script setup lang="ts">
import { ref } from "vue";
import { useClipboard, useShare } from "@vueuse/core";
import type { ShareOptions } from "@vueuse/core";
import type { components } from "@/api_types";

const props = defineProps<{
  readonly coords: components["schemas"]["Coordinate"];
  readonly name: string;
}>();

const { copy, copied, isSupported: clipboardIsSupported } = useClipboard({ source: "abc" });
const shareOptions = ref<ShareOptions>({
  title: props.name,
  text: document.title,
  url: window.location.href,
});
const { share, isSupported: shareIsSupported } = useShare(shareOptions);
</script>

<template>
  <div class="link-popover">
    <strong>{{ $t("view_view.header.external_link.open_in") }}</strong>
    <a
      class="btn"
      target="_blank"
      :href="`https://www.openstreetmap.org/?mlat=${coords.lat}&mlon=${coords.lon}#map=17/${coords.lat}/${coords.lon}&layers=T`"
      >OpenStreetMap</a
    ><br />
    <a
      class="btn"
      target="_blank"
      :href="`https://www.google.com/maps/search/?api=1&query=${coords.lat}%2C${coords.lon}`"
      >Google Maps</a
    >
    <a class="btn" :href="`geo:${coords.lat},${coords.lon}`">
      {{ $t("view_view.header.external_link.other_app") }}
    </a>
    <strong>{{ $t("view_view.header.external_link.share") }}</strong>
    <button class="btn" @click="share" v-if="shareIsSupported">
      {{ $t("view_view.header.external_link.share_link") }}
    </button>
    <button class="btn" @click="copy" v-if="clipboardIsSupported">
      {{ copied ? $t("view_view.header.external_link.copied") : $t("view_view.header.copy_link") }}
    </button>
  </div>
</template>
