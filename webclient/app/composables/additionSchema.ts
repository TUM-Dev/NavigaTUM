// Zod schemas for the "propose a new entry" forms. These intentionally mirror the Rust validators
// in `server/src/routes/feedback/proposed_edits/addition/{room,building,poi}.rs`. Whenever the
// backend rules change, update both - the matching `case` in each Rust validator's rstest table
// is the source of truth.
import { z } from "zod";
import type { AdditionDraft } from "~/composables/editProposal";

// Shared with `MAX_NAME_LEN` in the backend (room.rs / building.rs / poi.rs).
const MAX_NAME_LEN = 200;
// Shared with `MAX_KEY_LEN` in poi.rs.
const MAX_POI_KEY_LEN = 64;

// `is_allowed_roomcode_char` in room.rs (mirrored from `ALLOWED_ROOMCODE_CHARS` in
// `data/processors/tumonline.py`).
const ROOM_KEY_RE = /^[A-Za-z0-9.-]+$/;

// `is_arch_name_valid` in room.rs.
const ARCH_NAME_RE = /^[A-Za-z0-9._-]+@\d{4}$/;

// `is_valid_poi_key` in poi.rs: first char ascii-lowercase or digit; rest [a-z0-9_-].
const POI_KEY_RE = /^[a-z0-9][a-z0-9_-]*$/;

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

export type AdditionFieldErrors = Partial<Record<string, string>>;

export function validateAddition(draft: AdditionDraft): AdditionFieldErrors {
  if (!draft.kind) return {};
  const schema =
    draft.kind === "room"
      ? newRoomSchema
      : draft.kind === "building"
        ? newBuildingSchema
        : newPoiSchema;
  const result = schema.safeParse(draft);
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
  const schema =
    draft.kind === "room"
      ? newRoomSchema
      : draft.kind === "building"
        ? newBuildingSchema
        : newPoiSchema;
  return schema.safeParse(draft).success;
}
