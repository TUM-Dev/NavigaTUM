import type { DeepWritable } from "ts-essentials";
import type { components } from "~/api_types";

type EditRequest = components["schemas"]["EditRequest"];
type EditRequestData = Omit<EditRequest, "privacy_checked" | "token" | "edits" | "additions"> & {
  edits: NonNullable<EditRequest["edits"]>;
  additions: NonNullable<EditRequest["additions"]>;
};
type BuildingKind = components["schemas"]["BuildingKind"];
type AdditionKind = "room" | "building" | "poi";

type LinkDraft = { text_de: string; text_en: string; url: string };
type GenericPropDraft = { name_de: string; name_en: string; text: string };

type AdditionDraft = {
  kind: AdditionKind | null;
  id: string;
  parent_id: string;
  parent_name: string;
  coords: { lat: number; lon: number; picked: boolean };
  // room-only
  alt_name: string;
  arch_name: string;
  usage_id: number | null;
  floor_type: string;
  floor_level: string;
  seats: { sitting: number | null; standing: number | null; wheelchair: number | null };
  room_links: LinkDraft[];
  // building-only
  name: string;
  short_name: string;
  node_kind: BuildingKind | null;
  building_prefixes: string[];
  internal_id: string;
  visible_id: string;
  // poi-only
  usage_name: string;
  comment_de: string;
  comment_en: string;
  poi_links: LinkDraft[];
  generic_props: GenericPropDraft[];
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
  addOpen: boolean;
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
  pendingAddition: AdditionDraft;
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

function emptyAdditionDraft(): AdditionDraft {
  return {
    kind: null,
    id: "",
    parent_id: "",
    parent_name: "",
    coords: { lat: 0, lon: 0, picked: false },
    alt_name: "",
    arch_name: "",
    usage_id: null,
    floor_type: "",
    floor_level: "",
    seats: { sitting: null, standing: null, wheelchair: null },
    room_links: [],
    name: "",
    short_name: "",
    node_kind: null,
    building_prefixes: [],
    internal_id: "",
    visible_id: "",
    usage_name: "",
    comment_de: "",
    comment_en: "",
    poi_links: [],
    generic_props: [],
  };
}

export const useEditProposal = () =>
  useState<EditProposalState>("editProposal", () => ({
    open: false,
    addOpen: false,
    selected: {
      id: null,
      name: null,
    },
    data: {
      additional_context: "",
      edits: {},
      additions: {},
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
    pendingAddition: emptyAdditionDraft(),
  }));

function emptyRoomEdit() {
  return { coordinate: null, image: null, properties: null };
}

export type { AdditionDraft, AdditionKind, GenericPropDraft, LinkDraft, PropertyFields };
export { emptyAdditionDraft, emptyPropertyFields, emptyRoomEdit };
