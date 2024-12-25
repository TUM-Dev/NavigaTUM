import type { Ref } from "vue";
import type { components } from "~/api_types";
import { useLocalStorage } from "@vueuse/core";

type TokenResponse = components["schemas"]["TokenResponse"];

enum TokenStatus {
  SUCCESSFULLY_CREATED = 201,
  TOO_MANY_REQUESTS = 429,
  NOT_CONFIGURED = 503,
}

export function useFeedbackToken(t: ReturnType<typeof useI18n>["t"]): {
  error: Ref<{ message: string; blockSend: boolean }>;
  token: { value: TokenResponse | null };
} {
  const token = useLocalStorage<TokenResponse | null>("feedback-token", null, {
    serializer: {
      read: (v) => (v ? JSON.parse(v) : null),
      write: (v) => JSON.stringify(v),
    },
  });
  const error = useState('feedback_error', ()=>({
    message: "",
    blockSend: false,
  }));

  // Token are renewed much before being invalid on the server.
  const MS_PER_HOUR = 3600000;
  const TOKEN_VALIDITY_FRONTEND_HOURS = 6;
  const runtimeConfig = useRuntimeConfig();
  if (token.value === null || Date.now() - token.value.created_at > TOKEN_VALIDITY_FRONTEND_HOURS * MS_PER_HOUR) {
    fetch(`${runtimeConfig.public.feedbackURL}/api/feedback/get_token`, { method: "POST" })
      .then((r) => {
        if (r.status === TokenStatus.SUCCESSFULLY_CREATED) {
          r.json()
            .then((j: TokenResponse) => {
              token.value = j;
            })
            .catch((r) => {
              error.value.message = t("error.token_req_failed");
              console.error(r);
            });
        } else if (r.status === TokenStatus.TOO_MANY_REQUESTS) {
          error.value.message = t("error.too_many_requests");
          error.value.blockSend = true;
        } else if (r.status === TokenStatus.NOT_CONFIGURED) {
          error.value.message = t("error.feedback_not_configured");
          error.value.blockSend = true;
        } else {
          error.value.message = `${t("error.token_unexpected_status")}${r.status}`;
          error.value.blockSend = true;
        }
        if (r.status !== TokenStatus.SUCCESSFULLY_CREATED)
          document.getElementById("token-modal-error")?.scrollIntoView({ behavior: "smooth" });
      })
      .catch((r) => {
        error.value.message = t("error.token_req_failed");
        console.error(r);
      });
  }
  return { error, token };
}
