<script setup lang="ts">
import { computed } from "vue";

export interface Props {
  type?: "submit" | "reset" | "button";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  variant?: "action" | "base" | "info" | "primary";
  ariaLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  type: "button",
  size: "md",
  variant: "base",
  ariaLabel: "",
  disabled: false,
});
const emit = defineEmits(["click"]);
const variantClasses = computed(() => {
  switch (props.variant) {
    case "action":
      return "";
    case "primary":
      return "bg-tumBlue-500 text-tumBlue-50 hover:bg-tumBlue-600 hover:text-white";
    case "base":
      return "";
    default:
      return "";
  }
});
const sizeClasses = computed(() => {
  switch (props.size) {
    case "sm":
      return "text-md px-1.5 rounded-xs";
    case "md":
      return "text-md px-1.5 rounded-sm";
    case "lg":
      return "text-lg px-2.5 rounded-md";
    default:
      return "";
  }
});
</script>

<template>
  <button
    :aria-label="ariaLabel"
    :type="props.type"
    v-bind="{ disabled: disabled }"
    :class="`${variantClasses} ${sizeClasses}`"
    @click="emit('click')"
  >
    <slot />
  </button>
</template>
