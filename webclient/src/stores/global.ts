import { defineStore } from "pinia";
import type { components } from "@/api_types";
type TokenRequest = components["schemas"]["TokenRequest"];

export type Coord = {
  coords: {
    lat: number | undefined;
    lon: number | undefined;
  };
};
export const useGlobalStore = defineStore({
  id: "global",
  state: () => ({
    search_focused: false,
    error_message: null as string | null,
    feedback: {
      open: false,
      category: "general" as TokenRequest["category"],
      subject: "",
      body: "",
    },
  }),
  actions: {
    focusSearchBar(): void {
      this.search_focused = true;
    },
    unfocusSearchBar(): void {
      this.search_focused = false;
    },
    openFeedback(category: TokenRequest["category"] = "general", subject = "", body = ""): void {
      this.feedback.open = true;
      this.feedback.category = category;
      this.feedback.subject = subject;
      this.feedback.body = body;

      document.body.classList.add("no-scroll");
    },
    temporarilyCloseFeedback(): void {
      this.feedback.open = false;
      document.body.classList.remove("no-scroll");
    },
  },
});
