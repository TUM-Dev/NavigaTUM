<script setup lang="ts">
import { XMarkIcon } from "@heroicons/vue/24/outline";
withDefaults(
  defineProps<{
    id?: string;
    msg?: string;
    level?: "error" | "warning" | "info" | "default";
    dismissable?: boolean;
  }>(),
  { level: "default", msg: "", id: undefined, dismissable: false }
);
const shown = useCookie<string[]>("shownToasts", {
  default: () => [],
});
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div
    v-if="!dismissable || !shown.includes(id)"
    v-bind="{ id }"
    :data-cy="'toast-' + level"
    class="text-pretty rounded border p-1.5 text-sm leading-5"
    :class="{
      'text-red-900 bg-red-100 border-red-300': level === 'error',
      'text-orange-900 bg-orange-100 border-orange-300': level === 'warning',
      'text-blue-900 bg-blue-100 border-blue-300': level === 'info',
      'text-zinc-900 bg-zinc-100 border-zinc-300': level === 'default',
      'flex flex-row': dismissable,
    }"
  >
    <slot>{{ msg }}</slot>
    <button
      v-if="dismissable"
      type="button"
      :aria-label="t('close')"
      class="group text-zinc-800 p-2 hover:text-blue-800"
      @click.prevent="() => shown.push(id)"
    >
      <XMarkIcon v-if="dismissable" class="h-4 w-4 stroke-1 group-hover:stroke-2" />
    </button>
  </div>
</template>

<i18n lang="yaml">
de:
  close: Toast schließen
en:
  close: close toast
</i18n>
