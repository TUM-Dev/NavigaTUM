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
        <template #cta>
          <b class="font-bold">{{ t("disclaimer_1_cta") }}</b>
        </template>
      </I18nT>
      <I18nT tag="li" keypath="disclaimer_2">
        <template #cta1>
          <b class="font-bold">{{ t("disclaimer_2_cta1") }}</b>
        </template>
        <template #cta2>
          <b class="font-bold">{{ t("disclaimer_2_cta2") }}</b>
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
  disclaimer_1_cta: Transit-Routing
  disclaimer_1: "{cta}. Wir haben noch keine Möglichkeit gefunden haben, Daten von DEFAS einzubeziehen"
  disclaimer_2_cta1: Routenplanung
  disclaimer_2_cta2: Abkürzungen im Innenbereich
  disclaimer_2: "{cta1} und {cta2}. Der Import der CAD-Daten und die Implementierung des barrierefreien Routings noch nicht abgeschlossen sind"
  disclaimer_3: Wegen der Nichtberücksichtigung von {cta} könnten die Routen suboptimal sein
  disclaimer_cta: Wir würden wir uns trotzdem über dein feedback freuen
  open-feedback-form: Öffnet das Feedback-Formular
  found_issues: "Ich habe diese Probleme gefunden:"
  got_here_and_found_issues: "Ich habe die Navigation via {0} gefunden und mir ist dieses Problem aufgefallen:"
en:
  disclaimer_0: This is currently in a beta stage. These are the issues that are currently not implemented
  disclaimer_1_cta: Transit routing
  disclaimer_1: "{cta} as we have not found a way to incorporate DEFAS data yet"
  disclaimer_2_cta1: Indoor routing
  disclaimer_2_cta2: shortcuts
  disclaimer_2: "{cta1} and {cta1}. CAD-data import and accessible routing implementation is not yet done"
  disclaimer_cta: We would still appreciate your feedback
  open-feedback-form: Open the feedback form
  found_issues: "I have found these problems:"
  got_here_and_found_issues: "I found the navigation via {0} and I noticed these problems:"
</i18n>
