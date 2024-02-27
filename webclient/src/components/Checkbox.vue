<script setup lang="ts">
import { ref } from "vue";

const id = ref("checkbox_" + Math.random());
const model = defineModel<boolean>();
</script>

<template>
  <div class="mb-4 flex gap-2">
    <div class="flex h-5 min-w-4 items-center">
      <input
        :id="id"
        v-model="model"
        type="checkbox"
        class="focusable text-tumBlue-600 bg-zinc-100 border-zinc-300 z-40 min-h-4 min-w-4 rounded"
      />
    </div>
    <div class="ps-1">
      <label :for="id" class="text-zinc-900 z-30 text-sm font-medium">
        <slot />
      </label>
      <p v-if="$slots.helptext" class="text-zinc-500 text-xs font-normal">
        <slot name="helptext" />
      </p>
    </div>
  </div>
</template>
