import type { components } from "~/api_types";
import { useFeedbackToken } from "~/composables/feedbackToken";

type EditRequest = components["schemas"]["EditRequest"];
interface SubmittableEdits {
  additional_context?: string;
  edits: NonNullable<EditRequest["edits"]>;
  additions: NonNullable<EditRequest["additions"]>;
}

export interface ValidationFailure {
  key: string;
  error: string;
}

enum SubmissionStatus {
  SUCCESSFULLY_CREATED = 201,
  BAD_REQUEST = 400,
  FORBIDDEN = 403,
  UNPROCESSABLE_ENTITY = 422,
  UNAVAILABLE_FOR_LEGAL_REASONS = 451,
  SERVER_ERROR = 500,
}

// Server rejects tokens younger than this; wait it out so the first attempt isn't rejected.
const MINIMUM_TOKEN_AGE_MS = 10_000;

const MESSAGES = {
  de: {
    feedbackSubmission: {
      error: {
        please_accept_privacy_statement:
          "Du musst die Datenschutzerklärung akzeptiert haben, damit wir deinen Vorschlag via GitHub verarbeiten können.",
        send_no_token:
          "Ein unerwarteter Fehler ist aufgetreten (kein Token). Bitte lade die Seite neu.",
        send_invalid_token:
          "Formular-Token ungültig (vermutlich abgelaufen). Bitte lade die Seite neu.",
        send_req_failed:
          "Unerwarteter Fehler beim Senden des Vorschlags. Bitte versuche es später noch einmal.",
        send_unexpected_status: "Unerwarteter Status Code",
        bad_request: "Ungültige Anfrage. Nicht alle erforderlichen Felder sind vorhanden.",
        validation_failed: "Validierung fehlgeschlagen. Bitte überprüfe deine Eingaben.",
        server_error: "Server Fehler",
        too_many_requests:
          "Vorschläge senden ist aktuell nicht möglich aufgrund von rate-limiting. Bitte versuche es später nochmal oder schreibe eine Mail.",
        feedback_not_configured:
          "Das Senden von Vorschlägen ist auf dem Server aktuell nicht konfiguriert.",
        token_unexpected_status:
          "Unerwarteter Status Code beim Abrufen eines Feedback Tokens: ",
        token_req_failed:
          "Unerwarteter Fehler beim Laden des Bearbeitungsformulars. Das Senden von Vorschlägen ist gerade vermutlich nicht möglich. Bitte schreibe stattdessen eine Mail.",
      },
    },
  },
  en: {
    feedbackSubmission: {
      error: {
        please_accept_privacy_statement:
          "You have to accept the privacy statement for us to process the proposal via GitHub.",
        send_no_token: "An unexpected error occurred (no token). Please reload the page.",
        send_invalid_token: "Form token is invalid (probably expired). Please reload the page.",
        send_req_failed: "Unexpected error when sending the proposal. Please try again later.",
        send_unexpected_status: "Unexpected status code",
        bad_request: "Invalid request. Not all required fields are present.",
        validation_failed: "Validation failed. Please check your inputs.",
        server_error: "Server Error",
        too_many_requests:
          "Sending proposals is currently not possible due to rate-limiting. Please try again in a while or send a mail.",
        feedback_not_configured:
          "Sending proposals is currently not configured on the server.",
        token_unexpected_status: "Unexpected status code when retrieving a feedback token",
        token_req_failed:
          "Unexpected error when loading the edit proposal form. Sending proposals is currently probably not possible. Please send a mail instead.",
      },
    },
  },
} as const;

export function useFeedbackSubmission() {
  const i18n = useI18n({ useScope: "global" });
  // mergeLocaleMessage is idempotent; on SSR the i18n instance is per-request, so a module-level guard would silently skip subsequent requests.
  i18n.mergeLocaleMessage("de", MESSAGES.de);
  i18n.mergeLocaleMessage("en", MESSAGES.en);
  const t = (key: string) => i18n.t(`feedbackSubmission.${key}`);
  const { error: tokenError, token } = useFeedbackToken(
    ((key: string) => t(key)) as ReturnType<typeof useI18n>["t"]
  );
  const runtimeConfig = useRuntimeConfig();

  const submitting = ref(false);
  const submitError = ref("");
  const successUrl = ref("");
  const validationFailures = ref<ValidationFailure[]>([]);

  const blockedByToken = computed(() => tokenError.value.blockSend);

  async function submit(data: SubmittableEdits, privacyChecked: boolean): Promise<boolean> {
    submitError.value = "";
    validationFailures.value = [];
    if (!privacyChecked) {
      submitError.value = t("error.please_accept_privacy_statement");
      return false;
    }
    if (!token.value) {
      submitError.value = t("error.send_no_token");
      return false;
    }
    const age = Date.now() - token.value.created_at;
    submitting.value = true;
    if (age < MINIMUM_TOKEN_AGE_MS) {
      await new Promise((resolve) => setTimeout(resolve, MINIMUM_TOKEN_AGE_MS - age));
    }
    const body = {
      additional_context: data.additional_context ?? "",
      edits: data.edits,
      additions: data.additions,
      privacy_checked: true,
      token: token.value.token,
    };
    let response: Response;
    try {
      response = await fetch(`${runtimeConfig.public.feedbackURL}/api/feedback/propose_edits`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
    } catch (e) {
      submitting.value = false;
      submitError.value = t("error.send_req_failed");
      console.error(e);
      return false;
    }
    submitting.value = false;
    if (response.status === SubmissionStatus.SUCCESSFULLY_CREATED) {
      successUrl.value = await response.text();
      token.value = null;
      return true;
    }
    if (response.status === SubmissionStatus.SERVER_ERROR) {
      submitError.value = `${t("error.server_error")} (${await response.text()})`;
      return false;
    }
    if (response.status === SubmissionStatus.UNAVAILABLE_FOR_LEGAL_REASONS) {
      submitError.value = t("error.please_accept_privacy_statement");
      return false;
    }
    if (response.status === SubmissionStatus.FORBIDDEN) {
      token.value = null;
      submitError.value = `${t("error.send_invalid_token")} (${await response.text()})`;
      return false;
    }
    if (response.status === SubmissionStatus.BAD_REQUEST) {
      submitError.value = t("error.bad_request");
      return false;
    }
    if (response.status === SubmissionStatus.UNPROCESSABLE_ENTITY) {
      submitError.value = t("error.validation_failed");
      try {
        const parsed: unknown = await response.json();
        if (Array.isArray(parsed)) {
          validationFailures.value = (parsed as Array<{ key: unknown; error: unknown }>)
            .filter((e) => typeof e?.key === "string" && typeof e?.error === "string")
            .map((e) => ({ key: String(e.key), error: String(e.error) }));
        }
      } catch {
        // Body wasn't JSON.
      }
      return false;
    }
    token.value = null;
    submitError.value = `${t("error.send_unexpected_status")}: ${response.status}`;
    return false;
  }

  function reset() {
    submitError.value = "";
    successUrl.value = "";
    validationFailures.value = [];
  }

  return {
    submit,
    reset,
    submitting,
    submitError,
    successUrl,
    validationFailures,
    blockedByToken,
  };
}
