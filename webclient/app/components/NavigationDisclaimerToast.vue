<script setup lang="ts">
defineProps<{ comingFrom: string; selectedTo: string; selectedFrom: string }>();
const feedback = useFeedback();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <Toast id="nav-disclaimer" level="warning">
    {{ t("disclaimer_0") }}:
    <ul class="ms-5 list-outside list-disc">
      <I18nT tag="li" keypath="disclaimer_1">
        <template #route_planning>
          <b class="font-bold">{{ t("disclaimer_1_route_planning") }}</b>
        </template>
        <template #interior_shortcuts>
          <b class="font-bold">{{ t("disclaimer_1_interior_shortcuts") }}</b>
        </template>
      </I18nT>
      <I18nT tag="li" keypath="disclaimer_2">
        <template #transit_routing>
          <b class="font-bold">{{ t("disclaimer_2_transit_routing") }}</b>
        </template>
        <template #defas>
          <a href="https://mobilitaetsplattform.bayern/de/defas" target="_blank" class="text-blue-600 hover:underline"
            >DEFAS</a
          >
        </template>
      </I18nT>
    </ul>
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
  disclaimer_0: Dies ist derzeit in einer Beta-Phase. Die folgenden Punkte sind noch nicht implementiert
  disclaimer_1_route_planning: Routenplanung
  disclaimer_1_interior_shortcuts: Abkürzungen im Innenbereich
  disclaimer_1: "{route_planning} und {interior_shortcuts}. Der Import der CAD-Daten und Implementierung von barrierefreien Routing sind noch nicht abgeschlossen"
  disclaimer_2_transit_routing: Transit-Routing
  disclaimer_2: "{transit_routing}. Wir haben noch keine Möglichkeit gefunden, {defas}-Daten zu beziehen"
  disclaimer_cta: Wir würden wir uns trotzdem über dein Feedback freuen 😊
  open-feedback-form: Öffnet das Feedback-Formular
  found_issues: "Ich habe diese Probleme gefunden:"
  got_here_and_found_issues: "Ich habe die Navigation via {0} gefunden und mir ist dieses Problem aufgefallen:"
en:
  disclaimer_0: This is currently in a beta stage. These are the issues that are currently not implemented
  disclaimer_1_route_planning: Indoor routing
  disclaimer_1_interior_shortcuts: shortcuts
  disclaimer_1: "{route_planning} and {interior_shortcuts}. CAD-data import and accessible routing implementation is not yet done"
  disclaimer_2_transit_routing: Transit routing
  disclaimer_2: "{transit_routing}. We have not found a way to incorporate {defas}-data yet"
  disclaimer_cta: We would still appreciate your feedback 😊
  open-feedback-form: Open the feedback form
  found_issues: "I have found these problems:"
  got_here_and_found_issues: "I found the navigation via {0} and I noticed these problems:"
</i18n>
