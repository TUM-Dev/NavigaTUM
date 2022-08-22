import { defineStore } from "pinia";
import type { components } from "@/api_types";
type TokenRequest = components["schemas"]["TokenRequest"];

export const useGlobalStore = defineStore({
  id: "global",
  state: () => ({
    search_focused: false,
    error_message: null,
    information_modal: {
      header: null as string | null,
      body: null as string | null,
    },
    feedback: {
      open: false,
      category: "general" as TokenRequest["category"],
      subject: "",
      body: "",
    },
  }),
  actions: {
    focus_search() {
      this.search_focused = true;
    },
    unfocus_search() {
      this.search_focused = false;
    },
    openFeedback(category: TokenRequest["category"] = "general", subject = "", body = "") {
      this.feedback.open = true;
      this.feedback.category = category;
      this.feedback.subject = subject;
      this.feedback.body = body;

      document.body.classList.add("no-scroll");
    },
    temprarilyCloseFeedback() {
      this.feedback.open = false;
      document.body.classList.remove("no-scroll");
    },
    reopenFeedback() {
      this.feedback.open = false;
      document.body.classList.remove("no-scroll");
    },
    showInformationModal(body: string, header: string | null = null) {
      this.information_modal = { body, header };
    },
  },
});
