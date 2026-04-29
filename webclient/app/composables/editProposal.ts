import type { DeepWritable } from "ts-essentials";
import type { components } from "~/api_types";

type EditRequest = components["schemas"]["EditRequest"];
type EditRequestData = Omit<EditRequest, "privacy_checked" | "token" | "edits"> & {
  edits: NonNullable<EditRequest["edits"]>;
};
type PropertyFields = {
  name: string;
  shortName: string;
  categoryDe: string;
  categoryEn: string;
  categoryDin277: string;
  categoryDin277Desc: string;
  linkUrl: string;
  linkTextDe: string;
  linkTextEn: string;
};
type EditProposalState = {
  open: boolean;
  selected: {
    id: string | null;
    name: string | null;
  };
  data: DeepWritable<EditRequestData>;
  locationPicker: {
    open: boolean;
    lat: number;
    lon: number;
    floors: number[];
    floor: number | null;
  };
  imageUpload: {
    open: boolean;
    selectedFile: {
      base64: string;
      fileName: string;
    } | null;
    metadata: DeepWritable<components["schemas"]["ImageMetadata"]>;
  };
  propertyFields: PropertyFields;
  originalPropertyFields: PropertyFields;
};

function emptyPropertyFields(): PropertyFields {
  return {
    name: "",
    shortName: "",
    categoryDe: "",
    categoryEn: "",
    categoryDin277: "",
    categoryDin277Desc: "",
    linkUrl: "",
    linkTextDe: "",
    linkTextEn: "",
  };
}

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
      floors: [],
      floor: null,
    },
    imageUpload: {
      open: false,
      selectedFile: null,
      metadata: {
        author: "",
        license: { text: "", url: "" },
      },
    },
    propertyFields: emptyPropertyFields(),
    originalPropertyFields: emptyPropertyFields(),
  }));

function emptyRoomEdit() {
  return { coordinate: null, image: null, properties: null };
}

export type { PropertyFields };
export { emptyPropertyFields, emptyRoomEdit };
