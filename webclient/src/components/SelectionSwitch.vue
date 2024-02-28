<script setup lang="ts">
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from "@headlessui/vue";
import { CheckIcon, ChevronUpDownIcon } from "@heroicons/vue/24/outline";

interface Props {
  label?: string;
  values: [string, string];
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
          class="bg-zinc-400 relative w-full cursor-pointer rounded-lg py-2 pl-3 pr-10 text-left shadow-md focus-visible:border-tumBlue-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-white/75 focus-visible:ring-offset-2 focus-visible:ring-offset-tumBlue-300 sm:text-sm"
        >
          <span class="text-zinc-600 block truncate">{{ selectedValue }}</span>
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
            class="bg-zinc-400 absolute z-30 mt-1 max-h-60 w-full overflow-auto rounded-md py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
          >
            <ListboxOption v-slot="{ active, selected }" :value="values[0]" as="template">
              <li
                class="relative cursor-pointer select-none py-2 pl-10 pr-4"
                :class="[active ? 'text-tumBlue-900 bg-tumBlue-100' : 'text-zinc-900']"
              >
                <span class="block truncate" :class="[selected ? 'font-medium' : 'font-normal']">
                  <slot name="option1" />
                </span>
                <span v-if="selected" class="text-tumBlue-600 absolute inset-y-0 left-0 flex items-center pl-3">
                  <CheckIcon class="h-5 w-5" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
            <ListboxOption v-slot="{ active, selected }" :value="values[1]" as="template">
              <li
                class="relative cursor-pointer select-none py-2 pl-10 pr-4"
                :class="[active ? 'text-tumBlue-900 bg-tumBlue-100' : 'text-zinc-900']"
              >
                <span class="block truncate" :class="[selected ? 'font-medium' : 'font-normal']">
                  <slot name="option2" />
                </span>
                <span v-if="selected" class="text-tumBlue-600 absolute inset-y-0 left-0 flex items-center pl-3">
                  <CheckIcon class="h-5 w-5" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
          </ListboxOptions>
        </Transition>
      </div>
    </Listbox>
  </div>
</template>
