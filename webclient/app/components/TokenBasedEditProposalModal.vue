<script setup lang="ts">
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";
import { useFeedbackToken } from "~/composables/feedbackToken";

type EditRequest = components["schemas"]["EditRequest"];

const open = defineModel<boolean>("open", { required: true });

const props = withDefaults(
  defineProps<{
    data: Pick<EditRequest, "additional_context"> & {
      edits: NonNullable<EditRequest["edits"]>;
      additions: NonNullable<EditRequest["additions"]>;
    };
    title?: string;
  }>(),
  { title: "" }
);

const emit = defineEmits<{
  beforeSubmit: [];
}>();

const runtimeConfig = useRuntimeConfig();
const { t } = useI18n({ useScope: "local" });
const loading = ref(false);
const successUrl = ref("");
const validationFailures = ref<Array<{ key: string; error: string }>>([]);
const { error, token } = useFeedbackToken(t);
const privacyChecked = ref(false);
const editProposal = useEditProposal();
const modalTitle = computed(() => props.title || t("title"));

function closeForm() {
  open.value = false;
  editProposal.value.imageUpload.open = false;
  editProposal.value.locationPicker.open = false;
  successUrl.value = "";
  error.value.blockSend = false;
  error.value.message = "";
  validationFailures.value = [];
  privacyChecked.value = false;
}

function resetFormData() {
  // Reset form data after successful submission
  editProposal.value.data.additional_context = "";
  editProposal.value.data.edits = {};
  editProposal.value.data.additions = {};
  editProposal.value.selected.id = null;
  editProposal.value.selected.name = null;
  editProposal.value.imageUpload.selectedFile = null;
  editProposal.value.imageUpload.metadata = {
    author: "",
    license: { text: "", url: "" },
  };
  editProposal.value.locationPicker.lat = 0;
  editProposal.value.locationPicker.lon = 0;
}

enum SubmissionStatus {
  SUCCESSFULLY_CREATED = 201,
  UNAVAILABLE_FOR_LEGAL_REASONS = 451,
  SERVER_ERROR = 500,
  FORBIDDEN = 403,
  BAD_REQUEST = 400,
  UNPROCESSABLE_ENTITY = 422,
}

