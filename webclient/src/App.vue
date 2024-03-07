<script setup lang="ts">
import Footer from "@/components/AppFooter.vue";
import { useGlobalStore } from "@/stores/global";
import FeedbackModal from "@/components/feedback/FeedbackModal.vue";
import AppSearchBar from "@/components/AppSearchBar.vue";
import AppNavHeader from "@/components/AppNavHeader.vue";
import Btn from "@/components/Btn.vue";
import Toast from "@/components/Toast.vue";
import { useI18n } from "vue-i18n";
const global = useGlobalStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <AppNavHeader>
    <AppSearchBar />
  </AppNavHeader>

  <!-- Page content container -->
  <div
    class="mx-auto mb-16 mt-16 min-h-[calc(100vh-400px)] max-w-4xl transition-opacity"
    :class="{ 'opacity-70': global.search_focused }"
  >
    <div class="mx-5 -mb-1 flex flex-col gap-4 pt-5">
      <Toast level="info">
        {{ t("toast.released_many_changes") }}
        <Btn
          variant="link"
          size="ms-0 rounded-sm text-start pt-1.5"
          :aria-label="t('toast.open')"
          @click="global.openFeedback('general', t('toast.feedback_subject'), t('toast.feedback_body'))"
        >
          {{ t("toast.call_to_action") }}
        </Btn>
      </Toast>
      <Toast v-if="global.error_message" :msg="global.error_message" level="error" />
    </div>
    <div class="mx-5">
      <RouterView />
    </div>
  </div>

  <Footer />
  <FeedbackModal v-if="global.feedback.open" />
</template>

<i18n lang="yaml">
de:
  toast:
    released_many_changes: Wir haben vor ein paar Tagen eine neue Version unseres Frontends mit einer Vielzahl von Änderungen veröffentlicht.
    feedback_subject: Feedback zum neuen Frontend
    feedback_body: |
      Es gefällt mir, dass:
      - Detail 1
      - Einzelheit 2

      Ich denke, das sollte verbessert werden:
      - Verbesserung 1
      - Verbesserung 2
    call_to_action: Gibt es etwas, das du nicht gut findest? Erzähle uns bitte davon!
    open: Feedback Form öffnen
en:
  toast:
    released_many_changes: We have recently released a new version of our frontend with a ton of changes.
    feedback_subject: Feedback about new Frontend
    feedback_body: |
      I like:
      - detail 1
      - detail 2

      I think this should be improved:
      - improvement 1
      - improvement 2
    call_to_action: Is there something you don't like? Please tell us about it!
    open: Open the feedback-form
</i18n>
