import type { LocationQueryValue } from "#vue-router";

export function firstOrDefault(
  value: LocationQueryValue | LocationQueryValue[] | undefined,
  defaultValue: string
): string {
  if (Array.isArray(value)) return value[0] ?? defaultValue;
  return value ?? defaultValue;
}
