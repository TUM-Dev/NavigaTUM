<script setup lang="ts">
import { ref } from "vue";

const id = ref("checkbox_" + Math.random());
const model = defineModel<boolean>();
</script>

<template>
  <div class="flex mb-4 gap-2">
    <div class="flex items-center h-5">
      <input
        :id="id"
        v-model="model"
        type="checkbox"
        class="min-w-4 min-h-4 text-tumBlue-600 z-40 bg-zinc-100 border-zinc-300 rounded focusable dark:bg-zinc-700 dark:border-zinc-600"
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
