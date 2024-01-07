<script setup lang="ts">
import { computed, ref } from "vue";
import { useClipboard, useShare } from "@vueuse/core";
import type { UseShareOptions } from "@vueuse/core";
import type { components } from "@/api_types";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { ShareIcon } from "@heroicons/vue/24/outline";
import Modal from "@/components/Modal.vue";

const props = defineProps<{
  readonly coords: components["schemas"]["Coordinate"];
  readonly name: string;
}>();

const route = useRoute();
const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const { t } = useI18n({ useScope: "local" });
const { copy, copied, isSupported: clipboardIsSupported } = useClipboard({ source: clipboardSource });
const { share, isSupported: shareIsSupported } = useShare();

const modalOpen = ref(false);
const shareOptions = computed<UseShareOptions>(() => ({
  title: props.name,
  text: document.title,
  url: clipboardSource.value,
}));
</script>

<template>
  <button type="button" :title="t('external_link')" class="focusable rounded-sm" @click="modalOpen = true">
    <ShareIcon class="h-4 text-blue-600 w-4" />
  </button>
  <Modal v-model="modalOpen" :title="t('share')">
    <div class="flex flex-col gap-5">
      <div class="flex flex-col gap-2">
        <strong>{{ t("open_in") }}</strong>
        <a
          class="btn"
          target="_blank"
          :href="`https://www.google.com/maps/search/?api=1&query=${coords.lat}%2C${coords.lon}`"
          >Google Maps</a
        >
        <a
          class="btn"
          target="_blank"
          :href="`https://www.openstreetmap.org/?mlat=${coords.lat}&mlon=${coords.lon}#map=17/${coords.lat}/${coords.lon}&layers=T`"
          >OpenStreetMap</a
        >
        <a class="btn" :href="`geo:${coords.lat},${coords.lon}`">
          {{ t("other_app") }}
        </a>
      </div>
      <div class="flex flex-col gap-2">
        <strong>{{ t("share") }}</strong>
        <button v-if="shareIsSupported" type="button" class="btn" @click="share(shareOptions)">
          {{ t("share_link") }}
        </button>
        <button v-if="clipboardIsSupported" type="button" class="btn" @click="copy()">
          {{ copied ? t("copied") : t("copy_link") }}
        </button>
      </div>
    </div>
  </Modal>
</template>

<i18n lang="yaml">
de:
  copied: Kopiert
  copy_link: Link kopieren
  open_in: Öffnen in
  other_app: Andere App ...
  external_link: Externe Links
  share: Teilen
  share_link: Teilen mit ...
en:
  copied: Copied
  copy_link: Copy link
  open_in: Open in
  other_app: Other app ...
  external_link: External links
  share: Share
  share_link: Share with ...
</i18n>
