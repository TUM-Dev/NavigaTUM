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

const route = useRoute();
</script>

<template>
  <div class="pt-4" id="contentwrapper">
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
    @apply text-3xl font-medium;
  }
  h2 {
    @apply text-2xl font-medium;
  }
  h3 {
    @apply text-lg font-medium;
  }
  h4 {
    @apply text-sm font-medium;
  }

  code {
    @apply text-xs font-mono max-w-full overflow-auto inline-flex text-left items-center space-x-4 rounded-md px-4 py-3 bg-blue-50 text-blue-950 mb-4;
  }
}
</style>
