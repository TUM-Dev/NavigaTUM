<script setup lang="ts">
import { ListboxOption } from "@headlessui/vue";
import { mdiCheck } from "@mdi/js";

defineProps<{ value: string }>();
</script>

<template>
  <ListboxOption v-slot="{ active, selected }" :value="value" as="template">
    <li
      class="relative cursor-pointer select-none py-2 pl-4 pr-4"
      :class="[active ? 'text-blue-900 bg-blue-100' : 'text-zinc-900']"
    >
      <span class="flex flex-row gap-1.5 truncate text-sm" :class="[selected ? 'font-bold' : 'font-normal']">
        <slot />
      </span>
      <span v-if="selected" class="text-blue-600 absolute inset-y-0 right-0 flex items-center pr-3">
        <MdiIcon :path="mdiCheck" :size="20" aria-hidden="true" />
      </span>
    </li>
  </ListboxOption>
</template>
