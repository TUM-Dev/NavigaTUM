<script setup lang="ts">
import { useFeedback } from "~/composables/feedback";

const { t } = useI18n({ useScope: "local" });
const deleteIssueRequested = ref(false);
const feedback = useFeedback();
</script>

<template>
  <TokenBasedModal :data="feedback.data">
    <template #modal>
      <label class="text-zinc-600 text-sm font-semibold" for="feedback-subject"> {{ t("subject") }}</label>
      <div class="text-zinc-600 flex flex-row gap-2 pb-3">
        <select
          id="feedback-category"
          v-model="feedback.data.category"
          class="focusable bg-zinc-200 border-zinc-400 rounded border px-2 py-1"
          :aria-label="t('category')"
        >
          <option value="general">{{ t("type.general") }}</option>
          <option value="bug">{{ t("type.bug") }}</option>
          <option value="feature">{{ t("type.feature") }}</option>
          <option value="search">{{ t("type.search") }}</option>
          <option value="entry">{{ t("type.entry") }}</option>
        </select>
        <input
          id="feedback-subject"
          v-model="feedback.data.subject"
          class="focusable bg-zinc-200 border-zinc-400 flex-grow rounded border px-2 py-1"
          type="text"
          :placeholder="t('subject')"
        />
      </div>

      <div class="flex flex-col pb-5">
        <label class="text-zinc-600 text-sm font-semibold" for="feedback-body">
          {{ t("message") }}
        </label>
        <textarea
          id="feedback-body"
          v-model="feedback.data.body"
          class="focusable bg-zinc-200 border-zinc-400 resize-y rounded border px-2 py-1"
          :placeholder="t('message')"
          rows="6"
        />
        <p class="text-zinc-500 text-xs">
          {{
            {
              general: t("helptext.general"),
              bug: t("helptext.bug"),
              feature: t("helptext.feature"),
              search: t("helptext.search"),
              entry: t("helptext.entry"),
              other: t("helptext.other"), // This is only here to make the linter happy, backend uses "other" as a fallback if the category is not known
            }[feedback.data.category]
          }}
        </p>
      </div>

      <Checkbox id="delete-issue" v-model="deleteIssueRequested">{{ t("delete") }}</Checkbox>
    </template>
    <template #success="{ successUrl }">
      <p>{{ t("success.thank_you") }}</p>
      <p>
        {{ t("success.response_at") }}
        <Btn variant="linkButton" :href="successUrl">{{ t("success.this_issue") }}</Btn>
      </p>
    </template>
  </TokenBasedModal>
</template>

<i18n lang="yaml">
de:
  category: Feedback-Kategorie
  delete: Das zugehörige GitHub Issue löschen, sobald es gelöst wurde.
  helptext:
    bug: Welchen Fehler hast du gefunden? Wo hast du ihn gefunden? Bitte gib eine genaue Beschreibung an.
    entry: Feedback zu einem Eintrag. Wir können Räume/Gebäude/Standorte hinzufügen und alle Daten, die du siehst (Namen, Koordinaten, Adressen, ...) anpassen. Was können wir verbessern?
    feature: Features, die du gerne auf dieser Website haben würdest
    general: Generelles Feedback über diese Website
    other: "Feedback ist auf ein Problem gestoßen: Kategorie ungültig"
    search: Feedback zur Suche. Was war dein Suchbegriff? Was hättest du als Ergebnis erwartet?
  message: Nachricht
  subject: Betreff
  success:
    response_at: Antwort auf dein Feedback findest du auf
    thank_you: Vielen Dank für dein Feedback! Wir werden es schnellstmöglich bearbeiten.
    this_issue: diesem GitHub Issue
  type:
    bug: Fehler
    entry: Eintrag
    feature: Features
    general: Allgemein
    search: Suche
en:
  category: Feedback category
  delete: Delete this GitHub issue when resolved.
  helptext:
    bug: Which bug did you find? Where did you find it? Please provide a detailed description.
    entry: Feedback about an entry. We can add rooms/buildings/locations and adjust all data you see (names, coordinates, addresses, ...). What can we improve?
    feature: Features you would like to see on this website
    general: General Feedback about this website
    other: "Feedback encountered issue: Category invalid"
    search: Feedback about the search. What was your search query? What did you expect to see?
  message: Message
  subject: Subject
  success:
    response_at: You can see our response at
    thank_you: Thank you for giving your feedback. We will work on this as soon as possible.
    this_issue: this GitHub issue
  type:
    bug: Bug
    entry: Entry
    feature: Features
    general: General
    search: Search
</i18n>
