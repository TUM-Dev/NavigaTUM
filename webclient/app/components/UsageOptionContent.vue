<script setup lang="ts">
import type { UsageOption } from "~/composables/knownUsages";

defineProps<{
  usage: UsageOption;
  emphasised?: boolean;
}>();

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <span class="text-sm text-zinc-800 dark:text-zinc-100" :class="emphasised ? 'font-bold' : 'font-normal'">{{ usage.label }}</span>
  <span class="text-zinc-400 dark:text-zinc-500 ml-auto text-xs">{{ usage.occurrences }}×</span>
  <span class="text-zinc-400 dark:text-zinc-500 font-mono text-[10px]" :title="t('din_277_help')">{{ usage.din }}</span>
</template>

<i18n lang="yaml">
de:
  din_277_help: DIN 277 Klassifizierung
en:
  din_277_help: DIN 277 classification
</i18n>
