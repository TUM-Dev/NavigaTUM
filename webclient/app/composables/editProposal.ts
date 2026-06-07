import type { DeepWritable } from "ts-essentials";
import type { components } from "~/api_types";

type EditRequest = components["schemas"]["EditRequest"];
type EditRequestData = Omit<EditRequest, "privacy_checked" | "token" | "edits" | "additions"> & {
  edits: NonNullable<EditRequest["edits"]>;
  additions: NonNullable<EditRequest["additions"]>;
};
type BuildingKind = components["schemas"]["BuildingKind"];
type AdditionKind = "room" | "building" | "poi" | "event";

interface LinkDraft {
  text_de: string;
  text_en: string;
  url: string;
}
interface GenericPropDraft {
  name_de: string;
  name_en: string;
  text: string;
}

interface AdditionDraft {
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
  // event-only. `name` (shared) carries the event title; `description` is single-language and
  // rendered verbatim. `starts_at`/`ends_at` hold the `datetime-local` wall value (Europe/Berlin),
  // converted to RFC3339 only at submit time. The image rides inline in the addition (unlike the
  // separate image edit used for room/building/poi), so its bytes + author/license live here, and
  // `image_width`/`image_height` back the client-side minimum-dimension check.
  description: string;
  starts_at: string;
  ends_at: string;
  organising_org_id: number | null;
  image: { base64: string; fileName: string } | null;
  image_width: number | null;
  image_height: number | null;
  image_author: string;
  image_license_text: string;
  image_license_url: string;
}

interface PropertyFields {
  name: string;
  shortName: string;
  categoryDe: string;
  categoryEn: string;
  categoryDin277: string;
  categoryDin277Desc: string;
  linkUrl: string;
  linkTextDe: string;
  linkTextEn: string;
}
interface EditProposalState {
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
}

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
    description: "",
    starts_at: "",
    ends_at: "",
    organising_org_id: null,
    image: null,
    image_width: null,
    image_height: null,
    image_author: "",
    image_license_text: "CC BY 4.0",
    image_license_url: "https://creativecommons.org/licenses/by/4.0/",
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
