import type { components } from "~/api_types";
import { SELECTABLE_LEVELS } from "~/composables/mapLayers";

type MotisLegResponse = components["schemas"]["MotisLegResponse"];
type StepInstructionResponse = components["schemas"]["StepInstructionResponse"];

/** All OSM levels a leg touches: its endpoints plus every step transition. */
function legLevels(leg: MotisLegResponse): number[] {
  const levels = [leg.from.level, leg.to.level];
  for (const step of leg.steps ?? []) {
    levels.push(step.from_level, step.to_level);
  }
  return levels;
}

/**
 * The floor the indoor map should switch to when the user selects a leg or one of
 * its steps, or `null` when the selection should leave the floor selector untouched.
 *
 * Motis reports level 0 for plain outdoor geometry too, so a leg only counts as
 * level-aware when some part of it leaves the ground level. Levels the floor
 * selector cannot represent (half-levels, deep basements) also yield `null`.
 */
export function floorLevelForSelection(
  leg: MotisLegResponse,
  step?: StepInstructionResponse
): number | null {
  if (legLevels(leg).every((touched) => touched === 0)) return null;
  const level = step ? step.from_level : leg.from.level;
  return SELECTABLE_LEVELS.includes(level) ? level : null;
}
