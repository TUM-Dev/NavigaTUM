<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";

const route = useRoute();
const global = useGlobalStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <Toast level="warning" :msg="t('error_404')" />
  <h5 style="margin-top: 25px">{{ t("header") }}</h5>
  <p>{{ t("description") }}</p>
  <button
    type="button"
    class="btn btn-link p-0"
    :aria-label="t('open_feedback')"
    @click="global.openFeedback('bug', `404 on ${route.fullPath}`, t('got_here'))"
  >
    {{ t("call_to_action") }}
  </button>
</template>

<i18n lang="yaml">
de:
  error_404: 404 Nicht gefunden
  call_to_action: Falls du denkst, dass dies ein Fehler ist, teile es uns doch hier mit
  description: Dies könnte sein, weil wir einen Fehler gemacht haben.
  got_here: "Ich habe diesen Fehler so gefunden:\r\n1. ..."
  header: Die angeforderte Seite wurde nicht gefunden.
  open_feedback: Open the feedback-form
en:
  error_404: 404 Not found
  call_to_action: If you think this is a mistake, please let us know here
  description: This could be because we made a mistake.
  got_here: "I have found the error by:\r\n1. ..."
  header: The requested website could not to be found.
  open_feedback: Open the feedback-form
</i18n>
