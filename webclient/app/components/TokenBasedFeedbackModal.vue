<script setup lang="ts">
import { useFeedback } from "~/composables/feedback";
import { useFeedbackToken } from "~/composables/feedbackToken";

const props = defineProps<{
  data: { [index: string]: string | boolean | number };
}>();

const runtimeConfig = useRuntimeConfig();
const { t } = useI18n({ useScope: "local" });
const loading = ref(false);
const successUrl = ref("");
const { error, token } = useFeedbackToken(t);
const privacyChecked = ref(false);
const feedback = useFeedback();
const initialBody = feedback.value.data.body;

function closeForm() {
  feedback.value.open = false;
  successUrl.value = "";
  error.value.blockSend = false;
  error.value.message = "";
}

enum SubmissionStatus {
  SUCCESSFULLY_CREATED = 201,
  UNAVAILABLE_FOR_LEGAL_REASONS = 451,
  SERVER_ERROR = 500,
  FORBIDDEN = 403,
}

function _send() {
  // data is a `Window` which cannot be cloned by `structuredClone`, but can be by JSON.
  const data = JSON.parse(JSON.stringify(props.data));
  data.privacy_checked = privacyChecked.value;
  data.token = token.value?.token;
  fetch(`${runtimeConfig.public.feedbackURL}/api/feedback/feedback`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })
    .then(async (r) => {
      loading.value = false;
      if (r.status === SubmissionStatus.SUCCESSFULLY_CREATED) {
        token.value = null;
        successUrl.value = await r.text();
      } else if (r.status === SubmissionStatus.SERVER_ERROR) {
        error.value.message = `${t("status.server_error")} (${r.text()})`;
      } else if (r.status === SubmissionStatus.UNAVAILABLE_FOR_LEGAL_REASONS) {
        error.value.message = t("error.please_accept_privacy_statement");
      } else if (r.status === SubmissionStatus.FORBIDDEN) {
        token.value = null;
        error.value.message = `${t("error.send_invalid_token")} (${r.text()})`;
      } else {
        // we reset the token here to be sure that it is the cause of the error
        token.value = null;
        error.value.message = `${t("status.send_unexpected_status")}: ${r.status}`;
      }
      if (r.status !== SubmissionStatus.SUCCESSFULLY_CREATED) {
        document.getElementById("token-modal-error")?.scrollIntoView({ behavior: "smooth" });
      }
    })
    .catch((r) => {
      loading.value = false;
      error.value.message = t("error.send_req_failed");
      console.error(r);
      document.getElementById("token-modal-error")?.scrollIntoView({ behavior: "smooth" });
    });
}

function sendForm() {
  // validate the own form
  if (token.value === null) {
    error.value.message = t("error.send_no_token");
    error.value.blockSend = true;
    return;
  }
  if (!privacyChecked.value) {
    error.value.message = t("error.please_accept_privacy_statement");
    return;
  }

  // validate the foreign form
  if (feedback.value.data.subject.length < 3) {
    error.value.message = t("error.form.too_short_subject");
    return;
  }
  if (feedback.value.data.body.length < 10) {
    error.value.message = t("error.form.too_short_body");
    return;
  }
  if (initialBody && feedback.value.data.body.trim() === initialBody.trim()) {
    error.value.message = t("error.form.body_unchanged");
    return;
  }

  loading.value = true;
  // Token may only be used after a short delay.
  const MINIMUM_DELAY_MS = 10_000;
  const timeSinceTokenCreationInMs = Date.now() - token.value.created_at;
  if (timeSinceTokenCreationInMs < MINIMUM_DELAY_MS)
    setTimeout(_send, MINIMUM_DELAY_MS - timeSinceTokenCreationInMs);
  else _send();
}
</script>

