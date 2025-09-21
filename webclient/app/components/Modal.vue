<script setup lang="ts">
import { mdiClose } from "@mdi/js";

export interface Props {
  title: string;
  disableClose?: boolean;
  class?: string;
}

const props = withDefaults(defineProps<Props>(), {
  class: "",
});
const emit = defineEmits(["close"]);
const isOpen = defineModel<boolean>({ required: true });

const { t } = useI18n({ useScope: "local" });

// Handle body overflow only on client side
watchEffect(() => {
  if (import.meta.client && document) {
    if (isOpen.value) {
      document.body?.classList.add("overflow-y-hidden");
    } else {
      document.body?.classList.remove("overflow-y-hidden");
    }
  }
});

onMounted(() => {
  const handleEscape = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      close();
    }
  };

  document.addEventListener("keydown", handleEscape);

  // Cleanup function
  onBeforeUnmount(() => {
    document.removeEventListener("keydown", handleEscape);
    document.body?.classList.remove("overflow-y-hidden");
  });
});

function close() {
  if (props.disableClose) return;
  if (import.meta.client && document) {
    document.body?.classList.remove("overflow-y-hidden");
  }
  emit("close");
  isOpen.value = false;
}
</script>

<template>
  <ClientOnly>
    <Teleport to="body">
      <Transition
        enter-from-class="opacity-0"
        leave-to-class="opacity-0"
        enter-active-class="transition duration-100"
        leave-active-class="transition duration-100"
      >
        <div
          v-if="isOpen"
          class="fixed inset-0 z-50 flex h-screen w-full items-center justify-center backdrop-blur-sm backdrop-brightness-95"
          @click.self="close"
        >
          <div class="relative flex max-h-screen w-full max-w-2xl flex-col rounded-md shadow-2xl" :class="props.class">
            <div class="bg-zinc-200 flex w-full flex-row justify-between rounded-t-md p-5">
              <h2 v-if="props.title" class="text-zinc-800 text-lg font-semibold">{{ props.title }}</h2>
              <button
                v-if="!props.disableClose"
                type="button"
                :aria-label="t('close')"
                class="focusable text-zinc-800 mx-4 my-2 text-xl"
                @click.prevent="close"
              >
                <MdiIcon :path="mdiClose" :size="16" />
              </button>
            </div>
            <div class="bg-white text-zinc-600 max-h-screen w-full overflow-auto rounded-b-md p-6 dark:bg-zinc-100">
              <slot />
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </ClientOnly>
</template>

<i18n lang="yaml">
de:
  close: Modal schließen
en:
  close: close modal
</i18n>
