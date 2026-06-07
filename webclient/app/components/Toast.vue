<script setup lang="ts">
import { mdiClose } from "@mdi/js";

const props = withDefaults(
  defineProps<{
    id: string;
    msg?: string;
    level?: "error" | "warning" | "info" | "default";
    dismissable?: boolean;
  }>(),
  { level: "default", msg: "", dismissable: false }
);
// No `default: () => []`: that would make `useCookie` write the empty array back during SSR, which
// on `swr`-cached routes bakes a per-user `Set-Cookie` into the shared response (see the rationale
// in `userPreferences.ts`). The server only reads which toasts were dismissed; the client writes.
const dismissedToasts = useCookie<string[] | null>("shownToasts");
const isDismissed = computed(() => dismissedToasts.value?.includes(props.id) ?? false);

function dismiss() {
  dismissedToasts.value = [...(dismissedToasts.value ?? []), props.id];
}

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div
    v-if="!dismissable || !isDismissed"
    v-bind="{ id }"
    :data-cy="'toast-' + level"
    class="text-pretty rounded border p-1.5 text-sm leading-5"
    :class="{
      'text-red-900 dark:text-red-50 bg-red-100 dark:bg-red-800 border-red-300 dark:border-red-600': level === 'error',
      'text-orange-900 dark:text-orange-50 bg-orange-100 dark:bg-orange-800 border-orange-300 dark:border-orange-600': level === 'warning',
      'text-blue-900 dark:text-blue-50 bg-blue-100 dark:bg-blue-800 border-blue-300 dark:border-blue-600': level === 'info',
      'text-zinc-900 dark:text-zinc-50 bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600': level === 'default',
      'flex flex-row': dismissable,
    }"
  >
    <slot>{{ msg }}</slot>
    <button
      v-if="dismissable"
      type="button"
      :aria-label="t('close')"
      class="group text-zinc-800 dark:text-zinc-100 p-2 hover:text-blue-800 dark:hover:text-blue-100"
      @click.prevent="dismiss"
    >
      <MdiIcon v-if="dismissable" :path="mdiClose" :size="16" />
    </button>
  </div>
</template>

<i18n lang="yaml">
de:
  close: Toast schließen
en:
  close: close toast
</i18n>
