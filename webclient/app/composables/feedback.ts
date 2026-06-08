import type { components } from "~/api_types";

type PostFeedbackRequest = components["schemas"]["PostFeedbackRequest"];
interface FeedbackState {
  open: boolean;
  data: Omit<PostFeedbackRequest, "privacy_checked" | "token">;
}
export const useFeedback = () =>
  useState<FeedbackState>("feedback", () => ({
    open: false,
    data: {
      category: "general",
      subject: "",
      body: "",
    },
  }));
