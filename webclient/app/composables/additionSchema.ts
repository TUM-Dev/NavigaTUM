// Mirrors the Rust validators in `server/src/routes/feedback/proposed_edits/addition/`.
// Adding a kind means appending one variant block and one `additionRegistry` row.

import type { DeepWritable } from "ts-essentials";
import { z } from "zod";
import type { components } from "~/api_types";
import { rfc3339ToWallTime, wallTimeToRfc3339 } from "~/utils/datetime";

type BuildingKind = components["schemas"]["BuildingKind"];
type EventEntry = components["schemas"]["EventEntry"];
// openapi-typescript marks everything readonly, but the builders below produce fresh literals destined for the writable EditRequest map.
export type Addition = DeepWritable<
  components["schemas"]["LimitedHashMap_String_Addition"][string]
>;

const MAX_NAME_LEN = 200;
const MAX_POI_KEY_LEN = 64;
const MIN_IMAGE_DIM = 256;
const MAX_HORIZON_DAYS = 365;
const MAX_DURATION_DAYS = 30;
const DAY_MS = 24 * 60 * 60 * 1000;

const ROOM_KEY_RE = /^[A-Za-z0-9.-]+$/;
const ARCH_NAME_RE = /^[A-Za-z0-9._-]+@\d{4}$/;
const POI_KEY_RE = /^[a-z0-9][a-z0-9_-]*$/;
const EVENT_KEY_RE = /^event_[0-9a-f]{1,64}$/;
const BUILDING_PREFIX_RE = /^\d{4}$/;

const coordsSchema = z.object({
  lat: z.number(),
  lon: z.number(),
  picked: z.literal(true, { message: "error.coords_required" }),
});

export const roomKeySchema = z
  .string()
  .min(1, "error.id_required")
  .refine((k) => k.split(".").length === 3, "error.id_room_incomplete")
  .refine((k) => k.split(".").every((p) => p.length > 0), "error.id_room_incomplete")
  .refine((k) => ROOM_KEY_RE.test(k), "error.id_room_format");

export const archNameSchema = z
  .string()
  .min(1, "error.arch_name_required")
  .refine((s) => ARCH_NAME_RE.test(s), "error.arch_name_format");

export const poiKeySchema = z
  .string()
  .min(1, "error.id_required")
  .max(MAX_POI_KEY_LEN, "error.poi_key_too_long")
  .refine((k) => POI_KEY_RE.test(k), "error.poi_key_format");

export const eventKeySchema = z
  .string()
  .min(1, "error.id_required")
  .refine((k) => EVENT_KEY_RE.test(k), "error.event_key_format");

export const buildingPrefixSchema = z
  .string()
  .refine((p) => BUILDING_PREFIX_RE.test(p), "error.building_prefix_format");

export interface LinkDraft {
  text_de: string;
  text_en: string;
  url: string;
}

export interface GenericPropDraft {
  name_de: string;
  name_en: string;
  text: string;
}

export interface CoordsDraft {
  lat: number;
  lon: number;
  picked: boolean;
}

// `id` and `coords` live on every variant so the modal can read them without narrowing.
interface DraftBase {
  id: string;
  coords: CoordsDraft;
}

function freshCoords(): CoordsDraft {
  return { lat: 0, lon: 0, picked: false };
}

export interface NoKindDraft extends DraftBase {
  kind: null;
}

export interface RoomDraft extends DraftBase {
  kind: "room";
  parent_id: string;
  parent_name: string;
  alt_name: string;
  arch_name: string;
  usage_id: number | null;
  floor_type: string;
  floor_level: string;
  seats: { sitting: number | null; standing: number | null; wheelchair: number | null };
  room_links: LinkDraft[];
}

function emptyRoom(): RoomDraft {
  return {
    kind: "room",
    id: "",
    parent_id: "",
    parent_name: "",
    coords: freshCoords(),
    alt_name: "",
    arch_name: "",
    usage_id: null,
    floor_type: "",
    floor_level: "",
    seats: { sitting: null, standing: null, wheelchair: null },
    room_links: [],
  };
}

