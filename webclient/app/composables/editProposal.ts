import type { components } from "~/api_types";

type EditRequest = components["schemas"]["EditRequest"];
type Writeable<T> = { -readonly [P in keyof T]: T[P] };
type EditProposalState = {
  open: boolean;
  selected: {
    roomId: string;
    name: string;
  } | null;
  data: Writeable<Omit<EditRequest, "privacy_checked" | "token">>;
};

export const useEditProposal = () =>
  useState<EditProposalState>("editProposal", () => ({
    open: false,
    selected: null,
    data: {
      additional_context: "",
      edits: {},
    },
  }));
