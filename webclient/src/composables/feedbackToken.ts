import { reactive } from "vue";
import type { components } from "@/api_types";
import { useLocalStorage } from "@vueuse/core";
import { useI18n } from "vue-i18n";
type TokenResponse = components["schemas"]["TokenResponse"];

const { t } = useI18n({ inheritLocale: true, useScope: "global" });

enum TokenStatus {
  SUCCESSFULLY_CREATED = 201,
  TOO_MANY_REQUESTS = 429,
  NOT_CONFIGURED=  503,
}

export function useFeedbackToken() {
  const token = useLocalStorage<TokenResponse | null>("feedback-token", null, {
    serializer: {
      read: (v) => (v ? JSON.parse(v) : null),
      write: (v) => JSON.stringify(v),
    },
  });
  const error = reactive({
    message: "",
    blockSend: false,
  });

  // legacy migration function TODO: remove only after 31.09.2023, to give our users time to migrate to the new token format
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (token.value?.expiry) {
    token.value = null;
  }

  // Token are renewed much before being invalid on the server.
  const MS_PER_HOUR = 3600000;
  const TOKEN_VALIDITY_MS = MS_PER_HOUR * 6;
  if (token.value === null || Date.now() - token.value.created_at > TOKEN_VALIDITY_MS) {
    fetch(`/api/feedback/get_token`, { method: "POST" })
      .then((r) => {
        if (r.status === TokenStatus.SUCCESSFULLY_CREATED) {
          r.json()
            .then((j: TokenResponse) => {
              token.value = j;
            })
            .catch((r) => {
              error.message = t("feedback.error.token_req_failed");
              console.error(r);
            });
        } else if (r.status === TokenStatus.TOO_MANY_REQUESTS) {
          error.message = t("feedback.error.too_many_requests");
          error.blockSend = true;
        } else if (r.status === TokenStatus.NOT_CONFIGURED) {
          error.message = t("feedback.error.feedback_not_configured");
          error.blockSend = true;
        } else {
          error.message = `${t("feedback.error.token_unexpected_status")}${r.status}`;
          error.blockSend = true;
        }
      })
      .catch((r) => {
        error.message = t("feedback.error.token_req_failed");
        console.error(r);
      });
  }
  return { error, token };
}