const roomSchema = z.object({
  kind: z.literal("room"),
  id: roomKeySchema,
  parent_id: z.string().min(1, "error.parent_required"),
  alt_name: z.string().min(1, "error.name_required").max(MAX_NAME_LEN, "error.name_too_long"),
  arch_name: archNameSchema,
  usage_id: z.number({ message: "error.usage_required" }).int().nonnegative(),
  coords: coordsSchema,
});

function buildRoom(draft: RoomDraft): Addition {
  const seats =
    draft.seats.sitting !== null || draft.seats.standing !== null || draft.seats.wheelchair !== null
      ? {
          sitting: draft.seats.sitting,
          standing: draft.seats.standing,
          wheelchair: draft.seats.wheelchair,
        }
      : null;
  const links = draft.room_links.filter((l) => l.url.trim());
  // Schema guarantees `usage_id`; cast keeps the build signature non-nullable.
  return {
    kind: "room",
    parent_building_id: draft.parent_id,
    alt_name: draft.alt_name,
    arch_name: draft.arch_name,
    usage_id: draft.usage_id as number,
    coords: { lat: draft.coords.lat, lon: draft.coords.lon },
    seats,
    floor_type: draft.floor_type || null,
    floor_level: draft.floor_level || null,
    // Address is inherited from the parent building on the server.
    address: null,
    links: links.length > 0 ? links : undefined,
  } as Addition;
}

export interface BuildingDraft extends DraftBase {
  kind: "building";
  parent_id: string;
  parent_name: string;
  name: string;
  short_name: string;
  node_kind: BuildingKind | null;
  building_prefixes: string[];
  internal_id: string;
  visible_id: string;
}

function emptyBuilding(): BuildingDraft {
  return {
    kind: "building",
    id: "",
    parent_id: "",
    parent_name: "",
    coords: freshCoords(),
    name: "",
    short_name: "",
    node_kind: null,
    building_prefixes: [],
    internal_id: "",
    visible_id: "",
  };
}

const buildingSchema = z
  .object({
    kind: z.literal("building"),
    id: z.string().min(1, "error.id_required"),
    parent_id: z.string().min(1, "error.parent_required"),
    name: z.string().min(1, "error.name_required").max(MAX_NAME_LEN, "error.name_too_long"),
    node_kind: z.enum(["building", "joined_building", "area"], {
      message: "error.node_kind_required",
    }),
    building_prefixes: z.array(buildingPrefixSchema),
    coords: coordsSchema,
  })
  .superRefine((draft, ctx) => {
    const len = draft.building_prefixes.length;
    if (draft.node_kind === "building" && len !== 1) {
      ctx.addIssue({
        code: "custom",
        path: ["building_prefixes"],
        message: "error.building_needs_one_prefix",
      });
    }
    if (draft.node_kind === "joined_building" && len < 2) {
      ctx.addIssue({
        code: "custom",
        path: ["building_prefixes"],
        message: "error.joined_building_needs_multi_prefix",
      });
    }
  });

function buildBuilding(draft: BuildingDraft): Addition {
  // The schema guarantees `node_kind` is set.
  return {
    kind: "building",
    parent_id: draft.parent_id,
    name: draft.name,
    short_name: draft.short_name || null,
    node_kind: draft.node_kind as BuildingKind,
    building_prefixes: [...draft.building_prefixes],
    internal_id: draft.internal_id || null,
    visible_id: draft.visible_id || null,
    coords: { lat: draft.coords.lat, lon: draft.coords.lon },
  } as Addition;
}

export interface PoiDraft extends DraftBase {
  kind: "poi";
  parent_id: string;
  parent_name: string;
  name: string;
  usage_name: string;
  comment_de: string;
  comment_en: string;
  poi_links: LinkDraft[];
  generic_props: GenericPropDraft[];
}

function emptyPoi(): PoiDraft {
  return {
    kind: "poi",
    id: "",
    parent_id: "",
    parent_name: "",
    coords: freshCoords(),
    name: "",
    usage_name: "",
    comment_de: "",
    comment_en: "",
    poi_links: [],
    generic_props: [],
  };
}

const poiSchema = z.object({
  kind: z.literal("poi"),
  id: poiKeySchema,
  parent_id: z.string().min(1, "error.parent_required"),
  name: z.string().min(1, "error.name_required").max(MAX_NAME_LEN, "error.name_too_long"),
  usage_name: z.string().min(1, "error.usage_name_required"),
  coords: coordsSchema,
});

