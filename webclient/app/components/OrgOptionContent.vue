<script setup lang="ts">
import type { OrgOption } from "~/composables/useKnownOrgs";

defineProps<{
  org: OrgOption;
  emphasised?: boolean;
}>();
</script>

<template>
  <span class="text-sm text-zinc-800 dark:text-zinc-100" :class="emphasised ? 'font-bold' : 'font-normal'">{{ org.label }}</span>
  <span class="text-zinc-400 dark:text-zinc-500 ml-auto font-mono text-[10px]">{{ org.code }}</span>
</template>
