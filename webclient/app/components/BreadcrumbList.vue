<script setup lang="ts">
// `to` optional: items without a route render as plain text.
// `current` is rendered visually hidden so screen readers get the full trail without echoing the `<h1>`.
interface Item {
  to?: string;
  name: string;
  current?: boolean;
}
const props = withDefaults(defineProps<{ items: Item[]; class?: string }>(), {
  class: "",
});
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <nav :aria-label="t('aria_label')">
    <ol
      vocab="https://schema.org/"
      typeof="BreadcrumbList"
      class="flex flex-row flex-wrap gap-2 text-sm"
      :class="props.class"
    >
      <template v-for="(item, i) in props.items" :key="i">
        <span v-if="i > 0 && !item.current" aria-hidden="true" class="text-zinc-500 dark:text-zinc-400">/</span>
        <li property="itemListElement" typeof="ListItem" :class="{ 'sr-only': item.current }">
          <NuxtLinkLocale
            v-if="item.to"
            :to="item.to"
            property="item"
            typeof="WebPage"
            class="focusable rounded-sm hover:underline"
            :class="{
              'visited:text-blue-500 dark:visited:text-blue-400': i > 0,
              'visited:text-zinc-500 dark:visited:text-zinc-400': i === 0,
              'hover:text-blue-600 dark:hover:text-blue-300': i > 0,
              'hover:text-zinc-600 dark:hover:text-zinc-300': i === 0,
              'text-blue-500 dark:text-blue-400': i > 0,
              'text-zinc-500 dark:text-zinc-400': i === 0,
            }"
          >
            <span property="name">{{ item.name }}</span>
          </NuxtLinkLocale>
          <span
            v-else
            property="name"
            :aria-current="item.current ? 'page' : undefined"
            class="text-zinc-500 dark:text-zinc-400"
          >{{ item.name }}</span>
          <meta property="position" :content="`${i + 1}`" />
        </li>
      </template>
    </ol>
  </nav>
</template>

<i18n lang="yaml">
de:
  aria_label: Navigationspfad
en:
  aria_label: Breadcrumb
</i18n>