<template>
  <Modal v-if="!successUrl" v-model="feedback.open" :title="t('title')" @close="closeForm">
    <Toast v-if="error.message" class="mb-4" :msg="error.message" level="error" id="feedback-error-msg" />

    <div class="flex flex-col gap-1">
      <slot name="modal" />
      <div>
        <FeedbackConsentCheckbox id="privacy-checked" v-model="privacyChecked" kind="feedback" />
      </div>
    </div>

    <div class="float-right flex flex-row-reverse gap-2">
      <Btn
        variant="primary"
        size="md"
        :class="{
          '!text-blue-900 dark:!text-blue-50 !bg-blue-200 dark:!bg-blue-700 cursor-progress': loading,
          '!text-blue-50 dark:!text-blue-900 !bg-blue-300 dark:!bg-blue-600 cursor-not-allowed': error.blockSend,
        }"
        v-bind="{ disabled: loading || error.blockSend }"
        @click="sendForm"
      >
        <template v-if="loading">
          <Spinner class="my-auto h-4 w-4" />
          {{ t("sending") }}...
        </template>
        <template v-else-if="error.blockSend">{{ t("try_again_later") }}</template>
        <template v-else>{{ t("send") }}</template>
      </Btn>
      <Btn variant="linkButton" size="md" @click="closeForm">
        {{ t("cancel") }}
      </Btn>
    </div>
  </Modal>
  <Modal v-if="successUrl" v-model="feedback.open" :title="t('thank_you')" @close="closeForm">
    <slot name="success" :success-url="successUrl" />

    <Btn size="md" variant="primary" @click="closeForm">OK</Btn>
  </Modal>
</template>

<i18n lang="yaml">
de:
  title: Feedback senden
  cancel: Abbrechen
  error:
    token_unexpected_status: "Unerwarteter Status Code beim Abrufen eines Feedback Tokens: "
    token_req_failed: Unerwarteter Fehler beim Laden des Feedback-Formulars. Das Senden von Feedback ist gerade vermutlich nicht möglich. Bitte schreibe stattdessen eine Mail.
    too_many_requests: Feedback senden ist aktuell nicht möglich aufgrund von rate-limiting. Bitte versuche es später nochmal oder schreibe eine Mail.
    send_invalid_token: Formular-Token ungültig (vermutlich abgelaufen). Bitte kopiere den Text und öffne das Formular nochmal.
    please_accept_privacy_statement: Du musst die Datenschutzerklärung akzeptiert haben, damit wir dein Feedback via GitHub verarbeiten können.
    feedback_not_configured: Das Senden von Feedback ist auf dem Server aktuell nicht konfiguriert.
    send_no_token: Ein unerwarteter Fehler ist aufgetreten (Kein Token). Bitte kopiere den Text und öffne das Formular nochmal.
    send_req_failed: Unerwarteter Fehler beim Senden des Feedback-Formulars. Das Senden von Feedback ist gerade vermutlich nicht möglich. Bitte schreibe stattdessen eine Mail.
    form:
      too_short_body: "Fehler: Nachricht fehlt oder ist zu kurz"
      too_short_subject: "Fehler: Betreff fehlt oder ist zu kurz"
      body_unchanged: "Fehler: Bitte beschreibe dein Feedback in der Nachricht"
  status:
    send_unexpected_status: Unerwarteter Status Code
    server_error: Server Fehler
  sending: Wird gesendet
  try_again_later: Bitte versuche es später noch einmal
  send: Senden
  thank_you: Vielen Dank!
en:
  title: Send Feedback
  cancel: Cancel
  error:
    token_unexpected_status: "Unexpected status code when retrieving a feedback token: "
    send_invalid_token: Invalid form token (probably expired). Please copy the text and re-open the form.
    please_accept_privacy_statement: You have to accept the privacy statement for us to process the feedback via GitHub.
    feedback_not_configured: Sending feedback is currently not configured on the server.
    send_no_token: An unexpected error occured (no token). Please copy the text and re-open the form.
    send_req_failed: Unexpected error when sending the feedback form. Sending feedback is currently probably not possible. Please send a mail instead.
    token_req_failed: Unexpected error when loading the feedback form. Sending feedback is currently probably not possible. Please send a mail instead.
    too_many_requests: Sending feedback is currently not possible due to rate-limiting. Please try again in a while or send a mail.
    form:
      too_short_body: "Error: Message missing or too short"
      too_short_subject: "Error: Subject missing or too short"
      body_unchanged: "Error: Please describe your feedback in the message"
  status:
    server_error: Server Error
    send_unexpected_status: Unexpected status code
  sending: Sending
  try_again_later: not possible
  send: Send
  thank_you: Thank you!
</i18n>
