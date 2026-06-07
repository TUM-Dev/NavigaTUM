<script setup lang="ts">
const model = defineModel<boolean>();
defineProps<{ id: string }>();
</script>

<template>
  <div class="mb-4 flex gap-2">
    <div class="flex h-5 min-w-4 items-center rounded">
      <input
        :id="id"
        v-model="model"
        type="checkbox"
        class="focusable bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 z-40 min-h-4 min-w-4 accent-blue-400 dark:accent-blue-500"
      />
    </div>
    <div class="ps-1">
      <label :for="id" class="text-zinc-900 dark:text-zinc-50 z-30 text-sm font-medium">
        <slot />
      </label>
      <p v-if="$slots.helptext" class="text-zinc-500 dark:text-zinc-400 text-xs font-normal">
        <slot name="helptext" />
      </p>
    </div>
  </div>
</template>
