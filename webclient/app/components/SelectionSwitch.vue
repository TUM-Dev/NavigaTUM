<script setup lang="ts">
import { Listbox, ListboxButton, ListboxOptions } from "@headlessui/vue";
import { mdiUnfoldMoreHorizontal } from "@mdi/js";

interface Props {
  label?: string;
}
const model = defineModel<string>({ required: true });

const props = withDefaults(defineProps<Props>(), { label: "" });
</script>

<template>
  <div class="flex flex-col">
    <span class="text-sm font-semibold">{{ props.label }}</span>
    <Listbox v-model="model">
      <div class="relative mt-1">
        <ListboxButton
          class="bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 relative w-full cursor-pointer rounded-md border py-2 pl-3 pr-10 text-left shadow-md focus-visible:border-blue-500 dark:focus-visible:border-blue-400 focus:outline-none focus-visible:ring-2 focus-visible:ring-white/75 dark:focus-visible:ring-black/75 focus-visible:ring-offset-2 focus-visible:ring-offset-blue-300 dark:focus-visible:ring-offset-blue-600 sm:text-sm"
        >
          <span class="text-zinc-600 dark:text-zinc-300 block truncate">{{ model }}</span>
          <span class="absolute inset-y-0 right-0 flex items-center pr-2">
            <MdiIcon :path="mdiUnfoldMoreHorizontal" :size="20" class="text-zinc-600 dark:text-zinc-300" aria-hidden="true" />
          </span>
        </ListboxButton>

        <Transition leave-active-class="transition duration-100 ease-in" leave-from-class="opacity-100" leave-to-class="opacity-0">
          <ListboxOptions
            class="ring-black/5 dark:ring-white/5 bg-zinc-200 dark:bg-zinc-700 absolute z-30 mt-1 max-h-60 w-full overflow-auto rounded-md py-1 shadow-lg ring-1 focus:outline-none sm:text-sm"
          >
            <slot />
          </ListboxOptions>
        </Transition>
      </div>
    </Listbox>
  </div>
</template>
