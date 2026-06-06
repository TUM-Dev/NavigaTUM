<script setup lang="ts">
import { mdiInformation, mdiOpenInNew } from "@mdi/js";
import type { components } from "~/api_types";

type Props = components["schemas"]["PropsResponse"];

defineProps<{
  props: Props;
}>();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div v-if="props.links || props.computed" class="text-zinc-800 dark:text-zinc-100 flex flex-col gap-3">
    <p v-for="prop in props.computed" :key="prop.name" class="flex flex-col">
      <span class="text-zinc-500 dark:text-zinc-400 text-xs font-semibold uppercase">{{ prop.name }}</span>
      <span>{{ prop.text }}</span>
      <TinyModal v-if="prop.extra?.body" :content="prop.extra">
        <template #icon>
          <MdiIcon :path="mdiInformation" :size="16" />
        </template>
      </TinyModal>
    </p>
    <ul v-if="props.links" class="flex flex-col gap-1.5">
      <li v-for="link in props.links" :key="link.text">
        <Btn
          size="text-md gap-2.5 px-3 py-1.5 rounded leading-snug print:!text-blue-500 dark:print:!text-blue-400"
          variant="secondary"
          :to="link.url"
        >
          <MdiIcon :path="mdiOpenInNew" :size="20" class="my-auto min- min- pb-0.5" />
          {{ link.text }}
        </Btn>
      </li>
    </ul>
  </div>
  <div v-else>
    {{ t("no_information_known") }}
  </div>
</template>

<i18n lang="yaml">
de:
  links: Links
  no_information_known: Keine Informationen bekannt
en:
  links: Links
  no_information_known: No information known
</i18n>
