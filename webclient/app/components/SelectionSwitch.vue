<script setup lang="ts">
import { Listbox, ListboxButton, ListboxOptions } from "@headlessui/vue";
import { ChevronUpDownIcon } from "@heroicons/vue/24/outline";

interface Props {
  label?: string;
  current: string;
}

const props = withDefaults(defineProps<Props>(), { label: "" });
const selectedValue = defineModel<string>({ required: true });
</script>

<template>
  <div class="flex flex-col">
    <span class="text-sm font-semibold">{{ props.label }}</span>
    <Listbox v-model="selectedValue">
      <div class="relative mt-1">
        <ListboxButton
          class="bg-zinc-200 border-zinc-400 relative w-full cursor-pointer rounded-md border py-2 pl-3 pr-10 text-left shadow-md focus-visible:border-blue-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-white/75 focus-visible:ring-offset-2 focus-visible:ring-offset-blue-300 sm:text-sm"
        >
          <span class="text-zinc-600 block truncate">{{ current }}</span>
          <span class="absolute inset-y-0 right-0 flex items-center pr-2">
            <ChevronUpDownIcon class="text-zinc-600 h-5 w-5" aria-hidden="true" />
          </span>
        </ListboxButton>

        <Transition
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <ListboxOptions
            class="ring-black/5 bg-zinc-200 absolute z-30 mt-1 max-h-60 w-full overflow-auto rounded-md py-1 shadow-lg ring-1 focus:outline-none sm:text-sm"
          >
            <slot />
          </ListboxOptions>
        </Transition>
      </div>
    </Listbox>
  </div>
</template>