function buildPoi(draft: PoiDraft): Addition {
  const links = draft.poi_links
    .filter((l) => l.url.trim())
    .map((l) => ({ url: l.url, text: { de: l.text_de, en: l.text_en } }));
  const generic_props = draft.generic_props
    .filter((p) => p.name_de.trim() || p.name_en.trim() || p.text.trim())
    .map((p) => ({ name: { de: p.name_de, en: p.name_en }, text: p.text }));
  const comment =
    draft.comment_de.trim() || draft.comment_en.trim()
      ? { de: draft.comment_de, en: draft.comment_en }
      : null;
  return {
    kind: "poi",
    parent: draft.parent_id,
    name: draft.name,
    usage_name: draft.usage_name,
    coords: { lat: draft.coords.lat, lon: draft.coords.lon },
    comment,
    links: links.length > 0 ? links : undefined,
    generic_props: generic_props.length > 0 ? generic_props : undefined,
  } as Addition;
}

// The search hit a locked draft is based on: its key is the upsert identity,
// and its name and last-held dates feed the banner.
export interface EventBasedOn {
  id: string;
  name: string;
  starts_at: string;
  ends_at: string;
}

export interface EventDraft extends DraftBase {
  kind: "event";
  name: string;
  description: string;
  starts_at: string;
  ends_at: string;
  organising_org_id: number | null;
  image: { base64: string; fileName: string } | null;
  image_width: number | null;
  image_height: number | null;
  image_thumb_offset: number;
  image_header_offset: number;
  image_author: string;
  based_on: EventBasedOn | null;
}

function emptyEvent(): EventDraft {
  return {
    kind: "event",
    id: "",
    coords: freshCoords(),
    name: "",
    description: "",
    starts_at: "",
    ends_at: "",
    organising_org_id: null,
    image: null,
    image_width: null,
    image_height: null,
    image_thumb_offset: 0,
    image_header_offset: 0,
    image_author: "",
    based_on: null,
  };
}

// A fresh locked draft from a picked search hit; the image is fetched separately
// and rides in through the regular upload path.
export function eventDraftFromEntry(entry: EventEntry, now: number): EventDraft {
  const draft = emptyEvent();
  draft.id = entry.id;
  draft.based_on = {
    id: entry.id,
    name: entry.name,
    starts_at: entry.starts_at,
    ends_at: entry.ends_at,
  };
  draft.name = entry.name;
  draft.description = entry.description;
  draft.organising_org_id = entry.organising_org_id;
  draft.coords = { lat: entry.lat, lon: entry.lon, picked: true };
  draft.image_author = entry.image_author;
  // Past dates fail the server's EventEnded validation; only a still-running edition pre-fills them.
  if (Date.parse(entry.ends_at) > now) {
    draft.starts_at = rfc3339ToWallTime(entry.starts_at) ?? "";
    draft.ends_at = rfc3339ToWallTime(entry.ends_at) ?? "";
  }
  return draft;
}

