<script setup lang="ts">
import { Switch, SwitchGroup, SwitchLabel } from "@headlessui/vue";
import { computed } from "vue";

interface Props {
  label?: string;
  values: [string, string];
}
const props = withDefaults(defineProps<Props>(), { label: "" });
const selected = defineModel<string>({ required: true });

const firstValueSelected = computed(() => selected.value === props.values[0]);
</script>

<template>
  <SwitchGroup>
    <div class="mt-2 flex flex-row justify-between">
      <SwitchLabel v-if="props.label.length">{{ props.label }}</SwitchLabel>
      <Switch
        class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent text-slate-400 transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-tumBlue-500 focus:ring-offset-2"
        :class="[firstValueSelected ? 'bg-slate-200' : 'bg-tumBlue-500']"
        @update:model-value="(val: boolean) => (selected = props.values[val ? 1 : 0])"
      >
        <span class="sr-only">Use setting</span>
        <span
          class="pointer-events-none relative inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
          :class="[firstValueSelected ? 'translate-x-0' : 'translate-x-5']"
        >
          <span
            class="absolute inset-0 flex h-full w-full items-center justify-center transition-opacity"
            :class="[firstValueSelected ? 'opacity-100 duration-200 ease-in' : 'opacity-0 duration-100 ease-out']"
            aria-hidden="true"
          >
            <slot name="option1" />
          </span>
          <span
            class="absolute inset-0 flex h-full w-full items-center justify-center transition-opacity"
            :class="[firstValueSelected ? 'opacity-0 duration-100 ease-out' : 'opacity-100 duration-200 ease-in']"
            aria-hidden="true"
          >
            <slot name="option2" />
          </span>
        </span>
      </Switch>
    </div>
  </SwitchGroup>
</template>
