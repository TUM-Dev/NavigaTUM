<script setup lang="ts">
interface ActionBase {
  key: string;
  icon: string;
  label: string;
  shortLabel: string;
  visible?: boolean;
}
export type DetailAction = ActionBase &
  ({ href: string; onClick?: never } | { onClick: () => void; href?: never });

const props = defineProps<{
  actions: DetailAction[];
}>();

const visibleActions = computed(() => props.actions.filter((a) => a.visible !== false));

const tileClass =
  "focusable flex h-full w-full cursor-pointer flex-col items-center justify-center gap-0.5 rounded-sm border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 px-2 py-1.5 text-blue-600 dark:text-blue-300 hover:border-blue-300 dark:hover:border-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900 hover:text-blue-900 dark:hover:text-blue-50 transition-colors";
const labelClass = "text-center text-xs font-medium leading-tight";
const mobileOnlyLabelClass = `${labelClass} sm:hidden`;
const desktopOnlyLabelClass = `hidden ${labelClass} sm:block`;
</script>

<template>
  <ul class="m-0 flex list-none flex-row flex-wrap items-stretch gap-2 p-0 print:hidden">
    <li
      v-for="action in visibleActions"
      :key="action.key"
      class="min-w-0 grow basis-[calc(50%-0.5rem)] sm:basis-0"
    >
      <NuxtLinkLocale
        v-if="action.href !== undefined"
        :to="action.href"
        :class="tileClass"
        :aria-label="action.label"
        prefetch-on="interaction"
      >
        <MdiIcon :path="action.icon" :size="20" class="shrink-0" />
        <span v-if="action.shortLabel === action.label" :class="labelClass">{{ action.label }}</span>
        <template v-else>
          <span :class="mobileOnlyLabelClass">{{ action.shortLabel }}</span>
          <span :class="desktopOnlyLabelClass">{{ action.label }}</span>
        </template>
      </NuxtLinkLocale>
      <button
        v-else
        type="button"
        :class="tileClass"
        :aria-label="action.label"
        @click="action.onClick?.()"
      >
        <MdiIcon :path="action.icon" :size="20" class="shrink-0" />
        <span v-if="action.shortLabel === action.label" :class="labelClass">{{ action.label }}</span>
        <template v-else>
          <span :class="mobileOnlyLabelClass">{{ action.shortLabel }}</span>
          <span :class="desktopOnlyLabelClass">{{ action.label }}</span>
        </template>
      </button>
    </li>
  </ul>
</template>
