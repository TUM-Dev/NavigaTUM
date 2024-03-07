<script setup lang="ts">
import PreferencesPopup from "@/components/PreferencesPopup.vue";
</script>

<template>
  <header class="bg-zinc-50 fixed top-0 z-20 w-full shadow-md shadow-black/5 dark:bg-zinc-100 dark:shadow-black/20">
    <nav class="mx-auto max-w-4xl">
      <div class="mx-5 my-2.5 flex place-content-between content-between justify-between gap-x-10 lg:mx-10">
        <RouterLink class="my-auto hidden lg:block" to="/">
          <img alt="Navigatum Logo" class="pt-2" width="201" height="32" src="@/assets/logos/navigatum.svg" />
        </RouterLink>
        <div class="my-auto max-w-lg flex-grow lg:col-span-3">
          <slot />
        </div>
        <div class="relative my-auto flex-shrink-0 xl:absolute xl:right-5 xl:top-3">
          <PreferencesPopup />
        </div>
      </div>
    </nav>
  </header>
</template>
