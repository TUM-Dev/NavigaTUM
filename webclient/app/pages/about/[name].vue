<script lang="ts" setup>
const route = useRoute();

if (route.path === "/about/about-us") navigateTo("/about/ueber-uns");
if (route.path === "/en/about/ueber-uns") navigateTo("/en/about/about-us");
if (route.path === "/about/imprint") navigateTo("/about/impressum");
if (route.path === "/en/about/impressum") navigateTo("/en/about/imprint");
if (route.path === "/about/privacy") navigateTo("/about/datenschutz");
if (route.path === "/en/about/datenschutz") navigateTo("/en/about/privacy");

const { data: page } = await useAsyncData(route.path, () => {
  return queryCollection("content").path(route.path).first();
});
</script>

<template>
  <div id="contentwrapper" class="pt-4">
    <ContentRenderer v-if="page" :value="page" :prose="true" />
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
  ol {
    @apply text-zinc-700 py-1 text-sm font-medium leading-6;
  }

  h4,
  h5,
  h6 {
    @apply text-zinc-700 py-1 text-sm font-semibold uppercase leading-6;
  }

  li {
    @apply ms-5 list-outside list-disc;
  }

  code {
    @apply text-blue-900 bg-blue-100 mb-4 flex max-w-full flex-col items-start space-x-4 overflow-auto rounded-md px-4 py-3 text-left font-mono text-xs text-blue-950 dark:bg-blue-50;
  }
  code span {
    @apply !ms-0;
  }

  p a {
    @apply text-blue-600 gap-1 rounded bg-transparent visited:text-blue-600 hover:underline focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus-visible:outline-0;
  }
}
</style>
