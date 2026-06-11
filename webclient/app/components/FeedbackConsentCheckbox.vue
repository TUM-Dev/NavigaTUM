<script setup lang="ts">
type ConsentKind = "proposal" | "feedback";

withDefaults(
  defineProps<{
    id?: string;
    kind?: ConsentKind;
  }>(),
  { id: "feedback-privacy-checked", kind: "proposal" }
);

const checked = defineModel<boolean>({ required: true });
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <Checkbox :id="id" v-model="checked">
    <I18nT tag="span" :keypath="`consent.${kind}`">
      <template #github>
        <NuxtLink
          tabindex="1"
          class="text-blue-600 dark:text-blue-300 visited:text-blue-600 dark:visited:text-blue-300 hover:underline"
          to="https://github.com/TUM-Dev/NavigaTUM"
          target="_blank"
          external
        >
          GitHub
        </NuxtLink>
      </template>
      <template #privacy_policy>
        <NuxtLink
          tabindex="1"
          :to="t('privacy_policy_url')"
          class="text-blue-600 dark:text-blue-300 visited:text-blue-600 dark:visited:text-blue-300 hover:underline"
        >
          {{ t("privacy_policy") }}
        </NuxtLink>
      </template>
    </I18nT>
  </Checkbox>
</template>

<i18n lang="yaml">
de:
  consent:
    proposal: Ich verstehe, dass mein Vorschlag auf {github} veröffentlicht und gemäß der {privacy_policy} verarbeitet wird. Mit dem Absenden dieses Formulars willige ich in diese Verarbeitung ein.
    feedback: Ich verstehe, dass mein Feedback auf {github} veröffentlicht und gemäß der {privacy_policy} verarbeitet wird. Mit dem Absenden dieses Formulars willige ich in diese Verarbeitung ein.
  privacy_policy: Datenschutzerklärung
  privacy_policy_url: /about/datenschutz
en:
  consent:
    proposal: I understand that my proposal will be published on {github} and processed in accordance with the {privacy_policy}. By submitting this form, I consent to this processing.
    feedback: I understand that my feedback will be published on {github} and processed in accordance with the {privacy_policy}. By submitting this form, I consent to this processing.
  privacy_policy: Privacy Policy
  privacy_policy_url: /en/about/privacy
</i18n>
