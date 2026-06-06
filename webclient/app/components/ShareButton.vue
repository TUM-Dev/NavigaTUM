<script setup lang="ts">
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { mdiClipboard, mdiClipboardCheck, mdiCodeTags, mdiQrcode, mdiShare } from "@mdi/js";
import type { UseShareOptions } from "@vueuse/core";
import { useClipboard, useShare } from "@vueuse/core";
import type { components } from "~/api_types";

const props = defineProps<{
  readonly coords: components["schemas"]["Coordinate"];
  readonly name: string;
  readonly id: string;
}>();

const route = useRoute();
const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const { t } = useI18n({ useScope: "local" });
const {
  copy,
  copied,
  isSupported: clipboardIsSupported,
} = useClipboard({ source: clipboardSource });
const { share, isSupported: shareIsSupported } = useShare();
const runtimeConfig = useRuntimeConfig();

const modalOpen = ref(false);
const shareOptions = () =>
  ({
    title: props.name,
    text: document.title,
    url: clipboardSource.value,
  }) as UseShareOptions;

const qrCodeUrl = computed(
  () => `${runtimeConfig.public.apiURL}/api/locations/${props.id}/qr-code`
);

const embedSnippet = computed(
  () =>
    `<iframe src="https://nav.tum.de/embed/${props.id}" width="560" height="420" title="${props.name}" loading="lazy" referrerpolicy="strict-origin-when-cross-origin" allow="fullscreen; geolocation"></iframe>`
);
const {
  copy: copyEmbed,
  copied: embedCopied,
  isSupported: embedClipboardSupported,
} = useClipboard({ source: embedSnippet });

const tabs = [
  { key: "share", label: () => t("share"), icon: mdiShare },
  { key: "qr_code", label: () => t("qr_code"), icon: mdiQrcode },
  { key: "embed", label: () => t("embed"), icon: mdiCodeTags },
] as const;
</script>

<template>
  <button
    type="button"
    :title="t('external_link')"
    :aria-label="t('sharing_options')"
    class="focusable rounded-sm"
    @click="modalOpen = true"
  >
    <MdiIcon :path="mdiShare" :size="28" class="text-blue-600 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-50" />
  </button>
  <ClientOnly>
    <LazyModal v-model="modalOpen" :title="t('share')">
      <TabGroup>
        <TabList class="mb-4 flex flex-wrap gap-1 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1">
          <Tab v-for="tab in tabs" :key="tab.key" as="template" v-slot="{ selected }">
            <button
              type="button"
              :class="[
                'flex flex-1 items-center justify-center gap-1 rounded-md px-3 py-2 text-sm font-medium leading-5 transition-all',
                'ring-white/60 dark:ring-black/60 ring-offset-2 ring-offset-blue-400 dark:ring-offset-blue-500 focus:outline-none focus:ring-2',
                selected
                  ? 'bg-white dark:bg-black text-zinc-700 dark:text-zinc-200 shadow'
                  : 'text-zinc-500 dark:text-zinc-400 hover:bg-white/60 dark:hover:bg-black/60 hover:text-zinc-700 dark:hover:text-zinc-200',
              ]"
            >
              <MdiIcon :path="tab.icon" :size="16" class="shrink-0" />
              <span class="truncate">{{ tab.label() }}</span>
            </button>
          </Tab>
        </TabList>
        <TabPanels>
          <TabPanel class="flex flex-col gap-4 focus:outline-none">
            <div class="flex flex-col gap-2">
              <Btn v-if="shareIsSupported" variant="primary" @click="share(shareOptions)">
                <MdiIcon :path="mdiShare" v-if="copied" :size="24" class="my-auto h-4 w-4" />
                {{ t("share_link") }}
              </Btn>
              <Btn v-if="clipboardIsSupported" variant="primary" @click="copy()">
                <MdiIcon
                  :path="copied ? mdiClipboardCheck : mdiClipboard"
                  :size="24"
                  class="my-auto h-4 w-4"
                />
                {{ copied ? t("copied") : t("copy_link") }}
              </Btn>
            </div>
            <div class="flex flex-col gap-1">
              <h3 class="text-sm font-semibold text-zinc-600 dark:text-zinc-300">{{ t("open_in") }}</h3>
              <Btn
                variant="link"
                :to="`https://www.google.com/maps/search/?api=1&query=${coords.lat}%2C${coords.lon}`"
              >
                Google Maps
              </Btn>
              <Btn
                variant="link"
                :to="`https://www.openstreetmap.org/?mlat=${coords.lat}&mlon=${coords.lon}#map=17/${coords.lat}/${coords.lon}&layers=T`"
              >
                OpenStreetMap
              </Btn>
              <Btn variant="link" :to="`geo:${coords.lat},${coords.lon}`">
                {{ t("other_app") }}
              </Btn>
            </div>
          </TabPanel>
          <TabPanel class="focus:outline-none">
            <div class="flex justify-center">
              <img
                :src="qrCodeUrl"
                :alt="t('qr_code_alt')"
                width="500"
                height="500"
                class="bg-zinc-50 dark:bg-zinc-900 w-100 max-w-64"
              />
            </div>
          </TabPanel>
          <TabPanel class="flex flex-col gap-2 focus:outline-none">
            <p class="text-sm text-zinc-500 dark:text-zinc-400">{{ t("embed_description") }}</p>
            <textarea
              readonly
              rows="3"
              class="focusable rounded-sm border border-zinc-300 dark:border-zinc-600 bg-zinc-50 dark:bg-zinc-900 p-2 font-mono text-xs text-zinc-700 dark:text-zinc-200"
              :value="embedSnippet"
              @focus="($event.target as HTMLTextAreaElement).select()"
            />
            <Btn v-if="embedClipboardSupported" variant="primary" @click="copyEmbed()">
              <MdiIcon
                :path="embedCopied ? mdiClipboardCheck : mdiClipboard"
                :size="24"
                class="my-auto h-4 w-4"
              />
              {{ embedCopied ? t("copied") : t("copy_embed") }}
            </Btn>
          </TabPanel>
        </TabPanels>
      </TabGroup>
    </LazyModal>
  </ClientOnly>
</template>

<i18n lang="yaml">
de:
  copied: Kopiert
  copy_link: Link kopieren
  copy_embed: Einbettungs-Code kopieren
  open_in: Öffnen in
  other_app: Andere App ...
  external_link: Externe Links
  sharing_options: Externe Links und optionen diese seite zu teilen
  share: Teilen
  share_link: Teilen mit ...
  qr_code: QR-Code
  qr_code_alt: QR-Code für diese Seite
  embed: Einbetten
  embed_description: Diesen Ort als interaktive Karte auf deiner eigenen Seite einbetten.
en:
  copied: Copied
  copy_link: Copy link
  copy_embed: Copy embed code
  open_in: Open in
  other_app: Other app ...
  external_link: External links
  sharing_options: External links and options to share this page
  share: Share
  share_link: Share with ...
  qr_code: QR Code
  qr_code_alt: QR code for this page
  embed: Embed
  embed_description: Embed this location as an interactive map on your own page.
</i18n>