const eventSchema = z
  .object({
    kind: z.literal("event"),
    id: eventKeySchema,
    name: z
      .string()
      .max(MAX_NAME_LEN, "error.name_too_long")
      .refine((s) => s.trim().length > 0, "error.name_required"),
    description: z.string().refine((s) => s.trim().length > 0, "error.description_required"),
    coords: coordsSchema,
    organising_org_id: z.number({ message: "error.org_required" }).int().positive(),
    image: z.object({ base64: z.string() }).nullable(),
    starts_at: z.string(),
    ends_at: z.string(),
    image_width: z.number().nullable(),
    image_height: z.number().nullable(),
    image_author: z.string(),
  })
  .superRefine((draft, ctx) => {
    const startRfc = wallTimeToRfc3339(draft.starts_at);
    const endRfc = wallTimeToRfc3339(draft.ends_at);
    if (startRfc === null) {
      ctx.addIssue({ code: "custom", path: ["starts_at"], message: "error.starts_at_required" });
    }
    if (endRfc === null) {
      ctx.addIssue({ code: "custom", path: ["ends_at"], message: "error.ends_at_required" });
    }
    if (startRfc !== null && endRfc !== null) {
      const start = Date.parse(startRfc);
      const end = Date.parse(endRfc);
      const now = Date.now();
      if (end < start) {
        ctx.addIssue({
          code: "custom",
          path: ["ends_at"],
          message: "error.event_ends_before_start",
        });
      } else if (end <= now) {
        ctx.addIssue({ code: "custom", path: ["ends_at"], message: "error.event_ended" });
      }
      if (start > now + MAX_HORIZON_DAYS * DAY_MS) {
        ctx.addIssue({ code: "custom", path: ["starts_at"], message: "error.event_too_far_out" });
      }
      if (end - start > MAX_DURATION_DAYS * DAY_MS) {
        ctx.addIssue({ code: "custom", path: ["ends_at"], message: "error.event_too_long" });
      }
    }
    if (!draft.image?.base64) {
      ctx.addIssue({ code: "custom", path: ["image"], message: "error.image_required" });
    } else if (
      draft.image_width !== null &&
      draft.image_height !== null &&
      Math.min(draft.image_width, draft.image_height) < MIN_IMAGE_DIM
    ) {
      ctx.addIssue({ code: "custom", path: ["image"], message: "error.image_too_small" });
    }
    if (!draft.image_author.trim()) {
      ctx.addIssue({
        code: "custom",
        path: ["image_author"],
        message: "error.image_author_required",
      });
    }
  });

function buildEvent(draft: EventDraft): Addition {
  // The schema guarantees `image` and `organising_org_id` are set.
  const image = draft.image as { base64: string; fileName: string };
  return {
    kind: "event",
    name: draft.name,
    description: draft.description,
    starts_at: wallTimeToRfc3339(draft.starts_at) ?? "",
    ends_at: wallTimeToRfc3339(draft.ends_at) ?? "",
    coords: { lat: draft.coords.lat, lon: draft.coords.lon },
    organising_org_id: draft.organising_org_id as number,
    image: {
      content: image.base64,
      metadata: {
        author: draft.image_author,
        license: { text: "CC BY 4.0", url: "https://creativecommons.org/licenses/by/4.0/" },
        offsets:
          draft.image_thumb_offset === 0 && draft.image_header_offset === 0
            ? null
            : { thumb: draft.image_thumb_offset, header: draft.image_header_offset },
      },
    },
  } as Addition;
}

export type AdditionKind = "room" | "building" | "poi" | "event";
export type AdditionDraft = NoKindDraft | RoomDraft | BuildingDraft | PoiDraft | EventDraft;

interface AdditionRegistryEntry<K extends AdditionKind> {
  empty(): Extract<AdditionDraft, { kind: K }>;
  schema: z.ZodTypeAny;
  build(draft: Extract<AdditionDraft, { kind: K }>): Addition;
}

export const additionRegistry: { readonly [K in AdditionKind]: AdditionRegistryEntry<K> } = {
  room: { empty: emptyRoom, schema: roomSchema, build: buildRoom },
  building: { empty: emptyBuilding, schema: buildingSchema, build: buildBuilding },
  poi: { empty: emptyPoi, schema: poiSchema, build: buildPoi },
  event: { empty: emptyEvent, schema: eventSchema, build: buildEvent },
};

export function emptyAdditionDraft(): NoKindDraft {
  return { kind: null, id: "", coords: freshCoords() };
}

export type AdditionFieldErrors = Partial<Record<string, string>>;

export function validateAddition(draft: AdditionDraft): AdditionFieldErrors {
  if (draft.kind === null) return {};
  const result = additionRegistry[draft.kind].schema.safeParse(draft);
  if (result.success) return {};
  const errors: AdditionFieldErrors = {};
  for (const issue of result.error.issues) {
    const path = issue.path.join(".") || "_";
    if (!errors[path]) errors[path] = issue.message;
  }
  return errors;
}

export function isAdditionValid(draft: AdditionDraft): boolean {
  if (draft.kind === null) return false;
  return additionRegistry[draft.kind].schema.safeParse(draft).success;
}

export function buildAddition(draft: AdditionDraft): Addition | null {
  if (draft.kind === null) return null;
  // TS can't narrow `draft` against the registry entry by the shared kind discriminant.
  return (additionRegistry[draft.kind].build as (d: AdditionDraft) => Addition)(draft);
}
