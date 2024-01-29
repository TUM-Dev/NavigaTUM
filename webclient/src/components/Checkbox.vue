<script setup lang="ts">
import { ref } from "vue";

const id = ref("checkbox_" + Math.random());
const model = defineModel<boolean>();
</script>

<template>
  <div class="mb-4 flex gap-2">
    <div class="flex h-5 items-center">
      <input
        :id="id"
        v-model="model"
        type="checkbox"
        class="focusable z-40 min-h-4 min-w-4 rounded border-zinc-300 bg-zinc-100 text-tumBlue-600 dark:border-zinc-600 dark:bg-zinc-700"
      />
    </div>
    <div class="ms-2">
      <label :for="id" class="z-30 text-sm font-medium text-zinc-900 dark:text-zinc-300">
        <slot />
      </label>
      <p v-if="$slots.helptext" class="text-xs font-normal text-gray-500 dark:text-gray-300">
        <slot name="helptext" />
      </p>
    </div>
  </div>
</template>
