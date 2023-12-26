<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
const route = useRoute();
import { ArrowRightIcon } from "@heroicons/vue/24/outline";
import Button from "@/components/Button.vue";
const global = useGlobalStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div class="mt-4 max-w-xl mx-auto">
    <img class="" src="@/assets/404_navigatum.svg" :alt="t('img_alt')" />
    <div>
      <h5 class="mt-5">{{ t("header") }}</h5>
      <p>{{ t("description") }}</p>
      <div class="flex flex-row gap-4">
        <Button to="/" variant="primary">
          {{ t("go_home") }}
        </Button>
        <Button variant="link" @click="global.openFeedback('bug', `404 on \`${route.fullPath}\``, t('got_here'))">
          {{ t("call_to_action") }} <ArrowRightIcon class="h-4 w-4 mt-0.5" />
        </Button>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  open_feedback: Open the feedback-form
  description: Dies könnte sein, weil wir einen Fehler gemacht haben.
  got_here: "Ich habe diesen Fehler so gefunden:\r\n1. ..."
  header: Die angeforderte Seite wurde nicht gefunden.
  img_alt: Illustration einer männlichen Person, die auf großen '404'-Buchstaben sitzt und einer weiblichen person vor dem TUM Hauptgebäude
  go_home: Zurück gehen
  call_to_action: Feedback geben
en:
  open_feedback: Open the feedback-form
  description: This could be because we made a mistake.
  got_here: "I have found the error by:\r\n1. ..."
  header: The requested website could not to be found.
  img_alt: illustration of a female and male person sitting on large '404'-letters in front of the tum main building
  go_home: Go back home
  call_to_action: Give Feedback
</i18n>
