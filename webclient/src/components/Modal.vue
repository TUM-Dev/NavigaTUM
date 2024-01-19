<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from "vue";
import { XMarkIcon } from "@heroicons/vue/24/outline";
import { useI18n } from "vue-i18n";

export interface Props {
  title: string;
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
const emit = defineEmits(["close"]);
const isOpen = defineModel<boolean>({ required: true });

const { t } = useI18n({ useScope: "local" });
watch(props, () => {
  if (isOpen) {
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
  isOpen.value = false;
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
        v-if="isOpen"
        class="bg-smoke-800 fixed inset-0 z-50 m-5 flex h-screen w-full items-center justify-center"
        :class="props.classes.background"
        @click.self="closeIfShown"
      >
        <div
          class="relative flex max-h-screen w-full max-w-2xl flex-col rounded-md shadow-2xl"
          :class="props.classes.modal"
        >
          <div class="flex w-full flex-row justify-between rounded-t-md bg-gray-100 p-5">
            <div v-if="props.title" class="text-xl">{{ props.title }}</div>
            <button
              v-if="!props.disableClose"
              type="button"
              :aria-label="t('close')"
              class="mx-4 my-2 text-xl text-gray-700"
              :class="props.classes.close"
              @click.prevent="close"
            >
              <XMarkIcon class="h-4 w-4" />
            </button>
          </div>
          <div class="max-h-screen w-full overflow-auto rounded-b-md bg-white p-6">
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
