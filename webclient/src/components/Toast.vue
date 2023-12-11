<script setup lang="ts">
const props = defineProps<{
  msg: string;
  level?: "error" | "warning" | "info";
}>();
</script>

<template>
  <div
    class="rounded-sm p-1.5"
    :class="{
      'bg-red-300': props.level === 'error',
      'text-red-950': props.level === 'error',
      'bg-orange-200': props.level === 'warning',
      'text-orange-950': props.level === 'warning',
      'bg-blue-300': props.level === 'info',
      'text-blue-950': props.level === 'info',
      'bg-gray-300': props.level === undefined,
      'text-gray-950': props.level === undefined,
    }"
  >
    {{ props.msg }}
  </div>
</template>
