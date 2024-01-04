<script setup lang="ts">
import { computed } from "vue";

export interface Props {
  type?: "submit" | "reset" | "button";
  size?: "sm" | "md" | "lg" | string;
  to?: string;
  disabled?: boolean;
  variant?: "action" | "link" | "info" | "primary" | string;
  ariaLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  type: "button",
  size: "md",
  to: "",
  variant: "primary",
  ariaLabel: "",
  disabled: false,
});
const emit = defineEmits(["click"]);
const variantClasses = computed(() => {
  switch (props.variant) {
    case "action":
      return "";
    case "primary":
      return "bg-tumBlue-500 visited:text-tumBlue-50 text-tumBlue-50 hover:bg-tumBlue-600 hover:text-white";
    case "link":
      return "bg-transparent visited:text-tumBlue-600 text-tumBlue-600 hover:bg-tumBlue-50 hover:text-tumBlue-500";
    default:
      return props.variant;
  }
});
const sizeClasses = computed(() => {
  switch (props.size) {
    case "sm":
      return "text-xs font-semibold px-1.5 rounded-md";
    case "md":
      return "text-md px-4 py-1.5 rounded-sm";
    case "lg":
      return "text-lg px-2.5 rounded-md";
    default:
      return props.size;
  }
});
</script>

<template>
  <RouterLink
    v-if="props.to.length"
    :to="props.to"
    :aria-label="ariaLabel"
    :type="props.type"
    v-bind="{ disabled: disabled }"
    :class="`flex flex-row gap-1 !no-underline focusable ${variantClasses} ${sizeClasses}`"
    @click="emit('click')"
  >
    <slot />
  </RouterLink>
  <button
    v-else
    :aria-label="ariaLabel"
    :type="props.type"
    v-bind="{ disabled: disabled }"
    :class="`flex flex-row gap-1 focusable ${variantClasses} ${sizeClasses}`"
    @click="emit('click')"
  >
    <slot />
  </button>
</template>
