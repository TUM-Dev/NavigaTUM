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
import { useRoute } from "vue-router";

const route = useRoute();
</script>

<template>
  <div id="view-md">
    <Datenschutz v-if="route.params.name === 'datenschutz'" />
    <Privacy v-else-if="route.params.name === 'privacy'" />
    <AboutUs v-else-if="route.params.name === 'about-us'" />
    <UeberUns v-else-if="route.params.name === 'ueber-uns'" />
    <Impressum v-else-if="route.params.name === 'impressum'" />
    <p v-else>Not found</p>
  </div>
</template>

<style lang="scss">
@import "@/assets/variables";

#view-md {
  padding-top: 15px;

  h1 {
    font-size: 1.8rem;
  }

  h2 {
    font-size: 1.5rem;
  }

  h1,
  h2,
  h3 {
    font-weight: 500;
  }

  code {
    background: $code-bg;
  }

  .code code {
    background: $bg-color;
  }
}
</style>
