<script setup lang="ts">
import { ArrowTopRightOnSquareIcon, InformationCircleIcon } from "@heroicons/vue/24/outline";
import type { components } from "~/api_types";

type Props = components["schemas"]["PropsResponse"];

defineProps<{ props: Props }>();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div v-if="props.links || props.computed" class="text-zinc-800 flex flex-col gap-3">
    <p v-for="prop in props.computed" :key="prop.name" class="flex flex-col">
      <span class="text-zinc-500 text-xs font-semibold uppercase">{{ prop.name }}</span>
      <span>{{ prop.text }}</span>
      <TinyModal v-if="prop.extra?.body" :content="prop.extra">
        <template #icon>
          <InformationCircleIcon class="h-4 w-4" />
        </template>
      </TinyModal>
    </p>
    <div>
      <ul v-if="props.links" class="flex flex-col gap-1.5">
        <li v-for="link in props.links" :key="link.text">
          <Btn
            size="text-md gap-2.5 px-3 py-1.5 rounded leading-snug print:!text-blue-500"
            variant="secondary"
            :to="link.url"
          >
            <ArrowTopRightOnSquareIcon class="my-auto h-5 min-h-5 w-5 min-w-5 pb-0.5" />
            {{ link.text }}
          </Btn>
        </li>
      </ul>
    </div>
  </div>
  <div v-else>
    {{ t("no_information_known") }}
  </div>
</template>

<i18n lang="yaml">
de:
  links: Links
  no_information_known: No information known
en:
  links: Links
  no_information_known: Keine Informationen bekannt
</i18n>
