<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import TokenBasedModal from "@/components/feedback/TokenBasedModal.vue";

const { t } = useI18n({ inheritLocale: true, useScope: "global" });
const global = useGlobalStore();
const deleteIssueRequested = ref(false);
</script>

<template>
  <TokenBasedModal :data="global.feedback.data">
    <template v-slot:modal>
      <div class="form-group">
        <div id="feedback-coordinate-picker-helptext" class="d-none toast toast-primary">
          {{ $t("feedback.coordinatepicker.helptext.enter_serveral") }}<br />
          {{ $t("feedback.coordinatepicker.helptext.saved_for_12h") }}<br />
        </div>
        <label class="form-label" for="feedback-subject"> {{ $t("feedback.subject") }}</label>
        <div class="input-group">
          <select
            class="form-select"
            id="feedback-category"
            :aria-label="$t('feedback.category')"
            v-model="global.feedback.data.category"
          >
            <option value="general">{{ $t("feedback.type.general") }}</option>
            <option value="bug">{{ $t("feedback.type.bug") }}</option>
            <option value="features">{{ $t("feedback.type.features") }}</option>
            <option value="search">{{ $t("feedback.type.search") }}</option>
            <option value="entry">{{ $t("feedback.type.entry") }}</option>
          </select>
          <input
            class="form-input"
            type="text"
            :placeholder="$t('feedback.subject')"
            v-model="global.feedback.data.subject"
            id="feedback-subject"
          />
        </div>
      </div>

      <div class="form-group">
        <div>
          <label class="form-label" for="feedback-body">
            {{ $t("feedback.message") }}
          </label>
          <button
            id="feedback-coordinate-picker"
            v-if="global.feedback.data.category === 'entry'"
            class="btn btn-sm btn-link"
          >
            {{ $t("feedback.coordinatepicker.title") }}
          </button>
        </div>
        <textarea
          class="form-input"
          id="feedback-body"
          :placeholder="$t('feedback.message')"
          v-model="global.feedback.data.body"
          rows="6"
        >
        </textarea>
        <p class="text-gray text-tiny">
          {{
            {
              general: t("feedback.helptext.general"),
              bug: t("feedback.helptext.bug"),
              feature: t("feedback.helptext.features"),
              search: t("feedback.helptext.search"),
              entry: t("feedback.helptext.entry"),
              other: t("feedback.helptext.other"), // This is only here to make the linter happy, backend uses "other" as a fallback if the category is not known
            }[global.feedback.data.category]
          }}
        </p>
      </div>

      <!-- only visible if called through a view, because then the context of the calling building is availible -->
      <div>
        <button id="feedback-coordinate-picker" class="btn btn-sm d-none">
          {{ $t("feedback.coordinatepicker.title") }}
        </button>
      </div>
      <div class="form-group">
        <label class="form-checkbox" id="feedback-delete-label">
          <input type="checkbox" id="feedback-delete" v-model="deleteIssueRequested" />
          <i class="form-icon" /> {{ $t("feedback.delete") }}
        </label>
      </div>
    </template>
    <template v-slot:success="{ successUrl }">
      <p>{{ $t("feedback.success.thank_you") }}</p>
      <p>
        {{ $t("feedback.success.response_at") }}
        <a id="feedback-success-url" class="btn-link" :href="successUrl">{{ $t("feedback.success.this_issue") }}</a>
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

  #feedback-coordinate-picker {
    float: right;
    margin-top: 0.5em;
  }

  #feedback-coordinate-picker-helptext {
    font-size: 14px;
  }
}
</style>
