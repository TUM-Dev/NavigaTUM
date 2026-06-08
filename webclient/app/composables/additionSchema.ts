// Mirrors the Rust validators in `server/src/routes/feedback/proposed_edits/addition/`.
import { z } from "zod";
import type { AdditionDraft } from "~/composables/editProposal";
import { wallTimeToRfc3339 } from "~/utils/datetime";

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

export const buildingPrefixSchema = z
  .string()
  .refine((p) => BUILDING_PREFIX_RE.test(p), "error.building_prefix_format");

const newRoomSchema = z.object({
  kind: z.literal("room"),
  id: roomKeySchema,
  parent_id: z.string().min(1, "error.parent_required"),
  alt_name: z.string().min(1, "error.name_required").max(MAX_NAME_LEN, "error.name_too_long"),
  arch_name: archNameSchema,
  usage_id: z.number({ message: "error.usage_required" }).int().nonnegative(),
  coords: coordsSchema,
});

const newBuildingSchema = z
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

const newPoiSchema = z.object({
  kind: z.literal("poi"),
  id: poiKeySchema,
  parent_id: z.string().min(1, "error.parent_required"),
  name: z.string().min(1, "error.name_required").max(MAX_NAME_LEN, "error.name_too_long"),
  usage_name: z.string().min(1, "error.usage_name_required"),
  coords: coordsSchema,
});

export const eventKeySchema = z
  .string()
  .min(1, "error.id_required")
  .refine((k) => EVENT_KEY_RE.test(k), "error.event_key_format");

const newEventSchema = z
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

export type AdditionFieldErrors = Partial<Record<string, string>>;

function schemaForKind(kind: NonNullable<AdditionDraft["kind"]>) {
  switch (kind) {
    case "room":
      return newRoomSchema;
    case "building":
      return newBuildingSchema;
    case "poi":
      return newPoiSchema;
    case "event":
      return newEventSchema;
  }
}

export function validateAddition(draft: AdditionDraft): AdditionFieldErrors {
  if (!draft.kind) return {};
  const result = schemaForKind(draft.kind).safeParse(draft);
  if (result.success) return {};
  const errors: AdditionFieldErrors = {};
  for (const issue of result.error.issues) {
    const path = issue.path.join(".") || "_";
    if (!errors[path]) errors[path] = issue.message;
  }
  return errors;
}

export function isAdditionValid(draft: AdditionDraft): boolean {
  if (!draft.kind) return false;
  return schemaForKind(draft.kind).safeParse(draft).success;
}
