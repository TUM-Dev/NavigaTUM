import { defineStore } from "pinia";
import type { components } from "../api_types";

type PostFeedbackRequest = components["schemas"]["PostFeedbackRequest"];

export const useGlobalStore = defineStore({
  id: "global",
  state: () => ({
    error_message: null as string | null,
    feedback: {
      open: false,
      data: {
        category: "general",
        subject: "",
        body: "",
        deletion_requested: false,
      } as Omit<PostFeedbackRequest, "privacy_checked" | "token">,
    },
  }),
  actions: {
    openFeedback(category: PostFeedbackRequest["category"] = "general", subject = "", body = ""): void {
      this.feedback.open = true;
      this.feedback.data = { category, subject, body, deletion_requested: false };

      document.body.classList.add("overflow-y-hidden");
    },
  },
});
