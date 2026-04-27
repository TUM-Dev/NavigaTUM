import type { components } from "~/api_types";

type PostFeedbackRequest = components["schemas"]["PostFeedbackRequest"];
type FeedbackState = {
  open: boolean;
  data: Omit<PostFeedbackRequest, "privacy_checked" | "token">;
};
export const useFeedback = () =>
  useState<FeedbackState>("feedback", () => ({
    open: false,
    data: {
      category: "general",
      subject: "",
      body: "",
      deletion_requested: false,
    },
  }));
