<script setup lang="ts">
import type { components } from "~/api_types";

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

const route = useRoute();
const localePath = useLocalePath();
const runtimeConfig = useRuntimeConfig();
const { locale } = useI18n();
const id = route.params.id as string;

const { data } = await useFetch<LocationDetailsResponse, string>(
  `${runtimeConfig.public.apiURL}/api/locations/${id}?lang=${locale.value}`,
  { dedupe: "cancel", credentials: "omit" }
);

const targetPath = data.value?.redirect_url ?? `/view/${id}`;
await navigateTo(
  { path: localePath(targetPath), query: { "calendar[]": id } },
  { redirectCode: 301, replace: true }
);
</script>

<template>
  <div />
</template>
