<script setup lang="ts">
defineProps<{ comingFrom: string; selectedTo: string; selectedFrom: string }>();
const feedback = useFeedback();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <Toast id="nav-disclaimer" level="warning">
    <I18nT keypath="disclaimer">
      <template #route_planning>
        <b class="font-bold">{{ t("disclaimer_route_planning") }}</b>
      </template>
      <template #barrier_free_routing>
        <b class="font-bold">{{ t("disclaimer_barrier_free_routing") }}</b>
      </template>
    </I18nT>
    <Btn
      variant="link"
      :aria-label="t('open-feedback-form')"
      :title="t('open-feedback-form')"
      @click="
        () => {
          feedback.open = true;
          feedback.data = {
            category: 'navigation',
            subject: `navigation from \`${selectedFrom}\` to \`${selectedTo}\``,
            body: !!comingFrom ? t('got_here_and_found_issues', [comingFrom]) : t('found_issues'),
            deletion_requested: false,
          };
        }
      "
    >
      {{ t("disclaimer_cta") }}
    </Btn>
  </Toast>
</template>

<i18n lang="yaml">
de:
  disclaimer: "Beta-Phase: Noch nicht implementiert ist {route_planning} und {barrier_free_routing}."
  disclaimer_route_planning: indoor
  disclaimer_barrier_free_routing: barrierefreies Routing
  disclaimer_cta: Wir würden uns trotzdem über dein Feedback freuen 😊
  open-feedback-form: Öffnet das Feedback-Formular
  found_issues: "Ich habe diese Probleme gefunden:"
  got_here_and_found_issues: "Ich habe die Navigation via {0} gefunden und mir ist dieses Problem aufgefallen:"
en:
  disclaimer: "Beta stage: Not implemented is {route_planning} and {barrier_free_routing}."
  disclaimer_route_planning: indoor
  disclaimer_barrier_free_routing: barrier-free routing
  disclaimer_cta: We would still appreciate your feedback 😊
  open-feedback-form: Open the feedback form
  found_issues: "I have found these problems:"
  got_here_and_found_issues: "I found the navigation via {0} and I noticed these problems:"
</i18n>