function _send() {
  // data is a `Window` which cannot be cloned by `structuredClone`, but can be by JSON.
  const data = JSON.parse(JSON.stringify(props.data));
  data.privacy_checked = privacyChecked.value;
  data.token = token.value?.token;

  fetch(`${runtimeConfig.public.feedbackURL}/api/feedback/propose_edits`, {
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
        resetFormData(); // Reset form data only on successful submission
        successUrl.value = await r.text();
      } else if (r.status === SubmissionStatus.SERVER_ERROR) {
        error.value.message = `${t("status.server_error")} (${await r.text()})`;
      } else if (r.status === SubmissionStatus.UNAVAILABLE_FOR_LEGAL_REASONS) {
        error.value.message = t("error.please_accept_privacy_statement");
      } else if (r.status === SubmissionStatus.FORBIDDEN) {
        token.value = null;
        error.value.message = `${t("error.send_invalid_token")} (${await r.text()})`;
      } else if (r.status === SubmissionStatus.BAD_REQUEST) {
        error.value.message = t("error.bad_request");
      } else if (r.status === SubmissionStatus.UNPROCESSABLE_ENTITY) {
        error.value.message = t("error.validation_failed");
        try {
          const body: unknown = await r.json();
          if (Array.isArray(body)) {
            validationFailures.value = (body as Array<{ key: unknown; error: unknown }>)
              .filter((e) => typeof e?.key === "string" && typeof e?.error === "string")
              .map((e) => ({ key: String(e.key), error: String(e.error) }));
          }
        } catch {
          // body wasn't JSON - leave the generic message in place.
        }
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

  // Inject property edits before validation
  emit("beforeSubmit");

  // Reset stale per-key validation failures from a previous submission attempt.
  validationFailures.value = [];

  // validate the foreign form - require context, edits, or additions
  const hasContext = editProposal.value.data.additional_context.length >= 10;
  const hasEdits = Object.keys(editProposal.value.data.edits).length > 0;
  const hasAdditions = Object.keys(editProposal.value.data.additions).length > 0;

  if (!hasContext && !hasEdits && !hasAdditions) {
    error.value.message = t("error.form.no_content");
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
  <Modal v-if="!successUrl" v-model="open" :title="modalTitle" @close="closeForm">
    <Toast v-if="error.message" id="token-modal-error" class="mb-4" :msg="error.message" level="error" />
    <div
      v-if="validationFailures.length"
      class="bg-red-50 dark:bg-red-900 border-red-300 dark:border-red-600 mb-4 rounded border px-3 py-2"
      data-cy="validation-failures"
    >
      <p class="text-red-900 dark:text-red-50 text-sm font-semibold">{{ t("validation_failures.title") }}</p>
      <ul class="mt-1 list-disc pl-5 text-sm text-red-900 dark:text-red-50">
        <li v-for="failure in validationFailures" :key="failure.key">
          <code class="bg-red-100 dark:bg-red-800 rounded px-1 py-0.5 text-xs">{{ failure.key }}</code>
          - {{ failure.error }}
        </li>
      </ul>
    </div>

    <div class="flex flex-col gap-1">
      <slot name="modal" />
      <div class="mt-6">
        <Checkbox id="privacy-checked" v-model="privacyChecked">
          <I18nT tag="span" keypath="public.consent">
            <template #github>
              <NuxtLink
                tabindex="1"
                class="text-blue-600 dark:text-blue-300 visited:text-blue-600 dark:visited:text-blue-300 hover:underline"
                to="https://github.com/TUM-Dev/NavigaTUM"
                target="_blank"
                external
              >
                GitHub
              </NuxtLink>
            </template>
            <template #privacy_policy>
              <NuxtLink
                tabindex="1"
                :to="t('public.privacy_policy_url')"
                class="text-blue-600 dark:text-blue-300 visited:text-blue-600 dark:visited:text-blue-300 hover:underline"
              >
                {{ t("public.privacy_policy") }}
              </NuxtLink>
            </template>
          </I18nT>
        </Checkbox>
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
  <Modal v-if="successUrl" v-model="open" :title="t('thank_you')" @close="closeForm">
    <slot name="success" :success-url="successUrl" />

    <Btn size="md" variant="primary" @click="closeForm">OK</Btn>
  </Modal>
</template>

<i18n lang="yaml">
de:
  title: Änderungen vorschlagen
  cancel: Abbrechen
  error:
    token_unexpected_status: "Unerwarteter Status Code beim Abrufen eines Feedback Tokens: "
    token_req_failed: Unerwarteter Fehler beim Laden des Bearbeitungsformulars. Das Senden von Änderungsvorschlägen ist gerade vermutlich nicht möglich. Bitte schreibe stattdessen eine Mail.
    too_many_requests: Änderungsvorschläge senden ist aktuell nicht möglich aufgrund von rate-limiting. Bitte versuche es später nochmal oder schreibe eine Mail.
    send_invalid_token: Formular-Token ungültig (vermutlich abgelaufen). Bitte kopiere den Text und öffne das Formular nochmal.
    please_accept_privacy_statement: Du musst die Datenschutzerklärung akzeptiert haben, damit wir deinen Änderungsvorschlag via GitHub verarbeiten können.
    feedback_not_configured: Das Senden von Änderungsvorschlägen ist auf dem Server aktuell nicht konfiguriert.
    send_no_token: Ein unerwarteter Fehler ist aufgetreten (Kein Token). Bitte kopiere den Text und öffne das Formular nochmal.
    send_req_failed: Unerwarteter Fehler beim Senden des Bearbeitungsformulars. Das Senden von Änderungsvorschlägen ist gerade vermutlich nicht möglich. Bitte schreibe stattdessen eine Mail.
    bad_request: Ungültige Anfrage. Nicht alle erforderlichen Felder sind vorhanden.
    validation_failed: Validierung fehlgeschlagen. Bitte überprüfe deine Eingaben.
    form:
      no_content: "Fehler: Bitte füge zusätzlichen Kontext (mindestens 10 Zeichen), konkrete Änderungen oder neue Einträge hinzu"
  status:
    send_unexpected_status: Unerwarteter Status Code
    server_error: Server Fehler
  validation_failures:
    title: "Folgende Einträge konnten nicht angelegt werden:"
  public:
    consent: Ich verstehe, dass mein Änderungsvorschlag auf {github} veröffentlicht und gemäß der {privacy_policy} verarbeitet wird. Mit dem Absenden dieses Formulars willige ich in diese Verarbeitung ein.
    privacy_policy: Datenschutzerklärung
    privacy_policy_url: /about/datenschutz
  sending: Wird gesendet
  try_again_later: Bitte versuche es später noch einmal
  send: Senden
  thank_you: Vielen Dank!
en:
  title: Propose Changes
  cancel: Cancel
  error:
    token_unexpected_status: Unexpected status code when retrieving a feedback token
    send_invalid_token: Invalid form token (probably expired). Please copy the text and re-open the form.
    please_accept_privacy_statement: You have to accept the privacy statement for us to process the edit proposal via GitHub.
    feedback_not_configured: Sending edit proposals is currently not configured on the server.
    send_no_token: An unexpected error occured (no token). Please copy the text and re-open the form.
    send_req_failed: Unexpected error when sending the edit proposal form. Sending edit proposals is currently probably not possible. Please send a mail instead.
    token_req_failed: Unexpected error when loading the edit proposal form. Sending edit proposals is currently probably not possible. Please send a mail instead.
    too_many_requests: Sending edit proposals is currently not possible due to rate-limiting. Please try again in a while or send a mail.
    bad_request: Invalid request. Not all required fields are present.
    validation_failed: Validation failed. Please check your inputs.
    form:
      no_content: "Error: Please add additional context (at least 10 characters), concrete edits, or new entries"
  status:
    server_error: Server Error
    send_unexpected_status: Unexpected status code
  validation_failures:
    title: "These entries could not be created:"
  public:
    consent: I understand that my edit proposal will be published on {github} and processed in accordance with the {privacy_policy}. By submitting this form, I consent to this processing.
    privacy_policy: Privacy Policy
    privacy_policy_url: /en/about/privacy
  sending: Sending
  try_again_later: Please try again later
  send: Send
  thank_you: Thank you!
</i18n>
