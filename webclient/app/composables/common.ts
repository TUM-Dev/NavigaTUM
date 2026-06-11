import type { LocationQueryValue } from "#vue-router";

export function firstOrDefault(
  value: LocationQueryValue | LocationQueryValue[] | undefined,
  defaultValue: string
): string {
  if (Array.isArray(value)) return value[0] ?? defaultValue;
  return value ?? defaultValue;
}

export function allValues(value: LocationQueryValue | LocationQueryValue[]): string[] {
  if (!value) return [];
  if (Array.isArray(value)) return value.filter((v): v is string => v !== null);
  return [value];
}

// During SSR, retrying a 5xx would stall the whole page render.
export function clientOnlyRetries(retries: number): number | false {
  return import.meta.server ? false : retries;
}
