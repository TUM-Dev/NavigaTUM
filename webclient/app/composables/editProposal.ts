import type { DeepWritable } from "ts-essentials";
import type { components } from "~/api_types";

type EditRequest = components["schemas"]["EditRequest"];
type EditProposalState = {
  open: boolean;
  selected: {
    id: string | null;
    name: string | null;
  };
  data: DeepWritable<Omit<EditRequest, "privacy_checked" | "token">>;
  locationPicker: {
    open: boolean;
    lat: number;
    lon: number;
  };
};

export const useEditProposal = () => 
  useState<EditProposalState>("editProposal", () => ({
    open: false,
    selected: {
      id: null,
      name: null
    },
    data: {
      additional_context: "",
      edits: {},
    },
    locationPicker: {
      open: false,
      lat: 0,
      lon: 0,
    },
  }));
