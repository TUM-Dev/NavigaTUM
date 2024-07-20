<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    id?: string;
    msg?: string;
    level?: "error" | "warning" | "info" | "default";
  }>(),
  { level: "default", msg: "", id: undefined },
);
</script>

<template>
  <div
    v-bind="{ id }"
    :data-cy="'toast-' + props.level"
    class="text-pretty rounded border p-1.5 text-sm leading-5"
    :class="{
      'text-red-900 bg-red-100 border-red-300': props.level === 'error',
      'text-orange-900 bg-orange-100 border-orange-300': props.level === 'warning',
      'text-blue-900 bg-blue-100 border-blue-300': props.level === 'info',
      'text-zinc-900 bg-zinc-100 border-zinc-300': props.level === 'default',
    }"
  >
    <slot>{{ props.msg }}</slot>
  </div>
</template>
