<script setup lang="ts">
import Datenschutz from "../assets/md/datenschutz.md";
import Privacy from "../assets/md/privacy.md";
import AboutUs from "../assets/md/about-us.md";
import UeberUns from "../assets/md/ueber-uns.md";
import Impressum from "../assets/md/impressum.md";

import { ref } from "vue";
const url = ref(location.pathname.substring(7));
</script>

<template>
  <div id="view-md">
    <Datenschutz v-if="url === 'datenschutz'" />
    <Privacy v-if="url === 'privacy'" />
    <AboutUs v-if="url === 'about-us'" />
    <UeberUns v-if="url === 'ueber-uns'" />
    <Impressum v-if="url === 'impressum'" />
  </div>
</template>

<style lang="scss" scoped>
@import "../assets/variables";

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
