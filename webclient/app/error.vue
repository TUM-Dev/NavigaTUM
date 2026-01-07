<script setup lang="ts">
import { mdiArrowRight } from "@mdi/js";

const props = defineProps({
  error: Object,
});

const is404 = computed(() => props.error?.statusCode === 404);

const route = useRoute();
const feedback = useFeedback();
const { t } = useI18n({ useScope: "local" });

const currentPath = computed(() => error?.url || route.fullPath);
</script>

<template>
  <NuxtLayout>
    <div class="mx-auto max-w-xl pt-4" v-if="is404">
      <img src="./assets/404_navigatum.svg" :alt="t('img_alt')" />
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
    <div v-else class="mx-auto max-w-xl pt-4">
      <div class="flex flex-col items-center gap-4 p-5">
        <h5 class="text-zinc-800 text-xl font-bold">{{ error?.statusCode || "Error" }}</h5>
        <p class="text-md text-zinc-600">{{ error?.statusMessage || "An error occurred" }}</p>
        <p v-if="error?.message" class="text-sm text-zinc-500 mt-2">
          {{ error.message }}
        </p>
        <Btn @click="clearError({ redirect: '/' })" variant="primary" class="mt-4"> Go home </Btn>
      </div>
    </div>
  </NuxtLayout>
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
