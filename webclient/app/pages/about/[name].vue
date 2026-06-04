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

// Extract title and description from the markdown frontmatter
const pageTitle = computed(() => {
  return page.value?.title ? `${page.value.title} - NavigaTUM` : "NavigaTUM";
});
const pageDescription = computed(() => page.value?.description ?? undefined);

useSeoMeta({
  title: pageTitle,
  ogTitle: pageTitle,
  description: pageDescription,
  ogDescription: pageDescription,
  ogType: "website",
  twitterCard: "summary_large_image",
});
</script>

<template>
  <div id="contentwrapper" class="pt-4">
    <ContentRenderer v-if="page" :value="page" :prose="true" />
  </div>
</template>

