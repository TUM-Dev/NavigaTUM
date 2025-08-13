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
  imageUpload: {
    open: boolean;
    selectedFile: {
      base64: string;
      fileName: string;
    } | null;
    metadata: DeepWritable<components["schemas"]["ImageMetadata"]>;
  };
};

export const useEditProposal = () =>
  useState<EditProposalState>("editProposal", () => ({
    open: false,
    selected: {
      id: null,
      name: null,
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
    imageUpload: {
      open: false,
      selectedFile: null,
      metadata: {
        author: "",
        license: { text: "", url: "" },
      },
    },
  }));
