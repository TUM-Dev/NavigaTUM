<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from "vue";
import { XMarkIcon } from "@heroicons/vue/24/outline";
import { useI18n } from "vue-i18n";

export interface Props {
  title: string;
  open: boolean;
  disableClose?: boolean;
  classes?: {
    background?: string;
    close?: string;
    modal?: string;
  };
}

const props = withDefaults(defineProps<Props>(), {
  classes: () => ({
    background: "",
    close: "",
    modal: "",
  }),
});
const emit = defineEmits(["close", "update:open"]);

const { t } = useI18n({ useScope: "local" });
watch(props, () => {
  if (props.open) {
    return document.querySelector("body")?.classList.add("overflow-hidden");
  } else {
    return document.querySelector("body")?.classList.remove("overflow-hidden");
  }
});

onMounted(() => {
  if (props.disableClose) return;
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      close();
    }
  });
});
onBeforeUnmount(() => document.querySelector("body")?.classList.remove("overflow-hidden"));

function close() {
  document.querySelector("body")?.classList.remove("overflow-hidden");
  emit("close");
  emit("update:open", false);
}

function closeIfShown() {
  if (props.disableClose) return;
  close();
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
      enter-active-class="transition duration-75"
      leave-active-class="transition duration-75"
    >
      <div
        v-if="props.open"
        class="bg-smoke-800 fixed m-5 flex h-screen inset-0 items-center justify-center w-full z-50"
        :class="props.classes.background"
        @click.self="closeIfShown"
      >
        <div
          class="flex flex-col max-h-screen max-w-2xl relative rounded-md shadow-2xl w-full"
          :class="props.classes.modal"
        >
          <div class="bg-gray-100 flex flex-row justify-between p-5 rounded-t-md w-full">
            <div v-if="props.title" class="text-xl">{{ props.title }}</div>
            <button
              v-if="!props.disableClose"
              type="button"
              :aria-label="t('close')"
              class="mx-4 my-2 text-gray-700 text-xl"
              :class="props.classes.close"
              @click.prevent="close"
            >
              <XMarkIcon class="h-4 w-4" />
            </button>
          </div>
          <div class="bg-white max-h-screen overflow-auto rounded-md p-6 w-full">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<i18n lang="yaml">
de:
  close: Modal schließen
en:
  close: close modal
</i18n>
