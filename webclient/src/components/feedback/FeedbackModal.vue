<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import TokenBasedModal from "@/components/feedback/TokenBasedModal.vue";

const { t } = useI18n({ useScope: "local" });
const global = useGlobalStore();
const deleteIssueRequested = ref(false);
</script>

<template>
  <TokenBasedModal :data="global.feedback.data">
    <template v-slot:modal>
      <div class="form-group">
        <label class="form-label" for="feedback-subject"> {{ t("subject") }}</label>
        <div class="input-group">
          <select
            class="form-select"
            id="feedback-category"
            :aria-label="t('category')"
            v-model="global.feedback.data.category"
          >
            <option value="general">{{ t("type.general") }}</option>
            <option value="bug">{{ t("type.bug") }}</option>
            <option value="features">{{ t("type.features") }}</option>
            <option value="search">{{ t("type.search") }}</option>
            <option value="entry">{{ t("type.entry") }}</option>
          </select>
          <input
            class="form-input"
            type="text"
            :placeholder="t('subject')"
            v-model="global.feedback.data.subject"
            id="feedback-subject"
          />
        </div>
      </div>

      <div class="form-group">
        <label class="form-label" for="feedback-body">
          {{ t("message") }}
        </label>
        <textarea
          class="form-input"
          id="feedback-body"
          :placeholder="t('message')"
          v-model="global.feedback.data.body"
          rows="6"
        >
        </textarea>
        <p class="text-gray text-tiny">
          {{
            {
              general: t("helptext.general"),
              bug: t("helptext.bug"),
              feature: t("helptext.features"),
              search: t("helptext.search"),
              entry: t("helptext.entry"),
              other: t("helptext.other"), // This is only here to make the linter happy, backend uses "other" as a fallback if the category is not known
            }[global.feedback.data.category]
          }}
        </p>
      </div>

      <div class="form-group">
        <label class="form-checkbox" id="feedback-delete-label">
          <input type="checkbox" id="feedback-delete" v-model="deleteIssueRequested" />
          <i class="form-icon" /> {{ t("delete") }}
        </label>
      </div>
    </template>
    <template v-slot:success="{ successUrl }">
      <p>{{ t("success.thank_you") }}</p>
      <p>
        {{ t("success.response_at") }}
        <a id="feedback-success-url" class="btn-link" :href="successUrl">{{ t("success.this_issue") }}</a>
      </p>
    </template>
  </TokenBasedModal>
</template>

<style lang="scss" scoped>
@import "@/assets/variables";

.modal {
  label {
    width: fit-content;
    display: inline-block;
  }

  .form-select {
    flex: none;
  }

  #feedback-body {
    min-width: 100%;
  }
}
</style>
<i18n>

</i18n>
<i18n>
de:
  category: Feedback-Kategorie
  delete: Das zugehörige GitHub Issue löschen, sobald es gelöst wurde.
  helptext:
    bug: Welchen Fehler hast du gefunden? Wo hast du ihn gefunden? Bitte gib eine genaue Beschreibung an.
    entry: Feedback zu einem Eintrag. Wir können Räume/Gebäude/Standorte hinzufügen und alle Daten, die du siehst (Namen, Koordinaten, Adressen, ...) anpassen. Was können wir verbessern?
    features: Features, die du gerne auf dieser Website haben würdest
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
    features: Features
    general: Allgemein
    search: Suche
en:
  category: Feedback category
  delete: Delete this GitHub issue when resolved.
  helptext:
    bug: Which bug did you find? Where did you find it? Please provide a detailed description.
    entry: Feedback about an entry. We can add rooms/buildings/locations and adjust all data you see (names, coordinates, addresses, ...). What can we improve?
    features: Features you would like to see on this website
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
    features: Features
    general: General
    search: Search
</i18n>
