<script setup lang="ts">
import PreferencesPopup from "@/components/PreferencesPopup.vue";
</script>

<template>
  <header class="fixed top-0 z-10 w-full bg-white shadow-md shadow-black/10">
    <!-- dark:bg-gray-900 dark:shadow-black/40"> -->
    <nav class="mx-auto max-w-4xl">
      <div class="mx-10 my-2.5 flex place-content-between content-between justify-between gap-x-10">
        <RouterLink class="hidden my-auto lg:block" to="/">
          <img alt="Navigatum Logo" class="my-auto h-10" width="200" src="@/assets/logos/navigatum.svg" />
          <!-- 7px for logo1 -->
        </RouterLink>
        <div class="my-auto max-w-lg flex-grow lg:col-span-3">
          <slot />
        </div>
        <PreferencesPopup />
      </div>
    </nav>
  </header>
</template>
