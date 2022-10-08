import { defineStore } from "pinia";
import {TokenRequest} from "@/codegen";
import Category = TokenRequest.CategoryEnum;

export const useGlobalStore = defineStore({
  id: "global",
  state: () => ({
    search_focused: false,
    error_message: null,
    feedback: {
      open: false,
      category: Category.General,
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
    openFeedback(category = Category.General, subject = "", body = "") {
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
  },
});
