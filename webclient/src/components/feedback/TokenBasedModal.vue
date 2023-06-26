<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { ref } from "vue";
import { Translation, useI18n } from "vue-i18n";
import { useFeedbackToken } from "@/composables/feedbackToken";

const { t } = useI18n({ inheritLocale: true, useScope: "global" });

const props = defineProps<{
  data: { [index: string]: string | boolean | number };
}>();
const global = useGlobalStore();
const loading = ref(false);
const successUrl = ref("");
const { error, token } = useFeedbackToken(t);

const privacyChecked = ref(false);
function closeForm() {
  global.feedback.open = false;
  successUrl.value = "";
  error.blockSend = false;
  error.message = "";
  document.body.classList.remove("no-scroll");
}

enum SubmissionStatus {
  SUCCESSFULLY_CREATED = 201,
  UNAVAILABLE_FOR_LEGAL_REASONS = 451,
  SERVER_ERROR = 500,
  FORBIDDEN = 403,
}
function _send() {
  const data = structuredClone(props.data);
  data.privacy_checked = privacyChecked.value;
  data.token = token.value?.token;
  fetch(`/api/feedback/feedback`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })
    .then((r) => {
      loading.value = false;
      if (r.status === SubmissionStatus.SUCCESSFULLY_CREATED) {
        token.value = null;
        r.text().then((url) => (successUrl.value = url));
      } else if (r.status === SubmissionStatus.SERVER_ERROR) {
        error.message = `${t("feedback.error.server_error")} (${r.text()})`;
      } else if (r.status === SubmissionStatus.UNAVAILABLE_FOR_LEGAL_REASONS) {
        error.message = t("feedback.error.privacy_not_checked");
      } else if (r.status === SubmissionStatus.FORBIDDEN) {
        token.value = null;
        error.message = `${t("feedback.error.send_invalid_token")} (${r.text()})`;
      } else {
        // we reset the token here to be sure that it is the cause of the error
        token.value = null;
        error.message = `${t("feedback.error.send_unexpected_status")}: ${r.status}`;
      }
    })
    .catch((r) => {
      loading.value = false;
      error.message = t("feedback.error.send_req_failed");
      console.error(r);
    });
}

function sendForm() {
  // validate the own form
  if (token.value === null) {
    error.message = t("feedback.error.send_no_token");
    error.blockSend = true;
    return;
  }
  if (!privacyChecked.value) {
    error.message = t("feedback.error.privacy_not_checked");
    return;
  }

  // validate the foreign form
  if (global.feedback.data.subject.length < 3) {
    error.message = t("feedback.error.too_short_subject");
    return;
  }
  if (global.feedback.data.body.length < 10) {
    error.message = t("feedback.error.too_short_body");
    return;
  }

  loading.value = true;
  // Token may only be used after a short delay.
  const MINIMUM_DELAY_MS = 10_000;
  const timeSinceTokenCreationInMs = Date.now() - token.value.created_at;
  if (timeSinceTokenCreationInMs < MINIMUM_DELAY_MS)
    window.setTimeout(_send, MINIMUM_DELAY_MS - timeSinceTokenCreationInMs);
  else _send();
}
</script>

<template>
  <div class="modal active" data-cy="feedback-modal" v-if="!successUrl">
    <a class="modal-overlay" :aria-label="$t('close')" @click="closeForm" />
    <div class="modal-container">
      <div class="modal-header">
        <button class="btn btn-clear float-right" :aria-label="$t('close')" @click="closeForm" />
        <div class="modal-title h5">{{ $t("feedback.title") }}</div>
      </div>
      <div class="modal-body">
        <div class="content">
          <div class="text-error">{{ error.message }}</div>

          <slot name="modal" />

          <div class="form-group">
            <label class="form-checkbox">
              <input type="checkbox" id="feedback-privacy" v-model="privacyChecked" />
              <i class="form-icon" />
              <b>
                <Translation keypath="feedback.public.agreement" tag="span">
                  <template v-slot:github_project_issues_url>
                    <a href="https://github.com/TUM-Dev/navigatum/issues" target="_blank">
                      {{ $t("feedback.public.github_project_issues") }}
                    </a>
                  </template>
                </Translation>
              </b>
              <br />
              <Translation keypath="feedback.public.disclaimer" tag="span">
                <template v-slot:github_site_policy_url>
                  <a href="https://docs.github.com/en/github/site-policy" target="_blank">
                    {{ $t("feedback.public.github_site_policy") }}
                  </a>
                </template>
              </Translation>
              <span>
                {{ $t("feedback.public.processing_based_on_gdpr") }}
              </span>
              <span>
                {{ $t("feedback.public.right_to_information") }}
                {{ $t("feedback.public.right_of_appeal") }}
              </span>
              <Translation keypath="feedback.public.objection_instruction" tag="span">
                <template v-slot:imprint_url>
                  <RouterLink to="/about/impressum">
                    {{ $t("feedback.public.imprint") }}
                  </RouterLink>
                </template>
              </Translation>
              <Translation keypath="feedback.public.question_contact" tag="span">
                <template v-slot:tum_data_protection_url>
                  <a href="https://datenschutz.tum.de" target="_blank">datenschutz.tum.de</a>
                </template>
              </Translation>
            </label>
          </div>

          <div class="float-right">
            <button class="btn" @click="closeForm">
              {{ $t("feedback.cancel") }}
            </button>
            <button
              class="btn btn-primary"
              id="feedback-send"
              @click="sendForm"
              :class="{ loading: loading }"
              v-bind="{ disabled: loading || error.blockSend }"
            >
              {{ $t("feedback.send") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div class="modal active" data-cy="feedback-success-modal" v-if="successUrl">
    <a class="modal-overlay" :aria-label="$t('close')" @click="closeForm" />
    <div class="modal-container">
      <div class="modal-header">
        <button class="btn btn-clear float-right" :aria-label="$t('close')" @click="closeForm" />
        <div class="modal-title h5">{{ $t("feedback.success.title") }}</div>
      </div>
      <div class="modal-body">
        <div class="content">
          <slot name="success" :successUrl="successUrl" />

          <div class="buttons">
            <button class="btn btn-primary" @click="closeForm">
              {{ $t("feedback.success.ok") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@import "@/assets/variables";

.modal {
  z-index: 3000;

  .modal-container {
    max-height: 95vh;
    box-shadow: $feedback-box-shadow;
  }

  .modal-overlay {
    background: $feedback-overlay-bg;
  }
}
</style>
