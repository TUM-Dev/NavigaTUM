<script setup lang="ts">
withDefaults(
  defineProps<{
    submitting: boolean;
    blocked: boolean;
    disabled?: boolean;
    /** Overrides the default "send" copy, e.g. when the submission updates an existing event. */
    label?: string;
  }>(),
  { disabled: false, label: undefined }
);

defineEmits<{ click: [] }>();

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <Btn
    variant="primary"
    size="md"
    :class="{
      '!text-blue-900 dark:!text-blue-50 !bg-blue-200 dark:!bg-blue-700 cursor-progress': submitting,
      '!text-blue-50 dark:!text-blue-900 !bg-blue-300 dark:!bg-blue-600 cursor-not-allowed': blocked,
    }"
    :disabled="disabled || submitting || blocked"
    @click="$emit('click')"
  >
    <template v-if="submitting">
      <Spinner class="my-auto h-4 w-4" />
      {{ t("sending") }}...
    </template>
    <template v-else-if="blocked">{{ t("try_again_later") }}</template>
    <template v-else>{{ label ?? t("send") }}</template>
  </Btn>
</template>

<i18n lang="yaml">
de:
  send: Senden
  sending: Wird gesendet
  try_again_later: Bitte versuche es später noch einmal
en:
  send: Send
  sending: Sending
  try_again_later: Please try again later
</i18n>
