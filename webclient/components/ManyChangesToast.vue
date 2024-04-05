<script setup lang="ts">
import { useFeedback } from "~/composables/feedback";

const { t } = useI18n({ useScope: "local" });
const feedback = useFeedback();
</script>

<template>
  <Toast level="info">
    {{ t("toast.released_many_changes") }}
    <Btn
      variant="link"
      size="ms-0 rounded-sm text-start pt-1.5"
      :aria-label="t('toast.open')"
      @click="
        () => {
          feedback.open = true;
          feedback.data = {
            category: 'general',
            subject: t('toast.feedback_subject'),
            body: t('toast.feedback_body'),
            deletion_requested: false,
          };
        }
      "
    >
      {{ t("toast.call_to_action") }}
    </Btn>
  </Toast>
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
