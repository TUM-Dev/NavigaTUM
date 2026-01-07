<script setup lang="ts">
import { mdiArrowRight } from "@mdi/js";

const props = defineProps<{
  path?: string;
}>();

const route = useRoute();
const feedback = useFeedback();
const { t } = useI18n({ useScope: "local" });

const currentPath = computed(() => props.path || route.fullPath);
</script>

<template>
  <div class="mx-auto max-w-xl pt-4">
    <img src="../assets/404_navigatum.svg" :alt="t('img_alt')" />
    <div class="flex flex-col items-center gap-1 p-5">
      <h5 class="text-zinc-800 text-lg">{{ t("header") }}</h5>
      <p class="text-md text-zinc-600">{{ t("description") }}</p>
    </div>
    <div class="flex flex-row items-center justify-evenly">
      <div class="flex flex-row gap-4">
        <Btn to="/" variant="primary">
          {{ t("go_home") }}
        </Btn>
        <Btn
          variant="linkButton"
          @click="
            () => {
              feedback.open = true;
              feedback.data = {
                category: 'bug',
                subject: `404 on \`${currentPath}\``,
                body: t('got_here'),
                deletion_requested: false,
              };
            }
          "
        >
          {{ t("call_to_action") }}
          <MdiIcon :path="mdiArrowRight" :size="16" class="mt-0.5" />
        </Btn>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  description: Dies könnte sein, weil wir einen Fehler gemacht haben.
  got_here: "Ich habe diesen Fehler so gefunden:\r\n1. ..."
  header: Die angeforderte Seite wurde nicht gefunden.
  img_alt: Illustration einer männlichen Person, die auf großen '404'-Buchstaben sitzt und einer weiblichen person vor dem TUM Hauptgebäude
  go_home: Zur Startseite
  call_to_action: Feedback geben
en:
  description: This could be because we made a mistake.
  got_here: "I have found the error by:\r\n1. ..."
  header: The requested website could not to be found.
  img_alt: illustration of a female and male person sitting on large '404'-letters in front of the tum main building
  go_home: Go home
  call_to_action: Give Feedback
</i18n>
