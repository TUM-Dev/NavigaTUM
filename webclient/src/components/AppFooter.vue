<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
import Btn from "@/components/Btn.vue";
import { computed } from "vue";

const global = useGlobalStore();
// If we do not include the image here like this, vite/rollup is unable to load it
const brandLogo = computed(() => new URL(`/src/assets/logos/tum_${global.theme}_${locale.value}.svg`, import.meta.url));
const { t, locale } = useI18n({ useScope: "local" });
const navigation = computed(() => [
  {
    name: t("sourcecode.text"),
    href: "https://github.com/TUM-Dev/navigatum",
  },
  {
    name: t("api.text"),
    href: "/api",
  },
  {
    name: t("about.text"),
    href: "/about/" + t("about.link"),
  },
  {
    name: t("privacy.text"),
    href: "/about/" + t("privacy.link"),
  },
  {
    name: t("imprint.text"),
    href: "/about/" + t("imprint.link"),
  },
]);
</script>

<template>
  <footer data-cy="main-footer" class="bg-zinc-100 mt-10">
    <div class="mx-auto max-w-7xl overflow-hidden px-6 py-20 sm:py-14 lg:px-8">
      <nav class="-mb-6 columns-2 text-center sm:columns-3 sm:justify-center sm:space-x-12 md:flex" aria-label="Footer">
        <div v-for="item in navigation" :key="item.name" class="pb-6 text-sm leading-6">
          <Btn variant="link" :to="item.href">{{ item.name }}</Btn>
        </div>
        <div class="pb-6 text-sm leading-6">
          <Btn variant="link" :aria-label="t('feedback.open')" @click="global.openFeedback()">
            {{ t("feedback.text") }}
          </Btn>
        </div>
      </nav>
      <div class="mt-10 flex justify-center space-x-10 text-center">
        <Btn to="https://tum.de" variant="rounded-xl px-3 py-2 focusable" size="sm">
          <p class="text-zinc-600 text-center text-xs">
            {{ t("official_roomfinder") }}<br />
            <img :alt="t('tum_logo_alt')" :src="brandLogo.href" width="200" class="h-20" aria-hidden="true" />
          </p>
        </Btn>
      </div>
    </div>
  </footer>
</template>

<i18n lang="yaml">
de:
  about:
    link: ueber-uns
    text: Über uns
  api:
    link: api
    text: API
  feedback:
    open: Feedback Form öffnen
    text: Feedback senden
  imprint:
    link: impressum
    text: Impressum
  language: Sprache
  official_roomfinder: Offizieller Roomfinder
  privacy:
    link: datenschutz
    text: Datenschutz
  sourcecode:
    text: Source Code
  theme: Theme
  tum_logo_alt: The Logo of the Technical University Munich
en:
  about:
    link: about-us
    text: About us
  api:
    link: api
    text: API
  feedback:
    open: Open the feedback-form
    text: Feedback
  imprint:
    link: imprint
    text: Imprint
  language: Language
  official_roomfinder: Official roomfinder
  privacy:
    link: privacy
    text: Privacy
  sourcecode:
    text: Source Code
  theme: Theme
  tum_logo_alt: Das Logo der Technischen Universität München
</i18n>
