<script setup lang="ts">
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import Datenschutz from "@/assets/md/datenschutz.md";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import Privacy from "@/assets/md/privacy.md";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import AboutUs from "@/assets/md/about-us.md";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import UeberUns from "@/assets/md/ueber-uns.md";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import Impressum from "@/assets/md/impressum.md";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import Imprint from "@/assets/md/imprint.md";
import { useRoute } from "vue-router";
import Toast from "@/components/Toast.vue";
import ManyChangesToast from "@/components/ManyChangesToast.vue";
import { useGlobalStore } from "@/stores/global";

const route = useRoute();
const global = useGlobalStore();
</script>

<template>
  <div class="-mb-1 flex flex-col gap-4 pt-5">
    <Toast v-if="global.error_message" :msg="global.error_message" level="error" />
    <ManyChangesToast />
  </div>
  <div id="contentwrapper" class="pt-4">
    <Datenschutz v-if="route.params.name === 'datenschutz'" />
    <Privacy v-else-if="route.params.name === 'privacy'" />
    <AboutUs v-else-if="route.params.name === 'about-us'" />
    <UeberUns v-else-if="route.params.name === 'ueber-uns'" />
    <Impressum v-else-if="route.params.name === 'impressum'" />
    <Imprint v-else-if="route.params.name === 'imprint'" />
    <p v-else>Not found</p>
  </div>
</template>

<style lang="postcss">
#contentwrapper {
  h1 {
    @apply text-zinc-900 pb-3 pt-4 text-3xl font-medium;
  }
  h2 {
    @apply text-zinc-900 pb-2 pt-3 text-2xl font-medium;
  }
  h3 {
    @apply text-zinc-800 pb-1 pt-2 text-lg font-medium;
  }
  p,
  ul,
  ol,
  h4,
  h5,
  h6 {
    @apply text-zinc-700 py-1 text-sm font-medium leading-6;
  }
  li {
    @apply ms-5 list-outside list-disc;
  }
  code {
    @apply text-tumBlue-900 bg-tumBlue-100 mb-4 inline-flex max-w-full items-center space-x-4 overflow-auto rounded-md px-4 py-3 text-left font-mono text-xs text-tumBlue-950 dark:bg-tumBlue-50;
  }
  a {
    @apply text-tumBlue-600 gap-1 rounded bg-transparent visited:text-tumBlue-600 hover:underline focus:ring-2 focus:ring-tumBlue-500 focus:ring-offset-2 focus-visible:outline-0;
  }
}
</style>
