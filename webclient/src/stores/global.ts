import { defineStore } from "pinia";
import type { components } from "@/api_types";
type PostFeedbackRequest = components["schemas"]["PostFeedbackRequest"];

export type Coord = {
  coords: {
    lat: number | undefined;
    lon: number | undefined;
  };
};
type UserTheme = "dark" | "light";
function initialUserTheme(): UserTheme {
  const storedPreference = localStorage.getItem("theme") as UserTheme;
  if (["dark", "light"].includes(storedPreference)) return storedPreference;

  const hasDarkPreference = window.matchMedia("(prefers-color-scheme: dark)").matches;
  return hasDarkPreference ? "dark" : "light";
}
export const useGlobalStore = defineStore({
  id: "global",
  state: () => ({
    search_focused: false,
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
    theme: initialUserTheme(),
  }),
  actions: {
    focusSearchBar(): void {
      this.search_focused = true;
    },
    unfocusSearchBar(): void {
      this.search_focused = false;
    },
    openFeedback(category: PostFeedbackRequest["category"] = "general", subject = "", body = ""): void {
      this.feedback.open = true;
      this.feedback.data = { category, subject, body, deletion_requested: false };

      document.body.classList.add("overflow-y-hidden");
    },
    temporarilyCloseFeedback(): void {
      this.feedback.open = false;
      document.body.classList.remove("overflow-y-hidden");
    },
  },
});
