<script setup lang="ts" generic="T extends string | boolean">
import { CheckIcon } from "@heroicons/vue/24/outline";
import { ListboxOption } from "@headlessui/vue";

defineProps<{ value: T }>();
</script>

<template>
  <ListboxOption v-slot="{ active, selected }" :value="value" as="template">
    <li
      class="relative cursor-pointer select-none py-2 pl-4 pr-4"
      :class="[active ? 'text-tumBlue-900 bg-tumBlue-100' : 'text-zinc-900']"
    >
      <span class="flex flex-row gap-1.5 text-sm truncate" :class="[selected ? 'font-medium' : 'font-normal']">
        <slot />
      </span>
      <span v-if="selected" class="text-tumBlue-600 absolute inset-y-0 right-0 flex items-center pr-3">
        <CheckIcon class="h-5 w-5" aria-hidden="true" />
      </span>
    </li>
  </ListboxOption>
</template>
